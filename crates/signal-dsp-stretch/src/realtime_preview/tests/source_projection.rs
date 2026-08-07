use super::*;

#[test]
fn realtime_preview_fixed_ratio_source_projection_reports_required_source_span() {
    let slow = project_realtime_preview_fixed_ratio_source_advance(480, 96, 2.0);
    assert_eq!(slow.ratio, 2.0);
    assert_eq!(slow.output_start_frame, 480);
    assert_eq!(slow.output_frames, 96);
    assert_eq!(slow.output_end_frame, 576);
    assert_eq!(slow.source_start_frame, 240.0);
    assert_eq!(slow.source_end_frame, 288.0);
    assert_eq!(slow.source_advance_frames, 48.0);
    assert_eq!(slow.source_frame_floor, 240);
    assert_eq!(slow.source_frame_ceil, 288);
    assert_eq!(slow.source_frames_required, 48);

    let fast = project_realtime_preview_fixed_ratio_source_advance(480, 96, 0.5);
    assert_eq!(fast.source_start_frame, 960.0);
    assert_eq!(fast.source_end_frame, 1152.0);
    assert_eq!(fast.source_advance_frames, 192.0);
    assert_eq!(fast.source_frames_required, 192);

    let identity = project_realtime_preview_fixed_ratio_source_advance(480, 96, 1.0);
    assert_eq!(identity.source_start_frame, 480.0);
    assert_eq!(identity.source_end_frame, 576.0);
    assert_eq!(identity.source_frames_required, 96);
}

#[test]
fn realtime_preview_fixed_ratio_source_projection_covers_fractional_source_bounds() {
    let projection = project_realtime_preview_fixed_ratio_source_advance(0, 256, 1.5);

    assert_eq!(projection.source_frame_floor, 0);
    assert_eq!(projection.source_frame_ceil, 171);
    assert_eq!(projection.source_frames_required, 171);
    assert!((projection.source_advance_frames - (256.0 / 1.5)).abs() < 1.0e-9);

    let sanitized = project_realtime_preview_fixed_ratio_source_advance(32, 64, f64::NAN);
    assert_eq!(sanitized.ratio, 1.0);
    assert_eq!(sanitized.source_start_frame, 32.0);
    assert_eq!(sanitized.source_end_frame, 96.0);
}

#[test]
fn realtime_preview_source_projection_state_advances_fractional_cursor() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");

    let first = state
        .advance_source_projection(96, 1.5)
        .expect("projection should stay within the configured block size");
    let second = state
        .advance_source_projection(96, 1.5)
        .expect("projection should stay within the configured block size");

    assert_eq!(first.output_start_frame, 0);
    assert_eq!(first.output_end_frame, 96);
    assert_eq!(first.source_start_frame, 0.0);
    assert_eq!(first.source_end_frame, 64.0);
    assert_eq!(first.source_frames_required, 64);
    assert_eq!(second.output_start_frame, 96);
    assert_eq!(second.output_end_frame, 192);
    assert_eq!(second.source_start_frame, 64.0);
    assert_eq!(second.source_end_frame, 128.0);
    assert_eq!(second.source_frames_required, 64);
    assert_eq!(state.source_projection_output_frame(), 192);
    assert_eq!(state.source_projection_source_cursor(), 128.0);
    assert_eq!(state.last_source_projection(), second);

    state.reset();
    assert_eq!(state.source_projection_output_frame(), 0);
    assert_eq!(state.source_projection_source_cursor(), 0.0);
    assert_eq!(
        state.last_source_projection(),
        project_realtime_preview_fixed_ratio_source_advance(0, 0, 1.0)
    );
}

#[test]
fn realtime_preview_source_projection_state_bounds_input_demand() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");

    let fast_limit = state.source_projection_input_demand_limit_frames(0.5);
    let fast = state
        .advance_source_projection(128, 0.5)
        .expect("projection should stay within the configured block size");
    assert_eq!(fast.source_advance_frames, 256.0);
    assert_eq!(fast.source_frames_required, 256);
    assert!(fast.source_frames_required <= fast_limit);

    let fractional_limit = state.source_projection_input_demand_limit_frames(3.0);
    let fractional = state
        .advance_source_projection(100, 3.0)
        .expect("projection should stay within the configured block size");
    assert!((fractional.source_advance_frames - (100.0 / 3.0)).abs() < 1.0e-9);
    assert_eq!(fractional.source_frame_floor, 256);
    assert_eq!(fractional.source_frame_ceil, 290);
    assert_eq!(fractional.source_frames_required, 34);
    assert!(fractional.source_frames_required <= fractional_limit);

    assert_eq!(
        state.advance_source_projection(129, 1.0),
        Err(
            RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                requested: 129,
                max: 128,
            }
        )
    );
}

#[test]
fn realtime_preview_source_projection_state_is_deterministic_for_fixed_ratio() {
    let mut first = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");
    let mut second = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");

    for _ in 0..16 {
        let first_report = first
            .advance_source_projection(100, 3.0)
            .expect("projection should stay within the configured block size");
        let second_report = second
            .advance_source_projection(100, 3.0)
            .expect("projection should stay within the configured block size");
        assert_eq!(first_report, second_report);
        assert!(first_report.source_frames_required <= 35);
    }

    assert_eq!(
        first.source_projection_output_frame(),
        second.source_projection_output_frame()
    );
    assert!(
        (first.source_projection_source_cursor() - second.source_projection_source_cursor()).abs()
            < 1.0e-9
    );
}

#[test]
fn realtime_preview_scheduled_source_projection_applies_ratio_change_on_grid() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        96,
    ))
    .expect("callback state config should validate");

    for _ in 0..5 {
        let report = state
            .advance_scheduled_source_projection(96, 1.0)
            .expect("projection should stay within the configured block size");
        assert!(!report.ratio_change_applied);
        assert_eq!(report.start_ratio, 1.0);
        assert_eq!(report.end_ratio, 1.0);
    }

    let changed = state
        .advance_scheduled_source_projection(96, 1.5)
        .expect("projection should stay within the configured block size");

    assert!(changed.ratio_change_applied);
    assert_eq!(changed.output_start_frame, 480);
    assert_eq!(changed.output_end_frame, 576);
    assert_eq!(changed.source_start_frame, 480.0);
    assert_eq!(changed.ratio_change_request_output_frame, 480);
    assert_eq!(changed.ratio_change_output_frame, 512);
    assert_eq!(changed.ratio_change_source_frame, 512.0);
    assert_eq!(changed.ratio_change_alignment_error_frames, 32);
    assert_eq!(changed.start_ratio, 1.0);
    assert_eq!(changed.end_ratio, 1.5);
    assert!((changed.source_end_frame - (512.0 + 64.0 / 1.5)).abs() < 1.0e-9);
    assert_eq!(state.source_projection_active_ratio(), 1.5);
    assert_eq!(state.source_projection_ratio_change_count(), 1);
    assert_eq!(
        state.last_source_projection_ratio_change_output_frame(),
        512
    );
    assert_eq!(
        state.last_source_projection_ratio_change_source_frame(),
        512.0
    );
    assert!(
        state.last_source_projection_ratio_change_alignment_error_frames()
            <= state.ratio_change_alignment_tolerance_frames()
    );

    let next = state
        .advance_scheduled_source_projection(96, 1.5)
        .expect("projection should stay within the configured block size");
    assert!(!next.ratio_change_applied);
    assert_eq!(next.start_ratio, 1.5);
    assert_eq!(next.end_ratio, 1.5);
    assert!((next.source_start_frame - changed.source_end_frame).abs() < 1.0e-9);
    assert_eq!(next.output_start_frame, changed.output_end_frame);
}

#[test]
fn realtime_preview_scheduled_source_projection_is_continuous_across_tempo_ramp() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        96,
    ))
    .expect("callback state config should validate");
    let mut previous_output_end = 0;
    let mut previous_source_end = 0.0;
    let mut changes = Vec::new();

    for block_index in 0..18 {
        let ratio = if block_index < 5 {
            0.75
        } else if block_index < 10 {
            1.0
        } else {
            1.5
        };
        let report = state
            .advance_scheduled_source_projection(96, ratio)
            .expect("projection should stay within the configured block size");

        assert_eq!(report.output_start_frame, previous_output_end);
        assert!((report.source_start_frame - previous_source_end).abs() < 1.0e-9);
        assert!(report.source_end_frame >= report.source_start_frame);
        assert!(report.source_frames_required <= 129);
        if report.ratio_change_applied {
            assert!(
                report.ratio_change_alignment_error_frames
                    <= state.ratio_change_alignment_tolerance_frames()
            );
            assert!(
                report.ratio_change_source_frame >= report.source_start_frame
                    && report.ratio_change_source_frame <= report.source_end_frame
            );
            changes.push((
                report.ratio_change_output_frame,
                report.ratio_change_source_frame,
            ));
        }

        previous_output_end = report.output_end_frame;
        previous_source_end = report.source_end_frame;
    }

    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].0, 0);
    assert_eq!(changes[1].0, 512);
    assert_eq!(changes[2].0, 1024);
    assert!(changes.windows(2).all(|pair| pair[0].1 <= pair[1].1));
    assert_eq!(state.source_projection_ratio_change_count(), 3);
    assert_eq!(state.source_projection_current_ratio(), 1.5);
    assert_eq!(state.source_projection_active_ratio(), 1.5);
    assert_eq!(
        state.last_dynamic_source_projection().output_end_frame,
        previous_output_end
    );
}
