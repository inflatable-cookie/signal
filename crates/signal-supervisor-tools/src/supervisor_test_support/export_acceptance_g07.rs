use std::fs;

use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_plugin::PluginFormat;
use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeClipFadeEnvelope, RuntimeClipGainEnvelope,
    RuntimeClipProcessingRegistration, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
    RuntimeSupervisorReport, RuntimeWarpClipRegistration, RuntimeWarpMode, SignalRuntime,
    TransportProjection,
};

use super::{
    integrated_acceptance_media_fixture_path, sample_backend_breadth_record,
    sample_discovered_type_record, sample_g07_acceptance_host_io,
    sample_g07_external_midi_snapshot, write_g07_acceptance_transient_wav,
};

pub(crate) fn verify_export_json_carries_cross_family_g07_acceptance_evidence() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    runtime
        .handshake(HandshakeRequest {
            client_version: "g07-integrated-acceptance-export".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("g07 integrated acceptance export handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("g07 integrated acceptance export configure should succeed");
    runtime
        .start()
        .expect("g07 integrated acceptance export start should succeed");

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
    ]);
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );

    let preview_path = integrated_acceptance_media_fixture_path("g07-preview-ready");
    write_g07_acceptance_transient_wav(&preview_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:g07-preview-ready".into(),
            content_hash: "g07-preview-ready".into(),
            source_path: preview_path.display().to_string(),
            file_name: "g07-preview-ready.wav".into(),
            byte_size: fs::metadata(&preview_path)
                .expect("g07 acceptance media fixture should exist")
                .len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .expect("g07 integrated acceptance media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:g07-preview-ready".into(),
            media_asset_id: Some("asset:sha256:g07-preview-ready".into()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("g07 integrated acceptance warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:g07-preview-ready".into(),
            media_asset_id: Some("asset:sha256:g07-preview-ready".into()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .expect("g07 integrated acceptance clip processing clip should reconcile");
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("g07 integrated acceptance transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:g07-preview-ready")
        .expect("g07 integrated acceptance media preview should start");

    let recorder = RuntimeEventRecorder::default();
    let mut report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    report
        .observation
        .execution_topology_summary
        .secondary_input_count = 1;
    report
        .observation
        .execution_topology_summary
        .required_secondary_input_count = 1;
    report
        .observation
        .execution_topology_summary
        .bus_connection_count = 1;
    report
        .observation
        .execution_topology_summary
        .auxiliary_path_count = 1;
    report
        .observation
        .execution_topology_summary
        .spatial_node_count = 1;
    report
        .observation
        .execution_topology_summary
        .active_spatial_node_count = 1;
    report
        .observation
        .execution_topology_summary
        .surround_bed_spatial_node_count = 1;
    report
        .observation
        .execution_topology_summary
        .expanded_fallback_spatial_node_count = 1;
    report.observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_g07_acceptance_host_io())
        .with_external_midi_snapshot(sample_g07_external_midi_snapshot());

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"plugin_discovery_snapshot\":{"));
    assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
    assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
    assert!(export.contains("\"default_multichannel_io\":{"));
    assert!(export.contains("\"execution_topology_summary\":{"));
    assert!(export.contains("\"secondary_input_count\":1"));
    assert!(export.contains("\"bus_connection_count\":1"));
    assert!(export.contains("\"spatial_node_count\":1"));
    assert!(export.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(export.contains("\"external_io_snapshot\":{"));
    assert!(export.contains("\"linux_backend_identity\":\"Alsa\""));
    assert!(export.contains("\"linux_backend_portability\":\"Portable\""));
    assert!(export.contains("\"linux_clocking_parity\":\"Portable\""));
    assert!(export.contains("\"linux_duplex_parity\":\"Aligned\""));
    assert!(export.contains("\"linux_endpoint_topology_parity\":\"Portable\""));
    assert!(export.contains("\"external_midi_snapshot\":{"));
    assert!(export.contains("\"provider_name\":\"signal-host-local\""));
    assert!(export.contains("\"control_surface_snapshot\":{"));
    assert!(export.contains("\"graph_state\":\"Guarded\""));
    assert!(export.contains("\"supports_widened_expression\":true"));
    assert!(export.contains("\"advanced_hardware_snapshot\":{"));
    assert!(export.contains("\"scripting_safe_posture\":\"Guarded\""));
    assert!(export.contains("\"feedback_channel_posture\":\"Guarded\""));
    assert!(export.contains("\"stretch_engine_snapshot\":{"));
    assert!(export.contains("\"marker_analysis_snapshot\":{"));
    assert!(export.contains("\"transform_artifact_snapshot\":{"));
    assert!(export.contains("\"preview_transform_snapshot\":{"));
    assert!(export.contains("\"tempo_assist_ready_clip_count\":1"));
    assert!(export.contains("\"reusable_clip_count\":1"));
    assert!(export.contains("\"active_audition_clip_count\":1"));

    let _ = fs::remove_file(&preview_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:g07-preview-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
