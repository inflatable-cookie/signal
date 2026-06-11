fn assert_detected_bpm(
    preset: RhythmPreset,
    result: &super::BeatAnalysisResult,
    expected_bpm: f32,
    tolerance: f32,
) {
    assert!(
        (result.bpm - expected_bpm).abs() < tolerance,
        "preset {:?} detected bpm {} expected {} +/- {}",
        preset,
        result.bpm,
        expected_bpm,
        tolerance
    );
}

fn assert_meter(
    preset: RhythmPreset,
    result: &super::BeatAnalysisResult,
    beats_per_bar: usize,
    min_confidence: f32,
) -> &super::MeterEstimate {
    let meter = result
        .meter
        .as_ref()
        .unwrap_or_else(|| panic!("preset {:?} expected meter estimate", preset));
    assert_eq!(
        meter.beats_per_bar, beats_per_bar,
        "preset {:?} beats_per_bar {}",
        preset, meter.beats_per_bar
    );
    assert!(
        meter.confidence.0 > min_confidence,
        "preset {:?} meter confidence {}",
        preset,
        meter.confidence.0
    );
    meter
}

