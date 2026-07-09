use super::*;

fn transient_train(length: usize, starts: &[usize]) -> Vec<Sample> {
    let mut samples = vec![0.0; length];
    for &start in starts {
        for offset in 0..48 {
            samples[start + offset] +=
                (1.0 - offset as f32 / 48.0) * if offset % 2 == 0 { 1.0 } else { -1.0 };
        }
    }
    samples
}

#[test]
fn transient_detail_reports_sample_frame_timing_offset() {
    let input = transient_train(8_192, &[1_024, 3_072, 5_120]);
    let output = transient_train(8_192, &[1_031, 3_079, 5_127]);

    let measurement = measure_transient_detail(&input, &output, 1.0, 512, 128);

    assert_eq!(measurement.matched_transients, 3);
    assert!((measurement.mean_signed_timing_offset_frames - 7.0).abs() < 1.0e-6);
    assert!((measurement.mean_absolute_timing_offset_frames - 7.0).abs() < 1.0e-6);
    assert!((measurement.max_absolute_timing_offset_frames - 7.0).abs() < 1.0e-6);
}

#[test]
fn transient_detail_crest_growth_ignores_whole_render_gain() {
    let input = transient_train(8_192, &[1_024, 3_072, 5_120]);
    let output = input.iter().map(|sample| sample * 0.25).collect::<Vec<_>>();

    let measurement = measure_transient_detail(&input, &output, 1.0, 512, 128);

    assert_eq!(measurement.matched_transients, 3);
    assert!(measurement.max_transient_crest_growth_db.abs() < 1.0e-6);
}

#[test]
fn transient_detail_rejects_invalid_ratio() {
    let measurement = measure_transient_detail(&[1.0], &[1.0], 0.0, 512, 128);

    assert_eq!(measurement.matched_transients, 0);
    assert!(measurement.mean_absolute_timing_offset_frames.is_nan());
}

#[test]
fn transient_event_detail_uses_the_supplied_source_event() {
    let input = transient_train(8_192, &[1_024, 3_072]);
    let output = transient_train(8_192, &[1_031, 3_079]);

    let event = measure_transient_event_detail(&input, &output, 1.0, 3_072, 512, 128)
        .expect("valid event detail");

    assert_eq!(event.input_frame, 3_072);
    assert_eq!(event.output_frame, 3_079);
    assert_eq!(event.timing_offset_frames, 7.0);
    assert!(event.crest_growth_db.abs() < 1.0e-6);
}
