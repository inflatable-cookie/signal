//! Tonal analysis surfaces for Signal.
//!
//! The crate currently exposes offline whole-track key detection driven by
//! STFT-based chroma accumulation.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_tonal::{KeyDetector, KeyDetectorConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut detector = KeyDetector::new(KeyDetectorConfig::default());
//! let result = detector.analyze(&audio);
//!
//! assert_eq!(detector.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.chroma.len(), 12);
//! ```

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_dsp_spectral::{Stft, StftConfig};
use signal_primitives::{AudioBuffer, FrameCount, Sample, SampleRate};

/// Tonal mode of a detected key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

/// Pitch-class tonic for a detected key.
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

/// Whole-track key estimate.
///
/// This is only present when the best-scoring profile correlation is positive.
/// Callers should still pair it with [`TonalAnalysisResult::confidence`] before
/// presenting the key as definitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: Tonic,
    pub mode: KeyMode,
}

/// Correlation profile family used for key scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyProfile {
    Krumhansl,
    Temperley,
}

/// Analysis depth profile controlling the speed / accuracy trade-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisProfile {
    /// 30-second centre segment, 2048-point FFT, no phase computation.
    /// Suitable for rapid library scanning.
    Low,
    /// 60-second centre segment, 4096-point FFT, no phase computation.
    /// Balanced accuracy and performance for interactive use.
    Medium,
    /// Full track, 4096-point FFT, full phase computation.
    /// Maximum accuracy for detailed analysis.
    High,
}

/// Configuration for the key detector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyDetectorConfig {
    pub stft: StftConfig,
    pub profile: KeyProfile,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed.
    pub analysis_duration_seconds: Option<u32>,
}

impl KeyDetectorConfig {
    /// Rapid scanning profile — 30-second centre segment, 4096-point FFT.
    ///
    /// Uses a 4096-point window (rather than the 8192 of higher tiers) for
    /// speed; bass-register pitch-class mapping is less precise but
    /// acceptable for quick scanning.
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(4096),
                hop_size: FrameCount(2048),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            analysis_duration_seconds: Some(30),
        }
    }

    /// Balanced profile — 60-second centre segment, 8192-point FFT.
    ///
    /// The 8192-point window gives ~5.4–5.9 Hz bin spacing (depending on
    /// sample rate), ensuring every semitone in octave 2 and above has at
    /// least one FFT bin mapping to it.  This is critical when combined
    /// with `1/f` chroma weighting, which amplifies bass-register bins
    /// that would otherwise be dwarfed by higher octaves.
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(8192),
                hop_size: FrameCount(4096),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            analysis_duration_seconds: Some(60),
        }
    }

    /// Full-accuracy profile — entire track, 8192-point FFT.
    pub fn high() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(8192),
                hop_size: FrameCount(4096),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            analysis_duration_seconds: None,
        }
    }
}

impl Default for KeyDetectorConfig {
    fn default() -> Self {
        Self::high()
    }
}

/// Whole-track tonal summary returned by [`KeyDetector`].
///
/// Practical integration order:
/// 1. Read `key` for the best whole-track tonic and mode hypothesis.
/// 2. Read `confidence` before treating that key as reliable; low values usually
///    mean the leading and runner-up profile correlations are close.
/// 3. Use `chroma` and `correlations` when a UI or downstream tool needs the
///    supporting evidence rather than just the winning label.
#[derive(Clone, Debug, PartialEq)]
pub struct TonalAnalysisResult {
    pub key: Option<Key>,
    pub confidence: Confidence,
    pub chroma: [f32; 12],
    pub correlations: [f32; 24],
}

/// Offline detector for global key and chroma summaries.
#[derive(Debug, Default)]
pub struct KeyDetector {
    config: KeyDetectorConfig,
}

impl KeyDetector {
    /// Create a key detector with the provided config.
    pub fn new(config: KeyDetectorConfig) -> Self {
        Self { config }
    }

    /// Return the current detector config.
    pub fn config(&self) -> KeyDetectorConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        // Truncate to configured duration from the centre of the track.
        let mono_samples = if let Some(duration) = self.config.analysis_duration_seconds {
            let max_samples = duration as usize * sample_rate.0 as usize;
            if mono_samples.len() > max_samples {
                let start = (mono_samples.len() - max_samples) / 2;
                &mono_samples[start..start + max_samples]
            } else {
                mono_samples
            }
        } else {
            mono_samples
        };

        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let chroma = spectrogram.chroma();

        score_chroma(chroma, self.config.profile)
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

/// Score a chroma vector against key profiles, returning the best-matching
/// key with its confidence and full correlation set.
fn score_chroma(chroma: [f32; 12], profile: KeyProfile) -> TonalAnalysisResult {
    let correlations = correlate_profiles(chroma, profile);

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
        .fold(f32::NEG_INFINITY, |best, score| best.max(score));

    let key = if best_score > second_best {
        Some(key_from_index(best_index))
    } else {
        None
    };

    let confidence = if best_score > second_best && best_score != 0.0 {
        Confidence::new(((best_score - second_best) / best_score.abs()).max(0.0))
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

/// Compute Pearson correlation coefficients between the observed chroma and
/// all 24 rotated key profiles.
///
/// Pearson correlation centres both vectors around their means before
/// computing the cosine of the angle between them.  This is the standard
/// approach in the Krumhansl-Schmuckler key-finding algorithm and is
/// substantially better at distinguishing relative major/minor keys (e.g.
/// A minor vs C major) than a raw dot product, because it measures the
/// *shape* of the distribution rather than being dominated by which pitch
/// classes carry the most absolute energy.
fn correlate_profiles(chroma: [f32; 12], profile: KeyProfile) -> [f32; 24] {
    let (major_profile, minor_profile) = match profile {
        KeyProfile::Krumhansl => (KRUMHANSL_MAJOR, KRUMHANSL_MINOR),
        KeyProfile::Temperley => (TEMPERLEY_MAJOR, TEMPERLEY_MINOR),
    };

    let mut correlations = [0.0; 24];
    for tonic in 0..12 {
        correlations[tonic] = pearson(chroma, rotate_profile(&major_profile, tonic));
        correlations[12 + tonic] = pearson(chroma, rotate_profile(&minor_profile, tonic));
    }
    correlations
}

fn rotate_profile(profile: &[f32; 12], tonic: usize) -> [f32; 12] {
    let mut rotated = [0.0; 12];
    for (index, value) in profile.iter().copied().enumerate() {
        rotated[(index + tonic) % 12] = value;
    }
    rotated
}

/// Pearson product-moment correlation coefficient between two 12-element
/// vectors.  Returns 0.0 when either vector has zero variance (e.g. all
/// values identical or all zeros after mean-centring).
fn pearson(x: [f32; 12], y: [f32; 12]) -> f32 {
    let n = 12.0f32;
    let x_mean = x.iter().copied().sum::<f32>() / n;
    let y_mean = y.iter().copied().sum::<f32>() / n;

    let mut numerator = 0.0f32;
    let mut x_var = 0.0f32;
    let mut y_var = 0.0f32;

    for i in 0..12 {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        numerator += dx * dy;
        x_var += dx * dx;
        y_var += dy * dy;
    }

    let denominator = (x_var * y_var).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
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

    #[test]
    fn low_profile_still_detects_key() {
        let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::low());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::C);
        assert_eq!(result.key.unwrap().mode, KeyMode::Major);
    }

    #[test]
    fn medium_profile_still_detects_key() {
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::A);
        assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
    }

    #[test]
    fn pearson_distinguishes_relative_major_minor() {
        // A minor chord (A-C-E) should be detected as A minor, not C major,
        // even though they share the same pitch classes.
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(key.tonic, Tonic::A);
        assert_eq!(key.mode, KeyMode::Minor);

        // The A minor correlation (index 12+9=21) should exceed C major (index 0).
        assert!(
            result.correlations[21] > result.correlations[0],
            "A minor correlation ({}) should exceed C major ({})",
            result.correlations[21],
            result.correlations[0],
        );
    }

    #[test]
    fn b_minor_bass_detected_correctly_at_44100() {
        // B minor triad rooted in bass register (B2-D4-F#4), at 44100 Hz.
        // With a 4096-point FFT, B2 (123.47 Hz) falls between bins that map
        // to A# and C — no bin maps to B.  The 8192-point FFT used by
        // medium/high profiles fixes this.
        let audio = tonal_mix(44_100, &[123.47, 293.66, 369.99], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(
            key.tonic,
            Tonic::B,
            "Expected B but got {:?}; chroma = {:?}",
            key.tonic,
            result.chroma,
        );
        assert_eq!(key.mode, KeyMode::Minor);
    }

    #[test]
    fn b_minor_bass_detected_correctly_at_48000() {
        let audio = tonal_mix(48_000, &[123.47, 293.66, 369.99], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(
            key.tonic,
            Tonic::B,
            "Expected B but got {:?}; chroma = {:?}",
            key.tonic,
            result.chroma,
        );
        assert_eq!(key.mode, KeyMode::Minor);
    }
}
