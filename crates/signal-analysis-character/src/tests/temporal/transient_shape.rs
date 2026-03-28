use super::*;

#[test]
fn transient_shape_strength_is_higher_for_pulses_than_steady_tone() {
    let pulse = adsr_pulse_audio(5, 10, 10, 350, 6, 48_000, 0.9);
    let tone = sine_audio(440.0, 2.2, 48_000, 0.9);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let pulse_result = analyzer.analyze(&pulse);
    let tone_result = analyzer.analyze(&tone);

    assert!(
        pulse_result.temporal_shape.peak_transient_strength
            > tone_result.temporal_shape.peak_transient_strength
    );
    assert!(
        pulse_result.temporal_shape.median_transient_strength
            >= tone_result.temporal_shape.median_transient_strength
    );
}

#[test]
fn temporal_shape_attack_time_tracks_slower_attacks() {
    let sharp = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
    let slow = adsr_pulse_audio(80, 10, 10, 500, 6, 48_000, 0.9);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let sharp_result = analyzer.analyze(&sharp);
    let slow_result = analyzer.analyze(&slow);

    assert!(
        slow_result.temporal_shape.attack_time_ms > sharp_result.temporal_shape.attack_time_ms,
        "slow attack {} ms was not greater than sharp attack {} ms",
        slow_result.temporal_shape.attack_time_ms,
        sharp_result.temporal_shape.attack_time_ms,
    );
}

#[test]
fn temporal_shape_decay_time_tracks_longer_decays() {
    let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
    let long = adsr_pulse_audio(5, 10, 120, 500, 6, 48_000, 0.9);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let short_result = analyzer.analyze(&short);
    let long_result = analyzer.analyze(&long);

    assert!(long_result.temporal_shape.decay_time_ms > short_result.temporal_shape.decay_time_ms);
}

#[test]
fn temporal_shape_sustain_ratio_tracks_longer_plateaus() {
    let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
    let long = adsr_pulse_audio(5, 140, 10, 500, 6, 48_000, 0.9);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let short_result = analyzer.analyze(&short);
    let long_result = analyzer.analyze(&long);

    assert!(
        long_result.temporal_shape.sustain_plateau_ratio
            > short_result.temporal_shape.sustain_plateau_ratio
    );
}

#[test]
fn reduction_policy_is_frozen_to_expected_modes() {
    let policy = CharacterDescriptorReductionPolicy::default();

    assert_eq!(
        policy.spectral_centroid_hz,
        DescriptorReduction::MedianAcrossFrames
    );
    assert_eq!(
        policy.normalized_mel_band_profile,
        DescriptorReduction::MeanAcrossFramesNormalized
    );
    assert_eq!(policy.rms_energy, DescriptorReduction::WholeSignal);
    assert_eq!(
        policy.peak_transient_strength,
        DescriptorReduction::PeakAcrossEvents
    );
    assert_eq!(
        policy.attack_time_ms,
        DescriptorReduction::MedianAcrossEvents
    );
}

#[test]
fn non_native_input_rate_preserves_descriptor_shape_under_frozen_analysis_rate() {
    let native = sine_audio(1_000.0, 2.0, 48_000, 1.0);
    let non_native = sine_audio(1_000.0, 2.0, 44_100, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let native_result = analyzer.analyze(&native);
    let non_native_result = analyzer.analyze(&non_native);

    assert!(
        (native_result.spectral_shape.centroid_hz - non_native_result.spectral_shape.centroid_hz)
            .abs()
            < 80.0,
        "centroid drifted from {} to {}",
        native_result.spectral_shape.centroid_hz,
        non_native_result.spectral_shape.centroid_hz,
    );
    assert!((native_result.dynamics.rms_energy - non_native_result.dynamics.rms_energy).abs() < 0.05);
    assert!(
        (native_result.temporal.zero_crossing_rate_hz - non_native_result.temporal.zero_crossing_rate_hz)
            .abs()
            < 25.0
    );
}
