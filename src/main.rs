use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};

fn main() {
    println!("Hello, world!");

    let vid = 0x0752;
    let pid = 0x8613;

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {
            Some((mut device, device_desc, mut handle)) => {
                println!("Opened {:04x}:{:04x}", vid, pid);
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
    handle: &mut DeviceHandle<T>
) { 
    handle.write_control(request_type, request, value, index, buf, timeout)
}