use std::io::Write;
use std::time::Duration;
use std::{fs, slice, path};
use std::{thread, time};
use rust_embed::RustEmbed;


use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Recipient, RequestType, Result,
    TransferType, UsbContext,
};

#[derive(RustEmbed)]
#[folder = "resources/Katsukity/firm.bin"]
struct Katsukity;


fn main() {
    let firmware = Katsukity::get("firm.bin").unwrap();
    let bitstream = Katsukity::get("bitstream.bin").unwrap();

    let vid = 0x0752;
    let pid = 0x8613;

    match Context::new() {
        Ok(mut context) => {
            match open_device(&mut context, vid, pid) {
                Some((mut device, device_desc, mut handle)) => {
                    println!("Opened {:04x}:{:04x}", vid, pid);
                    send_firmware(&mut handle, firmware.data.to_vec());
                }
                None => println!("could not find FX2 device {:04x}:{:04x}", vid, pid),
            }

            let sleep_time = time::Duration::from_millis(5000);
            thread::sleep(sleep_time);
            // todo: loop with device check instead of sleeping


            match open_device(&mut context, 0x0752, 0xf2c0) {
                Some((mut device, device_desc, mut handle)) => {
                    println!("Opened secondary device");
                    read_eeprom(&mut handle);
                    configure_fpga(&mut handle, bitstream.data.to_vec());
                    configure_port(&mut handle);
                    fifo_start(&mut handle);

                    let mut file = fs::File::create("./image2.bin").expect("WHY NO FILE");


                    let mut test = 0;
                    while test < 100 {
                        bulk_read(&mut handle, &mut file);
                        test += 1;
                    }
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

fn send_firmware<T: UsbContext>(handle: &mut DeviceHandle<T>, firmware: Vec<u8>) {
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

fn read_eeprom<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    let mut offset = 0;

    let mut eeprom = Vec::<u8>::with_capacity(128);

    let mut buf = [0; 16];

    while offset <= 0x70 {
        handle.write_bulk(1, &[0x38, offset, 0x10, 0x30], timeout);

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

fn configure_fpga<T: UsbContext>(handle: &mut DeviceHandle<T>, bitstream: Vec<u8>) {
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
        handle.write_bulk(1, &data, timeout);
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
        "71038f9db726685e0140c300000230ff6065",
        "64600200ff00ff600230ff60c2600120ff",
        "6107000f003e00f800100056800a0100",
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

fn configure_port<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x65], timeout);
}

fn fifo_start<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x5b, 0x59, 0x03], timeout);
    handle.write_bulk(1, &[0x40], timeout);
}

fn fifo_stop<T: UsbContext>(handle: &mut DeviceHandle<T>) {
    let timeout = Duration::from_secs(1);

    handle.write_bulk(1, &[0x41], timeout);
}

fn bulk_read<T: UsbContext>(handle: &mut DeviceHandle<T>, file: &mut fs::File) {
    let timeout = Duration::from_secs(1);

    let mut vec = Vec::<u8>::with_capacity(65563);
    let mut buf = unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

    handle.read_bulk(0x82, &mut buf, timeout);

    //println!("{}", buf.len());
    //println!("{:02X?}", buf);

    if buf.len() > 16384 {
        file.write_all(buf);
    }
}
