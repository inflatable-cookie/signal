use super::*;

fn stable_sine(frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|index| (std::f32::consts::TAU * 17.0 * index as f32 / 1_024.0).sin() * 0.5)
        .collect()
}

#[test]
fn hybrid_render_identity_returns_current_samples() {
    let input = stable_sine(8_192);
    let render = build_hybrid_render(&input, &input, 1.0);

    assert_eq!(render.samples, input);
    assert_eq!(render.applied_span_count, 0);
    assert!(render.transition_decisions.is_empty());
}

#[test]
fn hybrid_render_is_deterministic_and_exact_length() {
    let input = stable_sine(8_192);
    let mixed = transient_reset_phase_vocoder(&input, 12_288, 1.5, 2_048, 512);

    let first = build_hybrid_render(&input, &mixed, 1.5);
    let repeated = build_hybrid_render(&input, &mixed, 1.5);

    assert_eq!(first, repeated);
    assert_eq!(first.samples.len(), mixed.len());
    assert!(first.applied_span_count + first.rejected_span_count > 0);
}

#[test]
fn anti_correlated_transition_is_rejected() {
    let outgoing = stable_sine(256);
    let incoming = outgoing.iter().map(|sample| -*sample).collect::<Vec<_>>();
    let evaluation = evaluate_transition(&outgoing, &incoming, (0, 256));

    assert_eq!(
        evaluation.rejection,
        Some(StretchHybridTransitionRejection::LowCorrelation)
    );
}

#[test]
fn normalization_bound_rejects_marginal_correlation() {
    assert!(max_normalization_gain_db(0.50) > MAX_NORMALIZATION_GAIN_DB);
    assert!(max_normalization_gain_db(0.70) < MAX_NORMALIZATION_GAIN_DB);
}

#[test]
fn bounded_lag_review_recovers_a_known_local_shift() {
    let outgoing = (0..512)
        .map(|index| {
            let value = (index * 73 + index * index * 19 + 11) % 257;
            value as f32 / 128.0 - 1.0
        })
        .collect::<Vec<_>>();
    let mut incoming = vec![0.0; outgoing.len()];
    incoming[7..].copy_from_slice(&outgoing[..outgoing.len() - 7]);

    let evaluation = evaluate_transition(&outgoing, &incoming, (128, 384));

    assert_eq!(evaluation.best_lag_frames, 7);
    assert!(evaluation.best_lag_correlation > 0.999_999);
}

#[test]
fn accepted_transition_preserves_identical_samples() {
    let samples = stable_sine(256);
    let evaluation = evaluate_transition(&samples, &samples, (0, 256));
    let mut output = vec![0.0; 256];
    apply_transition(
        &mut output,
        &samples,
        &samples,
        (0, 256),
        evaluation.correlation,
    );

    assert_eq!(evaluation.rejection, None);
    assert_eq!(output, samples);
}
