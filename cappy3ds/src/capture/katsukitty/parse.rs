use bytes::BytesMut;

// 33CC 23C0 1800 0000 0000 1900 0000 0000 (then 240 pixels of image)
// 33CC 24C0 1900 0000 0000 1A00 0000 0000 (240)
// 33CC 25C0 1A00 0000 0000 1B00 0000 0000
// 33CC 26C0 1C00 0000 0000 1D00 0000 0000
// ...
// 33CC 2EC1 0701 0000 0000 0801 0000 0000

pub fn split_capture_buffer(data: &BytesMut) -> (BytesMut, BytesMut, BytesMut) {
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

    (upper_buffer, lower_buffer, sound_buffer)
}

pub fn rgb565_to_rgb(data: &BytesMut) -> BytesMut {
    let mut image_buffer = BytesMut::with_capacity(data.len() / 2 * 3);
    image_buffer.resize(data.len() / 2 * 3, 0);

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

    image_buffer
}

pub fn rgb565_to_rgba(data: &BytesMut) -> BytesMut {
    let mut image_buffer = BytesMut::with_capacity(data.len() / 2 * 4);
    image_buffer.resize(data.len() / 2 * 4, 0);

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
        image_buffer[j] = 0;
        j += 1;
    }

    image_buffer
}
