#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;

use std::fs;

use public_host_edge_media::{public_server_media_fixture_path, write_public_transient_test_wav};
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimePreviewBrowserQueuePosture, RuntimePreviewOutputRoutingPosture,
    RuntimePreviewTransformSchedulingOutcome, RuntimePreviewTransformServiceClass,
    RuntimeProjectionApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_preview_transform_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-preview-transform".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server preview-transform handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server preview-transform configure should succeed");

    let ready_path = public_server_media_fixture_path("preview-transform-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-preview-transform-ready".into(),
            content_hash: "host-server-preview-transform-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-preview-transform-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public server preview-transform media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-server-preview-transform-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server preview-transform warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-preview-transform".into(),
            media_asset_id: Some("asset:sha256:host-server-preview-transform-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server preview-transform clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server preview-transform transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:host-server-preview-transform-ready")
        .expect("public server preview-transform media preview should start");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.preview_transform_snapshot.clip_count, 1);
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert_eq!(
        report
            .observation
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        report.observation.preview_transform_snapshot.clips[0].service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"preview_transform_snapshot\":{"));
    assert!(rendered.contains("\"active_audition_clip_count\":1"));
    assert!(rendered.contains("\"artifact_backed_clip_count\":1"));
    assert!(rendered.contains("\"service_class\":\"ArtifactBacked\""));
    assert!(rendered.contains("\"routing_posture\":\"GuardedPreviewOutputRouting\""));
    assert!(rendered.contains("\"queue_posture\":\"SingleActivePreviewQueue\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
