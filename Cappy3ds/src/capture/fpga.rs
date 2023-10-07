use std::time::Duration;
use std::{thread, time};
extern crate libusb1_sys as usbffi;

use rusb::{
    DeviceHandle, UsbContext,
};

pub fn read_eeprom<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    let mut offset = 0;

    let mut eeprom = Vec::<u8>::with_capacity(128);

    let mut buf = [0; 16];

    while offset <= 0x70 {
        match handle.write_bulk(1, &[0x38, offset, 0x10, 0x30], timeout) {
            Ok(_) => {}
            Err(err) => println!("could not write to endpoint: {}", err),
        }

        match handle.read_bulk(0x81, &mut buf, timeout) {
            Ok(len) => {
                //println!("{:02X?}", buf);
            }
            Err(err) => println!("could not read from endpoint: {}", err),
        }

        eeprom.extend_from_slice(&buf);

        offset += 0x10;
    }

    //println!("{:02X?}", eeprom);
}

pub fn configure_fpga<T: UsbContext>(handle: &mut DeviceHandle<T>, bitstream: Vec<u8>) {
    let timeout = Duration::from_secs(1);

    // ???
    let commands = [
        "646001ffff600200ff00ff",
        "600230ff60c9600120ff610400ff00ff00ff80ff600101ff",
        "600230ff60c9600120ff610400ff00ff00ff80ff600101ff",
        "70",
        "600230ff60d0600230ff60cb600200ff00ff",
        "600230ff60f1600120ff610800ff00ff00ff00ff00ff00ff00ff80ff600101ff600200ff00ff",
        "600200ff00ff600230ff60cb600230ff60c56664600230ff60c5",
        "600120ff",
        "60040000000000008000",
        "600101ff",
        "600230ff60c5",
        "600120ff",
    ];

    for command in commands {
        let data = hex::decode(command).expect("Bad hex data");
        match handle.write_bulk(1, &data, timeout) {
            Ok(_) => {}
            Err(err) => println!("could not program fpga: {}", err),
        }
    }

    // bitstream?
    let chunks = bitstream.chunks(62);
    for chunk in chunks {
        let mut buf = vec![0x60, 0x1f];
        buf.extend(chunk);
        handle.write_bulk(1, &buf, timeout);
    }

    // config??
    let end_commands = [
        "600b00000004000000040000000400000004000000048000",
        "600101ff",
        "600230ff60d6600200ff00ff600230ff60ff600120ff",
        "60018000",
        "600101ff",
        "600230ff60cc600200ff00ff600230ff60ff600230ff60ff",
        "71038f9db726685e0140c300000230ff6065", // 565
        // "71038f9db726685e014f0800000230ff6065", //888
        "64600200ff00ff600230ff60c2600120ff",
        "6107000f003e00f800100056800a0100", // 565
                                            // "6107008f003c00f200380056800a0100" //888
    ];

    for command in end_commands {
        let data = hex::decode(command).expect("Bad hex data");
        handle.write_bulk(1, &data, timeout);
    }

    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);

    // bulk read to get (C)tan
    // 28432974616eff
    let mut buf = [0; 7];
    handle.read_bulk(0x81, &mut buf, timeout);
    println!("{:02X?}", buf);
}

pub fn check_fpga_programmed<T: UsbContext>(handle: &mut DeviceHandle<T>) -> bool {
    let timeout = Duration::from_secs(1);

    let mut buf = [0; 7];
    handle.read_bulk(0x81, &mut buf, timeout);
    println!("FPGA Response {:02X?}", buf);

    if buf == [0x09, 0x02, 0x27, 0x00, 0x01, 0x01, 0x00] {
        println!("FPGA Empty Response");
        return false;
    }

    return true;
}

pub fn configure_port<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x65], timeout);
}

pub fn fifo_start<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x5b, 0x59, 0x03], timeout);
    handle.write_bulk(1, &[0x40], timeout);
}

pub fn fifo_stop<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x41], timeout);
}