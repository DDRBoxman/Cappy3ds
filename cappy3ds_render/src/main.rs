use bytes::BytesMut;
use cappy3ds;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use image::{ImageBuffer, Rgb};
use ringbuf::HeapRb;

fn main() {
    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();

    let config: cpal::StreamConfig = output_device.default_output_config().unwrap().into();

    //config.sample_rate.0 = 32728;
    let latency_frames = (150.0 / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;

    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match consumer.pop() {
                Some(s) => s,
                None => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        if input_fell_behind {
            //eprintln!("input stream fell behind: try increasing latency");
        }
    };

    let output_stream = output_device
        .build_output_stream(&config, output_data_fn, err_fn, None)
        .unwrap();
    println!("Successfully built streams.");

    let mut cappy3ds = cappy3ds::Cappy3ds::new(
        |audio: &[i16], upper_buffer: BytesMut, lower_buffer: BytesMut| {
            print!("{:?}\n", upper_buffer.len());

            if upper_buffer.len() >= 288000 {
                let found_frames = "wow";

                // print lower image
                let result = ImageBuffer::<Rgb<u8>, _>::from_raw(240, 320, lower_buffer);
                if let Some(image) = result {
                    image.save(format!("./img_out/lower_{}.png", found_frames));
                }

                // print upper image
                let result: Option<ImageBuffer<Rgb<u8>, BytesMut>> =
                    ImageBuffer::<Rgb<u8>, _>::from_raw(240, 400, upper_buffer);
                if let Some(image) = result {
                    image.save(format!("./img_out/upper_{}.png", found_frames));
                }

                panic!("LOL");
            }

            /*for sample in data {
                //producer.push(0.0).unwrap();
            }*/
        },
    );

    cappy3ds.connect();

    //output_stream.play();

    cappy3ds.do_capture();
}

fn err_fn(err: cpal::StreamError) {
    //eprintln!("an error occurred on stream: {}", err);
}
