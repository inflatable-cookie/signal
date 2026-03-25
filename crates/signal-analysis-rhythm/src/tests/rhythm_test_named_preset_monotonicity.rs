use super::*;

#[test]
fn beat_tracker_calibrates_dropout_variant_monotonicity() {
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let (_, light) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Light));
    let (_, medium) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Medium));
    let (_, heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));

    let structured_meter = structured.meter.as_ref().expect("structured meter");
    assert_eq!(structured_meter.beats_per_bar, 4);
    assert!(light.meter.is_none());
    assert!(medium.meter.is_none());
    assert!(heavy.meter.is_none());
    assert!(light.confidence.0 > medium.confidence.0);
    assert!(heavy.confidence.0 > 0.6);
    assert!(structured.confidence.0 >= light.confidence.0 - 0.05);
}

#[test]
fn beat_tracker_calibrates_fill_density_variant_monotonicity() {
    let (_, medium) = analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Medium));
    let (_, dense) = analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Dense));

    let medium_meter = medium.meter.as_ref().expect("medium fill meter");
    let dense_meter = dense.meter.as_ref().expect("dense fill meter");
    assert_eq!(medium_meter.beats_per_bar, 4);
    assert_eq!(dense_meter.beats_per_bar, 4);
    assert!(dense.tempo_ambiguity.0 > medium.tempo_ambiguity.0);
    assert!(medium_meter.confidence.0 >= dense_meter.confidence.0 - 0.12);
    assert!(dense.confidence.0 > dense.tempo_ambiguity.0);
}

#[test]
fn beat_tracker_calibrates_harmonic_rhythm_variant_monotonicity() {
    let (_, sparse) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Sparse,
    ));
    let (_, active) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));

    let active_meter = active.meter.as_ref().expect("active harmonic meter");
    assert!(sparse.meter.is_none());
    assert_eq!(active_meter.beats_per_bar, 4);
    assert!(active.confidence.0 >= sparse.confidence.0 - 0.05);
    assert!(active.tempo_ambiguity.0 <= sparse.tempo_ambiguity.0 + 0.1);
}
