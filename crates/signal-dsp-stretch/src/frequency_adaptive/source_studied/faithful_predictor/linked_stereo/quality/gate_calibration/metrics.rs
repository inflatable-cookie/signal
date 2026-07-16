#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ControlKind {
    Tone,
    Image,
}

impl ControlKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Tone => "tone",
            Self::Image => "image",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct Metrics {
    pub ipd_error_radians: f64,
    pub mid_side_delta_db: f64,
    pub correlation_delta: f64,
    pub relation_residual: f64,
}

pub(super) fn control(
    kind: ControlKind,
    length: usize,
    frequency: f64,
    phase: f64,
) -> [Vec<f64>; 2] {
    match kind {
        ControlKind::Tone => std::array::from_fn(|channel| {
            (0..length)
                .map(|index| {
                    let offset = if channel == 0 { 0.0 } else { 0.83 };
                    0.28 * (std::f64::consts::TAU * frequency * index as f64 / 8_000.0
                        + phase
                        + offset)
                        .sin()
                })
                .collect()
        }),
        ControlKind::Image => {
            let left = multi_tone(length, phase, &[91.3, 223.7, 487.1], &[0.17, 0.11, 0.08]);
            let independent = multi_tone(
                length,
                phase + 0.41,
                &[137.9, 319.3, 701.7],
                &[0.13, 0.09, 0.06],
            );
            let right = left
                .iter()
                .zip(independent)
                .map(|(shared, other)| shared * 0.68 + other * 0.22)
                .collect();
            [left, right]
        }
    }
}

fn multi_tone(length: usize, phase: f64, frequencies: &[f64], amplitudes: &[f64]) -> Vec<f64> {
    (0..length)
        .map(|index| {
            frequencies
                .iter()
                .zip(amplitudes)
                .enumerate()
                .map(|(tone, (frequency, amplitude))| {
                    amplitude
                        * (std::f64::consts::TAU * frequency * index as f64 / 8_000.0
                            + phase
                            + tone as f64 * 0.73)
                            .sin()
                })
                .sum()
        })
        .collect()
}

pub(super) fn crop(channels: &[Vec<f64>; 2], trim: usize) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| channels[channel][trim..channels[channel].len() - trim].to_vec())
}

pub(super) fn evaluate(
    input: &[Vec<f64>; 2],
    output: &[Vec<f64>; 2],
    frequency: f64,
    sample_rate: usize,
) -> Metrics {
    let input_stats = image_stats(input);
    let output_stats = image_stats(output);
    Metrics {
        ipd_error_radians: wrap(
            ipd(output, frequency, sample_rate) - ipd(input, frequency, sample_rate),
        )
        .abs(),
        mid_side_delta_db: (output_stats[0] - input_stats[0]).abs(),
        correlation_delta: (output_stats[1] - input_stats[1]).abs(),
        relation_residual: gram_residual(input, output),
    }
}

pub(super) fn negative_control_residuals() -> [f64; 2] {
    let input = control(ControlKind::Image, 8_000, 124.0, 0.37);
    let collapsed = [input[0].clone(), input[0].clone()];
    let crossfed = [
        input[0]
            .iter()
            .zip(&input[1])
            .map(|(left, right)| left * 0.8 + right * 0.2)
            .collect(),
        input[0]
            .iter()
            .zip(&input[1])
            .map(|(left, right)| left * 0.2 + right * 0.8)
            .collect(),
    ];
    [
        gram_residual(&input, &collapsed),
        gram_residual(&input, &crossfed),
    ]
}

fn ipd(channels: &[Vec<f64>; 2], frequency: f64, sample_rate: usize) -> f64 {
    wrap(
        fitted_phase(&channels[1], frequency, sample_rate)
            - fitted_phase(&channels[0], frequency, sample_rate),
    )
}

fn fitted_phase(samples: &[f64], frequency: f64, sample_rate: usize) -> f64 {
    let [cc, ss, cs, yc, ys] = samples
        .iter()
        .enumerate()
        .fold([0.0; 5], |sum, (index, sample)| {
            let angle = std::f64::consts::TAU * frequency * index as f64 / sample_rate as f64;
            let cosine = angle.cos();
            let sine = angle.sin();
            [
                sum[0] + cosine * cosine,
                sum[1] + sine * sine,
                sum[2] + cosine * sine,
                sum[3] + sample * cosine,
                sum[4] + sample * sine,
            ]
        });
    let determinant = cc * ss - cs * cs;
    let cosine_coefficient = (yc * ss - ys * cs) / determinant;
    let sine_coefficient = (ys * cc - yc * cs) / determinant;
    cosine_coefficient.atan2(sine_coefficient)
}

fn image_stats(channels: &[Vec<f64>; 2]) -> [f64; 2] {
    let [ll, rr, lr] = gram(channels);
    let mid = (ll + rr + 2.0 * lr) * 0.25;
    let side = (ll + rr - 2.0 * lr) * 0.25;
    [
        10.0 * (side.max(f64::MIN_POSITIVE) / mid.max(f64::MIN_POSITIVE)).log10(),
        lr / (ll * rr).sqrt().max(f64::MIN_POSITIVE),
    ]
}

pub(super) fn gram_residual(input: &[Vec<f64>; 2], output: &[Vec<f64>; 2]) -> f64 {
    let normalize = |values: [f64; 3]| {
        let trace = (values[0] + values[1]).max(f64::MIN_POSITIVE);
        [values[0] / trace, values[1] / trace, values[2] / trace]
    };
    normalize(gram(input))
        .into_iter()
        .zip(normalize(gram(output)))
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

pub(super) fn gram(channels: &[Vec<f64>; 2]) -> [f64; 3] {
    channels[0]
        .iter()
        .zip(&channels[1])
        .fold([0.0; 3], |sum, (left, right)| {
            [
                sum[0] + left * left,
                sum[1] + right * right,
                sum[2] + left * right,
            ]
        })
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
