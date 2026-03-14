use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_server::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RestartRequest,
    RuntimeBlockDeadlinePressure, RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderRequest,
    RuntimePluginIsolationOutcome, RuntimePluginPlacementPolicy, RuntimePluginPlacementRule,
    RuntimePluginPlacementRuleMatcher, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeSupervisorApi, SignalRuntime,
    StopReason,
};

fn apply_public_capture_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .expect("public host-edge capture graph should apply");
}

fn apply_public_render_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "offline-inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                },
                GraphNodeProjection {
                    node_id: "offline-latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .expect("public host-edge render graph should apply");
}

fn apply_public_plugin_continuity_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    bindings: &[(&str, &str)],
) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeProjection {
                    node_id: (*node_id).into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeContractProjection {
                    node_id: (*node_id).into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:host-server:plugin-continuity".into()),
                        bus_group_id: Some("mix:host-server".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: bindings
                .iter()
                .map(|(node_id, sandbox_id)| PluginBackedNodeBinding {
                    node_id: (*node_id).into(),
                    sandbox_id: (*sandbox_id).into(),
                })
                .collect(),
        })
        .expect("public host-edge plugin continuity bindings should apply");
}

fn record_public_plugin_sandbox_ready(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        &format!("lease-{sandbox_id}"),
        &format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(epoch),
        None,
    );
}

#[test]
fn server_shared_host_edge_is_consumable_without_private_helpers() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["/srv/plugins/clap".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("public host-edge scan should succeed");
    host.ensure_plugin_sandbox(PluginSandboxSpec {
        sandbox_id: "public-host-edge-server".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    })
    .expect("public host-edge sandbox ensure should succeed");

    let report = host.supervisor_report();
    assert_eq!(report.observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes.len(),
        1
    );
    assert_eq!(
        report.observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );
    assert_eq!(
        report.observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        report.observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        None
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"fault_status\":{"));
    assert!(rendered.contains("\"fault_diagnostic_receipt\":{"));
    assert!(rendered.contains("\"interruption_summary\":{"));
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_type_id\":\"plugin:clap:server\""));
    assert!(rendered.contains("\"event_stream\":"));
}

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_recording_checkpoint_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut restartable_runtime,
        "graph:host-server:recording-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:restartable".into(),
            track_id: "track:server:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-server-host-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    restartable_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
        )
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(
        &mut terminal_runtime,
        "graph:host-server:recording-terminal",
    );
    terminal_runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:server:terminal".into(),
            track_id: "track:server:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-server-host-recording-terminal.wav".into(),
        })
        .unwrap();
    terminal_runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 63),
        )
        .unwrap();
    let _ = terminal_runtime.finish_recording_capture().unwrap_err();

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report.observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );

    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"recording_capture_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}

#[test]
fn server_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-plugin-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://host-server-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:host-server".into()),
            }],
        })
        .unwrap();
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:host-server:plugin-continuity",
        &[
            ("plugin-a", "sandbox-shared"),
            ("plugin-b", "sandbox-shared"),
            ("plugin-c", "sandbox-isolated"),
        ],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-shared",
        PluginFormat::Clap,
        "plugin://host-server-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://host-server-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "server shared crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "server shared timeout",
        Some(3),
    );

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let shared = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared host-server boundary should be visible");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:host-server");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);
    let isolated = report
        .observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated host-server boundary should remain visible");
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"shared:host-server\""));
    assert!(rendered.contains("\"shared_boundary_member_count\":2"));
    assert!(rendered.contains("\"continuity_class\":\"Terminal\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_fault_diagnostic_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .set_safe_mode(signal_runtime::SafeModeRequest { enabled: true })
        .expect("server host-edge fault diagnostic safe mode should enable");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-server:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("server host-edge fault diagnostic queue should defer");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        report
            .observation
            .fault_diagnostic_receipt
            .interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(report
        .observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .any(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
                && entry.active
        }));

    let rendered = report.render_json();
    assert!(rendered.contains("\"fault_diagnostic_receipt\":{"));
    assert!(rendered.contains("\"primary_family\":\"DeferredWorkPressure\""));
}

#[test]
fn server_shared_host_edge_exports_runtime_block_timing_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-server:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 49),
        )
        .expect("server host-edge block timing block should process");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let performance = report.performance_snapshot();

    assert_eq!(
        report.observation.engine_block_snapshot.last_block_sequence,
        Some(1)
    );
    assert_eq!(
        report
            .observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert!(
        report
            .observation
            .engine_block_snapshot
            .last_block_execution_time_ns
            .expect("server host-edge block timing should expose latest execution time")
            > 0
    );
    assert_eq!(
        performance.last_block_execution_time_ns,
        report
            .observation
            .engine_block_snapshot
            .last_block_execution_time_ns
    );
    assert_eq!(
        performance.last_block_deadline_pressure,
        report
            .observation
            .engine_block_snapshot
            .last_block_deadline_pressure
    );
    assert!(matches!(
        performance.last_block_deadline_pressure,
        RuntimeBlockDeadlinePressure::Normal
            | RuntimeBlockDeadlinePressure::Elevated
            | RuntimeBlockDeadlinePressure::Critical
            | RuntimeBlockDeadlinePressure::Overrun
    ));

    let rendered = report.render_json();
    assert!(rendered.contains("\"engine_block_snapshot\":{"));
    assert!(rendered.contains("\"last_block_execution_time_ns\":"));
    assert!(rendered.contains("\"last_block_deadline_pressure\":"));
}

#[test]
fn server_shared_host_edge_exports_runtime_critical_path_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:host-server:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 53),
        )
        .expect("server host-edge critical-path block should process");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let performance = report.performance_snapshot();

    assert!(performance.hot_latency_node_id.is_some());
    assert!(performance.hot_latency_group_node_count > 0);
    assert!(matches!(
        performance.critical_path_lane.as_deref(),
        Some("Realtime") | Some("Anticipative")
    ));
    assert!(!performance.worker_lane_summaries.is_empty());

    let critical_lane_summary = performance
        .worker_lane_summaries
        .iter()
        .find(|summary| {
            Some(match summary.lane {
                GraphExecutionLane::Realtime => "Realtime",
                GraphExecutionLane::Anticipative => "Anticipative",
            }) == performance.critical_path_lane.as_deref()
        })
        .expect("server host-edge critical-path lane should resolve to a typed worker summary");
    assert_eq!(
        performance.critical_path_lane_node_count,
        critical_lane_summary.node_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        critical_lane_summary.total_latency_samples
    );

    let rendered = performance.render_json();
    assert!(rendered.contains("\"critical_path_lane\":"));
    assert!(rendered.contains("\"worker_lane_summaries\":["));
}

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(
        &mut restartable_runtime,
        "graph:host-server:render-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    restartable_runtime
        .advance_offline_render_execution("render:host-server:restartable")
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();
    restartable_runtime
        .restart(RestartRequest { reconfigure: None })
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(&mut terminal_runtime, "graph:host-server:render-terminal");
    terminal_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-host-server-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    let mut terminal_error_observed = false;
    for _ in 0..16 {
        match terminal_runtime.advance_offline_render_execution("render:host-server:terminal") {
            Ok(_) => continue,
            Err(_) => {
                terminal_error_observed = true;
                break;
            }
        }
    }
    assert!(terminal_error_observed);

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Failed\""));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}
