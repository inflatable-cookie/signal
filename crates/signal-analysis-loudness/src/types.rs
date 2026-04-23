use signal_analysis::Confidence;
use signal_primitives::{ChannelCount, SampleRate};

/// Configuration for the offline loudness meter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeterConfig {
    /// Delivery target loudness, in LUFS. Used to compute `target_offset_lu` in results.
    pub target_lufs: f32,
    /// Momentary-loudness measurement block size, in seconds (EBU R 128: 0.4 s).
    pub block_seconds: f32,
    /// Analysis hop size shared by momentary and short-term passes, in seconds.
    pub hop_seconds: f32,
    /// Short-term loudness window size, in seconds (EBU R 128: 3.0 s).
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
    /// Gated integrated program loudness, in LUFS (EBU R 128 / BS.1770-4).
    pub integrated_lufs: f32,
    /// Loudness range across the analysed span, in LU (EBU R 128 LRA).
    pub loudness_range_lu: f32,
    /// Maximum true-peak level across all channels, in dBTP.
    pub true_peak_dbtp: f32,
    /// Confidence in the reported metrics; low for short or silent buffers.
    pub confidence: Confidence,
    /// Per-channel loudness summaries before aggregation.
    pub channels: Vec<LoudnessChannelSummary>,
    /// Aggregation contract used for this analysis run.
    pub aggregation: LoudnessAggregationSummary,
    /// Momentary loudness trace at `block_seconds` windows.
    pub momentary_trace: LoudnessTrace,
    /// Short-term loudness trace at `short_term_seconds` windows.
    pub short_term_trace: LoudnessTrace,
    /// Compact delivery-facing dynamics summary.
    pub dynamics: LoudnessDynamicsSummary,
}

/// Channel-weighting contract applied during loudness aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessChannelWeightSource {
    /// Single channel; weight is 1.0 applied directly.
    MonoDirect,
    /// Two channels; equal 1.0 weight per ITU-R BS.1770.
    StereoEqualWeight,
    /// Channel count not explicitly covered; equal 1.0 weights as fallback.
    GenericCountFallback,
}

/// Loudness weighting support used at the configured analysis sample rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessSampleRateSupport {
    /// Source was already 48 kHz; K-weighting applied directly.
    Native48kKWeighted,
    /// Source was resampled to 48 kHz before K-weighting.
    ResampledTo48kKWeighted,
    /// Analysis rate is not 48 kHz; K-weighting filter was skipped.
    UnweightedFallback,
}

/// Per-channel loudness evidence before cross-channel aggregation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessChannelSummary {
    /// Zero-based channel index.
    pub index: usize,
    /// Aggregation weight applied to this channel.
    pub weight: f32,
    /// Gated integrated loudness for this channel alone, in LUFS.
    pub integrated_lufs: f32,
    /// True-peak level for this channel, in dBTP.
    pub true_peak_dbtp: f32,
}

/// Summary of the loudness aggregation contract used for the current buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessAggregationSummary {
    /// Number of channels present in the source buffer.
    pub channel_count: ChannelCount,
    /// Channel-weighting strategy that was applied.
    pub channel_weight_source: LoudnessChannelWeightSource,
    /// K-weighting filter support level for the analysis sample rate.
    pub sample_rate_support: LoudnessSampleRateSupport,
    /// Sample rate at which analysis was performed.
    pub analysis_sample_rate: SampleRate,
    /// Oversampling factor used for true-peak estimation.
    pub true_peak_oversample_factor: usize,
}

/// One loudness trace point over a fixed analysis window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessTracePoint {
    /// Zero-based point index in the trace.
    pub index: usize,
    /// Window start time relative to the analysed buffer, in seconds.
    pub start_seconds: f32,
    /// Window end time relative to the analysed buffer, in seconds.
    pub end_seconds: f32,
    /// Loudness for this window, in LUFS.
    pub loudness_lufs: f32,
}

/// Time-series loudness trace over one fixed window size.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessTrace {
    /// Analysis window size for each point in this trace, in seconds.
    pub window_seconds: f32,
    /// Hop size between adjacent trace points, in seconds.
    pub hop_seconds: f32,
    /// Ordered trace points.
    pub points: Vec<LoudnessTracePoint>,
}

/// Compact delivery-facing dynamics summary built on the trace surfaces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessDynamicsSummary {
    /// Difference between `integrated_lufs` and the configured target, in LU.
    pub target_offset_lu: f32,
    /// Difference between `true_peak_dbtp` and `integrated_lufs`, in LU.
    pub peak_to_loudness_lu: f32,
    /// Loudest momentary block in the trace, in LUFS.
    pub momentary_max_lufs: f32,
    /// Loudest short-term block in the trace, in LUFS.
    pub short_term_max_lufs: f32,
    /// Range between loudest and quietest momentary blocks, in LU.
    pub momentary_range_lu: f32,
    /// Range between loudest and quietest short-term blocks, in LU.
    pub short_term_range_lu: f32,
}

/// Bounded loudness subset intended for runtime-diagnostics reuse.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessRuntimeDiagnosticsSummary {
    /// Gated integrated program loudness, in LUFS.
    pub integrated_lufs: f32,
    /// Maximum true-peak level across all channels, in dBTP.
    pub true_peak_dbtp: f32,
    /// Offset from the configured delivery target, in LU.
    pub target_offset_lu: f32,
    /// True-peak minus integrated loudness, in LU.
    pub peak_to_loudness_lu: f32,
    /// Loudness of the most recent momentary block, in LUFS.
    pub current_momentary_lufs: f32,
    /// Loudness of the most recent short-term block, in LUFS.
    pub current_short_term_lufs: f32,
    /// Loudest momentary block seen so far, in LUFS.
    pub momentary_max_lufs: f32,
    /// Loudest short-term block seen so far, in LUFS.
    pub short_term_max_lufs: f32,
    /// Tail of the momentary trace bounded to the most recent few points.
    pub recent_momentary: LoudnessTrace,
    /// Tail of the short-term trace bounded to the most recent few points.
    pub recent_short_term: LoudnessTrace,
}
