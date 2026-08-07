use super::support::*;
use super::*;

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
