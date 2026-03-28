use signal_analysis::Confidence;
use signal_primitives::{ChannelCount, SampleRate};

/// Configuration for the offline loudness meter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeterConfig {
    pub target_lufs: f32,
    pub block_seconds: f32,
    pub hop_seconds: f32,
    pub short_term_seconds: f32,
    /// Sample rate used by the loudness analysis path after input prep.
    ///
    /// Loudness weighting and confidence are currently calibrated for 48 kHz,
    /// so the default profiles freeze that rate and resample inputs when
    /// needed instead of silently degrading on non-48k material.
    pub analysis_sample_rate: SampleRate,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed (spec-compliant integrated
    /// LUFS).  Setting a value gives a faster estimate that may differ from
    /// the true whole-programme loudness.
    pub analysis_duration_seconds: Option<u32>,
}

impl LoudnessMeterConfig {
    /// Quick scanning profile — analyses a 30-second centre segment.
    pub fn low() -> Self {
        Self {
            analysis_duration_seconds: Some(30),
            ..Self::default()
        }
    }

    /// Balanced profile — analyses a 60-second centre segment.
    pub fn medium() -> Self {
        Self {
            analysis_duration_seconds: Some(60),
            ..Self::default()
        }
    }

    /// Full-accuracy profile — analyses the entire track.
    pub fn high() -> Self {
        Self::default()
    }
}

impl Default for LoudnessMeterConfig {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            block_seconds: 0.400,
            hop_seconds: 0.100,
            short_term_seconds: 3.0,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
        }
    }
}

/// Summary loudness metrics for one analyzed buffer.
///
/// Practical integration order:
/// 1. Read `integrated_lufs` as the program-level loudness figure.
/// 2. Read `loudness_range_lu` to gauge macro dynamics across the analyzed span.
/// 3. Read `true_peak_dbtp` before applying delivery or limiter decisions.
/// 4. Read `confidence` to determine whether the buffer was long and energetic
///    enough for the reported numbers to be treated as stable.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessAnalysisResult {
    pub integrated_lufs: f32,
    pub loudness_range_lu: f32,
    pub true_peak_dbtp: f32,
    pub confidence: Confidence,
    pub channels: Vec<LoudnessChannelSummary>,
    pub aggregation: LoudnessAggregationSummary,
    pub momentary_trace: LoudnessTrace,
    pub short_term_trace: LoudnessTrace,
    pub dynamics: LoudnessDynamicsSummary,
}

/// Channel-weighting contract applied during loudness aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessChannelWeightSource {
    MonoDirect,
    StereoEqualWeight,
    GenericCountFallback,
}

/// Loudness weighting support used at the configured analysis sample rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessSampleRateSupport {
    Native48kKWeighted,
    ResampledTo48kKWeighted,
    UnweightedFallback,
}

/// Per-channel loudness evidence before cross-channel aggregation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessChannelSummary {
    pub index: usize,
    pub weight: f32,
    pub integrated_lufs: f32,
    pub true_peak_dbtp: f32,
}

/// Summary of the loudness aggregation contract used for the current buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessAggregationSummary {
    pub channel_count: ChannelCount,
    pub channel_weight_source: LoudnessChannelWeightSource,
    pub sample_rate_support: LoudnessSampleRateSupport,
    pub analysis_sample_rate: SampleRate,
    pub true_peak_oversample_factor: usize,
}

/// One loudness trace point over a fixed analysis window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessTracePoint {
    pub index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub loudness_lufs: f32,
}

/// Time-series loudness trace over one fixed window size.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessTrace {
    pub window_seconds: f32,
    pub hop_seconds: f32,
    pub points: Vec<LoudnessTracePoint>,
}

/// Compact delivery-facing dynamics summary built on the trace surfaces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessDynamicsSummary {
    pub target_offset_lu: f32,
    pub peak_to_loudness_lu: f32,
    pub momentary_max_lufs: f32,
    pub short_term_max_lufs: f32,
    pub momentary_range_lu: f32,
    pub short_term_range_lu: f32,
}

/// Bounded loudness subset intended for runtime-diagnostics reuse.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessRuntimeDiagnosticsSummary {
    pub integrated_lufs: f32,
    pub true_peak_dbtp: f32,
    pub target_offset_lu: f32,
    pub peak_to_loudness_lu: f32,
    pub current_momentary_lufs: f32,
    pub current_short_term_lufs: f32,
    pub momentary_max_lufs: f32,
    pub short_term_max_lufs: f32,
    pub recent_momentary: LoudnessTrace,
    pub recent_short_term: LoudnessTrace,
}
