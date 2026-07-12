use std::path::PathBuf;

const FRAMES: usize = 16_384;

pub(super) struct DevelopmentCase {
    pub channels: Vec<Vec<f64>>,
    pub ratio: f64,
}

pub(super) fn development_cases() -> Vec<DevelopmentCase> {
    [
        ("0000-drums_percussion-000002.wav", 0.75),
        ("0000-drums_percussion-000002.wav", 1.25),
        ("0004-bass-000236.wav", 0.75),
        ("0004-bass-000236.wav", 1.25),
        ("0008-vocals-000010.wav", 0.75),
        ("0008-vocals-000010.wav", 1.25),
        ("0012-pads_sustains-000423.wav", 0.75),
        ("0016-full_mix-000144.wav", 0.75),
        ("0016-full_mix-000144.wav", 1.25),
    ]
    .into_iter()
    .map(|(name, ratio)| DevelopmentCase {
        channels: vec![read_mono(name)],
        ratio,
    })
    .collect()
}

fn read_mono(name: &str) -> Vec<f64> {
    let path = source_root().join(name);
    let mut reader = hound::WavReader::open(&path)
        .unwrap_or_else(|error| panic!("open development source {}: {error}", path.display()));
    let specification = reader.spec();
    assert!(matches!(specification.channels, 1 | 2));
    let samples = match specification.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| f64::from(sample.expect("float development sample")))
            .collect::<Vec<_>>(),
        hound::SampleFormat::Int => {
            let scale = 2.0_f64.powi(i32::from(specification.bits_per_sample) - 1);
            reader
                .samples::<i32>()
                .map(|sample| f64::from(sample.expect("integer development sample")) / scale)
                .collect::<Vec<_>>()
        }
    };
    let channels = usize::from(specification.channels);
    let available = samples.len() / channels;
    assert!(
        available >= FRAMES,
        "development source too short: {}",
        path.display()
    );
    (0..FRAMES)
        .map(|frame| {
            (0..channels)
                .map(|channel| samples[frame * channels + channel])
                .sum::<f64>()
                / channels as f64
        })
        .collect()
}

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-corpus-external-benchmark-pack-fma-broad/sources")
}

pub(super) fn synthetic_control() -> Vec<Vec<f64>> {
    let mut left = vec![0.0; FRAMES];
    let mut right = vec![0.0; FRAMES];
    for index in 0..FRAMES {
        let tone = (std::f64::consts::TAU * 997.0 * index as f64 / 48_000.0).sin();
        left[index] = 0.08 * tone;
        right[index] = 0.06 * tone;
    }
    for event in [2_048, 4_096, 4_224, 8_192, 12_288] {
        for offset in 0..32 {
            let pulse = 0.8 * (-(offset as f64) / 7.0).exp();
            left[event + offset] += pulse;
            right[event + offset] += pulse * 0.72;
        }
    }
    vec![left, right]
}
