use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleRate, StreamConfig,
};
use lerp::Lerp;
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44_100;
// const AUDIO_FRAME_RATE: u32 = 240 * 128 * 8;
const STREAM_CONFIG: StreamConfig = StreamConfig {
    channels: 1,
    sample_rate: SampleRate(SAMPLE_RATE),
    buffer_size: BufferSize::Default,
    // buffer_size: BufferSize::Fixed(SAMPLE_RATE / AUDIO_FRAME_RATE),
};

#[macroquad::main("BasicShapes")]
async fn main() -> Result<(), anyhow::Error> {
    let fft = Arc::new(Mutex::new(Vec::<i16>::new()));
    let err = Arc::new(Mutex::new(None));

    let host = cpal::default_host();
    let inpdev = host
        .default_input_device()
        .ok_or(anyhow::anyhow!("no input device detected"))?;
    let stream = inpdev.build_input_stream::<i16, _, _>(
        &STREAM_CONFIG,
        {
            let fft = Arc::clone(&fft);
            move |samples, _| {
                let mut fft = fft.lock().unwrap();
                fft.clear();
                fft.extend_from_slice(samples);
            }
        },
        {
            let err = Arc::clone(&err);
            move |e| {
                let mut err = err.lock().unwrap();
                *err = Some(e.into());
            }
        },
    )?;
    stream.play()?;

    let mut local_fft = Vec::<i16>::new();

    loop {
        clear_background(RED);

        {
            let fft = fft.lock().unwrap();
            local_fft.clear();
            local_fft.extend_from_slice(&fft);
        }

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text(&format!("{}", local_fft.len()), 20.0, 20.0, 20.0, DARKGRAY);
        // graph(&local_fft, )

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        if let Some(err) = std::mem::take(&mut *err.lock().unwrap()) {
            return Err(err);
        }

        next_frame().await
    }

    Ok(())

    // audio_thread.join().unwrap()
}

fn graph(dat: &[i16], left: f32, right: f32, top: f32, bottom: f32, thickness: f32, color: Color) {
    let len = dat.len() as f32;
    for (i, pair) in dat.windows(2).enumerate() {
        let (y1, y2) = (
            pair[0] as f32 / i16::max_value() as f32,
            pair[1] as f32 / i16::max_value() as f32,
        );
        let i = i as f32;
        draw_line(
            left.lerp(right, i / len),
            bottom.lerp(top, y1),
            left.lerp(right, (i + 1.0) / len),
            bottom.lerp(top, y2),
            thickness,
            color,
        )
    }
}
