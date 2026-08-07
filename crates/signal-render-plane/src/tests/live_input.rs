use super::support::*;
use super::*;

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
