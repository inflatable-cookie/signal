//! Character analysis surfaces for Signal.
//!
//! The crate computes timbral and energy descriptors from mono audio:
//! spectral centroid (brightness), onset density (busyness), zero-crossing
//! rate (noisiness), RMS energy (signal power), peak amplitude, transient
//! density, sustain ratio, and dynamic range.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
//! let result = analyzer.analyze(&audio);
//!
//! assert_eq!(analyzer.mode(), signal_analysis::AnalysisMode::Offline);
//! assert!(result.rms_energy >= 0.0 && result.rms_energy <= 1.0);
//! ```

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_dsp_spectral::{Stft, StftConfig};
use signal_primitives::{AudioBuffer, FrameCount, Sample, SampleRate};

/// Configuration for the offline character analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterAnalyzerConfig {
    pub stft: StftConfig,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed.
    pub analysis_duration_seconds: Option<u32>,
    /// Onset detection threshold relative to the mean spectral flux.
    /// A peak in the spectral flux envelope is counted as an onset when
    /// it exceeds `onset_threshold` times the mean flux.
    pub onset_threshold: f32,
}

impl CharacterAnalyzerConfig {
    /// Quick scanning profile — 30-second centre segment.
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(1024),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            analysis_duration_seconds: Some(30),
            onset_threshold: 1.5,
        }
    }

    /// Balanced profile — 60-second centre segment.
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(2048),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            analysis_duration_seconds: Some(60),
            onset_threshold: 1.5,
        }
    }

    /// Full-accuracy profile — entire track.
    pub fn high() -> Self {
        Self::default()
    }
}

impl Default for CharacterAnalyzerConfig {
    fn default() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(2048),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            analysis_duration_seconds: None,
            onset_threshold: 1.5,
        }
    }
}

/// Character analysis result.
///
/// Practical integration order:
/// 1. Read `spectral_centroid_hz` as a brightness measure (higher = brighter
///    timbre).
/// 2. Read `onset_density` for rhythmic busyness (onsets per second).
/// 3. Read `zero_crossing_rate_hz` as a noisiness/brightness proxy.
/// 4. Read `rms_energy` for overall signal power (0.0 = silence, ~0.707 =
///    full-scale sine).
/// 5. Read `peak_amplitude` for the loudest sample value.
/// 6. Read `transient_density` for waveform transient events per second.
/// 7. Read `sustain_ratio` for the fraction of the signal above the silence
///    threshold.
/// 8. Read `dynamic_range` (peak minus RMS) for crest headroom.
/// 9. Read `confidence` to decide whether the analyzed segment was long enough
///    to produce stable descriptors.
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterAnalysisResult {
    /// Median spectral centroid across STFT frames, in Hz.
    pub spectral_centroid_hz: f32,
    /// Onset density in onsets per second, derived from spectral flux peaks.
    pub onset_density: f32,
    /// Mean zero-crossing rate in Hz.
    pub zero_crossing_rate_hz: f32,
    /// RMS energy of the mono signal, clamped to 0.0..=1.0.
    pub rms_energy: f32,
    /// Peak absolute sample amplitude (0.0 = silence, 1.0 = full scale).
    pub peak_amplitude: f32,
    /// Transient density in events per second, counted via sample-slope
    /// detection with minimum spacing of `sample_rate / 20` samples.
    /// Captures sharp waveform transients (snare hits, plucks) rather than
    /// spectral-flux onsets.
    pub transient_density: f32,
    /// Fraction of samples whose absolute value meets or exceeds 0.02 (the
    /// silence threshold).  Ranges from 0.0 (silence) to 1.0 (sustained
    /// signal throughout).
    pub sustain_ratio: f32,
    /// Crest headroom: `peak_amplitude - rms_energy`, clamped to 0.0 or
    /// above.
    pub dynamic_range: f32,
    pub confidence: Confidence,
}

/// Offline character analyzer.
#[derive(Debug, Default)]
pub struct CharacterAnalyzer {
    config: CharacterAnalyzerConfig,
}

impl CharacterAnalyzer {
    /// Create a character analyzer with the provided config.
    pub fn new(config: CharacterAnalyzerConfig) -> Self {
        Self { config }
    }

    /// Return the current analyzer config.
    pub fn config(&self) -> CharacterAnalyzerConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> CharacterAnalysisResult {
        if mono_samples.is_empty() || sample_rate.0 == 0 {
            return CharacterAnalysisResult {
                spectral_centroid_hz: 0.0,
                onset_density: 0.0,
                zero_crossing_rate_hz: 0.0,
                rms_energy: 0.0,
                peak_amplitude: 0.0,
                transient_density: 0.0,
                sustain_ratio: 0.0,
                dynamic_range: 0.0,
                confidence: Confidence::new(0.0),
            };
        }

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

        let duration_seconds = mono_samples.len() as f32 / sample_rate.0 as f32;

        // --- Spectral centroid via STFT ---
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let spectral_centroid_hz = spectrogram.spectral_centroid();

        // --- Onset density via spectral flux peak counting ---
        let onset_density =
            compute_onset_density(&spectrogram, self.config.onset_threshold, duration_seconds);

        // --- Zero-crossing rate (direct time-domain computation) ---
        let zero_crossing_rate_hz = compute_zcr(mono_samples, sample_rate);

        // --- RMS energy (direct time-domain computation) ---
        let rms_energy = compute_rms(mono_samples);

        // --- Peak amplitude ---
        let peak_amplitude = compute_peak_amplitude(mono_samples);

        // --- Transient density (sample-slope based) ---
        let transient_density =
            compute_transient_density(mono_samples, sample_rate, duration_seconds);

        // --- Sustain ratio ---
        let sustain_ratio = compute_sustain_ratio(mono_samples);

        // --- Dynamic range ---
        let dynamic_range = (peak_amplitude - rms_energy).max(0.0);

        // --- Confidence ---
        let confidence = character_confidence(sample_rate, mono_samples.len());

        CharacterAnalysisResult {
            spectral_centroid_hz,
            onset_density,
            zero_crossing_rate_hz,
            rms_energy,
            peak_amplitude,
            transient_density,
            sustain_ratio,
            dynamic_range,
            confidence,
        }
    }
}

impl AnalysisStage<CharacterAnalysisResult> for CharacterAnalyzer {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> CharacterAnalysisResult {
        self.analyze_mono(audio.sample_rate(), &audio.to_mono())
    }
}

// ---- Private helpers ----

/// Count onsets per second from spectral flux peaks.
///
/// Spectral flux is the half-wave rectified frame-to-frame difference in
/// magnitude spectra.  Peaks above `threshold_multiplier × mean_flux` are
/// counted as onsets.
fn compute_onset_density(
    spectrogram: &signal_dsp_spectral::Spectrogram,
    threshold_multiplier: f32,
    duration_seconds: f32,
) -> f32 {
    if spectrogram.is_empty() || duration_seconds <= 0.0 {
        return 0.0;
    }

    // Compute half-wave rectified spectral flux.
    let mut flux = Vec::with_capacity(spectrogram.frames.len());
    let mut previous: Option<&[f32]> = None;

    for frame in &spectrogram.frames {
        let current = frame.magnitudes.as_slice();
        let value = if let Some(last) = previous {
            current
                .iter()
                .zip(last.iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum()
        } else {
            0.0
        };
        flux.push(value);
        previous = Some(current);
    }

    if flux.is_empty() {
        return 0.0;
    }

    let mean_flux: f32 = flux.iter().copied().sum::<f32>() / flux.len() as f32;
    let threshold = mean_flux * threshold_multiplier;

    // Count peaks above threshold (value > both neighbours).
    let mut onset_count = 0u32;
    for i in 1..flux.len().saturating_sub(1) {
        if flux[i] > threshold && flux[i] > flux[i - 1] && flux[i] > flux[i + 1] {
            onset_count += 1;
        }
    }

    onset_count as f32 / duration_seconds
}

/// Compute zero-crossing rate in Hz.
fn compute_zcr(samples: &[Sample], sample_rate: SampleRate) -> f32 {
    if samples.len() < 2 || sample_rate.0 == 0 {
        return 0.0;
    }

    let mut crossings = 0u64;
    for pair in samples.windows(2) {
        if (pair[0] >= 0.0) != (pair[1] >= 0.0) {
            crossings += 1;
        }
    }

    let duration_seconds = samples.len() as f64 / sample_rate.0 as f64;
    (crossings as f64 / duration_seconds) as f32
}

/// Compute RMS energy, clamped to 0.0..=1.0.
fn compute_rms(samples: &[Sample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    let rms = (sum_squares / samples.len() as f64).sqrt() as f32;
    rms.clamp(0.0, 1.0)
}

/// Peak absolute sample amplitude.
fn compute_peak_amplitude(samples: &[Sample]) -> f32 {
    samples.iter().fold(0.0f32, |peak, &s| peak.max(s.abs()))
}

/// Transient density via sample-slope detection.
///
/// Counts transitions where the absolute difference between consecutive
/// samples exceeds `SLOPE_THRESHOLD`, with a minimum spacing of
/// `sample_rate / 20` samples between counted events.  This mirrors the C++
/// engine's `derive_transient_density` algorithm so that downstream semantic
/// scoring thresholds remain valid.
fn compute_transient_density(
    samples: &[Sample],
    sample_rate: SampleRate,
    duration_seconds: f32,
) -> f32 {
    const SLOPE_THRESHOLD: f32 = 0.12;
    const MIN_SPACING_DIVISOR: u32 = 20;

    if samples.len() < 2 || sample_rate.0 == 0 || duration_seconds <= 0.0 {
        return 0.0;
    }

    let min_spacing = (sample_rate.0 / MIN_SPACING_DIVISOR).max(1) as usize;
    let mut count = 0u32;
    let mut samples_since_last = min_spacing;

    for i in 1..samples.len() {
        let slope = (samples[i] - samples[i - 1]).abs();
        samples_since_last += 1;
        if slope >= SLOPE_THRESHOLD && samples_since_last >= min_spacing {
            count += 1;
            samples_since_last = 0;
        }
    }

    count as f32 / duration_seconds
}

/// Sustain ratio: fraction of samples with absolute value >= 0.02.
fn compute_sustain_ratio(samples: &[Sample]) -> f32 {
    const SILENCE_THRESHOLD: f32 = 0.02;

    if samples.is_empty() {
        return 0.0;
    }

    let sustained = samples
        .iter()
        .filter(|&&s| s.abs() >= SILENCE_THRESHOLD)
        .count();

    sustained as f32 / samples.len() as f32
}

/// Confidence ramps linearly from 0.0 at 0 seconds to 1.0 at 10+ seconds.
fn character_confidence(sample_rate: SampleRate, sample_count: usize) -> Confidence {
    if sample_rate.0 == 0 {
        return Confidence::new(0.0);
    }
    let duration_seconds = sample_count as f32 / sample_rate.0 as f32;
    Confidence::new(duration_seconds / 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine_audio(frequency: f32, duration_seconds: f32, sample_rate: u32) -> AudioBuffer {
        let count = (duration_seconds * sample_rate as f32) as usize;
        let mut data = vec![0.0f32; count];
        for (i, sample) in data.iter_mut().enumerate() {
            let t = i as f32 / sample_rate as f32;
            *sample = (core::f32::consts::TAU * frequency * t).sin();
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, data)
    }

    #[test]
    fn centroid_near_1khz_for_sine() {
        let audio = sine_audio(1000.0, 2.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(
            result.spectral_centroid_hz > 800.0 && result.spectral_centroid_hz < 1200.0,
            "centroid was {}",
            result.spectral_centroid_hz,
        );
    }

    #[test]
    fn rms_energy_near_expected_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        // Full-scale sine RMS is ~0.707.
        assert!(
            result.rms_energy > 0.6 && result.rms_energy < 0.8,
            "rms was {}",
            result.rms_energy,
        );
    }

    #[test]
    fn silence_produces_zero_results() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.0; 48_000],
        );
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.rms_energy, 0.0);
        assert_eq!(result.zero_crossing_rate_hz, 0.0);
        assert_eq!(result.peak_amplitude, 0.0);
        assert_eq!(result.transient_density, 0.0);
        assert_eq!(result.sustain_ratio, 0.0);
        assert_eq!(result.dynamic_range, 0.0);
    }

    #[test]
    fn empty_audio_yields_zero_confidence() {
        let audio =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, Vec::new());
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.confidence, Confidence::new(0.0));
        assert_eq!(result.peak_amplitude, 0.0);
        assert_eq!(result.transient_density, 0.0);
        assert_eq!(result.sustain_ratio, 0.0);
        assert_eq!(result.dynamic_range, 0.0);
    }

    #[test]
    fn zcr_near_expected_for_440hz_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        // A 440 Hz sine crosses zero ~880 times per second (twice per cycle).
        assert!(
            result.zero_crossing_rate_hz > 800.0 && result.zero_crossing_rate_hz < 920.0,
            "zcr was {}",
            result.zero_crossing_rate_hz,
        );
    }

    #[test]
    fn onset_density_is_finite() {
        let audio = sine_audio(440.0, 2.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(result.onset_density.is_finite());
    }

    #[test]
    fn analysis_stage_trait_works() {
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = <CharacterAnalyzer as AnalysisStage<CharacterAnalysisResult>>::analyze(
            &mut analyzer,
            &audio,
        );
        assert!(result.rms_energy > 0.0);
        assert_eq!(analyzer.mode(), AnalysisMode::Offline);
    }

    #[test]
    fn low_profile_still_produces_results() {
        let audio = sine_audio(440.0, 4.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::low());
        let result = analyzer.analyze(&audio);

        assert!(result.spectral_centroid_hz > 0.0);
        assert!(result.rms_energy > 0.0);
    }

    #[test]
    fn peak_amplitude_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        // Full-scale sine has peak ~1.0.
        assert!(
            result.peak_amplitude > 0.95 && result.peak_amplitude <= 1.0,
            "peak was {}",
            result.peak_amplitude,
        );
    }

    #[test]
    fn peak_amplitude_for_half_scale_sine() {
        let count = 48_000;
        let mut data = vec![0.0f32; count];
        for (i, sample) in data.iter_mut().enumerate() {
            let t = i as f32 / 48_000.0;
            *sample = 0.5 * (core::f32::consts::TAU * 440.0 * t).sin();
        }
        let audio = AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, data);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.peak_amplitude > 0.45 && result.peak_amplitude < 0.55,
            "peak was {}",
            result.peak_amplitude,
        );
    }

    #[test]
    fn dynamic_range_is_peak_minus_rms() {
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        let expected = result.peak_amplitude - result.rms_energy;
        assert!(
            (result.dynamic_range - expected).abs() < 1e-6,
            "dynamic_range {} != peak {} - rms {}",
            result.dynamic_range,
            result.peak_amplitude,
            result.rms_energy,
        );
    }

    #[test]
    fn sustain_ratio_near_one_for_loud_signal() {
        // A full-scale sine is above 0.02 for the vast majority of its cycle.
        let audio = sine_audio(440.0, 1.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.sustain_ratio > 0.95,
            "sustain_ratio was {}",
            result.sustain_ratio,
        );
    }

    #[test]
    fn sustain_ratio_near_zero_for_very_quiet_signal() {
        // All samples well below the 0.02 threshold.
        let count = 48_000;
        let data = vec![0.001f32; count];
        let audio = AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, data);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.sustain_ratio, 0.0);
    }

    #[test]
    fn transient_density_is_finite_and_non_negative() {
        let audio = sine_audio(440.0, 2.0, 48_000);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(result.transient_density.is_finite());
        assert!(result.transient_density >= 0.0);
    }

    #[test]
    fn transient_density_increases_with_sharp_edges() {
        // Create a signal with periodic sharp transients: alternating
        // silence (0.0) and a sharp positive value (0.5).
        let sr = 48_000;
        let duration = 2.0;
        let count = (sr as f32 * duration) as usize;
        let mut data = vec![0.0f32; count];
        // Insert a sharp transition every 4800 samples (100 per second at
        // 48 kHz means 10 ms spacing, well above sr/20 = 2400).
        let spacing = 4800;
        for i in (0..count).step_by(spacing) {
            if i + 1 < count {
                data[i] = 0.0;
                data[i + 1] = 0.5;
            }
        }

        let audio = AudioBuffer::from_interleaved(SampleRate(sr), ChannelLayout::Mono, data);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        // We expect roughly count/spacing = 20 transients over 2 seconds → ~10/s.
        assert!(
            result.transient_density > 1.0,
            "transient_density was {}",
            result.transient_density,
        );
    }
}
