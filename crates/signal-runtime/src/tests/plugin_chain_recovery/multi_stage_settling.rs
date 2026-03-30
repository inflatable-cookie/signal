use super::super::*;

#[test]
fn runtime_plugin_chain_snapshot_tracks_mixed_settling_and_pending_stages_in_multi_stage_chain() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 8,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-b".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-c".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
            contract_count: 3,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-a".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-b".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-c".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-b".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-c".into(),
                    sandbox_id: "sandbox-c".into(),
                },
            ],
        })
        .expect("apply bindings");
    for sandbox_id in ["sandbox-a", "sandbox-b", "sandbox-c"] {
        runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
    }
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                PluginNodeRender {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 8,
                    tail_samples: 0,
                    bypassed: false,
                },
                PluginNodeRender {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-b".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 16,
                    tail_samples: 16,
                    bypassed: false,
                },
                PluginNodeRender {
                    node_id: "plugin-c".into(),
                    sandbox_id: "sandbox-c".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 24,
                    tail_samples: 40,
                    bypassed: false,
                },
            ],
        })
        .expect("apply render batch");
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process first block");
    runtime
        .process_engine_block(
            1,
            2,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process settling block");

    let snapshot = runtime.get_plugin_chain_snapshot();
    assert_eq!(snapshot.chain_count, 1);
    assert_eq!(snapshot.stage_count, 3);
    assert_eq!(snapshot.pending_render_stage_count, 1);
    assert_eq!(snapshot.settling_stage_count, 2);
    assert_eq!(snapshot.compensated_stage_count, 0);
    assert_eq!(snapshot.total_realized_latency_samples, 40);
    assert_eq!(snapshot.total_tail_samples, 48);
    assert_eq!(
        snapshot.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::PendingRender
    );
    assert_eq!(
        snapshot.chains[0].stages[1].compensation_state,
        RuntimePluginCompensationState::Settling
    );
    assert_eq!(snapshot.chains[0].stages[1].tail_samples, Some(12));
    assert_eq!(
        snapshot.chains[0].stages[2].compensation_state,
        RuntimePluginCompensationState::Settling
    );
    assert_eq!(snapshot.chains[0].stages[2].tail_samples, Some(36));
}
