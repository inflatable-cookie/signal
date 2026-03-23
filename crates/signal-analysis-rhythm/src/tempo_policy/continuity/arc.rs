use signal_analysis::Confidence;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityAction {
    Lock,
    Retain,
    Reacquire,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySource {
    CurrentTempo,
    PriorTempo,
    CoreWindow,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityReason {
    StableTempo,
    IntegerTempoSnap,
    CoreWindowCarry,
    RevalidationDecay,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySeverity {
    Confirmed,
    Guarded,
    Fragile,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityHistory {
    Reinforcing,
    Preserving,
    Degrading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArc {
    Recovering,
    Stalling,
    Collapsing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcRationale {
    RefreshStrength,
    StableCarry,
    UnresolvedDrift,
    BoundaryDrift,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcSupport {
    pub refresh_strength: Confidence,
    pub drift_pressure: Confidence,
    pub instability_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcRecommendation {
    KeepLock,
    MonitorRecovery,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcAction {
    LockCurrentTempo,
    PreservePriorTempo,
    PreferCoreWindowTempo,
    ReacquireCurrentTempo,
    ClearTempo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeRationale {
    StabilityWindowEnd,
    BoundaryDrift,
    AmbiguityCarry,
    PriorTempoDrift,
    RepeatedFailedRevalidation,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeSupport {
    pub stability_window_pressure: Confidence,
    pub boundary_drift_pressure: Confidence,
    pub ambiguity_pressure: Confidence,
    pub failed_revalidation_pressure: Confidence,
    pub evidence_loss_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeTrend {
    Rising,
    Stable,
    Easing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeTrendRationale {
    StabilityWindowCarry,
    BoundaryEscalation,
    AmbiguityCarry,
    RevalidationDecay,
    TerminalClearPressure,
    FlatCollapse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeInflectionStage {
    FlatWindow,
    NextStage,
    TerminalClear,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeTrendSupport {
    pub current_pressure: Confidence,
    pub next_stage_pressure: Confidence,
    pub terminal_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflection {
    pub stage: TempoContinuityArcDowngradeInflectionStage,
    pub after_beats: usize,
    pub next_stage_delta: Confidence,
    pub terminal_delta: Confidence,
    pub competing_stage: Option<TempoContinuityArcDowngradeInflectionStage>,
    pub competing_after_beats: usize,
    pub competing_delta: Confidence,
    pub competing_support: Confidence,
    pub balance: TempoContinuityArcDowngradeInflectionBalance,
    pub rationale_balance: TempoContinuityArcDowngradeInflectionRationaleBalance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflectionBalance {
    pub primary_weight: Confidence,
    pub competing_weight: Confidence,
    pub unattributed_weight: Confidence,
    pub dominance: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeStageRationale {
    NoPressure,
    StabilityWindow,
    BoundaryDrift,
    AmbiguityCarry,
    PriorTempoDrift,
    RevalidationDecay,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeStageRationaleWeights {
    pub dominant: TempoContinuityArcDowngradeStageRationale,
    pub stability_window: Confidence,
    pub boundary_drift: Confidence,
    pub ambiguity_carry: Confidence,
    pub prior_tempo_drift: Confidence,
    pub revalidation_decay: Confidence,
    pub evidence_loss: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflectionRationaleBalance {
    pub primary: TempoContinuityArcDowngradeStageRationaleWeights,
    pub competing: Option<TempoContinuityArcDowngradeStageRationaleWeights>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityArcActionExpiry {
    pub guaranteed_until_beats: usize,
    pub fallback_after_beats: usize,
    pub clear_after_beats: usize,
    pub max_failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityProvenance {
    IntegerSnap,
    StableRefinedEstimate,
    GuardedRefinedEstimate,
    CoreWindowEstimate,
    PriorTempoCarry,
    NoTempo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDecision {
    pub recommendation: TempoContinuityArcRecommendation,
    pub action: TempoContinuityArcAction,
    pub severity: TempoContinuitySeverity,
    pub fallback_action: TempoContinuityArcAction,
    pub downgrade_rationale: TempoContinuityArcDowngradeRationale,
    pub downgrade_support: TempoContinuityArcDowngradeSupport,
    pub downgrade_trend: TempoContinuityArcDowngradeTrend,
    pub downgrade_trend_rationale: TempoContinuityArcDowngradeTrendRationale,
    pub downgrade_trend_support: TempoContinuityArcDowngradeTrendSupport,
    pub downgrade_inflection: TempoContinuityArcDowngradeInflection,
    pub provenance: TempoContinuityProvenance,
    pub expiry: TempoContinuityArcActionExpiry,
    pub confidence: Confidence,
}
