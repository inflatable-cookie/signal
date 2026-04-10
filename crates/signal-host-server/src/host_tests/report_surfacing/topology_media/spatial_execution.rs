use super::super::*;

#[test]
fn server_host_shared_report_surfaces_runtime_spatial_execution_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake");
    host.runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure");
    host.runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:host-server:spatial".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "spatial-stereo".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                },
                GraphNodeProjection {
                    node_id: "spatial-surround".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 20,
                    stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                },
            ],
        })
        .expect("apply spatial graph");
    host.runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:host-server:spatial".into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "spatial-stereo".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:stereo".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:stereo".into()),
                        bus_group_id: Some("bus:spatial:stereo".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "spatial-surround".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:surround-in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:surround".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:surround".into()),
                        bus_group_id: Some("bus:spatial:surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("apply spatial contract");
    host.runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:host-server:spatial".into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: "sandbox:spatial-stereo".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: "sandbox:spatial-surround".into(),
                },
            ],
        })
        .expect("bind spatial nodes");

    let report = host.supervisor_report();
    assert_eq!(
        report.observation.execution_topology_summary.spatial_node_count,
        2
    );
    assert_eq!(
        report.observation.execution_topology_summary.active_spatial_node_count,
        1
    );
    assert_eq!(
        report
            .observation
            .execution_topology_summary
            .fallback_spatial_node_count,
        1
    );
    assert_eq!(
        report
            .observation
            .execution_topology_summary
            .surround_bed_spatial_node_count,
        1
    );
    assert_eq!(
        report
            .observation
            .execution_topology_summary
            .expanded_fallback_spatial_node_count,
        1
    );
    assert!(report
        .observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| stage.node_id == "spatial-surround"
            && stage
                .spatial_execution
                .as_ref()
                .is_some_and(|spatial| {
                    spatial.fallback_outcome
                        == Some(
                            signal_runtime::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing
                        )
                        && spatial.bed_class
                            == signal_runtime::RuntimeSpatialBedClass::CanonicalSurroundBed
                        && spatial.expanded_fallback_outcome
                            == Some(
                                signal_runtime::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                            )
                })));

    let rendered = report.render_json();
    assert!(rendered.contains("\"spatial_node_count\":2"));
    assert!(rendered.contains("\"active_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_spatial_node_count\":1"));
    assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
    assert!(rendered.contains("\"adapter_class\":\"Balance\""));
    assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
    assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"execution_mode\":\"Bypassed\""));
}
