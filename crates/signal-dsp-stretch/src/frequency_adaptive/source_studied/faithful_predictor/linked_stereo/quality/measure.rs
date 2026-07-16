use rustfft::num_complex::Complex64;

#[derive(Clone, Copy, Debug)]
pub(super) struct ImageDelta {
    pub(super) mid_side_ratio_db: f64,
    pub(super) correlation: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TransientQuality {
    pub(super) maximum_error: usize,
    pub(super) replica_failures: usize,
    pub(super) silent_peer_peak: f64,
}

pub(super) fn maximum_ipd_error(
    input: &[Vec<f64>; 2],
    output: &[Vec<f64>; 2],
    frequencies: &[f64],
    sample_rate: usize,
) -> f64 {
    frequencies
        .iter()
        .map(|frequency| {
            let input_ipd = ipd(input, *frequency, sample_rate);
            let output_ipd = ipd(output, *frequency, sample_rate);
            wrap(output_ipd - input_ipd).abs()
        })
        .fold(0.0, f64::max)
}

pub(super) fn maximum_expected_ipd_error(
    output: &[Vec<f64>; 2],
    expected_ipd: f64,
    frequencies: &[f64],
    sample_rate: usize,
) -> f64 {
    frequencies
        .iter()
        .map(|frequency| {
            let output_ipd = ipd(output, *frequency, sample_rate);
            wrap(output_ipd - expected_ipd).abs()
        })
        .fold(0.0, f64::max)
}

pub(super) fn ipd(channels: &[Vec<f64>; 2], frequency: f64, sample_rate: usize) -> f64 {
    wrap(
        projection(&channels[1], frequency, sample_rate).arg()
            - projection(&channels[0], frequency, sample_rate).arg(),
    )
}

pub(super) fn best_delay(left: &[f64], right: &[f64], radius: usize) -> usize {
    (0..=radius)
        .max_by(|left_delay, right_delay| {
            delay_correlation(left, right, *left_delay).total_cmp(&delay_correlation(
                left,
                right,
                *right_delay,
            ))
        })
        .unwrap_or(0)
}

pub(super) fn image_delta(input: &[Vec<f64>; 2], output: &[Vec<f64>; 2]) -> ImageDelta {
    let input = image_stats(input);
    let output = image_stats(output);
    ImageDelta {
        mid_side_ratio_db: (output.mid_side_ratio_db - input.mid_side_ratio_db).abs(),
        correlation: (output.correlation - input.correlation).abs(),
    }
}

pub(super) fn transient_quality(
    output: &[Vec<f64>; 2],
    source_events: &[usize],
    ratio: f64,
) -> TransientQuality {
    let targets = source_events
        .iter()
        .map(|event| (*event as f64 * ratio).round() as usize)
        .collect::<Vec<_>>();
    let mut protected = vec![false; output[0].len()];
    let mut maximum_error = 0;
    let mut minimum_attack_peak = f64::INFINITY;
    for target in targets {
        let start = target.saturating_sub(256);
        let end = (target + 257).min(output[0].len());
        protected[start..end].fill(true);
        let (offset, peak) = output[0][start..end]
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))
            .map(|(offset, sample)| (offset, sample.abs()))
            .unwrap_or((target - start, 0.0));
        maximum_error = maximum_error.max((start + offset).abs_diff(target));
        minimum_attack_peak = minimum_attack_peak.min(peak);
    }
    let intermediate_peak = output[0]
        .iter()
        .zip(protected)
        .filter(|(_, protected)| !protected)
        .map(|(sample, _)| sample.abs())
        .fold(0.0, f64::max);
    let silent_peer_peak = output[1]
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    TransientQuality {
        maximum_error,
        replica_failures: usize::from(intermediate_peak > minimum_attack_peak),
        silent_peer_peak,
    }
}

fn projection(samples: &[f64], frequency: f64, sample_rate: usize) -> Complex64 {
    samples
        .iter()
        .enumerate()
        .fold(Complex64::new(0.0, 0.0), |sum, (index, sample)| {
            let phase = -std::f64::consts::TAU * frequency * index as f64 / sample_rate as f64;
            sum + Complex64::from_polar(*sample, phase)
        })
}

fn delay_correlation(left: &[f64], right: &[f64], delay: usize) -> f64 {
    let length = left.len().min(right.len()).saturating_sub(delay);
    let left = &left[..length];
    let right = &right[delay..delay + length];
    normalized_correlation(left, right)
}

#[derive(Clone, Copy)]
struct ImageStats {
    mid_side_ratio_db: f64,
    correlation: f64,
}

fn image_stats(channels: &[Vec<f64>; 2]) -> ImageStats {
    let (left_energy, right_energy, cross, mid_energy, side_energy) = channels[0]
        .iter()
        .zip(&channels[1])
        .fold((0.0, 0.0, 0.0, 0.0, 0.0), |sum, (left, right)| {
            let mid = 0.5 * (left + right);
            let side = 0.5 * (left - right);
            (
                sum.0 + left * left,
                sum.1 + right * right,
                sum.2 + left * right,
                sum.3 + mid * mid,
                sum.4 + side * side,
            )
        });
    ImageStats {
        mid_side_ratio_db: 10.0
            * (side_energy.max(f64::MIN_POSITIVE) / mid_energy.max(f64::MIN_POSITIVE)).log10(),
        correlation: cross / (left_energy * right_energy).sqrt().max(f64::MIN_POSITIVE),
    }
}

fn normalized_correlation(left: &[f64], right: &[f64]) -> f64 {
    let (left_energy, right_energy, cross) =
        left.iter()
            .zip(right)
            .fold((0.0, 0.0, 0.0), |sum, (left, right)| {
                (
                    sum.0 + left * left,
                    sum.1 + right * right,
                    sum.2 + left * right,
                )
            });
    cross / (left_energy * right_energy).sqrt().max(f64::MIN_POSITIVE)
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
