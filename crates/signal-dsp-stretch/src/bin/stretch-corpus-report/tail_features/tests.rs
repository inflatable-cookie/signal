use super::*;

fn sine(frequency_hz: f32, frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|frame| (std::f32::consts::TAU * frequency_hz * frame as f32 / 48_000.0).sin() * 0.5)
        .collect()
}

#[test]
fn low_tone_has_more_low_band_energy_and_lower_centroid() {
    let low = sine(100.0, TAIL_FEATURE_WINDOW_FRAMES);
    let high = sine(4_000.0, TAIL_FEATURE_WINDOW_FRAMES);
    let low_features = measure_tail_local_features(48_000, &low, &low, &low);
    let high_features = measure_tail_local_features(48_000, &high, &high, &high);

    assert!(low_features.low_band_energy_share > 0.9);
    assert!(high_features.low_band_energy_share < 0.01);
    assert!(low_features.spectral_centroid_hz < high_features.spectral_centroid_hz);
}

#[test]
fn spectral_movement_distinguishes_stable_and_changing_tail() {
    let stable = sine(440.0, TAIL_FEATURE_WINDOW_FRAMES);
    let mut changing = sine(440.0, MOVEMENT_WINDOW_FRAMES);
    changing.extend(sine(4_000.0, MOVEMENT_WINDOW_FRAMES));
    let stable_features = measure_tail_local_features(48_000, &stable, &stable, &stable);
    let changing_features = measure_tail_local_features(48_000, &changing, &changing, &changing);

    assert!(stable_features.short_spectral_movement < 0.05);
    assert!(changing_features.short_spectral_movement > 0.9);
}

#[test]
fn zero_crossing_and_correction_energy_are_endpoint_local() {
    let mut current = vec![0.25; TAIL_FEATURE_WINDOW_FRAMES];
    current[TAIL_FEATURE_WINDOW_FRAMES - 5] = -0.25;
    let mut corrected = current.clone();
    *corrected.last_mut().expect("last sample") = 0.0;
    let features = measure_tail_local_features(48_000, &current, &current, &corrected);

    assert_eq!(features.zero_crossing_distance_frames, 4);
    assert_eq!(features.additive_correction_energy_ratio, 0.0);
    assert!(features.multiplicative_correction_energy_ratio > 0.0);
}

#[test]
fn formatted_row_exposes_every_selector_feature() {
    let current = sine(440.0, TAIL_FEATURE_WINDOW_FRAMES);
    let row = format_tail_local_feature_line(
        "stretch:test",
        "source.wav",
        1.25,
        48_000,
        &current,
        &current,
        &current,
    );

    assert!(row.starts_with("external_benchmark_tail_local_features "));
    assert!(row.contains("dc_offset_ratio="));
    assert!(row.contains("low_band_energy_share="));
    assert!(row.contains("spectral_centroid_hz="));
    assert!(row.contains("short_spectral_movement="));
    assert!(row.contains("zero_crossing_distance_frames="));
    assert!(row.contains("additive_correction_energy_ratio="));
    assert!(row.contains("multiplicative_correction_energy_ratio="));
}
