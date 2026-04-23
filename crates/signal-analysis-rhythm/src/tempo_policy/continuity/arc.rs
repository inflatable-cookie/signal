use signal_analysis::Confidence;

/// Action applied to the tempo continuity state on the next pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityAction {
    /// Hold the current tempo with full confidence.
    Lock,
    /// Keep the current tempo while evidence is re-evaluated.
    Retain,
    /// Attempt to recover a recently lost tempo.
    Reacquire,
    /// Discard the continuity state entirely.
    Clear,
}

/// Which tempo value is being carried forward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySource {
    /// The tempo just estimated in this pass.
    CurrentTempo,
    /// The tempo from the preceding pass.
    PriorTempo,
    /// The core-window estimate is being used as a stable reference.
    CoreWindow,
    /// No tempo value is being carried.
    Cleared,
}

/// Reason for the current tempo continuity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityReason {
    /// Consistent, well-supported tempo evidence.
    StableTempo,
    /// An integer snap locked the tempo.
    IntegerTempoSnap,
    /// The core-window estimate is carrying the state forward.
    CoreWindowCarry,
    /// Confidence is decaying because revalidation has not succeeded.
    RevalidationDecay,
    /// Not enough evidence to carry the state.
    InsufficientEvidence,
}

/// Confidence level of the tempo continuity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySeverity {
    /// High confidence; the tempo is well supported.
    Confirmed,
    /// Moderate confidence; some evidence of instability.
    Guarded,
    /// Low confidence; the tempo may be about to be lost.
    Fragile,
    /// No active tempo continuity state.
    Cleared,
}

/// Direction of the tempo continuity state over recent passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityHistory {
    /// Confidence has been improving across recent passes.
    Reinforcing,
    /// Confidence is stable with no clear trend.
    Preserving,
    /// Confidence has been declining across recent passes.
    Degrading,
}

/// Trajectory of the tempo continuity arc across the current window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArc {
    /// Evidence is improving and continuity is being rebuilt.
    Recovering,
    /// Evidence is not improving; continuity is stagnant.
    Stalling,
    /// Evidence is deteriorating; continuity is about to be lost.
    Collapsing,
}

/// Dominant factor that shaped the tempo continuity arc decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcRationale {
    /// High refresh strength is driving recovery.
    RefreshStrength,
    /// A stable carry from the prior state is sustaining the arc.
    StableCarry,
    /// Accumulated unresolved drift is pushing the arc toward collapse.
    UnresolvedDrift,
    /// Track-boundary drift is the primary pressure source.
    BoundaryDrift,
    /// Evidence loss is the primary driver.
    EvidenceLoss,
}

/// Pressure components feeding into the tempo continuity arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcSupport {
    /// How strongly the current evidence is refreshing continuity.
    pub refresh_strength: Confidence,
    /// Pressure from accumulated unresolved tempo drift.
    pub drift_pressure: Confidence,
    /// Pressure from beat-grid instability.
    pub instability_pressure: Confidence,
}

/// High-level recommendation from the arc evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcRecommendation {
    /// Maintain the current lock without change.
    KeepLock,
    /// Switch to monitoring mode while recovery is attempted.
    MonitorRecovery,
    /// Discard the lock and clear continuity.
    Clear,
}

/// Concrete action taken for the arc at the current analysis pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcAction {
    /// Lock and use the current tempo estimate.
    LockCurrentTempo,
    /// Continue using the tempo from the prior pass.
    PreservePriorTempo,
    /// Use the core-window estimate in preference to the current estimate.
    PreferCoreWindowTempo,
    /// Attempt to re-lock the current tempo after a disruption.
    ReacquireCurrentTempo,
    /// Remove the held tempo value.
    ClearTempo,
}

/// Primary reason the arc is downgrading toward collapse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeRationale {
    /// The stability window has ended without recovery.
    StabilityWindowEnd,
    /// Track-boundary drift is eroding confidence.
    BoundaryDrift,
    /// Persistent ambiguity is preventing revalidation.
    AmbiguityCarry,
    /// The prior tempo has drifted away from the current estimate.
    PriorTempoDrift,
    /// Multiple consecutive revalidation attempts have failed.
    RepeatedFailedRevalidation,
    /// Evidence was lost.
    EvidenceLoss,
}

/// Pressure scores feeding into the downgrade decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeSupport {
    /// Pressure from the stability window approaching its end.
    pub stability_window_pressure: Confidence,
    /// Pressure from track-boundary drift.
    pub boundary_drift_pressure: Confidence,
    /// Pressure from persistent ambiguity.
    pub ambiguity_pressure: Confidence,
    /// Pressure from consecutive failed revalidations.
    pub failed_revalidation_pressure: Confidence,
    /// Pressure from evidence loss.
    pub evidence_loss_pressure: Confidence,
}

/// Whether downgrade pressure is increasing, stable, or easing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeTrend {
    /// Downgrade pressure is increasing.
    Rising,
    /// Downgrade pressure is unchanged.
    Stable,
    /// Downgrade pressure is decreasing.
    Easing,
}

/// Reason for the current downgrade trend direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeTrendRationale {
    /// The stability window is carrying existing pressure forward.
    StabilityWindowCarry,
    /// Boundary drift is escalating the pressure.
    BoundaryEscalation,
    /// Persistent ambiguity is sustaining the trend.
    AmbiguityCarry,
    /// Revalidation failures are accumulating.
    RevalidationDecay,
    /// Terminal clear pressure is dominating.
    TerminalClearPressure,
    /// A flat collapse with no dominant single cause.
    FlatCollapse,
}

/// Stage that will be reached at the next downgrade inflection point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeInflectionStage {
    /// Pressure will remain flat within the current window.
    FlatWindow,
    /// The arc will move to the next downgrade stage.
    NextStage,
    /// The arc will reach a terminal clear.
    TerminalClear,
}

/// Pressure at each stage of the downgrade trend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeTrendSupport {
    /// Pressure at the current stage.
    pub current_pressure: Confidence,
    /// Pressure at the next projected stage.
    pub next_stage_pressure: Confidence,
    /// Pressure at the terminal clear stage.
    pub terminal_pressure: Confidence,
}

/// Projected inflection point in the downgrade arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflection {
    /// The primary stage expected at the next inflection.
    pub stage: TempoContinuityArcDowngradeInflectionStage,
    /// Beats until the primary inflection.
    pub after_beats: usize,
    /// Confidence delta when transitioning to the next stage.
    pub next_stage_delta: Confidence,
    /// Confidence delta when reaching the terminal clear.
    pub terminal_delta: Confidence,
    /// A competing inflection stage, if one exists.
    pub competing_stage: Option<TempoContinuityArcDowngradeInflectionStage>,
    /// Beats until the competing inflection.
    pub competing_after_beats: usize,
    /// Confidence delta for the competing inflection.
    pub competing_delta: Confidence,
    /// Support weight of the competing inflection.
    pub competing_support: Confidence,
    /// Weight balance between the primary and competing inflections.
    pub balance: TempoContinuityArcDowngradeInflectionBalance,
    /// Rationale weights for both inflections.
    pub rationale_balance: TempoContinuityArcDowngradeInflectionRationaleBalance,
}

/// Weight distribution between competing downgrade inflection stages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflectionBalance {
    /// Weight attributed to the primary inflection stage.
    pub primary_weight: Confidence,
    /// Weight attributed to the competing inflection stage.
    pub competing_weight: Confidence,
    /// Weight not attributed to either stage.
    pub unattributed_weight: Confidence,
    /// Degree to which the primary stage dominates the competing stage.
    pub dominance: Confidence,
}

/// Dominant reason for a specific downgrade stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityArcDowngradeStageRationale {
    /// No significant downgrade pressure at this stage.
    NoPressure,
    /// Stability window expiry is the primary pressure source.
    StabilityWindow,
    /// Boundary drift is the primary pressure source.
    BoundaryDrift,
    /// Ambiguity carry is the primary pressure source.
    AmbiguityCarry,
    /// Prior tempo drift is the primary pressure source.
    PriorTempoDrift,
    /// Revalidation decay is the primary pressure source.
    RevalidationDecay,
    /// Evidence loss is the primary pressure source.
    EvidenceLoss,
}

/// Per-cause pressure weights for a downgrade stage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeStageRationaleWeights {
    /// The dominant cause at this stage.
    pub dominant: TempoContinuityArcDowngradeStageRationale,
    /// Weight from stability window expiry.
    pub stability_window: Confidence,
    /// Weight from boundary drift.
    pub boundary_drift: Confidence,
    /// Weight from ambiguity carry.
    pub ambiguity_carry: Confidence,
    /// Weight from prior tempo drift.
    pub prior_tempo_drift: Confidence,
    /// Weight from revalidation decay.
    pub revalidation_decay: Confidence,
    /// Weight from evidence loss.
    pub evidence_loss: Confidence,
}

/// Rationale weights for both the primary and competing inflection stages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDowngradeInflectionRationaleBalance {
    /// Rationale weights for the primary stage.
    pub primary: TempoContinuityArcDowngradeStageRationaleWeights,
    /// Rationale weights for the competing stage, if one exists.
    pub competing: Option<TempoContinuityArcDowngradeStageRationaleWeights>,
}

/// Beat-count thresholds governing when an arc action expires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityArcActionExpiry {
    /// The action is guaranteed through at least this many beats.
    pub guaranteed_until_beats: usize,
    /// Beats after which the fallback action activates.
    pub fallback_after_beats: usize,
    /// Beats after which the state is cleared entirely.
    pub clear_after_beats: usize,
    /// Maximum consecutive failed revalidations before clearing.
    pub max_failed_revalidations: usize,
}

/// Origin of the BPM value being carried by the continuity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityProvenance {
    /// Value was locked via an integer snap.
    IntegerSnap,
    /// Value came from a stable refined beat-interval estimate.
    StableRefinedEstimate,
    /// Value came from a guarded (moderately stable) refined estimate.
    GuardedRefinedEstimate,
    /// Value came from the core-window (edge-trimmed) estimate.
    CoreWindowEstimate,
    /// Value was carried from the prior analysis pass.
    PriorTempoCarry,
    /// No tempo value is present.
    NoTempo,
}

/// Complete arc decision produced by the continuity arc evaluator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityArcDecision {
    /// High-level arc recommendation.
    pub recommendation: TempoContinuityArcRecommendation,
    /// Concrete arc action for this pass.
    pub action: TempoContinuityArcAction,
    /// Current confidence level.
    pub severity: TempoContinuitySeverity,
    /// Action to apply if the primary action expires.
    pub fallback_action: TempoContinuityArcAction,
    /// Primary reason the arc is downgrading.
    pub downgrade_rationale: TempoContinuityArcDowngradeRationale,
    /// Per-cause pressure scores feeding the downgrade.
    pub downgrade_support: TempoContinuityArcDowngradeSupport,
    /// Whether downgrade pressure is rising, stable, or easing.
    pub downgrade_trend: TempoContinuityArcDowngradeTrend,
    /// Reason for the current downgrade trend direction.
    pub downgrade_trend_rationale: TempoContinuityArcDowngradeTrendRationale,
    /// Pressure levels at each projected downgrade stage.
    pub downgrade_trend_support: TempoContinuityArcDowngradeTrendSupport,
    /// Projected next inflection point in the downgrade arc.
    pub downgrade_inflection: TempoContinuityArcDowngradeInflection,
    /// Origin of the BPM value being carried.
    pub provenance: TempoContinuityProvenance,
    /// Beat-count thresholds for action expiry.
    pub expiry: TempoContinuityArcActionExpiry,
    /// Arc confidence at the time of this decision.
    pub confidence: Confidence,
}
