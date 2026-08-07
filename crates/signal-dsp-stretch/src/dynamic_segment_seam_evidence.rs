//! Dynamic-segment seam evidence tests.

use super::*;
use crate::benchmark::measure_dynamic_segment_seam_click;

/// The independently-rendered segment path leaves a seam at a hard ratio
/// change; the resumable renderer does not.
///
/// The corpus case `stretch:tempo_ramp` changes ratio by `8%` and shows no
/// seam on either path, so it cannot support this claim. This curve steps
/// `1.6 -> 0.8` across a sustained `110 Hz` tone, which is where a phase
/// vocoder restart is audible, and it is the case the fixed
/// `DynamicSegmentSeamClickDbfs` measurement was built against.
#[test]
fn resumable_dynamic_ratio_has_no_seam_where_segmented_rendering_does() {
    let frame_count = 96_000usize;
    let mut frames = Vec::with_capacity(frame_count * 2);
    for index in 0..frame_count {
        let seconds = index as f32 / 48_000.0;
        let sample = (2.0 * std::f32::consts::PI * 110.0 * seconds).sin() * 0.5;
        frames.push(sample);
        frames.push(sample);
    }
    let curve = vec![
        StretchRatioPoint::new(0, 1.0),
        StretchRatioPoint::new(32_000, 1.6),
        StretchRatioPoint::new(64_000, 0.8),
    ];
    let seams = dynamic_ratio_output_boundaries(frame_count, &curve, 1.0);
    assert!(!seams.is_empty(), "the curve must produce segment joins");

    // Three renders: segments concatenated raw, the same with the seam
    // smoother, and the resumable renderer that has no join at all.
    let segments = coalesce_short_dynamic_ratio_segments(
        dynamic_ratio_segments(frame_count, &curve, 1.0),
        min_dynamic_ratio_segment_frames(DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP),
    );
    let mut unsmoothed = Vec::new();
    for segment in segments {
        unsmoothed.extend(stretch_to_exact_linked_stereo(
            &frames[segment.start_frame * 2..segment.end_frame * 2],
            segment.target_frames,
            DEFAULT_WINDOW_SIZE,
            DEFAULT_ANALYSIS_HOP,
        ));
    }
    let smoothed = stretch_dynamic_ratio_linked_stereo_with_engine(
        &frames,
        &curve,
        1.0,
        DEFAULT_WINDOW_SIZE,
        DEFAULT_ANALYSIS_HOP,
    )
    .expect("smoothed segmented render");
    let resumable = OfflineHighQualityStretcher::new(1.0)
        .stretch_dynamic_ratio_interleaved_stereo(&frames, &curve)
        .expect("resumable render");
    assert_eq!(smoothed.len(), resumable.len());
    assert_eq!(unsmoothed.len(), smoothed.len());

    let click =
        |data: &[Sample]| measure_dynamic_segment_seam_click(data, 2, &seams, 1.0).click_dbfs;
    let unsmoothed_click = click(&unsmoothed);
    let smoothed_click = click(&smoothed);
    let resumable_click = click(&resumable);
    println!(
        "unsmoothed {unsmoothed_click:.2} smoothed {smoothed_click:.2} \
         resumable {resumable_click:.2} dBFS"
    );

    // Both segmented renders leave a seam the measurement can see. This is
    // the half of the assertion that shows the measurement works: an
    // earlier version of this metric scored the smoothed render -240 dBFS
    // because the smoother sets the two samples it reads to their midpoint.
    assert!(
        unsmoothed_click > -40.0,
        "raw segment joins should be plainly visible, got {unsmoothed_click:.2} dBFS",
    );
    assert!(
        smoothed_click > -40.0,
        "the smoother spreads the join across its fade rather than removing \
         it, so it should still be visible, got {smoothed_click:.2} dBFS",
    );

    // The resumable renderer carries phase, detector, and overlap-add state
    // across the join, so there is no restart to hear.
    assert!(
        resumable_click < smoothed_click - 40.0,
        "resumable should be far below the smoothed segmented render: \
         {resumable_click:.2} vs {smoothed_click:.2} dBFS",
    );
}
