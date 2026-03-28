use super::*;

#[test]
fn rms_energy_near_expected_for_full_scale_sine() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.dynamics.rms_energy > 0.6 && result.dynamics.rms_energy < 0.8,
        "rms was {}",
        result.dynamics.rms_energy,
    );
}

#[test]
fn silence_produces_zero_results() {
    let audio =
        AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![0.0; 48_000]);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert_eq!(result.spectral_shape, SpectralShapeDescriptorPack::zero());
    assert_eq!(
        result.spectral_contrast,
        SpectralContrastDescriptorPack::zero()
    );
    assert_eq!(
        result.spectral_profile,
        SpectralProfileDescriptorPack::zero()
    );
    assert_eq!(result.temporal, TemporalDescriptorPack::zero());
    assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
    assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
}

#[test]
fn empty_audio_yields_zero_confidence() {
    let audio = AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, Vec::new());
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert_eq!(result.confidence, Confidence::new(0.0));
    assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
    assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
}

#[test]
fn zcr_near_expected_for_440hz_sine() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.temporal.zero_crossing_rate_hz > 800.0
            && result.temporal.zero_crossing_rate_hz < 920.0,
        "zcr was {}",
        result.temporal.zero_crossing_rate_hz,
    );
}

#[test]
fn onset_density_is_finite() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
    let result = analyzer.analyze(&audio);

    assert!(result.temporal.onset_density.is_finite());
}

#[test]
fn analysis_stage_trait_works() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = <CharacterAnalyzer as AnalysisStage<CharacterAnalysisResult>>::analyze(
        &mut analyzer,
        &audio,
    );

    assert!(result.dynamics.rms_energy > 0.0);
    assert_eq!(analyzer.mode(), signal_analysis::AnalysisMode::Offline);
}

#[test]
fn low_profile_still_produces_results() {
    let audio = sine_audio(440.0, 4.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::low());
    let result = analyzer.analyze(&audio);

    assert!(result.spectral_shape.centroid_hz > 0.0);
    assert!(result.dynamics.rms_energy > 0.0);
}

#[test]
fn peak_amplitude_for_full_scale_sine() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.dynamics.peak_amplitude > 0.95 && result.dynamics.peak_amplitude <= 1.0,
        "peak was {}",
        result.dynamics.peak_amplitude,
    );
}

#[test]
fn peak_amplitude_for_half_scale_sine() {
    let audio = sine_audio(440.0, 1.0, 48_000, 0.5);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.dynamics.peak_amplitude > 0.45 && result.dynamics.peak_amplitude < 0.55,
        "peak was {}",
        result.dynamics.peak_amplitude,
    );
}

#[test]
fn dynamic_range_is_peak_minus_rms() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    let expected = result.dynamics.peak_amplitude - result.dynamics.rms_energy;
    assert!(
        (result.dynamics.dynamic_range - expected).abs() < 1e-6,
        "dynamic_range {} != peak {} - rms {}",
        result.dynamics.dynamic_range,
        result.dynamics.peak_amplitude,
        result.dynamics.rms_energy,
    );
}

#[test]
fn sustain_ratio_near_one_for_loud_signal() {
    let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.temporal.sustain_ratio > 0.95,
        "sustain_ratio was {}",
        result.temporal.sustain_ratio,
    );
}

#[test]
fn sustain_ratio_near_zero_for_very_quiet_signal() {
    let audio =
        AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![0.001; 48_000]);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert_eq!(result.temporal.sustain_ratio, 0.0);
}

#[test]
fn transient_density_is_finite_and_non_negative() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(result.temporal.transient_density.is_finite());
    assert!(result.temporal.transient_density >= 0.0);
}

#[test]
fn transient_density_increases_with_sharp_edges() {
    let sample_rate_hz = 48_000;
    let duration_seconds = 2.0;
    let count = (sample_rate_hz as f32 * duration_seconds) as usize;
    let mut data = vec![0.0f32; count];
    let spacing = 4_800;
    for index in (0..count).step_by(spacing) {
        if index + 1 < count {
            data[index + 1] = 0.5;
        }
    }

    let audio =
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let result = analyzer.analyze(&audio);

    assert!(
        result.temporal.transient_density > 1.0,
        "transient_density was {}",
        result.temporal.transient_density,
    );
}
