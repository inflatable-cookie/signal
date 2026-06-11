//! First sound from the Signal stack: a two-second 440 Hz sine on the
//! default output device.
//!
//! Run with: `cargo run -p signal-hardware-output-cpal --example tone`

use std::f32::consts::TAU;

use signal_hardware::{OutputStreamBackend, OutputStreamSpec};
use signal_hardware_output_cpal::CpalOutputBackend;

fn main() {
    let sample_rate_hz = 48_000u32;
    let channels = 2u16;
    let mut phase = 0.0f32;
    let phase_step = 440.0 * TAU / sample_rate_hz as f32;

    let backend = CpalOutputBackend::new();
    let stream = backend
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz,
                channels,
                buffer_frames: None,
            },
            Box::new(move |frames| {
                for frame in frames.chunks_mut(channels as usize) {
                    let sample = (phase).sin() * 0.2;
                    phase = (phase + phase_step) % TAU;
                    for slot in frame {
                        *slot = sample;
                    }
                }
            }),
        )
        .expect("open output stream");

    println!(
        "playing 440 Hz for 2s on default device ({} Hz, {} ch, state {:?})",
        stream.sample_rate_hz(),
        stream.channels(),
        stream.state()
    );
    std::thread::sleep(std::time::Duration::from_secs(2));
    drop(stream);
    println!("stopped");
}
