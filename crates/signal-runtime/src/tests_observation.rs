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

#[test]
fn runtime_reconciles_warp_clips_against_media_readiness_and_project_tempo() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("warp-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:warp-ready".to_string(),
            content_hash: "warp-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "warp-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:warp-ready".to_string(),
            media_asset_id: Some("asset:sha256:warp-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let ready = runtime.get_warp_pipeline_snapshot();
    assert_eq!(ready.clip_count, 1);
    assert_eq!(ready.ready_clip_count, 1);
    assert_eq!(ready.degraded_clip_count, 0);
    assert_eq!(
        ready.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(ready.clips[0].readiness, RuntimeWarpReadiness::Ready);
    assert_eq!(
        ready.clips[0].project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert!((ready.clips[0].realized_ratio - 1.5).abs() < 0.000_1);

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 300.0,
            loop_state: None,
        })
        .unwrap();
    let degraded = runtime.get_warp_pipeline_snapshot();
    assert_eq!(degraded.ready_clip_count, 0);
    assert_eq!(degraded.degraded_clip_count, 1);
    assert_eq!(
        degraded.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(degraded.clips[0].readiness, RuntimeWarpReadiness::Degraded);
    assert!(degraded.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("outside baseline support"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_reconciles_clip_processing_against_media_and_warp_readiness() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("clip-processing-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:clip-processing-ready".to_string(),
            content_hash: "clip-processing-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "clip-processing-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 2_048,
                shape: RuntimeClipFadeShape::SmoothStep,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 4_096,
                shape: RuntimeClipFadeShape::EqualPower,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 0.82,
                end_linear: 0.64,
                shape: RuntimeClipGainShape::Linear,
            },
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let ready = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(ready.clip_count, 1);
    assert_eq!(ready.ready_clip_count, 1);
    assert_eq!(ready.pending_media_clip_count, 0);
    assert_eq!(ready.pending_warp_clip_count, 0);
    assert_eq!(ready.invalid_clip_count, 0);
    assert_eq!(ready.faded_clip_count, 1);
    assert_eq!(ready.gain_shaped_clip_count, 1);
    assert_eq!(ready.warped_clip_count, 1);
    assert_eq!(ready.treatment_stage_count, 4);
    assert_eq!(
        ready.clips[0].readiness,
        RuntimeClipProcessingReadiness::Ready
    );
    assert_eq!(ready.clips[0].fade_in_end_samples, 2_048);
    assert_eq!(ready.clips[0].fade_out_start_samples, 43_904);
    assert_eq!(
        ready.clips[0].treatment_stages,
        vec![
            RuntimeClipProcessingStage::Warp,
            RuntimeClipProcessingStage::FadeIn,
            RuntimeClipProcessingStage::GainShape,
            RuntimeClipProcessingStage::FadeOut,
        ]
    );
    assert_eq!(
        ready.clips[0].fade_in.shape,
        RuntimeClipFadeShape::SmoothStep
    );
    assert_eq!(
        ready.clips[0].fade_out.shape,
        RuntimeClipFadeShape::EqualPower
    );
    assert_eq!(ready.clips[0].clip_gain.shape, RuntimeClipGainShape::Linear);
    assert!((ready.clips[0].clip_gain.start_linear - 0.82).abs() < f32::EPSILON);
    assert!((ready.clips[0].clip_gain.end_linear - 0.64).abs() < f32::EPSILON);
    assert_eq!(
        ready.clips[0].project_tempo_source,
        Some(RuntimeTempoSource::TransportProjection)
    );
    assert!((ready.clips[0].realized_warp_ratio.unwrap_or_default() - 1.5).abs() < 0.000_1);

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 300.0,
            loop_state: None,
        })
        .unwrap();

    let invalid = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(invalid.clip_count, 1);
    assert_eq!(invalid.ready_clip_count, 0);
    assert_eq!(invalid.invalid_clip_count, 1);
    assert_eq!(
        invalid.clips[0].readiness,
        RuntimeClipProcessingReadiness::Invalid
    );
    assert!(invalid.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("outside baseline support"));

    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:processing-ready".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 2_048,
                shape: RuntimeClipFadeShape::Linear,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 4_096,
                shape: RuntimeClipFadeShape::Linear,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 0.82,
                end_linear: 0.64,
                shape: RuntimeClipGainShape::Hold,
            },
        }])
        .unwrap();
    let invalid_gain_shape = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(invalid_gain_shape.invalid_clip_count, 1);
    assert_eq!(
        invalid_gain_shape.clips[0].readiness,
        RuntimeClipProcessingReadiness::Invalid
    );
    assert!(invalid_gain_shape.clips[0]
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("hold clip gain shape requires identical start and end gain"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_observation_clip_render_and_offline_render_preview_surface_stretch_engine_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("stretch-engine-ready");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:stretch-engine-ready".to_string(),
            content_hash: "stretch-engine-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "stretch-engine-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:stretch-engine-ready".to_string(),
            media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:stretch-engine-ready".to_string(),
            media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(observation.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation.stretch_engine_snapshot.sample_domain_clip_count,
        1
    );
    assert_eq!(observation.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].readiness,
        RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].fallback_kind,
        RuntimeStretchFallbackKind::None
    );
    assert!(observation
        .render_compact()
        .contains("stretch_clips=1/1/1/0/0/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"stretch_engine_snapshot\":{\"clip_count\":1"));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:stretch-engine-ready".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.5; 8],
            ),
        })
        .unwrap();
    assert_eq!(
        rendered.stretch_engine_snapshot.engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.readiness,
        RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.fallback_kind,
        RuntimeStretchFallbackKind::None
    );
    assert!(rendered.summary.contains("stretch=SampleDomain/Ready/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:stretch-engine-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &runtime.get_plugin_recall_handoff_snapshot(),
    )
    .expect("build stretch engine offline render preview");
    assert_eq!(preview.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.sample_domain_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].readiness,
        RuntimeStretchReadiness::Ready
    );
    assert!(preview.summary.contains("stretch=1/fallback=0"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_marker_analysis_snapshot_derives_from_stretch_and_media_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("marker-analysis-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:marker-analysis-ready".to_string(),
            content_hash: "marker-analysis-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "marker-analysis-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:marker-analysis-ready".to_string(),
            media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:marker-analysis-ready".to_string(),
            media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let marker_analysis = runtime.get_marker_analysis_snapshot();
    assert_eq!(marker_analysis.clip_count, 1);
    assert_eq!(marker_analysis.ready_clip_count, 1);
    assert_eq!(marker_analysis.pending_media_clip_count, 0);
    assert_eq!(marker_analysis.degraded_clip_count, 0);
    assert_eq!(marker_analysis.invalidated_clip_count, 0);
    assert_eq!(marker_analysis.unsupported_clip_count, 0);
    assert_eq!(marker_analysis.tempo_assist_ready_clip_count, 1);
    assert!(marker_analysis.warp_marker_count > 0);
    assert!(marker_analysis.transient_anchor_count > 0);
    assert_eq!(
        marker_analysis.clips[0].readiness,
        RuntimeMarkerAnalysisReadiness::Ready
    );
    assert_eq!(
        marker_analysis.clips[0].tempo_assist_posture,
        RuntimeTempoAssistPosture::Ready
    );
    assert_eq!(
        marker_analysis.clips[0].tempo_assist_hint_source,
        RuntimeTempoAssistHintSource::SourceTempo
    );
    assert_eq!(marker_analysis.clips[0].tempo_assist_hint_bpm, Some(120.0));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(observation.marker_analysis_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(observation
        .render_compact()
        .contains("marker_analysis_clips=1/1/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("marker_analysis_clip_count=1"));
    assert!(multiline.contains("marker_analysis_tempo_assist_ready_clip_count=1"));
    assert!(supervisor
        .render_json()
        .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_transform_artifact_snapshot_derives_from_stretch_and_marker_analysis_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("transform-artifact-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:transform-artifact-ready".to_string(),
            content_hash: "transform-artifact-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "transform-artifact-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:transform-artifact-ready".to_string(),
            media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:transform-artifact-ready".to_string(),
            media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();

    let transform_artifact = runtime.get_transform_artifact_snapshot();
    assert_eq!(transform_artifact.clip_count, 1);
    assert_eq!(transform_artifact.ready_clip_count, 1);
    assert_eq!(transform_artifact.pending_media_clip_count, 0);
    assert_eq!(transform_artifact.degraded_clip_count, 0);
    assert_eq!(transform_artifact.invalidated_clip_count, 0);
    assert_eq!(transform_artifact.unsupported_clip_count, 0);
    assert_eq!(transform_artifact.cached_media_ready_clip_count, 1);
    assert_eq!(transform_artifact.reusable_clip_count, 1);
    assert_eq!(transform_artifact.requires_render_clip_count, 0);
    assert_eq!(transform_artifact.guarded_reuse_clip_count, 0);
    assert_eq!(
        transform_artifact.transform_persistence.persistence_posture,
        RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .retention_policy_class,
        RuntimeTransformRetentionPolicyClass::AssetLifetimeRetentionPolicy
    );
    assert_eq!(
        transform_artifact.transform_persistence.retention_authority,
        RuntimeTransformRetentionAuthority::RuntimeDefault
    );
    assert_eq!(
        transform_artifact.transform_persistence.retention_outcome,
        RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_posture,
        RuntimeTransformCachePlacementPosture::RuntimeCacheRootPlacement
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_authority,
        RuntimeTransformCachePlacementAuthority::RuntimeDefault
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .cache_placement_outcome,
        RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .persistent_clip_count,
        1
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .guarded_persistence_clip_count,
        0
    );
    assert_eq!(
        transform_artifact
            .transform_persistence
            .invalidated_persistence_clip_count,
        0
    );
    assert!(!transform_artifact
        .transform_persistence
        .cache_root_path
        .is_empty());
    assert_eq!(
        transform_artifact.clips[0].readiness,
        RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        transform_artifact.clips[0].reuse_state,
        RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(transform_artifact.clips[0].cached_media_ready);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(observation.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation.transform_artifact_snapshot.reusable_clip_count,
        1
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .persistence_posture,
        RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert!(observation
        .render_compact()
        .contains("transform_artifacts=1/1/0/0/0"));
    assert!(observation
        .render_json()
        .contains("\"transform_artifact_snapshot\":{\"clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"transform_persistence\":{\"persistence_posture\":\"AssetScopedTransformPersistence\""
    ));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:transform-artifact-ready".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.5; 8],
            ),
        })
        .unwrap();
    assert_eq!(
        rendered.transform_artifact_snapshot.readiness,
        RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        rendered.transform_artifact_snapshot.reuse_state,
        RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(rendered.transform_artifact_snapshot.cached_media_ready);
    assert!(rendered
        .summary
        .contains("transform=Ready/Reusable/cached_media=true"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:transform-artifact-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &runtime.get_plugin_recall_handoff_snapshot(),
    )
    .expect("build transform artifact offline render preview");
    assert_eq!(preview.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.reusable_clip_count, 1);
    assert_eq!(
        preview
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert!(preview.summary.contains("transform_artifacts=1/reusable=1"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_preview_transform_snapshot_derives_from_stretch_and_artifact_baselines() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("preview-transform-ready");
    write_transient_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:preview-transform-ready".to_string(),
            content_hash: "preview-transform-ready".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "preview-transform-ready.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 32,
        }])
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:preview-transform-ready".to_string(),
            media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:preview-transform-ready".to_string(),
            media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .unwrap();
    runtime
        .start_media_preview("asset:sha256:preview-transform-ready")
        .expect("preview transform media preview should start");

    let preview_transform = runtime.get_preview_transform_snapshot();
    assert_eq!(preview_transform.clip_count, 1);
    assert_eq!(preview_transform.active_audition_clip_count, 1);
    assert_eq!(preview_transform.scrub_supported_clip_count, 1);
    assert_eq!(preview_transform.ready_clip_count, 1);
    assert_eq!(preview_transform.pending_clip_count, 0);
    assert_eq!(preview_transform.degraded_clip_count, 0);
    assert_eq!(preview_transform.invalidated_clip_count, 0);
    assert_eq!(preview_transform.unsupported_clip_count, 0);
    assert_eq!(preview_transform.stretch_aligned_clip_count, 0);
    assert_eq!(preview_transform.artifact_backed_clip_count, 1);
    assert_eq!(preview_transform.fallback_clip_count, 0);
    assert_eq!(
        preview_transform.preview_device_policy.routing_posture,
        RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        preview_transform.preview_device_policy.audition_sink_class,
        RuntimeAuditionSinkClass::GuardedPreviewSink
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .audition_sink_authority,
        RuntimeAuditionSinkAuthority::RuntimeDefault
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .low_latency_device_policy_class,
        RuntimeLowLatencyDevicePolicyClass::GuardedLowLatencyDevicePolicy
    );
    assert_eq!(
        preview_transform
            .preview_device_policy
            .low_latency_device_policy_outcome,
        RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_posture,
        RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_class,
        RuntimePreviewBrowserQueueClass::SingleAssetAuditionQueue
    );
    assert_eq!(
        preview_transform.preview_workflow.queue_outcome,
        RuntimePreviewBrowserQueueOutcome::PreserveActivePreviewRequest
    );
    assert_eq!(
        preview_transform.preview_workflow.audition_posture,
        RuntimeMediaAuditionOrchestrationPosture::DirectRuntimeAuditionOrchestration
    );
    assert_eq!(
        preview_transform.preview_workflow.audition_authority,
        RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .audition_continuity_outcome,
        RuntimeMediaAuditionContinuityOutcome::PreserveActiveAudition
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_posture,
        RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_authority,
        RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .queued_preview_request_count,
        1
    );
    assert_eq!(
        preview_transform.preview_workflow.previewable_asset_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .pending_transform_clip_count,
        0
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .ready_transform_clip_count,
        1
    );
    assert_eq!(
        preview_transform
            .preview_workflow
            .fallback_transform_clip_count,
        0
    );
    assert_eq!(
        preview_transform.clips[0].service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        preview_transform.clips[0].readiness,
        RuntimePreviewTransformReadiness::Ready
    );
    assert_eq!(
        preview_transform.clips[0].fallback_kind,
        RuntimePreviewTransformFallbackKind::None
    );
    assert!(preview_transform.clips[0].audition_active);
    assert!(preview_transform.clips[0].scrub_supported);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.preview_transform_snapshot.clip_count, 1);
    assert_eq!(observation.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .preview_transform_snapshot
            .active_audition_clip_count,
        1
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert!(observation
        .render_json()
        .contains("\"preview_transform_snapshot\":{\"clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"preview_device_policy\":{\"routing_posture\":\"GuardedPreviewOutputRouting\""
    ));
    assert!(observation
        .render_json()
        .contains("\"preview_workflow\":{\"queue_posture\":\"SingleActivePreviewQueue\""));

    let rendered = runtime
        .render_clip_processing_buffer(RuntimeClipRenderRequest {
            clip_id: "clip:preview-transform-ready".to_string(),
            timeline_start_samples: 0,
            input_stage: RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.5; 8],
            ),
        })
        .unwrap();
    assert_eq!(
        rendered.preview_transform_snapshot.service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        rendered.preview_transform_snapshot.readiness,
        RuntimePreviewTransformReadiness::Ready
    );
    assert!(rendered.preview_transform_snapshot.audition_active);
    assert!(rendered
        .summary
        .contains("preview=ArtifactBacked/Ready/None/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:preview-transform-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &runtime.get_plugin_recall_handoff_snapshot(),
    )
    .expect("build preview transform offline render preview");
    assert_eq!(preview.preview_transform_snapshot.clip_count, 1);
    assert_eq!(preview.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        preview
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .active_audition_clip_count,
        0
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_class,
        RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_outcome,
        RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .audition_continuity_outcome,
        RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert!(preview
        .summary
        .contains("preview_transform=1/artifact_backed=1/fallback=0"));

    let _ = fs::remove_file(imported_path);
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
fn runtime_tempo_map_projection_drives_warp_ratio_and_export_reports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("warp-tempo-map");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:warp-tempo-map".to_string(),
            content_hash: "warp-tempo-map".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "warp-tempo-map.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .apply_tempo_map_projection(RuntimeTempoMapProjection {
            segment_count: 2,
            segments: vec![
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:intro".to_string(),
                    start_samples: 0,
                    end_samples: Some(48_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                },
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:lift".to_string(),
                    start_samples: 48_000,
                    end_samples: Some(96_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: Some(180.0),
                    interpolation: RuntimeTempoMapInterpolation::Linear,
                },
            ],
        })
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:warp-tempo-map".to_string(),
            media_asset_id: Some("asset:sha256:warp-tempo-map".to_string()),
            mode: RuntimeWarpMode::Repitch,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 72_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();

    let tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(tempo_map.segment_count, 2);
    assert_eq!(tempo_map.active_segment_id.as_deref(), Some("tempo:lift"));
    assert_eq!(tempo_map.active_segment_index, Some(1));
    assert_eq!(tempo_map.tempo_source, RuntimeTempoSource::TempoMapSegment);
    assert!((tempo_map.resolved_tempo_bpm - 150.0).abs() < 0.000_1);

    let warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(warp.clip_count, 1);
    assert_eq!(warp.ready_clip_count, 1);
    assert_eq!(warp.degraded_clip_count, 0);
    assert_eq!(
        warp.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.resolved_project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.resolved_project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert_eq!(
        warp.clips[0].project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.clips[0].project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.clips[0].project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert!((warp.clips[0].realized_ratio - 1.25).abs() < 0.000_1);

    let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        report.tempo_map_snapshot.tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        report.warp_pipeline_snapshot.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert!(report.render_compact().contains("tempo_map_segments=2"));
    assert!(report
        .render_compact()
        .contains("tempo_map_source=TempoMapSegment"));
    assert!(report.render_compact().contains("warp_clips=1/1/0/0"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("tempo_map_source=TempoMapSegment"));
    assert!(multiline.contains("warp_resolved_project_tempo_source=TempoMapSegment"));
    let json = supervisor.render_json();
    assert!(json.contains("\"tempo_map_snapshot\":{\"segment_count\":2"));
    assert!(json.contains("\"resolved_project_tempo_source\":\"TempoMapSegment\""));

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 120_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();
    let fallback_tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(fallback_tempo_map.active_segment_id, None);
    assert_eq!(
        fallback_tempo_map.tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert!((fallback_tempo_map.resolved_tempo_bpm - 90.0).abs() < 0.000_1);
    let fallback_warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(
        fallback_warp.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(fallback_warp.resolved_project_tempo_segment_id, None);
    assert!((fallback_warp.clips[0].realized_ratio - 0.75).abs() < 0.000_1);

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

fn runtime_emits_events_to_subscribers() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let sink = Box::new(TestSink::default());
    runtime.subscribe(sink);

    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();
    runtime.start().unwrap();
    runtime.set_active_output_device("coreaudio:default");
    runtime.set_active_plugin_sandboxes(2);

    let readiness = runtime.get_readiness();
    assert_eq!(readiness, RuntimeReadiness::Ready);
    assert_eq!(
        runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
        2
    );
}

#[test]
fn runtime_records_plugin_fault_events() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::ProtocolViolation,
        "epoch mismatch",
        Some(3),
    );

    assert_eq!(
        runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
        0
    );
}

#[test]
fn runtime_tracks_plugin_lifecycle_recovery_and_quarantine_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    let recorder = RuntimeEventRecorder::default();
    runtime.subscribe(Box::new(recorder.clone()));
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin-a".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 24,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
            }],
        })
        .expect("apply lifecycle receipt graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin-a".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply lifecycle receipt contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-a".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply lifecycle receipt binding");

    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );
    runtime.set_active_plugin_sandboxes(1);

    let ready = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(ready.active_sandbox_count, 1);
    assert_eq!(ready.ready_sandbox_count, 1);
    assert_eq!(ready.sandboxes[0].state, RuntimePluginLifecycleState::Ready);
    assert_eq!(
        ready.sandboxes[0].active_lease_id.as_deref(),
        Some("lease-a")
    );

    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::Crash,
        "sandbox crashed during process block",
        Some(2),
    );
    runtime.set_active_plugin_sandboxes(0);

    let faulted = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(faulted.faulted_sandbox_count, 1);
    assert_eq!(
        faulted.sandboxes[0].state,
        RuntimePluginLifecycleState::Faulted
    );
    assert_eq!(
        faulted.sandboxes[0].last_fault_detail.as_deref(),
        Some("sandbox crashed during process block")
    );

    runtime.record_recovery_cycle(
        "sandbox-a",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(3),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(3),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(4),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-b",
        "region-b",
        PluginSandboxTransportStage::Attached,
        Some(4),
        None,
    );
    runtime.set_active_plugin_sandboxes(1);

    let recovered = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(recovered.ready_sandbox_count, 1);
    assert_eq!(
        recovered.sandboxes[0].state,
        RuntimePluginLifecycleState::Ready
    );
    assert_eq!(recovered.sandboxes[0].restart_count, 1);
    assert_eq!(recovered.sandboxes[0].recovery_count, 1);
    assert_eq!(
        recovered.sandboxes[0].active_lease_id.as_deref(),
        Some("lease-b")
    );

    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::Timeout,
        "sandbox missed heartbeat twice",
        Some(5),
    );

    let quarantined = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(quarantined.quarantined_sandbox_count, 1);
    assert_eq!(
        quarantined.sandboxes[0].state,
        RuntimePluginLifecycleState::Quarantined
    );
    assert_eq!(quarantined.sandboxes[0].fault_count, 2);

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();
    assert_eq!(profiling.plugin_chain_stage_count, 1);
    assert_eq!(profiling.plugin_chain_degraded_stage_count, 1);
    assert_eq!(soak.plugin_fault_count, 2);
    assert_eq!(soak.recovery_event_count, 1);
    assert_eq!(soak.plugin_quarantined_sandbox_count, 1);
    assert_eq!(soak.recall_stage_count, 1);
    assert_eq!(soak.recovered_recall_stage_count, 0);
    assert_eq!(soak.unavailable_recall_stage_count, 1);
    assert_eq!(
        soak.last_recovery_intent,
        Some(RecoveryRestartIntent::CrashRecovery)
    );
    assert!(soak
        .render_json()
        .contains("\"plugin_quarantined_sandbox_count\":1"));
}

#[test]
fn runtime_owns_watchdog_restart_escalation() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().unwrap();

    let first = runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    assert_eq!(first.watchdog_restart_count, 1);
    assert!(!first.safe_mode_enabled);

    let second = runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });
    assert_eq!(second.watchdog_restart_count, 2);
    assert!(second.safe_mode_enabled);
    assert_eq!(
        second.last_watchdog_trigger,
        Some(RuntimeWatchdogTrigger::DeadlineMisses)
    );
    assert_eq!(second.last_processing_epoch, Some(2));
    assert!(matches!(
        runtime.get_readiness(),
        RuntimeReadiness::Degraded { .. }
    ));
}

#[test]
fn runtime_fault_status_snapshot_classifies_watchdog_plugin_fault_and_xrun_pressure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_xrun_overload(Some(1));
    runtime.record_xrun_overload(Some(2));
    runtime.record_xrun_overload(Some(3));
    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        PluginFaultKind::Crash,
        "sandbox crashed during process block",
        Some(2),
    );
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 3,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 4,
    });

    let status = RuntimeFaultStatusSnapshot::capture(
        runtime.get_readiness(),
        &runtime.get_control_snapshot(),
        &runtime.get_diagnostics_snapshot(),
        &runtime.get_supervision_snapshot(),
        &runtime.get_engine_block_snapshot(),
        &runtime.get_transport_concurrency_snapshot(),
        &runtime.get_plugin_lifecycle_snapshot(),
        false,
        0,
    );

    assert_eq!(status.recovery_state, RuntimeRecoveryState::Recovering);
    assert_eq!(
        status.primary_fault_cause,
        Some(RuntimeFaultCause::WatchdogRestart)
    );
    assert_eq!(status.active_fault_count, 3);
    assert!(status.xrun_overload_active);
    assert!(status.plugin_fault_active);
    assert!(status.watchdog_active);
    assert!(status.safe_mode_enabled);
    assert_eq!(status.plugin_fault_count, 1);
    assert_eq!(status.watchdog_restart_count, 2);
    assert!(status.summary.contains("primary=Some(WatchdogRestart)"));
}

#[test]
fn runtime_fault_status_snapshot_clears_watchdog_active_after_safe_mode_recovery() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });
    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("safe mode should clear after watchdog recovery");

    let status = RuntimeFaultStatusSnapshot::capture(
        runtime.get_readiness(),
        &runtime.get_control_snapshot(),
        &runtime.get_diagnostics_snapshot(),
        &runtime.get_supervision_snapshot(),
        &runtime.get_engine_block_snapshot(),
        &runtime.get_transport_concurrency_snapshot(),
        &runtime.get_plugin_lifecycle_snapshot(),
        false,
        0,
    );

    assert_eq!(status.recovery_state, RuntimeRecoveryState::Steady);
    assert_eq!(status.primary_fault_cause, None);
    assert_eq!(status.active_fault_count, 0);
    assert!(!status.watchdog_active);
    assert!(!status.safe_mode_enabled);
    assert_eq!(status.watchdog_restart_count, 2);
}

#[test]
fn runtime_observation_report_surfaces_restartable_interruption_summary() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

    assert_eq!(
        observation.fault_status.primary_fault_cause,
        Some(RuntimeFaultCause::WatchdogRestart)
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(observation.interruption_summary.active);
    assert!(!observation.interruption_summary.rebindable);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"fault_status\":{"));
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"interruption_summary\":{"));
    assert!(observation_json.contains("\"class\":\"Restartable\""));
}

#[test]
fn runtime_fault_diagnostic_receipt_maps_xrun_pressure_into_runtime_owned_primary_family() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_xrun_overload(Some(1));
    runtime.record_xrun_overload(Some(2));
    runtime.record_xrun_overload(Some(3));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let receipt = &observation.fault_diagnostic_receipt;
    let xrun = receipt
        .contributions
        .iter()
        .find(|entry| entry.family == crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure)
        .expect("xrun contribution should be present");

    assert_eq!(
        receipt.primary_family,
        Some(crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure)
    );
    assert_eq!(
        receipt.primary_fault_cause,
        Some(crate::interfaces::RuntimeFaultCause::XrunOverload)
    );
    assert_eq!(
        receipt.interruption_class,
        crate::interfaces::RuntimeInterruptionClass::Recoverable
    );
    assert!(xrun.active);
    assert_eq!(xrun.event_count, 3);
    assert_eq!(
        xrun.authority,
        crate::interfaces::RuntimeFaultDiagnosticAuthority::RuntimeCanonical
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"primary_family\":\"XrunPressure\""));
}

#[test]
fn runtime_fault_diagnostic_receipt_maps_deferred_work_pressure_without_faulting_runtime() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:fault-diagnostic:deferred".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");
    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let receipt = &observation.fault_diagnostic_receipt;
    let deferred_entry = receipt
        .contributions
        .iter()
        .find(|entry| {
            entry.family == crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
        })
        .expect("deferred-work contribution should be present");

    assert_eq!(
        receipt.primary_family,
        Some(crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(receipt.primary_fault_cause, None);
    assert_eq!(
        receipt.interruption_class,
        crate::interfaces::RuntimeInterruptionClass::Recoverable
    );
    assert!(deferred_entry.active);
    assert!(deferred_entry.event_count >= 1);
    assert!(deferred_entry
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("decision=Some(Defer)"));
}
