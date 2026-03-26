use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_history_across_transition_families() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));
    let (_, dropout_extended) =
        analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
    let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
    ));
    let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ModulationDenseFillExtended,
    ));

    assert_eq!(
        structured.meter_state.continuity.bar_length.history,
        super::MeterContinuityHistory::Reinforcing
    );
    assert_eq!(
        structured
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .history,
        super::MeterContinuityHistory::Reinforcing
    );

    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.history,
        super::MeterContinuityHistory::Preserving
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.downbeat_phase.history,
        super::MeterContinuityHistory::Degrading
    );

    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .history,
        super::MeterContinuityHistory::Degrading
    );
    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .history,
        super::MeterContinuityHistory::Degrading
    );

    assert_eq!(
        dropout_extended.meter_state.continuity.bar_length.history,
        super::MeterContinuityHistory::Preserving
    );
    assert_eq!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .history,
        super::MeterContinuityHistory::Degrading
    );

    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.history,
        super::MeterContinuityHistory::Preserving
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .history,
        super::MeterContinuityHistory::Preserving
    );
    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .history,
        super::MeterContinuityHistory::Reinforcing
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .history,
        super::MeterContinuityHistory::Reinforcing
    );

    assert_eq!(
        modulation_extended
            .meter_state
            .continuity
            .bar_length
            .history,
        super::MeterContinuityHistory::Degrading
    );
}
