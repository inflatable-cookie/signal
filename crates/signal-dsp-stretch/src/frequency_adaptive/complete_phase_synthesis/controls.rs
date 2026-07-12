pub(super) struct Control {
    pub channels: Vec<Vec<f64>>,
    pub events: Vec<usize>,
}

pub(super) fn controls() -> Vec<Control> {
    vec![
        control(false, false),
        control(true, false),
        control(true, true),
    ]
}

fn control(stretched: bool, boundary: bool) -> Control {
    let frames = 16_384;
    let events = if boundary {
        Vec::new()
    } else {
        vec![2_048, 4_096, 4_224, 8_192, 12_288]
    };
    let mut left = vec![0.0; frames];
    let mut right = vec![0.0; frames];
    for index in 0..frames {
        let tone = (std::f64::consts::TAU * 997.0 * index as f64 / 48_000.0).sin();
        left[index] = 0.08 * tone;
        right[index] =
            0.06 * tone + 0.02 * (std::f64::consts::TAU * 1499.0 * index as f64 / 48_000.0).sin();
    }
    for event in &events {
        for offset in 0..32 {
            let pulse = (-(offset as f64) / 7.0).exp() * if stretched { 8.0 } else { 7.0 };
            left[event + offset] += pulse;
            right[event + offset] += pulse * 0.72;
        }
    }
    if boundary {
        left[0] += 1.0;
        right[0] += 0.7;
        left[frames - 1] += 0.8;
        right[frames - 1] += 0.6;
    }
    Control {
        channels: vec![left, right],
        events,
    }
}

pub(super) fn peak_index(samples: &[f64], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

pub(super) fn tone_frequency(samples: &[f64], sample_rate: f64) -> f64 {
    let start = samples.len() * 35 / 100;
    let end = samples.len() * 47 / 100;
    let mut best = (0.0, 0.0);
    for frequency in 980..=1_014 {
        let omega = std::f64::consts::TAU * frequency as f64 / sample_rate;
        let coefficient =
            samples[start..end]
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |sum, (index, sample)| {
                    (
                        sum.0 + sample * (omega * index as f64).cos(),
                        sum.1 - sample * (omega * index as f64).sin(),
                    )
                });
        let power = coefficient.0 * coefficient.0 + coefficient.1 * coefficient.1;
        if power > best.1 {
            best = (frequency as f64, power);
        }
    }
    best.0
}
