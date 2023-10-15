mod capture;

use bytes::BytesMut;
use rusb::Context;

pub struct Cappy3ds<F> {
    data_callback: F,
    usb_context: Option<rusb::Context>,
    device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

impl<F> Cappy3ds<F>
where
    F: FnMut(&[i16], BytesMut, BytesMut),
{
    pub fn new(data_callback: F) -> Self {
        Self {
            data_callback,
            usb_context: None,
            device_handle: None,
        }
    }

    pub fn connect(&mut self) {
        match Context::new() {
            Ok(mut context) => match capture::katsukitty::connect(&mut context) {
                Ok(handle) => {
                    self.device_handle = Some(handle);
                    self.usb_context = Some(context);
                }
                Err(err) => {
                    println!("{}", err);
                    todo!();
                }
            },
            Err(e) => {
                println!("could not initialize libusb: {}", e);
                todo!();
            }
        }
    }

    pub fn do_capture(self) {
        capture::katsukitty::do_capture(&mut self.device_handle.unwrap(), self.data_callback);
    }
}
