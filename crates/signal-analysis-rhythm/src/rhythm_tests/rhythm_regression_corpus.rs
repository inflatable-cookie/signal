use super::*;

struct TempoExpectation {
    action: TempoStateAction,
    reason: TempoStateReason,
    continuity_action: TempoContinuityAction,
    continuity_reason: TempoContinuityReason,
}

struct MeterExpectation {
    action: MeterStateAction,
    reason: MeterStateReason,
    bar_action: MeterContinuityAction,
    bar_reason: MeterContinuityReason,
    downbeat_action: MeterContinuityAction,
}

fn assert_tempo_case(label: &str, result: &BeatAnalysisResult, expected: TempoExpectation) {
    assert_eq!(
        result.tempo_state.action, expected.action,
        "{label} tempo action"
    );
    assert_eq!(
        result.tempo_state.reason, expected.reason,
        "{label} tempo reason"
    );
    assert_eq!(
        result.tempo_state.continuity.action, expected.continuity_action,
        "{label} tempo continuity action"
    );
    assert_eq!(
        result.tempo_state.continuity.reason, expected.continuity_reason,
        "{label} tempo continuity reason"
    );
}

fn assert_meter_case(label: &str, result: &BeatAnalysisResult, expected: MeterExpectation) {
    assert_eq!(
        result.meter_state.action, expected.action,
        "{label} meter action"
    );
    assert_eq!(
        result.meter_state.reason, expected.reason,
        "{label} meter reason"
    );
    assert_eq!(
        result.meter_state.continuity.bar_length.action, expected.bar_action,
        "{label} bar continuity action"
    );
    assert_eq!(
        result.meter_state.continuity.bar_length.reason, expected.bar_reason,
        "{label} bar continuity reason"
    );
    assert_eq!(
        result.meter_state.continuity.downbeat_phase.action, expected.downbeat_action,
        "{label} downbeat continuity action"
    );
}

#[test]
fn rhythm_regression_bundle_preserves_post_normalization_tempo_and_meter_surface() {
    let (_, neutral_click) = analyze_preset(RhythmPreset::NeutralClick120);
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));
    let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyReset,
    ));
    let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::MixedLength,
    ));

    assert_tempo_case(
        "neutral_click",
        &neutral_click,
        TempoExpectation {
            action: TempoStateAction::Lock,
            reason: TempoStateReason::StableIntegerTempo,
            continuity_action: TempoContinuityAction::Lock,
            continuity_reason: TempoContinuityReason::IntegerTempoSnap,
        },
    );
    assert_tempo_case(
        "weak_backbeat",
        &weak_backbeat,
        TempoExpectation {
            action: TempoStateAction::Lock,
            reason: TempoStateReason::StableRefinedTempo,
            continuity_action: TempoContinuityAction::Lock,
            continuity_reason: TempoContinuityReason::StableTempo,
        },
    );

    assert_meter_case(
        "weak_backbeat",
        &weak_backbeat,
        MeterExpectation {
            action: MeterStateAction::Hold,
            reason: MeterStateReason::TentativeMeter,
            bar_action: MeterContinuityAction::Retain,
            bar_reason: MeterContinuityReason::TentativeEvidence,
            downbeat_action: MeterContinuityAction::Reacquire,
        },
    );
    assert_meter_case(
        "pickup_extended",
        &pickup_extended,
        MeterExpectation {
            action: MeterStateAction::Lock,
            reason: MeterStateReason::StableMeter,
            bar_action: MeterContinuityAction::Lock,
            bar_reason: MeterContinuityReason::StableEvidence,
            downbeat_action: MeterContinuityAction::Reacquire,
        },
    );
    assert_meter_case(
        "accelerating_reset",
        &accelerating_reset,
        MeterExpectation {
            action: MeterStateAction::Watch,
            reason: MeterStateReason::RecoveryEmerging,
            bar_action: MeterContinuityAction::Retain,
            bar_reason: MeterContinuityReason::RecoveryWindowSupport,
            downbeat_action: MeterContinuityAction::Reacquire,
        },
    );
    assert_meter_case(
        "mixed_length",
        &mixed_length,
        MeterExpectation {
            action: MeterStateAction::Clear,
            reason: MeterStateReason::MeterCleared,
            bar_action: MeterContinuityAction::Clear,
            bar_reason: MeterContinuityReason::InsufficientEvidence,
            downbeat_action: MeterContinuityAction::Clear,
        },
    );

    assert!(
        neutral_click.tempo_state.confidence.0 > weak_backbeat.tempo_state.confidence.0,
        "stable integer posture should still outrank guarded refined posture"
    );
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .refresh
            .confidence
            .0
            > pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .confidence
                .0,
        "pickup recovery should still refresh toward lock strength"
    );
    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.confidence.0, 0.0,
        "clear posture should keep explicit zero-confidence evidence loss"
    );
}
