#[path = "support/public_contract_boundary_graph_bus.rs"]
mod public_contract_boundary_graph_bus_support;
#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;
#[path = "support/public_contract_boundary_graph_multichannel.rs"]
mod public_contract_boundary_graph_multichannel_support;
#[path = "support/public_contract_boundary_graph_plugin_surface.rs"]
mod public_contract_boundary_graph_plugin_surface_support;
#[path = "support/public_contract_boundary_plugin_records_complex.rs"]
mod public_contract_boundary_plugin_records_complex_support;
#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_graph_bus_support::apply_public_multi_bus_graph;
use public_contract_boundary_graph_foundation_support::{
    apply_public_capture_graph, apply_public_plugin_continuity_graph, apply_public_render_graph,
    sample_lv2_breadth_record,
};
use public_contract_boundary_graph_multichannel_support::{
    apply_public_multichannel_graph, apply_public_sidechain_graph,
};
use public_contract_boundary_graph_plugin_surface_support::{
    apply_public_complex_io_graph, record_public_plugin_sandbox_ready,
};
use public_contract_boundary_plugin_records_complex_support::{
    sample_complex_bus_fx_record, sample_complex_multi_output_record,
};
use public_contract_boundary_plugin_records_core_support::{
    sample_au_breadth_record, sample_backend_breadth_record, sample_discovered_type_record,
};
use signal_graph::synthetic_stereo_block;
use signal_plugin::{PluginFeature, PluginFormat};
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RestartRequest,
    RuntimeAuxiliaryPathKind, RuntimeBusIntent, RuntimeBusRole, RuntimeCanonicalChannelLayout,
    RuntimeConfig, RuntimeConfigRequest, RuntimeDynamicBusNegotiationPosture, RuntimeEventRecorder,
    RuntimeExternalMidiDiscoveryState, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeLv2ExtensionNegotiationState, RuntimeLv2PatchExchangePosture,
    RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture, RuntimeObservationApi,
    RuntimeObservationReport, RuntimeOfflineRenderContractPreview,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderRequest,
    RuntimePluginBusCapableFxClass, RuntimePluginFormatPlatformCoverageRecord,
    RuntimePluginHostPlatform, RuntimePluginIsolationOutcome,
    RuntimePluginNegotiationFallbackOutcome, RuntimePluginParityBand,
    RuntimePluginPinGroupIdentity, RuntimePluginPinMatrixPosture, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState,
    RuntimeSecondaryInputAttachmentPolicy, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSupervisorReport, RuntimeWatchdogTrigger,
    SafeModeRequest, SignalRuntime, StopReason, WatchdogRestartRecord,
};

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
fn public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_backend_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        instance_id: "instance:public:vst3".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-vst3-sandbox",
        "lease-public-vst3",
        "region-public-vst3",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public vst3 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Vst3])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:vst3:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Vst3
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-vst3-sandbox")
        .expect("public vst3 sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:vst3:public-instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Vst3\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
}

#[test]
fn public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_au_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-au-sandbox".into(),
        plugin_type_id: "plugin:au:public-instrument".into(),
        instance_id: "instance:public:au".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-au-sandbox",
        "lease-public-au",
        "region-public-au",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public au transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Au])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:au:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Au
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-au-sandbox")
        .expect("public au sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:au:public-instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Au\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:au:public-instrument\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:au:public-instrument\""));
}

#[test]
fn public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime.record_plugin_format_platform_coverage(
        vec![RuntimePluginFormatPlatformCoverageRecord {
        format: PluginFormat::Lv2,
        supported_platforms: vec![RuntimePluginHostPlatform::Linux],
        unsupported_platforms: vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ],
        linux_parity_band: RuntimePluginParityBand::Portable,
        linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
        linux_strict_sandbox_default: true,
        summary:
            "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                .into(),
    }],
    );
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_lv2_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:public-linux-synth".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-lv2-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-lv2-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-lv2-sandbox".into(),
        plugin_type_id: "plugin:lv2:public-linux-synth".into(),
        instance_id: "instance:public:lv2".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-lv2-sandbox",
        "lease-public-lv2",
        "region-public-lv2",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public lv2 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Lv2])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:lv2:public-linux-synth"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Lv2
    );
    let parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public lv2 parity should be visible");
    assert_eq!(
        parity.supported_platforms,
        vec![RuntimePluginHostPlatform::Linux]
    );
    assert_eq!(
        parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-lv2-sandbox")
        .expect("public lv2 sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:lv2:public-linux-synth")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);
    assert_eq!(observation.lv2_extension_snapshot.plugin_type_count, 1);
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        1
    );
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .urid_negotiated_type_count,
        1
    );
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        1
    );
    let lv2_extension = observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:public-linux-synth")
        .expect("public lv2 extension record should be visible");
    assert_eq!(
        lv2_extension.worker_posture,
        RuntimeLv2WorkerPosture::WorkerRequiredAvailable
    );
    assert_eq!(
        lv2_extension.urid_negotiation_posture,
        RuntimeLv2UridNegotiationPosture::Negotiated
    );
    assert_eq!(
        lv2_extension.patch_exchange_posture,
        RuntimeLv2PatchExchangePosture::Supported
    );
    assert_eq!(
        lv2_extension.extension_negotiation_state,
        RuntimeLv2ExtensionNegotiationState::Negotiated
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Lv2\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:lv2:public-linux-synth\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));
    assert!(observation_json.contains("\"supported_platforms\":[\"Linux\"]"));
    assert!(observation_json.contains("\"lv2_extension_snapshot\":{"));
    assert!(observation_json.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(observation_json.contains("\"patch_exchange_posture\":\"Supported\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:lv2:public-linux-synth\""));
    assert!(supervisor_json.contains("\"lv2_extension_snapshot\":{"));
}

#[test]
fn public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime.record_plugin_format_platform_coverage(vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Au,
            supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Unsupported,
            linux_preferred_sandbox_outcome: None,
            linux_strict_sandbox_default: false,
            summary: "platforms=MacOs linux=Unsupported unsupported=Linux/Windows".into(),
        },
    ]);
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "~/.vst3".into(),
            "~/Library/Audio/Plug-Ins/Components".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
            sample_au_breadth_record(),
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-parity-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-parity-vst3-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-parity-vst3-sandbox".into(),
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        instance_id: "instance:public:parity:vst3".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-parity-vst3-sandbox",
        "lease-public-parity-vst3",
        "region-public-parity-vst3",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public parity vst3 transport attached".into()),
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-parity-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-parity-au-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-parity-au-sandbox".into(),
        plugin_type_id: "plugin:au:public-instrument".into(),
        instance_id: "instance:public:parity:au".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-parity-au-sandbox",
        "lease-public-parity-au",
        "region-public-parity-au",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public parity au transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.parity_coverage.len(),
        3
    );
    let clap_parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("clap parity should be exported on the public runtime boundary");
    assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(clap_parity.discovered_type_count, 1);
    assert_eq!(
        clap_parity.supported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let au_parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("au parity should be exported on the public runtime boundary");
    assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
    assert_eq!(au_parity.discovered_type_count, 1);
    assert_eq!(au_parity.sandbox_count, 1);
    assert_eq!(au_parity.ready_sandbox_count, 1);
    assert_eq!(au_parity.active_transport_count, 1);
    assert_eq!(
        au_parity.supported_platforms,
        vec![RuntimePluginHostPlatform::MacOs]
    );
    assert_eq!(
        au_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let vst3_parity = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("vst3 lifecycle parity should be exported on the public runtime boundary");
    assert_eq!(vst3_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(vst3_parity.sandbox_count, 1);
    assert_eq!(vst3_parity.ready_sandbox_count, 1);
    assert_eq!(vst3_parity.active_transport_count, 1);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"parity_coverage\":["));
    assert!(observation_json.contains("\"parity_band\":\"Portable\""));
    assert!(observation_json.contains("\"parity_band\":\"Guarded\""));
    assert!(observation_json.contains("\"supported_platforms\":[\"MacOs\",\"Linux\",\"Windows\"]"));
    assert!(observation_json.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"parity_coverage\":["));
}

#[test]
fn public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-plugin-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux parity configure should succeed");
    runtime.record_plugin_format_platform_coverage(vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                    .into(),
        },
    ]);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![
                RuntimePluginPlacementRule {
                    rule_id: "public-linux-share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("linux:clap".into()),
                },
                RuntimePluginPlacementRule {
                    rule_id: "public-linux-inline-vst3".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Vst3),
                    outcome: RuntimePluginIsolationOutcome::InProcess,
                    sandbox_group_key: None,
                },
            ],
        })
        .expect("public linux parity placement policy should apply");

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into(), "~/.lv2".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
            sample_lv2_breadth_record(),
        ],
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-clap-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:public-boundary".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-linux-clap-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "public-linux-clap-sandbox",
        "lease-public-linux-clap",
        "region-public-linux-clap",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_recovery_cycle(
        "public-linux-vst3-sandbox",
        signal_runtime::RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-linux-vst3-sandbox",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:public-linux-synth".into()),
    });
    runtime.record_plugin_sandbox_fault(
        "public-linux-lv2-sandbox",
        signal_runtime::PluginFaultKind::Crash,
        "public linux lv2 sandbox fault",
        Some(3),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let clap_discovery = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("public linux clap parity should be visible");
    assert_eq!(
        clap_discovery.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(clap_discovery.linux_supported);
    assert_eq!(
        clap_discovery.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap_discovery.linux_strict_sandbox_default);

    let vst3_lifecycle = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("public linux vst3 parity should be visible");
    assert_eq!(
        vst3_lifecycle.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(vst3_lifecycle.linux_supported);
    assert_eq!(vst3_lifecycle.in_process_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.restarting_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.rebindable_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.prepare_capable_type_count, 1);

    let lv2_lifecycle = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public linux lv2 parity should be visible");
    assert_eq!(
        lv2_lifecycle.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(lv2_lifecycle.linux_supported);
    assert_eq!(lv2_lifecycle.faulted_sandbox_count, 1);
    assert_eq!(
        lv2_lifecycle.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"linux_parity_band\":\"Portable\""));
    assert!(observation_json.contains("\"linux_supported\":true"));
    assert!(observation_json.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(observation_json.contains("\"restarting_sandbox_count\":1"));
    assert!(observation_json.contains("\"faulted_sandbox_count\":1"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"linux_strict_sandbox_default\":true"));
}

#[test]
fn public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.advanced_hardware_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Unavailable
    );
    assert_eq!(
        unavailable.advanced_hardware_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.advanced_hardware_snapshot.device_count, 0);
    assert!(unavailable.advanced_hardware_snapshot.devices.is_empty());

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
    };
    let advanced_hardware_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-advanced-hardware".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:advanced-hardware:1".into(),
            device_name: "Advanced Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Advanced Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:input".into(),
                endpoint_name: "Advanced Surface Input".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:output".into(),
                endpoint_name: "Advanced Surface Output".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-advanced-hardware state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    };

    let observation = unavailable
        .clone()
        .with_external_midi_snapshot(advanced_hardware_graph.clone());
    assert_eq!(
        observation.advanced_hardware_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Enumerated
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Guarded
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.provider_name,
        "public-advanced-hardware"
    );
    assert_eq!(observation.advanced_hardware_snapshot.device_count, 1);
    assert_eq!(
        observation.advanced_hardware_snapshot.portable_device_count,
        0
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.guarded_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .feedback_channel_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .display_transport_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .motor_transport_device_count,
        0
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .haptic_transport_device_count,
        0
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .scene_mapping_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .feedback_page_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .safe_action_graph_device_count,
        1
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].scripting_safe_posture,
        signal_runtime::RuntimeScriptingSafeDevicePolicyPosture::Guarded
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_channel_posture,
        signal_runtime::RuntimeGuardedFeedbackChannelPosture::Guarded
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_display_feedback
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_bank_navigation
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_macro_triggers
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].display_transport_posture,
        signal_runtime::RuntimeDisplayTransportPosture::GuardedDisplay
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].display_content_class,
        signal_runtime::RuntimeDisplayContentClass::GuardedVendorDisplay
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].motor_transport_posture,
        signal_runtime::RuntimeMotorTransportPosture::NoMotorTransport
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].haptic_transport_posture,
        signal_runtime::RuntimeHapticTransportPosture::NoHapticTransport
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_authority,
        signal_runtime::RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_outcome,
        signal_runtime::RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].scene_mapping_posture,
        signal_runtime::RuntimeSceneMappingPosture::GuardedSceneMapping
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_page_posture,
        signal_runtime::RuntimeFeedbackPagePosture::GuardedFeedbackPages
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_page_class,
        signal_runtime::RuntimeFeedbackPageClass::GuardedVendorPage
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].safe_action_graph_posture,
        signal_runtime::RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].action_authority,
        signal_runtime::RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].safe_action_outcome,
        signal_runtime::RuntimeSafeActionOutcome::CollapseToGuardedAction
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .action_classes,
        vec![
            signal_runtime::RuntimeAdvancedHardwareActionClass::DisplayFeedback,
            signal_runtime::RuntimeAdvancedHardwareActionClass::BankNavigation,
            signal_runtime::RuntimeAdvancedHardwareActionClass::MacroTrigger,
            signal_runtime::RuntimeAdvancedHardwareActionClass::DeviceStateObservation,
        ]
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(observation_json.contains("\"graph_state\":\"Guarded\""));
    assert!(observation_json.contains("\"provider_name\":\"public-advanced-hardware\""));
    assert!(observation_json.contains("\"feedback_channel_device_count\":1"));
    assert!(observation_json.contains("\"display_transport_device_count\":1"));
    assert!(observation_json.contains("\"scene_mapping_device_count\":1"));
    assert!(observation_json.contains("\"feedback_page_device_count\":1"));
    assert!(observation_json.contains("\"safe_action_graph_device_count\":1"));
    assert!(observation_json.contains("\"scripting_safe_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"feedback_channel_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"display_transport_posture\":\"GuardedDisplay\""));
    assert!(observation_json.contains("\"display_content_class\":\"GuardedVendorDisplay\""));
    assert!(observation_json.contains("\"feedback_authority\":\"RuntimeDefault\""));
    assert!(observation_json.contains("\"feedback_outcome\":\"CollapseToGuardedFeedback\""));
    assert!(observation_json.contains("\"scene_mapping_posture\":\"GuardedSceneMapping\""));
    assert!(observation_json.contains("\"feedback_page_posture\":\"GuardedFeedbackPages\""));
    assert!(observation_json.contains("\"feedback_page_class\":\"GuardedVendorPage\""));
    assert!(observation_json.contains("\"safe_action_graph_posture\":\"GuardedSafeActionGraph\""));
    assert!(observation_json.contains("\"action_authority\":\"RuntimeDefault\""));
    assert!(observation_json.contains("\"safe_action_outcome\":\"CollapseToGuardedAction\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(advanced_hardware_graph);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(supervisor_json.contains("\"supports_display_feedback\":true"));
    assert!(supervisor_json.contains("\"supports_macro_triggers\":true"));
    assert!(supervisor_json.contains("\"display_transport_posture\":\"GuardedDisplay\""));
    assert!(supervisor_json.contains("\"feedback_outcome\":\"CollapseToGuardedFeedback\""));
    assert!(supervisor_json.contains("\"scene_mapping_posture\":\"GuardedSceneMapping\""));
    assert!(supervisor_json.contains("\"feedback_page_posture\":\"GuardedFeedbackPages\""));
    assert!(supervisor_json.contains("\"safe_action_outcome\":\"CollapseToGuardedAction\""));
}

#[test]
fn public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multichannel-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:public:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_discovered_type_record()]);
    let recorder = RuntimeEventRecorder::default();

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    let track_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "surround-track")
        .expect("surround-track node should be present");
    assert_eq!(
        track_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        track_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(track_node.input_bus_intent, RuntimeBusIntent::MainProgram);
    assert_eq!(track_node.output_bus_intent, RuntimeBusIntent::MainProgram);

    let send_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "analysis-send")
        .expect("analysis-send node should be present");
    assert_eq!(
        send_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(
        send_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Quad)
    );
    assert_eq!(send_node.output_bus_intent, RuntimeBusIntent::AuxSend);

    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .input_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );

    let rendered = observation.render_json();
    assert!(rendered.contains("\"execution_topology_summary\":{"));
    assert!(rendered.contains("\"canonical_layout\":\"Surround5_1\""));
    assert!(rendered.contains("\"output_bus_intent\":\"AuxSend\""));
    assert!(rendered.contains("\"default_multichannel_io\":{"));
}

#[test]
fn public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-sidechain-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:public:sidechain");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:public:sidechain".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:public-boundary".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:sidechain",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:public:sidechain",
        "lease-public-sidechain",
        "region-public-sidechain",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public sidechain transport attached".into()),
    );
    runtime
        .process_engine_block(
            2,
            3,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime sidechain block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.secondary_input_count, 1);
    assert_eq!(topology.required_secondary_input_count, 1);
    let route = &topology.secondary_inputs[0];
    assert_eq!(route.source_id, "kick-sidechain");
    assert_eq!(route.target_id, "compressor");
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let compressor = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "compressor")
        .expect("compressor node should be present");
    let node_secondary_input = compressor
        .secondary_input
        .as_ref()
        .expect("compressor should carry sidechain receipt");
    assert_eq!(node_secondary_input.source_id, "kick-sidechain");
    assert_eq!(
        node_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );

    let stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .find(|chain| chain.stage_count == 1)
        .and_then(|chain| chain.stages.first())
        .expect("plugin chain stage should be present");
    let stage_secondary_input = stage
        .secondary_input
        .as_ref()
        .expect("plugin chain stage should carry sidechain receipt");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "compressor");

    let rendered = observation.render_json();
    assert!(rendered.contains("\"secondary_input_count\":1"));
    assert!(rendered.contains("\"target_kind\":\"NodeInput\""));
    assert!(rendered.contains("\"target_kind\":\"PluginInput\""));
    assert!(rendered.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
}

#[test]
fn public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multi-bus-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:public:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime multi-bus block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.bus_connection_count, 5);
    assert_eq!(topology.auxiliary_path_count, 3);
    assert!(topology.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == RuntimeBusRole::AuxSend
            && connection.target_bus_role == RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(RuntimeAuxiliaryPathKind::SendReturn)
            && connection.auxiliary_path_id.as_deref() == Some("send_return:fx:plate")
    }));
    assert!(topology.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:master"
            && path.path_kind == RuntimeAuxiliaryPathKind::Submix
            && path.bus_role == RuntimeBusRole::Submix
    }));
    assert_eq!(observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(observation.metering_snapshot.auxiliary_path_count, 3);
    assert!(observation
        .metering_snapshot
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id == "return-fx:bus:mix:master->output-main:bus:mix:master"
        }));

    let rendered = observation.render_json();
    assert!(rendered.contains("\"bus_connection_count\":5"));
    assert!(rendered.contains("\"auxiliary_path_count\":3"));
    assert!(rendered.contains("\"connection_id\":\"send-fx:bus:fx:plate->return-fx:bus:fx:plate\""));
    assert!(rendered.contains("\"auxiliary_path_id\":\"send_return:fx:plate\""));
}

#[test]
fn public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-complex-io-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:public:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_complex_multi_output_record(),
            sample_complex_bus_fx_record(),
        ],
    );
    runtime
        .apply_plugin_node_render_batch(signal_runtime::PluginNodeRenderBatch {
            graph_id: "graph:public:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:public:multiout".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                },
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:public:bus-fx".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 16,
                    tail_samples: 24,
                    bypassed: false,
                },
            ],
        })
        .expect("public complex io render batch should apply");
    runtime
        .process_engine_block(
            4,
            6,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3),
        )
        .expect("public runtime complex io block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.capability_coverage.complex_io_type_count, 2);
    assert_eq!(
        discovery.capability_coverage.multi_output_instrument_count,
        1
    );
    assert_eq!(discovery.capability_coverage.bus_capable_fx_count, 1);
    assert_eq!(
        discovery
            .capability_coverage
            .max_complex_io_port_group_count,
        4
    );
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-multiout"
            && record.complex_io_summary.multi_output_instrument
            && record.complex_io_summary.instrument_output_group_count == 2
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-bus-fx"
            && record.complex_io_summary.bus_capable_fx_class
                == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            && record.complex_io_summary.secondary_input_group_count == 1
    }));
    let pin_matrix = &observation.plugin_pin_matrix_snapshot;
    assert_eq!(pin_matrix.plugin_type_count, 2);
    assert_eq!(pin_matrix.negotiated_type_count, 2);
    assert_eq!(pin_matrix.dynamic_negotiated_type_count, 2);
    let multiout_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-multiout")
        .expect("public multi-output pin matrix record should be visible");
    assert_eq!(
        multiout_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::RoutePrimaryOnly
    );
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::PrimaryProgramPath));
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SecondaryProgramPath));
    let bus_fx_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-bus-fx")
        .expect("public bus-fx pin matrix record should be visible");
    assert_eq!(
        bus_fx_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
    );
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SidechainPath));
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::AuxReturnPath));

    let plugin_chain = &observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.chain_count, 1);
    assert_eq!(plugin_chain.stage_count, 2);
    let multiout_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-multiout")
        .expect("multi-output stage should be present");
    assert!(multiout_stage.complex_io_summary.has_complex_topology);
    assert!(multiout_stage.complex_io_summary.multi_output_instrument);
    assert_eq!(
        multiout_stage
            .complex_io_summary
            .instrument_output_group_count,
        2
    );
    let bus_fx_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-bus-fx")
        .expect("bus fx stage should be present");
    assert_eq!(
        bus_fx_stage.complex_io_summary.bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert_eq!(
        bus_fx_stage.complex_io_summary.secondary_input_group_count,
        1
    );

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public:complex-io".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("public complex io offline preview should build");
    assert_eq!(preview.chain_contract.complex_io_stage_count, 2);
    assert_eq!(
        preview.chain_contract.multi_output_instrument_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.bus_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.sidechain_capable_fx_stage_count, 1);
    assert!(preview
        .chain_contract
        .complex_io_stages
        .iter()
        .any(|stage| {
            stage.plugin_type_id.as_deref() == Some("plugin:vst3:public-multiout")
                && stage.topology.multi_output_instrument
        }));
    assert!(preview
        .chain_contract
        .complex_io_stages
        .iter()
        .any(|stage| {
            stage.plugin_type_id.as_deref() == Some("plugin:vst3:public-bus-fx")
                && stage.topology.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        }));

    let supervisor = signal_runtime::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_pin_matrix_snapshot\":{"));
    assert!(rendered.contains("\"complex_io_summary\":{"));
    assert!(rendered.contains("\"pin_matrix_posture\":\"Negotiated\""));
    assert!(rendered.contains("\"dynamic_bus_negotiation_posture\":\"Negotiated\""));
    assert!(rendered.contains("\"multi_output_instrument\":true"));
    assert!(rendered.contains("\"bus_capable_fx_class\":\"SendReturnCapableFx\""));
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
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-boundary-sandbox".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
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
