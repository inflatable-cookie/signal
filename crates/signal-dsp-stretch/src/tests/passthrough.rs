use super::support::{self, dominant_frequency_hz, sine};
use super::*;

#[test]
fn identity_ratio_is_passthrough() {
    let input = sine(440.0, 48_000.0, 10_000);
    let mut stretcher = PhaseVocoderStretcher::new(1.0);
    assert_eq!(
        stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        input
    );
}

#[test]
fn ratio_clamps_invalid_values_to_identity() {
    let mut stretcher = PhaseVocoderStretcher::new(f64::NAN);
    assert_eq!(stretcher.ratio(), 1.0);
    stretcher.set_ratio(-2.0);
    assert_eq!(stretcher.ratio(), 1.0);
    stretcher.set_ratio(1.5);
    assert_eq!(stretcher.ratio(), 1.5);
}

#[test]
fn stretch_honors_output_length_contract() {
    let input = sine(440.0, 48_000.0, 48_000);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut stretcher = PhaseVocoderStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        assert_eq!(
            output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}"
        );
    }
}
#[test]
fn offline_high_quality_identity_ratio_is_passthrough() {
    let input = sine(330.0, 48_000.0, 8_192);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    assert_eq!(
        stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        input
    );
}

#[test]
fn stretch_preserves_pitch_within_tolerance() {
    let sample_rate = 48_000.0;
    let input = sine(440.0, sample_rate, 48_000);
    for ratio in [0.75, 1.5, 2.0] {
        let mut stretcher = PhaseVocoderStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let frequency = dominant_frequency_hz(&output, sample_rate);
        assert!(
            (frequency - 440.0).abs() < 15.0,
            "ratio {ratio}: dominant frequency {frequency} Hz, expected ~440 Hz"
        );
        assert!(
            support::rms(&output) > 0.3,
            "ratio {ratio}: stretched output lost energy (rms {})",
            support::rms(&output)
        );
    }
}

#[test]
fn sub_window_input_scales_by_linear_fallback() {
    let input: Vec<f32> = (0..100).map(|index| index as f32 / 100.0).collect();
    let mut stretcher = PhaseVocoderStretcher::new(2.0);
    let output = stretcher
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    assert_eq!(output.len(), 200);
    // Monotone ramp stays monotone under linear scaling.
    assert!(output.windows(2).all(|pair| pair[1] >= pair[0] - 1.0e-6));
}
