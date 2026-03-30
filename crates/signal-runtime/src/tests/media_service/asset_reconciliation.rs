use super::super::*;

#[test]
fn runtime_reconciles_media_assets_into_shared_ready_cache_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("media-imported");
    let recorded_path = temp_capture_path("media-recorded");
    write_test_wav(&imported_path);
    write_test_wav(&recorded_path);

    runtime
        .reconcile_media_assets(vec![
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:imported".to_string(),
                content_hash: "imported".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "imported.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:recorded".to_string(),
                content_hash: "recorded".to_string(),
                source_path: recorded_path.display().to_string(),
                file_name: "recorded.wav".to_string(),
                byte_size: fs::metadata(&recorded_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
        ])
        .unwrap();

    let snapshot = runtime.get_media_pipeline_snapshot();
    assert_eq!(snapshot.asset_count, 2);
    assert_eq!(snapshot.ready_asset_count, 2);
    assert_eq!(snapshot.invalid_asset_count, 0);
    assert!(snapshot.assets.iter().all(|asset| {
        asset.state == Some(RuntimeMediaAssetState::Ready) && asset.cache_path.as_deref().is_some()
    }));

    let cached_path = PathBuf::from(
        snapshot.assets[0]
            .cache_path
            .as_deref()
            .expect("cached media should exist"),
    );
    fs::remove_file(&cached_path).unwrap();

    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:imported".to_string(),
            content_hash: "imported".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "imported.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();

    let rebuilt = runtime.get_media_pipeline_snapshot();
    assert_eq!(rebuilt.asset_count, 1);
    assert_eq!(rebuilt.ready_asset_count, 1);
    assert_eq!(rebuilt.assets[0].state, Some(RuntimeMediaAssetState::Ready));
    assert!(rebuilt.assets[0].rebuild_count >= 1);

    let _ = fs::remove_file(imported_path);
    let _ = fs::remove_file(recorded_path);
    if let Some(path) = rebuilt.assets[0].cache_path.as_deref() {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn runtime_media_service_snapshot_tracks_ready_previewable_and_invalidated_assets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let ready_path = temp_capture_path("media-service-ready");
    let missing_path = temp_capture_path("media-service-missing");
    write_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:ready".to_string(),
                content_hash: "ready".to_string(),
                source_path: ready_path.display().to_string(),
                file_name: "ready.wav".to_string(),
                byte_size: fs::metadata(&ready_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
            RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:missing".to_string(),
                content_hash: "missing".to_string(),
                source_path: missing_path.display().to_string(),
                file_name: "missing.wav".to_string(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            },
        ])
        .unwrap();

    let service = runtime.get_media_service_snapshot();
    assert_eq!(service.indexed_asset_count, 2);
    assert_eq!(service.analysis_ready_asset_count, 1);
    assert_eq!(service.waveform_ready_asset_count, 1);
    assert_eq!(service.waveform_pending_asset_count, 0);
    assert_eq!(service.previewable_asset_count, 1);
    assert_eq!(service.invalidated_asset_count, 1);
    assert!(service.invalidation_active);
    assert_eq!(
        service.indexing_state,
        crate::interfaces::RuntimeMediaIndexingState::Invalidated
    );
    assert_eq!(
        service.preview_state,
        crate::interfaces::RuntimeMediaPreviewState::Ready
    );
    assert_eq!(
        service.last_invalidated_asset_id.as_deref(),
        Some("asset:sha256:missing")
    );
    assert!(service.last_invalidation_error.is_some());

    let library = runtime.get_media_library_service_snapshot();
    assert_eq!(library.indexed_asset_count, 2);
    assert_eq!(library.ready_descriptor_count, 1);
    assert_eq!(library.invalidated_descriptor_count, 1);
    assert_eq!(library.unavailable_descriptor_count, 0);
    assert_eq!(library.loudness_ready_descriptor_count, 1);
    assert_eq!(library.character_ready_descriptor_count, 1);
    let ready = library
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:ready")
        .expect("ready descriptor");
    assert_eq!(
        ready.metadata_state,
        crate::RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert_eq!(
        ready.loudness_state,
        crate::RuntimeMediaAnalysisFamilyState::Ready
    );
    assert_eq!(
        ready.character_state,
        crate::RuntimeMediaAnalysisFamilyState::Ready
    );
    assert!(ready.loudness.is_some());
    assert!(ready.character.is_some());
    let missing = library
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:missing")
        .expect("missing descriptor");
    assert_eq!(
        missing.metadata_state,
        crate::RuntimeMediaAnalysisDescriptorState::Invalidated
    );

    let _ = fs::remove_file(ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
