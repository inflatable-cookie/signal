use super::support::*;
use super::*;

#[test]
fn offline_high_quality_dynamic_ratio_mono_sums_segment_targets() {
    let input = sine(440.0, 48_000.0, 48_000);
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);
    let output = stretcher
        .stretch_dynamic_ratio_mono(&input, &ratio_curve)
        .expect("render fits the offline output bound");

    assert_eq!(
        output.len(),
        dynamic_ratio_output_frames(input.len(), &ratio_curve, 1.0)
    );
    assert_eq!(output.len(), 52_000);
}

#[test]
fn offline_high_quality_dynamic_ratio_ignores_invalid_points() {
    let input = sine(440.0, 48_000.0, 8_000);
    let ratio_curve = [
        StretchRatioPoint::new(-128, 0.5),
        StretchRatioPoint::new(2_000, f64::NAN),
        StretchRatioPoint::new(4_000, -2.0),
    ];
    let mut dynamic = OfflineHighQualityStretcher::new(1.25);
    let mut fixed = OfflineHighQualityStretcher::new(1.25);

    // Invalid points are ignored, so this must render as the stretcher's
    // own static ratio. Compared through the same renderer with an empty
    // curve, because the invariant is about the curve, not about which
    // renderer runs.
    let curved = dynamic
        .stretch_dynamic_ratio_mono(&input, &ratio_curve)
        .expect("render fits the offline output bound");
    let empty_curve = dynamic
        .stretch_dynamic_ratio_mono(&input, &[])
        .expect("render fits the offline output bound");
    assert_eq!(curved, empty_curve);

    // `stretch_dynamic_ratio_mono` renders resumably and `stretch_mono` in
    // one shot, so they are close rather than identical at a static ratio.
    // Recorded as a bound because it is a real consequence of the dynamic
    // API moving to the resumable renderer: same length, same algorithm,
    // different state handling.
    let flat = fixed
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    assert_eq!(curved.len(), flat.len());
    let worst = curved
        .iter()
        .zip(flat.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1.0e-4,
        "resumable and one-shot static renders drifted apart: {worst}",
    );
}

#[test]
fn offline_high_quality_dynamic_ratio_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut first = OfflineHighQualityStretcher::new(1.0);
    let mut repeated = OfflineHighQualityStretcher::new(1.0);
    let first_output = first
        .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
        .expect("render fits the offline output bound");

    assert_eq!(
        first_output.len(),
        dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
    );
    assert_eq!(first_output, repeated_output);
}

#[test]
fn dynamic_segment_seam_smoothing_is_not_neutral_on_continuous_material() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    // Explicit boundaries: this owner tests the smoother, not the
    // segmentation law. Deriving them from a ratio curve coupled it to the
    // Contract `046` minimum segment length, and it broke when that
    // minimum grew past the curve's span length.
    let boundaries = vec![12_000, 28_000];
    let mut raw = frames.clone();
    let before = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);
    smooth_dynamic_segment_boundaries_interleaved(&mut raw, 2, &boundaries, 64);
    let after = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);

    // Continuous material with no join: the smoother has nothing to fix,
    // and drags 64 frames either side of each nominated frame toward the
    // midpoint of the pair it straddles. That is a discontinuity it
    // introduces, not one it removes. Measured -240 dBFS (nothing) before,
    // -70.9 dBFS after.
    assert!(
        before.click_dbfs <= -240.0,
        "clean sines should show no seam, got {:.2} dBFS",
        before.click_dbfs,
    );
    assert!(
        after.click_dbfs > before.click_dbfs + 100.0,
        "smoothing continuous material should introduce a measurable \
         discontinuity, got {:.2} dBFS",
        after.click_dbfs,
    );
    assert_eq!(raw.len(), frames.len());
}

#[test]
fn offline_high_quality_dynamic_ratio_pitch_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut first = OfflineHighQualityStretcher::new(1.0);
    let mut repeated = OfflineHighQualityStretcher::new(1.0);
    let first_output = first
        .stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        )
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        )
        .expect("render fits the offline output bound");

    assert_eq!(
        first_output.len(),
        dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
    );
    assert_eq!(first_output, repeated_output);
}
#[test]
fn dynamic_segment_seam_metric_reports_excess_over_the_renders_own_floor() {
    // Too short to hold any frame outside a seam window: there is no way to
    // tell a seam from the waveform, so the answer is "unmeasurable", not
    // "clean". The predecessor of this measurement answered "clean".
    let tiny = [0.0, 0.0, 0.1, 0.2, 0.9, -0.4, 1.0, -0.3];
    assert!(measure_dynamic_segment_seam_click(&tiny, 2, &[2], 1.0)
        .click_dbfs
        .is_nan());

    // A long, smooth ramp with one injected step. The step is 0.5 against a
    // per-frame background of 0.0001, so it must read close to 0.5 rather
    // than to the raw first difference.
    let frame_count = 8_000usize;
    let mut frames = Vec::with_capacity(frame_count * 2);
    for index in 0..frame_count {
        let value = index as f32 * 0.0001;
        frames.push(value);
        frames.push(value);
    }
    for sample in frames[4_000 * 2..].iter_mut() {
        *sample += 0.5;
    }
    let measurement = measure_dynamic_segment_seam_click(&frames, 2, &[4_000], 1.0);
    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.channels, 2);
    assert_eq!(measurement.seam_frames, vec![4_000]);
    assert!(
        (measurement.peak_seam_delta - 0.5).abs() < 1.0e-3,
        "expected the injected step less the floor, got {}",
        measurement.peak_seam_delta,
    );
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::DynamicSegmentSeamClickDbfs
    );
    assert_eq!(measurement.metric.value, measurement.click_dbfs);

    // And it stays visible through the smoother, which is the whole point:
    // the smoother sets the straddling pair equal, so a measurement that
    // read only that pair scored this -240 dBFS, the silence sentinel.
    // A linear ramp is the smoother's best case -- it really does spread
    // the 0.5 step over its 256-frame fade -- and even here the residue
    // reads -60.2 dBFS rather than silence.
    let mut smoothed = frames.clone();
    smooth_dynamic_segment_boundaries_interleaved(&mut smoothed, 2, &[4_000], 256);
    let after = measure_dynamic_segment_seam_click(&smoothed, 2, &[4_000], 1.0);
    assert!(
        after.click_dbfs > -120.0,
        "the smoother must not be able to hide the step, got {:.2} dBFS",
        after.click_dbfs,
    );
}
#[test]
fn dynamic_segment_boundary_smoothing_equalizes_join_edges() {
    let mut frames = [0.0, 0.0, 1.0, -1.0, -1.0, 1.0, 0.0, 0.0];

    smooth_dynamic_segment_boundaries_interleaved(&mut frames, 2, &[2], 1);

    assert!((frames[2] - frames[4]).abs() < 1.0e-6);
    assert!((frames[3] - frames[5]).abs() < 1.0e-6);
    assert_eq!(frames[0], 0.0);
    assert_eq!(frames[1], 0.0);
    assert_eq!(frames[6], 0.0);
    assert_eq!(frames[7], 0.0);
}
