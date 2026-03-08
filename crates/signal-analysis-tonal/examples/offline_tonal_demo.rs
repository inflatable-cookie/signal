use signal_analysis::AnalysisStage;
use signal_analysis_tonal::{KeyDetector, KeyDetectorConfig, KeyMode, Tonic};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

fn main() {
    let preset = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "c-major".to_string());
    let audio = match preset.as_str() {
        "a-minor" => tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0),
        _ => tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0),
    };

    let mut detector = KeyDetector::new(KeyDetectorConfig::default());
    let result = detector.analyze(&audio);

    match result.key {
        Some(key) => {
            println!("preset={preset}");
            println!("key={} {}", tonic_name(key.tonic), mode_name(key.mode));
            println!("confidence={:.4}", result.confidence.0);
            println!("chroma={:?}", result.chroma);
        }
        None => {
            println!("preset={preset}");
            println!("key=unknown");
            println!("confidence={:.4}", result.confidence.0);
        }
    }
}

fn tonal_mix(sample_rate: u32, freqs: &[f32], seconds: f32) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let mut samples = vec![0.0f32; frames];
    let scale = if freqs.is_empty() {
        0.0
    } else {
        1.0 / freqs.len() as f32
    };

    for (index, sample) in samples.iter_mut().enumerate() {
        let t = index as f32 / sample_rate as f32;
        let mut value = 0.0;
        for freq in freqs {
            value += (core::f32::consts::TAU * *freq * t).sin();
        }
        *sample = value * scale;
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn tonic_name(tonic: Tonic) -> &'static str {
    match tonic {
        Tonic::C => "C",
        Tonic::Cs => "C#",
        Tonic::D => "D",
        Tonic::Ds => "D#",
        Tonic::E => "E",
        Tonic::F => "F",
        Tonic::Fs => "F#",
        Tonic::G => "G",
        Tonic::Gs => "G#",
        Tonic::A => "A",
        Tonic::As => "A#",
        Tonic::B => "B",
    }
}

fn mode_name(mode: KeyMode) -> &'static str {
    match mode {
        KeyMode::Major => "major",
        KeyMode::Minor => "minor",
    }
}
