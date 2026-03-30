use super::*;

pub(crate) fn prepare_sidechain_runtime() -> SignalRuntime {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 128));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:sidechain-routing".into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track-input".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "sidechain-feed".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-compressor".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.84 }],
                },
                GraphNodeProjection {
                    node_id: "output-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::StereoBalance { balance: 0.0 }],
                },
            ],
        })
        .expect("apply sidechain graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:sidechain-routing".into(),
            contract_count: 4,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "track-input".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
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
                    node_id: "sidechain-feed".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:sidechain:kick".into(),
                            channels: ChannelLayout::Mono,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Utility),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-compressor".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:tracks".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        secondary_input: Some(RuntimeSecondaryInputContractProjection {
                            source_kind: crate::RuntimeSecondaryInputSourceKind::NodeOutput,
                            source_id: "sidechain-feed".into(),
                            source_bus_id: Some("bus:sidechain:kick".into()),
                            target_bus_id: "plugin:compressor:sidechain".into(),
                            attachment_policy:
                                crate::RuntimeSecondaryInputAttachmentPolicy::Required,
                            fallback_outcome:
                                crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation,
                        }),
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
                    node_id: "output-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:tracks".into(),
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
        .expect("apply sidechain graph contract");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:sidechain-routing".into(),
            bindings: vec![crate::PluginBackedNodeBinding {
                node_id: "plugin-compressor".into(),
                sandbox_id: "sandbox:compressor".into(),
            }],
        })
        .expect("bind sidechain plugin node");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:compressor",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime
}

pub(crate) fn prepare_spatial_runtime() -> SignalRuntime {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 128));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:spatial-baseline".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "spatial-stereo".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                },
                GraphNodeProjection {
                    node_id: "spatial-surround".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 20,
                    stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                },
            ],
        })
        .expect("apply spatial graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:spatial-baseline".into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "spatial-stereo".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:stereo".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:stereo".into()),
                        bus_group_id: Some("bus:spatial:stereo".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "spatial-surround".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:surround-in".into(),
                            channels: ChannelLayout::Count(signal_primitives::ChannelCount(6)),
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:surround".into(),
                            channels: ChannelLayout::Count(signal_primitives::ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:surround".into()),
                        bus_group_id: Some("bus:spatial:surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("apply spatial graph contract");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:spatial-baseline".into(),
            bindings: vec![
                crate::PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: "sandbox:spatial-stereo".into(),
                },
                crate::PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: "sandbox:spatial-surround".into(),
                },
            ],
        })
        .expect("bind spatial plugin nodes");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:spatial-stereo",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:spatial-surround",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime
}

pub(crate) fn prepare_offline_render_engine_runtime_without_cached_plugin_render(
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
