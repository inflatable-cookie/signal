use super::super::*;

#[test]
fn runtime_media_preview_clears_when_previewed_asset_is_invalidated() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let ready_path = temp_capture_path("media-preview-ready");
    write_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:previewed".to_string(),
            content_hash: "previewed".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "previewed.wav".to_string(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();

    runtime
        .start_media_preview("asset:sha256:previewed")
        .expect("preview should start for ready media");
    let previewing = runtime.get_media_service_snapshot();
    assert_eq!(
        previewing.preview_state,
        crate::RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        previewing.previewing_asset_id.as_deref(),
        Some("asset:sha256:previewed")
    );

    fs::remove_file(&ready_path).unwrap();
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:previewed".to_string(),
            content_hash: "previewed".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "previewed.wav".to_string(),
            byte_size: 0,
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();

    let invalidated = runtime.get_media_service_snapshot();
    assert_eq!(
        invalidated.preview_state,
        crate::RuntimeMediaPreviewState::Invalidated
    );
    assert_eq!(invalidated.previewing_asset_id, None);
    assert!(invalidated.last_preview_error.is_some());
}

#[test]
fn runtime_media_service_recovers_after_invalidation_and_supports_preview_again() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let ready_path = temp_capture_path("media-preview-recovered");
    write_test_wav(&ready_path);

    let registration = RuntimeMediaAssetRegistration {
        asset_id: "asset:sha256:recoverable".to_string(),
        content_hash: "recoverable".to_string(),
        source_path: ready_path.display().to_string(),
        file_name: "recoverable.wav".to_string(),
        byte_size: fs::metadata(&ready_path).unwrap().len(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        duration_samples: 128,
        waveform_bin_count: 8,
    };

    runtime
        .reconcile_media_assets(vec![registration.clone()])
        .expect("ready media should reconcile");
    runtime
        .start_media_preview("asset:sha256:recoverable")
        .expect("preview should start for ready media");
    assert_eq!(
        runtime.get_media_service_snapshot().preview_state,
        crate::RuntimeMediaPreviewState::Previewing
    );

    fs::remove_file(&ready_path).expect("source media should be removable");
    runtime
        .reconcile_media_assets(vec![registration.clone()])
        .expect("missing media should reconcile as invalid");

    let invalidated = runtime.get_media_service_snapshot();
    assert_eq!(
        invalidated.preview_state,
        crate::RuntimeMediaPreviewState::Invalidated
    );
    assert_eq!(invalidated.previewing_asset_id, None);
    assert_eq!(invalidated.invalidated_asset_count, 1);

    write_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            ..registration
        }])
        .expect("restored media should reconcile");

    let recovered = runtime.get_media_service_snapshot();
    assert_eq!(
        recovered.indexing_state,
        crate::RuntimeMediaIndexingState::Ready
    );
    assert_eq!(
        recovered.preview_state,
        crate::RuntimeMediaPreviewState::Ready
    );
    assert_eq!(recovered.invalidated_asset_count, 0);
    assert_eq!(recovered.previewing_asset_id, None);
    assert_eq!(recovered.last_invalidated_asset_id, None);

    runtime
        .start_media_preview("asset:sha256:recoverable")
        .expect("preview should restart after recovery");
    let previewing_again = runtime.get_media_service_snapshot();
    assert_eq!(
        previewing_again.preview_state,
        crate::RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        previewing_again.previewing_asset_id.as_deref(),
        Some("asset:sha256:recoverable")
    );

    let _ = fs::remove_file(ready_path);
}
