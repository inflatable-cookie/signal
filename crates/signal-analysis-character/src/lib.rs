//! Character analysis descriptor packs for Signal.
//!
//! The crate computes reusable offline descriptor packs from mono audio:
//! spectral shape, spectral contrast, coarse mel-profile shape, temporal
//! activity, and amplitude-domain dynamics.
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
//! assert!(result.dynamics.rms_energy >= 0.0 && result.dynamics.rms_energy <= 1.0);
//! ```

use core::cmp::Ordering;
use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode,
    AnalysisStage, Confidence,
};
use signal_dsp_spectral::{
    LogCompression, MelFilterNorm, MelFilterbankConfig, MelScale, MelSpectrogramConfig,
    Spectrogram, Stft, StftConfig,
};
use signal_primitives::{AudioBuffer, FrameCount, Sample, SampleRate, Seconds};

const MEL_PROFILE_BAND_COUNT: usize = 8;
const ROLLOFF_85_FRACTION: f32 = 0.85;
const ROLLOFF_95_FRACTION: f32 = 0.95;
const MEL_PROFILE_LOW_FREQUENCY_HZ: f32 = 30.0;
const MEL_PROFILE_HIGH_FREQUENCY_HZ: f32 = 12_000.0;
const TEMPORAL_SHAPE_LOOKBACK_FRAMES: usize = 32;
const TEMPORAL_SHAPE_LOOKAHEAD_FRAMES: usize = 32;
const TEMPORAL_SHAPE_PEAK_SEARCH_BACK_FRAMES: usize = 1;
const TEMPORAL_SHAPE_PEAK_SEARCH_FORWARD_FRAMES: usize = 16;
const SUSTAIN_PLATEAU_THRESHOLD_RATIO: f32 = 0.7;
const TRANSIENT_PEAK_MIN_SPACING_FRAMES: usize = 8;

/// Configuration for the offline character analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterAnalyzerConfig {
    pub stft: StftConfig,
    /// Sample rate used by the character analysis path after input prep.
    ///
    /// Freezing the analysis rate keeps descriptor packs more comparable across
    /// differently sampled source material.
    pub analysis_sample_rate: SampleRate,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed.
    pub analysis_duration_seconds: Option<u32>,
    /// Onset detection threshold relative to the mean spectral flux.
    /// A peak in the spectral flux envelope is counted as an onset when
    /// it exceeds `onset_threshold` times the mean flux.
    pub onset_threshold: f32,
}

impl CharacterAnalyzerConfig {
    /// Quick scanning profile - 30-second centre segment.
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(1024),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(30),
            onset_threshold: 1.5,
        }
    }

    /// Balanced profile - 60-second centre segment.
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(2048),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(60),
            onset_threshold: 1.5,
        }
    }

    /// Full-accuracy profile - entire track.
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
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
            onset_threshold: 1.5,
        }
    }
}

/// Reduction policy variants frozen for descriptor-pack reuse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorReduction {
    /// Descriptor is computed over the full analysed segment directly.
    WholeSignal,
    /// Descriptor is computed per STFT frame, then reduced by the frame median.
    MedianAcrossFrames,
    /// Descriptor is computed per STFT frame, then reduced by the frame mean.
    MeanAcrossFrames,
    /// Descriptor is computed per STFT frame, mean-reduced, then normalized.
    MeanAcrossFramesNormalized,
    /// Descriptor is reduced across detected temporal events by the event median.
    MedianAcrossEvents,
    /// Descriptor is reduced across detected temporal events by the event mean.
    MeanAcrossEvents,
    /// Descriptor takes the strongest detected temporal event.
    PeakAcrossEvents,
}

/// Explicit reduction policy for every current character descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterDescriptorReductionPolicy {
    pub spectral_centroid_hz: DescriptorReduction,
    pub spectral_spread_hz: DescriptorReduction,
    pub spectral_rolloff_85_hz: DescriptorReduction,
    pub spectral_rolloff_95_hz: DescriptorReduction,
    pub spectral_flatness: DescriptorReduction,
    pub spectral_contrast_db: DescriptorReduction,
    pub normalized_mel_band_profile: DescriptorReduction,
    pub onset_density: DescriptorReduction,
    pub zero_crossing_rate_hz: DescriptorReduction,
    pub transient_density: DescriptorReduction,
    pub sustain_ratio: DescriptorReduction,
    pub peak_transient_strength: DescriptorReduction,
    pub median_transient_strength: DescriptorReduction,
    pub attack_time_ms: DescriptorReduction,
    pub decay_time_ms: DescriptorReduction,
    pub sustain_plateau_ratio: DescriptorReduction,
    pub rms_energy: DescriptorReduction,
    pub peak_amplitude: DescriptorReduction,
    pub dynamic_range: DescriptorReduction,
}

impl Default for CharacterDescriptorReductionPolicy {
    fn default() -> Self {
        Self {
            spectral_centroid_hz: DescriptorReduction::MedianAcrossFrames,
            spectral_spread_hz: DescriptorReduction::MedianAcrossFrames,
            spectral_rolloff_85_hz: DescriptorReduction::MedianAcrossFrames,
            spectral_rolloff_95_hz: DescriptorReduction::MedianAcrossFrames,
            spectral_flatness: DescriptorReduction::MedianAcrossFrames,
            spectral_contrast_db: DescriptorReduction::MedianAcrossFrames,
            normalized_mel_band_profile: DescriptorReduction::MeanAcrossFramesNormalized,
            onset_density: DescriptorReduction::WholeSignal,
            zero_crossing_rate_hz: DescriptorReduction::WholeSignal,
            transient_density: DescriptorReduction::WholeSignal,
            sustain_ratio: DescriptorReduction::WholeSignal,
            peak_transient_strength: DescriptorReduction::PeakAcrossEvents,
            median_transient_strength: DescriptorReduction::MedianAcrossEvents,
            attack_time_ms: DescriptorReduction::MedianAcrossEvents,
            decay_time_ms: DescriptorReduction::MedianAcrossEvents,
            sustain_plateau_ratio: DescriptorReduction::MeanAcrossEvents,
            rms_energy: DescriptorReduction::WholeSignal,
            peak_amplitude: DescriptorReduction::WholeSignal,
            dynamic_range: DescriptorReduction::WholeSignal,
        }
    }
}

/// Spectral shape descriptors reduced from the linear spectrogram.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralShapeDescriptorPack {
    /// Median framewise spectral centroid, in Hz.
    pub centroid_hz: f32,
    /// Median framewise spectral spread around the centroid, in Hz.
    pub spread_hz: f32,
    /// Median framewise 85% spectral rolloff point, in Hz.
    pub rolloff_85_hz: f32,
    /// Median framewise 95% spectral rolloff point, in Hz.
    pub rolloff_95_hz: f32,
    /// Median framewise spectral flatness over non-DC power bins.
    pub flatness: f32,
}

impl SpectralShapeDescriptorPack {
    fn zero() -> Self {
        Self {
            centroid_hz: 0.0,
            spread_hz: 0.0,
            rolloff_85_hz: 0.0,
            rolloff_95_hz: 0.0,
            flatness: 0.0,
        }
    }
}

/// Spectral contrast summary reduced from framewise spectral percentiles.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralContrastDescriptorPack {
    /// Median framewise `20 * log10(p90 / p10)` contrast across non-DC bins.
    pub contrast_db: f32,
}

impl SpectralContrastDescriptorPack {
    fn zero() -> Self {
        Self { contrast_db: 0.0 }
    }
}

/// Coarse mel-band profile suitable for later embedding and search surfaces.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralProfileDescriptorPack {
    /// Mean mel-band energy profile normalized to sum to `1.0` when energy
    /// exists. Uses 8 Slaney mel bands over `30 Hz..=min(Nyquist, 12 kHz)`.
    pub normalized_mel_band_profile: [f32; MEL_PROFILE_BAND_COUNT],
}

impl SpectralProfileDescriptorPack {
    fn zero() -> Self {
        Self {
            normalized_mel_band_profile: [0.0; MEL_PROFILE_BAND_COUNT],
        }
    }
}

/// Activity-oriented temporal descriptors computed over the analysed segment.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalDescriptorPack {
    /// Onsets per second derived from spectral-flux peak counting.
    pub onset_density: f32,
    /// Mean zero-crossing rate in Hz.
    pub zero_crossing_rate_hz: f32,
    /// Transient events per second from sample-slope detection.
    pub transient_density: f32,
    /// Fraction of samples whose absolute value is at least `0.02`.
    pub sustain_ratio: f32,
}

impl TemporalDescriptorPack {
    fn zero() -> Self {
        Self {
            onset_density: 0.0,
            zero_crossing_rate_hz: 0.0,
            transient_density: 0.0,
            sustain_ratio: 0.0,
        }
    }
}

/// Event-oriented transient and envelope-shape descriptors.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalShapeDescriptorPack {
    /// Strongest normalized spectral-flux peak among detected transient events.
    pub peak_transient_strength: f32,
    /// Median normalized spectral-flux peak across detected transient events.
    pub median_transient_strength: f32,
    /// Median event attack time in milliseconds, measured from the local
    /// amplitude-envelope floor to the event peak.
    pub attack_time_ms: f32,
    /// Median event decay time in milliseconds, measured from the event peak to
    /// the local post-peak amplitude-envelope floor.
    pub decay_time_ms: f32,
    /// Mean ratio of post-peak event time that remains above the sustain
    /// threshold before clear decay takes over.
    pub sustain_plateau_ratio: f32,
}

impl TemporalShapeDescriptorPack {
    fn zero() -> Self {
        Self {
            peak_transient_strength: 0.0,
            median_transient_strength: 0.0,
            attack_time_ms: 0.0,
            decay_time_ms: 0.0,
            sustain_plateau_ratio: 0.0,
        }
    }
}

/// Amplitude-domain dynamics descriptors computed over the analysed segment.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicsDescriptorPack {
    /// RMS energy of the mono signal, clamped to `0.0..=1.0`.
    pub rms_energy: f32,
    /// Peak absolute sample amplitude.
    pub peak_amplitude: f32,
    /// Crest headroom: `peak_amplitude - rms_energy`, clamped to `0.0+`.
    pub dynamic_range: f32,
}

impl DynamicsDescriptorPack {
    fn zero() -> Self {
        Self {
            rms_energy: 0.0,
            peak_amplitude: 0.0,
            dynamic_range: 0.0,
        }
    }
}

/// Character analysis result grouped into reusable descriptor packs.
///
/// Practical integration order:
/// 1. Read `spectral_shape` for brightness, spread, and rolloff position.
/// 2. Read `spectral_contrast` and `spectral_profile` for timbral texture.
/// 3. Read `temporal` for onset, noisiness, transient, and sustain evidence.
/// 4. Read `temporal_shape` for transient strength and attack/decay behavior.
/// 5. Read `dynamics` for amplitude-domain level and crest information.
/// 6. Read `reduction_policy` and `confidence` before comparing results across
///    clips or persisting them into downstream feature stores.
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterAnalysisResult {
    pub spectral_shape: SpectralShapeDescriptorPack,
    pub spectral_contrast: SpectralContrastDescriptorPack,
    pub spectral_profile: SpectralProfileDescriptorPack,
    pub temporal: TemporalDescriptorPack,
    pub temporal_shape: TemporalShapeDescriptorPack,
    pub dynamics: DynamicsDescriptorPack,
    pub reduction_policy: CharacterDescriptorReductionPolicy,
    pub confidence: Confidence,
}

impl CharacterAnalysisResult {
    fn zero() -> Self {
        Self {
            spectral_shape: SpectralShapeDescriptorPack::zero(),
            spectral_contrast: SpectralContrastDescriptorPack::zero(),
            spectral_profile: SpectralProfileDescriptorPack::zero(),
            temporal: TemporalDescriptorPack::zero(),
            temporal_shape: TemporalShapeDescriptorPack::zero(),
            dynamics: DynamicsDescriptorPack::zero(),
            reduction_policy: CharacterDescriptorReductionPolicy::default(),
            confidence: Confidence::new(0.0),
        }
    }
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
        let prepared =
            prepare_mono_analysis(sample_rate, mono_samples, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }

    fn analysis_input_config(&self) -> AnalysisInputConfig {
        AnalysisInputConfig {
            max_duration: self
                .config
                .analysis_duration_seconds
                .map(|seconds| Seconds(seconds as f32)),
            target_sample_rate: Some(self.config.analysis_sample_rate),
            ..AnalysisInputConfig::default()
        }
    }

    fn analyze_prepared(
        &self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> CharacterAnalysisResult {
        if mono_samples.is_empty() || sample_rate.0 == 0 {
            return CharacterAnalysisResult::zero();
        }

        let duration_seconds = mono_samples.len() as f32 / sample_rate.0 as f32;
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let spectral_flux = spectral_flux_envelope(&spectrogram);
        let transient_peak_indices =
            detect_peak_indices(&spectral_flux, self.config.onset_threshold);
        let frame_envelope = frame_rms_envelope(mono_samples, self.config.stft);

        let temporal = TemporalDescriptorPack {
            onset_density: compute_event_density(transient_peak_indices.len(), duration_seconds),
            zero_crossing_rate_hz: compute_zcr(mono_samples, sample_rate),
            transient_density: compute_transient_density(
                mono_samples,
                sample_rate,
                duration_seconds,
            ),
            sustain_ratio: compute_sustain_ratio(mono_samples),
        };
        let temporal_shape = compute_temporal_shape_pack(
            sample_rate,
            self.config.stft,
            &spectral_flux,
            &frame_envelope,
            &transient_peak_indices,
        );

        let dynamics = {
            let rms_energy = compute_rms(mono_samples);
            let peak_amplitude = compute_peak_amplitude(mono_samples);
            DynamicsDescriptorPack {
                rms_energy,
                peak_amplitude,
                dynamic_range: (peak_amplitude - rms_energy).max(0.0),
            }
        };

        CharacterAnalysisResult {
            spectral_shape: compute_spectral_shape_pack(&spectrogram),
            spectral_contrast: compute_spectral_contrast_pack(&spectrogram),
            spectral_profile: compute_spectral_profile_pack(&spectrogram),
            temporal,
            temporal_shape,
            dynamics,
            reduction_policy: CharacterDescriptorReductionPolicy::default(),
            confidence: character_confidence(sample_rate, mono_samples.len()),
        }
    }
}

impl AnalysisStage<CharacterAnalysisResult> for CharacterAnalyzer {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> CharacterAnalysisResult {
        let prepared = prepare_audio_analysis(audio, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }
}

#[derive(Clone, Copy, Debug)]
struct FrameSpectralShape {
    centroid_hz: f32,
    spread_hz: f32,
    rolloff_85_hz: f32,
    rolloff_95_hz: f32,
    flatness: f32,
}

fn compute_spectral_shape_pack(spectrogram: &Spectrogram) -> SpectralShapeDescriptorPack {
    let mut centroids = Vec::new();
    let mut spreads = Vec::new();
    let mut rolloff_85 = Vec::new();
    let mut rolloff_95 = Vec::new();
    let mut flatness = Vec::new();

    for shape in spectrogram
        .frames
        .iter()
        .filter_map(|frame| frame_spectral_shape(spectrogram, &frame.magnitudes))
    {
        centroids.push(shape.centroid_hz);
        spreads.push(shape.spread_hz);
        rolloff_85.push(shape.rolloff_85_hz);
        rolloff_95.push(shape.rolloff_95_hz);
        flatness.push(shape.flatness);
    }

    if centroids.is_empty() {
        return SpectralShapeDescriptorPack::zero();
    }

    SpectralShapeDescriptorPack {
        centroid_hz: reduce_median(&mut centroids),
        spread_hz: reduce_median(&mut spreads),
        rolloff_85_hz: reduce_median(&mut rolloff_85),
        rolloff_95_hz: reduce_median(&mut rolloff_95),
        flatness: reduce_median(&mut flatness),
    }
}

fn compute_spectral_contrast_pack(spectrogram: &Spectrogram) -> SpectralContrastDescriptorPack {
    let mut frame_contrasts = Vec::new();

    for frame in &spectrogram.frames {
        let magnitudes: Vec<f32> = frame
            .magnitudes
            .iter()
            .copied()
            .skip(1)
            .filter(|magnitude| *magnitude > 0.0)
            .collect();
        if magnitudes.len() < 2 {
            continue;
        }

        let mut sorted = magnitudes;
        sort_f32(&mut sorted);
        let low = percentile_value(&sorted, 0.10);
        let high = percentile_value(&sorted, 0.90);
        if high <= 0.0 {
            continue;
        }

        let contrast_db = 20.0 * ((high + f32::EPSILON) / (low + f32::EPSILON)).log10();
        if contrast_db.is_finite() {
            frame_contrasts.push(contrast_db.max(0.0));
        }
    }

    if frame_contrasts.is_empty() {
        return SpectralContrastDescriptorPack::zero();
    }

    SpectralContrastDescriptorPack {
        contrast_db: reduce_median(&mut frame_contrasts),
    }
}

fn compute_spectral_profile_pack(spectrogram: &Spectrogram) -> SpectralProfileDescriptorPack {
    if spectrogram.is_empty() || spectrogram.sample_rate.0 == 0 {
        return SpectralProfileDescriptorPack::zero();
    }

    let nyquist_hz = spectrogram.sample_rate.0 as f32 * 0.5;
    let high_frequency_hz = nyquist_hz.clamp(
        MEL_PROFILE_LOW_FREQUENCY_HZ + 1.0,
        MEL_PROFILE_HIGH_FREQUENCY_HZ,
    );
    let mel = spectrogram.to_mel_spectrogram(&MelSpectrogramConfig {
        filterbank: MelFilterbankConfig {
            mel_bin_count: MEL_PROFILE_BAND_COUNT,
            low_frequency_hz: MEL_PROFILE_LOW_FREQUENCY_HZ,
            high_frequency_hz,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::UnitTri,
        },
        log_compression: LogCompression::None,
    });

    if mel.frames.is_empty() {
        return SpectralProfileDescriptorPack::zero();
    }

    let mut profile = [0.0; MEL_PROFILE_BAND_COUNT];
    for frame in &mel.frames {
        for (index, value) in frame
            .iter()
            .copied()
            .enumerate()
            .take(MEL_PROFILE_BAND_COUNT)
        {
            profile[index] += value;
        }
    }

    let frame_count = mel.frames.len() as f32;
    for value in &mut profile {
        *value /= frame_count;
    }

    normalize_slice(&mut profile);
    SpectralProfileDescriptorPack {
        normalized_mel_band_profile: profile,
    }
}

fn frame_spectral_shape(
    spectrogram: &Spectrogram,
    magnitudes: &[f32],
) -> Option<FrameSpectralShape> {
    if magnitudes.len() < 2
        || spectrogram.config.window_size.0 == 0
        || spectrogram.sample_rate.0 == 0
    {
        return None;
    }

    let mut weighted_sum = 0.0f32;
    let mut magnitude_sum = 0.0f32;
    let mut power_values = Vec::new();

    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        let frequency = signal_dsp_spectral::bin_frequency(
            bin_index,
            spectrogram.sample_rate,
            spectrogram.config.window_size.0,
        );
        weighted_sum += frequency * magnitude;
        magnitude_sum += magnitude;
        if bin_index > 0 && magnitude > 0.0 {
            power_values.push(magnitude * magnitude);
        }
    }

    if magnitude_sum <= 0.0 {
        return None;
    }

    let centroid_hz = weighted_sum / magnitude_sum;
    let mut spread_sum = 0.0f32;
    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        let frequency = signal_dsp_spectral::bin_frequency(
            bin_index,
            spectrogram.sample_rate,
            spectrogram.config.window_size.0,
        );
        spread_sum += (frequency - centroid_hz).powi(2) * magnitude;
    }

    let spread_hz = (spread_sum / magnitude_sum).sqrt();
    let rolloff_85_hz =
        frame_rolloff_frequency(magnitudes, spectrogram, ROLLOFF_85_FRACTION).unwrap_or(0.0);
    let rolloff_95_hz =
        frame_rolloff_frequency(magnitudes, spectrogram, ROLLOFF_95_FRACTION).unwrap_or(0.0);
    let flatness = spectral_flatness(&power_values);

    Some(FrameSpectralShape {
        centroid_hz,
        spread_hz,
        rolloff_85_hz,
        rolloff_95_hz,
        flatness,
    })
}

fn frame_rolloff_frequency(
    magnitudes: &[f32],
    spectrogram: &Spectrogram,
    fraction: f32,
) -> Option<f32> {
    let total_energy: f32 = magnitudes.iter().copied().sum();
    if total_energy <= 0.0 {
        return None;
    }

    let target = total_energy * fraction.clamp(0.0, 1.0);
    let mut cumulative = 0.0f32;
    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        cumulative += magnitude;
        if cumulative >= target {
            return Some(signal_dsp_spectral::bin_frequency(
                bin_index,
                spectrogram.sample_rate,
                spectrogram.config.window_size.0,
            ));
        }
    }

    Some(signal_dsp_spectral::bin_frequency(
        magnitudes.len().saturating_sub(1),
        spectrogram.sample_rate,
        spectrogram.config.window_size.0,
    ))
}

fn spectral_flatness(power_values: &[f32]) -> f32 {
    if power_values.is_empty() {
        return 0.0;
    }

    let epsilon = 1e-12f32;
    let log_mean = power_values
        .iter()
        .copied()
        .map(|value| (value + epsilon).ln())
        .sum::<f32>()
        / power_values.len() as f32;
    let arithmetic_mean = power_values.iter().copied().sum::<f32>() / power_values.len() as f32;
    if arithmetic_mean <= 0.0 {
        0.0
    } else {
        (log_mean.exp() / arithmetic_mean).clamp(0.0, 1.0)
    }
}

fn spectral_flux_envelope(spectrogram: &Spectrogram) -> Vec<f32> {
    if spectrogram.is_empty() {
        return Vec::new();
    }

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

    flux
}

fn detect_peak_indices(values: &[f32], threshold_multiplier: f32) -> Vec<usize> {
    if values.len() < 3 {
        return Vec::new();
    }

    let mean = values.iter().copied().sum::<f32>() / values.len() as f32;
    let threshold = mean * threshold_multiplier;
    let mut peaks = Vec::new();

    for index in 1..values.len().saturating_sub(1) {
        if values[index] > threshold
            && values[index] > values[index - 1]
            && values[index] > values[index + 1]
        {
            if let Some(last_peak) = peaks.last_mut() {
                if index - *last_peak <= TRANSIENT_PEAK_MIN_SPACING_FRAMES {
                    if values[index] > values[*last_peak] {
                        *last_peak = index;
                    }
                    continue;
                }
            }
            peaks.push(index);
        }
    }

    peaks
}

fn compute_event_density(event_count: usize, duration_seconds: f32) -> f32 {
    if duration_seconds <= 0.0 {
        0.0
    } else {
        event_count as f32 / duration_seconds
    }
}

fn frame_rms_envelope(samples: &[Sample], config: StftConfig) -> Vec<f32> {
    let window_size = config.window_size.0;
    let hop_size = config.hop_size.0.max(1);
    if samples.is_empty() || window_size == 0 {
        return Vec::new();
    }

    let mut envelope = Vec::new();
    let mut start = 0usize;

    while start + window_size <= samples.len() {
        envelope.push(compute_rms(&samples[start..start + window_size]));
        start += hop_size;
    }

    let overlap_tail = window_size.saturating_sub(hop_size);
    if start == 0 || (start < samples.len() && samples.len() - start > overlap_tail) {
        envelope.push(compute_rms(&samples[start..]));
    }

    envelope
}

fn compute_temporal_shape_pack(
    sample_rate: SampleRate,
    stft_config: StftConfig,
    spectral_flux: &[f32],
    frame_envelope: &[f32],
    transient_peak_indices: &[usize],
) -> TemporalShapeDescriptorPack {
    if sample_rate.0 == 0 || spectral_flux.is_empty() || frame_envelope.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    let flux_peak = spectral_flux.iter().copied().fold(0.0f32, f32::max);
    if flux_peak <= 0.0 || transient_peak_indices.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    let smoothed_envelope = smooth_series(frame_envelope, 1);
    let hop_seconds = stft_config.hop_size.0.max(1) as f32 / sample_rate.0 as f32;
    let mut transient_strengths = Vec::new();
    let mut attack_times_ms = Vec::new();
    let mut decay_times_ms = Vec::new();
    let mut sustain_plateau_ratios = Vec::new();

    for &peak_index in transient_peak_indices {
        if peak_index >= spectral_flux.len() || peak_index >= smoothed_envelope.len() {
            continue;
        }

        let normalized_strength = (spectral_flux[peak_index] / flux_peak).clamp(0.0, 1.0);
        transient_strengths.push(normalized_strength);

        let event_peak_index = local_argmax_index(
            &smoothed_envelope,
            peak_index.saturating_sub(TEMPORAL_SHAPE_PEAK_SEARCH_BACK_FRAMES),
            peak_index
                .saturating_add(TEMPORAL_SHAPE_PEAK_SEARCH_FORWARD_FRAMES)
                .min(smoothed_envelope.len().saturating_sub(1)),
        );
        let left_floor_index = local_argmin_index(
            &smoothed_envelope,
            event_peak_index.saturating_sub(TEMPORAL_SHAPE_LOOKBACK_FRAMES),
            event_peak_index,
        );
        let right_floor_index = local_argmin_index(
            &smoothed_envelope,
            event_peak_index,
            event_peak_index
                .saturating_add(TEMPORAL_SHAPE_LOOKAHEAD_FRAMES)
                .min(smoothed_envelope.len().saturating_sub(1)),
        );

        let peak_level = smoothed_envelope[event_peak_index];
        let left_baseline = smoothed_envelope[left_floor_index];
        let right_baseline = smoothed_envelope[right_floor_index];

        if peak_level > left_baseline {
            let attack_low = left_baseline + (peak_level - left_baseline) * 0.1;
            let attack_high = left_baseline + (peak_level - left_baseline) * 0.9;
            let attack_start_index = find_last_index_at_or_below(
                &smoothed_envelope,
                left_floor_index,
                event_peak_index,
                attack_low,
            );
            let attack_end_index = find_first_index_at_or_above(
                &smoothed_envelope,
                attack_start_index,
                event_peak_index,
                attack_high,
            );

            if attack_end_index > attack_start_index {
                attack_times_ms
                    .push((attack_end_index - attack_start_index) as f32 * hop_seconds * 1_000.0);
            }
        }

        if right_floor_index > event_peak_index && peak_level > right_baseline {
            let decay_high = right_baseline + (peak_level - right_baseline) * 0.9;
            let decay_low = right_baseline + (peak_level - right_baseline) * 0.1;
            let decay_start_index = find_first_index_at_or_below(
                &smoothed_envelope,
                event_peak_index,
                right_floor_index,
                decay_high,
            );
            let decay_end_index = find_first_index_at_or_below(
                &smoothed_envelope,
                decay_start_index,
                right_floor_index,
                decay_low,
            );

            if decay_end_index > decay_start_index {
                decay_times_ms
                    .push((decay_end_index - decay_start_index) as f32 * hop_seconds * 1_000.0);
            }
        }

        let baseline = left_baseline.min(right_baseline);
        let threshold =
            baseline + (peak_level - baseline).max(0.0) * SUSTAIN_PLATEAU_THRESHOLD_RATIO;

        if right_floor_index > event_peak_index && peak_level > baseline {
            let mut sustain_frames = 0usize;
            for &value in &smoothed_envelope[event_peak_index..=right_floor_index] {
                if value >= threshold {
                    sustain_frames += 1;
                } else {
                    break;
                }
            }

            let decay_frames = right_floor_index - event_peak_index;
            sustain_plateau_ratios
                .push((sustain_frames as f32 / decay_frames.max(1) as f32).clamp(0.0, 1.0));
        }
    }

    if transient_strengths.is_empty() {
        return TemporalShapeDescriptorPack::zero();
    }

    TemporalShapeDescriptorPack {
        peak_transient_strength: transient_strengths.iter().copied().fold(0.0, f32::max),
        median_transient_strength: reduce_median(&mut transient_strengths),
        attack_time_ms: reduce_median_or_zero(&mut attack_times_ms),
        decay_time_ms: reduce_median_or_zero(&mut decay_times_ms),
        sustain_plateau_ratio: mean_or_zero(&sustain_plateau_ratios).clamp(0.0, 1.0),
    }
}

fn smooth_series(values: &[f32], radius: usize) -> Vec<f32> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut smoothed = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        let start = index.saturating_sub(radius);
        let end = (index + radius).min(values.len() - 1);
        let slice = &values[start..=end];
        smoothed.push(slice.iter().copied().sum::<f32>() / slice.len() as f32);
    }

    smoothed
}

fn local_argmax_index(values: &[f32], start: usize, end: usize) -> usize {
    let mut best_index = start.min(values.len().saturating_sub(1));
    let mut best_value = values.get(best_index).copied().unwrap_or(0.0);
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .take(end.min(values.len().saturating_sub(1)) - start + 1)
    {
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }
    best_index
}

fn local_argmin_index(values: &[f32], start: usize, end: usize) -> usize {
    let mut best_index = start.min(values.len().saturating_sub(1));
    let mut best_value = values.get(best_index).copied().unwrap_or(0.0);
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .take(end.min(values.len().saturating_sub(1)) - start + 1)
    {
        if value < best_value {
            best_value = value;
            best_index = index;
        }
    }
    best_index
}

fn find_last_index_at_or_below(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for index in (start.min(bounded_end)..=bounded_end).rev() {
        if values[index] <= threshold {
            return index;
        }
    }
    start.min(bounded_end)
}

fn find_first_index_at_or_above(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start.min(bounded_end))
        .take(bounded_end - start.min(bounded_end) + 1)
    {
        if value >= threshold {
            return index;
        }
    }
    bounded_end
}

fn find_first_index_at_or_below(values: &[f32], start: usize, end: usize, threshold: f32) -> usize {
    let bounded_end = end.min(values.len().saturating_sub(1));
    for (index, value) in values
        .iter()
        .copied()
        .enumerate()
        .skip(start.min(bounded_end))
        .take(bounded_end - start.min(bounded_end) + 1)
    {
        if value <= threshold {
            return index;
        }
    }
    bounded_end
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

/// Compute RMS energy, clamped to `0.0..=1.0`.
fn compute_rms(samples: &[Sample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples
        .iter()
        .map(|&sample| (sample as f64) * (sample as f64))
        .sum();
    let rms = (sum_squares / samples.len() as f64).sqrt() as f32;
    rms.clamp(0.0, 1.0)
}

/// Peak absolute sample amplitude.
fn compute_peak_amplitude(samples: &[Sample]) -> f32 {
    samples
        .iter()
        .fold(0.0f32, |peak, &sample| peak.max(sample.abs()))
}

/// Transient density via sample-slope detection.
///
/// Counts transitions where the absolute difference between consecutive
/// samples exceeds `SLOPE_THRESHOLD`, with a minimum spacing of
/// `sample_rate / 20` samples between counted events.
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

    for index in 1..samples.len() {
        let slope = (samples[index] - samples[index - 1]).abs();
        samples_since_last += 1;
        if slope >= SLOPE_THRESHOLD && samples_since_last >= min_spacing {
            count += 1;
            samples_since_last = 0;
        }
    }

    count as f32 / duration_seconds
}

/// Sustain ratio: fraction of samples with absolute value >= `0.02`.
fn compute_sustain_ratio(samples: &[Sample]) -> f32 {
    const SILENCE_THRESHOLD: f32 = 0.02;

    if samples.is_empty() {
        return 0.0;
    }

    let sustained = samples
        .iter()
        .filter(|&&sample| sample.abs() >= SILENCE_THRESHOLD)
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

fn reduce_median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    sort_f32(values);
    values[values.len() / 2]
}

fn reduce_median_or_zero(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        reduce_median(values)
    }
}

fn sort_f32(values: &mut [f32]) {
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
}

fn percentile_value(sorted_values: &[f32], fraction: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let clamped = fraction.clamp(0.0, 1.0);
    let index = ((sorted_values.len().saturating_sub(1) as f32) * clamped).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

fn normalize_slice(values: &mut [f32]) {
    let sum = values.iter().copied().sum::<f32>();
    if sum > 0.0 {
        for value in values {
            *value /= sum;
        }
    }
}

fn mean_or_zero(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().copied().sum::<f32>() / values.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine_audio(
        frequency_hz: f32,
        duration_seconds: f32,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        for (index, sample) in data.iter_mut().enumerate() {
            let time = index as f32 / sample_rate_hz as f32;
            *sample = amplitude * (core::f32::consts::TAU * frequency_hz * time).sin();
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn noise_audio(duration_seconds: f32, sample_rate_hz: u32, amplitude: f32) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        let mut state = 0x1234_5678u32;
        for sample in &mut data {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let unit = ((state >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sample = amplitude * unit;
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn adsr_pulse_audio(
        attack_ms: u32,
        sustain_ms: u32,
        decay_ms: u32,
        interval_ms: u32,
        event_count: usize,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let interval_samples = (interval_ms as usize * sample_rate_hz as usize) / 1_000;
        let attack_samples = (attack_ms as usize * sample_rate_hz as usize) / 1_000;
        let sustain_samples = (sustain_ms as usize * sample_rate_hz as usize) / 1_000;
        let decay_samples = (decay_ms as usize * sample_rate_hz as usize) / 1_000;
        let total_samples = interval_samples * event_count.max(1);
        let mut data = vec![0.0f32; total_samples.max(1)];

        for event_index in 0..event_count {
            let start = event_index * interval_samples;

            for offset in 0..attack_samples {
                let index = start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = (offset + 1) as f32 / attack_samples.max(1) as f32;
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }

            let sustain_start = start + attack_samples;
            for offset in 0..sustain_samples {
                let index = sustain_start + offset;
                if index >= data.len() {
                    break;
                }
                data[index] = amplitude;
            }

            let decay_start = sustain_start + sustain_samples;
            for offset in 0..decay_samples {
                let index = decay_start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = 1.0 - ((offset + 1) as f32 / decay_samples.max(1) as f32);
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn character_metrics(result: &CharacterAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("spectral_flatness", result.spectral_shape.flatness),
            AnalysisMetricValue::new("spectral_spread_hz", result.spectral_shape.spread_hz),
            AnalysisMetricValue::new("rms_energy", result.dynamics.rms_energy),
            AnalysisMetricValue::new("sustain_ratio", result.temporal.sustain_ratio),
            AnalysisMetricValue::new(
                "peak_transient_strength",
                result.temporal_shape.peak_transient_strength,
            ),
            AnalysisMetricValue::new("descriptor_confidence", result.confidence.0),
        ]
    }

    fn character_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:tone:sine440",
                    AnalysisCorpusFamily::Tonal,
                    "Sustained tonal descriptor reference",
                ),
                sine_audio(440.0, 2.0, 48_000, 1.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_flatness",
                    Some(0.0),
                    Some(0.05),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.65),
                    Some(0.75),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:noise:deterministic",
                    AnalysisCorpusFamily::Noise,
                    "Broadband descriptor reference",
                ),
                noise_audio(2.0, 48_000, 0.5),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_spread_hz",
                    Some(2_000.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.45),
                    Some(0.55),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:pulse:adsr",
                    AnalysisCorpusFamily::Pulse,
                    "Transient-heavy descriptor reference",
                ),
                adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "peak_transient_strength",
                    Some(0.80),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.25),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    #[test]
    fn spectral_shape_tracks_frequency_position() {
        let low = sine_audio(220.0, 2.0, 48_000, 1.0);
        let high = sine_audio(4_000.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let low_result = analyzer.analyze(&low);
        let high_result = analyzer.analyze(&high);

        assert!(high_result.spectral_shape.centroid_hz > low_result.spectral_shape.centroid_hz);
        assert!(high_result.spectral_shape.rolloff_95_hz > low_result.spectral_shape.rolloff_95_hz);
    }

    #[test]
    fn centroid_near_1khz_for_sine() {
        let audio = sine_audio(1_000.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(
            result.spectral_shape.centroid_hz > 800.0
                && result.spectral_shape.centroid_hz < 1_200.0,
            "centroid was {}",
            result.spectral_shape.centroid_hz,
        );
    }

    #[test]
    fn noise_is_flatter_than_sine() {
        let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
        let noise = noise_audio(2.0, 48_000, 0.5);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let tone_result = analyzer.analyze(&tone);
        let noise_result = analyzer.analyze(&noise);

        assert!(noise_result.spectral_shape.flatness > tone_result.spectral_shape.flatness);
    }

    #[test]
    fn normalized_mel_profile_is_bounded_and_sums_to_one() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        let profile = result.spectral_profile.normalized_mel_band_profile;
        let sum = profile.iter().copied().sum::<f32>();
        assert!((sum - 1.0).abs() < 1e-4, "profile sum was {}", sum);
        assert!(profile.iter().all(|value| *value >= 0.0 && *value <= 1.0));
    }

    #[test]
    fn rms_energy_near_expected_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.rms_energy > 0.6 && result.dynamics.rms_energy < 0.8,
            "rms was {}",
            result.dynamics.rms_energy,
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

        assert_eq!(result.spectral_shape, SpectralShapeDescriptorPack::zero());
        assert_eq!(
            result.spectral_contrast,
            SpectralContrastDescriptorPack::zero()
        );
        assert_eq!(
            result.spectral_profile,
            SpectralProfileDescriptorPack::zero()
        );
        assert_eq!(result.temporal, TemporalDescriptorPack::zero());
        assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
        assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
    }

    #[test]
    fn empty_audio_yields_zero_confidence() {
        let audio =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, Vec::new());
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.confidence, Confidence::new(0.0));
        assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
        assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
    }

    #[test]
    fn zcr_near_expected_for_440hz_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.zero_crossing_rate_hz > 800.0
                && result.temporal.zero_crossing_rate_hz < 920.0,
            "zcr was {}",
            result.temporal.zero_crossing_rate_hz,
        );
    }

    #[test]
    fn onset_density_is_finite() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(result.temporal.onset_density.is_finite());
    }

    #[test]
    fn analysis_stage_trait_works() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = <CharacterAnalyzer as AnalysisStage<CharacterAnalysisResult>>::analyze(
            &mut analyzer,
            &audio,
        );

        assert!(result.dynamics.rms_energy > 0.0);
        assert_eq!(analyzer.mode(), AnalysisMode::Offline);
    }

    #[test]
    fn low_profile_still_produces_results() {
        let audio = sine_audio(440.0, 4.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::low());
        let result = analyzer.analyze(&audio);

        assert!(result.spectral_shape.centroid_hz > 0.0);
        assert!(result.dynamics.rms_energy > 0.0);
    }

    #[test]
    fn peak_amplitude_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.peak_amplitude > 0.95 && result.dynamics.peak_amplitude <= 1.0,
            "peak was {}",
            result.dynamics.peak_amplitude,
        );
    }

    #[test]
    fn peak_amplitude_for_half_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 0.5);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.peak_amplitude > 0.45 && result.dynamics.peak_amplitude < 0.55,
            "peak was {}",
            result.dynamics.peak_amplitude,
        );
    }

    #[test]
    fn dynamic_range_is_peak_minus_rms() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        let expected = result.dynamics.peak_amplitude - result.dynamics.rms_energy;
        assert!(
            (result.dynamics.dynamic_range - expected).abs() < 1e-6,
            "dynamic_range {} != peak {} - rms {}",
            result.dynamics.dynamic_range,
            result.dynamics.peak_amplitude,
            result.dynamics.rms_energy,
        );
    }

    #[test]
    fn sustain_ratio_near_one_for_loud_signal() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.sustain_ratio > 0.95,
            "sustain_ratio was {}",
            result.temporal.sustain_ratio,
        );
    }

    #[test]
    fn sustain_ratio_near_zero_for_very_quiet_signal() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.001; 48_000],
        );
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.temporal.sustain_ratio, 0.0);
    }

    #[test]
    fn transient_density_is_finite_and_non_negative() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(result.temporal.transient_density.is_finite());
        assert!(result.temporal.transient_density >= 0.0);
    }

    #[test]
    fn transient_density_increases_with_sharp_edges() {
        let sample_rate_hz = 48_000;
        let duration_seconds = 2.0;
        let count = (sample_rate_hz as f32 * duration_seconds) as usize;
        let mut data = vec![0.0f32; count];
        let spacing = 4_800;
        for index in (0..count).step_by(spacing) {
            if index + 1 < count {
                data[index + 1] = 0.5;
            }
        }

        let audio =
            AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.transient_density > 1.0,
            "transient_density was {}",
            result.temporal.transient_density,
        );
    }

    #[test]
    fn transient_shape_strength_is_higher_for_pulses_than_steady_tone() {
        let pulse = adsr_pulse_audio(5, 10, 10, 350, 6, 48_000, 0.9);
        let tone = sine_audio(440.0, 2.2, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let pulse_result = analyzer.analyze(&pulse);
        let tone_result = analyzer.analyze(&tone);

        assert!(
            pulse_result.temporal_shape.peak_transient_strength
                > tone_result.temporal_shape.peak_transient_strength
        );
        assert!(
            pulse_result.temporal_shape.median_transient_strength
                >= tone_result.temporal_shape.median_transient_strength
        );
    }

    #[test]
    fn temporal_shape_attack_time_tracks_slower_attacks() {
        let sharp = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let slow = adsr_pulse_audio(80, 10, 10, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let sharp_result = analyzer.analyze(&sharp);
        let slow_result = analyzer.analyze(&slow);

        assert!(
            slow_result.temporal_shape.attack_time_ms > sharp_result.temporal_shape.attack_time_ms,
            "slow attack {} ms was not greater than sharp attack {} ms",
            slow_result.temporal_shape.attack_time_ms,
            sharp_result.temporal_shape.attack_time_ms,
        );
    }

    #[test]
    fn temporal_shape_decay_time_tracks_longer_decays() {
        let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let long = adsr_pulse_audio(5, 10, 120, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let short_result = analyzer.analyze(&short);
        let long_result = analyzer.analyze(&long);

        assert!(
            long_result.temporal_shape.decay_time_ms > short_result.temporal_shape.decay_time_ms
        );
    }

    #[test]
    fn temporal_shape_sustain_ratio_tracks_longer_plateaus() {
        let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let long = adsr_pulse_audio(5, 140, 10, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let short_result = analyzer.analyze(&short);
        let long_result = analyzer.analyze(&long);

        assert!(
            long_result.temporal_shape.sustain_plateau_ratio
                > short_result.temporal_shape.sustain_plateau_ratio
        );
    }

    #[test]
    fn harness_character_descriptor_cases_meet_frozen_acceptance_thresholds() {
        let cases = character_acceptance_cases();
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let report = run_audio_acceptance_harness(
            &cases,
            |audio| analyzer.analyze(audio),
            character_metrics,
        );

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_character_acceptance_report_remains_interpretable_for_closeout() {
        let cases = character_acceptance_cases();
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let report = run_audio_acceptance_harness(
            &cases,
            |audio| analyzer.analyze(audio),
            character_metrics,
        );

        println!("character_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }

    #[test]
    fn descriptor_pack_examples_remain_interpretable_for_closeout() {
        let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
        let noise = noise_audio(2.0, 48_000, 0.5);
        let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let tone_result = analyzer.analyze(&tone);
        let noise_result = analyzer.analyze(&noise);
        let pulse_result = analyzer.analyze(&pulse);

        println!("tone_result={:#?}", tone_result);
        println!("noise_result={:#?}", noise_result);
        println!("pulse_result={:#?}", pulse_result);

        assert!(tone_result.spectral_shape.flatness < noise_result.spectral_shape.flatness);
        assert!(noise_result.spectral_shape.spread_hz > tone_result.spectral_shape.spread_hz);
        assert!(
            noise_result.spectral_contrast.contrast_db < tone_result.spectral_contrast.contrast_db
        );
        assert!(
            pulse_result.temporal_shape.peak_transient_strength
                > tone_result.temporal_shape.peak_transient_strength
        );
        assert!(pulse_result.temporal.onset_density > tone_result.temporal.onset_density);
        assert!(pulse_result.temporal_shape.sustain_plateau_ratio > 0.0);
        assert!(
            pulse_result.temporal_shape.decay_time_ms > pulse_result.temporal_shape.attack_time_ms
        );
    }

    #[test]
    fn reduction_policy_is_frozen_to_expected_modes() {
        let policy = CharacterDescriptorReductionPolicy::default();

        assert_eq!(
            policy.spectral_centroid_hz,
            DescriptorReduction::MedianAcrossFrames
        );
        assert_eq!(
            policy.normalized_mel_band_profile,
            DescriptorReduction::MeanAcrossFramesNormalized
        );
        assert_eq!(policy.rms_energy, DescriptorReduction::WholeSignal);
        assert_eq!(
            policy.peak_transient_strength,
            DescriptorReduction::PeakAcrossEvents
        );
        assert_eq!(
            policy.attack_time_ms,
            DescriptorReduction::MedianAcrossEvents
        );
    }

    #[test]
    fn non_native_input_rate_preserves_descriptor_shape_under_frozen_analysis_rate() {
        let native = sine_audio(1_000.0, 2.0, 48_000, 1.0);
        let non_native = sine_audio(1_000.0, 2.0, 44_100, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let native_result = analyzer.analyze(&native);
        let non_native_result = analyzer.analyze(&non_native);

        assert!(
            (native_result.spectral_shape.centroid_hz
                - non_native_result.spectral_shape.centroid_hz)
                .abs()
                < 80.0,
            "centroid drifted from {} to {}",
            native_result.spectral_shape.centroid_hz,
            non_native_result.spectral_shape.centroid_hz,
        );
        assert!(
            (native_result.dynamics.rms_energy - non_native_result.dynamics.rms_energy).abs()
                < 0.05
        );
        assert!(
            (native_result.temporal.zero_crossing_rate_hz
                - non_native_result.temporal.zero_crossing_rate_hz)
                .abs()
                < 25.0
        );
    }
}
