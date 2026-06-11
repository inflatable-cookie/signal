use super::super::*;

#[test]
fn runtime_observation_and_supervisor_reports_surface_media_service_baseline() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let ready_path = temp_capture_path("media-observation-preview");
    write_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:observation".to_string(),
            content_hash: "observation".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "observation.wav".to_string(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 16,
        }])
        .expect("ready media should reconcile");
    runtime
        .start_media_preview("asset:sha256:observation")
        .expect("preview should start for ready media");

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.media_pipeline_snapshot.asset_count, 1);
    assert_eq!(observation.media_pipeline_snapshot.ready_asset_count, 1);
    assert_eq!(observation.media_service_snapshot.indexed_asset_count, 1);
    assert_eq!(
        observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:observation")
    );
    assert_eq!(observation.media_library_snapshot.indexed_asset_count, 1);
    assert_eq!(observation.media_library_snapshot.ready_descriptor_count, 1);
    assert_eq!(
        observation
            .media_library_snapshot
            .loudness_ready_descriptor_count,
        1
    );
    assert_eq!(
        observation
            .media_library_snapshot
            .character_ready_descriptor_count,
        1
    );
    assert_eq!(
        observation.media_library_snapshot.descriptors[0].metadata_state,
        crate::RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert!(observation.media_library_snapshot.descriptors[0]
        .loudness
        .is_some());
    assert!(observation.media_library_snapshot.descriptors[0]
        .character
        .is_some());

    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());


    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
