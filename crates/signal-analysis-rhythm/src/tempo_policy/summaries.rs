use signal_analysis::Confidence;

use super::{
    TempoConsumptionSelection, TempoContinuityAction, TempoContinuityArc,
    TempoContinuityArcRationale, TempoContinuityHistory, TempoContinuityProvenance,
    TempoContinuitySeverity, TempoContinuitySource, TempoContinuityTrigger, TempoRecommendation,
    TempoStabilityScopeSummary, TempoStateAction, TempoStateReason, TempoTrendDiagnostics,
    TempoTrustLevel,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoSegmentKind {
    WholeTrack,
    EdgeTrimmedStable,
    StableCore,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoSegmentSummary {
    pub kind: TempoSegmentKind,
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub representative_bpm: f32,
    pub drift_span_bpm: f32,
    pub mean_abs_deviation_bpm: f32,
    pub coverage: Confidence,
    pub retained_windows: usize,
    pub total_windows: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuitySummary {
    pub action: TempoStateAction,
    pub reason: TempoStateReason,
    pub confidence: Confidence,
    pub trust: TempoTrustLevel,
    pub recommendation: TempoRecommendation,
    pub continuity_action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub arc: TempoContinuityArc,
    pub arc_rationale: TempoContinuityArcRationale,
    pub trigger: TempoContinuityTrigger,
    pub provenance: TempoContinuityProvenance,
    pub ambiguity: Confidence,
    pub refresh_strength: Confidence,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub fallback_after_beats: usize,
    pub clear_after_beats: usize,
    pub current: TempoConsumptionSelection,
    pub fallback: TempoConsumptionSelection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempoStructureSummary {
    pub trust: TempoTrustLevel,
    pub recommendation: TempoRecommendation,
    pub selected_bpm: Option<f32>,
    pub snapped_bpm: Option<f32>,
    pub core_window_bpm: f32,
    pub stability_scope: TempoStabilityScopeSummary,
    pub ambiguity: Confidence,
    pub trend: TempoTrendDiagnostics,
    pub segments: Vec<TempoSegmentSummary>,
    pub continuity: TempoContinuitySummary,
}
