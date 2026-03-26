use super::*;

#[test]
fn beat_tracker_calibrates_meter_confidence_between_steady_and_dropout_sections() {
    let sample_rate = 48_000;
    let bpm = 120.0;

    let mut steady_fixture = FixtureBuilder::new();
    steady_fixture.push_four_four_section(GrooveSection {
        bars: 6,
        beat_pattern: [0.5, 0.26, 0.38, 0.24],
        chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
        chord_every_bars: 1,
        section_marker: None,
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });
    let steady = analyze_fixture(&steady_fixture.build(sample_rate, bpm));

    let mut dropout_fixture = FixtureBuilder::new();
    dropout_fixture.push_four_four_section(GrooveSection {
        bars: 6,
        beat_pattern: [0.5, 0.26, 0.38, 0.24],
        chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
        chord_every_bars: 1,
        section_marker: Some((8, CHORD_D, 0.75)),
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[2],
    });
    let dropout = analyze_fixture(&dropout_fixture.build(sample_rate, bpm));

    let steady_meter = steady.meter.as_ref().expect("steady meter");
    let dropout_meter = dropout.meter.as_ref().expect("dropout meter");
    assert_eq!(steady_meter.beats_per_bar, 4);
    assert_eq!(dropout_meter.beats_per_bar, 4);
    assert!(steady_meter.confidence.0 > dropout_meter.confidence.0);
}

#[test]
fn beat_tracker_handles_fill_bar_with_harmonic_rhythm_changes() {
    let preset = RhythmPreset::FillTransition124(FillDensityVariant::Medium);
    let (bpm, result) = analyze_preset(preset);
    let meter = assert_meter(preset, &result, 4, 0.18);

    assert_detected_bpm(preset, &result, bpm, 3.0);
    assert!(result.confidence.0 > result.tempo_ambiguity.0);
    assert!(meter.downbeat_positions_seconds.len() >= 2);
}

#[test]
fn beat_tracker_prefers_unknown_meter_for_dropout_heavy_transition_fixture() {
    let preset = RhythmPreset::Dropout120(DropoutVariant::Heavy);
    let (bpm, result) = analyze_preset(preset);
    assert_detected_bpm(preset, &result, bpm, 3.0);
    assert!(result.meter.is_none());
}
