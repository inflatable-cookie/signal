//! RealtimePreview unit tests.

use super::{
    plan_realtime_preview_stream, project_realtime_preview_fixed_ratio_source_advance,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackState,
    RealtimePreviewCallbackTimelineMode, RealtimePreviewIntegrationMode, RealtimePreviewPlanError,
    RealtimePreviewStreamConfig, RealtimePreviewUnsupportedMode,
};
use crate::benchmark::{
    compare_synthetic_realtime_preview_backends, generate_synthetic_stretch_audio,
    measure_dynamic_segment_seam_click, StretchBenchmarkBackend, StretchBenchmarkPath,
    StretchCorpusFamily, StretchMetric,
};
use crate::{
    RealtimePreviewStretcher, Sample, StretchQuality, StretchRatioPoint, TimeStretcher,
    REALTIME_PREVIEW_ANALYSIS_HOP, REALTIME_PREVIEW_WINDOW_SIZE,
};
use signal_primitives::SampleRate;

fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
    (0..len)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin())
        .collect()
}

fn rms(samples: &[Sample]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

/// Dominant frequency estimate by zero-crossing count over a trimmed
/// interior span (skips windup/tail edges).
fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
}

#[test]
fn realtime_preview_contract_reports_latency_and_callback_blocker() {
    let contract =
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 2, 128))
            .expect("default preview contract should plan");

    assert_eq!(contract.config.window_size, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(contract.config.analysis_hop, REALTIME_PREVIEW_ANALYSIS_HOP);
    assert_eq!(contract.input_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(contract.output_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(
        contract.ratio_change_alignment_tolerance_frames,
        REALTIME_PREVIEW_ANALYSIS_HOP + 128
    );
    assert_eq!(
        contract.integration_mode,
        RealtimePreviewIntegrationMode::AnticipativePreRender
    );
    assert_eq!(
        contract.callback_timeline_mode,
        RealtimePreviewCallbackTimelineMode::QuantumLocked
    );
    assert!(!contract.audio_thread_processing_supported);
    assert_eq!(
        contract.unsupported_mode,
        Some(RealtimePreviewUnsupportedMode::SourceBufferingContract)
    );
}

#[test]
fn realtime_preview_contract_rejects_invalid_streams() {
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(0), 2, 128,)),
        Err(RealtimePreviewPlanError::InvalidSampleRate)
    );
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 6, 128,)),
        Err(RealtimePreviewPlanError::UnsupportedChannelCount(6))
    );
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 2, 0,)),
        Err(RealtimePreviewPlanError::InvalidBlockSize)
    );
}

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

#[test]
fn realtime_preview_callback_state_validates_stereo_geometry_without_enabling_contract() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");
    let input = vec![0.0; 128 * 2];
    let mut output = vec![0.25; 128 * 2];

    assert_eq!(state.config().channel_count, 2);
    assert_eq!(state.scratch_capacity_samples(), 128 * 2);
    assert!(state.input_ring_capacity_samples() >= REALTIME_PREVIEW_WINDOW_SIZE * 2);
    assert_eq!(
        state.input_ring_capacity_samples(),
        state.output_ring_capacity_samples()
    );
    assert_eq!(
        state.output_ring_capacity_samples(),
        state.normalization_ring_capacity_samples()
    );
    assert_eq!(state.window_size(), REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(
        state.spectral_scratch_samples(),
        REALTIME_PREVIEW_WINDOW_SIZE * 2
    );
    assert_eq!(
        state.phase_state_values(),
        (REALTIME_PREVIEW_WINDOW_SIZE / 2 + 1) * 2
    );
    assert!(!state.contract().audio_thread_processing_supported);
    let report = state
        .process(&input, &mut output, 128, 1.25)
        .expect("linked-stereo callback kernel should process");
    assert_eq!(report.input_frames, 128);
    assert_eq!(report.output_frames, 128);
    assert_eq!(report.processed_frames, 128);
    assert_eq!(state.current_ratio(), 1.25);
    assert!(output.iter().all(|sample| *sample == 0.0));

    state.reset();
    assert_eq!(state.current_ratio(), 1.0);
    assert_eq!(state.processed_frames(), 0);
}

#[test]
fn realtime_preview_callback_state_rejects_bad_callback_blocks() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");
    let input = vec![0.0; 128 * 2];
    let mut output = vec![0.0; 128 * 2];

    assert_eq!(
        state.process(&input, &mut output, 129, 1.0),
        Err(
            RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                requested: 129,
                max: 128,
            }
        )
    );
    assert_eq!(
        state.process(&input[..64], &mut output, 128, 1.0),
        Err(RealtimePreviewCallbackProcessError::BufferTooSmall {
            required_samples: 256,
            input_samples: 64,
            output_samples: 256,
        })
    );
}

#[test]
fn realtime_preview_callback_state_processes_mono_stream_without_allocation_contract_claim() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        128,
    ))
    .expect("callback state config should validate");
    let input = sine(440.0, 48_000.0, 128 * 48);
    let mut output = vec![0.0; input.len()];

    for block_index in 0..48 {
        let start = block_index * 128;
        let report = state
            .process(
                &input[start..start + 128],
                &mut output[start..start + 128],
                128,
                1.0,
            )
            .expect("mono callback kernel should process");
        assert_eq!(report.input_frames, 128);
        assert_eq!(report.output_frames, 128);
        assert_eq!(report.processed_frames, ((block_index + 1) * 128) as u64);
    }

    assert!(!state.contract().audio_thread_processing_supported);
    assert!(rms(&output[1024..]) > 0.05);
    assert!((dominant_frequency_hz(&output[1024..], 48_000.0) - 440.0).abs() < 20.0);
}

#[test]
fn realtime_preview_callback_state_is_deterministic_for_fixed_ratio() {
    let input = sine(330.0, 48_000.0, 128 * 48);
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
    let mut first_output = vec![0.0; input.len()];
    let mut second_output = vec![0.0; input.len()];

    for block_index in 0..48 {
        let start = block_index * 128;
        first
            .process(
                &input[start..start + 128],
                &mut first_output[start..start + 128],
                128,
                1.25,
            )
            .expect("first mono callback kernel should process");
        second
            .process(
                &input[start..start + 128],
                &mut second_output[start..start + 128],
                128,
                1.25,
            )
            .expect("second mono callback kernel should process");
    }

    assert_eq!(first_output, second_output);
    assert!(rms(&first_output[1024..]) > 0.02);
}

#[test]
fn realtime_preview_callback_state_processes_linked_stereo_stream() {
    let left = sine(330.0, 48_000.0, 128 * 64);
    let right = sine(660.0, 48_000.0, 128 * 64);
    let input = left
        .iter()
        .zip(right.iter())
        .flat_map(|(left, right)| [*left, *right])
        .collect::<Vec<_>>();
    let mut first = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");
    let mut second = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        2,
        128,
    ))
    .expect("callback state config should validate");
    let mut first_output = vec![0.0; input.len()];
    let mut second_output = vec![0.0; input.len()];

    for block_index in 0..64 {
        let start = block_index * 128 * 2;
        first
            .process(
                &input[start..start + 128 * 2],
                &mut first_output[start..start + 128 * 2],
                128,
                1.0,
            )
            .expect("first linked-stereo callback kernel should process");
        second
            .process(
                &input[start..start + 128 * 2],
                &mut second_output[start..start + 128 * 2],
                128,
                1.0,
            )
            .expect("second linked-stereo callback kernel should process");
    }

    let out_left = first_output
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>();
    let out_right = first_output
        .chunks_exact(2)
        .map(|frame| frame[1])
        .collect::<Vec<_>>();

    assert_eq!(first_output, second_output);
    assert!(rms(&out_left[1024..]) > 0.05);
    assert!(rms(&out_right[1024..]) > 0.05);
    assert!((dominant_frequency_hz(&out_left[1024..], 48_000.0) - 330.0).abs() < 20.0);
    assert!((dominant_frequency_hz(&out_right[1024..], 48_000.0) - 660.0).abs() < 25.0);
}

#[test]
fn realtime_preview_callback_state_schedules_ratio_changes_on_analysis_grid() {
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(48_000),
        1,
        96,
    ))
    .expect("callback state config should validate");
    let input = sine(440.0, 48_000.0, 96 * 16);
    let mut output = vec![0.0; input.len()];

    for block_index in 0..16 {
        let start = block_index * 96;
        let ratio = if block_index < 5 { 1.0 } else { 1.5 };
        let report = state
            .process(
                &input[start..start + 96],
                &mut output[start..start + 96],
                96,
                ratio,
            )
            .expect("callback kernel should process dynamic ratio");
        assert_eq!(report.ratio, ratio);
        assert!(
            report.ratio_change_alignment_error_frames
                <= state.ratio_change_alignment_tolerance_frames()
        );
    }

    assert_eq!(state.current_ratio(), 1.5);
    assert_eq!(state.active_ratio(), 1.5);
    assert_eq!(state.ratio_change_count(), 1);
    assert_eq!(state.last_ratio_change_request_frame(), 480);
    assert_eq!(state.last_ratio_change_applied_frame(), 512);
    assert_eq!(state.last_ratio_change_output_frame(), 1024);
    assert_eq!(state.last_ratio_change_alignment_error_frames(), 32);
    assert!(
        state.last_ratio_change_alignment_error_frames()
            <= state.ratio_change_alignment_tolerance_frames()
    );
}

#[test]
fn realtime_preview_callback_state_bounds_dynamic_ratio_seams_on_tempo_ramp() {
    let input = generate_synthetic_stretch_audio(StretchCorpusFamily::TempoRamp)
        .expect("tempo ramp synthetic case should exist");
    let ratio_change_frames = [input.frame_count() / 3, input.frame_count() * 2 / 3];
    let mut state = RealtimePreviewCallbackState::new(RealtimePreviewStreamConfig::new(
        SampleRate(input.sample_rate_hz),
        input.channels as usize,
        96,
    ))
    .expect("callback state config should validate");
    let mut output = vec![0.0; input.samples.len()];
    let mut seam_frames = Vec::new();
    let mut last_ratio_change_count = 0;

    for block_start in (0..input.frame_count()).step_by(96) {
        let frame_count = (input.frame_count() - block_start).min(96);
        let sample_start = block_start * input.channels as usize;
        let sample_end = sample_start + frame_count * input.channels as usize;
        let ratio = if block_start < ratio_change_frames[0] {
            0.75
        } else if block_start < ratio_change_frames[1] {
            1.0
        } else {
            1.5
        };
        let report = state
            .process(
                &input.samples[sample_start..sample_end],
                &mut output[sample_start..sample_end],
                frame_count,
                ratio,
            )
            .expect("callback kernel should process tempo ramp");
        if report.ratio_change_count > last_ratio_change_count
            && state.last_ratio_change_request_frame() > 0
        {
            seam_frames.push(report.ratio_change_output_frame as usize);
        }
        last_ratio_change_count = report.ratio_change_count;
    }

    let seam = measure_dynamic_segment_seam_click(&output, input.channels, &seam_frames, 1.0);

    assert_eq!(seam_frames.len(), 2);
    assert_eq!(seam.seam_frames, seam_frames);
    assert!(
        seam.peak_seam_delta < 0.35,
        "peak seam delta {}",
        seam.peak_seam_delta
    );
    assert!(
        seam.click_dbfs < -9.0,
        "seam click dBFS {}",
        seam.click_dbfs
    );
}

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

#[test]
fn realtime_preview_backend_comparison_covers_preview_subset() {
    let report = compare_synthetic_realtime_preview_backends();

    assert_eq!(report.comparisons.len(), 24);
    assert_eq!(
        report.improved_count
            + report.regressed_count
            + report.unchanged_count
            + report.inconclusive_count,
        report.comparisons.len()
    );
    for comparison in &report.comparisons {
        assert_eq!(comparison.baseline_backend, StretchBenchmarkBackend::Draft);
        assert_eq!(
            comparison.candidate_backend,
            StretchBenchmarkBackend::RealtimePreviewPrototype
        );
        assert!(comparison.ratio.is_finite());
        assert!(comparison.ratio > 0.0);
    }
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::StereoImageDelta
            && comparison.path == StretchBenchmarkPath::LinkedStereo
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:pitch_shift"
            && comparison.metric == StretchMetric::PitchErrorCents
            && comparison.path == StretchBenchmarkPath::PitchShift
            && comparison.pitch_shift_semitones == Some(12.0)
    }));
}
