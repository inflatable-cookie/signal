use super::*;

#[test]
fn spectral_shape_tracks_frequency_position() {
    let low = sine_audio(220.0, 2.0, 48_000, 1.0);
    let high = sine_audio(4_000.0, 2.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let low_result = analyzer.analyze(&low);
    let high_result = analyzer.analyze(&high);

    assert!(high_result.spectral_shape.centroid_hz > low_result.spectral_shape.centroid_hz);
    assert!(high_result.spectral_shape.rolloff_95_hz > low_result.spectral_shape.rolloff_95_hz);
}

#[test]
fn centroid_near_1khz_for_sine() {
    let audio = sine_audio(1_000.0, 2.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
    let result = analyzer.analyze(&audio);

    assert!(
        result.spectral_shape.centroid_hz > 800.0 && result.spectral_shape.centroid_hz < 1_200.0,
        "centroid was {}",
        result.spectral_shape.centroid_hz,
    );
}

#[test]
fn noise_is_flatter_than_sine() {
    let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
    let noise = noise_audio(2.0, 48_000, 0.5);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let tone_result = analyzer.analyze(&tone);
    let noise_result = analyzer.analyze(&noise);

    assert!(noise_result.spectral_shape.flatness > tone_result.spectral_shape.flatness);
}

#[test]
fn normalized_mel_profile_is_bounded_and_sums_to_one() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
    let result = analyzer.analyze(&audio);

    let profile = result.spectral_profile.normalized_mel_band_profile;
    let sum = profile.iter().copied().sum::<f32>();
    assert!((sum - 1.0).abs() < 1e-4, "profile sum was {}", sum);
    assert!(profile.iter().all(|value| *value >= 0.0 && *value <= 1.0));
}
