use std::path::Path;

pub(in crate::frequency_adaptive) struct MatchedCandidate {
    pub identity: String,
    pub samples: Vec<f64>,
    pub gain: f64,
}

pub(in crate::frequency_adaptive) struct MatchedGroup {
    pub source: Vec<f64>,
    pub candidates: Vec<MatchedCandidate>,
}

pub(in crate::frequency_adaptive) fn read_mono(path: &Path) -> Vec<f64> {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let specification = reader.spec();
    assert_eq!(specification.sample_rate, 44_100, "{}", path.display());
    assert!(matches!(specification.channels, 1 | 2));
    let samples = match specification.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| f64::from(sample.expect("float sample")))
            .collect::<Vec<_>>(),
        hound::SampleFormat::Int => {
            let scale = 2.0_f64.powi(i32::from(specification.bits_per_sample) - 1);
            reader
                .samples::<i32>()
                .map(|sample| f64::from(sample.expect("integer sample")) / scale)
                .collect::<Vec<_>>()
        }
    };
    let channels = usize::from(specification.channels);
    (0..samples.len() / channels)
        .map(|frame| {
            (0..channels)
                .map(|channel| samples[frame * channels + channel])
                .sum::<f64>()
                / channels as f64
        })
        .collect()
}

pub(in crate::frequency_adaptive) fn level_match(
    source: &[f64],
    candidates: Vec<(String, Vec<f64>)>,
) -> MatchedGroup {
    let target = candidates
        .iter()
        .map(|(_, samples)| rms(samples))
        .chain(std::iter::once(rms(source)))
        .fold(f64::INFINITY, f64::min)
        .max(1.0e-9);
    let source_gain = safe_gain(source, target);
    let source = source.iter().map(|sample| sample * source_gain).collect();
    let candidates = candidates
        .into_iter()
        .map(|(identity, samples)| {
            let gain = safe_gain(&samples, target);
            MatchedCandidate {
                identity,
                samples: samples.into_iter().map(|sample| sample * gain).collect(),
                gain,
            }
        })
        .collect();
    MatchedGroup { source, candidates }
}

pub(in crate::frequency_adaptive) fn write_mono(path: &Path, sample_rate: u32, samples: &[f64]) {
    let specification = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, specification)
        .unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
    for sample in samples {
        writer.write_sample(*sample as f32).expect("write sample");
    }
    writer.finalize().expect("finalize wav");
}

fn safe_gain(samples: &[f64], target_rms: f64) -> f64 {
    let rms_gain = target_rms / rms(samples).max(1.0e-12);
    let peak_gain = 0.95 / peak(samples).max(1.0e-12);
    rms_gain.min(peak_gain)
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
