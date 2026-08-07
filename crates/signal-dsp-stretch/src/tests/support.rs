use super::*;

pub(super) fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
    (0..len)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin())
        .collect()
}

/// Dominant frequency estimate by zero-crossing count over a trimmed
/// interior span (skips windup/tail edges).
pub(super) fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
}

pub(super) fn rms(samples: &[Sample]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

pub(super) fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
    let mut input = vec![0.0; len];
    input[..edge_frames].fill(0.5);
    input[len - edge_frames..].fill(-0.5);
    input
}

pub(super) fn add_decaying_burst(
    samples: &mut [Sample],
    start: usize,
    frames: usize,
    amplitude: f32,
) {
    for offset in 0..frames {
        let Some(sample) = samples.get_mut(start + offset) else {
            break;
        };
        let envelope = 1.0 - offset as f32 / frames as f32;
        let polarity = if offset % 2 == 0 { 1.0 } else { -1.0 };
        *sample += amplitude * envelope * polarity;
    }
}

pub(super) fn masked_soft_attack_probe(soft_attack_amplitude: f32) -> Vec<Sample> {
    let mut input = sine(180.0, 48_000.0, 48_000)
        .into_iter()
        .map(|sample| sample * 0.06)
        .collect::<Vec<_>>();
    add_decaying_burst(&mut input, 8_000, 96, 1.0);
    add_decaying_burst(&mut input, 24_000, 96, soft_attack_amplitude);
    input
}
