use signal_analysis::Confidence;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrustLevel {
    Stable,
    Guarded,
    Tentative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoRecommendation {
    UseRefined,
    UseCoreWindow,
    SnapInteger,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoInterpretationReason {
    StableRefinedPulse,
    StableCoreWindow,
    NearIntegerPulse,
    UnstableTempo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationSupport {
    pub core_consensus: Confidence,
    pub drift_stability: Confidence,
    pub grid_stability: Confidence,
    pub integer_closeness: Confidence,
    pub boundary_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationProfile {
    pub refined_bpm: f32,
    pub core_window_bpm: f32,
    pub nearest_integer_bpm: f32,
    pub snap_error_bpm: f32,
    pub stability_score: Confidence,
    pub boundary_edge_gap_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretation {
    pub trust: TempoTrustLevel,
    pub recommendation: TempoRecommendation,
    pub reason: TempoInterpretationReason,
    pub recommended_bpm: f32,
    pub snapped_bpm: Option<f32>,
    pub support: TempoInterpretationSupport,
    pub profile: TempoInterpretationProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStateAction {
    Lock,
    Monitor,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStateReason {
    StableIntegerTempo,
    StableTempoWithEdgeDamage,
    StableRefinedTempo,
    CoreStableTempo,
    CoreWindowFallback,
    TempoDeferred,
}
