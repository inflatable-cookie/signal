use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_reason_and_confidence_surface() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
    let (_, dropout_extended) =
        analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));
    let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
    ));
    let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::MixedLength,
    ));

    assert_eq!(
        structured.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::StableEvidence
    );
    assert_eq!(
        structured
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .reason,
        super::MeterContinuityReason::StableEvidence
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.lifecycle.decay[0].reason,
        super::MeterContinuityReason::RevalidationDecay
    );

    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::TentativeEvidence
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .reason,
        super::MeterContinuityReason::RevalidationDecay
    );

    assert_eq!(
        dropout_heavy.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::PriorStateCarry
    );
    assert_eq!(
        dropout_heavy
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .reason,
        super::MeterContinuityReason::RevalidationDecay
    );

    assert_eq!(
        dropout_extended.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::RecoveryWindowSupport
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::RecoveryWindowSupport
    );
    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .reason,
        super::MeterContinuityReason::StableEvidence
    );

    assert_eq!(
        pickup_extended.meter_state.continuity.downbeat_phase.reason,
        super::MeterContinuityReason::PhaseDisplacement
    );
    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .refresh
            .reason,
        super::MeterContinuityReason::StableEvidence
    );

    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.reason,
        super::MeterContinuityReason::InsufficientEvidence
    );
    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.confidence.0,
        0.0
    );

    assert!(
        structured.meter_state.continuity.bar_length.confidence.0
            > weak_backbeat.meter_state.continuity.bar_length.confidence.0
    );
    assert!(
        weak_backbeat.meter_state.continuity.bar_length.confidence.0
            > weak_backbeat
                .meter_state
                .continuity
                .downbeat_phase
                .confidence
                .0
    );
    assert!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .confidence
            .0
            > dropout_heavy.meter_state.continuity.bar_length.confidence.0
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
                .0
    );
    assert!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .confidence
            .0
            > sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .confidence
                .0
    );
    assert!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .confidence
            .0
            >= sustained_reset
                .meter_state
                .continuity
                .bar_length
                .confidence
                .0
    );
    assert!(
        dropout_heavy
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .confidence
            .0
            > dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .confidence
                .0
    );
}
