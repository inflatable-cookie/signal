use super::super::*;

#[test]
fn runtime_plugin_chain_snapshot_settles_tail_before_returning_to_pending_render() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-chain-settling".into(),
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
            graph_id: "graph:runtime:plugin-chain-settling".into(),
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
            graph_id: "graph:runtime:plugin-chain-settling".into(),
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
            graph_id: "graph:runtime:plugin-chain-settling".into(),
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

    let compensated = runtime.get_plugin_chain_snapshot();
    assert_eq!(compensated.compensated_stage_count, 1);
    assert_eq!(compensated.settling_stage_count, 0);
    assert_eq!(
        compensated.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::Compensated
    );
    assert_eq!(compensated.chains[0].stages[0].tail_samples, Some(48));

    runtime
        .process_engine_block(
            1,
            2,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process settling block");

    let settling = runtime.get_plugin_chain_snapshot();
    assert_eq!(settling.pending_render_stage_count, 0);
    assert_eq!(settling.settling_stage_count, 1);
    assert_eq!(settling.compensated_stage_count, 0);
    assert_eq!(settling.total_realized_latency_samples, 32);
    assert_eq!(settling.total_tail_samples, 44);
    assert_eq!(
        settling.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::Settling
    );
    assert_eq!(
        settling.chains[0].stages[0].realized_latency_samples,
        Some(32)
    );
    assert_eq!(settling.chains[0].stages[0].tail_samples, Some(44));

    for block_sequence in 3..=13 {
        runtime
            .process_engine_block(
                1,
                block_sequence,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process tail retirement block");
    }

    let pending = runtime.get_plugin_chain_snapshot();
    assert_eq!(pending.pending_render_stage_count, 1);
    assert_eq!(pending.settling_stage_count, 0);
    assert_eq!(pending.compensated_stage_count, 0);
    assert_eq!(pending.total_realized_latency_samples, 0);
    assert_eq!(pending.total_tail_samples, 0);
    assert_eq!(
        pending.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::PendingRender
    );
    assert_eq!(pending.chains[0].stages[0].realized_latency_samples, None);
    assert_eq!(pending.chains[0].stages[0].tail_samples, None);
}
