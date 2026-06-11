use signal_plugin::PluginFormat;
use signal_runtime::{
    HandshakeRequest, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage,
    PluginScanRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimePluginFormatPlatformCoverageRecord,
    RuntimePluginHostPlatform, RuntimePluginIsolationOutcome, RuntimePluginParityBand,
    RuntimePluginPlacementPolicy, RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher,
    RuntimeProjectionApi, RuntimeSupervisorReport, SignalRuntime, StopReason,
};

#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;
#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_graph_foundation_support::sample_lv2_breadth_record;
use public_contract_boundary_plugin_records_core_support::{
    sample_backend_breadth_record, sample_discovered_type_record,
};

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
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
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
    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

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
}
