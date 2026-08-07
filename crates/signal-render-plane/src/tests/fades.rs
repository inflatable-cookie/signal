use super::support::*;
use super::*;

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
