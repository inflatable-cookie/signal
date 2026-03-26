use super::*;

#[test]
fn beat_tracker_calibrates_meter_trust_levels_across_public_categories() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
    let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));

    let structured_meter = structured.meter.as_ref().expect("structured meter");
    let weak_backbeat_meter = weak_backbeat.meter.as_ref().expect("weak backbeat meter");
    let sustained_reset_meter = sustained_reset
        .meter
        .as_ref()
        .expect("sustained reset meter");

    assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
    assert_eq!(
        structured_meter.recommendation,
        super::MeterRecommendation::Lock
    );
    assert_eq!(structured.meter_state.action, super::MeterStateAction::Lock);
    assert_eq!(
        structured.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        structured.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.source,
        super::MeterContinuitySource::CurrentMeter
    );
    assert_eq!(
        structured
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.lifecycle.decay[0].action,
        super::MeterContinuityAction::Retain
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.lifecycle.decay[1].action,
        super::MeterContinuityAction::Clear
    );
    assert_eq!(weak_backbeat_meter.trust, super::MeterTrustLevel::Tentative);
    assert_eq!(
        weak_backbeat_meter.recommendation,
        super::MeterRecommendation::Defer
    );
    assert_eq!(
        weak_backbeat.meter_state.action,
        super::MeterStateAction::Hold
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Retain
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.source,
        super::MeterContinuitySource::CurrentMeter
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[0]
            .action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        weak_backbeat
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[1]
            .action,
        super::MeterContinuityAction::Clear
    );
    assert_eq!(
        sustained_reset_meter.trust,
        super::MeterTrustLevel::Recovering
    );
    assert_eq!(
        sustained_reset_meter.recommendation,
        super::MeterRecommendation::Monitor
    );
    assert_eq!(
        sustained_reset.meter_state.action,
        super::MeterStateAction::Watch
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Retain
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.bar_length.source,
        super::MeterContinuitySource::RecoveryWindow
    );
    assert_eq!(
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .refresh
            .action,
        super::MeterContinuityAction::Lock
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
        sustained_reset
            .meter_state
            .continuity
            .bar_length
            .lifecycle
            .decay[1]
            .action,
        super::MeterContinuityAction::Clear
    );
    assert!(
        structured_meter.support_profile.whole_track_strength.0
            >= weak_backbeat_meter.support_profile.whole_track_strength.0
    );
    assert!(
        sustained_reset_meter
            .support_profile
            .segment_recovery_strength
            .0
            > sustained_reset_meter.support_profile.whole_track_strength.0
    );
}
