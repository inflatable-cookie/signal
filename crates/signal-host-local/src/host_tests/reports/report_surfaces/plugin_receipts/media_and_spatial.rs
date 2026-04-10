use super::super::super::super::*;
use std::fs;

#[test]
fn local_host_shared_report_surfaces_runtime_media_service_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake");
    host.runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure");

    let imported_path = unique_test_path("local-host-media-service", "wav");
    write_test_wav(&imported_path);
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:local-media".into(),
            content_hash: "local-media".into(),
            source_path: imported_path.display().to_string(),
            file_name: "local-media.wav".into(),
            byte_size: fs::metadata(&imported_path).expect("wav metadata").len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 16,
        }])
        .expect("media reconcile");
    host.runtime
        .start_media_preview("asset:sha256:local-media")
        .expect("start media preview");

    let report = host.supervisor_report();
    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 1);
    assert_eq!(report.observation.media_pipeline_snapshot.ready_asset_count, 1);
    assert_eq!(report.observation.media_service_snapshot.indexed_asset_count, 1);
    assert_eq!(
        report.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        report.observation.media_service_snapshot.previewing_asset_id.as_deref(),
        Some("asset:sha256:local-media")
    );
    assert_eq!(report.observation.media_library_snapshot.indexed_asset_count, 1);
    assert_eq!(report.observation.media_library_snapshot.ready_descriptor_count, 1);
    assert_eq!(
        report.observation.media_library_snapshot.loudness_ready_descriptor_count,
        1
    );
    assert_eq!(
        report.observation.media_library_snapshot.character_ready_descriptor_count,
        1
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
    assert!(rendered.contains("\"media_service_snapshot\":{"));
    assert!(rendered.contains("\"media_library_snapshot\":{"));
    assert!(rendered.contains("\"preview_state\":\"Previewing\""));
    assert!(rendered.contains("\"ready_descriptor_count\":1"));

    let _ = fs::remove_file(&imported_path);
    if let Some(path) = host
        .runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn local_host_shared_report_surfaces_runtime_spatial_execution_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake");
    host.runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure");
    host.runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:host-local:spatial".into(),
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
            graph_id: "graph:host-local:spatial".into(),
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
            graph_id: "graph:host-local:spatial".into(),
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
    assert_eq!(report.observation.execution_topology_summary.spatial_node_count, 2);
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
            && stage.spatial_execution.as_ref().is_some_and(|spatial| {
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
