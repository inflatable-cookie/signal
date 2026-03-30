use crate::tempo_policy::*;
use crate::tempo_state_continuity_transition::{
    continuity_transition, TempoContinuityTransitionInputs,
};
use signal_analysis::Confidence;

mod tempo_state_continuity_helpers;
use tempo_state_continuity_helpers::*;
mod tempo_state_arc_decision;

pub fn tempo_state_recommendation_with_scope(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    stability_scope: TempoStabilityScopeSummary,
) -> TempoStateRecommendation {
    let base_confidence = (0.45 * interpretation.profile.stability_score.0
        + 0.25 * confidence.0
        + 0.15 * (1.0 - tempo_ambiguity.0)
        + 0.15 * interpretation.support.grid_stability.0)
        .clamp(0.0, 1.0);
    let localized_edge_horizons = || {
        if interpretation.support.boundary_pressure.0 >= 0.20 {
            (10, 6, 12, 18, 0.60)
        } else {
            (12, 8, 14, 20, 0.64)
        }
    };
    let localized_edge_scope = matches!(
        stability_scope.scope,
        TempoStabilityScope::StableWithLocalizedEdgeDamage
    );
    let core_stable_scope = matches!(stability_scope.scope, TempoStabilityScope::CoreStableOnly);
    let mid_track_unstable_scope =
        matches!(stability_scope.scope, TempoStabilityScope::MidTrackUnstable);
    let strong_integer_anchor = matches!(
        interpretation.recommendation,
        TempoRecommendation::SnapInteger
    ) && interpretation.support.integer_closeness.0 > 0.85
        && interpretation.support.core_consensus.0 > 0.8
        && interpretation.support.drift_stability.0 > 0.5
        && interpretation.support.grid_stability.0 > 0.35
        && interpretation.support.boundary_pressure.0 < 0.6;
    let ambiguity_guard = tempo_ambiguity.0 < 0.4 || strong_integer_anchor;

    match interpretation.recommendation {
        TempoRecommendation::SnapInteger
            if interpretation.trust != TempoTrustLevel::Tentative
                && (interpretation.profile.stability_score.0 >= 0.78 || strong_integer_anchor)
                && (interpretation.profile.snap_error_bpm >= 0.04
                    || interpretation.support.integer_closeness.0 > 0.9)
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.58
                } else {
                    0.48
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        TempoContinuityPlanInputs {
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            confidence: state_confidence,
                            trusted_beats: if core_stable_scope { 4 } else { 0 },
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                        },
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 4 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 0,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.92).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 8 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 1,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.64).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 12 } else { 0 },
                            action: TempoContinuityAction::Clear,
                            source: TempoContinuitySource::Cleared,
                            reason: TempoContinuityReason::InsufficientEvidence,
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 2,
                            confidence: Confidence::new(0.0),
                        }),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.76
            } else if strong_integer_anchor {
                0.80
            } else {
                0.82
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else {
                    TempoStateReason::StableIntegerTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::IntegerTempoSnap,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: if localized_edge_scope {
                            localized_trusted_beats
                        } else {
                            16
                        },
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::IntegerTempoSnap,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 1,
                        confidence: Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.72
                && interpretation.support.boundary_pressure.0 < 0.55
                && ambiguity_guard =>
        {
            if core_stable_scope || mid_track_unstable_scope {
                let state_confidence = Confidence::new(base_confidence.max(if core_stable_scope {
                    0.56
                } else {
                    0.46
                }));
                return TempoStateRecommendation {
                    action: if core_stable_scope {
                        TempoStateAction::Monitor
                    } else {
                        TempoStateAction::Defer
                    },
                    reason: if core_stable_scope {
                        TempoStateReason::CoreStableTempo
                    } else {
                        TempoStateReason::TempoDeferred
                    },
                    confidence: state_confidence,
                    continuity: continuity_plan(
                        TempoContinuityPlanInputs {
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            confidence: state_confidence,
                            trusted_beats: if core_stable_scope { 4 } else { 0 },
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                        },
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 4 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Lock
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::StableTempo
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 0,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.94).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 8 } else { 0 },
                            action: if core_stable_scope {
                                TempoContinuityAction::Reacquire
                            } else {
                                TempoContinuityAction::Clear
                            },
                            source: if core_stable_scope {
                                TempoContinuitySource::CurrentTempo
                            } else {
                                TempoContinuitySource::Cleared
                            },
                            reason: if core_stable_scope {
                                TempoContinuityReason::RevalidationDecay
                            } else {
                                TempoContinuityReason::InsufficientEvidence
                            },
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 1,
                            confidence: if core_stable_scope {
                                Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0))
                            } else {
                                Confidence::new(0.0)
                            },
                        }),
                        continuity_transition(TempoContinuityTransitionInputs {
                            after_beats: if core_stable_scope { 12 } else { 0 },
                            action: TempoContinuityAction::Clear,
                            source: TempoContinuitySource::Cleared,
                            reason: TempoContinuityReason::InsufficientEvidence,
                            boundary_pressure: interpretation.support.boundary_pressure,
                            tempo_ambiguity,
                            revalidate_after_beats: if core_stable_scope { 4 } else { 0 },
                            stage_index: 2,
                            confidence: Confidence::new(0.0),
                        }),
                    ),
                };
            }
            let state_confidence = Confidence::new(base_confidence.max(if localized_edge_scope {
                0.72
            } else {
                0.76
            }));
            let (
                localized_trusted_beats,
                localized_revalidate_after_beats,
                localized_downgrade_after_beats,
                localized_clear_after_beats,
                localized_decay_confidence_scale,
            ) = localized_edge_horizons();
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: if localized_edge_scope {
                    TempoStateReason::StableTempoWithEdgeDamage
                } else {
                    TempoStateReason::StableRefinedTempo
                },
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: if localized_edge_scope {
                            localized_trusted_beats
                        } else {
                            16
                        },
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_downgrade_after_beats
                        } else {
                            20
                        },
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 1,
                        confidence: Confidence::new(
                            (state_confidence.0
                                * if localized_edge_scope {
                                    localized_decay_confidence_scale
                                } else {
                                    0.72
                                })
                            .clamp(0.0, 1.0),
                        ),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: if localized_edge_scope {
                            localized_clear_after_beats
                        } else {
                            28
                        },
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: if localized_edge_scope {
                            localized_revalidate_after_beats
                        } else {
                            12
                        },
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseCoreWindow
            if interpretation.profile.stability_score.0 >= 0.55
                && interpretation.support.boundary_pressure.0 >= 0.45 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.58));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::CoreWindowFallback,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CoreWindow,
                        reason: TempoContinuityReason::CoreWindowCarry,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 8,
                        revalidate_after_beats: 4,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 4,
                        action: TempoContinuityAction::Retain,
                        source: TempoContinuitySource::CoreWindow,
                        reason: TempoContinuityReason::CoreWindowCarry,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 0,
                        confidence: state_confidence,
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 8,
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::PriorTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 1,
                        confidence: Confidence::new((state_confidence.0 * 0.68).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 12,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Guarded
                && interpretation.profile.stability_score.0 >= 0.58 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.56));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::StableRefinedTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 4,
                        revalidate_after_beats: 4,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 4,
                        action: TempoContinuityAction::Lock,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::StableTempo,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 0,
                        confidence: Confidence::new((state_confidence.0 * 0.96).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 8,
                        action: TempoContinuityAction::Reacquire,
                        source: TempoContinuitySource::CurrentTempo,
                        reason: TempoContinuityReason::RevalidationDecay,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 1,
                        confidence: Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0)),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 12,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 4,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
        _ => {
            let state_confidence = Confidence::new(
                (0.55 * (1.0 - interpretation.profile.stability_score.0)
                    + 0.45 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            );
            TempoStateRecommendation {
                action: TempoStateAction::Defer,
                reason: TempoStateReason::TempoDeferred,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityPlanInputs {
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        confidence: state_confidence,
                        trusted_beats: 0,
                        revalidate_after_beats: 0,
                    },
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 0,
                        confidence: Confidence::new(0.0),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 1,
                        confidence: Confidence::new(0.0),
                    }),
                    continuity_transition(TempoContinuityTransitionInputs {
                        after_beats: 0,
                        action: TempoContinuityAction::Clear,
                        source: TempoContinuitySource::Cleared,
                        reason: TempoContinuityReason::InsufficientEvidence,
                        boundary_pressure: interpretation.support.boundary_pressure,
                        tempo_ambiguity,
                        revalidate_after_beats: 0,
                        stage_index: 2,
                        confidence: Confidence::new(0.0),
                    }),
                ),
            }
        }
    }
}
