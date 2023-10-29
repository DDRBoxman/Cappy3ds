pub mod katsukitty;

use simple_error::SimpleError;
use rusb::{Device, DeviceDescriptor, DeviceHandle, UsbContext};

trait Capture {
    fn connect<T: UsbContext>(context: &mut T) -> Result<DeviceHandle<T>, SimpleError>;

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
}

