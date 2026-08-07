use super::*;

#[test]
fn realtime_preview_mono_is_deterministic_and_pitch_preserving() {
    let input = sine(440.0, 48_000.0, 12_000);
    let mut first = RealtimePreviewStretcher::new(1.25);
    let mut second = RealtimePreviewStretcher::new(1.25);

    let first_output = first
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let second_output = second
        .stretch_mono(&input)
        .expect("render fits the offline output bound");

    assert_eq!(first.quality(), StretchQuality::RealtimePreview);
    assert_eq!(
        first_output.len(),
        (input.len() as f64 * 1.25).round() as usize
    );
    assert_eq!(first_output, second_output);
    assert!((dominant_frequency_hz(&first_output, 48_000.0) - 440.0).abs() < 20.0);
}

#[test]
fn realtime_preview_linked_stereo_is_deterministic_and_exact_length() {
    let left = sine(330.0, 48_000.0, 16_000);
    let right = sine(660.0, 48_000.0, 16_000);
    let input = left
        .iter()
        .zip(right.iter())
        .flat_map(|(left, right)| [*left, *right])
        .collect::<Vec<_>>();
    let mut first = RealtimePreviewStretcher::new(0.75);
    let mut second = RealtimePreviewStretcher::new(0.75);

    let first_output = first
        .stretch_interleaved_stereo(&input)
        .expect("render fits the offline output bound");
    let second_output = second
        .stretch_interleaved_stereo(&input)
        .expect("render fits the offline output bound");

    assert_eq!(
        first_output.len(),
        (16_000.0_f64 * 0.75).round() as usize * 2
    );
    assert_eq!(first_output, second_output);
}

#[test]
fn realtime_preview_dynamic_ratio_curve_keeps_sample_domain_length() {
    let input = sine(220.0, 48_000.0, 16_000);
    let ratio_curve = [
        StretchRatioPoint {
            timeline_frame: 0,
            ratio: 1.0,
        },
        StretchRatioPoint {
            timeline_frame: 8_000,
            ratio: 1.5,
        },
    ];
    let mut stretcher = RealtimePreviewStretcher::new(1.0);

    let output = stretcher
        .stretch_dynamic_ratio_mono(&input, &ratio_curve)
        .expect("render fits the offline output bound");

    assert_eq!(output.len(), 20_000);
}

#[test]
fn realtime_preview_pitch_shift_preserves_tempo_length_contract() {
    let input = sine(440.0, 48_000.0, 12_000);
    let mut stretcher = RealtimePreviewStretcher::new(1.25);

    let output = stretcher
        .stretch_pitch_mono(&input, SampleRate(48_000), 12.0)
        .expect("render fits the offline output bound");

    assert_eq!(output.len(), 15_000);
    assert!((dominant_frequency_hz(&output, 48_000.0) - 880.0).abs() < 35.0);
}
