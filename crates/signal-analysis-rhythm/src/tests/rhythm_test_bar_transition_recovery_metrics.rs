use super::*;
use crate::{MeterDetectionKind, MeterTrustLevel};

#[test]
fn beat_tracker_calibrates_multi_stage_reentry_harmonic_drift() {
    let (_, reentry) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::Reentry,
    ));
    let (_, accelerating) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmony,
    ));
    let (_, decelerating) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmony,
    ));

    let accelerating_meter = accelerating
        .meter
        .as_ref()
        .expect("accelerating recovery meter");
    let decelerating_meter = decelerating
        .meter
        .as_ref()
        .expect("decelerating recovery meter");
    assert_eq!(accelerating_meter.beats_per_bar, 4);
    assert_eq!(decelerating_meter.beats_per_bar, 4);
    assert!(accelerating_meter.confidence.0 > 0.18);
    assert!(decelerating_meter.confidence.0 > 0.18);
    assert!(accelerating.confidence.0 > accelerating.tempo_ambiguity.0);
    assert!(decelerating.confidence.0 > decelerating.tempo_ambiguity.0);
    assert!(accelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
    assert!(decelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
    assert!(
        decelerating_meter.confidence.0 >= accelerating_meter.confidence.0 - 0.12,
        "decelerating confidence {} accelerating {}",
        decelerating_meter.confidence.0,
        accelerating_meter.confidence.0
    );
}

#[test]
fn beat_tracker_calibrates_multistage_reentry_density_vs_accent_drift() {
    let (_, accelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
    ));
    let (_, decelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
    ));
    let (_, accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
    ));
    let (_, decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
    ));

    let accelerating_dense_meter = accelerating_dense
        .meter
        .as_ref()
        .expect("accelerating dense recovery meter");
    let decelerating_dense_meter = decelerating_dense
        .meter
        .as_ref()
        .expect("decelerating dense recovery meter");
    assert_eq!(accelerating_dense_meter.beats_per_bar, 4);
    assert_eq!(decelerating_dense_meter.beats_per_bar, 4);
    assert!(accelerating_accent.meter.is_none());
    assert!(decelerating_accent.meter.is_none());
    assert!(accelerating_dense.tempo_ambiguity.0 > 0.12);
    assert!(decelerating_dense.tempo_ambiguity.0 > 0.12);
    assert!(accelerating_accent.tempo_ambiguity.0 >= accelerating_dense.tempo_ambiguity.0 - 0.03);
    assert!(decelerating_accent.tempo_ambiguity.0 >= decelerating_dense.tempo_ambiguity.0 - 0.03);
    assert!(accelerating_dense.confidence.0 >= accelerating_accent.confidence.0 - 0.05);
    assert!(decelerating_dense.confidence.0 >= decelerating_accent.confidence.0 - 0.05);
}

#[test]
fn beat_tracker_calibrates_reanchor_recovery_after_destabilized_window() {
    let (_, accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
    ));
    let (_, decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
    ));
    let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyReset,
    ));
    let (_, decelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyReset,
    ));
    let (_, accelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, decelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
    ));
    let (_, accelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
    ));
    let (_, decelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
    ));

    assert!(accelerating_accent.meter.is_none());
    assert!(decelerating_accent.meter.is_none());
    assert!(accelerating_reset.meter.is_none());
    assert!(decelerating_reset.meter.is_none());
    assert!(accelerating_cadential.meter.is_none());
    assert!(decelerating_cadential.meter.is_none());
    assert_eq!(
        accelerating_sustained_reset
            .meter
            .as_ref()
            .expect("accelerating sustained reset meter")
            .beats_per_bar,
        4
    );
    assert_eq!(
        decelerating_sustained_reset
            .meter
            .as_ref()
            .expect("decelerating sustained reset meter")
            .beats_per_bar,
        4
    );
    assert!(accelerating_reset.confidence.0 > 0.18);
    assert!(decelerating_reset.confidence.0 > 0.18);
    assert!(accelerating_sustained_reset.confidence.0 >= accelerating_reset.confidence.0 - 0.03);
    assert!(decelerating_sustained_reset.confidence.0 >= decelerating_reset.confidence.0 - 0.03);
    assert!(accelerating_cadential.confidence.0 >= accelerating_reset.confidence.0 - 0.05);
    assert!(decelerating_cadential.confidence.0 >= decelerating_reset.confidence.0 - 0.05);
    assert!(accelerating_cadential.confidence.0 > 0.18);
    assert!(decelerating_cadential.confidence.0 > 0.18);
}

#[test]
fn beat_tracker_calibrates_sustained_segment_recovery_vs_prolonged_modulation() {
    let (_, accelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, decelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
    ));
    let (_, prolonged_modulation) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ModulationDenseFillExtended,
    ));

    let accelerating_meter = accelerating_sustained_reset
        .meter
        .as_ref()
        .expect("accelerating sustained recovery meter");
    let decelerating_meter = decelerating_sustained_reset
        .meter
        .as_ref()
        .expect("decelerating sustained recovery meter");
    assert_eq!(accelerating_meter.beats_per_bar, 4);
    assert_eq!(decelerating_meter.beats_per_bar, 4);
    assert!(accelerating_meter.confidence.0 > 0.18);
    assert!(decelerating_meter.confidence.0 > 0.18);
    assert_eq!(
        accelerating_meter.detection_kind,
        MeterDetectionKind::SegmentRecovery
    );
    assert_eq!(accelerating_meter.trust, MeterTrustLevel::Recovering);
    assert_eq!(
        decelerating_meter.detection_kind,
        MeterDetectionKind::SegmentRecovery
    );
    assert_eq!(decelerating_meter.trust, MeterTrustLevel::Recovering);
    let accelerating_recovery = accelerating_meter
        .recovery
        .as_ref()
        .expect("accelerating recovery context");
    let decelerating_recovery = decelerating_meter
        .recovery
        .as_ref()
        .expect("decelerating recovery context");
    assert!(accelerating_recovery.recovered_beats >= 8);
    assert!(decelerating_recovery.recovered_beats >= 8);
    assert!(accelerating_recovery.supporting_windows >= 2);
    assert!(decelerating_recovery.supporting_windows >= 2);
    assert!(accelerating_recovery.end_seconds > accelerating_recovery.start_seconds);
    assert!(decelerating_recovery.end_seconds > decelerating_recovery.start_seconds);
    assert!(
        accelerating_meter
            .support_profile
            .segment_recovery_strength
            .0
            > accelerating_meter.support_profile.whole_track_strength.0
    );
    assert!(
        decelerating_meter
            .support_profile
            .segment_recovery_strength
            .0
            > decelerating_meter.support_profile.whole_track_strength.0
    );
    assert!(
        accelerating_meter
            .support_profile
            .recovery_duration_strength
            .0
            > 0.5
    );
    assert!(
        decelerating_meter
            .support_profile
            .recovery_duration_strength
            .0
            > 0.5
    );
    assert!(prolonged_modulation.meter.is_none());
    assert!(
        prolonged_modulation.tempo_ambiguity.0
            >= accelerating_sustained_reset.tempo_ambiguity.0 - 0.02
    );
    assert!(
        prolonged_modulation.tempo_ambiguity.0
            >= decelerating_sustained_reset.tempo_ambiguity.0 - 0.02
    );
}
