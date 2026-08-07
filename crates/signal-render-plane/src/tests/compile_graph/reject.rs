use super::super::support::*;
use super::super::*;

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
