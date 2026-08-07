use super::support::*;
use super::*;

#[test]
fn sample_buffers_compare_by_pointer_for_cheap_spec_equality() {
    let data: Arc<[f32]> = vec![0.0f32; 8].into();
    let a = RenderSampleBuffer::stereo(48_000, Arc::clone(&data));
    let b = RenderSampleBuffer::stereo(48_000, data);
    let c = RenderSampleBuffer::stereo(48_000, vec![0.0f32; 8].into());
    assert_eq!(a, b);
    assert_ne!(a, c);
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
