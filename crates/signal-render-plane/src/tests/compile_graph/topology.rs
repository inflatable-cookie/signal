use super::super::support::*;
use super::super::*;

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
