use signal_graph::GraphNodeTopologyRole;
use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphStageSpec,
};
use signal_plugin::{
    PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
    PluginStateContract,
};
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, HandshakeRequest,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RestartRequest,
    RuntimeBlockDeadlinePressure, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeObservationReport,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderRequest,
    RuntimePluginDiscoveredTypeRecord, RuntimePluginIsolationOutcome, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeSupervisorReport,
    RuntimeWatchdogTrigger, SafeModeRequest, SignalRuntime, StopReason, WatchdogRestartRecord,
};

fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:public-boundary".into(),
        plugin_id: "com.signal.public-boundary".into(),
        vendor: "Signal".into(),
        name: "Signal Public Boundary".into(),
        format: PluginFormat::Clap,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        audio_bus_count: 2,
        parameter_count: 8,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: true,
            accepts_midi: true,
            accepts_note_events: true,
            produces_midi: true,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        },
        summary: "public boundary discovered plugin".into(),
    }
}

fn sample_backend_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        plugin_id: "com.signal.public-instrument".into(),
        vendor: "Signal".into(),
        name: "Signal Public Instrument".into(),
        format: PluginFormat::Vst3,
        version: Some("2.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        audio_bus_count: 1,
        parameter_count: 12,
        state_contract: PluginStateContract {
            supports_snapshot: false,
            supports_reset: true,
            supports_bypass: false,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: false,
            supports_reset_while_active: false,
        },
        summary: "public boundary backend breadth plugin".into(),
    }
}

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
        .expect("public capture graph projection should succeed");
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
        .expect("public render graph projection should succeed");
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
        .expect("public plugin continuity graph projection should succeed");
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
                        track_lane_id: Some("track:public:plugin-continuity".into()),
                        bus_group_id: Some("mix:public".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("public plugin continuity contracts should succeed");
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
        .expect("public plugin continuity bindings should succeed");
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
fn public_runtime_contract_boundary_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-boundary-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-boundary-sandbox",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();

    assert_eq!(profiling.sample_rate_hz, 48_000);
    assert_eq!(profiling.block_size, 512);
    assert_eq!(soak.event_stream_count, 0);
    assert_eq!(
        observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert!(!observation.interruption_summary.active);
    assert_eq!(observation.fault_diagnostic_receipt.primary_family, None);
    assert_eq!(observation.fault_diagnostic_receipt.contributions.len(), 4);
    assert_eq!(
        observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Idle)
    );
    assert_eq!(observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        2
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:clap:public-boundary"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].features,
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    );
    assert!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .multi_format_catalog
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .requires_main_thread_for_state_count,
        1
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.format_coverage[1].format,
        PluginFormat::Vst3
    );
    assert_eq!(
        observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"sample_rate\":48000"));
    assert!(observation_json.contains("\"block_size\":512"));
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"fault_status\":{"));
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"interruption_summary\":{"));
    assert!(observation_json.contains("\"recording_capture_snapshot\":{"));
    assert!(observation_json.contains("\"class\":\"Steady\""));
    assert!(observation_json.contains("\"execution_topology_summary\":{"));
    assert!(observation_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:clap:public-boundary\""));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
    assert!(observation_json.contains("\"discovered_format_count\":2"));
    assert!(observation_json.contains("\"multi_format_catalog\":true"));
    assert!(observation_json.contains("\"supports_snapshot\":true"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"sample_rate\":48000"));
    assert!(supervisor_json.contains("\"block_size\":512"));
    assert!(supervisor_json.contains("\"event_stream\":0"));
    assert!(supervisor_json.contains("\"fault_status\":{"));
    assert!(supervisor_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(supervisor_json.contains("\"interruption_summary\":{"));
    assert!(supervisor_json.contains("\"recording_capture_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"discovered_type_count\":2"));
    assert!(supervisor_json.contains("\"format_coverage\":["));

    let profiling_json = profiling.render_json();
    assert!(profiling_json.contains("\"sample_rate_hz\":48000"));
    assert!(profiling_json.contains("\"block_size\":512"));
    assert!(profiling_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(profiling_json.contains("\"summary\":"));

    let soak_json = soak.render_json();
    assert!(soak_json.contains("\"event_stream_count\":0"));
    assert!(soak_json.contains("\"summary\":"));

    assert!(profiling
        .render_multiline()
        .contains("sample_rate_hz=48000"));
    assert!(soak.render_multiline().contains("event_stream_count=0"));
}

#[test]
fn public_runtime_plugin_discovery_coverage_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let coverage = &observation.plugin_discovery_snapshot.capability_coverage;
    let format_coverage = &observation.plugin_discovery_snapshot.format_coverage;

    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert!(coverage.multi_format_catalog);
    assert_eq!(coverage.audio_effect_count, 1);
    assert_eq!(coverage.instrument_count, 1);
    assert_eq!(coverage.requires_main_thread_for_state_count, 1);
    assert_eq!(coverage.max_parameter_count, 12);
    assert_eq!(format_coverage.len(), 2);
    assert_eq!(format_coverage[0].format, PluginFormat::Clap);
    assert_eq!(format_coverage[0].supports_activate_count, 1);
    assert_eq!(format_coverage[1].format, PluginFormat::Vst3);
    assert_eq!(format_coverage[1].instrument_count, 1);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"discovered_format_count\":2"));
    assert!(observation_json.contains("\"format_coverage\":["));
    assert!(observation_json.contains("\"multi_format_catalog\":true"));
    assert!(observation_json.contains("\"requires_main_thread_for_state_count\":1"));
}

#[test]
fn public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-plugin-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public plugin continuity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public plugin continuity configure should succeed");
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://public-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:public".into()),
            }],
        })
        .expect("public plugin continuity policy should apply");
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:public:plugin-continuity",
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
        "plugin://public-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://public-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        signal_runtime::PluginFaultKind::Crash,
        "shared public crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        signal_runtime::PluginFaultKind::Timeout,
        "shared public timeout",
        Some(3),
    );

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let lifecycle = &supervisor.observation.plugin_lifecycle_snapshot;
    let shared = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared boundary should be visible on public runtime boundary");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:public");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);

    let isolated = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated boundary should remain visible on public runtime boundary");
    assert_eq!(
        isolated.placement_outcome,
        RuntimePluginIsolationOutcome::IsolatedSandbox
    );
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"placement_rule_id\":\"share-verified-clap\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"shared:public\""));
    assert!(rendered.contains("\"shared_boundary_member_count\":2"));
    assert!(rendered.contains("\"continuity_class\":\"Terminal\""));
}

#[test]
fn public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public fault diagnostic safe mode should enable");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("public fault diagnostic render queue should defer");
    assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let deferred_contribution = observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .find(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
        })
        .expect("public fault diagnostic deferred-work contribution should be present");

    assert_eq!(
        observation.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        observation.fault_diagnostic_receipt.interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(deferred_contribution.active);
    assert!(deferred_contribution.event_count >= 1);
    assert_eq!(
        supervisor
            .observation
            .fault_diagnostic_receipt
            .primary_family,
        observation.fault_diagnostic_receipt.primary_family
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"primary_family\":\"DeferredWorkPressure\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(supervisor_json.contains("\"primary_family\":\"DeferredWorkPressure\""));
}

#[test]
fn public_runtime_block_timing_boundary_reports_bounded_runtime_measurements() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 47),
        )
        .expect("public block timing block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[observation.clone()]);

    assert_eq!(
        observation.engine_block_snapshot.last_block_sequence,
        Some(1)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert!(
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
            .expect("public block timing should expose latest execution time")
            > 0
    );
    assert_eq!(
        performance.last_block_execution_time_ns,
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
    );
    assert_eq!(
        performance.last_block_deadline_budget_ns,
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns
    );
    assert_eq!(
        performance.last_block_deadline_pressure,
        observation
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
    assert_eq!(
        supervisor
            .performance_snapshot()
            .last_block_execution_time_ns,
        performance.last_block_execution_time_ns
    );
    assert_eq!(trace.observation_count, 1);
    assert_eq!(
        trace.peak_block_execution_time_ns,
        performance
            .last_block_execution_time_ns
            .expect("trace should preserve the public latest block timing")
    );
    assert_eq!(
        trace.peak_block_budget_utilization_percent,
        performance
            .last_block_budget_utilization_percent
            .expect("trace should preserve public budget utilization")
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"last_block_execution_time_ns\":"));
    assert!(observation_json.contains("\"last_block_deadline_pressure\":"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"engine_block_snapshot\":{"));
    assert!(supervisor_json.contains("\"last_block_deadline_budget_ns\":1000000"));

    let performance_json = performance.render_json();
    assert!(performance_json.contains("\"last_block_execution_time_ns\":"));
    assert!(performance_json.contains("\"last_block_deadline_pressure\":"));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"peak_block_execution_time_ns\":"));
    assert!(trace_json.contains("\"peak_block_budget_utilization_percent\":"));
}

#[test]
fn public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 31),
        )
        .expect("public critical-path block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[observation.clone()]);

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
        .expect("public critical-path lane should resolve to a typed worker-lane summary");
    assert_eq!(
        performance.critical_path_lane_node_count,
        critical_lane_summary.node_count
    );
    assert_eq!(
        performance.critical_path_lane_plugin_backed_node_count,
        critical_lane_summary.plugin_backed_node_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        critical_lane_summary.total_latency_samples
    );
    assert_eq!(
        supervisor.performance_snapshot().critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_hot_latency_group_node_count,
        performance.hot_latency_group_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_critical_path_lane_total_latency_samples,
        performance.critical_path_lane_total_latency_samples
    );

    let performance_json = performance.render_json();
    assert!(performance_json.contains("\"hot_latency_group_node_count\":"));
    assert!(performance_json.contains("\"worker_lane_summaries\":["));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"peak_critical_path_lane\":"));
    assert!(trace_json.contains("\"peak_hot_latency_group_node_count\":"));
}

#[test]
fn public_runtime_interruption_boundary_reports_restartable_runtime_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-interruption".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public interruption boundary handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public interruption boundary configure should succeed");
    runtime
        .start()
        .expect("public interruption boundary start should succeed");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-boundary-sandbox".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

    assert_eq!(
        observation.fault_status.recovery_state,
        RuntimeRecoveryState::Recovering
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(!observation.interruption_summary.rebindable);

    let rendered = observation.render_json();
    assert!(rendered.contains("\"fault_status\":{"));
    assert!(rendered.contains("\"interruption_summary\":{"));
    assert!(rendered.contains("\"class\":\"Restartable\""));
    assert!(rendered.contains("\"primary_fault_cause\":\"WatchdogRestart\""));
}

#[test]
fn public_runtime_interruption_boundary_reports_resumable_deferred_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public interruption boundary safe mode should enable");

    let queue = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public-interruption:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer public interruption boundary queue");

    assert_eq!(
        queue.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(!queue.orchestration.interruption_rebindable);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let deferred = observation
        .last_deferred_service_receipt
        .expect("deferred receipt should be exported on the public observation boundary");
    assert_eq!(
        deferred.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(!deferred.interruption_rebindable);
}

#[test]
fn public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states()
{
    let recorder = RuntimeEventRecorder::default();

    let mut resumable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    resumable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    resumable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut resumable, "graph:public:recording-resumable");
    resumable
        .start()
        .expect("public recording continuity start should succeed");
    resumable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:resumable".into(),
            track_id: "track:public:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .expect("public recording capture should start");
    resumable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 44),
        )
        .expect("public recording block should process");
    resumable
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public recording safe mode should enable");
    let resumable_report = RuntimeObservationReport::capture(&resumable, &recorder);
    assert_eq!(
        resumable_report
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let mut restartable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    restartable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    restartable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut restartable, "graph:public:recording-restartable");
    restartable
        .start()
        .expect("public recording start should succeed");
    restartable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:restartable".into(),
            track_id: "track:public:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .expect("public restartable capture should start");
    restartable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 45),
        )
        .expect("public restartable block should process");
    restartable
        .stop(signal_runtime::StopReason::DeviceReconfigure)
        .expect("public restartable stop should succeed");
    let restartable_report = RuntimeObservationReport::capture(&restartable, &recorder);
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    terminal
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    terminal
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut terminal, "graph:public:recording-terminal");
    terminal
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:terminal".into(),
            track_id: "track:public:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-public-recording-terminal.wav".into(),
        })
        .expect("public terminal capture should start");
    terminal
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 46),
        )
        .expect("public terminal block should process");
    let terminal_error = terminal.finish_recording_capture().unwrap_err();
    assert_eq!(
        terminal_error.kind,
        signal_runtime::RuntimeErrorKind::ResourceUnavailable
    );
    let terminal_report = RuntimeObservationReport::capture(&terminal, &recorder);
    assert_eq!(
        terminal_report.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
}

#[test]
fn public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states(
) {
    let recorder = RuntimeEventRecorder::default();

    let mut resumable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    resumable
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    resumable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut resumable, "graph:public:render-resumable");
    resumable
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:resumable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public resumable render should begin");
    resumable
        .advance_offline_render_execution("render:public:resumable")
        .expect("public resumable render should advance");
    resumable
        .pause_offline_render_execution("render:public:resumable")
        .expect("public resumable render should pause");
    let resumable_report = RuntimeObservationReport::capture(&resumable, &recorder);
    assert_eq!(
        resumable_report
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let mut restartable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    restartable
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    restartable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut restartable, "graph:public:render-restartable");
    restartable
        .start()
        .expect("public render runtime should start");
    restartable
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public restartable render should begin");
    restartable
        .advance_offline_render_execution("render:public:restartable")
        .expect("public restartable render should advance");
    restartable
        .stop(StopReason::DeviceReconfigure)
        .expect("public restartable render should stop");
    restartable
        .restart(RestartRequest { reconfigure: None })
        .expect("public restartable render should restart");
    let restartable_report = RuntimeObservationReport::capture(&restartable, &recorder);
    assert_eq!(
        restartable_report
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    let mut terminal = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    terminal
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    terminal
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut terminal, "graph:public:render-terminal");
    terminal
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-public-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public terminal render should begin");
    let mut terminal_error_observed = false;
    for _ in 0..16 {
        match terminal.advance_offline_render_execution("render:public:terminal") {
            Ok(_) => continue,
            Err(_) => {
                terminal_error_observed = true;
                break;
            }
        }
    }
    assert!(terminal_error_observed);
    let terminal_report = RuntimeObservationReport::capture(&terminal, &recorder);
    assert_eq!(
        terminal_report
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        terminal_report
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
