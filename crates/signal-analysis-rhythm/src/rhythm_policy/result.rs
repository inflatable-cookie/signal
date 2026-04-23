use signal_analysis::Confidence;

use super::{
    tempo_helpers::{
        push_tempo_segment_summary, stable_span_tempo_segment_summary,
        whole_track_tempo_segment_summary,
    },
    MeterEstimate, MeterStateRecommendation, RhythmStructureAmbiguitySummary,
};
use crate::{
    TempoCandidate, TempoConsumptionDecision, TempoConsumptionSelection, TempoConsumptionSource,
    TempoContinuityArcAction, TempoContinuitySource, TempoDiagnostics, TempoInterpretation,
    TempoSegmentKind, TempoStabilityScope, TempoStabilityScopeSummary, TempoStateAction,
    TempoStateRecommendation, TempoStructureSummary,
};

/// Full output of a single [`BeatTracker`](crate::BeatTracker) analysis pass.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatAnalysisResult {
    /// Primary tempo estimate in beats per minute.
    pub bpm: f32,
    /// Overall confidence in the tempo and beat grid.
    pub confidence: Confidence,
    /// Beat onset times in seconds, in ascending order.
    pub beat_positions_seconds: Vec<f32>,
    /// Normalised onset-strength envelope used for tempo and beat estimation.
    pub onset_envelope: Vec<f32>,
    /// Ranked list of competing tempo hypotheses.
    pub tempo_candidates: Vec<TempoCandidate>,
    /// Detailed per-beat and windowed tempo stability diagnostics.
    pub tempo_diagnostics: TempoDiagnostics,
    /// High-level tempo trust level and snap recommendation.
    pub tempo_interpretation: TempoInterpretation,
    /// Continuity-aware tempo state and handoff recommendation.
    pub tempo_state: TempoStateRecommendation,
    /// Confidence that a strong alternative tempo exists (higher = more ambiguous).
    pub tempo_ambiguity: Confidence,
    /// Continuity-aware meter state and handoff recommendation.
    pub meter_state: MeterStateRecommendation,
    /// Meter estimate (beats per bar and downbeat positions), if detected.
    pub meter: Option<MeterEstimate>,
    /// Summary of any competing or ambiguous meter/downbeat hypotheses.
    pub structure_ambiguity: RhythmStructureAmbiguitySummary,
}

impl BeatAnalysisResult {
    /// Build a compact tempo structure summary suitable for display or logging.
    pub fn tempo_structure_summary(&self) -> TempoStructureSummary {
        let consumption = self.tempo_consumption(None);
        let mut segments = Vec::new();
        push_tempo_segment_summary(&mut segments, whole_track_tempo_segment_summary(self));
        push_tempo_segment_summary(
            &mut segments,
            stable_span_tempo_segment_summary(
                TempoSegmentKind::EdgeTrimmedStable,
                &self.tempo_diagnostics.windowed_tempi,
                self.tempo_diagnostics.edge_trimmed_stable_span,
                self.tempo_diagnostics.windowed_median_bpm,
                self.tempo_diagnostics.windowed_drift_span_bpm,
                self.tempo_diagnostics.windowed_mean_abs_deviation_bpm,
            ),
        );
        push_tempo_segment_summary(
            &mut segments,
            stable_span_tempo_segment_summary(
                TempoSegmentKind::StableCore,
                &self.tempo_diagnostics.windowed_tempi,
                self.tempo_diagnostics.stable_core_span,
                self.tempo_diagnostics.core_windowed_median_bpm,
                self.tempo_diagnostics.core_windowed_drift_span_bpm,
                self.tempo_diagnostics.core_windowed_mean_abs_deviation_bpm,
            ),
        );

        TempoStructureSummary {
            trust: self.tempo_interpretation.trust,
            recommendation: self.tempo_interpretation.recommendation,
            selected_bpm: consumption.current.bpm,
            snapped_bpm: self.tempo_interpretation.snapped_bpm,
            core_window_bpm: self.tempo_interpretation.profile.core_window_bpm,
            stability_scope: self.tempo_diagnostics.stability_scope,
            ambiguity: self.tempo_ambiguity,
            trend: self.tempo_diagnostics.trend,
            segments,
            continuity: crate::TempoContinuitySummary {
                action: self.tempo_state.action,
                reason: self.tempo_state.reason,
                confidence: self.tempo_state.confidence,
                trust: self.tempo_interpretation.trust,
                recommendation: self.tempo_interpretation.recommendation,
                continuity_action: self.tempo_state.continuity.action,
                source: self.tempo_state.continuity.source,
                severity: self.tempo_state.continuity.severity,
                history: self.tempo_state.continuity.history,
                arc: self.tempo_state.continuity.arc,
                arc_rationale: self.tempo_state.continuity.arc_rationale,
                trigger: self.tempo_state.continuity.trigger,
                provenance: self.tempo_state.continuity.provenance,
                ambiguity: self.tempo_ambiguity,
                refresh_strength: self.tempo_state.continuity.refresh_strength,
                trusted_beats: self.tempo_state.continuity.trusted_beats,
                revalidate_after_beats: self.tempo_state.continuity.revalidate_after_beats,
                fallback_after_beats: consumption.fallback_after_beats,
                clear_after_beats: self.tempo_state.continuity.expiry.clear_after_beats,
                current: consumption.current,
                fallback: consumption.fallback,
            },
        }
    }

    /// Decide which BPM value to expose and when to fall back, given an optional
    /// BPM from a previous analysis pass.
    pub fn tempo_consumption(&self, prior_bpm: Option<f32>) -> TempoConsumptionDecision {
        fn should_clear_core_stable_without_prior_early(
            confidence: Confidence,
            interpretation: TempoInterpretation,
            stability_scope: TempoStabilityScopeSummary,
        ) -> bool {
            interpretation.support.boundary_pressure.0 < 0.18
                && confidence.0 >= 0.76
                && stability_scope.support.edge_trimmed_coverage.0 >= 0.85
                && stability_scope.support.interior_stability.0 >= 0.90
        }

        fn current_tempo_selection(
            interpretation: TempoInterpretation,
            source: TempoContinuitySource,
            prior_bpm: Option<f32>,
        ) -> TempoConsumptionSelection {
            match source {
                TempoContinuitySource::CurrentTempo => {
                    if let Some(snapped_bpm) = interpretation.snapped_bpm {
                        TempoConsumptionSelection {
                            bpm: Some(snapped_bpm),
                            source: TempoConsumptionSource::SnappedCurrentTempo,
                        }
                    } else {
                        TempoConsumptionSelection {
                            bpm: Some(interpretation.recommended_bpm),
                            source: TempoConsumptionSource::RefinedCurrentTempo,
                        }
                    }
                }
                TempoContinuitySource::CoreWindow => TempoConsumptionSelection {
                    bpm: Some(interpretation.profile.core_window_bpm),
                    source: TempoConsumptionSource::CoreWindowTempo,
                },
                TempoContinuitySource::PriorTempo => TempoConsumptionSelection {
                    bpm: prior_bpm,
                    source: if prior_bpm.is_some() {
                        TempoConsumptionSource::PriorTempo
                    } else {
                        TempoConsumptionSource::NoTempo
                    },
                },
                TempoContinuitySource::Cleared => TempoConsumptionSelection {
                    bpm: None,
                    source: TempoConsumptionSource::NoTempo,
                },
            }
        }

        fn fallback_tempo_selection(
            interpretation: TempoInterpretation,
            action: TempoContinuityArcAction,
            prior_bpm: Option<f32>,
        ) -> TempoConsumptionSelection {
            match action {
                TempoContinuityArcAction::LockCurrentTempo
                | TempoContinuityArcAction::ReacquireCurrentTempo => {
                    if let Some(snapped_bpm) = interpretation.snapped_bpm {
                        TempoConsumptionSelection {
                            bpm: Some(snapped_bpm),
                            source: TempoConsumptionSource::SnappedCurrentTempo,
                        }
                    } else {
                        TempoConsumptionSelection {
                            bpm: Some(interpretation.recommended_bpm),
                            source: TempoConsumptionSource::RefinedCurrentTempo,
                        }
                    }
                }
                TempoContinuityArcAction::PreferCoreWindowTempo => TempoConsumptionSelection {
                    bpm: Some(interpretation.profile.core_window_bpm),
                    source: TempoConsumptionSource::CoreWindowTempo,
                },
                TempoContinuityArcAction::PreservePriorTempo => TempoConsumptionSelection {
                    bpm: prior_bpm,
                    source: if prior_bpm.is_some() {
                        TempoConsumptionSource::PriorTempo
                    } else {
                        TempoConsumptionSource::NoTempo
                    },
                },
                TempoContinuityArcAction::ClearTempo => TempoConsumptionSelection {
                    bpm: None,
                    source: TempoConsumptionSource::NoTempo,
                },
            }
        }

        fn prior_tempo_selection(prior_bpm: Option<f32>) -> TempoConsumptionSelection {
            TempoConsumptionSelection {
                bpm: prior_bpm,
                source: if prior_bpm.is_some() {
                    TempoConsumptionSource::PriorTempo
                } else {
                    TempoConsumptionSource::NoTempo
                },
            }
        }

        let stability_scope = self.tempo_diagnostics.stability_scope;
        let mut fallback = fallback_tempo_selection(
            self.tempo_interpretation,
            self.tempo_state.continuity.arc_decision.fallback_action,
            prior_bpm,
        );
        let mut fallback_after_beats = self
            .tempo_state
            .continuity
            .arc_decision
            .expiry
            .fallback_after_beats;

        if matches!(self.tempo_state.action, TempoStateAction::Monitor)
            && matches!(stability_scope.scope, TempoStabilityScope::CoreStableOnly)
        {
            if prior_bpm.is_some() {
                fallback = prior_tempo_selection(prior_bpm);
                fallback_after_beats = self.tempo_state.continuity.expiry.downgrade_after_beats;
            } else if should_clear_core_stable_without_prior_early(
                self.tempo_state.confidence,
                self.tempo_interpretation,
                stability_scope,
            ) {
                fallback = TempoConsumptionSelection {
                    bpm: None,
                    source: TempoConsumptionSource::NoTempo,
                };
                fallback_after_beats = self.tempo_state.continuity.expiry.downgrade_after_beats;
            } else {
                fallback = TempoConsumptionSelection {
                    bpm: None,
                    source: TempoConsumptionSource::NoTempo,
                };
                fallback_after_beats = self.tempo_state.continuity.expiry.clear_after_beats;
            }
        }

        TempoConsumptionDecision {
            action: self.tempo_state.action,
            reason: self.tempo_state.reason,
            continuity_action: self.tempo_state.continuity.action,
            confidence: self.tempo_state.confidence,
            stability_scope,
            current: current_tempo_selection(
                self.tempo_interpretation,
                self.tempo_state.continuity.source,
                prior_bpm,
            ),
            fallback,
            fallback_after_beats,
        }
    }
}
