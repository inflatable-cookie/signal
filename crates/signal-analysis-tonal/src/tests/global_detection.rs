use super::*;

#[test]
fn key_detector_finds_c_major_triad() {
    let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::default());
    let result = detector.analyze(&audio);

    assert_eq!(result.key.unwrap().tonic, Tonic::C);
    assert_eq!(result.key.unwrap().mode, KeyMode::Major);
    assert!(result.confidence.0 > 0.01);
    assert_eq!(result.tuning.source, TuningReferenceSource::Estimated);
    assert!((result.tuning.reference_hz - 440.0).abs() <= 2.0);
    assert_eq!(result.scoring.profile, KeyProfile::Krumhansl);
    assert_eq!(result.scoring.best.unwrap().key.tonic, Tonic::C);
}

#[test]
fn key_detector_finds_a_minor_triad() {
    let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::default());
    let result = detector.analyze(&audio);

    assert_eq!(result.key.unwrap().tonic, Tonic::A);
    assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
    assert!(result.confidence.0 > 0.001);
}

#[test]
fn low_profile_still_detects_key() {
    let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::low());
    let result = detector.analyze(&audio);

    assert_eq!(result.key.unwrap().tonic, Tonic::C);
    assert_eq!(result.key.unwrap().mode, KeyMode::Major);
    assert_eq!(
        detector.config().tuning_reference,
        TuningReferenceMode::Estimate
    );
    assert_eq!(detector.config().tuning_step_cents, 10);
}

#[test]
fn medium_profile_still_detects_key() {
    let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    assert_eq!(result.key.unwrap().tonic, Tonic::A);
    assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
    assert_eq!(detector.config().tuning_step_cents, 5);
}

#[test]
fn pearson_distinguishes_relative_major_minor() {
    let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::default());
    let result = detector.analyze(&audio);

    let key = result.key.unwrap();
    assert_eq!(key.tonic, Tonic::A);
    assert_eq!(key.mode, KeyMode::Minor);

    assert!(
        result.correlations[21] > result.correlations[0],
        "A minor correlation ({}) should exceed C major ({})",
        result.correlations[21],
        result.correlations[0],
    );
}

#[test]
fn b_minor_bass_detected_correctly_at_44100() {
    let audio = tonal_mix(44_100, &[123.47, 293.66, 369.99], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    let key = result.key.unwrap();
    assert_eq!(
        key.tonic,
        Tonic::B,
        "Expected B but got {:?}; chroma = {:?}",
        key.tonic,
        result.chroma,
    );
    assert_eq!(key.mode, KeyMode::Minor);
}

#[test]
fn b_minor_bass_detected_correctly_at_48000() {
    let audio = tonal_mix(48_000, &[123.47, 293.66, 369.99], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    let key = result.key.unwrap();
    assert_eq!(
        key.tonic,
        Tonic::B,
        "Expected B but got {:?}; chroma = {:?}",
        key.tonic,
        result.chroma,
    );
    assert_eq!(key.mode, KeyMode::Minor);
}

#[test]
fn non_native_input_rate_preserves_key_under_frozen_analysis_rate() {
    let native = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
    let non_native = tonal_mix(44_100, &[261.63, 329.63, 392.0], 4.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::default());

    let native_result = detector.analyze(&native);
    let non_native_result = detector.analyze(&non_native);

    assert_eq!(native_result.key, non_native_result.key);
    assert!(
        (native_result.confidence.0 - non_native_result.confidence.0).abs() < 0.1,
        "confidence drifted from {} to {}",
        native_result.confidence.0,
        non_native_result.confidence.0,
    );
}

#[test]
fn detector_estimates_detuned_reference_for_c_major_material() {
    let audio = detuned_tonal_mix(48_000, &[261.63, 329.63, 392.0], 5.0, 432.0);
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    assert_eq!(result.key.unwrap().tonic, Tonic::C);
    assert_eq!(result.key.unwrap().mode, KeyMode::Major);
    assert_eq!(result.tuning.source, TuningReferenceSource::Estimated);
    assert!((result.tuning.reference_hz - 432.0).abs() <= 2.5);
    assert!(result.tuning.cents_offset < -20.0);
    assert!(result.tuning.runner_up.is_some());
    assert!(result.scoring.runner_up.is_some());
}

#[test]
fn fixed_tuning_reference_is_reported_explicitly() {
    let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
    let mut config = KeyDetectorConfig::medium();
    config.tuning_reference = TuningReferenceMode::Fixed(442.0);
    let mut detector = KeyDetector::new(config);
    let result = detector.analyze(&audio);

    assert_eq!(result.tuning.source, TuningReferenceSource::FixedReference);
    assert!((result.tuning.reference_hz - 442.0).abs() < 0.01);
    assert!(result.tuning.confidence.0 >= 1.0);
    assert!((result.tuning.cents_offset - cents_offset_from_standard(442.0)).abs() < 0.01);
}

#[test]
fn tuning_reference_helpers_round_trip_standard_offsets() {
    let offset = -31.766;
    let reference = reference_hz_from_cents(offset);

    assert!((reference - 432.0).abs() < 1.5);
    assert!((cents_offset_from_standard(reference) - offset).abs() < 0.1);
}
