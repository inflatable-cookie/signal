use rustfft::num_complex::Complex64;

use super::*;

#[derive(Clone, Copy)]
pub(super) enum Signal {
    A,
    B,
    Silence,
}

pub(super) type Frame = [Vec<Complex64>; CHANNEL_CAPACITY];

pub(super) fn frame(
    signal: Signal,
    frequencies_hz: &[f64],
    sample_rate: usize,
    hop: usize,
    time: isize,
) -> Vec<Complex64> {
    frequencies_hz
        .iter()
        .enumerate()
        .map(|(band, frequency)| {
            let (gain, offset) = match signal {
                Signal::A => (1.0, 0.17),
                Signal::B => (0.63, -0.29),
                Signal::Silence => return Complex64::default(),
            };
            let peak = if band % 11 == 3 { 1.0 } else { 0.18 };
            let magnitude = gain * (peak + (band % 7) as f64 * 0.004);
            let phase =
                std::f64::consts::TAU * frequency / sample_rate as f64 * hop as f64 * time as f64
                    + band as f64 * 0.013
                    + offset;
            Complex64::from_polar(magnitude, phase)
        })
        .collect()
}

pub(super) fn layers(state: &Frame, active: [Option<usize>; 2]) -> Vec<Frame> {
    active
        .into_iter()
        .flatten()
        .map(|slice| {
            std::array::from_fn(|channel| {
                state[channel]
                    .iter()
                    .enumerate()
                    .map(|(band, value)| {
                        let scale = 0.55 + ((slice + band) % 4) as f64 * 0.09;
                        let offset = ((slice * 3 + band) % 9) as f64 * 0.017 - 0.068;
                        Complex64::from_polar(value.norm() * scale, value.arg() + offset)
                    })
                    .collect()
            })
        })
        .collect()
}

pub(super) fn scenario(
    signals: [Signal; CHANNEL_CAPACITY],
    frequencies_hz: &[f64],
    geometry: &Geometry,
    time: isize,
    active: [Option<usize>; 2],
) -> (Frame, Vec<Frame>) {
    let state = std::array::from_fn(|channel| {
        frame(
            signals[channel],
            frequencies_hz,
            geometry.sample_rate,
            geometry.hop,
            time,
        )
    });
    let layers = layers(&state, active);
    (state, layers)
}
