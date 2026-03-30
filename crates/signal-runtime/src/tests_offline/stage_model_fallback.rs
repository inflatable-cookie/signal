use super::*;

#[test]
fn runtime_offline_render_decodes_non_wav_cached_media_assets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 32));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);

    let imported_path = temp_media_path("offline-render-aiff", "aiff");
    let content_hash = imported_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("offline render AIFF helper path should have a file stem")
        .to_string();
    let asset_id = format!("asset:sha256:{content_hash}");
    write_test_aiff(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: asset_id.clone(),
            content_hash: content_hash.clone(),
            source_path: imported_path.display().to_string(),
            file_name: "offline-render-aiff.aiff".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:offline-render-aiff".into(),
            media_asset_id: Some(asset_id),
            warp_mode: RuntimeWarpMode::Off,
            start_samples: 0,
            duration_samples: 64,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .unwrap();
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:offline-render-aiff".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "track".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
            }],
        })
        .unwrap();
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-aiff".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "track".into(),
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
        .unwrap();

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:aiff".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should decode AIFF media");

    assert_eq!(result.main_mix.as_ref().unwrap().sample_rate().0, 48_000);
    assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 64);
    assert!(result.main_mix_peak_level.unwrap() > 0.45);
    assert!(result.main_mix_rms_level.unwrap() > 0.15);

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
fn runtime_offline_render_falls_back_to_plugin_stage_model_without_cached_render() {
    let (runtime, imported_path) =
        prepare_offline_render_engine_runtime_without_cached_plugin_render();

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:stage-model".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should fall back to the plugin stage model");

    assert_eq!(result.rendered_frame_count, 64);
    assert!(result.main_mix_peak_level.unwrap() <= 0.5 + 1.0e-6);
    assert!(result.main_mix_peak_level.unwrap() >= 0.49);
    let first_samples = &result.main_mix.as_ref().unwrap().samples()[..4];
    assert!((first_samples[0] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[1] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[2] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[3] + 0.5).abs() < 1.0e-6);

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
fn runtime_offline_render_ignores_stale_plugin_override_and_uses_stage_model() {
    let (mut runtime, imported_path) =
        prepare_offline_render_engine_runtime_without_cached_plugin_render();
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:offline-render-stage-model".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![PluginNodeRender {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
                output: AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
                latency_samples: 0,
                tail_samples: 0,
                bypassed: false,
            }],
        })
        .expect("seed a zero-valued live plugin render override");
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
        )
        .expect("consume the seeded live plugin render override");
    runtime
        .process_engine_block(
            1,
            2,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
        )
        .expect("advance the live engine beyond the last plugin render override");

    let result = runtime
        .render_offline(RuntimeOfflineRenderRequest {
            request_id: "render:stale-plugin-override".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("offline render should fall back after the live override becomes stale");

    assert_eq!(result.rendered_frame_count, 64);
    assert!((result.main_mix_peak_level.unwrap() - 0.5).abs() < 1.0e-6);
    assert_eq!(result.plugin_execution_boundary.stage_count, 1);
    assert_eq!(
        result.plugin_execution_boundary.fresh_override_stage_count,
        0
    );
    assert_eq!(
        result.plugin_execution_boundary.stale_override_stage_count,
        1
    );
    assert_eq!(
        result.plugin_execution_boundary.stages[0].override_state,
        RuntimeOfflinePluginOverrideState::StaleLatestBlock
    );
    let first_samples = &result.main_mix.as_ref().unwrap().samples()[..6];
    assert!((first_samples[0] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[1] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[2] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[3] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[4] + 0.5).abs() < 1.0e-6);
    assert!((first_samples[5] + 0.5).abs() < 1.0e-6);

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
