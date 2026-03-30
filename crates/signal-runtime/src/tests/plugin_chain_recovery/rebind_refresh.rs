use super::super::*;

#[test]
fn runtime_execution_topology_summary_clears_stale_plugin_chain_state_on_rebind_and_refresh() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-rebind-refresh".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 24,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
            }],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:plugin-rebind-refresh".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-rebind-refresh".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:plugin-rebind-refresh".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![PluginNodeRender {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
                output: AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
                latency_samples: 32,
                tail_samples: 48,
                bypassed: false,
            }],
        })
        .expect("apply render batch");
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process first block");

    let realized = runtime.get_execution_topology_summary();
    assert_eq!(realized.plugin_chain.total_realized_latency_samples, 32);
    assert_eq!(
        realized.nodes[0].plugin_compensation_state,
        Some(RuntimePluginCompensationState::Compensated)
    );
    assert_eq!(
        realized.nodes[0].plugin_recall_state,
        Some(RuntimePluginRecallState::Warm)
    );
    assert_eq!(
        realized.nodes[0]
            .plugin_recall
            .as_ref()
            .map(|recall| recall.state),
        Some(RuntimePluginRecallState::Warm)
    );
    assert_eq!(
        realized.nodes[0]
            .plugin_recall
            .as_ref()
            .and_then(|recall| recall.payload.sandbox_id.as_deref()),
        Some("sandbox-a")
    );

    let realized_handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(realized_handoff.stage_count, 1);
    assert_eq!(
        realized_handoff.stages[0]
            .recall_payload
            .sandbox_id
            .as_deref(),
        Some("sandbox-a")
    );
    assert_eq!(realized.nodes[0].plugin_realized_latency_samples, Some(32));
    assert_eq!(realized.nodes[0].plugin_tail_samples, Some(48));

    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-rebind-refresh".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-b".into(),
            }],
        })
        .expect("rebind plugin");

    let rebound = runtime.get_execution_topology_summary();
    assert_eq!(rebound.plugin_chain.total_realized_latency_samples, 0);
    assert_eq!(rebound.plugin_chain.pending_render_stage_count, 1);
    assert_eq!(
        rebound.nodes[0].plugin_compensation_state,
        Some(RuntimePluginCompensationState::PendingRender)
    );
    assert_eq!(
        rebound.nodes[0].plugin_recall_state,
        Some(RuntimePluginRecallState::Cold)
    );
    assert_eq!(
        rebound.nodes[0]
            .plugin_recall
            .as_ref()
            .map(|recall| recall.state),
        Some(RuntimePluginRecallState::Cold)
    );
    assert_eq!(
        rebound.nodes[0]
            .plugin_recall
            .as_ref()
            .and_then(|recall| recall.payload.sandbox_id.as_deref()),
        Some("sandbox-b")
    );
    assert_eq!(
        rebound.nodes[0]
            .plugin_recall
            .as_ref()
            .and_then(|recall| recall.payload.lifecycle_state),
        None
    );

    let rebound_handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(rebound_handoff.stage_count, 1);
    assert_eq!(rebound_handoff.cold_stage_count, 1);
    assert_eq!(
        rebound_handoff.stages[0]
            .recall_payload
            .sandbox_id
            .as_deref(),
        Some("sandbox-b")
    );
    assert_eq!(rebound.nodes[0].plugin_realized_latency_samples, None);
    assert_eq!(rebound.nodes[0].plugin_tail_samples, None);

    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:utility-refresh".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "utility".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
            }],
        })
        .expect("apply refreshed graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:utility-refresh".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "utility".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::Utility),
                    track_lane_id: None,
                    bus_group_id: None,
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply refreshed contracts");

    let refreshed = runtime.get_execution_topology_summary();
    assert_eq!(refreshed.plugin_chain.chain_count, 0);
    assert_eq!(refreshed.plugin_chain.stage_count, 0);
    assert_eq!(refreshed.track_lanes.len(), 0);
    assert_eq!(refreshed.nodes.len(), 1);
    assert_eq!(refreshed.nodes[0].node_id, "utility");
    assert_eq!(refreshed.nodes[0].plugin_recall_state, None);
    assert_eq!(refreshed.nodes[0].plugin_recall, None);
    assert_eq!(refreshed.nodes[0].plugin_compensation_state, None);
    assert_eq!(refreshed.nodes[0].plugin_realized_latency_samples, None);

    let refreshed_handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(refreshed_handoff.stage_count, 0);
}
