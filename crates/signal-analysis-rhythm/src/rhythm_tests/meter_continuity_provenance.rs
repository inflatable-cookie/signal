use super::*;

#[test]
fn beat_tracker_calibrates_meter_continuity_across_transition_families() {
    let (_, pickup) = analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
    let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::MixedLength,
    ));
    let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, cadential_reanchor) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
    ));

    assert!(pickup.meter.is_some());
    assert!(mixed_length.meter.is_none());
    assert!(sustained_reset.meter.is_some());
    assert!(cadential_reanchor.meter.is_none());

    assert_eq!(
        pickup.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        pickup
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        pickup.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        pickup
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .refresh
            .action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Retain
    );
    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .downbeat_phase
            .lifecycle
            .decay[1]
            .action,
        super::MeterContinuityAction::Clear
    );
    assert_eq!(
        mixed_length.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Clear
    );
    assert_eq!(
        mixed_length.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Clear
    );
    assert_eq!(
        cadential_reanchor.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Retain
    );
    assert_eq!(
        cadential_reanchor
            .meter_state
            .continuity
            .downbeat_phase
            .action,
        super::MeterContinuityAction::Reacquire
    );
}
