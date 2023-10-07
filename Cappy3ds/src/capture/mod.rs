use bytes::{buf, Buf, BufMut, BytesMut};
use memchr::memmem;
use rust_embed::RustEmbed;
use std::ffi::c_void;
use std::io::Write;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{ffi, fs, path, slice};
use std::{thread, time};
extern crate libusb1_sys as usbffi;

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Recipient, RequestType, Result,
    TransferType, UsbContext,
};

mod fpga;
mod fx2;
mod image;

#[derive(RustEmbed)]
#[folder = "resources/Katsukity/"]
struct Katsukity;

pub fn do_capture() {
    let firmware = Katsukity::get("firm.bin").unwrap();
    let bitstream = Katsukity::get("bitstream.bin").unwrap();

    let vid = 0x0752;
    let pid = 0x8613;

    match Context::new() {
        Ok(mut context) => {
            let mut flashed_fx2 = false;
            match open_device(&mut context, vid, pid) {
                Some((mut device, device_desc, mut handle)) => {
                    println!("Opened {:04x}:{:04x}", vid, pid);
                    fx2::send_firmware(&mut handle, firmware.data.to_vec());
                    flashed_fx2 = true;
                }
                None => println!("could not find FX2 device {:04x}:{:04x}", vid, pid),
            }

            if flashed_fx2 {
                println!("Waiting for second interface");
                let sleep_time = time::Duration::from_millis(5000);
                thread::sleep(sleep_time);
                // todo: loop with device check instead of sleeping
            }

            match open_device(&mut context, 0x0752, 0xf2c0) {
                Some((mut device, device_desc, mut handle)) => {
                    println!("Opened secondary device");
                    match handle.claim_interface(0) {
                        Ok(_) => {}
                        Err(err) => panic!("could not claim second device: {}", err),
                    }

                    if fpga::check_fpga_programmed(&mut handle) {
                    } else {
                        fpga::read_eeprom(&mut handle);
                        fpga::configure_fpga(&mut handle, bitstream.data.to_vec());
                        fpga::configure_port(&mut handle);
                    }
                    fpga::fifo_start(&mut handle);

                    bulk_read(&mut handle);
                }
                None => println!("secondary device missing, firmware upload failed?"),
            }
        }
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => panic!("Device found but failed to open: {}", e),
            }
        }
    }

    None
}

extern "system" fn transfer_finished<T: UsbContext>(transfer_ptr: *mut usbffi::libusb_transfer) {
    let transfer: &mut usbffi::libusb_transfer = unsafe { &mut *transfer_ptr };

    let user_data = transfer.user_data;

    if user_data.is_null() {
        // todo: some sort of cleanup????
        return;
    }

    let buf = transfer.buffer;

    let s = unsafe { slice::from_raw_parts(buf, 0x4000) };
    let mut ss = [0; 0x4000];
    ss.copy_from_slice(s);

    let bulk_transfer = unsafe { user_data as *mut BulkTransfer };

    unsafe {
        let mut handler = (*bulk_transfer).capture_handler.lock().unwrap();

        if handler.buffers[handler.current_buffer].len() > 0x4000 {
            handler.current_buffer += 1;
        }

        if handler.current_buffer == NUM_BUFFERS {
            // return;
        }

        let current_buffer = handler.current_buffer;

        handler.buffers[current_buffer].extend_from_slice(&ss);

        if !handler.should_stop.load(Ordering::Relaxed) {
            usbffi::libusb_submit_transfer(transfer_ptr);
        }
    }
}

struct CaptureHandler {
    buffers: Vec<Vec<u8>>,
    current_buffer: usize,
    should_stop: AtomicBool,
}

const NUM_BUFFERS: usize = 20;

impl CaptureHandler {
    fn new() -> Self {
        let mut buffers = Vec::<Vec<u8>>::with_capacity(NUM_BUFFERS);

        for i in 0..NUM_BUFFERS {
            buffers.push(Vec::<u8>::with_capacity(0x4000 * 2));
        }

        Self {
            current_buffer: 0,
            buffers,
            should_stop: AtomicBool::new(false),
        }
    }
}

struct BulkTransfer {
    //lib_usb_transfer: *mut usbffi::libusb_transfer,
    capture_handler: Arc<Mutex<CaptureHandler>>,
}

impl BulkTransfer {
    fn new(capture_handler: Arc<Mutex<CaptureHandler>>) -> Self {
        /*  let transfer;
        unsafe {
           // transfer = usbffi::libusb_alloc_transfer(0);
        }*/

        Self { capture_handler }
    }
}

fn bulk_read<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    println!("Starting Bulk Read");

    let capture_handler = Arc::new(Mutex::new(CaptureHandler::new()));

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
            let mut in_buf = [0u8; 0x4000];
            let transfer = Box::new(BulkTransfer::new(capture_handler.clone()));

            let raw_transfer = Box::into_raw(transfer) as *mut c_void;

            //transfers.push(transfer);

            let lib_usb_transfer = unsafe { usbffi::libusb_alloc_transfer(0) };

            // let user_data = Arc::into_raw(transfer.clone()) as *mut c_void;

            unsafe {
                usbffi::libusb_fill_bulk_transfer(
                    lib_usb_transfer,
                    handle.as_raw(),
                    0x82,
                    in_buf.as_mut_ptr(),
                    0x4000,
                    transfer_finished::<T> as _,
                    raw_transfer,
                    1000,
                );

                usbffi::libusb_submit_transfer(lib_usb_transfer);
            }
        }

        loop {
            let current_buffer = capture_handler.lock().unwrap().current_buffer;
            if current_buffer >= NUM_BUFFERS - 10 {
                break;
            }
            //let ten_millis = time::Duration::from_millis(1);
            //thread::sleep(ten_millis);
        }

        println!("Stopping Capture");

        capture_handler.lock().unwrap().should_stop.store(true, Ordering::Relaxed);

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

    // 33CC 23C0 1800 0000 0000 1900 0000 0000 (then 240 pixels of image)
    // 33CC 24C0 1900 0000 0000 1A00 0000 0000 (240)
    // 33CC 25C0 1A00 0000 0000 1B00 0000 0000
    // 33CC 26C0 1C00 0000 0000 1D00 0000 0000
    // ...
    // 33CC 2EC1 0701 0000 0000 0801 0000 0000
}
