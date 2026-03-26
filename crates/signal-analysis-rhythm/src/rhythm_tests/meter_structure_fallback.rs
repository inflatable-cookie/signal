use super::*;

#[test]
fn beat_tracker_exposes_whole_track_structure_summary_for_stable_meter() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));

    let summary = structured
        .rhythm_structure_summary()
        .expect("structured rhythm structure summary");

    assert_eq!(summary.beats_per_bar, 4);
    assert_eq!(
        summary.detection_kind,
        super::MeterDetectionKind::WholeTrack
    );
    assert_eq!(summary.trust, super::MeterTrustLevel::Stable);
    assert_eq!(summary.recommendation, super::MeterRecommendation::Lock);
    assert_eq!(summary.continuity.action, structured.meter_state.action);
    assert_eq!(
        summary.continuity.bar_length_action,
        structured.meter_state.continuity.bar_length.action
    );
    assert_eq!(
        summary.continuity.downbeat_phase_action,
        structured.meter_state.continuity.downbeat_phase.action
    );
    assert_eq!(summary.bar_count, summary.downbeat_positions_seconds.len());
    assert!(summary.bar_count >= 2);
    assert_eq!(summary.recovered_bar_count, 0);
    assert!(summary.recovery.is_none());
    assert!(summary
        .bars
        .iter()
        .all(|bar| matches!(bar.support, super::BarSupportKind::WholeTrack)));
    assert_eq!(
        summary.bars.first().map(|bar| bar.start_seconds),
        summary.downbeat_positions_seconds.first().copied()
    );
}

#[test]
fn beat_tracker_exposes_recovery_backed_structure_summary_for_segment_meter() {
    let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));

    let summary = sustained_reset
        .rhythm_structure_summary()
        .expect("recovery-backed rhythm structure summary");

    assert_eq!(
        summary.detection_kind,
        super::MeterDetectionKind::SegmentRecovery
    );
    assert!(summary.recovery.is_some());
    assert!(summary.recovered_bar_count > 0);
    assert!(summary
        .bars
        .iter()
        .any(|bar| matches!(bar.support, super::BarSupportKind::RecoveryWindow)));
    assert_eq!(
        summary.continuity.action,
        sustained_reset.meter_state.action
    );
    assert_eq!(
        summary.continuity.reason,
        sustained_reset.meter_state.reason
    );
    let recovery = summary.recovery.as_ref().expect("recovery context");
    assert!(summary.bars.iter().any(|bar| {
        matches!(bar.support, super::BarSupportKind::RecoveryWindow)
            && bar.start_seconds <= recovery.end_seconds
    }));
}

#[test]
fn beat_tracker_structure_assessment_surfaces_weak_accent_ambiguity() {
    let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);

    let assessment = weak_backbeat.rhythm_structure_assessment();

    assert!(assessment.structure.is_some());
    assert_eq!(
        assessment.ambiguity.kind,
        super::RhythmStructureAmbiguityKind::WeakAccent
    );
    assert!(assessment.ambiguity.runner_up.is_some());
    assert!(assessment.ambiguity.confidence.0 > 0.2);
    assert_eq!(assessment.fallback.action, super::MeterStateAction::Hold);
    assert_eq!(
        assessment.fallback.downbeat_phase_action,
        super::MeterContinuityAction::Reacquire
    );
}

#[test]
fn beat_tracker_structure_assessment_surfaces_competing_meter_ambiguity() {
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

    let assessment = ambiguous.rhythm_structure_assessment();

    let primary = assessment
        .ambiguity
        .primary
        .expect("primary ambiguity candidate");
    let runner_up = assessment
        .ambiguity
        .runner_up
        .expect("runner-up ambiguity candidate");

    assert_ne!(primary.beats_per_bar, runner_up.beats_per_bar);
    assert!(assessment.ambiguity.confidence.0 > 0.2);
}

#[test]
fn beat_tracker_structure_assessment_surfaces_phase_fallback_for_pickup_extension() {
    let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::PickupExtended,
    ));

    let assessment = pickup_extended.rhythm_structure_assessment();

    assert!(assessment.structure.is_some());
    assert_ne!(
        assessment.ambiguity.kind,
        super::RhythmStructureAmbiguityKind::InsufficientEvidence
    );
    assert_eq!(
        assessment.fallback.downbeat_phase_action,
        super::MeterContinuityAction::Reacquire
    );
}

#[test]
fn beat_tracker_structure_assessment_surfaces_recovery_window_fallback_without_meter() {
    let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyReset,
    ));

    let assessment = accelerating_reset.rhythm_structure_assessment();

    assert!(assessment.structure.is_none());
    assert!(assessment.fallback.recovery_window_available);
    assert_eq!(assessment.fallback.action, super::MeterStateAction::Watch);
    assert_eq!(
        assessment.ambiguity.kind,
        super::RhythmStructureAmbiguityKind::RecoveryWindowFallback
    );
    assert!(assessment.fallback.trailing_recovery_confidence.0 > 0.0);
}

#[test]
fn beat_tracker_calibrates_meter_recommendations_across_action_categories() {
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

    assert_eq!(
        structured_meter.recommendation,
        super::MeterRecommendation::Lock
    );
    assert_eq!(
        structured.meter_state.reason,
        super::MeterStateReason::StableMeter
    );
    assert_eq!(
        structured.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Lock
    );
    assert_eq!(
        sustained_reset_meter.recommendation,
        super::MeterRecommendation::Monitor
    );
    assert_eq!(
        sustained_reset.meter_state.reason,
        super::MeterStateReason::RecoveringMeter
    );
    assert_eq!(
        sustained_reset.meter_state.continuity.downbeat_phase.action,
        super::MeterContinuityAction::Reacquire
    );
    assert_eq!(
        weak_backbeat_meter.recommendation,
        super::MeterRecommendation::Defer
    );
    assert_eq!(
        weak_backbeat.meter_state.reason,
        super::MeterStateReason::TentativeMeter
    );
    assert_eq!(
        weak_backbeat.meter_state.continuity.bar_length.action,
        super::MeterContinuityAction::Retain
    );
    assert!(structured_meter.confidence.0 > weak_backbeat_meter.confidence.0);
    assert!(
        sustained_reset_meter
            .support_profile
            .recovery_duration_strength
            .0
            > 0.5
    );
    assert!(weak_backbeat_meter.recovery.is_none());
}
