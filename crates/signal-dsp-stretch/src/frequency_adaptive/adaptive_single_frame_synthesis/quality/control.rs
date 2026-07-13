use std::f64::consts::TAU;

use super::super::super::study_local_schedule::SOURCE_FRAMES;

pub(in crate::frequency_adaptive) const SAMPLE_RATE: f64 = 48_000.0;
pub(super) const RATIOS: [f64; 4] = [1.0, 0.75, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum Control {
    LowTone,
    MidTone,
    HighTone,
    TwoTone,
    LinearChirp,
    ExponentialChirp,
    IsolatedImpulse,
    DenseEvent,
    Boundary,
    Noise,
    Mixed,
    Silence,
}

impl Control {
    pub(in crate::frequency_adaptive) fn tone_hz(self) -> Option<f64> {
        match self {
            Self::LowTone => Some(55.0),
            Self::MidTone => Some(440.0),
            Self::HighTone => Some(8_000.0),
            _ => None,
        }
    }

    pub(super) fn texture(self) -> bool {
        matches!(
            self,
            Self::LowTone
                | Self::MidTone
                | Self::HighTone
                | Self::TwoTone
                | Self::LinearChirp
                | Self::ExponentialChirp
                | Self::Mixed
        )
    }
}

pub(in crate::frequency_adaptive) fn controls() -> Vec<(Control, Vec<f64>)> {
    vec![
        (Control::LowTone, tone(55.0)),
        (Control::MidTone, tone(440.0)),
        (Control::HighTone, tone(8_000.0)),
        (
            Control::TwoTone,
            (0..SOURCE_FRAMES)
                .map(|index| 0.6 * sinusoid(220.0, index) + 0.4 * sinusoid(3_000.0, index))
                .collect(),
        ),
        (
            Control::LinearChirp,
            chirp(|position| 55.0 + (8_000.0 - 55.0) * position),
        ),
        (
            Control::ExponentialChirp,
            chirp(|position| 55.0 * (8_000.0_f64 / 55.0).powf(position)),
        ),
        (
            Control::IsolatedImpulse,
            impulses(&[(SOURCE_FRAMES / 2, 1.0)]),
        ),
        (
            Control::DenseEvent,
            impulses(&[
                (SOURCE_FRAMES / 2 - 128, 1.0),
                (SOURCE_FRAMES / 2 + 128, 0.75),
            ]),
        ),
        (
            Control::Boundary,
            impulses(&[(0, 1.0), (SOURCE_FRAMES - 1, -0.75)]),
        ),
        (Control::Noise, noise()),
        (
            Control::Mixed,
            noise()
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    0.45 * sinusoid(220.0, index)
                        + 0.08 * value
                        + if index == SOURCE_FRAMES / 2 { 1.0 } else { 0.0 }
                })
                .collect(),
        ),
        (Control::Silence, vec![0.0; SOURCE_FRAMES]),
    ]
}

pub(super) fn tone(hz: f64) -> Vec<f64> {
    (0..SOURCE_FRAMES)
        .map(|index| sinusoid(hz, index))
        .collect()
}

pub(super) fn sinusoid(hz: f64, index: usize) -> f64 {
    (TAU * hz * index as f64 / SAMPLE_RATE).sin()
}

fn chirp(frequency: impl Fn(f64) -> f64) -> Vec<f64> {
    let mut phase = 0.0_f64;
    (0..SOURCE_FRAMES)
        .map(|index| {
            let sample = phase.sin();
            phase += TAU * frequency(index as f64 / SOURCE_FRAMES as f64) / SAMPLE_RATE;
            sample
        })
        .collect()
}

fn impulses(events: &[(usize, f64)]) -> Vec<f64> {
    let mut result = vec![0.0; SOURCE_FRAMES];
    for (index, amplitude) in events {
        result[*index] = *amplitude;
    }
    result
}

fn noise() -> Vec<f64> {
    let mut state = 0x8f3d_9b17_u32;
    (0..SOURCE_FRAMES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as f64 / u32::MAX as f64 * 2.0 - 1.0
        })
        .collect()
}
