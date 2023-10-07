use std::time::Duration;
extern crate libusb1_sys as usbffi;

use rusb::{
    DeviceHandle, Direction, Recipient, RequestType, UsbContext,
};

pub(crate) fn send_firmware<T: UsbContext>(handle: &mut DeviceHandle<T>, firmware: Vec<u8>) {
    let timeout = Duration::from_secs(1);

    let request_type = rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Device);

    handle.set_active_configuration(1);

    handle.claim_interface(0);

    // Reset Device
    match handle.write_control(request_type, 0xa0, 0xe600, 0, &[0x01], timeout) {
        Ok(_) => println!("Successfully reset the FX2 chip for programming"),
        Err(_) => todo!(),
    };

    // Send Firmware bits (can be changed for config??)
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

    // Send more Firmware
    let mut rom_address = 0x0080;
    let chunks = firmware.chunks(1023);
    for chunk in chunks {
        println!("Address {:x}", rom_address);
        match handle.write_control(request_type, 0xa0, rom_address, 0, chunk, timeout) {
            Ok(_) => print!("."),
            Err(_) => todo!(),
        };
        rom_address += chunk.len() as u16;
    }

    println!("\nFX2 Firmware Flashed");

    // Reset Again
    match handle.write_control(request_type, 0xa0, 0xe600, 0, &[0x00], timeout) {
        Ok(_) => println!("Successfully reset the FX2 chip for re-enumeration"),
        Err(_) => todo!(),
    };
}