use super::support::*;
use super::*;

#[test]
fn transient_detector_finds_synthetic_attack_frames() {
    let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
        .expect("extreme-ratio synthetic audio exists");
    let events = detect_stretch_transients(&audio.samples, 1024, 256);

    assert!(
        events.len() >= 10,
        "expected repeated synthetic attacks, got {events:?}"
    );
    for expected in [8_000usize, 16_000, 24_000, 32_000, 40_000] {
        assert!(
            events
                .iter()
                .any(|event| event.frame_index.abs_diff(expected) <= 768),
            "missing transient near frame {expected}, got {events:?}"
        );
    }
    assert!(events.iter().all(|event| event.energy_score.is_finite()
        && event.spectral_flux_score.is_finite()
        && event.combined_score.is_finite()));
}

#[test]
fn transient_detector_default_policy_matches_production_entry_point() {
    let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
        .expect("extreme-ratio synthetic audio exists");

    assert_eq!(
        detect_stretch_transients(&audio.samples, 1024, 256),
        detect_stretch_transients_with_policy(
            &audio.samples,
            1024,
            256,
            StretchTransientDetectorPolicy::production()
        )
    );
}

#[test]
fn candidate_transient_detector_recovers_masked_soft_attack() {
    let input = masked_soft_attack_probe(0.25);
    let production = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::production(),
    );
    let candidate = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::candidate_review(),
    );

    assert!(
        production
            .iter()
            .all(|event| event.frame_index.abs_diff(24_000) > 768),
        "production policy should miss the softened probe attack: {production:?}"
    );
    assert!(
        candidate
            .iter()
            .any(|event| event.frame_index.abs_diff(24_000) <= 768),
        "candidate policy should recover the softened probe attack: {candidate:?}"
    );
}

#[test]
fn transient_detector_stays_quiet_on_plain_sustain() {
    let input = sine(440.0, 48_000.0, 48_000);
    let events = detect_stretch_transients(&input, 1024, 256);

    assert!(
        events.len() <= 1,
        "plain sustain should not generate repeated transient events: {events:?}"
    );
}

#[test]
fn candidate_transient_detector_stays_quiet_on_plain_sustain() {
    let input = sine(440.0, 48_000.0, 48_000);
    let events = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::candidate_review(),
    );

    assert!(
        events.len() <= 1,
        "candidate policy should not generate repeated sustain events: {events:?}"
    );
}

#[test]
fn transient_smear_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_transient_smear(1.5);

    assert_eq!(measurement.ratio, 1.5);
    assert!(measurement.input_transients >= 10);
    assert!(measurement.output_transients > 0);
    assert!(measurement.matched_transients > 0);
    assert_eq!(
        measurement.input_transients,
        measurement.matched_transients + measurement.missed_transients
    );
    assert!(measurement.mean_smear_frames.is_finite());
    assert!(measurement.max_smear_frames.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::TransientSmearFrames
    );
    assert_eq!(measurement.metric.value, measurement.max_smear_frames);
}

#[test]
fn transient_reset_smear_metric_reports_synthetic_case() {
    let draft = measure_draft_transient_smear(1.5);
    let reset = measure_transient_reset_transient_smear(1.5);

    assert_eq!(reset.ratio, 1.5);
    assert_eq!(reset.input_transients, draft.input_transients);
    assert!(reset.output_transients > 0);
    assert!(reset.matched_transients > 0);
    assert_eq!(
        reset.input_transients,
        reset.matched_transients + reset.missed_transients
    );
    assert!(reset.max_smear_frames.is_finite());
    assert_eq!(reset.metric.metric, StretchMetric::TransientSmearFrames);
}

#[test]
fn transient_smear_metric_penalizes_missing_matches() {
    let mut input = vec![0.0; 64];
    input[20] = 1.0;
    input[21] = 0.5;
    input[22] = 0.25;
    let output = vec![0.0; 64];
    let measurement = measure_transient_smear(
        &input,
        &output,
        1.0,
        16,
        4,
        StretchTransientSmearPolicies::production(),
    );

    assert!(measurement.input_transients > 0);
    assert_eq!(measurement.output_transients, 0);
    assert_eq!(measurement.matched_transients, 0);
    assert_eq!(measurement.missed_transients, measurement.input_transients);
    assert_eq!(measurement.mean_smear_frames, 16.0);
    assert_eq!(measurement.max_smear_frames, 16.0);
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::TransientSmearFrames
    );
    assert_eq!(measurement.metric.value, 16.0);
}

#[test]
fn transient_smear_entry_point_uses_promoted_output_recovery_policy() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let promoted = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::production(),
    );
    let strict = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
    );
    let recovery = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
        },
    );

    assert_eq!(promoted, recovery);
    assert!(promoted.matched_transients > strict.matched_transients);
    assert!(promoted.missed_transients < strict.missed_transients);
}

#[test]
fn candidate_transient_smear_counts_masked_soft_attack() {
    let input = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &input,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
    );
    let candidate = measure_transient_smear(
        &input,
        &input,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::candidate_review()),
    );

    assert!(candidate.input_transients > production.input_transients);
    assert!(candidate.matched_transients > production.matched_transients);
    assert_eq!(candidate.missed_transients, 0);
    assert_eq!(candidate.max_smear_frames, 0.0);
}

#[test]
fn candidate_output_policy_recovers_production_input_match() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: None,
        },
    );
    let candidate_output = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::candidate_review(),
            output_recovery: None,
        },
    );

    assert_eq!(
        candidate_output.input_transients,
        production.input_transients
    );
    assert!(candidate_output.matched_transients > production.matched_transients);
    assert!(candidate_output.missed_transients < production.missed_transients);
}

#[test]
fn output_recovery_policy_keeps_primary_matches_before_candidate_recovery() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: None,
        },
    );
    let recovery = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
        },
    );

    assert_eq!(recovery.input_transients, production.input_transients);
    assert_eq!(recovery.output_transients, production.output_transients);
    assert!(recovery.matched_transients > production.matched_transients);
    assert!(recovery.missed_transients < production.missed_transients);
    assert!(recovery.max_smear_frames <= production.max_smear_frames);
}

#[test]
fn transient_smear_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_transient_smear(1.25);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::TransientSmearFrames,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:extreme_ratio", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=TransientSmearFrames"));
    assert!(formatted.contains("status=Pass"));
}
