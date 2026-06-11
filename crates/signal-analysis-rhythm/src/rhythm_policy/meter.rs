use signal_analysis::Confidence;

/// Evidence strengths that support the meter estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterSupportProfile {
    /// Confidence derived from whole-track meter evidence.
    pub whole_track_strength: Confidence,
    /// Confidence derived from segment-level recovery evidence.
    pub segment_recovery_strength: Confidence,
    /// Confidence derived from the duration of the recovery window.
    pub recovery_duration_strength: Confidence,
}

/// Component scores that make up the overall meter confidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterConfidenceBreakdown {
    /// Margin between the winning and runner-up downbeat phase hypotheses.
    pub phase_margin: f32,
    /// Raw onset-support score at the inferred bar length.
    pub support: f32,
    /// Normalised meter-specific support score.
    pub meter_support: f32,
    /// Beat-grid regularity within detected bars.
    pub regularity: f32,
    /// Stability of recent windowed meter estimates.
    pub recent_stability: f32,
    /// Onset salience at downbeat positions relative to other beats.
    pub salience: f32,
}

/// How the meter estimate was derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterDetectionKind {
    /// Meter was inferred from evidence spanning the full track.
    WholeTrack,
    /// Meter was recovered from a stable sub-segment after a disruption.
    SegmentRecovery,
}

/// How much confidence to place in the current meter estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterTrustLevel {
    /// Strong, consistent meter evidence across the track.
    Stable,
    /// Evidence is emerging or was recently disrupted but is rebuilding.
    Recovering,
    /// Weak or inconsistent evidence; treat the estimate as provisional.
    Tentative,
}

/// Suggested consumer action for the meter estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterRecommendation {
    /// Commit to this meter value for downstream use.
    Lock,
    /// Use tentatively; keep watching for stronger evidence.
    Monitor,
    /// Do not use; evidence is insufficient.
    Defer,
}

/// Metadata describing the recovery window used for segment-level meter detection.
#[derive(Clone, Debug, PartialEq)]
pub struct MeterRecoveryContext {
    /// Beat index of the recovery window start.
    pub start_beat_index: usize,
    /// Beat index of the recovery window end.
    pub end_beat_index: usize,
    /// Number of beats covered by the recovery window.
    pub recovered_beats: usize,
    /// Number of complete bars recovered within the window.
    pub recovered_bars: usize,
    /// Audio time at the start of the recovery window, in seconds.
    pub start_seconds: f32,
    /// Audio time at the end of the recovery window, in seconds.
    pub end_seconds: f32,
    /// Number of analysis windows that supported the recovery.
    pub supporting_windows: usize,
}

/// Meter estimate produced by the rhythm analyzer.
#[derive(Clone, Debug, PartialEq)]
pub struct MeterEstimate {
    /// Number of beats per bar (e.g. 4 for 4/4).
    pub beats_per_bar: usize,
    /// Overall meter detection confidence.
    pub confidence: Confidence,
    /// Whether the estimate came from whole-track or segment-recovery analysis.
    pub detection_kind: MeterDetectionKind,
    /// How much trust to place in the estimate.
    pub trust: MeterTrustLevel,
    /// Suggested consumer action.
    pub recommendation: MeterRecommendation,
    /// Evidence strength breakdown by detection path.
    pub support_profile: MeterSupportProfile,
    /// Component scores that compose the overall confidence.
    pub confidence_breakdown: MeterConfidenceBreakdown,
    /// Recovery context if the meter was recovered from a sub-segment.
    pub recovery: Option<MeterRecoveryContext>,
    /// Downbeat times in seconds, in ascending order.
    pub downbeat_positions_seconds: Vec<f32>,
}
