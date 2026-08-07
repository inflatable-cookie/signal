use super::*;

#[test]
fn offline_soft_limiter_matches_the_known_transfer_curve() {
    // Constant 0.9 through threshold 0.5 / knee 0.2: knee_start 0.4,
    // knee_end 0.6, knee_end_output 0.55, saturation_range 0.45 ->
    // transfer(0.9) = 1 - 0.45^2 / (0.5 * 0.3 + 0.45) = 0.6625. Attack
    // is instant, so EVERY frame of a constant over-threshold signal
    // lands exactly on the curve.
    let spec = RenderLimiterSpec {
        threshold: 0.5,
        knee_width: 0.2,
        release_seconds: 0.05,
    };
    let mut samples = vec![0.9f32; 4_800 * 2];
    apply_soft_limiter_to_pcm(&mut samples, 2, 48_000, &spec);
    let expected = 0.6625f32;
    assert!(samples
        .iter()
        .all(|sample| (sample - expected).abs() < 1e-4));
    assert!(samples.iter().all(|sample| *sample <= 1.0));

    // Below the knee start the limiter is bit-transparent (gain 1.0).
    let mut quiet = vec![0.3f32; 480 * 2];
    apply_soft_limiter_to_pcm(&mut quiet, 2, 48_000, &spec);
    assert!(quiet
        .iter()
        .all(|sample| sample.to_bits() == 0.3f32.to_bits()));
}
