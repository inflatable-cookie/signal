use super::*;
use crate::EMBEDDING_DIMENSIONS;

#[test]
fn tonal_audio_prefers_tonal_focus() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let result = embedder.analyze(&audio);

    assert_eq!(top_label(&result), SemanticTagLabel::TonalFocus);
    assert_eq!(
        result.diagnostics.top_tag_label,
        Some(SemanticTagLabel::TonalFocus)
    );
    assert_eq!(result.embedding.len(), EMBEDDING_DIMENSIONS);
    assert!(result.source_descriptors.spectral_shape.flatness < 1e-4);
    assert_eq!(
        result.semantic_tags[0].evidence.primary_driver,
        "harmonic_focus"
    );
}

#[test]
fn noisy_audio_prefers_textural_noise() {
    let audio = noise_audio(2.0, 48_000, 0.5);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let result = embedder.analyze(&audio);

    assert_eq!(top_label(&result), SemanticTagLabel::TexturalNoise);
    assert_eq!(
        result.diagnostics.top_tag_label,
        Some(SemanticTagLabel::TexturalNoise)
    );
    assert!(result.source_descriptors.spectral_shape.spread_hz > 1_000.0);
    assert_eq!(
        result.semantic_tags[0].evidence.primary_driver,
        "spectral_complexity"
    );
}

#[test]
fn pulse_audio_prefers_pulse_driven_or_dynamic_punch() {
    let audio = adsr_pulse_audio(5, 120, 100, 500, 6, 48_000, 0.9);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
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
    assert!(result.semantic_tags[0].evidence.evidence_strength > 0.0);
}

#[test]
fn max_tag_count_limits_ranked_output() {
    let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
        max_tag_count: 2,
        ..SemanticEmbedderConfig::default()
    });
    let result = embedder.analyze(&audio);

    assert_eq!(result.semantic_tags.len(), 2);
}

#[test]
fn semantic_diagnostics_are_bounded() {
    let audio = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let result = embedder.analyze(&audio);

    assert!(result.diagnostics.embedding_l2_norm.is_finite());
    assert!(result.diagnostics.active_embedding_dimensions > 0);
    assert!(result.diagnostics.semantic_confidence.0 >= 0.0);
    assert!(result.diagnostics.semantic_confidence.0 <= 1.0);
    assert!(
        result
            .diagnostics
            .confidence_components
            .top_margin_component
            >= 0.0
    );
    assert!(
        result
            .diagnostics
            .confidence_components
            .top_margin_component
            <= 1.0
    );
    assert!(
        result
            .diagnostics
            .confidence_components
            .embedding_activity_component
            >= 0.0
    );
    assert!(
        result
            .diagnostics
            .confidence_components
            .embedding_activity_component
            <= 1.0
    );
    assert!(
        result
            .diagnostics
            .confidence_components
            .descriptor_confidence_component
            >= 0.0
    );
    assert!(
        result
            .diagnostics
            .confidence_components
            .descriptor_confidence_component
            <= 1.0
    );
}

#[test]
fn frozen_semantic_calibration_report_has_expected_top_tag_and_confidence_posture() {
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let report = semantic_calibration_report(&mut embedder);

    println!("semantic_calibration_report={report:#?}");

    assert_eq!(report.case_reports.len(), 3);

    let tone = &report.case_reports[0];
    assert_eq!(tone.case_id, "semantic:tone:sine440");
    assert_eq!(tone.top_tag, SemanticTagLabel::TonalFocus);
    assert_eq!(tone.primary_driver, "harmonic_focus");
    assert!(tone.top_confidence.0 >= 0.15);
    assert!(tone.top_tag_margin >= 0.05);

    let noise = &report.case_reports[1];
    assert_eq!(noise.case_id, "semantic:noise:deterministic");
    assert_eq!(noise.top_tag, SemanticTagLabel::TexturalNoise);
    assert_eq!(noise.primary_driver, "spectral_complexity");
    assert!(noise.top_confidence.0 >= 0.08);
    assert!(noise.top_tag_margin >= 0.02);

    let pulse = &report.case_reports[2];
    assert_eq!(pulse.case_id, "semantic:pulse:adsr");
    assert!(matches!(
        pulse.top_tag,
        SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
    ));
    assert!(matches!(
        pulse.primary_driver,
        "rhythmic_activity" | "dynamic_punch"
    ));
    assert!(pulse.top_confidence.0 >= 0.08);
    assert!(pulse.top_tag_margin >= 0.0);
}

#[test]
fn frozen_semantic_cases_have_explicit_confidence_ordering() {
    let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
    let noise = noise_audio(2.0, 48_000, 0.5);
    let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);

    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let tone_result = embedder.analyze(&tone);
    let noise_result = embedder.analyze(&noise);
    let pulse_result = embedder.analyze(&pulse);

    assert!(
        tone_result.diagnostics.semantic_confidence.0
            >= noise_result.diagnostics.semantic_confidence.0
    );
    assert!(
        pulse_result.diagnostics.semantic_confidence.0
            >= noise_result.diagnostics.semantic_confidence.0 * 0.8
    );
    assert!(
        tone_result
            .diagnostics
            .confidence_components
            .top_margin_component
            >= noise_result
                .diagnostics
                .confidence_components
                .top_margin_component
    );
}

#[test]
fn harness_semantic_cases_meet_frozen_acceptance_thresholds() {
    let cases = semantic_acceptance_cases();
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());

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
    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());

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

    let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
    let tone_result = embedder.analyze(&tone);
    let noise_result = embedder.analyze(&noise);
    let pulse_result = embedder.analyze(&pulse);

    println!("tone_semantic={:#?}", tone_result);
    println!("noise_semantic={:#?}", noise_result);
    println!("pulse_semantic={:#?}", pulse_result);

    assert_eq!(top_label(&tone_result), SemanticTagLabel::TonalFocus);
    assert_eq!(top_label(&noise_result), SemanticTagLabel::TexturalNoise);
    assert!(matches!(
        top_label(&pulse_result),
        SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
    ));
    assert_eq!(
        tone_result.semantic_tags[0].evidence.primary_driver,
        "harmonic_focus"
    );
    assert_eq!(
        noise_result.semantic_tags[0].evidence.primary_driver,
        "spectral_complexity"
    );
    assert_eq!(
        tone_result.diagnostics.top_tag_label,
        Some(SemanticTagLabel::TonalFocus)
    );
    assert_eq!(
        noise_result.diagnostics.top_tag_label,
        Some(SemanticTagLabel::TexturalNoise)
    );
    assert!(tone_result.diagnostics.semantic_confidence.0 > 0.0);
    assert!(noise_result.diagnostics.semantic_confidence.0 > 0.0);
    assert!(pulse_result.diagnostics.semantic_confidence.0 > 0.0);
}
