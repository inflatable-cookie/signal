pub(super) use super::*;
pub(super) use crate::StretchBackendTier;

pub(super) const SAMPLE_RATE: SampleRate = SampleRate(8_000);

pub(super) fn mono_input(frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|frame| {
            (0.4 * (std::f64::consts::TAU * 220.0 * frame as f64 / f64::from(SAMPLE_RATE.0)).sin())
                as Sample
        })
        .collect()
}

pub(super) fn stereo_input(frames: usize) -> Vec<Sample> {
    let mut input = Vec::with_capacity(frames * 2);
    for frame in 0..frames {
        let time = frame as f64 / f64::from(SAMPLE_RATE.0);
        input.push((0.35 * (std::f64::consts::TAU * 180.0 * time).sin()) as Sample);
        input.push((0.25 * (std::f64::consts::TAU * 310.0 * time).sin()) as Sample);
    }
    input
}

pub(super) fn dream_parity_targets(source_frames: usize) -> [usize; 9] {
    [
        source_frames * 4,
        source_frames * 4 + 1,
        source_frames * 9 / 2,
        source_frames * 6,
        source_frames * 8,
        source_frames * 10,
        source_frames * 31 / 2,
        source_frames * 16 - 1,
        source_frames * 16,
    ]
}

pub(super) fn cyclic_parity_targets(source_frames: usize) -> [usize; 12] {
    [
        source_frames * 2,
        source_frames * 2 + 1,
        source_frames * 5 / 2,
        source_frames * 3,
        source_frames * 4 - 1,
        source_frames * 4,
        source_frames * 4 + 1,
        source_frames * 5,
        source_frames * 6,
        source_frames * 15 / 2,
        source_frames * 8 - 1,
        source_frames * 8,
    ]
}

pub(super) fn dream_private_render(
    input: &[Sample],
    channels: usize,
    target_frames: usize,
    space: f32,
) -> Vec<Sample> {
    render_dream(DreamCandidateRequest {
        input,
        channels,
        sample_rate: SAMPLE_RATE.0,
        target_frames,
        seed: ADMISSION_SEED,
        space,
    })
    .expect("private reference render")
}

pub(super) fn cyclic_private_render(
    input: &[Sample],
    channels: usize,
    target_frames: usize,
    cycle_us: u32,
) -> Vec<Sample> {
    render_cyclic(CyclicRequest {
        input,
        channels,
        sample_rate: SAMPLE_RATE.0,
        target_frames,
        cycle_us,
    })
    .expect("private Cyclic reference render")
}

mod boundaries;
mod constants;
mod errors;
mod parity;
mod render;
mod request;
mod surface;
mod validation;
