use signal_plugin::PluginFormat;
use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeEventRecorder,
    RuntimeObservationReport, RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeSupervisorReport, SignalRuntime,
};

#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_plugin_records_core_support::{
    sample_au_breadth_record, sample_backend_breadth_record, sample_discovered_type_record,
};

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
    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

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
}
