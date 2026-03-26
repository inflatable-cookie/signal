use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_arc_rationales_and_support() {
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
        structured.meter_state.continuity.bar_length.arc_rationale,
        super::MeterContinuityArcRationale::RefreshStrength
    );
    assert!(
        structured
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .refresh_strength
            .0
            > structured
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .drift_pressure
                .0
    );

    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .arc_rationale,
        super::MeterContinuityArcRationale::UnresolvedDrift
    );
    assert!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .drift_pressure
            .0
            > weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
    );

    assert_eq!(
        ambiguous.meter_state.continuity.bar_length.arc_rationale,
        super::MeterContinuityArcRationale::EvidenceLoss
    );
    assert!(
        ambiguous
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .structural_pressure
            .0
            >= 0.5
    );

    assert_eq!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .arc_rationale,
        super::MeterContinuityArcRationale::UnresolvedDrift
    );
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .arc_support
            .refresh_strength
            .0
            > 0.8
    );
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .arc_support
            .drift_pressure
            .0
            > pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_support
                .structural_pressure
                .0
    );

    assert_eq!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .arc_rationale,
        super::MeterContinuityArcRationale::StructuralInstability
    );
    assert!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .structural_pressure
            .0
            > sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
    );

    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .arc_rationale,
        super::MeterContinuityArcRationale::RefreshStrength
    );
    assert_eq!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .arc_rationale,
        super::MeterContinuityArcRationale::RefreshStrength
    );
    assert!(
        long_sustained_reset
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .refresh_strength
            .0
            >= sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
    );

    assert_eq!(
        modulation_extended
            .meter_state
            .continuity
            .bar_length
            .arc_rationale,
        super::MeterContinuityArcRationale::EvidenceLoss
    );
    assert!(
        modulation_extended
            .meter_state
            .continuity
            .bar_length
            .arc_support
            .structural_pressure
            .0
            >= ambiguous
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
    );
}
