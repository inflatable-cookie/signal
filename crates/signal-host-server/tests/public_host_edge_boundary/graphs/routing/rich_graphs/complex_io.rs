use super::super::super::super::*;

pub(crate) fn apply_public_complex_io_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-multiout".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-bus-fx".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("public server complex io graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-multiout".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-server:complex-io".into()),
                        bus_group_id: Some("mix:host-server:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-bus-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-server:complex-io".into()),
                        bus_group_id: Some("mix:host-server:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public server complex io contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:host-server:multiout".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:host-server:bus-fx".into(),
                },
            ],
        })
        .expect("public server complex io bindings should apply");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-server:multiout".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-server-multiout".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:host-server:bus-fx".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:host-server-bus-fx".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:multiout",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-server:multiout",
        "lease-host-server-multiout",
        "region-host-server-multiout",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("host server complex io multiout attached".into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:bus-fx",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:host-server:bus-fx",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:host-server:bus-fx",
        "lease-host-server-bus-fx",
        "region-host-server-bus-fx",
        PluginSandboxTransportStage::Attached,
        Some(2),
        Some("host server complex io bus fx attached".into()),
    );
}
