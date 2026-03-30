use super::super::*;

#[test]
fn runtime_plugin_placement_policy_drives_shared_and_isolated_assignment_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![
                RuntimePluginPlacementRule {
                    rule_id: "isolate-instrument".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                        "plugin://instrument".into(),
                    ),
                    outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
                    sandbox_group_key: None,
                },
                RuntimePluginPlacementRule {
                    rule_id: "share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("format:clap".into()),
                },
            ],
        })
        .expect("apply plugin placement policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-placement".into(),
            node_count: 3,
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
                GraphNodeProjection {
                    node_id: "plugin-c".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .expect("apply plugin placement graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:plugin-placement".into(),
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
        .expect("apply plugin placement contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-placement".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-shared".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-b".into(),
                    sandbox_id: "sandbox-shared".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-c".into(),
                    sandbox_id: "sandbox-isolated".into(),
                },
            ],
        })
        .expect("apply plugin placement bindings");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-shared".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin://shared-effect".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-isolated".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin://instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-shared",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-shared",
        "lease-shared",
        "region-shared",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-isolated",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-isolated",
        "lease-isolated",
        "region-isolated",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    let lifecycle = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(lifecycle.shared_sandbox_count, 1);
    assert_eq!(lifecycle.isolated_sandbox_count, 1);
    let shared = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared sandbox");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(shared.placement_rule_id.as_deref(), Some("share-clap"));
    assert_eq!(shared.sandbox_group_key, "format:clap");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Steady);
    let isolated = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated sandbox");
    assert_eq!(
        isolated.placement_outcome,
        RuntimePluginIsolationOutcome::IsolatedSandbox
    );
    assert_eq!(
        isolated.placement_rule_id.as_deref(),
        Some("isolate-instrument")
    );
    assert_eq!(isolated.shared_boundary_member_count, 1);

    let chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(chain.shared_sandbox_stage_count, 2);
    assert_eq!(chain.isolated_sandbox_stage_count, 1);
    assert_eq!(chain.rebindable_stage_count, 0);
    assert_eq!(chain.terminal_stage_count, 0);
    assert!(chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.placement_outcome == RuntimePluginIsolationOutcome::SharedSandbox)
        .all(|stage| {
            stage.sandbox_group_key.as_deref() == Some("format:clap")
                && stage.shared_boundary_member_count == 2
                && stage.continuity_class == RuntimeInterruptionClass::Steady
        }));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"format:clap\""));
}
