use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeEventRecorder,
    RuntimeLv2ExtensionNegotiationState, RuntimeLv2PatchExchangePosture,
    RuntimeLv2PreparedNegotiationRecord, RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture,
    RuntimeObservationReport, RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimePluginScanDiagnosticKind,
    RuntimePluginScanDiagnosticRecord, RuntimeSupervisorReport, SignalRuntime,
};

#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;

use public_contract_boundary_graph_foundation_support::sample_lv2_breadth_record;

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
    }],
    );
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results_with_diagnostics(
        scan_handle,
        vec![sample_lv2_breadth_record()],
        vec![RuntimePluginScanDiagnosticRecord {
            format: PluginFormat::Lv2,
            root: "/usr/lib/lv2".into(),
            bundle_root: "/usr/lib/lv2/Broken Public LV2.lv2".into(),
            manifest_path: Some("/usr/lib/lv2/Broken Public LV2.lv2/manifest.ttl".into()),
            plugin_type_id: Some("plugin:lv2:unsupported-public".into()),
            kind: RuntimePluginScanDiagnosticKind::UnsupportedRequiredFeature,
        }],
    );
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
    runtime.record_plugin_sandbox_lv2_prepared_negotiation(
        "public-lv2-sandbox",
        RuntimeLv2PreparedNegotiationRecord {
            worker_posture: RuntimeLv2WorkerPosture::WorkerRequiredAvailable,
            urid_negotiation_posture: RuntimeLv2UridNegotiationPosture::Negotiated,
            patch_exchange_posture: RuntimeLv2PatchExchangePosture::Supported,
            extension_negotiation_state: RuntimeLv2ExtensionNegotiationState::Negotiated,
        },
    );
    runtime.record_plugin_sandbox_transport(
        "public-lv2-sandbox",
        "lease-public-lv2",
        "region-public-lv2",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public lv2 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

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
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.discovery_diagnostic_count),
        Some(1)
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
    assert_eq!(
        sandbox
            .lv2_prepared_negotiation
            .as_ref()
            .map(|record| record.extension_negotiation_state),
        Some(RuntimeLv2ExtensionNegotiationState::Negotiated)
    );
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

}
