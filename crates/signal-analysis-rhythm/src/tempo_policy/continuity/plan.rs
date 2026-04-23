use signal_analysis::Confidence;

use super::{
    TempoContinuityAction, TempoContinuityArc, TempoContinuityArcDecision,
    TempoContinuityArcRationale, TempoContinuityArcSupport, TempoContinuityHistory,
    TempoContinuityProvenance, TempoContinuityReason, TempoContinuitySeverity,
    TempoContinuitySource,
};
use crate::tempo_policy::{TempoStabilityScopeSummary, TempoStateAction, TempoStateReason};

/// Event that triggered the current tempo continuity plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityTrigger {
    /// A revalidation window confirmed the current tempo.
    StableRevalidation,
    /// Track-boundary drift triggered a state change.
    BoundaryDrift,
    /// Persistent ambiguity is carrying the state forward without new confirmation.
    AmbiguityCarry,
    /// The prior tempo drifted enough to require a transition.
    PriorTempoDrift,
    /// Evidence was lost, forcing a state downgrade.
    EvidenceLoss,
}

/// Duration of the current unresolved tempo continuity gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityUnresolvedSpan {
    /// Number of beats without a successful revalidation.
    pub beats: usize,
    /// Count of consecutive failed revalidation attempts.
    pub failed_revalidations: usize,
}

/// Root cause contributing to a tempo continuity plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityCause {
    /// Strong, stable tempo evidence is the primary driver.
    StableTempoEvidence,
    /// Track-boundary drift is contributing to the transition.
    BoundaryDrift,
    /// Tempo ambiguity is undermining confidence.
    TempoAmbiguity,
    /// The prior tempo is being carried forward.
    PriorTempoCarry,
    /// The core-window estimate is carrying the state.
    CoreWindowCarry,
    /// Evidence was lost.
    EvidenceLoss,
}

/// Ordered stack of causes driving a tempo continuity plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityCauseStack {
    /// Dominant cause.
    pub primary: TempoContinuityCause,
    /// Up to two contributing secondary causes.
    pub secondary: [Option<TempoContinuityCause>; 2],
    /// Total number of active causes (including primary).
    pub count: usize,
}

/// Beat-count thresholds governing when a continuity plan expires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityExpiry {
    /// The plan is guaranteed through at least this many beats.
    pub guaranteed_until_beats: usize,
    /// Beats after which the plan downgrades (e.g. Lock → Monitor).
    pub downgrade_after_beats: usize,
    /// Beats after which the state is cleared entirely.
    pub clear_after_beats: usize,
    /// Maximum consecutive failed revalidations before clearing.
    pub max_failed_revalidations: usize,
}

/// A single planned tempo continuity transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityTransition {
    /// Number of beats after which this transition activates.
    pub after_beats: usize,
    /// Continuity action to apply at transition time.
    pub action: TempoContinuityAction,
    /// Tempo source at transition time.
    pub source: TempoContinuitySource,
    /// Confidence level at transition time.
    pub severity: TempoContinuitySeverity,
    /// Historical trend leading into this transition.
    pub history: TempoContinuityHistory,
    /// Reason for the transition.
    pub reason: TempoContinuityReason,
    /// Event that triggered the transition.
    pub trigger: TempoContinuityTrigger,
    /// Duration of the unresolved gap at transition time.
    pub unresolved: TempoContinuityUnresolvedSpan,
    /// Ordered cause stack driving the transition.
    pub causes: TempoContinuityCauseStack,
    /// Origin of the BPM value at transition time.
    pub provenance: TempoContinuityProvenance,
    /// Confidence score at transition time.
    pub confidence: Confidence,
    /// Refresh strength at transition time.
    pub refresh_strength: Confidence,
}

/// Planned lifecycle transitions for a tempo continuity plan.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityLifecycle {
    /// The refresh transition that reinforces the plan on revalidation.
    pub refresh: TempoContinuityTransition,
    /// The two decay transitions (downgrade then clear) if revalidation fails.
    pub decay: [TempoContinuityTransition; 2],
}

/// Full tempo continuity plan for a single analysis pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityPlan {
    /// Action to apply immediately.
    pub action: TempoContinuityAction,
    /// Source of the tempo value being carried.
    pub source: TempoContinuitySource,
    /// Current confidence level.
    pub severity: TempoContinuitySeverity,
    /// Recent history trend.
    pub history: TempoContinuityHistory,
    /// Continuity arc trajectory.
    pub arc: TempoContinuityArc,
    /// Dominant factor shaping the arc.
    pub arc_rationale: TempoContinuityArcRationale,
    /// Pressure components feeding the arc.
    pub arc_support: TempoContinuityArcSupport,
    /// Complete arc decision including downgrade projections.
    pub arc_decision: TempoContinuityArcDecision,
    /// Reason for the current state.
    pub reason: TempoContinuityReason,
    /// Event that triggered this plan.
    pub trigger: TempoContinuityTrigger,
    /// Duration of the current unresolved gap.
    pub unresolved: TempoContinuityUnresolvedSpan,
    /// Ordered cause stack.
    pub causes: TempoContinuityCauseStack,
    /// Origin of the BPM value being carried.
    pub provenance: TempoContinuityProvenance,
    /// Confidence score.
    pub confidence: Confidence,
    /// Strength with which the current evidence refreshes continuity.
    pub refresh_strength: Confidence,
    /// Number of beats this plan is fully trusted.
    pub trusted_beats: usize,
    /// Number of beats before a revalidation check is expected.
    pub revalidate_after_beats: usize,
    /// Beat-count expiry thresholds.
    pub expiry: TempoContinuityExpiry,
    /// Planned future transitions.
    pub lifecycle: TempoContinuityLifecycle,
}

/// Top-level tempo state recommendation returned by the analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStateRecommendation {
    /// High-level action for the tempo value.
    pub action: TempoStateAction,
    /// Reason that drove the action decision.
    pub reason: TempoStateReason,
    /// Overall tempo confidence.
    pub confidence: Confidence,
    /// Full continuity plan including arc and lifecycle details.
    pub continuity: TempoContinuityPlan,
}

/// Which tempo value is being exposed to consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoConsumptionSource {
    /// The integer-snapped current tempo.
    SnappedCurrentTempo,
    /// The refined (beat-interval) current tempo.
    RefinedCurrentTempo,
    /// The core-window (edge-trimmed) tempo.
    CoreWindowTempo,
    /// The tempo from the prior analysis pass.
    PriorTempo,
    /// No tempo value is available.
    NoTempo,
}

/// A BPM value together with its source, used by consumers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoConsumptionSelection {
    /// The BPM to use, or `None` if no tempo is available.
    pub bpm: Option<f32>,
    /// Where this BPM value came from.
    pub source: TempoConsumptionSource,
}

/// Consumption decision returned by
/// [`BeatAnalysisResult::tempo_consumption`](crate::BeatAnalysisResult::tempo_consumption).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoConsumptionDecision {
    /// High-level action for the tempo.
    pub action: TempoStateAction,
    /// Reason for the action.
    pub reason: TempoStateReason,
    /// Continuity-layer action.
    pub continuity_action: TempoContinuityAction,
    /// Overall tempo confidence.
    pub confidence: Confidence,
    /// Stability scope classification and evidence.
    pub stability_scope: TempoStabilityScopeSummary,
    /// The BPM recommended for immediate use.
    pub current: TempoConsumptionSelection,
    /// The BPM to use after `fallback_after_beats`.
    pub fallback: TempoConsumptionSelection,
    /// Number of beats before `fallback` should be used instead of `current`.
    pub fallback_after_beats: usize,
}
