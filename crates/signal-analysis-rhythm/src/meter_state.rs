#[derive(Clone, Copy, Debug)]
pub(crate) struct MeterSuppressionProfile {
    pub best_confidence: Confidence,
    pub best_support: f32,
    pub best_regularity: f32,
    pub trailing_confidence: Confidence,
    pub trailing_recent_stability: f32,
}

pub(crate) struct MeterDecision {
    pub estimate: Option<MeterEstimate>,
    pub suppression_profile: MeterSuppressionProfile,
    pub ambiguity: RhythmStructureAmbiguitySummary,
}

use crate::beat_tempo_core::beat_frames_to_seconds;
use crate::rhythm_policy::*;
use crate::{downbeat_frames_for_hypothesis, neighborhood_peak};
use signal_analysis::Confidence;
use signal_primitives::SampleRate;

pub(crate) fn meter_state_recommendation(
    estimate: Option<&MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
    rhythm_confidence: Confidence,
    tempo_ambiguity: Confidence,
    bpm: f32,
    beat_positions_seconds: &[f32],
) -> MeterStateRecommendation {
    fn push_cause(
        slots: &mut [Option<MeterContinuityCause>; 3],
        count: &mut usize,
        cause: MeterContinuityCause,
    ) {
        if slots.iter().flatten().any(|existing| *existing == cause) {
            return;
        }
        if *count < slots.len() {
            slots[*count] = Some(cause);
            *count += 1;
        }
    }

    #[derive(Clone, Copy)]
    struct MeterContinuityCauseInputs {
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        trigger: MeterContinuityTrigger,
        suppression_profile: MeterSuppressionProfile,
        tempo_ambiguity: Confidence,
        phase_displaced: bool,
        stage_index: usize,
    }

    #[derive(Clone, Copy)]
    struct MeterContinuityStageContext {
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        stage_index: usize,
    }

    #[derive(Clone, Copy)]
    struct MeterContinuityArcInputs {
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    }

    #[derive(Clone, Copy)]
    struct MeterContinuityPlanInputs {
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        trusted_beats: usize,
        revalidate_after_beats: usize,
    }

    #[derive(Clone, Copy)]
    struct MeterContinuityInputs<'a> {
        estimate: Option<&'a MeterEstimate>,
        suppression_profile: MeterSuppressionProfile,
        confidence: Confidence,
        tempo_ambiguity: Confidence,
        bpm: f32,
        beat_positions_seconds: &'a [f32],
    }

    fn cause_stack(inputs: MeterContinuityCauseInputs) -> MeterContinuityCauseStack {
        let MeterContinuityCauseInputs {
            action,
            source,
            reason,
            trigger,
            suppression_profile,
            tempo_ambiguity,
            phase_displaced,
            stage_index,
        } = inputs;
        let mut causes = [None; 3];
        let mut count = 0usize;

        match reason {
            MeterContinuityReason::StableEvidence => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityReason::PriorStateCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityReason::RecoveryWindowSupport => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityReason::PhaseDisplacement => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityReason::InsufficientEvidence => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
            MeterContinuityReason::TentativeEvidence | MeterContinuityReason::RevalidationDecay => {
            }
        }

        match trigger {
            MeterContinuityTrigger::StableRevalidation => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityTrigger::TentativeCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::SparseMeterSupport,
                );
            }
            MeterContinuityTrigger::PhaseRecovery => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityTrigger::PriorStateDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityTrigger::RecoveryWindowDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityTrigger::EvidenceLoss => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
        }

        if phase_displaced {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PhaseDisplacement,
            );
        }

        if tempo_ambiguity.0 >= 0.28 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::TempoAmbiguity,
            );
        }

        if suppression_profile.best_support < 0.58 || suppression_profile.best_confidence.0 < 0.24 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::SparseMeterSupport,
            );
        }

        if suppression_profile.best_regularity < 0.32
            || (stage_index > 0 && suppression_profile.trailing_recent_stability < 0.30)
        {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::IrregularBarStructure,
            );
        }

        if matches!(source, MeterContinuitySource::Cleared)
            || matches!(action, MeterContinuityAction::Clear)
        {
            push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
        }

        let primary = causes[0].unwrap_or(match action {
            MeterContinuityAction::Lock => MeterContinuityCause::StableMeterEvidence,
            MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                MeterContinuityCause::SparseMeterSupport
            }
            MeterContinuityAction::Clear => MeterContinuityCause::EvidenceLoss,
        });

        MeterContinuityCauseStack {
            primary,
            secondary: [causes[1], causes[2]],
            count: count.max(1),
        }
    }

    fn has_cause(stack: MeterContinuityCauseStack, cause: MeterContinuityCause) -> bool {
        stack.primary == cause
            || stack
                .secondary
                .into_iter()
                .flatten()
                .any(|entry| entry == cause)
    }

    fn continuity_history(context: MeterContinuityStageContext) -> MeterContinuityHistory {
        let MeterContinuityStageContext {
            action,
            source,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            stage_index,
        } = context;
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let has_phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);

        match action {
            MeterContinuityAction::Clear => MeterContinuityHistory::Degrading,
            MeterContinuityAction::Lock
                if matches!(source, MeterContinuitySource::CurrentMeter)
                    && matches!(reason, MeterContinuityReason::StableEvidence)
                    && matches!(trigger, MeterContinuityTrigger::StableRevalidation)
                    && confidence.0 >= 0.28
                    && unresolved.failed_revalidations == 0
                    && !has_evidence_loss =>
            {
                MeterContinuityHistory::Reinforcing
            }
            MeterContinuityAction::Lock => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Retain
                if stage_index > 0
                    || has_evidence_loss
                    || (matches!(source, MeterContinuitySource::PriorMeter)
                        && unresolved.failed_revalidations >= 2)
                    || (matches!(source, MeterContinuitySource::RecoveryWindow)
                        && has_irregularity
                        && confidence.0 < 0.30) =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Retain => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Reacquire
                if matches!(reason, MeterContinuityReason::PhaseDisplacement)
                    || stage_index > 0
                    || unresolved.failed_revalidations > 0
                    || has_evidence_loss
                    || has_phase_displacement =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Reacquire => MeterContinuityHistory::Preserving,
        }
    }

    fn continuity_arc_support(
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityArcSupport {
        let refresh_bonus = match refresh.history {
            MeterContinuityHistory::Reinforcing => 0.28,
            MeterContinuityHistory::Preserving => 0.12,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let current_bonus = match current {
            MeterContinuityHistory::Reinforcing => 0.22,
            MeterContinuityHistory::Preserving => 0.08,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let decay_penalty = match first_decay.history {
            MeterContinuityHistory::Degrading => 0.08,
            _ => 0.0,
        } + match final_decay.history {
            MeterContinuityHistory::Degrading => 0.12,
            _ => 0.0,
        };
        let refresh_strength = Confidence::new(
            (refresh.confidence.0 + refresh_bonus + current_bonus - decay_penalty).clamp(0.0, 1.0),
        );

        let drift_pressure = Confidence::new(
            ((unresolved.failed_revalidations as f32 * 0.18)
                + (unresolved.bars as f32 * 0.08)
                + match current {
                    MeterContinuityHistory::Degrading => 0.18,
                    MeterContinuityHistory::Preserving => 0.08,
                    MeterContinuityHistory::Reinforcing => 0.0,
                }
                + match first_decay.history {
                    MeterContinuityHistory::Degrading => 0.16,
                    _ => 0.0,
                }
                + match final_decay.history {
                    MeterContinuityHistory::Degrading => 0.20,
                    _ => 0.0,
                })
            .clamp(0.0, 1.0),
        );

        let evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);
        let tempo_ambiguity = has_cause(causes, MeterContinuityCause::TempoAmbiguity);
        let structural_pressure = Confidence::new(
            ((if evidence_loss { 0.42f32 } else { 0.0f32 })
                + (if irregularity { 0.28f32 } else { 0.0f32 })
                + (if phase_displacement { 0.18f32 } else { 0.0f32 })
                + (if tempo_ambiguity { 0.12f32 } else { 0.0f32 }))
            .clamp(0.0, 1.0),
        );

        MeterContinuityArcSupport {
            refresh_strength,
            drift_pressure,
            structural_pressure,
        }
    }

    fn continuity_arc_assessment(
        inputs: MeterContinuityArcInputs,
    ) -> (
        MeterContinuityArc,
        MeterContinuityArcRationale,
        MeterContinuityArcSupport,
    ) {
        let MeterContinuityArcInputs {
            source,
            reason,
            confidence,
            unresolved,
            causes,
            current,
            refresh,
            first_decay,
            final_decay,
        } = inputs;
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let persistent_decay = matches!(first_decay.history, MeterContinuityHistory::Degrading)
            && matches!(final_decay.history, MeterContinuityHistory::Degrading);
        let support = continuity_arc_support(
            unresolved,
            causes,
            current,
            refresh,
            first_decay,
            final_decay,
        );

        if matches!(current, MeterContinuityHistory::Degrading)
            && (persistent_decay || has_evidence_loss)
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        if matches!(refresh.history, MeterContinuityHistory::Reinforcing) && !has_evidence_loss {
            if matches!(current, MeterContinuityHistory::Reinforcing) {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }

            if matches!(current, MeterContinuityHistory::Preserving)
                && matches!(source, MeterContinuitySource::RecoveryWindow)
                && matches!(reason, MeterContinuityReason::RecoveryWindowSupport)
                && confidence.0 >= 0.80
                && unresolved.failed_revalidations <= 2
                && !has_irregularity
            {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }
        }

        if has_evidence_loss
            || (persistent_decay && confidence.0 < 0.24)
            || (matches!(current, MeterContinuityHistory::Degrading)
                && !matches!(refresh.history, MeterContinuityHistory::Reinforcing))
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else if has_irregularity {
                    MeterContinuityArcRationale::StructuralInstability
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        (
            MeterContinuityArc::Stalling,
            if has_irregularity {
                MeterContinuityArcRationale::StructuralInstability
            } else if unresolved.failed_revalidations >= 2 {
                MeterContinuityArcRationale::UnresolvedDrift
            } else {
                MeterContinuityArcRationale::StableCarry
            },
            support,
        )
    }

    fn continuity_trigger(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
    ) -> MeterContinuityTrigger {
        match reason {
            MeterContinuityReason::StableEvidence => MeterContinuityTrigger::StableRevalidation,
            MeterContinuityReason::TentativeEvidence => MeterContinuityTrigger::TentativeCarry,
            MeterContinuityReason::PriorStateCarry => MeterContinuityTrigger::PriorStateDrift,
            MeterContinuityReason::RecoveryWindowSupport => {
                MeterContinuityTrigger::RecoveryWindowDrift
            }
            MeterContinuityReason::PhaseDisplacement => MeterContinuityTrigger::PhaseRecovery,
            MeterContinuityReason::RevalidationDecay => match source {
                MeterContinuitySource::PriorMeter => MeterContinuityTrigger::PriorStateDrift,
                MeterContinuitySource::RecoveryWindow => {
                    MeterContinuityTrigger::RecoveryWindowDrift
                }
                MeterContinuitySource::CurrentMeter => match action {
                    MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                        MeterContinuityTrigger::TentativeCarry
                    }
                    MeterContinuityAction::Lock => MeterContinuityTrigger::StableRevalidation,
                    MeterContinuityAction::Clear => MeterContinuityTrigger::EvidenceLoss,
                },
                MeterContinuitySource::Cleared => MeterContinuityTrigger::EvidenceLoss,
            },
            MeterContinuityReason::InsufficientEvidence => MeterContinuityTrigger::EvidenceLoss,
        }
    }

    fn unresolved_span(
        trigger: MeterContinuityTrigger,
        beat_span: usize,
        revalidate_after_beats: usize,
        beats_per_bar: usize,
        phase_displacement_beats: usize,
        stage_index: usize,
    ) -> MeterContinuityUnresolvedSpan {
        let beats = match trigger {
            MeterContinuityTrigger::StableRevalidation => 0,
            MeterContinuityTrigger::PhaseRecovery => beat_span
                .max(revalidate_after_beats)
                .max(phase_displacement_beats.max(1)),
            MeterContinuityTrigger::TentativeCarry
            | MeterContinuityTrigger::PriorStateDrift
            | MeterContinuityTrigger::RecoveryWindowDrift => {
                beat_span.max(revalidate_after_beats.max(1))
            }
            MeterContinuityTrigger::EvidenceLoss => beat_span,
        };
        let bars = if beats == 0 {
            0
        } else {
            beats.div_ceil(beats_per_bar.max(1))
        };
        let failed_revalidations = if beats == 0 || revalidate_after_beats == 0 {
            0
        } else {
            beats.div_ceil(revalidate_after_beats).max(stage_index)
        };
        MeterContinuityUnresolvedSpan {
            beats,
            bars,
            failed_revalidations,
        }
    }

    fn continuity_reason(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        phase_displaced: bool,
        is_decay: bool,
    ) -> MeterContinuityReason {
        if matches!(action, MeterContinuityAction::Clear)
            || matches!(source, MeterContinuitySource::Cleared)
        {
            return MeterContinuityReason::InsufficientEvidence;
        }

        if phase_displaced && matches!(action, MeterContinuityAction::Reacquire) {
            return MeterContinuityReason::PhaseDisplacement;
        }

        if is_decay {
            return MeterContinuityReason::RevalidationDecay;
        }

        match source {
            MeterContinuitySource::CurrentMeter => match action {
                MeterContinuityAction::Lock => MeterContinuityReason::StableEvidence,
                MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                    MeterContinuityReason::TentativeEvidence
                }
                MeterContinuityAction::Clear => MeterContinuityReason::InsufficientEvidence,
            },
            MeterContinuitySource::PriorMeter => MeterContinuityReason::PriorStateCarry,
            MeterContinuitySource::RecoveryWindow => MeterContinuityReason::RecoveryWindowSupport,
            MeterContinuitySource::Cleared => MeterContinuityReason::InsufficientEvidence,
        }
    }

    fn continuity_severity(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
    ) -> MeterContinuitySeverity {
        match action {
            MeterContinuityAction::Lock => MeterContinuitySeverity::Confirmed,
            MeterContinuityAction::Retain => match source {
                MeterContinuitySource::CurrentMeter | MeterContinuitySource::RecoveryWindow => {
                    MeterContinuitySeverity::Guarded
                }
                MeterContinuitySource::PriorMeter => MeterContinuitySeverity::Fragile,
                MeterContinuitySource::Cleared => MeterContinuitySeverity::Cleared,
            },
            MeterContinuityAction::Reacquire => MeterContinuitySeverity::Fragile,
            MeterContinuityAction::Clear => MeterContinuitySeverity::Cleared,
        }
    }

    fn continuity_confidence(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        state_confidence: Confidence,
        beat_span: usize,
        stage_index: usize,
    ) -> Confidence {
        let action_scale = match action {
            MeterContinuityAction::Lock => 1.0,
            MeterContinuityAction::Retain => 0.72,
            MeterContinuityAction::Reacquire => 0.45,
            MeterContinuityAction::Clear => 0.0,
        };
        let source_bias = match source {
            MeterContinuitySource::CurrentMeter => 0.12,
            MeterContinuitySource::RecoveryWindow => 0.06,
            MeterContinuitySource::PriorMeter => -0.02,
            MeterContinuitySource::Cleared => -0.30,
        };
        let span_bias = (beat_span as f32 / 24.0).clamp(0.0, 0.25);
        let decay_penalty = stage_index as f32 * 0.12;
        Confidence::new(
            (state_confidence.0 * action_scale + source_bias + span_bias - decay_penalty)
                .clamp(0.0, 1.0),
        )
    }

    fn transition(
        after_beats: usize,
        context: MeterContinuityStageContext,
    ) -> MeterContinuityTransition {
        let MeterContinuityStageContext {
            action,
            source,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            ..
        } = context;
        MeterContinuityTransition {
            after_beats,
            action,
            source,
            severity: continuity_severity(action, source),
            history: continuity_history(context),
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
        }
    }

    fn continuity_plan(
        plan: MeterContinuityPlanInputs,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityPlan {
        let MeterContinuityPlanInputs {
            action,
            source,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            trusted_beats,
            revalidate_after_beats,
        } = plan;
        let history = continuity_history(MeterContinuityStageContext {
            action,
            source,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            stage_index: 0,
        });
        let (arc, arc_rationale, arc_support) =
            continuity_arc_assessment(MeterContinuityArcInputs {
                source,
                reason,
                confidence,
                unresolved,
                causes,
                current: history,
                refresh,
                first_decay,
                final_decay,
            });
        MeterContinuityPlan {
            action,
            source,
            severity: continuity_severity(action, source),
            history,
            arc,
            arc_rationale,
            arc_support,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            trusted_beats,
            revalidate_after_beats,
            lifecycle: MeterContinuityLifecycle {
                refresh,
                decay: [first_decay, final_decay],
            },
        }
    }

    fn continuity_for(
        action: MeterStateAction,
        reason: MeterStateReason,
        inputs: MeterContinuityInputs<'_>,
    ) -> MeterContinuityRecommendation {
        let MeterContinuityInputs {
            estimate,
            suppression_profile,
            confidence,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        } = inputs;
        let beat_duration = if bpm > 0.0 { 60.0 / bpm } else { 0.0 };
        let pickup_like_phase = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| beat_duration > 0.0 && first_downbeat >= beat_duration * 1.5)
            .unwrap_or(false);
        let phase_displacement_beats = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| {
                if beat_duration > 0.0 {
                    let downbeat_guard = first_downbeat - beat_duration * 0.25;
                    beat_positions_seconds
                        .iter()
                        .copied()
                        .take_while(|&beat| beat < downbeat_guard)
                        .count()
                } else {
                    0
                }
            })
            .unwrap_or(0);
        let recovery_beats = estimate
            .and_then(|estimate| estimate.recovery.as_ref())
            .map(|recovery| recovery.recovered_beats)
            .unwrap_or(0);
        let beats_per_bar = estimate
            .map(|estimate| estimate.beats_per_bar)
            .unwrap_or(4)
            .max(1);
        let support_beats = ((confidence.0 * 12.0).round() as usize).clamp(2, 12);
        let retained_beats = if estimate.is_some() {
            recovery_beats.clamp(6, 24).max(support_beats)
        } else {
            support_beats
        };
        let stage = |after_beats: usize,
                     stage_action: MeterContinuityAction,
                     stage_source: MeterContinuitySource,
                     stage_reason: MeterContinuityReason,
                     stage_index: usize| {
            let stage_trigger = continuity_trigger(stage_action, stage_source, stage_reason);
            let stage_unresolved = unresolved_span(
                stage_trigger,
                after_beats,
                after_beats,
                beats_per_bar,
                phase_displacement_beats,
                stage_index,
            );
            let stage_causes = cause_stack(MeterContinuityCauseInputs {
                action: stage_action,
                source: stage_source,
                reason: stage_reason,
                trigger: stage_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displaced: phase_displacement_beats > 0,
                stage_index,
            });
            transition(
                after_beats,
                MeterContinuityStageContext {
                    action: stage_action,
                    source: stage_source,
                    reason: stage_reason,
                    confidence: continuity_confidence(
                        stage_action,
                        stage_source,
                        confidence,
                        after_beats,
                        stage_index,
                    ),
                    trigger: stage_trigger,
                    unresolved: stage_unresolved,
                    causes: stage_causes,
                    stage_index,
                },
            )
        };
        let plan = |plan_action: MeterContinuityAction,
                    plan_source: MeterContinuitySource,
                    plan_reason: MeterContinuityReason,
                    trusted_beats: usize,
                    revalidate_after_beats: usize,
                    refresh: MeterContinuityTransition,
                    first_decay: MeterContinuityTransition,
                    final_decay: MeterContinuityTransition| {
            let plan_trigger = continuity_trigger(plan_action, plan_source, plan_reason);
            let plan_unresolved = unresolved_span(
                plan_trigger,
                trusted_beats,
                revalidate_after_beats,
                beats_per_bar,
                phase_displacement_beats,
                0,
            );
            let plan_causes = cause_stack(MeterContinuityCauseInputs {
                action: plan_action,
                source: plan_source,
                reason: plan_reason,
                trigger: plan_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displaced: phase_displacement_beats > 0,
                stage_index: 0,
            });
            continuity_plan(
                MeterContinuityPlanInputs {
                    action: plan_action,
                    source: plan_source,
                    reason: plan_reason,
                    confidence: continuity_confidence(
                        plan_action,
                        plan_source,
                        confidence,
                        trusted_beats,
                        0,
                    ),
                    trigger: plan_trigger,
                    unresolved: plan_unresolved,
                    causes: plan_causes,
                    trusted_beats,
                    revalidate_after_beats,
                },
                refresh,
                first_decay,
                final_decay,
            )
        };

        match (action, reason) {
            (MeterStateAction::Lock, _) if pickup_like_phase => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::PhaseDisplacement,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::PhaseDisplacement,
                        1,
                    ),
                    stage(
                        8,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Lock, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, MeterStateReason::TentativeMeter) => {
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        retained_beats.min(8),
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        0,
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            4,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            continuity_reason(
                                MeterContinuityAction::Reacquire,
                                MeterContinuitySource::CurrentMeter,
                                false,
                                true,
                            ),
                            1,
                        ),
                        stage(
                            6,
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Hold, MeterStateReason::DestabilizedHold) => {
                let trailing_beats = if suppression_profile.trailing_confidence.0 > 0.0 {
                    (((suppression_profile.trailing_confidence.0
                        + suppression_profile.trailing_recent_stability)
                        * 8.0)
                        .round() as usize)
                        .clamp(4, 8)
                } else {
                    4
                };
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats,
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Retain,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::PriorStateCarry,
                            0,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats.saturating_sub(2).max(2),
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::RecoveryWindow,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Watch, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    retained_beats,
                    retained_beats.saturating_div(2).max(4),
                    stage(
                        retained_beats.saturating_div(2).max(4),
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        retained_beats.saturating_add(4),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.saturating_add(8),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        6,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Clear, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(6),
                    4,
                    stage(
                        4,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        0,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(4),
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(1),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(2),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
        }
    }

    fn build_meter_state(
        action: MeterStateAction,
        reason: MeterStateReason,
        inputs: MeterContinuityInputs<'_>,
    ) -> MeterStateRecommendation {
        MeterStateRecommendation {
            action,
            reason,
            confidence: inputs.confidence,
            continuity: continuity_for(action, reason, inputs),
        }
    }

    if let Some(estimate) = estimate {
        return match estimate.recommendation {
            MeterRecommendation::Lock => build_meter_state(
                MeterStateAction::Lock,
                MeterStateReason::StableMeter,
                MeterContinuityInputs {
                    estimate: Some(estimate),
                    suppression_profile,
                    confidence: estimate.confidence,
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            ),
            MeterRecommendation::Monitor if estimate.trust == MeterTrustLevel::Recovering => {
                build_meter_state(
                    MeterStateAction::Watch,
                    MeterStateReason::RecoveringMeter,
                    MeterContinuityInputs {
                        estimate: Some(estimate),
                        suppression_profile,
                        confidence: Confidence::new(
                            0.5 * estimate.support_profile.segment_recovery_strength.0
                                + 0.3 * estimate.support_profile.recovery_duration_strength.0
                                + 0.2 * estimate.confidence.0,
                        ),
                        tempo_ambiguity,
                        bpm,
                        beat_positions_seconds,
                    },
                )
            }
            MeterRecommendation::Monitor | MeterRecommendation::Defer => build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::TentativeMeter,
                MeterContinuityInputs {
                    estimate: Some(estimate),
                    suppression_profile,
                    confidence: Confidence::new(
                        0.6 * estimate.confidence.0
                            + 0.4 * estimate.support_profile.whole_track_strength.0,
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            ),
        };
    }

    let pulse_stability =
        (0.65 * rhythm_confidence.0 + 0.35 * (1.0 - tempo_ambiguity.0)).clamp(0.0, 1.0);
    let trailing_recovery_strength = (0.6 * suppression_profile.trailing_confidence.0
        + 0.4 * suppression_profile.trailing_recent_stability)
        .clamp(0.0, 1.0);

    if trailing_recovery_strength >= 0.24 && pulse_stability >= 0.58 {
        if tempo_ambiguity.0 >= 0.43 {
            return build_meter_state(
                MeterStateAction::Clear,
                MeterStateReason::MeterCleared,
                MeterContinuityInputs {
                    estimate: None,
                    suppression_profile,
                    confidence: Confidence::new(
                        (0.5 * tempo_ambiguity.0
                            + 0.3 * trailing_recovery_strength
                            + 0.2 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0)))
                        .clamp(0.0, 1.0),
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            );
        }

        if tempo_ambiguity.0 <= 0.33 {
            return build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::DestabilizedHold,
                MeterContinuityInputs {
                    estimate: None,
                    suppression_profile,
                    confidence: Confidence::new(
                        (0.55 * pulse_stability
                            + 0.25 * suppression_profile.best_confidence.0
                            + 0.20 * suppression_profile.best_support)
                            .clamp(0.0, 1.0),
                    ),
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                },
            );
        }

        build_meter_state(
            MeterStateAction::Watch,
            MeterStateReason::RecoveryEmerging,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.55 * trailing_recovery_strength + 0.45 * pulse_stability).clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    } else if pulse_stability >= 0.55
        && suppression_profile.best_confidence.0 >= 0.12
        && suppression_profile.best_support >= 0.48
        && suppression_profile.best_regularity >= 0.20
    {
        build_meter_state(
            MeterStateAction::Hold,
            MeterStateReason::DestabilizedHold,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.5 * pulse_stability
                        + 0.3 * suppression_profile.best_confidence.0
                        + 0.2 * suppression_profile.best_support)
                        .clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    } else {
        build_meter_state(
            MeterStateAction::Clear,
            MeterStateReason::MeterCleared,
            MeterContinuityInputs {
                estimate: None,
                suppression_profile,
                confidence: Confidence::new(
                    (0.45 * (1.0 - suppression_profile.best_confidence.0)
                        + 0.35 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0))
                        + 0.20 * tempo_ambiguity.0)
                        .clamp(0.0, 1.0),
                ),
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            },
        )
    }
}

pub(crate) fn infer_meter(
    onset_envelope: &[f32],
    meter_cue: &[f32],
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
) -> MeterDecision {
    if onset_envelope.is_empty() || beat_frames.len() < 6 {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
            ambiguity: RhythmStructureAmbiguitySummary {
                kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
                confidence: Confidence::new(0.0),
                primary: None,
                runner_up: None,
                trailing_recovery_confidence: Confidence::new(0.0),
            },
        };
    }

    let beat_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(onset_envelope, *frame, 3))
        .collect();
    let meter_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(meter_cue, *frame, 3))
        .collect();
    let hypotheses = meter_hypotheses(&beat_strengths, &meter_strengths);

    let Some(best) = hypotheses.first().copied() else {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
            ambiguity: RhythmStructureAmbiguitySummary {
                kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
                confidence: Confidence::new(0.0),
                primary: None,
                runner_up: None,
                trailing_recovery_confidence: Confidence::new(0.0),
            },
        };
    };
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let confidence = meter_hypothesis_confidence(best, runner_up);
    let global_estimate = if best.score >= 0.12
        && confidence.0 >= 0.18
        && best.regularity >= 0.35
        && best.support_ratio >= 0.60
    {
        let confidence_breakdown = meter_confidence_breakdown(best, runner_up);
        let downbeat_frames = downbeat_frames_for_hypothesis(beat_frames, 0, best);
        Some((
            MeterEstimate {
                beats_per_bar: best.beats_per_bar,
                confidence,
                detection_kind: MeterDetectionKind::WholeTrack,
                trust: MeterTrustLevel::Tentative,
                recommendation: MeterRecommendation::Defer,
                support_profile: MeterSupportProfile {
                    whole_track_strength: confidence,
                    segment_recovery_strength: Confidence::new(0.0),
                    recovery_duration_strength: Confidence::new(0.0),
                },
                confidence_breakdown,
                recovery: None,
                downbeat_positions_seconds: beat_frames_to_seconds(
                    &downbeat_frames,
                    sample_rate,
                    hop_size,
                ),
            },
            best.regularity,
        ))
    } else {
        None
    };

    let segment_candidate = select_segment_meter_candidate(&beat_strengths, &meter_strengths);
    let trailing_candidate = trailing_meter_window_candidate(&beat_strengths, &meter_strengths);
    let ambiguity = rhythm_structure_ambiguity_summary(&hypotheses, trailing_candidate);
    let support_profile = meter_support_profile(
        global_estimate
            .as_ref()
            .map(|(estimate, _)| estimate.confidence),
        segment_candidate,
    );

    let global_estimate = global_estimate.map(|(mut estimate, regularity)| {
        estimate.support_profile = support_profile;
        estimate.trust = meter_trust_level(
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        estimate.recommendation = meter_recommendation(
            estimate.trust,
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        (estimate, regularity)
    });

    let segment_estimate = if let Some(segment_best) = segment_candidate {
        let downbeat_frames = downbeat_frames_for_hypothesis(
            beat_frames,
            segment_best.start_beat,
            segment_best.hypothesis,
        );
        Some(MeterEstimate {
            beats_per_bar: segment_best.hypothesis.beats_per_bar,
            confidence: segment_best.confidence,
            detection_kind: MeterDetectionKind::SegmentRecovery,
            trust: meter_trust_level(
                MeterDetectionKind::SegmentRecovery,
                segment_best.confidence,
                support_profile,
                segment_best.confidence_breakdown,
            ),
            recommendation: MeterRecommendation::Monitor,
            support_profile,
            confidence_breakdown: segment_best.confidence_breakdown,
            recovery: Some(meter_recovery_context(
                beat_frames,
                sample_rate,
                hop_size,
                segment_best,
            )),
            downbeat_positions_seconds: beat_frames_to_seconds(
                &downbeat_frames,
                sample_rate,
                hop_size,
            ),
        })
        .map(|mut estimate| {
            estimate.recommendation = meter_recommendation(
                estimate.trust,
                estimate.detection_kind,
                estimate.confidence,
                estimate.support_profile,
                estimate.confidence_breakdown,
            );
            estimate
        })
    } else {
        None
    };

    let estimate = match (global_estimate, segment_estimate) {
        (Some((global, global_regularity)), Some(segment))
            if segment.confidence.0 >= global.confidence.0 + 0.04 && global_regularity < 0.58 =>
        {
            Some(segment)
        }
        (Some((global, _)), _) => Some(global),
        (None, Some(segment)) => Some(segment),
        (None, None) => None,
    };

    MeterDecision {
        estimate,
        suppression_profile: MeterSuppressionProfile {
            best_confidence: confidence,
            best_support: best.support_ratio,
            best_regularity: best.regularity,
            trailing_confidence: trailing_candidate
                .map(|candidate| candidate.confidence)
                .unwrap_or(Confidence::new(0.0)),
            trailing_recent_stability: trailing_candidate
                .map(|candidate| candidate.hypothesis.recent_strength)
                .unwrap_or(0.0),
        },
        ambiguity,
    }
}
