use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_severity_across_lifecycle_stages() {
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
        structured.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Confirmed
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.lifecycle.decay[0].severity,
        super::MeterContinuitySeverity::Guarded
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.lifecycle.decay[1].severity,
        super::MeterContinuitySeverity::Cleared
    );

    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Guarded
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .severity,
        super::MeterContinuitySeverity::Fragile
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[1]
            .severity,
        super::MeterContinuitySeverity::Cleared
    );

    assert_eq!(
        dropout_heavy.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Fragile
    );
    assert_eq!(
        dropout_heavy.meter_state.continuity.downbeat_phase.severity,
        super::MeterContinuitySeverity::Fragile
    );
    assert_eq!(
        dropout_extended.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Guarded
    );
    assert_eq!(
        dropout_extended
            .meter_state
            .continuity
            .downbeat_phase
            .severity,
        super::MeterContinuitySeverity::Fragile
    );

    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .severity,
        super::MeterContinuitySeverity::Fragile
    );
    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .refresh
            .severity,
        super::MeterContinuitySeverity::Confirmed
    );

    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Guarded
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .severity,
        super::MeterContinuitySeverity::Guarded
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .severity,
        super::MeterContinuitySeverity::Fragile
    );
    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.severity,
        super::MeterContinuitySeverity::Cleared
    );
    assert_eq!(
        mixed_length
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .refresh
            .severity,
        super::MeterContinuitySeverity::Cleared
    );
}
