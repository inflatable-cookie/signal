use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_plugin::PluginFormat;
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeSupervisorReport,
    SafeModeRequest, SignalRuntime,
};

use super::{
    assert_integrated_acceptance_export, integrated_acceptance_media_fixture_path,
    sample_au_breadth_record, sample_backend_breadth_record, sample_discovered_type_record,
    sample_integrated_acceptance_host_io, write_integrated_acceptance_test_wav,
};

pub(crate) fn verify_export_json_carries_cross_family_integrated_acceptance_evidence() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "integrated-acceptance-export".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("integrated acceptance export handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("integrated acceptance export configure should succeed");
    runtime
        .start()
        .expect("integrated acceptance export start should succeed");

    runtime.record_watchdog_restart(signal_runtime::WatchdogRestartRecord {
        sandbox_id: "integrated-acceptance-sandbox".into(),
        trigger: signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(signal_runtime::WatchdogRestartRecord {
        sandbox_id: "integrated-acceptance-sandbox".into(),
        trigger: signal_runtime::RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });

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
    let scan_handle = runtime.record_plugin_scan_request(&signal_runtime::PluginScanRequest {
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
    runtime.record_plugin_sandbox_spec(&signal_runtime::PluginSandboxSpec {
        sandbox_id: "integrated-acceptance-vst3".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:export-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "integrated-acceptance-vst3",
        signal_runtime::PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("integrated acceptance export safe mode should enable");
    let deferred = runtime
        .render_offline_queue(vec![signal_runtime::RuntimeOfflineRenderRequest {
            request_id: "render:integrated-acceptance".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("integrated acceptance export queue should defer in safe mode");
    assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

    let ready_path = integrated_acceptance_media_fixture_path("ready");
    let missing_path = integrated_acceptance_media_fixture_path("missing");
    write_integrated_acceptance_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:integrated-ready".into(),
                content_hash: "integrated-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "integrated-ready.wav".into(),
                byte_size: std::fs::metadata(&ready_path)
                    .expect("integrated acceptance media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:integrated-missing".into(),
                content_hash: "integrated-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "integrated-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("integrated acceptance media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:integrated-ready")
        .expect("integrated acceptance media preview should start");

    let recorder = RuntimeEventRecorder::default();
    let mut report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    report.observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_integrated_acceptance_host_io());
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert_integrated_acceptance_export(&export);

    let _ = std::fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:integrated-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = std::fs::remove_file(path);
    }
}
