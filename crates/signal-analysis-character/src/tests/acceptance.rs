use super::*;

#[test]
fn harness_character_descriptor_cases_meet_frozen_acceptance_thresholds() {
    let cases = character_acceptance_cases();
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let report =
        run_audio_acceptance_harness(&cases, |audio| analyzer.analyze(audio), character_metrics);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert!(report
        .cases
        .iter()
        .all(|case| case.status == AcceptanceStatus::Pass));
}

#[test]
fn frozen_character_acceptance_report_remains_interpretable_for_closeout() {
    let cases = character_acceptance_cases();
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let report =
        run_audio_acceptance_harness(&cases, |audio| analyzer.analyze(audio), character_metrics);

    println!("character_acceptance_report={:#?}", report);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert_eq!(report.cases.len(), 3);
}

#[test]
fn descriptor_pack_examples_remain_interpretable_for_closeout() {
    let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
    let noise = noise_audio(2.0, 48_000, 0.5);
    let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
    let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

    let tone_result = analyzer.analyze(&tone);
    let noise_result = analyzer.analyze(&noise);
    let pulse_result = analyzer.analyze(&pulse);

    println!("tone_result={:#?}", tone_result);
    println!("noise_result={:#?}", noise_result);
    println!("pulse_result={:#?}", pulse_result);

    assert!(tone_result.spectral_shape.flatness < noise_result.spectral_shape.flatness);
    assert!(noise_result.spectral_shape.spread_hz > tone_result.spectral_shape.spread_hz);
    assert!(noise_result.spectral_contrast.contrast_db < tone_result.spectral_contrast.contrast_db);
    assert!(
        pulse_result.temporal_shape.peak_transient_strength
            > tone_result.temporal_shape.peak_transient_strength
    );
    assert!(pulse_result.temporal.onset_density > tone_result.temporal.onset_density);
    assert!(pulse_result.temporal_shape.sustain_plateau_ratio > 0.0);
    assert!(pulse_result.temporal_shape.decay_time_ms > pulse_result.temporal_shape.attack_time_ms);
}
