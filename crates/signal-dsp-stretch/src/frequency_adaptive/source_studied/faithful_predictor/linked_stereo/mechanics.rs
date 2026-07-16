pub(super) fn primary_control(sample_rate: usize) -> Vec<f64> {
    (0..sample_rate)
        .map(|index| {
            let time = index as f64 / sample_rate as f64;
            let mut sample = 0.28 * (std::f64::consts::TAU * 110.0 * time).sin()
                + 0.17 * (std::f64::consts::TAU * 523.251 * time).sin();
            if index == sample_rate / 3 || index == sample_rate * 2 / 3 {
                sample += 0.65;
            }
            sample
        })
        .collect()
}

pub(super) fn secondary_control(sample_rate: usize) -> Vec<f64> {
    (0..sample_rate)
        .map(|index| {
            let time = index as f64 / sample_rate as f64;
            let mut sample = 0.21 * (std::f64::consts::TAU * 164.8138 * time + 0.7).sin()
                + 0.13 * (std::f64::consts::TAU * 880.0 * time - 0.4).sin();
            if index == sample_rate / 2 {
                sample -= 0.55;
            }
            sample
        })
        .collect()
}

pub(super) fn scaled(input: &[f64], gain: f64) -> Vec<f64> {
    input.iter().map(|sample| sample * gain).collect()
}

pub(super) fn mismatch_count(actual: &[f64], expected: &[f64]) -> usize {
    actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count()
        + actual.len().abs_diff(expected.len())
}

pub(super) fn signed_mismatch_count(actual: &[f64], expected: &[f64], gain: f64) -> usize {
    actual
        .iter()
        .zip(expected)
        .filter(|(actual, expected)| actual.to_bits() != (**expected * gain).to_bits())
        .count()
        + actual.len().abs_diff(expected.len())
}

pub(super) fn ownership_crossing_control(sample_rate: usize) -> [Vec<f64>; 2] {
    let mut channels = [
        Vec::with_capacity(sample_rate),
        Vec::with_capacity(sample_rate),
    ];
    for index in 0..sample_rate {
        let progress = index as f64 / (sample_rate - 1) as f64;
        let carrier = (std::f64::consts::TAU * 125.0 * index as f64 / sample_rate as f64).sin();
        channels[0].push(carrier * (0.8 - 0.6 * progress));
        channels[1].push(carrier * (0.2 + 0.6 * progress));
    }
    channels
}

pub(super) fn switch_step_growth_db(
    channels: &[Vec<f64>; 2],
    ratio: f64,
    source_frames: usize,
    hop: usize,
) -> f64 {
    let center = (source_frames as f64 * 0.5 * ratio).round() as usize;
    let switch_peak = maximum_step(channels, center.saturating_sub(hop), center + hop);
    let flank_peak = maximum_step(channels, center.saturating_sub(4 * hop), center - 2 * hop)
        .max(maximum_step(channels, center + 2 * hop, center + 4 * hop));
    20.0 * (switch_peak.max(f64::MIN_POSITIVE) / flank_peak.max(f64::MIN_POSITIVE)).log10()
}

fn maximum_step(channels: &[Vec<f64>; 2], start: usize, end: usize) -> f64 {
    channels
        .iter()
        .flat_map(|channel| {
            let end = end.min(channel.len());
            channel[start.min(end)..end]
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).abs())
        })
        .fold(0.0, f64::max)
}
