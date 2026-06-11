#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;

use std::fs;

use public_host_edge_media::{public_local_media_fixture_path, write_public_transient_test_wav};
use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeProjectionApi, RuntimeTempoAssistPosture, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_marker_analysis_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-marker-analysis".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local marker-analysis handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local marker-analysis configure should succeed");

    let ready_path = public_local_media_fixture_path("marker-analysis-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:host-local-marker-analysis-ready".into(),
            content_hash: "host-local-marker-analysis-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "host-local-marker-analysis-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public local marker-analysis media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:host-local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-local-marker-analysis-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public local marker-analysis warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:host-local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:host-local-marker-analysis-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public local marker-analysis clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public local marker-analysis transport projection should apply");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .ready_clip_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .warp_marker_count
            > 0
    );
    assert!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .transient_anchor_count
            > 0
    );
    assert_eq!(
        report
            .observation
            .observation
            .marker_analysis_snapshot
            .clips[0]
            .tempo_assist_posture,
        RuntimeTempoAssistPosture::Ready
    );

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
