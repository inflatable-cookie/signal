pub(super) fn identity_error(input: &[f64], output: &[f64]) -> f64 {
    input
        .iter()
        .zip(output)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

pub(super) fn tone_error(output: &[f64]) -> f64 {
    let start = output.len() * 35 / 100;
    let end = output.len() * 47 / 100;
    let mut best = (0.0, 0.0);
    for frequency in 980..=1_014 {
        let omega = std::f64::consts::TAU * frequency as f64 / 48_000.0;
        let (real, imaginary) =
            output[start..end]
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |sum, (index, sample)| {
                    (
                        sum.0 + sample * (omega * index as f64).cos(),
                        sum.1 - sample * (omega * index as f64).sin(),
                    )
                });
        let power = real * real + imaginary * imaginary;
        if power > best.1 {
            best = (frequency as f64, power);
        }
    }
    (best.0 - 997.0).abs()
}

pub(super) fn event_error(output: &[f64], ratio: f64) -> usize {
    [2_048, 4_096, 4_224, 8_192, 12_288]
        .into_iter()
        .map(|event| {
            let expected = (event as f64 * ratio).round() as usize;
            let start = expected.saturating_sub(256);
            let end = (expected + 257).min(output.len());
            let peak = (start..end)
                .max_by(|left, right| output[*left].abs().total_cmp(&output[*right].abs()))
                .unwrap_or(start);
            peak.abs_diff(expected)
        })
        .max()
        .unwrap_or(0)
}

pub(super) fn quality(input: &[f64], output: &[f64], _ratio: f64) -> [f64; 5] {
    let input_rms = rms(input);
    let output_rms = rms(output);
    let crest =
        (peak(output) / output_rms.max(1.0e-12) - peak(input) / input_rms.max(1.0e-12)).abs();
    let tone = (zero_crossing_rate(output) - zero_crossing_rate(input)).abs();
    let derivative = (mean_derivative(output) / output_rms.max(1.0e-12)
        - mean_derivative(input) / input_rms.max(1.0e-12))
    .abs();
    let endpoint = output
        .windows(2)
        .take(256)
        .chain(output.windows(2).rev().take(256))
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0, f64::max);
    let residual = second_difference(output) / output_rms.max(1.0e-12);
    [crest, tone, derivative, endpoint, residual]
}

pub(super) fn dominates(left: [f64; 5], right: [f64; 5]) -> bool {
    left.iter().zip(right).all(|(left, right)| left <= &right)
        && left.iter().zip(right).any(|(left, right)| left < &right)
}

fn zero_crossing_rate(samples: &[f64]) -> f64 {
    samples
        .windows(2)
        .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
        .count() as f64
        / samples.len().max(1) as f64
}

fn rms(samples: &[f64]) -> f64 {
    (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
}
fn peak(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max)
}
fn mean_derivative(samples: &[f64]) -> f64 {
    samples
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .sum::<f64>()
        / samples.len().max(1) as f64
}
fn second_difference(samples: &[f64]) -> f64 {
    samples
        .windows(3)
        .map(|part| (part[2] - 2.0 * part[1] + part[0]).abs())
        .sum::<f64>()
        / samples.len().max(1) as f64
}
