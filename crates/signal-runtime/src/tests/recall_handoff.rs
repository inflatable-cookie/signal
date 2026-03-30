use super::*;

#[test]
fn runtime_plugin_recall_handoff_snapshot_resolves_consumer_selection_without_export_parsing() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-recall-selection".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-b".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:plugin-recall-selection".into(),
            contract_count: 2,
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
            ],
        })
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-recall-selection".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-b".into(),
                },
            ],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_recovery_cycle(
        "sandbox-b",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-b",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(3),
    );

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let selection = RuntimePluginRecallHandoffSelection {
        stage_count: 2,
        stage_ids: handoff
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect(),
    };

    let resolved = handoff
        .resolve_selection(&selection)
        .expect("resolve recall handoff selection");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].stage_id, selection.stage_ids[0]);
    assert_eq!(resolved[0].recall_payload, handoff.stages[0].recall_payload);
    assert_eq!(resolved[1].stage_id, selection.stage_ids[1]);
    assert_eq!(
        resolved[1].recall_state,
        RuntimePluginRecallState::Recovered
    );
    assert_eq!(
        resolved[1].recall_payload.last_restart_intent,
        Some(RecoveryRestartIntent::CrashRecovery)
    );

    let mut missing_selection = selection.clone();
    missing_selection
        .stage_ids
        .push(crate::interfaces::RuntimePluginRecallHandoffStageId {
            chain_id: "track:lead".into(),
            stage_index: 99,
            node_id: "plugin-missing".into(),
        });
    missing_selection.stage_count = missing_selection.stage_ids.len();
    assert!(handoff.resolve_selection(&missing_selection).is_none());
}
