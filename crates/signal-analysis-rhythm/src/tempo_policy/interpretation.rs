use signal_analysis::Confidence;

/// How reliably the tempo estimate can be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrustLevel {
    /// Consistent, well-supported estimate.
    Stable,
    /// Estimate is plausible but shows signs of instability.
    Guarded,
    /// Estimate is uncertain; treat as provisional.
    Tentative,
}

/// Suggested consumer action for the tempo estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoRecommendation {
    /// Use the refined BPM from beat-interval analysis.
    UseRefined,
    /// Use the core-window median BPM rather than the refined estimate.
    UseCoreWindow,
    /// Round to the nearest integer BPM.
    SnapInteger,
    /// Do not use the tempo; evidence is insufficient.
    Defer,
}

/// Reason that shaped the tempo interpretation and recommendation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoInterpretationReason {
    /// The refined pulse was stable enough to use directly.
    StableRefinedPulse,
    /// The core-window estimate was more reliable than the refined estimate.
    StableCoreWindow,
    /// The refined BPM is close enough to an integer to snap.
    NearIntegerPulse,
    /// Tempo evidence is too unstable to produce a reliable estimate.
    UnstableTempo,
}

/// Evidence scores that drove the tempo interpretation decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationSupport {
    /// Agreement between the core-window and whole-track tempo estimates.
    pub core_consensus: Confidence,
    /// Stability of beat-interval drift across the track.
    pub drift_stability: Confidence,
    /// Regularity of beat-grid residuals.
    pub grid_stability: Confidence,
    /// Closeness of the refined BPM to a whole-number value.
    pub integer_closeness: Confidence,
    /// Pressure from track-boundary drift that may skew the estimate.
    pub boundary_pressure: Confidence,
}

/// Key BPM values and scores computed during tempo interpretation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationProfile {
    /// Beat-interval refined BPM.
    pub refined_bpm: f32,
    /// Core-window (edge-trimmed) median BPM.
    pub core_window_bpm: f32,
    /// Nearest integer to `refined_bpm`.
    pub nearest_integer_bpm: f32,
    /// Absolute error between `refined_bpm` and `nearest_integer_bpm`, in BPM.
    pub snap_error_bpm: f32,
    /// Combined stability score used for trust-level assignment.
    pub stability_score: Confidence,
    /// Gap between the predicted and actual beat position at the track boundary, in ms.
    pub boundary_edge_gap_ms: f32,
}

/// High-level tempo interpretation produced by the interpretation pipeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretation {
    /// Trust level for the tempo estimate.
    pub trust: TempoTrustLevel,
    /// What the consumer should do with the estimate.
    pub recommendation: TempoRecommendation,
    /// Reason that led to the recommendation.
    pub reason: TempoInterpretationReason,
    /// The recommended BPM (refined or core-window, per `recommendation`).
    pub recommended_bpm: f32,
    /// Integer-snapped BPM if `recommendation` is [`TempoRecommendation::SnapInteger`].
    pub snapped_bpm: Option<f32>,
    /// Evidence scores that drove the decision.
    pub support: TempoInterpretationSupport,
    /// Key BPM values and stability metrics.
    pub profile: TempoInterpretationProfile,
}
