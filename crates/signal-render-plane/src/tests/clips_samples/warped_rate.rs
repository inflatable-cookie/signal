use super::super::support::*;
use super::super::*;

// ── Warped (rate-multiplied) sources (g12.027) ──────────────────────

#[test]
fn warped_samples_clip_plays_at_the_rate_multiplied_step() {
    // 440 Hz sine at the stream rate warped by 1.5: playback must track
    // the analytic sine advanced at 1.5 source frames per stream frame
    // (i.e. sound at 660 Hz), through the same sinc path as SRC.
    let (mut controller, mut executor) = render_plane();
    let rate = 1.5f64;
    let frequency = 440.0f64;
    let mut data = Vec::new();
    for n in 0..96_000 {
        let value = (std::f64::consts::TAU * frequency * n as f64 / 48_000.0).sin() as f32;
        data.push(value);
        data.push(value);
    }
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1006,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Warped {
                source: Box::new(RenderSource::Samples(RenderSampleBuffer::stereo(
                    48_000,
                    data.into(),
                ))),
                rate,
            },
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4);

    let mut frames = vec![0.0f32; 2048];
    executor.render_block(&mut frames);
    let mut error = 0.0f64;
    let mut power = 0.0f64;
    for frame_index in 0..1024usize {
        let stream_frame = 1024 + frame_index as u64;
        let position = stream_frame as f64 * rate;
        let expected = (std::f64::consts::TAU * frequency * position / 48_000.0).sin();
        let actual = frames[frame_index * 2] as f64;
        error += (actual - expected) * (actual - expected);
        power += expected * expected;
    }
    let snr = 10.0 * (power / error.max(1e-30)).log10();
    assert!(snr > 60.0, "warped playback SNR {snr:.1} dB");
}

#[test]
fn warped_source_exhausts_early_at_faster_rates() {
    // A 1 s buffer at rate 2.0 runs out of source after 0.5 s of stream
    // time: the second half of the clip window renders silence.
    let (mut controller, mut executor) = render_plane();
    let data: Vec<f32> = std::iter::repeat_n([0.5f32, 0.5f32], 48_000)
        .flatten()
        .collect();
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1007,
            start_frames: 0,
            end_frames: 96_000,
            source: RenderSource::Warped {
                source: Box::new(RenderSource::Samples(RenderSampleBuffer::stereo(
                    48_000,
                    data.into(),
                ))),
                rate: 2.0,
            },
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    // Render to just past the source-exhaustion point (24 000 stream
    // frames) and confirm content, then silence.
    let mut audible_at_20k = 0.0f32;
    let mut audible_at_30k = 0.0f32;
    let mut frames = vec![0.0f32; 512];
    for block in 0..128u64 {
        executor.render_block(&mut frames);
        let block_start = block * 256;
        if block_start == 19_968 {
            audible_at_20k = frames.iter().fold(0.0f32, |a, s| a.max(s.abs()));
        }
        if block_start == 29_952 {
            audible_at_30k = frames.iter().fold(0.0f32, |a, s| a.max(s.abs()));
        }
    }
    assert!(
        audible_at_20k > 0.1,
        "expected audible content before exhaustion, peak {audible_at_20k}"
    );
    assert!(
        audible_at_30k < 1.0e-3,
        "expected silence after source exhaustion, peak {audible_at_30k}"
    );
}

#[test]
fn warped_source_over_non_media_rejects_at_compile() {
    let (mut controller, _executor) = render_plane();
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1008,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Warped {
                source: Box::new(RenderSource::TestTone {
                    frequency_hz: 440.0,
                }),
                rate: 1.5,
            },
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    let error = controller.install_plan(&spec).unwrap_err();
    let expected = RenderPlanCompileError::WarpedSourceUnsupported {
        stage_id: LANE_ID,
        clip_id: 1008,
    };
    assert!(
        error.message.contains(&expected.to_string()),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn warped_source_sanitizes_degenerate_rates_to_identity() {
    // NaN/zero/negative rates compile as 1.0: playback matches the
    // unwarped source sample for sample.
    for rate in [f64::NAN, 0.0, -2.0] {
        let (mut controller, mut executor) = render_plane();
        let data: Vec<f32> = (0..9_600)
            .flat_map(|n| {
                let value = ((n as f32 * 0.13).sin()) * 0.5;
                [value, value]
            })
            .collect();
        let buffer = RenderSampleBuffer::stereo(48_000, data.into());
        let warped = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1009,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Warped {
                    source: Box::new(RenderSource::Samples(buffer.clone())),
                    rate,
                },
                loop_source: false,
                fade_in_frames: 0,
                fade_out_frames: 0,
            }],
        );
        controller.install_plan(&warped).unwrap();
        controller.set_playing(true).unwrap();
        let mut warped_frames = vec![0.0f32; 2048];
        executor.render_block(&mut warped_frames);

        let (mut controller, mut executor) = render_plane();
        let plain = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1009,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Samples(buffer),
                loop_source: false,
                fade_in_frames: 0,
                fade_out_frames: 0,
            }],
        );
        controller.install_plan(&plain).unwrap();
        controller.set_playing(true).unwrap();
        let mut plain_frames = vec![0.0f32; 2048];
        executor.render_block(&mut plain_frames);

        assert_eq!(warped_frames, plain_frames, "rate {rate}");
    }
}
