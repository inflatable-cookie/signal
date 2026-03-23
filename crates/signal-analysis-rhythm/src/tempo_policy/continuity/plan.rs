use signal_analysis::Confidence;

use super::{
    TempoContinuityAction, TempoContinuityArc, TempoContinuityArcDecision,
    TempoContinuityArcRationale, TempoContinuityArcSupport, TempoContinuityHistory,
    TempoContinuityProvenance, TempoContinuityReason, TempoContinuitySeverity,
    TempoContinuitySource,
};
use crate::tempo_policy::{TempoStabilityScopeSummary, TempoStateAction, TempoStateReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityTrigger {
    StableRevalidation,
    BoundaryDrift,
    AmbiguityCarry,
    PriorTempoDrift,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityUnresolvedSpan {
    pub beats: usize,
    pub failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityCause {
    StableTempoEvidence,
    BoundaryDrift,
    TempoAmbiguity,
    PriorTempoCarry,
    CoreWindowCarry,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityCauseStack {
    pub primary: TempoContinuityCause,
    pub secondary: [Option<TempoContinuityCause>; 2],
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityExpiry {
    pub guaranteed_until_beats: usize,
    pub downgrade_after_beats: usize,
    pub clear_after_beats: usize,
    pub max_failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityTransition {
    pub after_beats: usize,
    pub action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub reason: TempoContinuityReason,
    pub trigger: TempoContinuityTrigger,
    pub unresolved: TempoContinuityUnresolvedSpan,
    pub causes: TempoContinuityCauseStack,
    pub provenance: TempoContinuityProvenance,
    pub confidence: Confidence,
    pub refresh_strength: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityLifecycle {
    pub refresh: TempoContinuityTransition,
    pub decay: [TempoContinuityTransition; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityPlan {
    pub action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub arc: TempoContinuityArc,
    pub arc_rationale: TempoContinuityArcRationale,
    pub arc_support: TempoContinuityArcSupport,
    pub arc_decision: TempoContinuityArcDecision,
    pub reason: TempoContinuityReason,
    pub trigger: TempoContinuityTrigger,
    pub unresolved: TempoContinuityUnresolvedSpan,
    pub causes: TempoContinuityCauseStack,
    pub provenance: TempoContinuityProvenance,
    pub confidence: Confidence,
    pub refresh_strength: Confidence,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub expiry: TempoContinuityExpiry,
    pub lifecycle: TempoContinuityLifecycle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStateRecommendation {
    pub action: TempoStateAction,
    pub reason: TempoStateReason,
    pub confidence: Confidence,
    pub continuity: TempoContinuityPlan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoConsumptionSource {
    SnappedCurrentTempo,
    RefinedCurrentTempo,
    CoreWindowTempo,
    PriorTempo,
    NoTempo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoConsumptionSelection {
    pub bpm: Option<f32>,
    pub source: TempoConsumptionSource,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoConsumptionDecision {
    pub action: TempoStateAction,
    pub reason: TempoStateReason,
    pub continuity_action: TempoContinuityAction,
    pub confidence: Confidence,
    pub stability_scope: TempoStabilityScopeSummary,
    pub current: TempoConsumptionSelection,
    pub fallback: TempoConsumptionSelection,
    pub fallback_after_beats: usize,
}
