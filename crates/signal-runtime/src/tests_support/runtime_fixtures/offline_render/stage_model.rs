use super::super::super::*;

pub(super) fn prepare_offline_render_engine_runtime_without_cached_plugin_render(
) -> (SignalRuntime, PathBuf) {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 32));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);

    let imported_path = temp_capture_path("offline-render-engine-stage-model");
    let content_hash = imported_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("offline render helper path should have a file stem")
        .to_string();
    let asset_id = format!("asset:sha256:{content_hash}");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: asset_id.clone(),
            content_hash: content_hash.clone(),
            source_path: imported_path.display().to_string(),
            file_name: "offline-render-engine-stage-model.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:offline-engine-stage-model".into(),
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
            graph_id: "graph:runtime:offline-render-stage-model".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
            }],
        })
        .unwrap();
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-stage-model".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin".into(),
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
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:offline-render-stage-model".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .unwrap();
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );

    (runtime, imported_path)
}
