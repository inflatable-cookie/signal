use super::support::*;
use super::*;

#[test]
fn sample_clips_play_buffer_content_at_their_window() {
    let (mut controller, mut executor) = render_plane();
    // 1024 source frames: value = index / 1024.
    let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
    // Window starts at frame 512, well past the edge ramp warm-up.
    let spec = samples_spec(&values, 512, 512 + 1024, false);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    // Two 256-frame blocks open the edge ramp and reach frame 512.
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 512+128 plays source frame 128, past the clip edge fade.
    let index = 128usize;
    let expected = 128.0 / 1024.0;
    assert!((frames[index * 2] - expected).abs() < 1e-5);
    // Same-rate playback: equality on both channels.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
}

#[test]
fn mono_source_upmixes_into_stereo_stage() {
    let (mut controller, mut executor) = render_plane();
    // A MONO source (channels = 1): value = index / 1024.
    let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 2001,
            start_frames: 512,
            end_frames: 512 + 1024,
            source: RenderSource::Samples(RenderSampleBuffer::mono(48_000, values.into())),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Source frame 128, up-mixed mono→stereo at the equal-power 1/√2 gain.
    let index = 128usize;
    let expected = (128.0 / 1024.0) * std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        (frames[index * 2] - expected).abs() < 1e-5,
        "L = {}",
        frames[index * 2]
    );
    assert!(
        (frames[index * 2 + 1] - expected).abs() < 1e-5,
        "R = {}",
        frames[index * 2 + 1]
    );
    // Mono is duplicated equally to both ears.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
}

#[test]
fn sample_clips_play_their_final_frame() {
    let (mut controller, mut executor) = render_plane();
    // 256 source frames of a constant; window longer than the source.
    let values = vec![0.5f32; 256];
    let spec = samples_spec(&values, 0, u64::MAX, false);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 1);

    // Frames 0..256 played in the warm-up block. The final source frame
    // (255) must have rendered; beyond the source, silence.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Replay from the start and inspect the last in-range frame.
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 255 is the final source frame; with the clamp it plays.
    assert!(frames[255 * 2].abs() > 0.1);
}
#[test]
fn looping_sample_clips_wrap_to_their_start() {
    let (mut controller, mut executor) = render_plane();
    // 100 source frames: value = (index + 1) / 100, looped.
    let values: Vec<f32> = (0..100).map(|index| (index + 1) as f32 / 100.0).collect();
    let spec = samples_spec(&values, 0, u64::MAX, true);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2); // 512 frames: ramp open, loop wrapped 5x.

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Block covers frames 512..768; frame 512 plays source 512 % 100 = 12.
    let expected = 13.0 / 100.0;
    assert!((frames[0] - expected).abs() < 1e-5);
    // Frame 600 wraps to source 0.
    let wrapped = (600 - 512) * 2;
    assert!((frames[wrapped] - 1.0 / 100.0).abs() < 1e-5);
}
#[test]
fn loop_region_wraps_sample_accurately_with_a_microfade() {
    // Ramp content (value = frame / 1024) under loop region [480, 992).
    // The block covering 768..1024 crosses 992 at buffer frame 224:
    // frames 0..224 play 768..992, frames 224..256 play 480..512, with
    // the 64-frame micro-fade out before the wrap and in after it.
    let (mut controller, mut executor) = render_plane();
    let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
    let spec = samples_spec(&values, 0, u64::MAX, false);
    controller.install_plan(&spec).unwrap();
    controller.set_loop_region(Some((480, 992))).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 3); // 768 frames: edge ramp open, no wrap yet.
    assert_eq!(controller.position_frames(), 768);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Clock wrapped: 480 + (256 - 224) = 512.
    assert_eq!(controller.position_frames(), 512);

    let wrap = 224usize;
    let fade = LOOP_WRAP_FADE_FRAMES;
    // Before the fade-out window: linear playback of the pre-wrap span.
    for index in [10usize, 100, wrap - fade - 1] {
        let expected = (768 + index) as f32 / 1024.0;
        assert!(
            (frames[index * 2] - expected).abs() < 1e-5,
            "pre-wrap frame {index}: {} vs {expected}",
            frames[index * 2],
        );
    }
    // Fade-out: content × linear ramp down to zero at the wrap point.
    for step in 0..fade {
        let index = wrap - fade + step;
        let gain = (fade - 1 - step) as f32 / fade as f32;
        let expected = (768 + index) as f32 / 1024.0 * gain;
        assert!(
            (frames[index * 2] - expected).abs() < 1e-5,
            "fade-out frame {index}: {} vs {expected}",
            frames[index * 2],
        );
    }
    // Fade-in: wrapped content (from loop_start) × linear ramp up.
    let fade_in = 256 - wrap; // 32 frames of post-wrap audio in the block.
    for step in 0..fade_in {
        let index = wrap + step;
        let gain = (step + 1) as f32 / fade_in as f32;
        let expected = (480 + step) as f32 / 1024.0 * gain;
        assert!(
            (frames[index * 2] - expected).abs() < 1e-5,
            "fade-in frame {index}: {} vs {expected}",
            frames[index * 2],
        );
    }

    // The next block continues linearly from loop_start + remainder.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(controller.position_frames(), 768);
    let expected = (512 + 100) as f32 / 1024.0;
    assert!((frames[100 * 2] - expected).abs() < 1e-5);
}

#[test]
fn seeking_outside_the_loop_region_plays_without_wrapping() {
    // Loop only triggers when crossing loop_end: a position past the
    // region renders linearly.
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_loop_region(Some((0, 256))).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 1); // Wraps once: 0..256 crosses 256.
    assert_eq!(controller.position_frames(), 0);

    controller.seek(96_000).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames); // Ramp-out block; seek lands.
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    // Past the region: the clock advances linearly, no wrap.
    assert_eq!(controller.position_frames(), 96_000 + 512);
}

#[test]
fn clearing_the_loop_region_restores_linear_playback() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_loop_region(Some((0, 256))).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 1);
    assert_eq!(controller.position_frames(), 0);

    controller.set_loop_region(None).unwrap();
    warm_up(&mut executor, 2);
    assert_eq!(controller.position_frames(), 512);
}

#[test]
fn looped_stream_clips_underrun_at_most_transiently_across_wraps() {
    // A short loop over a streamed clip: the wrap jumps wanted_frame
    // backwards, the feeder re-serves it like any seek, and underruns
    // stay transient per the documented stream semantics. Lenient by
    // design — the exact underrun count depends on chunk timing.
    let (mut controller, mut executor) = render_plane();
    let total = 48_000u64;
    let (feeder, handle) = render_stream(48_000, total);
    controller
        .install_plan(&stream_spec(&handle, 0, total))
        .unwrap();
    controller.set_loop_region(Some((0, 4_096))).unwrap();
    controller.set_playing(true).unwrap();
    feed_ramp(&feeder, total, 0, 4_096, 512);

    let mut frames = [0.0f32; 512];
    let mut audible_blocks = 0usize;
    let mut rendered_frames = 0u64;
    // Cursor-based feeding mirroring the production feeder (pulse's
    // `service_streams`): sequential decode toward wanted + lookahead,
    // cursor reset on seek-shaped jumps, no duplicate re-sends.
    let chunk_frames = 512u64;
    let mut cursor = 4_096u64;
    for _ in 0..64 {
        executor.render_block(&mut frames);
        rendered_frames += 256;
        if frames.iter().any(|sample| sample.abs() > 1e-4) {
            audible_blocks += 1;
        }
        drop(feeder.collect_retired());
        let wanted = feeder.wanted_frame().min(total);
        let aligned = wanted - wanted % chunk_frames;
        let target = (wanted + 2_048).min(total);
        if cursor < aligned || cursor > target + chunk_frames {
            cursor = aligned;
        }
        while cursor < target {
            let count = chunk_frames.min(total - cursor);
            let mut data = Vec::with_capacity(count as usize * 2);
            for frame in cursor..cursor + count {
                let value = frame as f32 / total as f32;
                data.push(value);
                data.push(value);
            }
            if feeder
                .try_send_chunk(StreamChunk {
                    start_frame: cursor,
                    frames: data.into(),
                })
                .is_err()
            {
                break;
            }
            cursor += count;
        }
    }
    // 64 × 256 = 16_384 frames over a 4_096-frame loop: four wraps.
    // Most blocks must have been audible and underruns must stay a
    // fraction of the rendered span (transient, not systemic).
    assert!(audible_blocks > 48, "only {audible_blocks} audible blocks");
    assert!(
        handle.underrun_frames() < rendered_frames / 4,
        "underruns not transient: {} of {rendered_frames}",
        handle.underrun_frames(),
    );
}
#[test]
fn rate_converted_clips_play_through_the_sinc_path() {
    // 1 kHz sine at 44.1k played on a 48k stream: after the edge ramp
    // and clip fade, output must track the analytic sine to ~60 dB
    // (linear interpolation fails this at ~35-40 dB).
    let (mut controller, mut executor) = render_plane();
    let source_rate = 44_100u32;
    let frequency = 1_000.0f64;
    let mut data = Vec::new();
    for n in 0..44_100 {
        let value =
            (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
        data.push(value);
        data.push(value);
    }
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1005,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Samples(RenderSampleBuffer::stereo(source_rate, data.into())),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4); // 1024 frames: ramp open, fades passed.

    let mut frames = vec![0.0f32; 2048];
    executor.render_block(&mut frames);
    let step = source_rate as f64 / 48_000.0;
    let mut error = 0.0f64;
    let mut power = 0.0f64;
    for frame_index in 0..1024usize {
        let stream_frame = 1024 + frame_index as u64;
        let position = stream_frame as f64 * step;
        let expected = (std::f64::consts::TAU * frequency * position / source_rate as f64).sin();
        let actual = frames[frame_index * 2] as f64;
        error += (actual - expected) * (actual - expected);
        power += expected * expected;
    }
    let snr = 10.0 * (power / error.max(1e-30)).log10();
    assert!(snr > 60.0, "rate-converted playback SNR {snr:.1} dB");
}

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
