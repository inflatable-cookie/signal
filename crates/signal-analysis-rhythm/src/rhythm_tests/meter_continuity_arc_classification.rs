use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_arcs_across_transition_families() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
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
        structured.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Recovering
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Stalling
    );
    assert_eq!(
        ambiguous.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Collapsing
    );
    assert_eq!(
        pickup_extended.meter_state.continuity.downbeat_phase.arc,
        super::MeterContinuityArc::Collapsing
    );
    assert_eq!(
        dropout_extended.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Stalling
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Recovering
    );
    assert_eq!(
        long_sustained_reset.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Recovering
    );
    assert_eq!(
        modulation_extended.meter_state.continuity.bar_length.arc,
        super::MeterContinuityArc::Collapsing
    );
}
