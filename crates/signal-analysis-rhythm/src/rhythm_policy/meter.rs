use signal_analysis::Confidence;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterSupportProfile {
    pub whole_track_strength: Confidence,
    pub segment_recovery_strength: Confidence,
    pub recovery_duration_strength: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterConfidenceBreakdown {
    pub phase_margin: f32,
    pub support: f32,
    pub meter_support: f32,
    pub regularity: f32,
    pub recent_stability: f32,
    pub salience: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterDetectionKind {
    WholeTrack,
    SegmentRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterTrustLevel {
    Stable,
    Recovering,
    Tentative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterRecommendation {
    Lock,
    Monitor,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateAction {
    Lock,
    Hold,
    Watch,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateReason {
    StableMeter,
    RecoveringMeter,
    TentativeMeter,
    DestabilizedHold,
    RecoveryEmerging,
    MeterCleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityAction {
    Lock,
    Retain,
    Reacquire,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySource {
    CurrentMeter,
    PriorMeter,
    RecoveryWindow,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySeverity {
    Confirmed,
    Guarded,
    Fragile,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityReason {
    StableEvidence,
    TentativeEvidence,
    PriorStateCarry,
    RecoveryWindowSupport,
    PhaseDisplacement,
    RevalidationDecay,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeterContinuityTrigger {
    StableRevalidation,
    TentativeCarry,
    PhaseRecovery,
    PriorStateDrift,
    RecoveryWindowDrift,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityUnresolvedSpan {
    pub beats: usize,
    pub bars: usize,
    pub failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityCause {
    StableMeterEvidence,
    TempoAmbiguity,
    PhaseDisplacement,
    SparseMeterSupport,
    IrregularBarStructure,
    PriorContinuityCarry,
    RecoveryWindowInstability,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityCauseStack {
    pub primary: MeterContinuityCause,
    pub secondary: [Option<MeterContinuityCause>; 2],
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityHistory {
    Reinforcing,
    Preserving,
    Degrading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArc {
    Recovering,
    Stalling,
    Collapsing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArcRationale {
    RefreshStrength,
    StableCarry,
    UnresolvedDrift,
    StructuralInstability,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityArcSupport {
    pub refresh_strength: Confidence,
    pub drift_pressure: Confidence,
    pub structural_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityTransition {
    pub after_beats: usize,
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub severity: MeterContinuitySeverity,
    pub history: MeterContinuityHistory,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityLifecycle {
    pub refresh: MeterContinuityTransition,
    pub decay: [MeterContinuityTransition; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityPlan {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub severity: MeterContinuitySeverity,
    pub history: MeterContinuityHistory,
    pub arc: MeterContinuityArc,
    pub arc_rationale: MeterContinuityArcRationale,
    pub arc_support: MeterContinuityArcSupport,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub lifecycle: MeterContinuityLifecycle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityRecommendation {
    pub bar_length: MeterContinuityPlan,
    pub downbeat_phase: MeterContinuityPlan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterStateRecommendation {
    pub action: MeterStateAction,
    pub reason: MeterStateReason,
    pub confidence: Confidence,
    pub continuity: MeterContinuityRecommendation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeterRecoveryContext {
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub recovered_beats: usize,
    pub recovered_bars: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub supporting_windows: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeterEstimate {
    pub beats_per_bar: usize,
    pub confidence: Confidence,
    pub detection_kind: MeterDetectionKind,
    pub trust: MeterTrustLevel,
    pub recommendation: MeterRecommendation,
    pub support_profile: MeterSupportProfile,
    pub confidence_breakdown: MeterConfidenceBreakdown,
    pub recovery: Option<MeterRecoveryContext>,
    pub downbeat_positions_seconds: Vec<f32>,
}
