use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_triggers_and_unresolved_spans() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, pickup) = analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));
    let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
    let (_, dropout_extended) =
        analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
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
        structured.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::StableRevalidation
    );
    assert_eq!(
        structured
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .failed_revalidations,
        0
    );
    assert_eq!(
        structured
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .beats,
        0
    );

    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::TentativeCarry
    );
    assert!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .failed_revalidations
            >= 1
    );

    assert_eq!(
        pickup.meter_state.continuity.downbeat_phase.trigger,
        super::MeterContinuityTrigger::PhaseRecovery
    );
    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .trigger,
        super::MeterContinuityTrigger::PhaseRecovery
    );
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .unresolved
            .beats
            > pickup
                .meter_state
                .continuity
                .downbeat_phase
                .unresolved
                .beats
    );
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .unresolved
            .failed_revalidations
            > pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[0]
                .unresolved
                .failed_revalidations
    );

    assert_eq!(
        dropout_heavy.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::PriorStateDrift
    );
    assert_eq!(
        dropout_extended.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::RecoveryWindowDrift
    );
    assert!(
        dropout_heavy
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .failed_revalidations
            >= 1
    );
    assert!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[1]
            .unresolved
            .failed_revalidations
            > dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .unresolved
                .failed_revalidations
    );

    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::RecoveryWindowDrift
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .trigger,
        super::MeterContinuityTrigger::RecoveryWindowDrift
    );
    assert!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .beats
            >= sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats
    );
    assert!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .failed_revalidations
            >= sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
    );

    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.trigger,
        super::MeterContinuityTrigger::EvidenceLoss
    );
    assert_eq!(
        mixed_length
            .meter_state
            .continuity
            .bar_length
            .unresolved
            .beats,
        0
    );
}
