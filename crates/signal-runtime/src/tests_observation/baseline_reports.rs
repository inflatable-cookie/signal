use super::*;

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
        observation.media_service_snapshot.waveform_ready_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        observation.media_service_snapshot.previewing_asset_id.as_deref(),
        Some("asset:sha256:observation")
    );
    assert_eq!(observation.media_library_snapshot.indexed_asset_count, 1);
    assert_eq!(observation.media_library_snapshot.ready_descriptor_count, 1);
    assert_eq!(
        observation.media_library_snapshot.loudness_ready_descriptor_count,
        1
    );
    assert_eq!(
        observation.media_library_snapshot.character_ready_descriptor_count,
        1
    );
    assert_eq!(
        observation.media_library_snapshot.descriptors[0].metadata_state,
        crate::RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert!(observation.media_library_snapshot.descriptors[0].loudness.is_some());
    assert!(observation.media_library_snapshot.descriptors[0].character.is_some());

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("media_asset_count=1"));
    assert!(multiline.contains("media_preview_state=Previewing"));
    assert!(multiline.contains("media_library_ready_descriptor_count=1"));

    let json = supervisor.render_json();
    assert!(json.contains("\"media_pipeline_snapshot\":{"));
    assert!(json.contains("\"media_service_snapshot\":{"));
    assert!(json.contains("\"media_library_snapshot\":{"));
    assert!(json.contains("\"preview_state\":\"Previewing\""));
    assert!(json.contains("\"waveform_ready_asset_count\":1"));
    assert!(json.contains("\"ready_descriptor_count\":1"));

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

#[test]
fn runtime_observation_and_supervisor_reports_surface_external_midi_endpoint_baseline() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation.external_midi_snapshot.discovery_state,
        crate::RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        observation.external_midi_snapshot.graph_state,
        crate::RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(observation.external_midi_snapshot.device_count, 0);
    assert_eq!(observation.external_midi_snapshot.endpoint_count, 0);

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("external_midi_discovery_state=Unavailable"));
    assert!(multiline.contains("external_midi_graph_state=Unavailable"));

    let json = supervisor.render_json();
    assert!(json.contains("\"external_midi_snapshot\":{"));
    assert!(json.contains("\"discovery_state\":\"Unavailable\""));
    assert!(json.contains("\"graph_state\":\"Unavailable\""));
    assert!(json.contains("\"provider_name\":\"runtime-unavailable\""));
}

#[test]
fn runtime_acceptance_receipt_scopes_integrated_runtime_lanes_and_targets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:acceptance-scope");

    let ready_path = temp_capture_path("acceptance-media-ready");
    write_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:acceptance".to_string(),
            content_hash: "acceptance".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "acceptance.wav".to_string(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .expect("ready media should reconcile");
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:acceptance".to_string(),
            track_id: "track:acceptance".to_string(),
            start_samples: 0,
            capture_path: temp_capture_path("acceptance-take").display().to_string(),
        })
        .expect("recording capture should start");

    let receipt = runtime.get_acceptance_receipt();
    assert_eq!(receipt.runtime_lane_count, 6);
    assert!(receipt.playback_ready);
    assert!(receipt.recording_ready);
    assert!(receipt.media_ready);
    assert!(!receipt.clip_processing_ready);
    assert!(!receipt.plugin_ready);
    assert!(receipt.recovery_ready);
    assert_eq!(receipt.minimum_trace_observation_count, 128);
    assert_eq!(receipt.minimum_soak_event_count, 64);
    assert_eq!(receipt.runtime_ready_lane_count, 4);

    let _ = fs::remove_file(ready_path);
}
