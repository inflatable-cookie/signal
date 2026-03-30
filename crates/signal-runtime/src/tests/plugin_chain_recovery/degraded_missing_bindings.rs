use super::super::*;

#[test]
fn runtime_plugin_chain_snapshot_preserves_degraded_and_missing_binding_states() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-chain-degraded".into(),
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
            graph_id: "graph:runtime:plugin-chain-degraded".into(),
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
            graph_id: "graph:runtime:plugin-chain-degraded".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-a".into(),
                sandbox_id: "sandbox-faulted".into(),
            }],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-faulted",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-faulted",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-faulted",
        PluginFaultKind::Crash,
        "sandbox faulted before render",
        Some(2),
    );

    let snapshot = runtime.get_plugin_chain_snapshot();
    assert_eq!(snapshot.chain_count, 1);
    assert_eq!(snapshot.stage_count, 2);
    assert_eq!(snapshot.degraded_stage_count, 1);
    assert_eq!(snapshot.missing_binding_stage_count, 1);
    assert_eq!(
        snapshot.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::Degraded
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall_state,
        RuntimePluginRecallState::Unavailable
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall.payload.lifecycle_state,
        Some(RuntimePluginLifecycleState::Faulted)
    );
    assert_eq!(snapshot.chains[0].stages[0].recall.payload.fault_count, 1);
    assert_eq!(
        snapshot.chains[0].stages[0].recall.payload.last_fault_kind,
        Some(PluginFaultKind::Crash)
    );
    assert_eq!(
        snapshot.chains[0].stages[0]
            .recall
            .payload
            .last_fault_detail
            .as_deref(),
        Some("sandbox faulted before render")
    );
    assert_eq!(
        snapshot.chains[0].stages[1].compensation_state,
        RuntimePluginCompensationState::MissingBinding
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall_state,
        RuntimePluginRecallState::Unbound
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall.state,
        RuntimePluginRecallState::Unbound
    );
    assert_eq!(snapshot.chains[0].stages[1].recall.payload.sandbox_id, None);

    let unavailable_handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(unavailable_handoff.stage_count, 2);
    assert_eq!(unavailable_handoff.unavailable_stage_count, 1);
    assert_eq!(unavailable_handoff.unbound_stage_count, 1);
    assert_eq!(
        unavailable_handoff.stages[0].recall_payload.lifecycle_state,
        Some(RuntimePluginLifecycleState::Faulted)
    );

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let unavailable_json = supervisor.render_json();
    assert!(unavailable_json.contains("\"recall\":{\"state\":\"Unavailable\""));
    assert!(unavailable_json.contains("\"payload\":{\"sandbox_id\":\"sandbox-faulted\""));
    assert!(unavailable_json.contains("\"lifecycle_state\":\"Faulted\""));

    runtime.record_plugin_sandbox_fault(
        "sandbox-faulted",
        PluginFaultKind::Timeout,
        "sandbox missed heartbeat twice",
        Some(3),
    );

    let quarantined = runtime.get_plugin_chain_snapshot();
    assert_eq!(
        quarantined.chains[0].stages[0].recall.state,
        RuntimePluginRecallState::Unavailable
    );
    assert_eq!(
        quarantined.chains[0].stages[0]
            .recall
            .payload
            .lifecycle_state,
        Some(RuntimePluginLifecycleState::Quarantined)
    );
    assert_eq!(
        quarantined.chains[0].stages[0].recall.payload.fault_count,
        2
    );
    assert_eq!(
        quarantined.chains[0].stages[0]
            .recall
            .payload
            .last_fault_kind,
        Some(PluginFaultKind::Timeout)
    );

    let quarantined_handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(quarantined_handoff.unavailable_stage_count, 1);
    assert_eq!(
        quarantined_handoff.stages[0].recall_payload.lifecycle_state,
        Some(RuntimePluginLifecycleState::Quarantined)
    );

    let quarantined_supervisor =
        RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let quarantined_multiline = quarantined_supervisor.render_multiline();
    assert!(quarantined_multiline.contains("recall=Unavailable/sandbox=Some(\"sandbox-faulted\")"));
    let quarantined_json = quarantined_supervisor.render_json();
    assert!(quarantined_json.contains("\"lifecycle_state\":\"Quarantined\""));
}
