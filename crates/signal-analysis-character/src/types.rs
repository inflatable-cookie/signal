use signal_analysis::Confidence;
use signal_dsp_spectral::StftConfig;
use signal_primitives::{FrameCount, SampleRate};

pub(crate) const MEL_PROFILE_BAND_COUNT: usize = 8;

/// Configuration for the offline character analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterAnalyzerConfig {
    /// STFT parameters for spectral analysis.
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
    /// Reduction applied to produce `SpectralShapeDescriptorPack::centroid_hz`.
    pub spectral_centroid_hz: DescriptorReduction,
    /// Reduction applied to produce `SpectralShapeDescriptorPack::spread_hz`.
    pub spectral_spread_hz: DescriptorReduction,
    /// Reduction applied to produce `SpectralShapeDescriptorPack::rolloff_85_hz`.
    pub spectral_rolloff_85_hz: DescriptorReduction,
    /// Reduction applied to produce `SpectralShapeDescriptorPack::rolloff_95_hz`.
    pub spectral_rolloff_95_hz: DescriptorReduction,
    /// Reduction applied to produce `SpectralShapeDescriptorPack::flatness`.
    pub spectral_flatness: DescriptorReduction,
    /// Reduction applied to produce `SpectralContrastDescriptorPack::contrast_db`.
    pub spectral_contrast_db: DescriptorReduction,
    /// Reduction applied to produce `SpectralProfileDescriptorPack::normalized_mel_band_profile`.
    pub normalized_mel_band_profile: DescriptorReduction,
    /// Reduction applied to produce `TemporalDescriptorPack::onset_density`.
    pub onset_density: DescriptorReduction,
    /// Reduction applied to produce `TemporalDescriptorPack::zero_crossing_rate_hz`.
    pub zero_crossing_rate_hz: DescriptorReduction,
    /// Reduction applied to produce `TemporalDescriptorPack::transient_density`.
    pub transient_density: DescriptorReduction,
    /// Reduction applied to produce `TemporalDescriptorPack::sustain_ratio`.
    pub sustain_ratio: DescriptorReduction,
    /// Reduction applied to produce `TemporalShapeDescriptorPack::peak_transient_strength`.
    pub peak_transient_strength: DescriptorReduction,
    /// Reduction applied to produce `TemporalShapeDescriptorPack::median_transient_strength`.
    pub median_transient_strength: DescriptorReduction,
    /// Reduction applied to produce `TemporalShapeDescriptorPack::attack_time_ms`.
    pub attack_time_ms: DescriptorReduction,
    /// Reduction applied to produce `TemporalShapeDescriptorPack::decay_time_ms`.
    pub decay_time_ms: DescriptorReduction,
    /// Reduction applied to produce `TemporalShapeDescriptorPack::sustain_plateau_ratio`.
    pub sustain_plateau_ratio: DescriptorReduction,
    /// Reduction applied to produce `DynamicsDescriptorPack::rms_energy`.
    pub rms_energy: DescriptorReduction,
    /// Reduction applied to produce `DynamicsDescriptorPack::peak_amplitude`.
    pub peak_amplitude: DescriptorReduction,
    /// Reduction applied to produce `DynamicsDescriptorPack::dynamic_range`.
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
    pub(crate) fn zero() -> Self {
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
    pub(crate) fn zero() -> Self {
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
    pub(crate) fn zero() -> Self {
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
    pub(crate) fn zero() -> Self {
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
    pub(crate) fn zero() -> Self {
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
    pub(crate) fn zero() -> Self {
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
    /// Spectral shape descriptors: centroid, spread, rolloff, and flatness.
    pub spectral_shape: SpectralShapeDescriptorPack,
    /// Spectral contrast descriptor.
    pub spectral_contrast: SpectralContrastDescriptorPack,
    /// Coarse mel-band energy profile.
    pub spectral_profile: SpectralProfileDescriptorPack,
    /// Activity-oriented temporal descriptors.
    pub temporal: TemporalDescriptorPack,
    /// Event-oriented transient and envelope-shape descriptors.
    pub temporal_shape: TemporalShapeDescriptorPack,
    /// Amplitude-domain dynamics descriptors.
    pub dynamics: DynamicsDescriptorPack,
    /// Reduction policy applied to produce each descriptor in this result.
    pub reduction_policy: CharacterDescriptorReductionPolicy,
    /// Confidence in the descriptor pack; low for short or silent buffers.
    pub confidence: Confidence,
}

impl CharacterAnalysisResult {
    pub(crate) fn zero() -> Self {
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
