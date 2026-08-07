use super::support::{self, dominant_frequency_hz, sine};
use super::*;

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
fn offline_high_quality_pitch_shift_preserves_tempo_length_contract() {
    let input = sine(440.0, 48_000.0, 48_000);
    for (ratio, semitones) in [(1.0, 12.0), (1.5, -7.0), (0.75, 5.0)] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_pitch_mono(&input, SampleRate(48_000), semitones)
            .expect("render fits the offline output bound");

        assert_eq!(
            output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}, semitones {semitones}"
        );
    }
}

#[test]
fn offline_high_quality_pitch_shift_raises_tonal_pitch() {
    let sample_rate = 48_000.0;
    let input = sine(440.0, sample_rate, 48_000);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    let output = stretcher
        .stretch_pitch_mono(&input, SampleRate(48_000), 12.0)
        .expect("render fits the offline output bound");
    let frequency = dominant_frequency_hz(&output, sample_rate);

    assert_eq!(output.len(), input.len());
    assert!(
        (frequency - 880.0).abs() < 35.0,
        "expected pitch near 880 Hz, got {frequency} Hz"
    );
}

#[test]
fn offline_high_quality_pitch_shift_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    let mut first = OfflineHighQualityStretcher::new(1.25);
    let mut repeated = OfflineHighQualityStretcher::new(1.25);
    let first_output = first
        .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
        .expect("render fits the offline output bound");

    assert_eq!(first_output.len(), (48_000f64 * 1.25).round() as usize * 2);
    assert_eq!(first_output, repeated_output);
}
#[test]
fn pitch_shift_metric_reports_dominant_frequency_error() {
    let sample_rate_hz = 48_000;
    let sample_rate = sample_rate_hz as f32;
    let input = sine(440.0, sample_rate, sample_rate_hz as usize);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);
    let output = stretcher
        .stretch_pitch_mono(&input, SampleRate(sample_rate_hz), 12.0)
        .expect("render fits the offline output bound");
    let measurement = measure_pitch_shift_error_cents(&output, sample_rate_hz, 440.0, 12.0, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.pitch_shift_semitones, 12.0);
    assert!((measurement.expected_frequency_hz - 880.0).abs() < 1.0e-6);
    assert!(measurement.measured_frequency_hz > 850.0);
    assert!(measurement.measured_frequency_hz < 910.0);
    assert!(measurement.pitch_error_cents < 75.0);
    assert_eq!(measurement.metric.metric, StretchMetric::PitchErrorCents);
    assert_eq!(measurement.metric.value, measurement.pitch_error_cents);
}
