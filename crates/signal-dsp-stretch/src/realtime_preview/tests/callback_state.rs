use super::*;

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
