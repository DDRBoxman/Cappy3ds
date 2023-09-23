use std::time::Duration;
use std::fs;
use std::{thread, time};

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
    RequestType, Recipient
};

fn main() {
    println!("Hello, world!");

    let firmware = fs::read("firm.bin").expect("Can't load fx2 firmware");

    let vid = 0x0752;
    let pid = 0x8613;

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {
            Some((mut device, device_desc, mut handle)) => {
                println!("Opened {:04x}:{:04x}", vid, pid);
                send_firmware(&mut handle, firmware);
            }
            None => println!("could not find device {:04x}:{:04x}", vid, pid),
        },
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

fn send_firmware<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    firmware: Vec<u8>
) { 
    let timeout = Duration::from_secs(1);

    let request_type = rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
    
    handle.set_active_configuration(1);

    handle.claim_interface(0);

    // Reset Device
    match handle.write_control(request_type, 0xa0, 0xe600, 0, &[0x01], timeout) {
        Ok(_) => println!("Successfully reset the FX2 chip for programming"),
        Err(_) => todo!(),
    };

    // Send Firmware
    match handle.write_control(request_type, 0xa0, 0x0, 0, &[0x02, 0x09, 0x92], timeout) {
        Ok(_) => print!("."),
        Err(_) => todo!(),
    };

    match handle.write_control(request_type, 0xa0, 0x000b, 0, &[0x02, 0x0d, 0x9b], timeout) {
        Ok(_) => print!("."),
        Err(_) => todo!(),
    };

    match handle.write_control(request_type, 0xa0, 0x0033, 0, &[0x02, 0x0d, 0xe9], timeout) {
        Ok(_) => print!("."),
        Err(_) => todo!(),
    };

    match handle.write_control(request_type, 0xa0, 0x0043, 0, &[0x02, 0x08, 0x00], timeout) {
        Ok(_) => print!("."),
        Err(_) => todo!(),
    };

    match handle.write_control(request_type, 0xa0, 0x0053, 0, &[0x02, 0x08, 0x00], timeout) {
        Ok(_) => print!("."),
        Err(_) => todo!(),
    };

    println!("");

    // Send more Firmware
    let mut rom_address = 0x0080;
    let chunks = firmware.chunks(1023);
    for chunk in chunks {
        println!("Address {:x}", rom_address);
        match handle.write_control(request_type, 0xa0, rom_address, 0, chunk, timeout) {
            Ok(bytes) => println!("Uploaded {} bytes", bytes),
            Err(_) => todo!(),
        };
        rom_address += chunk.len() as u16;
    }

    // Reset Again
    match handle.write_control(request_type,0xa0, 0xe600, 0, &[0x00], timeout) {
        Ok(_) => println!("Successfully reset the FX2 chip for re-enumeration"),
        Err(_) => todo!(),
    };
}
