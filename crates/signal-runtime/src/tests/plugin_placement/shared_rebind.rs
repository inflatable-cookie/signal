use super::super::*;

#[test]
fn runtime_shared_sandbox_rebind_receipts_track_restartable_and_terminal_boundaries() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::SharedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("format:clap".into()),
            }],
        })
        .expect("apply shared plugin placement policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:shared-rebind".into(),
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
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .expect("apply shared rebind graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:shared-rebind".into(),
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
        .expect("apply shared rebind contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:shared-rebind".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-shared".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-shared".into(),
                },
            ],
        })
        .expect("apply shared rebind bindings");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-shared".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin://shared-effect".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-shared",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-shared",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_transport(
        "sandbox-shared",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachRequested,
        Some(2),
        Some("replacement attach requested".into()),
    );

    let restartable = runtime.get_plugin_lifecycle_snapshot();
    let shared = restartable
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared sandbox");
    assert_eq!(
        shared.continuity_class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(shared.rebindable);
    assert_eq!(
        shared.transport_stage,
        Some(PluginSandboxTransportStage::DetachRequested)
    );

    let restartable_chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(restartable_chain.rebindable_stage_count, 2);
    assert!(restartable_chain.chains[0]
        .stages
        .iter()
        .all(
            |stage| stage.continuity_class == RuntimeInterruptionClass::Restartable
                && stage.rebindable
                && stage.transport_stage == Some(PluginSandboxTransportStage::DetachRequested)
        ));

    let restartable_supervisor =
        RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let restartable_json = restartable_supervisor.render_json();
    assert!(restartable_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(restartable_json.contains("\"continuity_class\":\"Restartable\""));

    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "shared sandbox crash",
        Some(3),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "shared sandbox timeout",
        Some(4),
    );

    let terminal = runtime.get_plugin_lifecycle_snapshot();
    let shared = terminal
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("terminal shared sandbox");
    assert_eq!(shared.state, RuntimePluginLifecycleState::Quarantined);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);

    let terminal_chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(terminal_chain.terminal_stage_count, 2);
    assert!(terminal_chain.chains[0]
        .stages
        .iter()
        .all(
            |stage| stage.continuity_class == RuntimeInterruptionClass::Terminal
                && !stage.rebindable
        ));

    let terminal_supervisor =
        RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let terminal_json = terminal_supervisor.render_json();
    assert!(terminal_json.contains("\"terminal_stage_count\":2"));
    assert!(terminal_json.contains("\"continuity_class\":\"Terminal\""));
}
