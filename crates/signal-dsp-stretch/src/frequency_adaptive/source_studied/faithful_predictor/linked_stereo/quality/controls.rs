pub(super) const SAMPLE_RATE: usize = 8_000;
pub(super) const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];
pub(super) const TONE_FREQUENCIES: [f64; 3] = [124.0, 372.0, 876.0];
pub(super) const PHASE_OFFSETS: [f64; 3] = [0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI];

pub(super) struct TransientControl {
    pub(super) samples: Vec<f64>,
    pub(super) events: Vec<usize>,
}

pub(super) fn tone_control(phase_offset: f64, frequency: f64) -> [Vec<f64>; 2] {
    std::array::from_fn(|channel| {
        (0..SAMPLE_RATE)
            .map(|index| {
                let time = index as f64 / SAMPLE_RATE as f64;
                let phase = if channel == 0 { 0.0 } else { phase_offset };
                0.3 * (std::f64::consts::TAU * frequency * time + phase).sin()
            })
            .collect()
    })
}

pub(super) fn delay_control() -> [Vec<f64>; 2] {
    let left = noise(SAMPLE_RATE, 0x8e6d_27a4_19c5_b301);
    let mut right = vec![0.0; left.len()];
    right[11..].copy_from_slice(&left[..left.len() - 11]);
    [left, right]
}

pub(super) fn correlated_control() -> [Vec<f64>; 2] {
    let left = noise(SAMPLE_RATE, 0x632b_a941_6f37_9d05);
    let independent = noise(SAMPLE_RATE, 0x9f42_1138_a8e7_c2d1);
    let right = left
        .iter()
        .zip(independent)
        .map(|(shared, other)| shared * 0.68 + other * 0.22)
        .collect();
    [left, right]
}

pub(super) fn decorrelated_control() -> [Vec<f64>; 2] {
    [
        noise(SAMPLE_RATE, 0x41cd_8e27_654a_f903),
        noise(SAMPLE_RATE, 0xb329_0d7f_18e4_56ac),
    ]
}

pub(super) fn isolated_transient_control() -> TransientControl {
    let events = vec![SAMPLE_RATE / 2];
    TransientControl {
        samples: impulses(&events, &[1.0]),
        events,
    }
}

pub(super) fn dense_transient_control() -> TransientControl {
    let events = vec![2_000, 3_000, 4_000, 5_000, 6_000];
    TransientControl {
        samples: impulses(&events, &[1.0, 0.82, 0.94, 0.76, 0.88]),
        events,
    }
}

fn impulses(events: &[usize], amplitudes: &[f64]) -> Vec<f64> {
    let mut samples = vec![0.0; SAMPLE_RATE];
    for (&event, &amplitude) in events.iter().zip(amplitudes) {
        samples[event] = amplitude;
    }
    samples
}

fn noise(length: usize, seed: u64) -> Vec<f64> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (((state >> 11) as f64 / ((1_u64 << 53) as f64)) * 2.0 - 1.0) * 0.35
        })
        .collect()
}
