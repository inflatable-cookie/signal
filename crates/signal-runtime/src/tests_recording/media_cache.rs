use super::*;

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
