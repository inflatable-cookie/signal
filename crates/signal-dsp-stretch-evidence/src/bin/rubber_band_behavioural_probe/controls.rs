use std::f32::consts::TAU;

const FRAMES: usize = 16_384;
const RATE: f32 = 48_000.0;

pub(super) struct Control {
    pub id: &'static str,
    pub family: &'static str,
    pub channels: usize,
    pub samples: Vec<f32>,
    pub events: Vec<usize>,
}

impl Control {
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels
    }
}

pub(super) fn controls() -> Vec<Control> {
    let mut result = vec![
        mono("bass-tone", "tonal", sine(55.0), vec![]),
        mono("mid-tone", "tonal", sine(440.0), vec![]),
        mono(
            "two-tone",
            "tonal",
            combine(&sine(110.0), &sine(4_000.0), 0.6, 0.3),
            vec![],
        ),
        mono("linear-chirp", "moving-tonal", chirp(), vec![]),
        mono(
            "hard-impulse",
            "event",
            impulses(&[(FRAMES / 2, 1.0)]),
            vec![FRAMES / 2],
        ),
        mono("soft-onset", "event", soft_onset(), vec![FRAMES / 2]),
        mono(
            "dense-impulses",
            "dense-event",
            impulses(&[(FRAMES / 2 - 128, 1.0), (FRAMES / 2 + 128, 0.8)]),
            vec![FRAMES / 2 - 128, FRAMES / 2 + 128],
        ),
        mono(
            "boundary-impulses",
            "boundary-event",
            impulses(&[(0, 1.0), (FRAMES - 1, -0.8)]),
            vec![0, FRAMES - 1],
        ),
        mono(
            "tonal-impulse",
            "mixed",
            combine(&sine(220.0), &impulses(&[(FRAMES / 2, 1.0)]), 0.4, 0.8),
            vec![FRAMES / 2],
        ),
        mono("noise", "noise", noise(), vec![]),
        mono(
            "complex-mix",
            "mixed",
            complex_mix(),
            vec![FRAMES / 3, 2 * FRAMES / 3],
        ),
        mono("silence", "silence", vec![0.0; FRAMES], vec![]),
    ];
    result.extend(stereo_controls());
    result
}

fn mono(id: &'static str, family: &'static str, samples: Vec<f32>, events: Vec<usize>) -> Control {
    Control {
        id,
        family,
        channels: 1,
        samples,
        events,
    }
}

fn sine(frequency: f32) -> Vec<f32> {
    (0..FRAMES)
        .map(|index| 0.5 * (TAU * frequency * index as f32 / RATE).sin())
        .collect()
}

fn chirp() -> Vec<f32> {
    let mut phase = 0.0;
    (0..FRAMES)
        .map(|index| {
            let frequency = 80.0 + 8_000.0 * index as f32 / (FRAMES - 1) as f32;
            phase += TAU * frequency / RATE;
            0.5 * phase.sin()
        })
        .collect()
}

fn impulses(values: &[(usize, f32)]) -> Vec<f32> {
    let mut samples = vec![0.0; FRAMES];
    for (frame, value) in values {
        samples[*frame] = *value;
    }
    samples
}

fn soft_onset() -> Vec<f32> {
    let start = FRAMES / 2;
    (0..FRAMES)
        .map(|index| {
            if index < start {
                0.0
            } else {
                let attack = ((index - start) as f32 / 512.0).min(1.0);
                let envelope = 0.5 - 0.5 * (std::f32::consts::PI * attack).cos();
                0.5 * envelope * (TAU * 440.0 * index as f32 / RATE).sin()
            }
        })
        .collect()
}

fn noise() -> Vec<f32> {
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    (0..FRAMES)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 40) as f32 / (1_u64 << 24) as f32 - 0.5) * 0.5
        })
        .collect()
}

fn combine(a: &[f32], b: &[f32], a_gain: f32, b_gain: f32) -> Vec<f32> {
    a.iter()
        .zip(b)
        .map(|(a, b)| a * a_gain + b * b_gain)
        .collect()
}

fn complex_mix() -> Vec<f32> {
    let mut samples = combine(&sine(55.0), &chirp(), 0.45, 0.2);
    let onset = soft_onset();
    for (sample, onset) in samples.iter_mut().zip(onset) {
        *sample += onset * 0.3;
    }
    samples[FRAMES / 3] += 0.8;
    samples[2 * FRAMES / 3] -= 0.65;
    samples
}

fn stereo_controls() -> Vec<Control> {
    let hard = impulses(&[(FRAMES / 2, 1.0)]);
    let tonal = sine(220.0);
    let mixed = combine(&tonal, &hard, 0.4, 0.8);
    let side = impulses(&[(FRAMES / 3, 0.8), (2 * FRAMES / 3, -0.8)]);
    vec![
        stereo(
            "stereo-linked-impulse",
            "stereo-event",
            &hard,
            &hard,
            vec![FRAMES / 2],
        ),
        stereo(
            "stereo-unequal-mixed",
            "stereo-mixed",
            &mixed,
            &tonal,
            vec![FRAMES / 2],
        ),
        stereo(
            "stereo-centre-side",
            "stereo-image",
            &combine(&tonal, &side, 0.4, 0.5),
            &combine(&tonal, &side, 0.4, -0.5),
            vec![FRAMES / 3, 2 * FRAMES / 3],
        ),
        stereo(
            "stereo-antiphase",
            "stereo-phase",
            &tonal,
            &tonal.iter().map(|sample| -*sample).collect::<Vec<_>>(),
            vec![],
        ),
    ]
}

fn stereo(
    id: &'static str,
    family: &'static str,
    left: &[f32],
    right: &[f32],
    events: Vec<usize>,
) -> Control {
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for index in 0..FRAMES {
        samples.push(left[index]);
        samples.push(right[index]);
    }
    Control {
        id,
        family,
        channels: 2,
        samples,
        events,
    }
}
