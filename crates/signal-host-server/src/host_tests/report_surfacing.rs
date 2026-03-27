use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
    temp_media_fixture_path,
};
use super::super::ServerRuntimeHost;
use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
    GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind, RuntimeExternalIoDeviceChangeState,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeMediaPreviewState, RuntimeObservationApi, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
    RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage, SignalRuntime,
    StopReason, TransportAttachIntent,
};
use std::{fs, path::Path};

#[test]
fn server_host_shared_report_surfaces_unavailable_external_io_monitoring_state() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_io_snapshot.health_state,
        RuntimeExternalIoHealthState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.device_change_state,
        RuntimeExternalIoDeviceChangeState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::Unavailable
    );
    assert_eq!(
        report.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report
            .observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert_eq!(
        report.observation.external_io_snapshot.endpoint_topology,
        signal_runtime::RuntimeHostEndpointTopology::Unconfigured
    );
    assert_eq!(
        report.observation.external_io_snapshot.fallback_state,
        signal_runtime::RuntimeHostClockFallbackState::Unconfigured
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"health_state\":\"Unavailable\""));
    assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
    assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
    assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.external_midi_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        report.observation.external_midi_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
    assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
    );
    assert_eq!(
        report
            .observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Guarded
    );
    assert!(report.observation.external_midi_snapshot.devices.is_empty());
    assert!(report
        .observation
        .external_midi_snapshot
        .endpoints
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"live_ownership\":{"));
    assert!(rendered.contains("\"discovery_state\":\"Idle\""));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_linux_backend_session_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    let snapshot = &report.observation.linux_backend_session_snapshot;
    assert_eq!(
        snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        snapshot.lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        snapshot.device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        snapshot.session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        snapshot.ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );
    assert_eq!(snapshot.backend_name, "pipewire");
    assert_eq!(snapshot.device_id, "pipewire:default-graph");
    assert!(snapshot.simulated);

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
    assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_jack_coordination_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    let snapshot = &report.observation.jack_coordination_snapshot;
    assert_eq!(
        snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(snapshot.backend_name, "jack");
    assert_eq!(
        snapshot.transport_posture,
        signal_runtime::RuntimeJackTransportPosture::Detached
    );
    assert_eq!(
        snapshot.graph_state,
        signal_runtime::RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        snapshot.client_role,
        signal_runtime::RuntimeJackClientRole::PrimaryAudioIo
    );
    assert_eq!(
        snapshot.guarded_state,
        signal_runtime::RuntimeJackGuardedCoordinationState::GraphGuarded
    );
    assert_eq!(snapshot.device_id, "jack:graph-main");
    assert!(snapshot.simulated);

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"backend_identity\":\"Jack\""));
    assert!(rendered.contains("\"transport_posture\":\"Detached\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_control_surface_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.control_surface_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report.observation.control_surface_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
    assert!(report
        .observation
        .control_surface_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_advanced_hardware_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .advanced_hardware_snapshot
            .discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(
        report.observation.advanced_hardware_snapshot.device_count,
        0
    );
    assert!(report
        .observation
        .advanced_hardware_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}

#[test]
fn server_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.stretch_engine_snapshot.clip_count, 0);
    assert_eq!(
        report.observation.stretch_engine_snapshot.ready_clip_count,
        0
    );
    assert!(report.observation.stretch_engine_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"sample_domain_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_marker_analysis_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 0);
    assert_eq!(
        report.observation.marker_analysis_snapshot.ready_clip_count,
        0
    );
    assert_eq!(
        report
            .observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        0
    );
    assert!(report.observation.marker_analysis_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"tempo_assist_ready_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_transform_artifact_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 0);
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .ready_clip_count,
        0
    );
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .reusable_clip_count,
        0
    );
    assert!(report
        .observation
        .transform_artifact_snapshot
        .clips
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"reusable_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_preview_transform_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.preview_transform_snapshot.clip_count, 0);
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        0
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .ready_clip_count,
        0
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        0
    );
    assert!(report
        .observation
        .preview_transform_snapshot
        .clips
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"preview_transform_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"artifact_backed_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_media_service_baseline() {
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

    let imported_path = temp_media_fixture_path("server-media-service");
    fs::write(&imported_path, b"signal media fixture").expect("write media fixture");
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:server-media".into(),
            content_hash: "server-media".into(),
            source_path: imported_path.display().to_string(),
            file_name: "server-media.bin".into(),
            byte_size: fs::metadata(&imported_path)
                .expect("fixture metadata")
                .len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 12,
        }])
        .expect("media reconcile");
    host.runtime
        .start_media_preview("asset:sha256:server-media")
        .expect("start media preview");

    let report = host.supervisor_report();
    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 1);
    assert_eq!(
        report.observation.media_pipeline_snapshot.ready_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .indexed_asset_count,
        1
    );
    assert_eq!(
        report.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:server-media")
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .indexed_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .ready_descriptor_count,
        0
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .loudness_ready_descriptor_count,
        0
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .character_ready_descriptor_count,
        0
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .unavailable_descriptor_count,
        1
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
    assert!(rendered.contains("\"media_service_snapshot\":{"));
    assert!(rendered.contains("\"media_library_snapshot\":{"));
    assert!(rendered.contains("\"preview_state\":\"Previewing\""));
    assert!(rendered.contains("\"unavailable_descriptor_count\":1"));

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
        report
            .observation
            .execution_topology_summary
            .spatial_node_count,
        2
    );
    assert_eq!(
        report
            .observation
            .execution_topology_summary
            .active_spatial_node_count,
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

