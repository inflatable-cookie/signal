use super::*;

#[test]
fn beat_tracker_calibrates_bar_transition_variant_monotonicity() {
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
    assert!(pickup_meter.confidence.0 > 0.2);
    assert!(late_shift_meter.confidence.0 > 0.18);
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
    assert!(pickup_meter.confidence.0 >= late_shift_meter.confidence.0 - 0.12);
    assert!(late_shift.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
    assert!(pickup.confidence.0 > mixed_length.confidence.0);
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
    assert!(modulation.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
}
