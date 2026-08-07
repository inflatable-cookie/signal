use super::support::*;
use super::*;

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
