pub(super) const FRAMES: usize = 8_192;
const SAMPLE_RATE: f64 = 48_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Silence,
    Steady,
    Impulse,
    DenseImpulses,
    BoundaryImpulses,
    Chirp,
    Noise,
    Mixed,
}

#[derive(Clone)]
pub(super) struct Control {
    pub kind: Kind,
    pub samples: Vec<f64>,
}

pub(super) fn controls() -> Vec<Control> {
    let sine = |frequency: f64| {
        (0..FRAMES)
            .map(|index| {
                0.5 * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE).sin()
            })
            .collect::<Vec<_>>()
    };
    let mut impulse = vec![0.0; FRAMES];
    impulse[FRAMES / 2] = 1.0;
    let mut dense = vec![0.0; FRAMES];
    dense[FRAMES / 2 - 128] = 1.0;
    dense[FRAMES / 2 + 128] = 0.75;
    let mut boundary = vec![0.0; FRAMES];
    boundary[0] = 1.0;
    boundary[FRAMES - 1] = 0.75;
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    let noise = (0..FRAMES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 40) as f64 / (1_u64 << 24) as f64 - 0.5
        })
        .collect::<Vec<_>>();
    let linear = (0..FRAMES)
        .map(|index| {
            let t = index as f64 / SAMPLE_RATE;
            (std::f64::consts::TAU * (100.0 * t + 0.5 * 20_000.0 * t * t)).sin()
        })
        .collect::<Vec<_>>();
    let exponential = (0..FRAMES)
        .map(|index| {
            let progress = index as f64 / FRAMES as f64;
            let frequency = 55.0 * (8_000.0_f64 / 55.0).powf(progress);
            (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE).sin()
        })
        .collect::<Vec<_>>();
    let low = sine(220.0);
    let high = sine(3_000.0);
    let two_tone = low.iter().zip(&high).map(|(a, b)| a + 0.5 * b).collect();
    let mixed = low
        .iter()
        .zip(&noise)
        .enumerate()
        .map(|(index, (tone, noise))| {
            0.5 * tone + 0.1 * noise + if index == FRAMES / 2 { 0.8 } else { 0.0 }
        })
        .collect();
    vec![
        Control {
            kind: Kind::Silence,
            samples: vec![0.0; FRAMES],
        },
        Control {
            kind: Kind::Steady,
            samples: sine(55.0),
        },
        Control {
            kind: Kind::Steady,
            samples: sine(440.0),
        },
        Control {
            kind: Kind::Steady,
            samples: sine(8_000.0),
        },
        Control {
            kind: Kind::Steady,
            samples: two_tone,
        },
        Control {
            kind: Kind::Impulse,
            samples: impulse,
        },
        Control {
            kind: Kind::DenseImpulses,
            samples: dense,
        },
        Control {
            kind: Kind::BoundaryImpulses,
            samples: boundary,
        },
        Control {
            kind: Kind::Chirp,
            samples: linear,
        },
        Control {
            kind: Kind::Chirp,
            samples: exponential,
        },
        Control {
            kind: Kind::Noise,
            samples: noise,
        },
        Control {
            kind: Kind::Mixed,
            samples: mixed,
        },
    ]
}

pub(super) fn perturbed(samples: &[f64]) -> Vec<f64> {
    let peak = samples.iter().copied().map(f64::abs).fold(0.0, f64::max);
    samples
        .iter()
        .enumerate()
        .map(|(index, sample)| sample + peak * 1.0e-6 * ((index * 17 % 31) as f64 / 15.0 - 1.0))
        .collect()
}
