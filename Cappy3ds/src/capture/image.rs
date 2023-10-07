use bytes::{BufMut, BytesMut};
use memchr::memmem;
extern crate libusb1_sys as usbffi;

pub fn parse_image_data(data: &Vec<u8>) {
    let mut found_frames = 0;

    let mut frame_iter = memmem::find_iter(&data, &[0x33, 0xCC, 0x00, 0x00]);
    while let Some(frame_start) = frame_iter.next() {
        //println!("{}", frame_start);

        let mut out_buf = BytesMut::with_capacity(0x40000);

        //let mut buf =  BytesMut::from(data.as_slice());
        let thing = &data[frame_start + 3..];

        let mut it = memmem::find_iter(&thing, &[0x33, 0xCC]);

        while let Some(res) = it.next() {
            /*if (thing[res + 2] == 0x90 ) {
                break;
            }*/
            //println!("{}", res);
            if res + (248 * 2) > thing.len() {
                break;
            }
            out_buf.put(&thing[res..res + (248 * 2)])
        }

        let mut image_buffer = BytesMut::with_capacity(0x40000);
        image_buffer.resize((out_buf.len() / 2 * 3), 0);

        let mut j = 0;
        for i in (0..out_buf.len()).step_by(2) {
            let high = (out_buf[i + 1] as u16) << 8;
            let c: u16 = out_buf[i] as u16 + high;
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

        use image::{ImageBuffer, Rgb};
        let buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(
            248,
            (out_buf.len() / 2 / 248).try_into().unwrap(),
            image_buffer,
        )
        .unwrap();

        buffer.save(format!("./test_{}.png", found_frames));

        //let mut file = fs::File::create(format!("./test_{}.bin", found_frames)).expect("WHY NO FILE");

        //file.write_all(&out_buf);

        found_frames += 1;
    }
}
