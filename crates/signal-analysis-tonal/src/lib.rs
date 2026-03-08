//! Tonal analysis surfaces for Signal.

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_dsp_spectral::{Stft, StftConfig};
use signal_primitives::{AudioBuffer, Sample, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tonic {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: Tonic,
    pub mode: KeyMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyProfile {
    Krumhansl,
    Temperley,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyDetectorConfig {
    pub stft: StftConfig,
    pub profile: KeyProfile,
}

impl Default for KeyDetectorConfig {
    fn default() -> Self {
        Self {
            stft: StftConfig::new(4096, 2048),
            profile: KeyProfile::Krumhansl,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TonalAnalysisResult {
    pub key: Option<Key>,
    pub confidence: Confidence,
    pub chroma: [f32; 12],
    pub correlations: [f32; 24],
}

#[derive(Debug, Default)]
pub struct KeyDetector {
    config: KeyDetectorConfig,
}

impl KeyDetector {
    pub fn new(config: KeyDetectorConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> KeyDetectorConfig {
        self.config
    }

    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let chroma = spectrogram.chroma();
        let correlations = correlate_profiles(chroma, self.config.profile);

        let (best_index, best_score) = correlations
            .iter()
            .copied()
            .enumerate()
            .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
            .unwrap_or((0, 0.0));

        let second_best = correlations
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, _)| *index != best_index)
            .map(|(_, score)| score)
            .fold(0.0f32, |best, score| best.max(score));

        let key = if best_score > 0.0 {
            Some(key_from_index(best_index))
        } else {
            None
        };

        let confidence = if best_score > 0.0 {
            Confidence::new(((best_score - second_best) / best_score).max(0.0))
        } else {
            Confidence::new(0.0)
        };

        TonalAnalysisResult {
            key,
            confidence,
            chroma,
            correlations,
        }
    }
}

impl AnalysisStage<TonalAnalysisResult> for KeyDetector {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> TonalAnalysisResult {
        self.analyze_mono(audio.sample_rate(), &audio.to_mono())
    }
}

const KRUMHANSL_MAJOR: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const KRUMHANSL_MINOR: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];
const TEMPERLEY_MAJOR: [f32; 12] = [5.0, 2.0, 3.5, 2.0, 4.5, 4.0, 2.0, 4.5, 2.0, 3.5, 1.5, 4.0];
const TEMPERLEY_MINOR: [f32; 12] = [5.0, 2.0, 3.5, 4.5, 2.0, 4.0, 2.0, 4.5, 3.5, 2.0, 1.5, 4.0];

fn correlate_profiles(chroma: [f32; 12], profile: KeyProfile) -> [f32; 24] {
    let (major_profile, minor_profile) = match profile {
        KeyProfile::Krumhansl => (KRUMHANSL_MAJOR, KRUMHANSL_MINOR),
        KeyProfile::Temperley => (TEMPERLEY_MAJOR, TEMPERLEY_MINOR),
    };

    let major_profile = normalize_profile(major_profile);
    let minor_profile = normalize_profile(minor_profile);

    let mut correlations = [0.0; 24];
    for tonic in 0..12 {
        correlations[tonic] = dot(chroma, rotate_profile(&major_profile, tonic));
        correlations[12 + tonic] = dot(chroma, rotate_profile(&minor_profile, tonic));
    }
    correlations
}

fn normalize_profile(profile: [f32; 12]) -> [f32; 12] {
    let sum = profile.iter().copied().sum::<f32>();
    if sum == 0.0 {
        return profile;
    }

    let mut normalized = [0.0; 12];
    for (index, value) in profile.into_iter().enumerate() {
        normalized[index] = value / sum;
    }
    normalized
}

fn rotate_profile(profile: &[f32; 12], tonic: usize) -> [f32; 12] {
    let mut rotated = [0.0; 12];
    for (index, value) in profile.iter().copied().enumerate() {
        rotated[(index + tonic) % 12] = value;
    }
    rotated
}

fn dot(lhs: [f32; 12], rhs: [f32; 12]) -> f32 {
    lhs.into_iter().zip(rhs).map(|(a, b)| a * b).sum()
}

fn key_from_index(index: usize) -> Key {
    let tonic = tonic_from_index(index % 12);
    let mode = if index < 12 {
        KeyMode::Major
    } else {
        KeyMode::Minor
    };
    Key { tonic, mode }
}

fn tonic_from_index(index: usize) -> Tonic {
    match index % 12 {
        0 => Tonic::C,
        1 => Tonic::Cs,
        2 => Tonic::D,
        3 => Tonic::Ds,
        4 => Tonic::E,
        5 => Tonic::F,
        6 => Tonic::Fs,
        7 => Tonic::G,
        8 => Tonic::Gs,
        9 => Tonic::A,
        10 => Tonic::As,
        _ => Tonic::B,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyDetector, KeyDetectorConfig, KeyMode, Tonic};
    use signal_analysis::AnalysisStage;
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

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

    #[test]
    fn key_detector_finds_c_major_triad() {
        let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::C);
        assert_eq!(result.key.unwrap().mode, KeyMode::Major);
        assert!(result.confidence.0 > 0.01);
    }

    #[test]
    fn key_detector_finds_a_minor_triad() {
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::A);
        assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
        assert!(result.confidence.0 > 0.001);
    }
}
