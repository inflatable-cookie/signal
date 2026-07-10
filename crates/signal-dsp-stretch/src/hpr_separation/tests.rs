use super::*;

const SAMPLE_RATE: SampleRate = SampleRate(48_000);

#[test]
fn hpr_separation_reconstructs_source_with_binary_exhaustive_masks() {
    let input = mixed_control(48_000);
    let review = separate_hpr_review_mono(&input, SAMPLE_RATE);
    assert_eq!(review.harmonic.len(), input.len());
    assert_eq!(review.residual.len(), input.len());
    assert_eq!(review.percussive.len(), input.len());
    assert!(review.evidence.masks_partition_exactly);
    assert_eq!(review.evidence.mask_partition_error_bins, 0);
    assert_eq!(review.evidence.long_uncovered_source_samples, 0);
    assert_eq!(review.evidence.short_uncovered_source_samples, 0);
    assert!(review.evidence.reconstruction_peak_error <= 1.0e-5);
    assert!(review.evidence.reconstruction_rms_error <= 1.0e-6);
    assert!(review.evidence.reconstruction_head_error <= 1.0e-5);
    assert!(review.evidence.reconstruction_tail_error <= 1.0e-5);
    assert!(review.evidence.harmonic.all_samples_finite);
    assert!(review.evidence.residual.all_samples_finite);
    assert!(review.evidence.percussive.all_samples_finite);
    eprintln!(
        "hpr reconstruction peak={:.9e} rms={:.9e} head={:.9e} tail={:.9e}",
        review.evidence.reconstruction_peak_error,
        review.evidence.reconstruction_rms_error,
        review.evidence.reconstruction_head_error,
        review.evidence.reconstruction_tail_error,
    );
}

#[test]
fn hpr_separation_is_deterministic() {
    let input = mixed_control(24_000);
    let first = separate_hpr_review_mono(&input, SAMPLE_RATE);
    let repeated = separate_hpr_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
}

#[test]
fn hpr_separation_assigns_steady_bin_sine_to_harmonic() {
    let window = stage_config(48_000.0, LONG_WINDOW_SECONDS).window_size;
    let input = (0..48_000)
        .map(|index| (std::f32::consts::TAU * 211.0 * index as f32 / window as f32).sin() * 0.5)
        .collect::<Vec<_>>();
    let review = separate_hpr_review_mono(&input, SAMPLE_RATE);
    eprintln!(
        "hpr sine harmonic margin={:.6} dB",
        review.evidence.harmonic.dominance_margin_db
    );
    assert_margin(review.evidence.harmonic.dominance_margin_db);
}

#[test]
fn hpr_separation_assigns_isolated_impulse_to_percussive() {
    let mut input = vec![0.0; 48_000];
    input[24_000] = 1.0;
    let review = separate_hpr_review_mono(&input, SAMPLE_RATE);
    eprintln!(
        "hpr impulse percussive margin={:.6} dB",
        review.evidence.percussive.dominance_margin_db
    );
    assert_margin(review.evidence.percussive.dominance_margin_db);
}

#[test]
fn hpr_separation_assigns_stationary_noise_to_residual() {
    let mut state = 0x1234_5678_u32;
    let input = (0..48_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state as f32 / u32::MAX as f32 - 0.5) * 0.5
        })
        .collect::<Vec<_>>();
    let review = separate_hpr_review_mono(&input, SAMPLE_RATE);
    eprintln!(
        "hpr noise residual margin={:.6} dB",
        review.evidence.residual.dominance_margin_db
    );
    assert_margin(review.evidence.residual.dominance_margin_db);
}

#[test]
fn hpr_additive_identity_is_bit_exact() {
    let input = mixed_control(24_000);
    let render = stretch_hpr_additive_review_mono(&input, SAMPLE_RATE, 1.0);

    assert_eq!(render.samples, input);
    assert!(render.component_lengths_match);
    assert!(render.percussive_positions_monotonic);
    assert_eq!(render.percussive_uncovered_output_frames, 0);
    assert!(!render.hidden_component_gain_applied);
}

#[test]
fn hpr_additive_render_is_deterministic_exact_length_and_additive() {
    let input = mixed_control(24_000);
    for ratio in [0.75, 1.25, 1.5] {
        let first = stretch_hpr_additive_review_mono(&input, SAMPLE_RATE, ratio);
        let repeated = stretch_hpr_additive_review_mono(&input, SAMPLE_RATE, ratio);
        let target_len = (input.len() as f64 * ratio).round() as usize;

        assert_eq!(first, repeated);
        assert_eq!(first.samples.len(), target_len);
        assert!(first.component_lengths_match);
        assert!(first.percussive_positions_monotonic);
        assert_eq!(first.percussive_uncovered_output_frames, 0);
        assert!(!first.hidden_component_gain_applied);
        assert!(first.samples.iter().all(|sample| sample.is_finite()));
        for index in 0..target_len {
            assert_eq!(
                first.samples[index],
                first.harmonic[index] + first.residual[index] + first.percussive[index]
            );
        }
    }
}

fn assert_margin(margin_db: f64) {
    assert!(margin_db >= 12.0, "ownership margin was {margin_db:.3} dB");
}

fn mixed_control(len: usize) -> Vec<Sample> {
    let mut input = (0..len)
        .map(|index| (std::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin() * 0.25)
        .collect::<Vec<_>>();
    if len > 2 {
        input[len / 2] += 0.75;
    }
    input
}
