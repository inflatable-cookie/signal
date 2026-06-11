use super::super::super::super::super::*;
use std::fs;

#[test]
fn local_host_shared_report_surfaces_runtime_preview_transform_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake");
    host.runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure");

    let imported_path = unique_test_path("local-host-preview-transform", "wav");
    write_test_wav(&imported_path);
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:local-preview-transform".into(),
            content_hash: "local-preview-transform".into(),
            source_path: imported_path.display().to_string(),
            file_name: "local-preview-transform.wav".into(),
            byte_size: fs::metadata(&imported_path).expect("wav metadata").len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 16,
        }])
        .expect("media reconcile");
    host.runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:local-preview-transform".into(),
            media_asset_id: Some("asset:sha256:local-preview-transform".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 128,
        }])
        .expect("warp reconcile");
    host.runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:local-preview-transform".into(),
            media_asset_id: Some("asset:sha256:local-preview-transform".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 128,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("clip processing reconcile");
    host.runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("transport projection");
    host.runtime
        .start_media_preview("asset:sha256:local-preview-transform")
        .expect("preview transform media preview should start");

    let report = host.supervisor_report();
    assert_eq!(report.observation.preview_transform_snapshot.clip_count, 1);
    assert_eq!(
        report.observation.preview_transform_snapshot.active_audition_clip_count,
        1
    );
    assert_eq!(report.observation.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        report.observation.preview_transform_snapshot.artifact_backed_clip_count,
        1
    );

    let _ = fs::remove_file(&imported_path);
    if let Some(path) = host
        .runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
