use super::super::support::*;
use super::super::*;

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
