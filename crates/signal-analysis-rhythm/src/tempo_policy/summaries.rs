use signal_analysis::Confidence;

use super::{
    TempoConsumptionSelection, TempoContinuityAction, TempoContinuityArc,
    TempoContinuityArcRationale, TempoContinuityHistory, TempoContinuityProvenance,
    TempoContinuitySeverity, TempoContinuitySource, TempoContinuityTrigger, TempoRecommendation,
    TempoStabilityScopeSummary, TempoStateAction, TempoStateReason, TempoTrendDiagnostics,
    TempoTrustLevel,
};

/// Which part of the track a tempo segment covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoSegmentKind {
    /// Segment spans the entire track.
    WholeTrack,
    /// Segment spans the stable region after trimming unstable edges.
    EdgeTrimmedStable,
    /// Segment spans the most stable interior core only.
    StableCore,
}

/// Tempo statistics for one track segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoSegmentSummary {
    /// Which segment of the track this covers.
    pub kind: TempoSegmentKind,
    /// First beat index of the segment.
    pub start_beat_index: usize,
    /// Last beat index of the segment.
    pub end_beat_index: usize,
    /// Audio time at the start of the segment, in seconds.
    pub start_seconds: f32,
    /// Audio time at the end of the segment, in seconds.
    pub end_seconds: f32,
    /// Median windowed BPM within the segment.
    pub representative_bpm: f32,
    /// Peak-to-peak BPM range within the segment.
    pub drift_span_bpm: f32,
    /// Mean absolute deviation from the segment median, in BPM.
    pub mean_abs_deviation_bpm: f32,
    /// Fraction of the full track covered by this segment.
    pub coverage: Confidence,
    /// Number of analysis windows retained within the segment.
    pub retained_windows: usize,
    /// Total number of analysis windows in the segment.
    pub total_windows: usize,
}

/// Flattened tempo continuity state for embedding in a structure summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuitySummary {
    /// High-level tempo state action.
    pub action: TempoStateAction,
    /// Reason for the tempo state action.
    pub reason: TempoStateReason,
    /// Overall tempo confidence.
    pub confidence: Confidence,
    /// Trust level of the tempo estimate.
    pub trust: TempoTrustLevel,
    /// Interpretation-layer recommendation.
    pub recommendation: TempoRecommendation,
    /// Continuity-layer action.
    pub continuity_action: TempoContinuityAction,
    /// Source of the tempo value being carried.
    pub source: TempoContinuitySource,
    /// Confidence level of the continuity state.
    pub severity: TempoContinuitySeverity,
    /// Historical trend of the continuity state.
    pub history: TempoContinuityHistory,
    /// Continuity arc trajectory.
    pub arc: TempoContinuityArc,
    /// Dominant factor shaping the arc.
    pub arc_rationale: TempoContinuityArcRationale,
    /// Event that triggered the current transition.
    pub trigger: TempoContinuityTrigger,
    /// Origin of the tempo value being carried.
    pub provenance: TempoContinuityProvenance,
    /// Ambiguity of the tempo estimate (higher = more ambiguous).
    pub ambiguity: Confidence,
    /// Strength with which the current evidence refreshes continuity.
    pub refresh_strength: Confidence,
    /// Number of beats this state is fully trusted.
    pub trusted_beats: usize,
    /// Number of beats before the next revalidation check.
    pub revalidate_after_beats: usize,
    /// Number of beats before the state downgrades to fallback.
    pub fallback_after_beats: usize,
    /// Number of beats before the state is cleared entirely.
    pub clear_after_beats: usize,
    /// The BPM value and source recommended for immediate use.
    pub current: TempoConsumptionSelection,
    /// The BPM value and source to use after `fallback_after_beats`.
    pub fallback: TempoConsumptionSelection,
}

/// High-level tempo structure summary returned by
/// [`BeatAnalysisResult::tempo_structure_summary`](crate::BeatAnalysisResult::tempo_structure_summary).
#[derive(Clone, Debug, PartialEq)]
pub struct TempoStructureSummary {
    /// Trust level for the tempo estimate.
    pub trust: TempoTrustLevel,
    /// Suggested consumer action.
    pub recommendation: TempoRecommendation,
    /// The BPM value selected for use (may be absent if deferred).
    pub selected_bpm: Option<f32>,
    /// Integer-snapped BPM if snapping was recommended.
    pub snapped_bpm: Option<f32>,
    /// Core-window (edge-trimmed) median BPM.
    pub core_window_bpm: f32,
    /// Stability scope classification and its evidence.
    pub stability_scope: TempoStabilityScopeSummary,
    /// Ambiguity of the tempo estimate.
    pub ambiguity: Confidence,
    /// Overall tempo trend across the track.
    pub trend: TempoTrendDiagnostics,
    /// Per-segment tempo statistics (whole track, edge-trimmed, core).
    pub segments: Vec<TempoSegmentSummary>,
    /// Flattened continuity state.
    pub continuity: TempoContinuitySummary,
}
