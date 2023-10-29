use memchr::memmem;
use rust_embed::RustEmbed;
use std::ffi::c_void;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{thread, time};
extern crate libusb1_sys as usbffi;
use bytes::BytesMut;
use simple_error::SimpleError;



use rusb::{Device, DeviceDescriptor, DeviceHandle, UsbContext};

use super::Capture;

mod fpga;
mod fx2;
mod image;
mod parse;

#[derive(RustEmbed)]
#[folder = "resources/Katsukity/"]
struct KatsukityResources;

pub struct Katsukity {

}

impl Katsukity {
    pub fn new() -> Self {
        return Self{

        };
    }
}

impl Capture for Katsukity {
    fn connect<T: UsbContext>(context: &mut T) -> Result<DeviceHandle<T>, SimpleError> {
        let firmware = KatsukityResources::get("firm.bin").unwrap();
        let bitstream = KatsukityResources::get("bitstream.bin").unwrap();
    
        let vid = 0x0752;
        let pid = 0x8613;
    
        let mut flashed_fx2 = false;
        match Self::open_device(context, vid, pid) {
            Some((mut device, device_desc, mut handle)) => {
                println!("Opened {:04x}:{:04x}", vid, pid);
                fx2::send_firmware(&mut handle, firmware.data.to_vec());
                flashed_fx2 = true;
            }
            None => {
                println!("could not find FX2 device");
            }
        }
    
        if flashed_fx2 {
            println!("Waiting for second interface");
            let sleep_time = time::Duration::from_millis(5000);
            thread::sleep(sleep_time);
            // todo: loop with device check instead of sleeping
        }
    
        match Self::open_device(context, 0x0752, 0xf2c0) {
            Some((mut device, device_desc, mut handle)) => {
                println!("Opened secondary device");
                match handle.claim_interface(0) {
                    Ok(_) => {}
                    Err(err) => panic!("could not claim second device: {}", err),
                }
    
                // bleh apparently relesase runs fast enough to break this
                // add in some sleeps
                //if fpga::check_fpga_programmed(&mut handle) {
                //} else {
                fpga::read_eeprom(&mut handle);
                fpga::configure_fpga(&mut handle, bitstream.data.to_vec());
                fpga::configure_port(&mut handle);
                //}
    
                fpga::fifo_start(&mut handle);
    
                Ok(handle)
            }
            None => Err(SimpleError::new(
                "secondary device missing, firmware upload failed?",
            )),
        }
    }
}


pub fn do_capture<T: UsbContext, F>(handle: &mut DeviceHandle<T>, data_callback: F)
where
    F: FnMut(&[i16], BytesMut, BytesMut),
{
    bulk_read(handle, data_callback);
}

extern "system" fn transfer_finished<T: UsbContext, F>(transfer_ptr: *mut usbffi::libusb_transfer)
where
    F: FnMut(&[i16], BytesMut, BytesMut),
{
    let transfer: &mut usbffi::libusb_transfer = unsafe { &mut *transfer_ptr };

    let user_data = transfer.user_data;

    if user_data.is_null() {
        // todo: some sort of cleanup????
        return;
    }

    let buf = transfer.buffer;

    let s = unsafe { slice::from_raw_parts(buf, TRANSFER_SIZE) };

    let start = memmem::find(s, &[0x33, 0xCC, 0x00, 0x00]);

    let handler = user_data as *mut Arc<Mutex<CaptureHandler<F>>>;

    let mut handler = unsafe { (*handler).lock().unwrap() };

    let current_buffer = handler.current_buffer;
    let frame_buffer_len = handler.buffers[current_buffer].len();

    let mut local_offset = 0;
    let mut local_end = TRANSFER_SIZE;
    let mut wait = false;

    if let Some(start) = start {
        if frame_buffer_len == 0 {
            local_offset = start;
        } else {
            local_end = start;
        }
    } else if frame_buffer_len == 0 {
        // Wait for us to get the frame start code
        wait = true;
    }

    // if we don't need to wait for a new frame to start
    if !wait {
        // Move the data into the frame buffer
        handler.buffers[current_buffer].extend(&s[local_offset..local_end]);

        if let Some(start) = start {
            if frame_buffer_len > 0 {
                // parse since we have a frame
                let (upper_buffer, lower_buffer, sound_buffer) =
                    parse::split_capture_buffer(&handler.buffers[current_buffer]);

                let (_, short, _) = unsafe { sound_buffer.align_to::<i16>() };
                (handler.data_callback)(
                    short,
                    parse::rgb565_to_rgba(&upper_buffer),
                    parse::rgb565_to_rgba(&lower_buffer),
                );

                handler.current_buffer += 1;
                if handler.current_buffer >= NUM_BUFFERS {
                    handler.current_buffer = 0;
                }

                let current_buffer = handler.current_buffer;

                handler.buffers[current_buffer].clear();
                handler.buffers[current_buffer].extend(&s[start..]);
            }
        }
    }

    if !handler.should_stop.load(Ordering::Relaxed) {
        unsafe {
            usbffi::libusb_submit_transfer(transfer_ptr);
        }
    }
}

#[derive(Debug)]
struct CaptureHandler<F> {
    buffers: Vec<BytesMut>,
    current_buffer: usize,
    new_frame: bool,
    should_stop: AtomicBool,
    data_callback: F,
}

const NUM_BUFFERS: usize = 20;

// Top + Bottom width
// 400 + 320 = 720
// height + audio data
// 240 + 8
// RGB565
// 2 bytes per pixel
const FRAM_BUFFER_SIZE: usize = 720 * 248 * 2;

const TRANSFER_SIZE: usize = 0x4000;

impl<F> CaptureHandler<F>
where
    F: FnMut(&[i16], BytesMut, BytesMut),
{
    fn new(data_callback: F) -> Self {
        let mut buffers = Vec::<BytesMut>::with_capacity(NUM_BUFFERS);

        for i in 0..NUM_BUFFERS {
            buffers.push(BytesMut::with_capacity(FRAM_BUFFER_SIZE));
        }

        Self {
            new_frame: false,
            current_buffer: 0,
            buffers,
            should_stop: AtomicBool::new(false),
            data_callback,
        }
    }
}

fn bulk_read<T: UsbContext, F>(handle: &mut DeviceHandle<T>, data_callback: F)
where
    F: FnMut(&[i16], BytesMut, BytesMut),
{
    println!("Starting Bulk Read");

    let capture_handler = Arc::new(Mutex::new(CaptureHandler::new(data_callback)));

    let should_stop = Arc::new(AtomicBool::new(false));
    let stop_internal = Arc::clone(&should_stop);

    let timeout = libc::timeval {
        tv_sec: 1,
        tv_usec: 0,
    };

    thread::scope(|s| {
        let thread_join_handle = s.spawn(|| loop {
            if stop_internal.load(Ordering::Relaxed) {
                println!("GOT STOP");
                break;
            }
            unsafe {
                usbffi::libusb_handle_events_timeout(
                    handle.context().as_raw(),
                    &timeout as *const libc::timeval,
                );
            }
        });

        //let mut transfers = vec![];

        for _n in 0..10 {
            let mut in_buf = [0u8; TRANSFER_SIZE];
            let capture_handler = Box::new(capture_handler.clone());

            let raw_transfer = Box::into_raw(capture_handler) as *mut c_void;

            //transfers.push(transfer);

            let lib_usb_transfer = unsafe { usbffi::libusb_alloc_transfer(0) };

            // let user_data = Arc::into_raw(transfer.clone()) as *mut c_void;

            unsafe {
                usbffi::libusb_fill_bulk_transfer(
                    lib_usb_transfer,
                    handle.as_raw(),
                    0x82,
                    in_buf.as_mut_ptr(),
                    TRANSFER_SIZE.try_into().unwrap(),
                    transfer_finished::<T, F> as _,
                    raw_transfer,
                    1000,
                );

                usbffi::libusb_submit_transfer(lib_usb_transfer);
            }
        }

        loop {
            //let current_buffer = capture_handler.lock().unwrap().current_buffer;
            //if current_buffer >= NUM_BUFFERS - 10 {
            //break;
            //}
            //let ten_millis = time::Duration::from_millis(1);
            //thread::sleep(ten_millis);
        }

        println!("Stopping Capture");

        capture_handler
            .lock()
            .unwrap()
            .should_stop
            .store(true, Ordering::Relaxed);

        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        should_stop.store(true, Ordering::Relaxed);

        println!("Waiting on Stop");

        thread::sleep(ten_millis);

        match thread_join_handle.join() {
            Ok(_) => println!("Thread OK"),
            Err(e) => println!("Thread Err {:?}", e),
        }

        println!("PARSING IMAGE");

        let handler = &capture_handler.lock().unwrap();

        let mut image = Vec::<u8>::new();

        for buffer in &handler.buffers {
            image.extend_from_slice(&buffer);
        }

        image::parse_image_data(&image);

        // for transfer in transfers {
        //unsafe { usbffi::libusb_free_transfer(transfer.lib_usb_transfer); }
        // }
    });
}
