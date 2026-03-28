use super::*;

#[test]
fn tonal_audio_prefers_tonal_focus() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let result = embedder.analyze(&audio);

    assert_eq!(top_label(&result), SemanticTagLabel::TonalFocus);
    assert_eq!(result.embedding.values.len(), EMBEDDING_DIMENSIONS);
    assert!(result.source_descriptors.spectral_shape.flatness < 1e-4);
}

#[test]
fn noisy_audio_prefers_textural_noise() {
    let audio = noise_audio(2.0, 48_000, 0.5);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let result = embedder.analyze(&audio);

    assert_eq!(top_label(&result), SemanticTagLabel::TexturalNoise);
    assert!(result.source_descriptors.spectral_shape.spread_hz > 1_000.0);
}

#[test]
fn pulse_audio_prefers_pulse_driven_or_dynamic_punch() {
    let audio = adsr_pulse_audio(5, 120, 100, 500, 6, 48_000, 0.9);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let result = embedder.analyze(&audio);

    assert!(matches!(
        top_label(&result),
        SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
    ));
    assert!(
        result
            .source_descriptors
            .temporal_shape
            .peak_transient_strength
            > 0.9
    );
    assert!(result.diagnostics.top_tag_margin >= 0.0);
}

#[test]
fn max_tag_count_limits_ranked_output() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
        max_tag_count: 2,
        ..SemanticEmbedderConfig::default()
    })
    .unwrap();
    let result = embedder.analyze(&audio);

    assert_eq!(result.semantic_tags.len(), 2);
}

#[test]
fn semantic_diagnostics_are_bounded() {
    let audio = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let result = embedder.analyze(&audio);

    assert!(result.diagnostics.embedding_l2_norm.is_finite());
    assert!(result.diagnostics.active_embedding_dimensions > 0);
    assert!(result.diagnostics.semantic_confidence.0 >= 0.0);
    assert!(result.diagnostics.semantic_confidence.0 <= 1.0);
}

#[test]
fn harness_semantic_cases_meet_frozen_acceptance_thresholds() {
    let cases = semantic_acceptance_cases();
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();

    let report =
        run_audio_acceptance_harness(&cases, |audio| embedder.analyze(audio), semantic_metrics);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert!(report
        .cases
        .iter()
        .all(|case| case.status == AcceptanceStatus::Pass));
}

#[test]
fn frozen_semantic_acceptance_report_remains_interpretable_for_closeout() {
    let cases = semantic_acceptance_cases();
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();

    let report =
        run_audio_acceptance_harness(&cases, |audio| embedder.analyze(audio), semantic_metrics);

    println!("semantic_acceptance_report={:#?}", report);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert_eq!(report.cases.len(), 3);
}

#[test]
fn semantic_examples_remain_interpretable_for_closeout() {
    let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
    let noise = noise_audio(2.0, 48_000, 0.5);
    let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);

    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let tone_result = embedder.analyze(&tone);
    let noise_result = embedder.analyze(&noise);
    let pulse_result = embedder.analyze(&pulse);

    let mut fallback_embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
        requested_model_id: Some("signal:missing-model".to_string()),
        fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
        ..SemanticEmbedderConfig::default()
    })
    .unwrap();
    let fallback_result = fallback_embedder.analyze(&tone);

    println!("tone_semantic={:#?}", tone_result);
    println!("noise_semantic={:#?}", noise_result);
    println!("pulse_semantic={:#?}", pulse_result);
    println!("fallback_semantic={:#?}", fallback_result);

    assert_eq!(top_label(&tone_result), SemanticTagLabel::TonalFocus);
    assert_eq!(top_label(&noise_result), SemanticTagLabel::TexturalNoise);
    assert!(matches!(
        top_label(&pulse_result),
        SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
    ));
    assert!(fallback_result.diagnostics.fallback_used);
    assert_eq!(
        fallback_result.embedding.model_id,
        BUILTIN_DESCRIPTOR_MODEL_ID
    );
    assert!(tone_result.diagnostics.semantic_confidence.0 > 0.0);
    assert!(noise_result.diagnostics.semantic_confidence.0 > 0.0);
    assert!(pulse_result.diagnostics.semantic_confidence.0 > 0.0);
}
