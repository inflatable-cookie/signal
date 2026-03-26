use super::super::*;

pub(super) struct ArcSurfaceCases {
    pub(super) integer: TempoStateRecommendation,
    pub(super) core_window: TempoStateRecommendation,
    pub(super) guarded_refined: TempoStateRecommendation,
    pub(super) deferred: TempoStateRecommendation,
}

pub(super) fn arc_surface_cases() -> ArcSurfaceCases {
    ArcSurfaceCases {
        integer: tempo_state_recommendation(
            synthetic_tempo_interpretation(
                TempoRecommendation::SnapInteger,
                TempoTrustLevel::Stable,
                TempoInterpretationReason::NearIntegerPulse,
                120.0,
                Some(120.0),
                0.86,
                0.08,
                0.22,
                0.82,
            ),
            Confidence::new(0.9),
            Confidence::new(0.12),
        ),
        core_window: tempo_state_recommendation(
            synthetic_tempo_interpretation(
                TempoRecommendation::UseCoreWindow,
                TempoTrustLevel::Guarded,
                TempoInterpretationReason::StableCoreWindow,
                90.0,
                None,
                0.64,
                0.07,
                0.72,
                0.64,
            ),
            Confidence::new(0.72),
            Confidence::new(0.18),
        ),
        guarded_refined: tempo_state_recommendation(
            synthetic_tempo_interpretation(
                TempoRecommendation::UseRefined,
                TempoTrustLevel::Guarded,
                TempoInterpretationReason::StableRefinedPulse,
                117.8,
                None,
                0.61,
                0.09,
                0.32,
                0.62,
            ),
            Confidence::new(0.71),
            Confidence::new(0.21),
        ),
        deferred: tempo_state_recommendation(
            synthetic_tempo_interpretation(
                TempoRecommendation::Defer,
                TempoTrustLevel::Tentative,
                TempoInterpretationReason::UnstableTempo,
                89.9,
                None,
                0.38,
                0.03,
                0.8,
                0.3,
            ),
            Confidence::new(0.42),
            Confidence::new(0.55),
        ),
    }
}
