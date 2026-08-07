//! Unit tests for the render plane.

use super::*;
use crate::live_input::LIVE_INPUT_MAX_BACKLOG_FRAMES;
use crate::plan_render::clip_window_gain;
use crate::plane::LOOP_WRAP_FADE_FRAMES;
use signal_dsp::equal_power_pan_matrix;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

const MASTER_ID: u64 = 1_000;
const LANE_ID: u64 = 1;

fn lane_node(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain,
        gain_automation: None,
        kind: RenderStageKind::Source { clips },
        inputs: Vec::new(),
    }
}

fn master_node(inputs: Vec<RenderEdgeSpec>) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id: MASTER_ID,
        format: ChannelFormat::stereo(),
        gain: 1.0,
        gain_automation: None,
        kind: RenderStageKind::Output,
        inputs,
    }
}

fn identity_edge(source_stage_id: u64) -> RenderEdgeSpec {
    RenderEdgeSpec {
        source_stage_id,
        gain: 1.0,
        matrix: None,
    }
}

/// The old flat shape: one stereo lane summed into a stereo master.
fn lane_master_spec(lane_gain: f32, clips: Vec<RenderClipSpec>) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(LANE_ID, lane_gain, clips),
            master_node(vec![identity_edge(LANE_ID)]),
        ],
    }
}

fn tone_clip(frequency_hz: f32) -> RenderClipSpec {
    RenderClipSpec {
        clip_id: 1003,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::TestTone { frequency_hz },
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    }
}

fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
    lane_master_spec(0.5, vec![tone_clip(frequency_hz)])
}

#[test]
fn renders_silence_without_plan_and_when_stopped() {
    let (mut controller, mut executor) = render_plane();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    controller.install_plan(&tone_spec(440.0)).unwrap();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), 0);
}

#[test]
fn renders_tone_and_advances_clock_when_playing() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(controller.position_frames(), 256);
    assert!(controller.playing());

    // Both channels carry the same mono sum.
    assert_eq!(frames[10], frames[11]);
}

#[test]
fn seek_moves_the_stream_clock() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    controller.seek(96_000).unwrap();

    let mut frames = [0.0f32; 128];
    executor.render_block(&mut frames);
    assert_eq!(controller.position_frames(), 96_000 + 64);
}

#[test]
fn windows_gate_lane_audibility_on_the_stream_clock() {
    let (mut controller, mut executor) = render_plane();
    let mut clip = tone_clip(440.0);
    clip.start_frames = 128;
    clip.end_frames = 256;
    let spec = lane_master_spec(0.5, vec![clip]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    // Block 0 covers frames 0..128: outside the window, silent.
    let mut frames = [0.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Block 1 covers frames 128..256: inside the window, audible.
    let mut frames = [0.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
}

fn samples_spec(
    values: &[f32],
    start_frames: u64,
    end_frames: u64,
    loop_source: bool,
) -> RenderPlanSpec {
    // Stereo frames with identical channels at the stream rate.
    let mut data = Vec::new();
    for value in values {
        data.push(*value);
        data.push(*value);
    }
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1004,
            start_frames,
            end_frames,
            source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
            loop_source,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Run blocks until the transport edge ramp has fully opened.
fn warm_up(executor: &mut RenderPlaneExecutor, blocks: usize) {
    let mut frames = [0.0f32; 512];
    for _ in 0..blocks {
        executor.render_block(&mut frames);
    }
}

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

// ── Per-clip equal-power fades (g13.024) ────────────────────────────────

/// DC-1.0 stereo samples clip filling its window exactly, with fades.
fn dc_clip(
    clip_id: u64,
    start_frames: u64,
    end_frames: u64,
    fade_in_frames: u32,
    fade_out_frames: u32,
) -> RenderClipSpec {
    let frames = (end_frames - start_frames) as usize;
    RenderClipSpec {
        clip_id,
        start_frames,
        end_frames,
        source: RenderSource::Samples(RenderSampleBuffer::stereo(
            48_000,
            vec![1.0f32; frames * 2].into(),
        )),
        loop_source: false,
        fade_in_frames,
        fade_out_frames,
    }
}

#[test]
fn clip_window_gain_known_answers() {
    // Declick-only (both fades 0) reproduces the historical expression
    // byte-for-byte: linear ramps over the edge fade, min-combined.
    for fade in [1u64, 5, 32] {
        for frame in 100u64..200 {
            let historical = (((frame - 100 + 1) as f32) / fade as f32)
                .min(((200 - frame) as f32) / fade as f32)
                .min(1.0);
            assert_eq!(
                clip_window_gain(frame, 100, 200, fade, 0, 0),
                historical,
                "declick divergence at frame {frame}, fade {fade}"
            );
        }
    }
    // Equal-power midpoints: sin(π/4) = √0.5 on each side.
    let mid_in = clip_window_gain(1_128, 1_000, 10_000, 32, 256, 0);
    assert!(
        (mid_in - 0.5f32.sqrt()).abs() < 1e-6,
        "fade-in mid {mid_in}"
    );
    let mid_out = clip_window_gain(9_872, 1_000, 10_000, 32, 0, 256);
    assert!(
        (mid_out - 0.5f32.sqrt()).abs() < 1e-6,
        "fade-out mid {mid_out}"
    );
    // A fade-out/fade-in pair overlapped by the fade length is equal
    // power: gains are complementary quarter-waves, a² + b² = 1 at every
    // frame of the overlap.
    let fade = 256u64;
    for frame in 744u64..1_000 {
        let a = clip_window_gain(frame, 0, 1_000, 32, 0, fade);
        let b = clip_window_gain(frame, 744, 1_744, 32, fade, 0);
        let power = a * a + b * b;
        assert!(
            (power - 1.0).abs() < 1e-6,
            "power {power} at frame {frame} (a {a}, b {b})"
        );
    }
    // A requested fade replaces the declick on its side only: the
    // fade-0 side keeps the declick ramp.
    let first = clip_window_gain(0, 0, 1_000, 32, 0, 256);
    assert_eq!(first, 1.0 / 32.0, "declick retained on fade-0 side");
    let faded_first = clip_window_gain(0, 0, 1_000, 32, 256, 0);
    assert_eq!(faded_first, 0.0, "equal-power fade-in starts at exact 0");
}

#[test]
fn overlapping_clips_crossfade_at_equal_power() {
    // A fades out over [1280, 1536); B overlaps that exact span with its
    // fade-in. Same lane stage: clips sum additively into the scratch.
    // For correlated (DC) content the sum is the analytic sin+cos curve —
    // never below unity through the crossfade (no dip, unlike the
    // declick butt joint), and the gains themselves are equal power.
    let (mut controller, mut executor) = render_plane();
    let spec = lane_master_spec(
        1.0,
        vec![
            dc_clip(9001, 512, 1_536, 0, 256),
            dc_clip(9002, 1_280, 2_304, 256, 0),
        ],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Blocks 512..1280: A alone at unity (past its start declick).
    let mut frames = [0.0f32; 512];
    for _ in 0..3 {
        executor.render_block(&mut frames);
    }
    assert!((frames[255 * 2] - 1.0).abs() < 1e-5, "pre-overlap unity");

    // Block 1280..1536: the crossfade region.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    for index in 0..256usize {
        let position = index as f32 / 256.0;
        let a = (std::f32::consts::FRAC_PI_2 * (1.0 - position)).sin();
        let b = (std::f32::consts::FRAC_PI_2 * position).sin();
        let rendered = frames[index * 2];
        assert!(
            (rendered - (a + b)).abs() < 1e-4,
            "crossfade frame {index}: {rendered} vs {}",
            a + b
        );
        assert!(
            rendered >= 1.0 - 1e-4,
            "crossfade dipped below unity at frame {index}: {rendered}"
        );
    }

    // Block 1536..1792: B alone at unity.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!((frames[128 * 2] - 1.0).abs() < 1e-5, "post-overlap unity");
}

#[test]
fn fade_envelope_is_continuous_across_block_boundaries() {
    // A fade-in longer than a block and not block-aligned (600 frames
    // from 512): every rendered frame must sit on the analytic curve —
    // the envelope is a pure function of position, so block boundaries
    // (768, 1024) cannot introduce steps.
    let (mut controller, mut executor) = render_plane();
    let spec = lane_master_spec(1.0, vec![dc_clip(9003, 512, 4_096, 600, 0)]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut position = 0u64;
    for _ in 0..3 {
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        for index in 0..256usize {
            let expected = if position + index as u64 >= 600 {
                1.0
            } else {
                (std::f32::consts::FRAC_PI_2 * (position + index as u64) as f32 / 600.0).sin()
            };
            let rendered = frames[index * 2];
            assert!(
                (rendered - expected).abs() < 1e-4,
                "fade frame {}: {rendered} vs {expected}",
                position + index as u64
            );
        }
        position += 256;
    }
}

#[test]
fn seek_into_the_middle_of_a_fade_renders_the_correct_envelope() {
    // The envelope is stateless: seeking into a fade-out span must
    // reproduce the exact curve values with no ramp history.
    let (mut controller, mut executor) = render_plane();
    let spec = lane_master_spec(1.0, vec![dc_clip(9004, 0, 20_000, 0, 8_192)]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    // Fade-out spans [11808, 20000). Seek well inside it.
    controller.seek(12_288).unwrap();
    // Two blocks open the transport edge ramp (240 frames at 48 kHz).
    warm_up(&mut executor, 2);

    // Block 12800..13056, deep inside the fade-out.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    for index in 0..256usize {
        let frame = 12_800 + index as u64;
        let expected = (std::f32::consts::FRAC_PI_2 * (20_000 - frame) as f32 / 8_192.0).sin();
        let rendered = frames[index * 2];
        assert!(
            (rendered - expected).abs() < 1e-4,
            "post-seek fade frame {frame}: {rendered} vs {expected}"
        );
    }
}

#[test]
fn streamed_sources_honor_clip_fades() {
    // The envelope applies to produced frames regardless of source kind:
    // a streamed clip fades in on the same analytic curve.
    let (mut controller, mut executor) = render_plane();
    let total = 4_096u64;
    let (feeder, handle) = render_stream(48_000, total);
    let spec = lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 9005,
            start_frames: 512,
            end_frames: 512 + total,
            source: RenderSource::Stream(handle.clone()),
            loop_source: false,
            fade_in_frames: 512,
            fade_out_frames: 0,
        }],
    );
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    // Feed constant 1.0 for the whole fade span.
    let mut start = 0u64;
    while start < 1_024 {
        let count = 256u64;
        let data = vec![1.0f32; count as usize * 2];
        feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .unwrap();
        start += count;
    }
    warm_up(&mut executor, 2);

    // Blocks 512..768 and 768..1024 cover the 512-frame fade-in.
    let mut position = 0u64;
    for _ in 0..2 {
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        for index in 0..256usize {
            let expected =
                (std::f32::consts::FRAC_PI_2 * (position + index as u64) as f32 / 512.0).sin();
            let rendered = frames[index * 2];
            assert!(
                (rendered - expected).abs() < 1e-4,
                "streamed fade frame {}: {rendered} vs {expected}",
                position + index as u64
            );
        }
        position += 256;
    }
    assert_eq!(handle.underrun_frames(), 0);
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
fn transport_stop_ramps_out_instead_of_stepping() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    controller.set_playing(false).unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    // Ramp-out block: starts audible, ends silent, no step bigger than
    // the tone's own slope plus the ramp slope.
    assert!(frames[0].abs() > 0.0 || frames[2].abs() > 0.0);
    let tail = &frames[1000..];
    assert!(tail.iter().all(|sample| *sample == 0.0));
    let max_step = frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(max_step < 0.05, "stop produced a step of {max_step}");

    // Fully stopped afterwards: silence and a held clock.
    let position = controller.position_frames();
    let mut frames = [1.0f32; 256];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), position);
}

#[test]
fn seek_while_playing_ramps_out_then_jumps() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);
    let before = controller.position_frames();

    controller.seek(96_000).unwrap();
    let mut frames = [0.0f32; 512];
    // Ramp-out block at the old position; seek lands at its end.
    executor.render_block(&mut frames);
    assert_eq!(controller.position_frames(), 96_000);
    let _ = before;
    // Next block plays from the new position, ramping back in.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(controller.position_frames(), 96_000 + 256);
}

#[test]
fn loop_region_rejects_inverted_or_empty_bounds() {
    let (controller, _executor) = render_plane();
    assert!(controller.set_loop_region(Some((100, 100))).is_err());
    assert!(controller.set_loop_region(Some((200, 100))).is_err());
    assert!(controller.set_loop_region(Some((0, 1))).is_ok());
    assert!(controller.set_loop_region(None).is_ok());
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

fn max_left_step(frames: &[f32]) -> f32 {
    frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max)
}

#[test]
fn master_limiter_caps_a_hot_mix_at_the_boundary() {
    let (mut controller, mut executor) = render_plane();
    // Two full-scale tones summed at unity: peaks near 2.0 unlimited.
    let mut spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: Some(RenderLimiterSpec {
            threshold: 0.7,
            knee_width: 0.3,
            release_seconds: 0.05,
        }),
        stages: vec![
            lane_node(1, 1.0, vec![tone_clip(330.0)]),
            lane_node(2, 1.0, vec![tone_clip(331.0)]),
            master_node(vec![identity_edge(1), identity_edge(2)]),
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    let mut frames = [0.0f32; 1024];
    for _ in 0..20 {
        executor.render_block(&mut frames);
        assert!(
            frames.iter().all(|sample| sample.abs() <= 1.0),
            "limited master exceeded 0 dBFS",
        );
    }
    // Without the limiter the same mix clips past 1.0 — prove the test
    // has teeth.
    spec.master_limiter = None;
    controller.install_plan(&spec).unwrap();
    let mut hot = false;
    for _ in 0..20 {
        executor.render_block(&mut frames);
        hot |= frames.iter().any(|sample| sample.abs() > 1.0);
    }
    assert!(hot, "unlimited reference mix never exceeded 1.0");
}

#[test]
fn set_stage_gain_retargets_without_recompile() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Fast path: no install, just a retarget; the smoothing ramp keeps
    // the transition step-free.
    controller.set_stage_gain(LANE_ID, 1.0).unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    let step = max_left_step(&frames);
    assert!(step < 0.08, "fast-path gain stepped audio by {step}");

    // Unknown stage: typed error, callers fall back to install.
    assert!(controller.set_stage_gain(999, 0.5).is_err());
}

#[test]
fn gain_automation_follows_the_envelope_sample_accurately() {
    let (mut controller, mut executor) = render_plane();
    // Constant-amplitude source (DC-ish loopable samples) under a gain
    // ramp envelope 0.0 -> 1.0 over 9600 frames, then hold.
    let values = vec![0.5f32; 480];
    let mut spec = samples_spec(&values, 0, u64::MAX, true);
    spec.stages[0].gain_automation = Some(vec![(0, 0.0), (9_600, 1.0), (19_200, 0.25)]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    // Render 19_200 frames in 256-frame blocks; spot-check the envelope.
    let mut output = Vec::new();
    let mut frames = [0.0f32; 512];
    for _ in 0..75 {
        executor.render_block(&mut frames);
        output.extend(frames.chunks_exact(2).map(|frame| frame[0]));
    }
    // At frame 9_600 the gain is 1.0: sample value 0.5 * 1.0.
    let mid = output[9_600];
    assert!((mid - 0.5).abs() < 0.02, "envelope peak read {mid}");
    // At frame 14_400 (halfway down to 0.25): gain ≈ 0.625.
    let down = output[14_400];
    assert!(
        (down - 0.5 * 0.625).abs() < 0.02,
        "envelope descent read {down}"
    );
    // Monotonic rise across the first segment (block-ramped).
    assert!(output[2_000] < output[4_000] && output[4_000] < output[8_000]);
}

#[test]
fn envelope_swap_mid_play_stays_continuous() {
    let (mut controller, mut executor) = render_plane();
    let values = vec![0.5f32; 480];
    let mut spec = samples_spec(&values, 0, u64::MAX, true);
    spec.stages[0].gain_automation = Some(vec![(0, 1.0)]);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Swap to a very different envelope mid-play: the block ramp anchors
    // at the inherited smoothed gain, so no step.
    let mut louder = samples_spec(&values, 0, u64::MAX, true);
    louder.stages[0].gain_automation = Some(vec![(0, 0.1)]);
    controller.install_plan(&louder).unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    let step = max_left_step(&frames);
    assert!(step < 0.05, "envelope swap stepped audio by {step}");
}

#[test]
fn gain_only_spec_diffs_take_the_fast_path() {
    let base = tone_spec(440.0);
    let mut louder = base.clone();
    louder.stages[0].gain = 0.9;
    assert_eq!(
        base.differs_only_in_gains(&louder),
        Some(vec![(LANE_ID, 0.9)])
    );
    // Structural change: no fast path.
    let mut reshaped = base.clone();
    reshaped.stages[0].gain_automation = Some(vec![(0, 1.0)]);
    assert_eq!(base.differs_only_in_gains(&reshaped), None);
    assert_eq!(base.differs_only_in_gains(&base), Some(vec![]));
}

#[test]
fn mid_lane_clip_insert_preserves_neighbour_state() {
    // A tone clip keeps its phase when a new clip is inserted BEFORE it
    // in the lane's clip list — the clip-id inheritance map prevents the
    // zip-index cross-wiring the old code had.
    let (mut controller, mut executor) = render_plane();
    let survivor = RenderClipSpec {
        clip_id: 7,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::TestTone {
            frequency_hz: 440.0,
        },
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    };
    controller
        .install_plan(&lane_master_spec(0.5, vec![survivor.clone()]))
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Insert a silent clip at index 0; survivor moves to index 1.
    let inserted = RenderClipSpec {
        clip_id: 8,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::Silence,
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    };
    controller
        .install_plan(&lane_master_spec(0.5, vec![inserted, survivor]))
        .unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    // Phase carried: the 440 Hz tone continues without a step.
    let step = max_left_step(&frames);
    assert!(step < 0.05, "clip insert stepped audio by {step}");
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
}

#[test]
fn stage_reorder_preserves_state_through_the_identity_map() {
    // Two tone lanes swap positions in the stage list across a plan
    // swap; both keep phase and smoothed gain (no audible step).
    let (mut controller, mut executor) = render_plane();
    let lane_a = lane_node(10, 0.4, vec![tone_clip(330.0)]);
    let lane_b = lane_node(11, 0.4, vec![tone_clip(550.0)]);
    let master = master_node(vec![identity_edge(10), identity_edge(11)]);
    controller
        .install_plan(&RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![lane_a.clone(), lane_b.clone(), master.clone()],
        })
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    controller
        .install_plan(&RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![lane_b, lane_a, master],
        })
        .unwrap();
    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    let step = max_left_step(&frames);
    // Two tones at 0.4 gain: their combined slope stays well under this
    // bound only if both phases carried.
    assert!(step < 0.07, "stage reorder stepped audio by {step}");
}

#[test]
fn plan_churn_keeps_a_surviving_tone_continuous() {
    // Property-style: a seeded LCG drives 24 plan installs mid-play —
    // adding/removing extra lanes, inserting silent clips around the
    // survivor, jittering other lanes' gains. The surviving tone lane
    // must never step.
    let (mut controller, mut executor) = render_plane();
    let survivor_clip = RenderClipSpec {
        clip_id: 1,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::TestTone {
            frequency_hz: 440.0,
        },
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    };
    let mut seed: u64 = 0x5EED_CAFE;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };
    let build = |extra_lanes: u32, clips_before: u32, extra_gain: f32| -> RenderPlanSpec {
        let mut clips = Vec::new();
        for index in 0..clips_before {
            clips.push(RenderClipSpec {
                clip_id: 100 + index as u64,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Silence,
                loop_source: false,
                fade_in_frames: 0,
                fade_out_frames: 0,
            });
        }
        clips.push(survivor_clip.clone());
        let mut stages = vec![lane_node(LANE_ID, 0.5, clips)];
        let mut edges = vec![identity_edge(LANE_ID)];
        for index in 0..extra_lanes {
            let stage_id = 50 + index as u64;
            // Extra lanes are silent so the survivor's continuity is the
            // only signal under test.
            stages.push(lane_node(stage_id, extra_gain, vec![]));
            edges.push(identity_edge(stage_id));
        }
        stages.push(master_node(edges));
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages,
        }
    };
    controller.install_plan(&build(0, 0, 0.3)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut worst_step = 0.0f32;
    let mut previous_tail = None::<f32>;
    for _ in 0..24 {
        let extra = next() % 4;
        let before = next() % 3;
        let gain = (next() % 100) as f32 / 100.0;
        controller
            .install_plan(&build(extra, before, gain))
            .unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        if let Some(tail) = previous_tail {
            worst_step = worst_step.max((frames[0] - tail).abs());
        }
        worst_step = worst_step.max(max_left_step(&frames));
        previous_tail = Some(frames[frames.len() - 2]);
    }
    assert!(
        worst_step < 0.05,
        "plan churn stepped the surviving tone by {worst_step}",
    );
}

#[test]
fn plan_swap_inherits_smoothed_gain_without_stepping() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Same plan with lane gain doubled: swap mid-play. Stage ids are
    // stable, so the smoothed gain carries over and ramps.
    let louder = lane_master_spec(1.0, vec![tone_clip(440.0)]);
    controller.install_plan(&louder).unwrap();

    let mut frames = [0.0f32; 1024];
    executor.render_block(&mut frames);
    let max_step = frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max);
    // 440 Hz at 48k moves at most ~0.058/sample at unity; the gain ramp
    // must not add a visible step on top.
    assert!(max_step < 0.08, "gain swap produced a step of {max_step}");
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

// ── Plugin processors (g11.012) ─────────────────────────────────────

/// Fake in-process backend: multiplies every sample by `gain` and counts
/// calls. Stands in for a live plugin without any child process.
struct FakeGainProcessor {
    gain: f32,
    calls: AtomicU64,
}

impl PluginBlockProcessor for FakeGainProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.calls.fetch_add(1, Ordering::Relaxed);
        for sample in &mut scratch[..frame_count * channels] {
            *sample *= self.gain;
        }
        true
    }
}

/// Fake backend that always misses: returns `false` and must leave the
/// scratch untouched (the bypass contract under test).
struct AlwaysMissProcessor {
    misses: AtomicU64,
}

impl PluginBlockProcessor for AlwaysMissProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.misses.fetch_add(1, Ordering::Relaxed);
        false
    }
}

/// Minimal alloc-free instrument backend: note-on starts a constant
/// signal at the event velocity; note-off returns to silence.
struct EventInstrumentProcessor {
    amplitude_bits: AtomicU32,
}

impl EventInstrumentProcessor {
    fn render(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        let mut amplitude = f32::from_bits(self.amplitude_bits.load(Ordering::Relaxed));
        let mut event_index = 0;
        for frame in 0..frame_count {
            while event_index < events.len() && events[event_index].offset_frames as usize == frame
            {
                amplitude = match events[event_index].kind {
                    RenderPluginEventKind::NoteOn { velocity, .. } => velocity,
                    RenderPluginEventKind::NoteOff { .. } => 0.0,
                    RenderPluginEventKind::ControlChange { .. }
                    | RenderPluginEventKind::PitchBend { .. }
                    | RenderPluginEventKind::ChannelPressure { .. }
                    | RenderPluginEventKind::NoteExpression { .. }
                    | RenderPluginEventKind::VoiceStart { .. }
                    | RenderPluginEventKind::VoiceStop { .. }
                    | RenderPluginEventKind::VoiceParam { .. } => amplitude,
                };
                event_index += 1;
            }
            for channel in 0..channels {
                scratch[frame * channels + channel] = amplitude;
            }
        }
        self.amplitude_bits
            .store(amplitude.to_bits(), Ordering::Relaxed);
        true
    }
}

impl PluginBlockProcessor for EventInstrumentProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.render(scratch, frame_count, channels, &[])
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.render(scratch, frame_count, channels, events)
    }
}

/// Constant-content plan with a Sum insert stage carrying `processor`.
fn processor_spec(processor: Option<RenderPluginProcessor>) -> RenderPlanSpec {
    let values = vec![0.5f32; 480];
    let mut data = Vec::new();
    for value in &values {
        data.push(*value);
        data.push(*value);
    }
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(
                LANE_ID,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 2001,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                    loop_source: true,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor,
                events: None,
                stage_id: 77,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(77)]),
        ],
    }
}

fn impulse_delay_spec(delay_frames: u32) -> RenderPlanSpec {
    let mut data = vec![0.0f32; 512 * 2];
    data[100 * 2] = 1.0;
    data[100 * 2 + 1] = 1.0;
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(
                LANE_ID,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 2002,
                    start_frames: 0,
                    end_frames: 512,
                    source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                    loop_source: false,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 78,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Delay {
                    frames: delay_frames,
                },
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(78)]),
        ],
    }
}

#[test]
fn delay_stage_moves_an_impulse_by_exact_stream_frames() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&impulse_delay_spec(7)).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = [0.0f32; 256 * 2];
    executor.render_block(&mut frames);

    assert_eq!(frames[100 * 2], 0.0);
    assert!(frames[107 * 2] > 0.0);
    assert_eq!(frames[107 * 2], frames[107 * 2 + 1]);
    assert_eq!(frames[106 * 2], 0.0);
    assert_eq!(frames[108 * 2], 0.0);
}

#[test]
fn delay_stage_carries_ring_state_across_plan_swap() {
    let (mut controller, mut executor) = render_plane();
    let spec = impulse_delay_spec(300);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut first = [0.0f32; 256 * 2];
    executor.render_block(&mut first);
    assert!(first.iter().all(|sample| *sample == 0.0));

    controller.install_plan(&spec).unwrap();
    let mut second = [0.0f32; 256 * 2];
    executor.render_block(&mut second);
    assert_eq!(second[144 * 2], 1.0);
    assert_eq!(second[144 * 2 + 1], 1.0);
}

#[test]
fn sum_stage_processor_transforms_the_summed_scratch() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(FakeGainProcessor {
        gain: 0.5,
        calls: AtomicU64::new(0),
    });
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&processor_spec(Some(handle)))
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Past the edge ramp and clip fade: 0.5 content × 0.5 plugin gain.
    assert!(
        (frames[100 * 2] - 0.25).abs() < 1e-5,
        "processed sample read {}",
        frames[100 * 2],
    );
    assert!(backend.calls.load(Ordering::Relaxed) > 0);
}

#[test]
fn processor_miss_bypasses_and_counts_without_touching_scratch() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    });
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&processor_spec(Some(handle)))
        .unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 4);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Bypass: dry content flows through the insert untouched.
    assert!(
        (frames[100 * 2] - 0.5).abs() < 1e-5,
        "bypassed sample read {}",
        frames[100 * 2],
    );
    assert!(backend.misses.load(Ordering::Relaxed) > 0);
}

#[test]
fn processor_absent_and_bypassed_render_identically() {
    let render = |processor: Option<RenderPluginProcessor>| -> Vec<f32> {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&processor_spec(processor)).unwrap();
        controller.set_playing(true).unwrap();
        let mut collected = Vec::new();
        let mut frames = [0.0f32; 512];
        for _ in 0..8 {
            executor.render_block(&mut frames);
            collected.extend_from_slice(&frames);
        }
        collected
    };
    let dry = render(None);
    let bypassed = render(Some(RenderPluginProcessor::new(Arc::new(
        AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        },
    ))));
    assert_eq!(dry, bypassed, "bypass must be bit-identical to absent");
}

#[test]
fn compile_rejects_processors_on_non_sum_stages() {
    let handle = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let mut spec = tone_spec(440.0);
    spec.stages[0].processor = Some(handle);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("not a Sum stage"), "{error}");
}

#[test]
fn processor_swap_is_structural_not_a_gain_fast_path() {
    let handle_a = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let handle_b = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
        misses: AtomicU64::new(0),
    }));
    let with_a = processor_spec(Some(handle_a.clone()));
    // Clone (same sample-buffer Arc) so only the processor may differ.
    let with_a_again = with_a.clone();
    let mut with_b = with_a.clone();
    with_b.stages[1].processor = Some(handle_b);
    let _ = handle_a;
    // Same handle: gain-only diff logic sees no change.
    assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
    // Different handle (same everything else): structural.
    assert_eq!(with_a.differs_only_in_gains(&with_b), None);
}

// ── Plugin event delivery (g12.034 follow-up) ───────────────────────

/// Recording backend: captures the event slice of every
/// `process_with_events` call. A bare `process` call records a sentinel
/// (`offset_frames == u32::MAX`) — stages carrying an event buffer must
/// never take that path.
struct RecordingEventProcessor {
    calls: std::sync::Mutex<Vec<Vec<RenderBlockPluginEvent>>>,
}

impl RecordingEventProcessor {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: std::sync::Mutex::new(Vec::new()),
        })
    }

    fn calls(&self) -> Vec<Vec<RenderBlockPluginEvent>> {
        self.calls.lock().unwrap().clone()
    }
}

impl PluginBlockProcessor for RecordingEventProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.calls
            .lock()
            .unwrap()
            .push(vec![RenderBlockPluginEvent {
                offset_frames: u32::MAX,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 0 },
            }]);
        true
    }

    fn process_with_events(
        &self,
        _scratch: &mut [f32],
        _frames: usize,
        _channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.calls.lock().unwrap().push(events.to_vec());
        true
    }
}

fn event_buffer(events: Vec<RenderPluginEvent>) -> RenderPluginEventBuffer {
    RenderPluginEventBuffer {
        events: events.into(),
    }
}

/// `processor_spec` with an event stream on the insert stage.
fn events_spec(handle: RenderPluginProcessor, events: RenderPluginEventBuffer) -> RenderPlanSpec {
    let mut spec = processor_spec(Some(handle));
    spec.stages[1].events = Some(events);
    spec
}

#[test]
fn processor_stage_delivers_events_at_intra_block_sample_offsets() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        },
        RenderPluginEvent {
            frame: 519,
            channel: 0,
            kind: RenderPluginEventKind::ControlChange {
                controller: 74,
                value: 0.33,
            },
        },
        RenderPluginEvent {
            frame: 700,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 64 },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Two 512-frame blocks from position 0.
    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert_eq!(calls.len(), 2, "one delivery per rendered block");
    assert_eq!(
        calls[0],
        vec![RenderBlockPluginEvent {
            offset_frames: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        }],
        "block 1 carries the note-on at its absolute frame",
    );
    assert_eq!(
        calls[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 7,
                channel: 0,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.33,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 188,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ],
        "block 2 events land at frame − block start",
    );
}

#[test]
fn hosted_instrument_events_generate_audio_from_a_silent_lane() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(EventInstrumentProcessor {
        amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
    });
    let handle = RenderPluginProcessor::new(backend as Arc<_>);
    let events = event_buffer(vec![
        RenderPluginEvent {
            frame: 64,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.5,
            },
        },
        RenderPluginEvent {
            frame: 320,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 60 },
        },
    ]);
    let mut spec = events_spec(handle, events);
    let RenderStageKind::Source { clips } = &mut spec.stages[0].kind else {
        panic!("fixture lane source");
    };
    let RenderSource::Samples(samples) = &mut clips[0].source else {
        panic!("fixture sample source");
    };
    samples.frames = vec![0.0; samples.frames.len()].into();
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 512 * 2];
    executor.render_block(&mut frames);
    assert!(frames[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(frames[96 * 2..256 * 2].iter().all(|sample| *sample > 0.0));
    assert!(frames[320 * 2..].iter().all(|sample| *sample == 0.0));
    assert!(controller.meters().iter().any(|(_, peak, _)| *peak > 0.0));

    let offline = crate::offline::render_plan_to_pcm(
        &spec,
        &crate::offline::OfflineRenderOptions {
            start_frame: 0,
            frame_count: 512,
            block_frames: 128,
            capture_stage_ids: Vec::new(),
        },
    )
    .expect("offline hosted instrument render");
    assert!(offline.master[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(offline.master[96 * 2..256 * 2]
        .iter()
        .all(|sample| *sample > 0.0));
    assert!(offline.master[320 * 2..]
        .iter()
        .all(|sample| *sample == 0.0));
}

#[test]
fn event_delivery_is_playback_gated() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 60,
            velocity: 1.0,
        },
    }]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_playing(true).unwrap();
    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    // Stop: the edge ramp keeps rendering blocks briefly, but the
    // position no longer advances — re-delivering the same events would
    // double-trigger notes, so delivery gates on playback.
    controller.set_playing(false).unwrap();
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert!(calls.len() >= 2, "ramp-out still processes audio");
    assert_eq!(calls[0].len(), 1, "playing block delivers");
    for call in &calls[1..] {
        assert!(call.is_empty(), "stopped blocks must deliver no events");
    }
}

#[test]
fn seek_chases_held_plugin_note_controller_and_expression_state() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::ControlChange {
                controller: 74,
                value: 0.25,
            },
        },
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::PitchBend { value: 0.4 },
        },
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
        },
        RenderPluginEvent {
            frame: 100,
            channel: 2,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Pressure,
                value: 0.7,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Timbre,
                value: 0.8,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Tuning,
                value: 12.0,
            },
        },
        RenderPluginEvent {
            frame: 500,
            channel: 2,
            kind: RenderPluginEventKind::NoteOff { key: 64 },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.seek(300).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    assert_eq!(
        backend.calls()[0],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.25,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::PitchBend { value: 0.4 },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteOn {
                    key: 64,
                    velocity: 0.75,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Pressure,
                    value: 0.7,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.8,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Tuning,
                    value: 12.0,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 200,
                channel: 2,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ],
    );
}

#[test]
fn loop_wrap_delivers_both_segments_with_buffer_relative_offsets() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.5,
            },
        },
        RenderPluginEvent {
            frame: 550,
            channel: 1,
            kind: RenderPluginEventKind::ControlChange {
                controller: 1,
                value: 1.0,
            },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_loop_region(Some((0, 600))).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // [0, 512): note at 100
    executor.render_block(&mut frames); // [512, 600) + wrap [0, 424)

    let calls = backend.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].len(), 1);
    assert_eq!(calls[0][0].offset_frames, 100);
    assert_eq!(
        calls[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 38, // 550 − 512, first segment
                channel: 1,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 1,
                    value: 1.0,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 88, // wrap: release note active at loop end
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 60 },
            },
            RenderBlockPluginEvent {
                offset_frames: 188, // 88 wrap offset + frame 100
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            },
        ],
        "wrapped block delivers both segments, buffer-relative",
    );
}

#[test]
fn loop_wrap_chases_held_note_controller_and_expression_at_wrap_offset() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 50,
            channel: 3,
            kind: RenderPluginEventKind::PitchBend { value: -0.25 },
        },
        RenderPluginEvent {
            frame: 100,
            channel: 3,
            kind: RenderPluginEventKind::NoteOn {
                key: 67,
                velocity: 0.9,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 3,
            kind: RenderPluginEventKind::NoteExpression {
                key: 67,
                expression: RenderNoteExpressionKind::Timbre,
                value: 0.45,
            },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_loop_region(Some((300, 600))).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // [0, 512)
    executor.render_block(&mut frames); // [512, 600) + wrap [300, 724)

    assert_eq!(
        backend.calls()[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteOff { key: 67 },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::PitchBend { value: -0.25 },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteOn {
                    key: 67,
                    velocity: 0.9,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 67,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.45,
                },
            },
        ],
    );
}

#[test]
fn compile_rejects_events_without_processor_and_unsorted_events() {
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    }]);
    let mut spec = processor_spec(None);
    spec.stages[1].events = Some(buffer);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error.message.contains("without a plugin processor"),
        "{error}"
    );

    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let unsorted = event_buffer(vec![
        RenderPluginEvent {
            frame: 10,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
        RenderPluginEvent {
            frame: 5,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
    ]);
    let spec = events_spec(handle, unsorted);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("not sorted by frame"), "{error}");
}

#[test]
fn event_buffer_swap_is_structural_not_a_gain_fast_path() {
    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let event = RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    };
    let with_a = events_spec(handle, event_buffer(vec![event]));
    // Clone shares the Arc: gain-only diff logic sees no change.
    let with_a_again = with_a.clone();
    assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
    // A rebuilt buffer (same content, new Arc) is structural.
    let mut with_b = with_a.clone();
    with_b.stages[1].events = Some(event_buffer(vec![event]));
    assert_eq!(with_a.differs_only_in_gains(&with_b), None);
}

#[test]
fn sample_buffers_compare_by_pointer_for_cheap_spec_equality() {
    let data: Arc<[f32]> = vec![0.0f32; 8].into();
    let a = RenderSampleBuffer::stereo(48_000, Arc::clone(&data));
    let b = RenderSampleBuffer::stereo(48_000, data);
    let c = RenderSampleBuffer::stereo(48_000, vec![0.0f32; 8].into());
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ── Disk-streaming sources ──────────────────────────────────────────────

/// Spec with one stream clip windowed `[start, end)` at lane gain 1.
fn stream_spec(handle: &RenderStreamHandle, start_frames: u64, end_frames: u64) -> RenderPlanSpec {
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1006,
            start_frames,
            end_frames,
            source: RenderSource::Stream(handle.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Feed `[from, to)` of a ramp (value = frame / total) in fixed chunks.
fn feed_ramp(feeder: &StreamFeeder, total: u64, from: u64, to: u64, chunk_frames: u64) {
    let mut start = from - from % chunk_frames;
    while start < to.min(total) {
        let count = chunk_frames.min(total - start);
        let mut data = Vec::with_capacity(count as usize * 2);
        for frame in start..start + count {
            let value = frame as f32 / total as f32;
            data.push(value);
            data.push(value);
        }
        if feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .is_err()
        {
            return; // Mailbox full: enough read-ahead for the test.
        }
        start += count;
    }
}

#[test]
fn stream_clips_play_fed_chunks_sample_accurately() {
    let (mut controller, mut executor) = render_plane();
    let total = 4_096u64;
    let (feeder, handle) = render_stream(48_000, total);
    // Window starts at frame 512, well past the edge ramp warm-up.
    let spec = stream_spec(&handle, 512, 512 + total);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    feed_ramp(&feeder, total, 0, 1_024, 256);

    // Two 256-frame blocks open the edge ramp and reach frame 512.
    warm_up(&mut executor, 2);
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // Frame 512+128 plays source frame 128, past the clip edge fade.
    let index = 128usize;
    let expected = 128.0 / total as f32;
    assert!((frames[index * 2] - expected).abs() < 1e-6);
    // 1:1 streaming: identical channels, zero underruns.
    assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    assert_eq!(handle.underrun_frames(), 0);
    // The next block starts past the clip anchor: the read hint follows.
    executor.render_block(&mut frames);
    assert_eq!(feeder.wanted_frame(), 256);
}

#[test]
fn stream_underruns_render_silence_and_count() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_stream(48_000, 48_000);
    controller
        .install_plan(&stream_spec(&handle, 0, 48_000))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Nothing fed: every in-window frame is an underrun, output silent.
    let mut frames = [0.1f32; 512];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(handle.underrun_frames(), 512);

    // Feed the region the executor wants: audio resumes, count holds.
    feed_ramp(
        &feeder,
        48_000,
        feeder.wanted_frame(),
        feeder.wanted_frame() + 2_048,
        512,
    );
    let before = handle.underrun_frames();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.001));
    assert_eq!(handle.underrun_frames(), before);
}

#[test]
fn stream_seek_retires_stale_chunks_and_resumes_once_fed() {
    let (mut controller, mut executor) = render_plane();
    let total = 1_000_000u64;
    let (feeder, handle) = render_stream(48_000, total);
    controller
        .install_plan(&stream_spec(&handle, 0, total))
        .unwrap();
    controller.set_playing(true).unwrap();
    feed_ramp(&feeder, total, 0, 1_024, 256);
    warm_up(&mut executor, 3);
    assert_eq!(handle.underrun_frames(), 0);

    // Seek far past the retire lookahead: held chunks for the old
    // region must come back via the retired mailbox.
    let target = 600_000u64;
    controller.seek(target).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames); // Ramp-out block; seek lands.
    executor.render_block(&mut frames); // First block at the new region.
    assert!(feeder.wanted_frame() >= target);
    // Old-region chunks retire within a few blocks (stale arrivals can
    // sit one block in a held slot first).
    let mut retired = Vec::new();
    for _ in 0..4 {
        retired.extend(feeder.collect_retired());
        executor.render_block(&mut frames);
    }
    retired.extend(feeder.collect_retired());
    assert!(
        retired.iter().all(|chunk| chunk.start_frame < 1_024),
        "only old-region chunks should retire",
    );
    assert!(!retired.is_empty(), "stale chunks should have retired");

    // Feed the new region: playback resumes with the right content.
    let wanted = feeder.wanted_frame();
    feed_ramp(&feeder, total, wanted, wanted + 4_096, 512);
    let before = handle.underrun_frames();
    assert!(before > 0, "seek without data should have underrun");
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), before);
    let position = controller.position_frames() - 256;
    let expected = position as f32 / total as f32;
    assert!(
        (frames[0] - expected).abs() < 1e-5,
        "resumed at the wrong content"
    );
}

#[test]
fn rate_converted_streams_play_through_the_sinc_path() {
    // 1 kHz sine at 44.1k streamed onto a 48k plan: same SNR bar as the
    // in-memory rate-converted test — proof the stream path shares the
    // polyphase interpolation.
    let (mut controller, mut executor) = render_plane();
    let source_rate = 44_100u32;
    let total = 44_100u64;
    let frequency = 1_000.0f64;
    let (feeder, handle) = render_stream(source_rate, total);
    controller
        .install_plan(&stream_spec(&handle, 0, u64::MAX))
        .unwrap();
    controller.set_playing(true).unwrap();
    // Feed the whole second up front in 8 large chunks.
    let chunk_frames = total.div_ceil(8);
    let mut start = 0u64;
    while start < total {
        let count = chunk_frames.min(total - start);
        let mut data = Vec::with_capacity(count as usize * 2);
        for n in start..start + count {
            let value =
                (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
            data.push(value);
            data.push(value);
        }
        feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .unwrap();
        start += count;
    }
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
    assert!(snr > 60.0, "rate-converted stream SNR {snr:.1} dB");
    assert_eq!(handle.underrun_frames(), 0);
}

#[test]
fn plan_swap_mid_stream_keeps_held_chunks_without_underrun() {
    let (mut controller, mut executor) = render_plane();
    let total = 48_000u64;
    let (feeder, handle) = render_stream(48_000, total);
    let spec = stream_spec(&handle, 0, total);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    // Feed only what fits in the mailbox + held slots; after the swap no
    // further feeding happens, so continuity proves the held chunks
    // moved across the plan boundary via the clip inheritance map.
    feed_ramp(&feeder, total, 0, 2_048, 256);
    warm_up(&mut executor, 2); // 512 frames consumed, chunks held.

    // Identity recompile mid-stream (the handle is pointer-equal, so
    // the spec is too — hosts would skip this install; force it).
    controller.install_plan(&spec.clone()).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), 0, "swap dropped held chunks");
    // Content continues exactly: frame 512 plays source frame 512.
    let expected = 512.0 / total as f32;
    assert!((frames[0] - expected).abs() < 1e-6);
}

// ── Live input monitor sources ──────────────────────────────────────────

/// Spec with one live-input clip windowed `[0, u64::MAX)` at `lane_gain`.
fn live_input_spec(handle: &RenderLiveInputHandle, lane_gain: f32) -> RenderPlanSpec {
    lane_master_spec(
        lane_gain,
        vec![RenderClipSpec {
            clip_id: 1007,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::LiveInput(handle.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Push `count` stereo frames of a ramp starting at `value_base`
/// (value = (value_base + i) / 10_000) and return the next base.
fn push_ramp(feeder: &LiveInputFeeder, value_base: u64, count: usize) -> u64 {
    let mut data = Vec::with_capacity(count * 2);
    for index in 0..count {
        let value = (value_base + index as u64) as f32 / 10_000.0;
        data.push(value);
        data.push(value);
    }
    assert_eq!(feeder.push_slice(&data), count, "test ring overflowed");
    value_base + count as u64
}

#[test]
fn live_input_clips_render_pushed_audio_through_the_chain() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    // Lane gain 0.5: the chain's fader applies to monitored input like
    // any other source.
    controller
        .install_plan(&live_input_spec(&handle, 0.5))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Feed exactly one block per render so content is deterministic.
    let mut base = 0u64;
    let mut frames = [0.0f32; 512];
    // Warm-up: edge ramp opens, clip edge fade passes.
    for _ in 0..2 {
        base = push_ramp(&feeder, base, 256);
        executor.render_block(&mut frames);
    }
    assert_eq!(handle.underrun_frames(), 0);

    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    // Block 2 renders pushed frames 512..768 at lane gain 0.5.
    for index in [0usize, 100, 255] {
        let expected = (512 + index) as f32 / 10_000.0 * 0.5;
        assert!(
            (frames[index * 2] - expected).abs() < 1e-6,
            "frame {index}: {} vs {expected}",
            frames[index * 2],
        );
        // Stereo feed: identical channels.
        assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    }
    assert_eq!(handle.underrun_frames(), 0);
    let _ = base;
}

#[test]
fn live_input_underruns_render_silence_and_count() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    controller
        .install_plan(&live_input_spec(&handle, 1.0))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Nothing fed: the whole block underruns, output stays silent.
    let mut frames = [0.1f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(handle.underrun_frames(), 256);

    // Half a block fed: 128 frames render, 128 count as underrun.
    push_ramp(&feeder, 50_000, 128);
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), 256 + 128);
    assert!(frames.iter().any(|sample| sample.abs() > 0.001));

    // Fed again: audio resumes, the count holds.
    push_ramp(&feeder, 50_128, 256);
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), 256 + 128);
}

#[test]
fn live_input_survives_plan_swaps_without_dropping_audio() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    let spec = live_input_spec(&handle, 1.0);
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut base = 0u64;
    let mut frames = [0.0f32; 512];
    for _ in 0..2 {
        base = push_ramp(&feeder, base, 256);
        executor.render_block(&mut frames);
    }

    // Recompile mid-feed (identity spec — pointer-equal handle keeps the
    // spec equal too; force the install). The ring lives in the shared
    // handle, so the feed continues without a gap or reset.
    controller.install_plan(&spec.clone()).unwrap();
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    assert_eq!(handle.underrun_frames(), 0, "swap dropped live audio");
    // Content continues exactly: first frame plays pushed frame 512.
    let expected = 512.0 / 10_000.0;
    assert!((frames[0] - expected).abs() < 1e-6);
    let _ = base;
}

#[test]
fn live_input_trims_stale_backlog_to_bound_monitoring_latency() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    controller
        .install_plan(&live_input_spec(&handle, 1.0))
        .unwrap();

    // Feeder ran while the executor was stopped: a deep stale backlog.
    let mut base = 0u64;
    for _ in 0..16 {
        base = push_ramp(&feeder, base, 256); // 4_096 frames total.
    }
    controller.set_playing(true).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    // The executor discarded down to span + LIVE_INPUT_MAX_BACKLOG and
    // rendered from there — near the END of the pushed data, not frame
    // 0. Sample index 250 sits past both the transport edge ramp
    // (240 frames at 48 kHz) and the clip edge fade, so the raw pushed
    // value reads back unscaled.
    let value = f64::from(frames[250 * 2]) * 10_000.0;
    let cutoff = (4_096 - 256 - LIVE_INPUT_MAX_BACKLOG_FRAMES) as f64;
    assert!(
        value >= cutoff,
        "stale backlog replayed pushed frame {value} (cutoff {cutoff})",
    );
    // What remains buffered is bounded (latency stays shallow).
    assert!(
        handle.buffered_frames() <= LIVE_INPUT_MAX_BACKLOG_FRAMES,
        "backlog left at {} frames",
        handle.buffered_frames(),
    );
    let _ = base;
}

#[test]
fn live_input_handles_compare_by_pointer_for_cheap_spec_equality() {
    let (_feeder_a, a) = render_live_input(1_024);
    let b = a.clone();
    let (_feeder_c, c) = render_live_input(1_024);
    assert_eq!(a, b);
    assert_ne!(a, c);
    // Spec equality follows handle equality: idempotent recompiles.
    assert_eq!(live_input_spec(&a, 1.0), live_input_spec(&b, 1.0));
    assert_ne!(live_input_spec(&a, 1.0), live_input_spec(&c, 1.0));
}

// ── Live render posture + live event injection (g13.018) ───────────────

const LIVE_INSERT_ID: u64 = 77;

/// Silent lane into a Sum instrument stage that accepts live events.
fn live_instrument_spec(handle: RenderPluginProcessor) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(LANE_ID, 1.0, Vec::new()),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: true,
                processor: Some(handle),
                events: None,
                stage_id: LIVE_INSERT_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(LIVE_INSERT_ID)]),
        ],
    }
}

fn live_note_on(frame: u64, key: u8, velocity: f32) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn { key, velocity },
    }
}

fn live_note_off(frame: u64, key: u8) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key },
    }
}

#[test]
fn compile_rejects_live_event_flag_without_processor_and_gain_fast_path_treats_it_structural() {
    let (mut controller, _executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut spec = live_instrument_spec(handle);
    spec.stages[1].processor = None;
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error
            .message
            .contains("accepts live events without a plugin processor"),
        "{error}",
    );

    // Non-Sum stages cannot accept live events either (no processor is
    // even representable there).
    let mut lane_flagged = tone_spec(440.0);
    lane_flagged.stages[0].accepts_live_events = true;
    assert!(controller.install_plan(&lane_flagged).is_err());

    // Flipping the flag is a structural change, never a gain fast path.
    let with_flag = tone_spec(440.0);
    let mut without_flag = with_flag.clone();
    without_flag.stages[0].accepts_live_events = false;
    let mut flagged = with_flag.clone();
    flagged.stages[0].accepts_live_events = true;
    assert_eq!(without_flag.differs_only_in_gains(&flagged), None);
}

#[test]
fn push_live_events_validates_stage_identity_and_flag() {
    let (mut controller, _executor) = render_plane();
    let events = [live_note_on(0, 60, 0.5)];
    assert!(
        controller
            .push_live_events(LIVE_INSERT_ID, &events)
            .is_err(),
        "push without an installed plan must error",
    );

    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();
    assert!(
        controller.push_live_events(9_999, &events).is_err(),
        "unknown stage must error",
    );
    assert!(
        controller.push_live_events(LANE_ID, &events).is_err(),
        "stage without accepts_live_events must error",
    );
    controller
        .push_live_events(LIVE_INSERT_ID, &events)
        .expect("accepting stage takes the push");
    controller
        .push_live_events(LIVE_INSERT_ID, &[])
        .expect("empty push is a no-op");
}

#[test]
fn live_events_sound_through_a_hosted_instrument_while_transport_is_stopped() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(EventInstrumentProcessor {
        amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
    });
    let handle = RenderPluginProcessor::new(backend as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();

    // Stopped, posture off: the render gate silences everything.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture on, still stopped: a pushed note sounds.
    controller.set_live_render(true).unwrap();
    controller
        .push_live_events(LIVE_INSERT_ID, &[live_note_on(0, 60, 0.5)])
        .unwrap();
    executor.render_block(&mut frames); // Edge envelope ramps in.
    assert!(controller.live_render());
    executor.render_block(&mut frames);
    assert!(
        frames.iter().all(|sample| (*sample - 0.5).abs() < 1e-3),
        "held live note renders at its velocity while stopped",
    );
    // Meters publish as normal under the posture.
    assert!(
        controller
            .meters()
            .iter()
            .any(|(id, peak, _)| *id == LIVE_INSERT_ID && *peak > 0.4),
        "instrument stage meters while stopped: {:?}",
        controller.meters(),
    );
    // The transport position never advanced.
    assert_eq!(controller.position_frames(), 0);

    // A note-off with a stale (past) frame clamps to "now" and stops
    // the voice.
    controller
        .push_live_events(LIVE_INSERT_ID, &[live_note_off(0, 60)])
        .unwrap();
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture off again: back to the silent early return.
    controller.set_live_render(false).unwrap();
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert!(!controller.live_render());
    assert_eq!(controller.position_frames(), 0);
}

#[test]
fn live_and_compiled_events_merge_ordered_by_offset_while_playing() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 519,
            channel: 0,
            kind: RenderPluginEventKind::ControlChange {
                controller: 74,
                value: 0.33,
            },
        },
        RenderPluginEvent {
            frame: 700,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 64 },
        },
    ]);
    let mut spec = events_spec(handle, buffer);
    spec.stages[1].accepts_live_events = true;
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // Block 1: frames 0..512.

    // Before block 2 (frames 512..1024): one live event already in the
    // past (clamps to offset 0) and one inside the block (offset 88).
    controller
        .push_live_events(
            LIVE_INSERT_ID,
            &[live_note_on(200, 1, 0.9), live_note_on(600, 2, 0.8)],
        )
        .unwrap();
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert_eq!(calls.len(), 2);
    let offsets: Vec<u32> = calls[1].iter().map(|event| event.offset_frames).collect();
    assert_eq!(
        offsets,
        vec![0, 7, 88, 188],
        "live + compiled events interleave sorted by in-block offset",
    );
    assert_eq!(
        calls[1][0].kind,
        RenderPluginEventKind::NoteOn {
            key: 1,
            velocity: 0.9,
        },
        "past live event clamps to offset 0",
    );
    assert_eq!(
        calls[1][2].kind,
        RenderPluginEventKind::NoteOn {
            key: 2,
            velocity: 0.8,
        },
    );
}

#[test]
fn live_event_ring_overflow_drops_and_counts() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();
    controller.set_live_render(true).unwrap();

    let flood: Vec<RenderPluginEvent> = (0..(LIVE_EVENT_RING_CAPACITY as u64 + 32))
        .map(|index| live_note_on(index, (index % 128) as u8, 0.5))
        .collect();
    controller.push_live_events(LIVE_INSERT_ID, &flood).unwrap();
    assert_eq!(controller.live_event_drop_count(), 0);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(
        controller.live_event_drop_count(),
        32,
        "events past the ring capacity drop and count",
    );
    let calls = backend.calls();
    assert_eq!(
        calls.last().unwrap().len(),
        LIVE_EVENT_RING_CAPACITY,
        "the ring's worth of events delivers this block",
    );

    // The ring drained: the next block has no pending live events, so
    // the stage takes the plain (event-less) processing path — the
    // recording backend marks that with its sentinel entry.
    executor.render_block(&mut frames);
    assert_eq!(
        backend.calls().last().unwrap().as_slice(),
        &[RenderBlockPluginEvent {
            offset_frames: u32::MAX,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        }],
    );
    assert_eq!(controller.live_event_drop_count(), 32);
}

#[test]
fn live_input_monitoring_passes_while_stopped_under_live_render() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    controller
        .install_plan(&live_input_spec(&handle, 1.0))
        .unwrap();

    // Stopped, posture off (the g11.010 limit): silence.
    let mut base = push_ramp(&feeder, 0, 256);
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture on, still stopped: the input monitors through the chain.
    controller.set_live_render(true).unwrap();
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames); // Edge envelope ramps in.
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    assert!(
        frames.iter().any(|sample| sample.abs() > 0.01),
        "monitored input must be audible while stopped",
    );
    assert_eq!(controller.position_frames(), 0);

    // Posture off: one block rides the edge ramp-out (declick), then
    // the render gate silences and the position still never moved.
    controller.set_live_render(false).unwrap();
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), 0);
    let _ = base;
}

#[test]
fn compiled_events_and_timeline_clips_stay_gated_while_stopped_under_live_render() {
    // Compiled plugin events must not fire while stopped (frozen
    // position would re-trigger them every block).
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 100,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 64,
            velocity: 0.75,
        },
    }]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_live_render(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    let calls = backend.calls();
    assert_eq!(calls.len(), 2, "the stage renders while stopped");
    assert!(
        calls.iter().all(|events| events.is_empty()),
        "compiled events stay playing-gated: {calls:?}",
    );
    assert_eq!(controller.position_frames(), 0);

    // Rolling delivers the compiled stream from the held position.
    controller.set_playing(true).unwrap();
    executor.render_block(&mut frames);
    let calls = backend.calls();
    assert_eq!(calls[2].len(), 1);
    assert_eq!(calls[2][0].offset_frames, 100);

    // Timeline clip content is silent while stopped under the posture.
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_live_render(true).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(
        frames.iter().all(|sample| *sample == 0.0),
        "a stopped transport must not replay frozen clip content",
    );
    controller.set_playing(true).unwrap();
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
}

#[test]
fn stream_handles_compare_by_pointer_for_cheap_spec_equality() {
    let (_feeder_a, a) = render_stream(48_000, 1_000);
    let b = a.clone();
    let (_feeder_c, c) = render_stream(48_000, 1_000);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.source_sample_rate_hz(), 48_000);
    assert_eq!(a.total_frames(), 1_000);
}

// ── Note sources (built-in instrument) ─────────────────────────────────

fn note(start_frame: u64, duration_frames: u64, degree: i32, velocity: f32) -> RenderNote {
    RenderNote {
        start_frame,
        duration_frames,
        degree,
        pitch_intent: None,
        velocity,
    }
}

#[test]
fn degree_frequency_derivation_is_bit_identical_to_the_u8_pitch_formula() {
    // g12.034 widening compatibility pin: a degree with no pitch intent
    // must derive the EXACT bits the pre-widening
    // `440 * 2^((pitch - 69) / 12)` path produced for every u8 pitch.
    for pitch in 0u8..=127 {
        let old = 440.0 * f64::powf(2.0, (f64::from(pitch) - 69.0) / 12.0);
        let new = note(0, 1, i32::from(pitch), 1.0).frequency_hz();
        assert_eq!(old.to_bits(), new.to_bits(), "diverged at pitch {pitch}");
    }
}

#[test]
fn pitch_intent_overrides_or_offsets_the_degree_frequency() {
    let mut absolute = note(0, 1, 69, 1.0);
    absolute.pitch_intent = Some(RenderPitchIntent::FrequencyHz(432.0));
    assert_eq!(absolute.frequency_hz(), 432.0);

    let mut offset = note(0, 1, 69, 1.0);
    offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(1200.0));
    assert!((offset.frequency_hz() - 880.0).abs() < 1e-9);

    let mut zero_offset = note(0, 1, 69, 1.0);
    zero_offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(0.0));
    assert_eq!(zero_offset.frequency_hz(), 440.0);
}

fn note_buffer(notes: Vec<RenderNote>) -> RenderNoteBuffer {
    RenderNoteBuffer {
        notes: notes.into(),
    }
}

/// Spec with one notes clip windowed `[start, end)` at lane gain 1.
fn notes_spec(buffer: &RenderNoteBuffer, start_frames: u64, end_frames: u64) -> RenderPlanSpec {
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1008,
            start_frames,
            end_frames,
            source: RenderSource::Notes(buffer.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Offline-render `frame_count` frames of `spec` from `start_frame` and
/// return the LEFT channel (channels are identical for note sources).
fn render_notes_left(spec: &RenderPlanSpec, start_frame: u64, frame_count: u64) -> Vec<f32> {
    let output = crate::render_plan_to_pcm(
        spec,
        &crate::OfflineRenderOptions {
            start_frame,
            frame_count,
            ..crate::OfflineRenderOptions::default()
        },
    )
    .expect("offline note render");
    output
        .master
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect()
}

#[test]
fn note_clips_render_at_the_note_frequency() {
    // A4 (pitch 69) sustained for one second: past the attack and clip
    // edge fade, the output must be a 440 Hz sine at the velocity
    // amplitude. Quadrature projection at 440 Hz recovers the amplitude
    // and the residual bounds everything that is not that sine.
    let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let left = render_notes_left(&spec, 0, 24_000);

    let start = 4_800usize; // Past attack (144 frames) and edge fade.
    let count = 14_400usize; // Whole periods of 440 at 48k every 300.
    let mut in_phase = 0.0f64;
    let mut quadrature = 0.0f64;
    for index in 0..count {
        let n = (start + index) as f64;
        let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
        let sample = f64::from(left[start + index]);
        in_phase += sample * angle.sin();
        quadrature += sample * angle.cos();
    }
    let amplitude = 2.0 * (in_phase * in_phase + quadrature * quadrature).sqrt() / count as f64;
    assert!(
        (amplitude - 1.0).abs() < 0.01,
        "440 Hz amplitude read {amplitude}",
    );
    // Residual after removing the projected 440 Hz component: > 60 dB
    // below the tone (proof the output is that sine, not something else).
    let sine_gain = 2.0 * in_phase / count as f64;
    let cosine_gain = 2.0 * quadrature / count as f64;
    let mut error = 0.0f64;
    let mut power = 0.0f64;
    for index in 0..count {
        let n = (start + index) as f64;
        let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
        let expected = sine_gain * angle.sin() + cosine_gain * angle.cos();
        let actual = f64::from(left[start + index]);
        error += (actual - expected) * (actual - expected);
        power += expected * expected;
    }
    let snr = 10.0 * (power / error.max(1e-30)).log10();
    assert!(snr > 60.0, "note tone SNR {snr:.1} dB");
}

#[test]
fn note_envelope_is_silent_before_attacks_and_releases() {
    // Note at frame 4_800, 4_800 frames long: silence before the start,
    // ramping attack (3 ms = 144 frames), full velocity through the
    // sustain, and a 40 ms release tail that ends in exact silence.
    let buffer = note_buffer(vec![note(4_800, 4_800, 69, 1.0)]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let left = render_notes_left(&spec, 0, 24_000);

    assert!(
        left[..4_800].iter().all(|sample| *sample == 0.0),
        "audio before the note start",
    );
    let peak =
        |range: std::ops::Range<usize>| left[range].iter().fold(0.0f32, |max, s| max.max(s.abs()));
    // Attack: the first 72 frames stay under the half-ramped level.
    assert!(peak(4_800..4_872) < 0.55, "attack did not ramp");
    // Sustain: full velocity once the attack completes.
    let sustain_peak = peak(5_200..9_000);
    assert!(
        (sustain_peak - 1.0).abs() < 0.05,
        "sustain peak {sustain_peak}",
    );
    // Release: decaying after the note end...
    let release_start = 4_800 + 4_800;
    let release_end = release_start + 1_920; // 40 ms at 48 kHz.
    assert!(peak(release_start + 960..release_end) < 0.6, "release flat");
    // ...and exactly silent once the tail ends.
    assert!(
        left[release_end + 1..].iter().all(|sample| *sample == 0.0),
        "audio after the release tail",
    );
}

#[test]
fn chords_render_as_the_sum_of_their_notes() {
    let pitches = [60i32, 64, 67];
    let chord = note_buffer(pitches.iter().map(|p| note(0, 24_000, *p, 0.3)).collect());
    let chord_left = render_notes_left(&notes_spec(&chord, 0, u64::MAX), 0, 12_000);

    let mut summed = vec![0.0f32; 12_000];
    for pitch in pitches {
        let single = note_buffer(vec![note(0, 24_000, pitch, 0.3)]);
        let left = render_notes_left(&notes_spec(&single, 0, u64::MAX), 0, 12_000);
        for (accumulator, sample) in summed.iter_mut().zip(left.iter()) {
            *accumulator += *sample;
        }
    }
    for (index, (chord_sample, sum_sample)) in chord_left.iter().zip(summed.iter()).enumerate() {
        assert!(
            (chord_sample - sum_sample).abs() < 1e-5,
            "chord diverged from note sum at {index}: {chord_sample} vs {sum_sample}",
        );
    }
}

#[test]
fn seeking_into_a_sustained_note_reproduces_played_through_samples() {
    // Statelessness proof: rendering from a mid-note seek must produce
    // the SAME bits as playing through from zero — there is no voice
    // state whose absence a seek could expose.
    let buffer = note_buffer(vec![
        note(0, 48_000, 57, 0.8),
        note(6_000, 30_000, 64, 0.6),
        note(12_000, 12_000, 72, 1.0),
    ]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let played_through = render_notes_left(&spec, 0, 48_000);
    let seeked = render_notes_left(&spec, 24_000, 4_800);
    for (index, (seek_sample, through_sample)) in seeked
        .iter()
        .zip(played_through[24_000..24_000 + 4_800].iter())
        .enumerate()
    {
        assert_eq!(
            seek_sample.to_bits(),
            through_sample.to_bits(),
            "seek diverged from play-through at offset {index}",
        );
    }
}

#[test]
fn note_polyphony_caps_at_the_limit_keeping_earliest_started() {
    // 33 simultaneous notes: the render must equal the first 32 alone
    // (the 33rd — latest in sorted order — is skipped), and dropping to
    // 31 must change the output (the cap has teeth).
    let make = |count: usize| {
        note_buffer(
            (0..count)
                .map(|index| note(index as u64, 24_000, 40 + index as i32, 0.02))
                .collect(),
        )
    };
    let render =
        |count: usize| render_notes_left(&notes_spec(&make(count), 0, u64::MAX), 0, 12_000);
    let with_33 = render(33);
    let with_32 = render(32);
    let with_31 = render(31);
    assert_eq!(
        with_33
            .iter()
            .zip(with_32.iter())
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count(),
        0,
        "the 33rd simultaneous note leaked past the polyphony cap",
    );
    assert!(
        with_32
            .iter()
            .zip(with_31.iter())
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "32nd note should be audible (cap test has no teeth)",
    );
}

#[test]
fn unsorted_note_buffers_are_rejected_at_compile() {
    let (mut controller, _executor) = render_plane();
    let buffer = note_buffer(vec![note(1_000, 100, 60, 1.0), note(0, 100, 62, 1.0)]);
    let error = controller
        .install_plan(&notes_spec(&buffer, 0, u64::MAX))
        .unwrap_err();
    assert!(error.message.contains("sorted"), "{}", error.message);
}

#[test]
fn note_buffers_compare_by_pointer_for_cheap_spec_equality() {
    let notes: Arc<[RenderNote]> = vec![note(0, 100, 60, 1.0)].into();
    let a = RenderNoteBuffer {
        notes: Arc::clone(&notes),
    };
    let b = RenderNoteBuffer { notes };
    let c = note_buffer(vec![note(0, 100, 60, 1.0)]);
    assert_eq!(a, b);
    assert_ne!(a, c);
    // Spec equality follows buffer equality: idempotent recompiles.
    assert_eq!(notes_spec(&a, 0, 100), notes_spec(&b, 0, 100));
    assert_ne!(notes_spec(&a, 0, 100), notes_spec(&c, 0, 100));
}

#[test]
fn note_clip_windows_gate_notes_on_the_stream_clock() {
    // Clip windowed [1_000, 2_000): a clip-relative note at 0 sounds at
    // stream frame 1_000, and nothing sounds past the window end even
    // though the note's tail extends beyond it.
    let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
    let spec = notes_spec(&buffer, 1_000, 2_000);
    let left = render_notes_left(&spec, 0, 4_000);
    assert!(left[..1_000].iter().all(|sample| *sample == 0.0));
    assert!(
        left[1_100..1_900].iter().any(|sample| sample.abs() > 0.5),
        "windowed note inaudible",
    );
    assert!(left[2_000..].iter().all(|sample| *sample == 0.0));
}

#[test]
fn retired_plans_return_to_the_control_side() {
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    let mut frames = [0.0f32; 64];
    executor.render_block(&mut frames);

    controller.install_plan(&tone_spec(880.0)).unwrap();
    executor.render_block(&mut frames);

    assert_eq!(controller.collect_retired(), 1);
    assert_eq!(controller.retired_parked_blocks(), 0);
}

// ── Graph-shaped plans ──────────────────────────────────────────────────

#[test]
fn compile_rejects_cycles() {
    let (mut controller, _executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 1,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(2)],
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 2,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(1)],
            },
            master_node(vec![identity_edge(1)]),
        ],
    };
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("cycle"), "{}", error.message);
}

#[test]
fn compile_rejects_duplicate_node_ids() {
    let (mut controller, _executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(7, 1.0, vec![]),
            lane_node(7, 1.0, vec![]),
            master_node(vec![identity_edge(7)]),
        ],
    };
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("duplicate"), "{}", error.message);
}

#[test]
fn compile_rejects_wrong_master_count() {
    let (mut controller, _executor) = render_plane();
    let no_master = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![lane_node(1, 1.0, vec![])],
    };
    let error = controller.install_plan(&no_master).unwrap_err();
    assert!(
        error.message.contains("exactly one output stage"),
        "{}",
        error.message
    );

    let mut two_masters = master_node(vec![]);
    two_masters.stage_id = MASTER_ID + 1;
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![master_node(vec![]), two_masters],
    };
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error.message.contains("exactly one output stage"),
        "{}",
        error.message
    );
}

#[test]
fn compile_rejects_unknown_inputs_and_bad_matrices() {
    let (mut controller, _executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![master_node(vec![identity_edge(99)])],
    };
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("unknown input"), "{}", error.message);

    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(1, 1.0, vec![]),
            master_node(vec![RenderEdgeSpec {
                source_stage_id: 1,
                gain: 1.0,
                matrix: Some(vec![1.0, 0.0, 0.0]), // 2x2 edge needs 4.
            }]),
        ],
    };
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error.message.contains("matrix") && error.message.contains("expected 4"),
        "{}",
        error.message
    );
}

#[test]
fn bus_chain_renders_in_topological_order() {
    // Stages listed deliberately out of order: output first, then the
    // Sum chain, then the source. The schedule must still run source →
    // sum A → sum B → output, with each stage's gain applied at
    // consumption.
    let (mut controller, mut executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            master_node(vec![identity_edge(20)]),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 20,
                format: ChannelFormat::stereo(),
                gain: 0.5,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(10)],
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 10,
                format: ChannelFormat::stereo(),
                gain: 0.5,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(LANE_ID)],
            },
            lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Reference: the same tone through a single lane at the chain's
    // composite gain (0.5 × 0.5 = 0.25).
    let (mut reference_controller, mut reference_executor) = render_plane();
    reference_controller
        .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
        .unwrap();
    reference_controller.set_playing(true).unwrap();
    warm_up(&mut reference_executor, 2);

    let mut chained = [0.0f32; 512];
    let mut reference = [0.0f32; 512];
    executor.render_block(&mut chained);
    reference_executor.render_block(&mut reference);
    for (a, b) in chained.iter().zip(reference.iter()) {
        assert!((a - b).abs() < 1e-6, "chain diverged: {a} vs {b}");
    }
}

#[test]
fn pan_matrix_places_a_lane_in_the_stereo_field() {
    // The pan primitive per chorus a14: an explicit 2×2 equal-power
    // matrix on the lane → master edge.
    let render_with_pan = |pan: f32| -> [f32; 512] {
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
                master_node(vec![RenderEdgeSpec {
                    source_stage_id: LANE_ID,
                    gain: 1.0,
                    matrix: Some(equal_power_pan_matrix(pan).to_vec()),
                }]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        frames
    };

    let hard_left = render_with_pan(-1.0);
    assert!(hard_left.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
    assert!(hard_left.chunks_exact(2).all(|frame| frame[1] == 0.0));

    let hard_right = render_with_pan(1.0);
    assert!(hard_right
        .chunks_exact(2)
        .all(|frame| frame[0].abs() < 1e-6));
    assert!(hard_right.chunks_exact(2).any(|frame| frame[1].abs() > 0.1));

    let center = render_with_pan(0.0);
    let minus_3db = std::f32::consts::FRAC_1_SQRT_2;
    for frame in center.chunks_exact(2) {
        assert!((frame[0] - frame[1]).abs() < 1e-6);
    }
    // Center sits -3 dB against the hard-left reference.
    let left_peak = hard_left
        .chunks_exact(2)
        .map(|frame| frame[0].abs())
        .fold(0.0f32, f32::max);
    let center_peak = center
        .chunks_exact(2)
        .map(|frame| frame[0].abs())
        .fold(0.0f32, f32::max);
    assert!((center_peak - left_peak * minus_3db).abs() < 1e-3);
}

#[test]
fn mono_lane_upmixes_to_stereo_through_the_default_adapter() {
    let (mut controller, mut executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: LANE_ID,
                format: ChannelFormat::mono(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![tone_clip(440.0)],
                },
                inputs: Vec::new(),
            },
            master_node(vec![identity_edge(LANE_ID)]),
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    let peak = frames
        .chunks_exact(2)
        .map(|frame| frame[0].abs())
        .fold(0.0f32, f32::max);
    assert!(peak > 0.1, "mono lane should be audible after upmix");
    for frame in frames.chunks_exact(2) {
        // Equal distribution at -3 dB: both channels identical.
        assert_eq!(frame[0], frame[1]);
    }
    // -3 dB against a stereo lane at unity.
    assert!((peak - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.05);
}

#[test]
fn send_topology_sums_both_paths() {
    // Source feeds Sum A and Sum B (a send-like fan-out); both feed the
    // output. The
    // output must be exactly double the single-path render.
    let (mut controller, mut executor) = render_plane();
    let sum_stage = |stage_id: u64| RenderStageSpec {
        parameter_envelopes: Vec::new(),
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain: 1.0,
        gain_automation: None,
        kind: RenderStageKind::Sum,
        inputs: vec![identity_edge(LANE_ID)],
    };
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(LANE_ID, 0.25, vec![tone_clip(440.0)]),
            sum_stage(10),
            sum_stage(11),
            master_node(vec![identity_edge(10), identity_edge(11)]),
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let (mut reference_controller, mut reference_executor) = render_plane();
    reference_controller
        .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
        .unwrap();
    reference_controller.set_playing(true).unwrap();
    warm_up(&mut reference_executor, 2);

    let mut sent = [0.0f32; 512];
    let mut single = [0.0f32; 512];
    executor.render_block(&mut sent);
    reference_executor.render_block(&mut single);
    for (doubled, reference) in sent.iter().zip(single.iter()) {
        assert!(
            (doubled - reference * 2.0).abs() < 1e-6,
            "send sum diverged: {doubled} vs 2×{reference}",
        );
    }
}

#[test]
fn wider_master_downmixes_at_the_hardware_boundary() {
    // 4-channel master on a 2-channel stream: the boundary matrix
    // (compiled at install, when the stream is known) folds channels
    // 0/2 onto left and 1/3 onto right at equal weight.
    let (mut controller, mut executor) = render_plane();
    controller.set_stream_channels(2).unwrap();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: LANE_ID,
                format: ChannelFormat::mono(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![tone_clip(440.0)],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: MASTER_ID,
                format: ChannelFormat {
                    channels: 4,
                    layout: ChannelLayout::Generic,
                },
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                // Distinct synthetic spread: [1.0, 0.5, 0.25, 0.75].
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: LANE_ID,
                    gain: 1.0,
                    matrix: Some(vec![1.0, 0.5, 0.25, 0.75]),
                }],
            },
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    // Mono reference at unity for the same tone.
    let (mut reference_controller, mut reference_executor) = render_plane();
    reference_controller
        .install_plan(&lane_master_spec(1.0, vec![tone_clip(440.0)]))
        .unwrap();
    reference_controller.set_playing(true).unwrap();
    warm_up(&mut reference_executor, 2);

    let mut downmixed = [0.0f32; 512];
    let mut reference = [0.0f32; 512];
    executor.render_block(&mut downmixed);
    reference_executor.render_block(&mut reference);
    // Boundary fold (4→2): L = (c0 + c2)/2 = (1.0 + 0.25)/2 = 0.625×tone,
    // R = (c1 + c3)/2 = (0.5 + 0.75)/2 = 0.625×tone.
    for (frame, reference_frame) in downmixed.chunks_exact(2).zip(reference.chunks_exact(2)) {
        let tone = reference_frame[0];
        assert!((frame[0] - tone * 0.625).abs() < 1e-5);
        assert!((frame[1] - tone * 0.625).abs() < 1e-5);
    }
    // Clock advances by stream frames (2-channel framing): 512+256.
    assert_eq!(controller.position_frames(), 768);
}

#[test]
fn narrower_master_leaves_extra_stream_channels_silent() {
    // Mono master on a stereo stream: the hardware stage never invents
    // an upmix — channel 0 carries the master, channel 1 stays silent.
    let (mut controller, mut executor) = render_plane();
    controller.set_stream_channels(2).unwrap();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: LANE_ID,
                format: ChannelFormat::mono(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![tone_clip(440.0)],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: MASTER_ID,
                format: ChannelFormat::mono(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![identity_edge(LANE_ID)],
            },
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
    assert!(frames.chunks_exact(2).all(|frame| frame[1] == 0.0));
}

#[test]
fn meters_publish_per_stage_levels_and_zero_when_silent() {
    let (mut controller, mut executor) = render_plane();
    // No install yet: nothing to resolve against.
    assert!(controller.meters().is_empty());

    controller.install_plan(&tone_spec(440.0)).unwrap();
    // Installed but not yet rendered: the shared table still carries the
    // previous generation, so the controller refuses to mislabel slots.
    assert!(controller.meters().is_empty());

    controller.set_playing(true).unwrap();
    warm_up(&mut executor, 2);

    let meters = controller.meters();
    assert_eq!(meters.len(), 2);
    let (lane_id, lane_peak, lane_rms) = meters[0];
    let (master_id, master_peak, _) = meters[1];
    assert_eq!(lane_id, LANE_ID);
    assert_eq!(master_id, MASTER_ID);
    assert!(lane_peak > 0.01, "lane peak {lane_peak}");
    assert!(lane_rms > 0.001 && lane_rms <= lane_peak);
    assert!(master_peak > 0.01, "master peak {master_peak}");

    // Stop: after the ramp-out, silent blocks publish zeros.
    controller.set_playing(false).unwrap();
    warm_up(&mut executor, 4);
    let meters = controller.meters();
    assert!(meters
        .iter()
        .all(|(_, peak, rms)| *peak == 0.0 && *rms == 0.0));
}

/// Wall-clock soak gate; see the identical helper in the soak test files.
/// This test asserts that two back-to-back blocks produce zero xruns, which
/// is a claim about how fast the host is, so it belongs in the soak lane
/// rather than the correctness suite.
fn soak_tests_enabled() -> bool {
    if std::env::var("SIGNAL_SOAK_TESTS").as_deref() == Ok("1") {
        return true;
    }
    eprintln!("SKIPPED: wall-clock soak test; set SIGNAL_SOAK_TESTS=1 (or run `effigy test:soak`)");
    false
}

#[test]
fn callback_health_counters_advance_and_infer_xruns() {
    if !soak_tests_enabled() {
        return;
    }
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = [0.0f32; 512]; // 256 frames ≈ 5.3 ms at 48 kHz.
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert_eq!(controller.callback_count(), 2);
    assert!(
        controller.max_callback_duration_micros() >= controller.last_callback_duration_micros()
    );
    // Back-to-back blocks are far faster than the deadline: no xruns.
    assert_eq!(controller.xrun_count(), 0);

    // Starve the callback past 1.5 × the block duration: one xrun.
    std::thread::sleep(std::time::Duration::from_millis(20));
    executor.render_block(&mut frames);
    assert_eq!(controller.xrun_count(), 1);
}

/// FNV-1a 64 over the bit pattern of rendered samples.
fn fnv1a_hash_pcm(frames: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for sample in frames {
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

#[test]
fn golden_render_hash_is_stable() {
    // Reference plan: two tone lanes panned hard left/right through a
    // Sum stage, mixed with a centered mono source at the output. Renders 8 ×
    // 256-frame blocks from transport start (edge ramp included) and
    // hashes the PCM. Gates every render-plane change: any behavioral
    // drift in declick, smoothing, scheduling, or matrix application
    // moves the hash.
    //
    // Regenerating after an INTENTIONAL change: run with the assert
    // relaxed (or print `hash`), paste the new value, and justify the
    // change in the commit. Never regenerate to silence a failure you
    // cannot explain.
    let (mut controller, mut executor) = render_plane();
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 0.8,
        master_limiter: None,
        stages: vec![
            lane_node(1, 0.5, vec![tone_clip(440.0)]),
            lane_node(2, 0.4, vec![tone_clip(660.0)]),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 3,
                format: ChannelFormat::mono(),
                gain: 0.3,
                gain_automation: None,
                kind: RenderStageKind::Source {
                    clips: vec![tone_clip(220.0)],
                },
                inputs: Vec::new(),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 10,
                format: ChannelFormat::stereo(),
                gain: 0.9,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![
                    RenderEdgeSpec {
                        source_stage_id: 1,
                        gain: 1.0,
                        matrix: Some(equal_power_pan_matrix(-1.0).to_vec()),
                    },
                    RenderEdgeSpec {
                        source_stage_id: 2,
                        gain: 0.8,
                        matrix: Some(equal_power_pan_matrix(1.0).to_vec()),
                    },
                ],
            },
            master_node(vec![identity_edge(10), identity_edge(3)]),
        ],
    };
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut frames = [0.0f32; 512];
    for _ in 0..8 {
        executor.render_block(&mut frames);
        // Chain the per-block hashes by re-seeding from the running value.
        hash ^= fnv1a_hash_pcm(&frames);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Recorded on first run (see regeneration note above).
    assert_eq!(
        hash, GOLDEN_RENDER_HASH,
        "golden render drifted: {hash:#018x}"
    );
}

/// Recorded output hash for `golden_render_hash_is_stable` (captured on
/// the test's first run; see the regeneration note in the test body).
const GOLDEN_RENDER_HASH: u64 = 0x494b_7128_ef17_1a6a;
