use super::*;

pub(crate) fn write_test_wav(path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).expect("test wav should be created");
    for frame in 0..128 {
        let sample = ((frame as f32 / 128.0) * 2.0) - 1.0;
        writer
            .write_sample(sample)
            .expect("test wav sample should be written");
    }
    writer.finalize().expect("test wav should finalize");
}

pub(crate) fn write_transient_test_wav(path: &Path) {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: HoundSampleFormat::Float,
    };
    let mut writer = WavWriter::create(path, spec).expect("test wav should be created");
    for frame in 0..48_000 {
        let sample = if frame % 6_000 == 0 { 1.0 } else { 0.0 };
        writer
            .write_sample(sample)
            .expect("test wav sample should be written");
    }
    writer.finalize().expect("test wav should finalize");
}

pub(crate) fn write_test_aiff(path: &Path) {
    use std::io::Write;

    let frames = 128u32;
    let sample_rate_extended = [0x40, 0x0E, 0xBB, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let samples = (0..frames)
        .map(|frame| {
            let sample = ((frame as f32 / 128.0) * 2.0) - 1.0;
            (sample * i16::MAX as f32) as i16
        })
        .collect::<Vec<_>>();
    let data_size = samples.len() as u32 * 2;
    let ssnd_size = 8 + data_size;
    let form_size = 4 + (8 + 18) + (8 + ssnd_size);
    let mut file = fs::File::create(path).expect("test aiff should be created");
    file.write_all(b"FORM").expect("write FORM");
    file.write_all(&form_size.to_be_bytes())
        .expect("write FORM size");
    file.write_all(b"AIFF").expect("write AIFF signature");
    file.write_all(b"COMM").expect("write COMM");
    file.write_all(&18u32.to_be_bytes())
        .expect("write COMM size");
    file.write_all(&1u16.to_be_bytes())
        .expect("write channel count");
    file.write_all(&frames.to_be_bytes())
        .expect("write frame count");
    file.write_all(&16u16.to_be_bytes())
        .expect("write sample size");
    file.write_all(&sample_rate_extended)
        .expect("write sample rate");
    file.write_all(b"SSND").expect("write SSND");
    file.write_all(&ssnd_size.to_be_bytes())
        .expect("write SSND size");
    file.write_all(&0u32.to_be_bytes()).expect("write offset");
    file.write_all(&0u32.to_be_bytes())
        .expect("write block size");
    for sample in samples {
        file.write_all(&sample.to_be_bytes())
            .expect("write AIFF sample");
    }
}

pub(crate) fn prepare_offline_render_engine_runtime() -> (SignalRuntime, PathBuf) {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 32));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);

    let imported_path = temp_capture_path("offline-render-engine-proof");
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
            file_name: "offline-render-engine-proof.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:offline-engine".into(),
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
            graph_id: "graph:runtime:offline-render-engine".into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
                },
                GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 8,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "bus-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
                },
                GraphNodeProjection {
                    node_id: "console-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
                },
            ],
        })
        .unwrap();
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-engine".into(),
            contract_count: 4,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "track".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection::default(),
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection::default(),
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:lead".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "bus-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Bus),
                        track_lane_id: None,
                        bus_group_id: Some("mix:master".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "console-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: Some("console:main".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .unwrap();
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:offline-render-engine".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .unwrap();
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-a".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_recovery_cycle(
        "sandbox-a",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:offline-render-engine".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![PluginNodeRender {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
                output: AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
                latency_samples: 8,
                tail_samples: 0,
                bypassed: false,
            }],
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(32)),
        )
        .unwrap();

    (runtime, imported_path)
}
