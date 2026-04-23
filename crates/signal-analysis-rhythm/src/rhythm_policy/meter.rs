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

/// Continuity-layer action for the meter value across analysis passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateAction {
    /// Commit the current meter and hold it across passes.
    Lock,
    /// Retain the previous meter while evidence is monitored.
    Hold,
    /// Track the meter but do not commit it yet.
    Watch,
    /// Discard any held meter value.
    Clear,
}

/// Reason that drove the meter state action decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateReason {
    /// Evidence is strong and consistent — meter is locked.
    StableMeter,
    /// Evidence was disrupted but is rebuilding.
    RecoveringMeter,
    /// Evidence is present but not yet reliable enough to lock.
    TentativeMeter,
    /// A previously locked meter lost its supporting evidence.
    DestabilizedHold,
    /// A recovery window is emerging from disruption.
    RecoveryEmerging,
    /// No usable meter evidence remains.
    MeterCleared,
}

/// Action applied to the meter continuity state on the next pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityAction {
    /// Hold the current meter value with full confidence.
    Lock,
    /// Keep the current value while evidence is re-evaluated.
    Retain,
    /// Attempt to regain a recently lost meter.
    Reacquire,
    /// Discard the continuity state entirely.
    Clear,
}

/// Which meter value is being carried forward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySource {
    /// The meter just estimated in this pass.
    CurrentMeter,
    /// The meter from the preceding pass.
    PriorMeter,
    /// A meter recovered from a stable sub-segment.
    RecoveryWindow,
    /// No meter value is being carried.
    Cleared,
}

/// Confidence level of the meter continuity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySeverity {
    /// High confidence; the meter is well supported.
    Confirmed,
    /// Moderate confidence; some evidence of instability.
    Guarded,
    /// Low confidence; the meter may be about to be lost.
    Fragile,
    /// No active meter continuity state.
    Cleared,
}

/// Reason for the current meter continuity state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityReason {
    /// Consistent, well-supported meter evidence.
    StableEvidence,
    /// Evidence is present but not fully reliable.
    TentativeEvidence,
    /// Previous state carried forward due to limited new evidence.
    PriorStateCarry,
    /// Recovery window provided the supporting evidence.
    RecoveryWindowSupport,
    /// The downbeat phase shifted relative to the prior estimate.
    PhaseDisplacement,
    /// Confidence is decaying because revalidation has not succeeded.
    RevalidationDecay,
    /// Not enough evidence to determine a reason.
    InsufficientEvidence,
}

/// Event that triggered the current meter continuity transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeterContinuityTrigger {
    /// A revalidation window confirmed the current meter.
    StableRevalidation,
    /// The prior meter was tentatively carried without new evidence.
    TentativeCarry,
    /// The downbeat phase was recovered after a displacement.
    PhaseRecovery,
    /// The prior state drifted enough to require a transition.
    PriorStateDrift,
    /// The recovery window itself drifted, reducing confidence.
    RecoveryWindowDrift,
    /// Supporting evidence was lost.
    EvidenceLoss,
}

/// Duration of the current unresolved continuity gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityUnresolvedSpan {
    /// Number of beats without a successful revalidation.
    pub beats: usize,
    /// Number of bars without a successful revalidation.
    pub bars: usize,
    /// Count of consecutive failed revalidation attempts.
    pub failed_revalidations: usize,
}

/// Root cause contributing to a meter continuity transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityCause {
    /// Strong, stable meter evidence is driving the transition.
    StableMeterEvidence,
    /// Tempo ambiguity is undermining meter confidence.
    TempoAmbiguity,
    /// The downbeat phase shifted, disrupting continuity.
    PhaseDisplacement,
    /// Too few onset events at the expected bar boundaries.
    SparseMeterSupport,
    /// Bar lengths are inconsistent with a regular meter.
    IrregularBarStructure,
    /// Continuity is being carried from the prior state.
    PriorContinuityCarry,
    /// The recovery window shows its own instability.
    RecoveryWindowInstability,
    /// Evidence was lost entirely.
    EvidenceLoss,
}

/// Ordered stack of causes driving a meter continuity transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityCauseStack {
    /// Dominant cause.
    pub primary: MeterContinuityCause,
    /// Up to two contributing secondary causes.
    pub secondary: [Option<MeterContinuityCause>; 2],
    /// Total number of active causes (including primary).
    pub count: usize,
}

/// Direction of the meter continuity state over recent passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityHistory {
    /// Confidence has been improving across recent passes.
    Reinforcing,
    /// Confidence is stable with no clear trend.
    Preserving,
    /// Confidence has been declining across recent passes.
    Degrading,
}

/// Trajectory of the meter continuity arc across the current window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArc {
    /// Evidence is improving and continuity is being rebuilt.
    Recovering,
    /// Evidence is not improving; continuity is stagnant.
    Stalling,
    /// Evidence is deteriorating; continuity is about to be lost.
    Collapsing,
}

/// Dominant factor that shaped the meter continuity arc decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArcRationale {
    /// High refresh strength is driving recovery.
    RefreshStrength,
    /// A stable carry from the prior state is sustaining the arc.
    StableCarry,
    /// Accumulated unresolved drift is pushing the arc toward collapse.
    UnresolvedDrift,
    /// Structural bar instability is the primary driver.
    StructuralInstability,
    /// Evidence loss is the primary driver.
    EvidenceLoss,
}

/// Pressure components feeding into the meter continuity arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityArcSupport {
    /// How strongly the current evidence is refreshing continuity.
    pub refresh_strength: Confidence,
    /// Pressure from accumulated unresolved drift.
    pub drift_pressure: Confidence,
    /// Pressure from irregular bar structure.
    pub structural_pressure: Confidence,
}

/// A single planned meter continuity transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityTransition {
    /// Number of beats after which this transition activates.
    pub after_beats: usize,
    /// Continuity action to apply at transition time.
    pub action: MeterContinuityAction,
    /// Meter value source at transition time.
    pub source: MeterContinuitySource,
    /// Confidence level at transition time.
    pub severity: MeterContinuitySeverity,
    /// Historical trend leading into this transition.
    pub history: MeterContinuityHistory,
    /// Reason for the transition.
    pub reason: MeterContinuityReason,
    /// Confidence score at transition time.
    pub confidence: Confidence,
    /// Event that triggered the transition.
    pub trigger: MeterContinuityTrigger,
    /// Duration of the unresolved gap that led to this transition.
    pub unresolved: MeterContinuityUnresolvedSpan,
    /// Ordered cause stack driving the transition.
    pub causes: MeterContinuityCauseStack,
}

/// Planned lifecycle transitions for a meter continuity state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityLifecycle {
    /// The refresh transition that reinforces continuity on revalidation.
    pub refresh: MeterContinuityTransition,
    /// The two decay transitions (downgrade then clear) if revalidation fails.
    pub decay: [MeterContinuityTransition; 2],
}

/// Full continuity plan for one meter dimension (bar length or downbeat phase).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityPlan {
    /// Action to apply immediately.
    pub action: MeterContinuityAction,
    /// Source of the meter value being carried.
    pub source: MeterContinuitySource,
    /// Current confidence level.
    pub severity: MeterContinuitySeverity,
    /// Recent history trend.
    pub history: MeterContinuityHistory,
    /// Continuity arc trajectory.
    pub arc: MeterContinuityArc,
    /// Dominant factor shaping the arc.
    pub arc_rationale: MeterContinuityArcRationale,
    /// Pressure components feeding the arc.
    pub arc_support: MeterContinuityArcSupport,
    /// Reason for the current state.
    pub reason: MeterContinuityReason,
    /// Confidence score.
    pub confidence: Confidence,
    /// Event that triggered this plan.
    pub trigger: MeterContinuityTrigger,
    /// Duration of the current unresolved gap.
    pub unresolved: MeterContinuityUnresolvedSpan,
    /// Ordered cause stack.
    pub causes: MeterContinuityCauseStack,
    /// Number of beats this plan is considered fully trusted.
    pub trusted_beats: usize,
    /// Number of beats before a revalidation check is expected.
    pub revalidate_after_beats: usize,
    /// Planned future transitions.
    pub lifecycle: MeterContinuityLifecycle,
}

/// Continuity plans for both meter dimensions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityRecommendation {
    /// Continuity plan for the bar-length (beats per bar) dimension.
    pub bar_length: MeterContinuityPlan,
    /// Continuity plan for the downbeat-phase dimension.
    pub downbeat_phase: MeterContinuityPlan,
}

/// Top-level meter state recommendation returned by the analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterStateRecommendation {
    /// High-level action for the meter value.
    pub action: MeterStateAction,
    /// Reason that drove the action decision.
    pub reason: MeterStateReason,
    /// Overall meter confidence.
    pub confidence: Confidence,
    /// Detailed continuity plans for bar length and downbeat phase.
    pub continuity: MeterContinuityRecommendation,
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
