use super::super::support::*;
use super::super::*;

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
