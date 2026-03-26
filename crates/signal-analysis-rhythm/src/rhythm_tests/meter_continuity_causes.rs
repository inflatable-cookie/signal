use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_cause_stacks_for_stacked_instability() {
    let contains_cause = |stack: super::MeterContinuityCauseStack,
                          cause: super::MeterContinuityCause| {
        stack.primary == cause
            || stack
                .secondary
                .into_iter()
                .flatten()
                .any(|entry| entry == cause)
    };

    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));
    let (_, dropout_extended) =
        analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
    let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ModulationDenseFillExtended,
    ));

    assert!(contains_cause(
        ambiguous.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::EvidenceLoss,
    ));
    assert!(contains_cause(
        ambiguous.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::TempoAmbiguity,
    ));
    assert!(contains_cause(
        ambiguous.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::SparseMeterSupport,
    ));
    assert!(ambiguous.meter_state.continuity.bar_length.causes.count >= 2);

    assert!(contains_cause(
        pickup_extended.meter_state.continuity.downbeat_phase.causes,
        super::MeterContinuityCause::PhaseDisplacement,
    ));
    assert!(contains_cause(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .causes,
        super::MeterContinuityCause::EvidenceLoss,
    ));
    assert!(
        pickup_extended
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .causes
            .count
            >= 2
    );

    assert!(contains_cause(
        dropout_extended.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::RecoveryWindowInstability,
    ));
    assert!(contains_cause(
        dropout_extended.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::TempoAmbiguity,
    ));
    assert!(contains_cause(
        dropout_extended.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::IrregularBarStructure,
    ));
    assert!(
        dropout_extended
            .meter_state
            .continuity
            .bar_length
            .causes
            .count
            >= 2
    );

    assert!(contains_cause(
        modulation_extended.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::EvidenceLoss,
    ));
    assert!(contains_cause(
        modulation_extended.meter_state.continuity.bar_length.causes,
        super::MeterContinuityCause::TempoAmbiguity,
    ));
    assert!(
        modulation_extended
            .meter_state
            .continuity
            .bar_length
            .causes
            .count
            >= 2
    );
}
