use bytes::{BufMut, BytesMut};
use image::{ImageBuffer, Rgb};
use itertools::Itertools;
use memchr::memmem;

extern crate libusb1_sys as usbffi;

use crate::capture::katsukitty::parse;

pub fn parse_image_data(data: &Vec<u8>) {
    let mut found_frames = 0;

    let finder = memmem::Finder::new(&[0x33, 0xCC, 0x00, 0x00]);

    let iter = finder.find_iter(&data);

    let result = iter.tuple_windows().collect::<Vec<_>>();

    for (start, end) in result {
        let mut out_buf = BytesMut::with_capacity(0x40000);
        out_buf.put(&data[start..end]);

        let (upper_buffer, lower_buffer, sound_buffer) = parse::split_capture_buffer(&out_buf);

        // print lower image
        let result =
            ImageBuffer::<Rgb<u8>, _>::from_raw(240, 320, parse::rgb565_to_rgb(&lower_buffer));
        if let Some(image) = result {
            image.save(format!("./img_out/lower_{}.png", found_frames));
        }

        // print upper image
        let result: Option<ImageBuffer<Rgb<u8>, BytesMut>> =
            ImageBuffer::<Rgb<u8>, _>::from_raw(240, 400, parse::rgb565_to_rgb(&upper_buffer));
        if let Some(image) = result {
            image.save(format!("./img_out/upper_{}.png", found_frames));
        }

        // print audio for fun
        let result: Option<ImageBuffer<Rgb<u8>, BytesMut>> =
            ImageBuffer::<Rgb<u8>, _>::from_raw(8, 801, parse::rgb565_to_rgb(&sound_buffer));
        if let Some(image) = result {
            image.save(format!("./img_out/audio{}.png", found_frames));
        }

        found_frames += 1;
    }
}
