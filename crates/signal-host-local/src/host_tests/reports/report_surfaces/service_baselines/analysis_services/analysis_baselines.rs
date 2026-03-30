use super::super::super::super::super::*;

#[test]
fn local_host_shared_report_surfaces_runtime_marker_analysis_baseline() {
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

    let imported_path = unique_test_path("local-host-marker-analysis", "wav");
    write_test_wav(&imported_path);
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:local-marker-analysis".into(),
            content_hash: "local-marker-analysis".into(),
            source_path: imported_path.display().to_string(),
            file_name: "local-marker-analysis.wav".into(),
            byte_size: fs::metadata(&imported_path).expect("wav metadata").len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 16,
        }])
        .expect("media reconcile");
    host.runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:local-marker-analysis".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("warp reconcile");
    host.runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:local-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:local-marker-analysis".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
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

    let report = host.supervisor_report();
    assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(report.observation.marker_analysis_snapshot.ready_clip_count, 1);
    assert_eq!(
        report.observation.marker_analysis_snapshot.tempo_assist_ready_clip_count,
        1
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":1"));
    assert!(rendered.contains("\"tempo_assist_ready_clip_count\":1"));

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

#[test]
fn local_host_shared_report_surfaces_runtime_transform_artifact_baseline() {
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

    let imported_path = unique_test_path("local-host-transform-artifact", "wav");
    write_test_wav(&imported_path);
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:local-transform-artifact".into(),
            content_hash: "local-transform-artifact".into(),
            source_path: imported_path.display().to_string(),
            file_name: "local-transform-artifact.wav".into(),
            byte_size: fs::metadata(&imported_path).expect("wav metadata").len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 16,
        }])
        .expect("media reconcile");
    host.runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:local-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:local-transform-artifact".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("warp reconcile");
    host.runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:local-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:local-transform-artifact".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
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

    let report = host.supervisor_report();
    assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(report.observation.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(
        report.observation.transform_artifact_snapshot.reusable_clip_count,
        1
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":1"));
    assert!(rendered.contains("\"reusable_clip_count\":1"));

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
