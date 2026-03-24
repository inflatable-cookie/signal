#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;

use std::fs;

use public_host_edge_media::{public_server_media_fixture_path, write_public_transient_test_wav};
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeProjectionApi, RuntimeTransformArtifactReuseState, RuntimeTransformPersistencePosture,
    SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_transform_artifact_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-transform-artifact".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server transform-artifact handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public server transform-artifact configure should succeed");

    let ready_path = public_server_media_fixture_path("transform-artifact-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-server-transform-artifact-ready".into(),
            content_hash: "host-server-transform-artifact-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-server-transform-artifact-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public server transform-artifact media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-server-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-server-transform-artifact-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public server transform-artifact warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-server-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:host-server-transform-artifact-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public server transform-artifact clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public server transform-artifact transport projection should apply");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .reusable_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .transform_artifact_snapshot
            .transform_persistence
            .persistence_posture,
        RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        report.observation.transform_artifact_snapshot.clips[0].reuse_state,
        RuntimeTransformArtifactReuseState::Reusable
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":1"));
    assert!(rendered.contains("\"reusable_clip_count\":1"));
    assert!(rendered.contains("\"reuse_state\":\"Reusable\""));
    assert!(rendered.contains("\"persistence_posture\":\"AssetScopedTransformPersistence\""));

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
