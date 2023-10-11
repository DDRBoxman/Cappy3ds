use bytes::{BufMut, BytesMut};
use memchr::memmem;
extern crate libusb1_sys as usbffi;
use image::{ImageBuffer, Rgb};
use itertools::Itertools;

pub fn parse_image_data(data: &Vec<u8>) {
    let mut found_frames = 0;

    let finder = memmem::Finder::new(&[0x33, 0xCC, 0x00, 0x00]);

    let iter = finder.find_iter(&data);

    let result = iter.tuple_windows().collect::<Vec<_>>();
    
    for (start, end) in result {
        let mut out_buf = BytesMut::with_capacity(0x40000);
        out_buf.put(&data[start..end]);

        let (upper_buffer, lower_buffer, sound_buffer) = split_capture_buffer(&out_buf);

        // print lower image
        let result = rgb565_to_rgb(240, 320, &lower_buffer);
        if let Some(image) = result {
            image.save(format!("./img_out/lower_{}.png", found_frames));
        }

        // print upper image
        let result = rgb565_to_rgb(240, 400, &upper_buffer);
        if let Some(image) = result {
            image.save(format!("./img_out/upper_{}.png", found_frames));
        }

        // print audio for fun
        let result = rgb565_to_rgb(8, 801, &sound_buffer);
        if let Some(image) = result {
            image.save(format!("./img_out/audio{}.png", found_frames));
        }

        found_frames += 1;
    }
}

fn split_capture_buffer(data: &BytesMut) -> (BytesMut, BytesMut, BytesMut) {
    let mut upper_buffer = BytesMut::with_capacity(400 * 240 * 2);
    let mut lower_buffer = BytesMut::with_capacity(320 * 240 * 2);
    let mut sound_buffer = BytesMut::with_capacity(720 * 8 * 2);

    // split apart buffer into parts
    // todo: replace with a nicer byte stream reader
    let mut pos = 0;
    while pos < data.len() - 496 {
        if pos < 81 * 496 {
            // copy preamble audio
            sound_buffer.extend(&data[pos..pos + 16]);
            pos += 496;
        } else if pos >= 81 * 496 && pos < 400 * 496 {
            sound_buffer.extend(&data[pos..pos + 16]);
            pos += 16;
            lower_buffer.extend(&data[pos..pos + 480]);
            pos += 480;
        } else if pos == 400 * 496 {
            lower_buffer.extend(&data[pos..pos + 480]);
            pos += 480;
            sound_buffer.extend(&data[pos..pos + 16]);
            pos += 16;
        } else if pos > 400 * 496 {
            upper_buffer.extend(&data[pos..pos + 480]);
            pos += 480;
            sound_buffer.extend(&data[pos..pos + 16]);
            pos += 16;
        }
    }

    return (upper_buffer, lower_buffer, sound_buffer);
}

fn rgb565_to_rgb(
    width: u32,
    height: u32,
    data: &BytesMut,
) -> Option<ImageBuffer<Rgb<u8>, BytesMut>> {
    let mut image_buffer = BytesMut::new();
    image_buffer.resize((width * height * 3).try_into().unwrap(), 0);

    let mut j = 0;
    for i in (0..data.len()).step_by(2) {
        let high = (data[i + 1] as u16) << 8;
        let c: u16 = data[i] as u16 + high;
        let r = (((c & 0xF800) >> 11) << 3) as u8;
        let g = (((c & 0x7E0) >> 5) << 2) as u8;
        let b = ((c & 0x1F) << 3) as u8;

        image_buffer[j] = r;
        j += 1;
        image_buffer[j] = g;
        j += 1;
        image_buffer[j] = b;
        j += 1;
    }

    ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, image_buffer)
}
