use super::*;
use crate::MeterDetectionKind;

#[test]
fn beat_tracker_calibrates_named_preset_families() {
    let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, structured_sparse) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Sparse,
    ));
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
    let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
    let (_, fill) = analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Medium));
    let (_, fill_dense) =
        analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Dense));
    let (_, dropout_light) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Light));
    let (_, dropout_medium) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Medium));
    let (_, dropout) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
    let (_, pickup) = analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
    let (_, late_shift) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::LateShift,
    ));
    let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::MixedLength,
    ));
    let (_, modulation) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::Modulation,
    ));
    let (_, reentry) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::Reentry,
    ));
    let (_, cadential) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::CadentialElongation,
    ));
    let (_, reentry_harmonic) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryHarmonicShift,
    ));
    let (_, reentry_fill) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDenseFill,
    ));
    let (_, reentry_accelerating) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmony,
    ));
    let (_, reentry_decelerating) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmony,
    ));
    let (_, reentry_accelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
    ));
    let (_, reentry_decelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
    ));
    let (_, reentry_accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
    ));
    let (_, reentry_decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
    ));
    let (_, reentry_accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyReset,
    ));
    let (_, reentry_decelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyReset,
    ));
    let (_, reentry_accelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
    ));
    let (_, reentry_decelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
    ));
    let (_, reentry_accelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
    ));
    let (_, reentry_decelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
    ));
    let (_, modulation_fill) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ModulationDenseFill,
    ));
    let (_, modulation_fill_extended) = analyze_preset(RhythmPreset::BarTransition120(
        BarTransitionVariant::ModulationDenseFillExtended,
    ));

    let structured_meter = structured.meter.as_ref().expect("structured meter");
    let section_meter = section.meter.as_ref().expect("section meter");
    let fill_meter = fill.meter.as_ref().expect("fill meter");
    let fill_dense_meter = fill_dense.meter.as_ref().expect("dense fill meter");
    let pickup_meter = pickup.meter.as_ref().expect("pickup meter");
    let late_shift_meter = late_shift.meter.as_ref().expect("late-shift meter");
    let reentry_meter = reentry.meter.as_ref().expect("reentry meter");
    let reentry_harmonic_meter = reentry_harmonic
        .meter
        .as_ref()
        .expect("reentry harmonic meter");
    let reentry_fill_meter = reentry_fill.meter.as_ref().expect("reentry fill meter");
    let reentry_accelerating_meter = reentry_accelerating
        .meter
        .as_ref()
        .expect("reentry accelerating meter");
    let reentry_decelerating_meter = reentry_decelerating
        .meter
        .as_ref()
        .expect("reentry decelerating meter");
    let reentry_accelerating_dense_meter = reentry_accelerating_dense
        .meter
        .as_ref()
        .expect("reentry accelerating dense meter");
    let reentry_decelerating_dense_meter = reentry_decelerating_dense
        .meter
        .as_ref()
        .expect("reentry decelerating dense meter");
    let reentry_accelerating_sustained_reset_meter = reentry_accelerating_sustained_reset
        .meter
        .as_ref()
        .expect("reentry accelerating sustained reset meter");
    let reentry_decelerating_sustained_reset_meter = reentry_decelerating_sustained_reset
        .meter
        .as_ref()
        .expect("reentry decelerating sustained reset meter");

    assert!(neutral.meter.is_none());
    assert!(structured_sparse.meter.is_none());
    assert!(dropout_light.meter.is_none());
    assert!(dropout_medium.meter.is_none());
    assert!(dropout.meter.is_none());
    assert!(mixed_length.meter.is_none());
    assert!(modulation.meter.is_none());
    assert!(cadential.meter.is_none());
    assert!(reentry_accelerating_accent.meter.is_none());
    assert!(reentry_decelerating_accent.meter.is_none());
    assert!(reentry_accelerating_reset.meter.is_none());
    assert!(reentry_decelerating_reset.meter.is_none());
    assert!(reentry_accelerating_cadential.meter.is_none());
    assert!(reentry_decelerating_cadential.meter.is_none());
    assert!(modulation_fill.meter.is_none());
    assert!(modulation_fill_extended.meter.is_none());
    assert!(ambiguous.tempo_ambiguity.0 > neutral.tempo_ambiguity.0);
    assert!(ambiguous.tempo_ambiguity.0 > fill.tempo_ambiguity.0);
    assert!(structured_meter.confidence.0 > 0.2);
    assert_eq!(
        structured_meter.detection_kind,
        MeterDetectionKind::WholeTrack
    );
    assert!(structured_meter.recovery.is_none());
    assert!(
        structured_meter.support_profile.whole_track_strength.0
            > structured_meter.support_profile.segment_recovery_strength.0
    );
    assert!(structured.confidence.0 >= structured_sparse.confidence.0 - 0.05);
    assert!(section_meter.confidence.0 > 0.2);
    assert!(fill_meter.confidence.0 > 0.18);
    assert!(fill_dense_meter.confidence.0 > 0.18);
    assert!(fill_dense.tempo_ambiguity.0 > fill.tempo_ambiguity.0);
    assert!(dropout_light.confidence.0 > dropout_medium.confidence.0);
    assert!(dropout.confidence.0 > 0.6);
    assert_eq!(pickup_meter.beats_per_bar, 4);
    assert_eq!(late_shift_meter.beats_per_bar, 4);
    assert_eq!(reentry_meter.beats_per_bar, 4);
    assert_eq!(reentry_harmonic_meter.beats_per_bar, 4);
    assert_eq!(reentry_fill_meter.beats_per_bar, 4);
    assert_eq!(reentry_accelerating_meter.beats_per_bar, 4);
    assert_eq!(reentry_decelerating_meter.beats_per_bar, 4);
    assert_eq!(reentry_accelerating_dense_meter.beats_per_bar, 4);
    assert_eq!(reentry_decelerating_dense_meter.beats_per_bar, 4);
    assert_eq!(reentry_accelerating_sustained_reset_meter.beats_per_bar, 4);
    assert_eq!(reentry_decelerating_sustained_reset_meter.beats_per_bar, 4);
    assert!(pickup_meter.confidence.0 >= late_shift_meter.confidence.0 - 0.1);
    assert!(reentry_meter.confidence.0 > 0.18);
    assert!(reentry_harmonic_meter.confidence.0 > 0.18);
    assert!(reentry_fill_meter.confidence.0 > 0.18);
    assert!(reentry_accelerating_meter.confidence.0 > 0.18);
    assert!(reentry_decelerating_meter.confidence.0 > 0.18);
    assert!(reentry_accelerating_dense_meter.confidence.0 > 0.18);
    assert!(reentry_decelerating_dense_meter.confidence.0 > 0.18);
    assert!(reentry_accelerating_sustained_reset_meter.confidence.0 > 0.18);
    assert!(reentry_decelerating_sustained_reset_meter.confidence.0 > 0.18);
    assert!(reentry_accelerating_cadential.confidence.0 > 0.18);
    assert!(reentry_decelerating_cadential.confidence.0 > 0.18);
    assert!(late_shift.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
    assert!(mixed_length.confidence.0 < pickup.confidence.0);
    assert!(reentry.confidence.0 > modulation.confidence.0);
    assert!(reentry_accelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
    assert!(reentry_decelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
    assert!(reentry_accelerating_dense.confidence.0 > reentry_accelerating_dense.tempo_ambiguity.0);
    assert!(reentry_decelerating_dense.confidence.0 > reentry_decelerating_dense.tempo_ambiguity.0);
    assert!(
        reentry_accelerating_accent.tempo_ambiguity.0
            >= reentry_accelerating_dense.tempo_ambiguity.0 - 0.03
    );
    assert!(
        reentry_decelerating_accent.tempo_ambiguity.0
            >= reentry_decelerating_dense.tempo_ambiguity.0 - 0.03
    );
    assert!(reentry_accelerating_reset.confidence.0 > 0.18);
    assert!(reentry_decelerating_reset.confidence.0 > 0.18);
    assert!(
        reentry_accelerating_sustained_reset.confidence.0
            >= reentry_accelerating_reset.confidence.0 - 0.03
    );
    assert!(
        reentry_decelerating_sustained_reset.confidence.0
            >= reentry_decelerating_reset.confidence.0 - 0.03
    );
    assert!(
        reentry_accelerating_cadential.confidence.0
            >= reentry_accelerating_reset.confidence.0 - 0.05
    );
    assert!(
        reentry_decelerating_cadential.confidence.0
            >= reentry_decelerating_reset.confidence.0 - 0.05
    );
    assert!(modulation_fill.tempo_ambiguity.0 >= reentry_harmonic.tempo_ambiguity.0);
    assert!(modulation_fill.tempo_ambiguity.0 >= reentry_fill.tempo_ambiguity.0 - 0.05);
    assert!(modulation_fill_extended.tempo_ambiguity.0 >= modulation_fill.tempo_ambiguity.0);
    assert!(cadential.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.05);
    assert!(section.confidence.0 > section.tempo_ambiguity.0);
    assert!(fill.confidence.0 > fill.tempo_ambiguity.0);
    assert!(section.confidence.0 > ambiguous.confidence.0 - 0.1);
}
