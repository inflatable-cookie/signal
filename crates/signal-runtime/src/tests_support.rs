#![allow(dead_code, unused_imports)]

// Tests for signal-runtime
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{RuntimeConfig, RuntimeMeteringStateModel, RuntimeProfile, SignalRuntime};
use crate::interfaces::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection,
    HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode, LingeringCleanupTrigger,
    ParameterBatch, ParameterEvent, PluginBackedNodeBinding, PluginBackedNodeBindingProjection,
    PluginFaultKind, PluginNodeRender, PluginNodeRenderBatch, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RestartRequest, RuntimeAuditionSinkAuthority, RuntimeAuditionSinkClass,
    RuntimeAutomationInterpolation, RuntimeAutomationLaneProjection,
    RuntimeAutomationPointProjection, RuntimeAutomationProjection, RuntimeAutomationResolution,
    RuntimeAutomationTargetProjection, RuntimeBlockDeadlinePressure, RuntimeClipFadeEnvelope,
    RuntimeClipFadeShape, RuntimeClipGainEnvelope, RuntimeClipGainShape,
    RuntimeClipProcessingReadiness, RuntimeClipProcessingRegistration, RuntimeClipProcessingStage,
    RuntimeClipRenderInputStage, RuntimeClipRenderRequest, RuntimeConfigRequest,
    RuntimeControllerExpressionMidi2Posture, RuntimeControllerExpressionMpePosture,
    RuntimeDeferredServiceBackpressureSource, RuntimeDeferredServiceCancellationCause,
    RuntimeDeferredServiceClass, RuntimeDeferredServiceDecision,
    RuntimeDeferredServicePriorityBand, RuntimeDeferredServiceReason, RuntimeError,
    RuntimeErrorKind, RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionPhase,
    RuntimeFaultCause, RuntimeFaultStatusSnapshot, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeLowLatencyDevicePolicyClass, RuntimeLowLatencyDevicePolicyOutcome,
    RuntimeMarkerAnalysisReadiness, RuntimeMediaAssetRegistration, RuntimeMediaAssetState,
    RuntimeMediaAuditionContinuityOutcome, RuntimeMediaAuditionOrchestrationAuthority,
    RuntimeMediaAuditionOrchestrationPosture, RuntimeMediaPreviewState, RuntimeMeterSourceRole,
    RuntimeMeterSourceSnapshot, RuntimeObservationApi, RuntimeObservationReport,
    RuntimeOfflineFreezeArtifactRequest, RuntimeOfflinePluginDelegatedExecutionMerge,
    RuntimeOfflinePluginDelegatedExecutionOutcome, RuntimeOfflinePluginDelegatedExecutionReceipt,
    RuntimeOfflinePluginDelegatedExecutionStageReceipt,
    RuntimeOfflinePluginDelegatedExecutionStatus,
    RuntimeOfflinePluginDelegatedFreezeArtifactOutput, RuntimeOfflinePluginDelegatedStemOutput,
    RuntimeOfflinePluginExecutionBoundary, RuntimeOfflinePluginExecutionOwner,
    RuntimeOfflinePluginExecutionStageBoundary, RuntimeOfflinePluginOverrideState,
    RuntimeOfflineRenderArtifactKind, RuntimeOfflineRenderCheckpointStage,
    RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderExecutionState,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderRequest, RuntimeOfflineRenderStemTarget,
    RuntimeOfflineRenderTargetKind, RuntimePluginBusCapableFxClass, RuntimePluginCompensationState,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginLifecycleState, RuntimePluginParityBand,
    RuntimePluginPlacementPolicy, RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher,
    RuntimePluginRecallHandoffSelection, RuntimePluginRecallHandoffStageId,
    RuntimePluginRecallPayload, RuntimePluginRecallState, RuntimePreviewBrowserQueueClass,
    RuntimePreviewBrowserQueueOutcome, RuntimePreviewBrowserQueuePosture,
    RuntimePreviewOutputRoutingPosture, RuntimePreviewTransformFallbackKind,
    RuntimePreviewTransformReadiness, RuntimePreviewTransformSchedulingAuthority,
    RuntimePreviewTransformSchedulingOutcome, RuntimePreviewTransformSchedulingPosture,
    RuntimePreviewTransformServiceClass, RuntimePreworkBacklogClass, RuntimePreworkCacheState,
    RuntimePreworkForecastMode, RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState, RuntimeRecoveryState,
    RuntimeSchedulerState, RuntimeSchedulerTopologyIssue, RuntimeSecondaryInputContractProjection,
    RuntimeSecondaryInputTargetKind, RuntimeStretchEngineClass, RuntimeStretchFallbackKind,
    RuntimeStretchReadiness, RuntimeSupervisorReport, RuntimeTempoAssistHintSource,
    RuntimeTempoAssistPosture, RuntimeTempoMapInterpolation, RuntimeTempoMapProjection,
    RuntimeTempoSource, RuntimeTransformArtifactReadiness, RuntimeTransformArtifactReuseState,
    RuntimeTransformCachePlacementAuthority, RuntimeTransformCachePlacementOutcome,
    RuntimeTransformCachePlacementPosture, RuntimeTransformPersistencePosture,
    RuntimeTransformRetentionAuthority, RuntimeTransformRetentionOutcome,
    RuntimeTransformRetentionPolicyClass, RuntimeWarpClipRegistration, RuntimeWarpMode,
    RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest, SandboxOperationFailureStage,
    ScheduleProjection, StopReason, TransportAttachIntent, TransportProjection,
    TransportSessionProvenance, WatchdogRestartRecord,
};
use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};
use signal_graph::{
    synthetic_stereo_block, ExecutableGraph, GraphExecutionLane, GraphNodeBufferContract,
    GraphNodeBusEndpoint, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeSpec,
    GraphNodeTopologyMetadata, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    CompletionState, EventPacketSummary, ParameterAutomationSummary, PluginFeature, PluginFormat,
    PluginIoLayout, PluginLifecycleContract, PluginProcessingContract, PluginStateContract,
};
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};

#[derive(Default)]
struct TestSink {
    events: Vec<RuntimeEvent>,
}

impl RuntimeEventSink for TestSink {
    fn push(&mut self, event: RuntimeEvent) {
        self.events.push(event);
    }
}

fn handshake_and_configure(runtime: &mut SignalRuntime) {
    handshake_and_configure_with_anticipative(runtime, true);
}

fn handshake_and_configure_with_anticipative(
    runtime: &mut SignalRuntime,
    anticipative_enabled: bool,
) {
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    let mut request = RuntimeConfigRequest::new(48_000, 256);
    request.anticipative_enabled = anticipative_enabled;
    runtime.configure(request).unwrap();
}

static TEMP_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_media_path(label: &str, extension: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic enough for temp files")
        .as_nanos();
    let sequence = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "signal-runtime-{label}-{nonce}-{sequence}.{extension}"
    ))
}

fn temp_capture_path(label: &str) -> PathBuf {
    temp_media_path(label, "wav")
}

fn temp_artifact_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic enough for temp dirs")
        .as_nanos();
    let sequence = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!("signal-runtime-{label}-{nonce}-{sequence}"))
}

fn apply_plugin_continuity_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    bindings: &[(&str, &str)],
) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeProjection {
                    node_id: (*node_id).into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                })
                .collect(),
        })
        .expect("plugin continuity graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: bindings.len(),
            nodes: bindings
                .iter()
                .map(|(node_id, _)| GraphNodeContractProjection {
                    node_id: (*node_id).into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:plugin-continuity".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("plugin continuity contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: bindings
                .iter()
                .map(|(node_id, sandbox_id)| PluginBackedNodeBinding {
                    node_id: (*node_id).into(),
                    sandbox_id: (*sandbox_id).into(),
                })
                .collect(),
        })
        .expect("plugin continuity bindings should apply");
}

fn record_ready_plugin_sandbox(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    processing_epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(processing_epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        format!("lease-{sandbox_id}"),
        format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(processing_epoch),
        None,
    );
}

fn write_test_wav(path: &Path) {
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

fn write_transient_test_wav(path: &Path) {
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

fn write_test_aiff(path: &Path) {
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

fn prepare_offline_render_engine_runtime() -> (SignalRuntime, PathBuf) {
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

fn prepare_sidechain_runtime() -> SignalRuntime {
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

fn prepare_spatial_runtime() -> SignalRuntime {
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

fn prepare_offline_render_engine_runtime_without_cached_plugin_render() -> (SignalRuntime, PathBuf)
{
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

fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    let mut buffer = AudioBuffer::new(
        SampleRate(sample_rate_hz),
        ChannelLayout::Stereo,
        FrameCount(frames),
    );
    buffer.samples_mut().fill(value);
    buffer
}

fn handshake_and_configure_with_disabled_forecast(
    runtime: &mut SignalRuntime,
    anticipative_enabled: bool,
) {
    handshake_and_configure_with_anticipative(runtime, anticipative_enabled);
    runtime
        .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
        .unwrap();
}

fn seed_pending_prework_targets(
    runtime: &mut SignalRuntime,
    admitted_from_block_sequence: u64,
    target_block_sequences: &[u64],
) {
    runtime.engine.pending_prework_targets.clear();
    let targets = target_block_sequences
        .iter()
        .map(|target_block_sequence| RuntimePreworkWindowTarget {
            target_block_sequence: *target_block_sequence,
            admitted_from_block_sequence,
            buffer: synthetic_stereo_block(
                runtime.config.sample_rate,
                FrameCount(runtime.config.graph.block_size),
                *target_block_sequence,
            ),
            parameter_epoch_override: None,
            transport_override: None,
        })
        .collect::<Vec<_>>();
    let graph_id = runtime
        .engine
        .graph
        .as_ref()
        .map(|graph| graph.graph_id().to_string());
    runtime.engine.reconcile_pending_prework_targets(
        &targets,
        graph_id.as_deref(),
        runtime.projection_epoch,
        runtime.latest_parameter_epoch,
        runtime.applied_transport,
        runtime.config.graph.block_size,
    );
}

fn apply_current_forecast_block_state(runtime: &mut SignalRuntime, block_sequence: u64) {
    let policy = runtime
        .prework_forecast_policy
        .clone()
        .expect("forecast policy configured");
    runtime
        .apply_forecast_transport_projection(
            runtime.forecast_transport_projection_for_block(block_sequence, &policy),
        )
        .expect("apply forecast transport projection");
    runtime
        .apply_parameter_batch(runtime.forecast_parameter_batch_for_block(block_sequence, &policy))
        .expect("apply forecast parameter batch");
}

fn apply_latency_runtime_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
}

fn install_scheduler_topology_runtime_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    track_lane_ids: &[&str],
    include_missing_track_lane_id: bool,
) {
    let mut nodes = vec![GraphNodeSpec {
        node_id: "lookahead".into(),
        execution_class: GraphNodeExecutionClass::LatencyBearing,
        latency_samples: 32,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:lookahead", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Utility),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
    }];

    for (index, lane_id) in track_lane_ids.iter().enumerate() {
        nodes.push(GraphNodeSpec {
            node_id: format!("track-{index}"),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: Some((*lane_id).into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
        });
    }

    if include_missing_track_lane_id {
        nodes.push(GraphNodeSpec {
            node_id: "track-missing".into(),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: None,
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
        });
    }

    nodes.push(GraphNodeSpec {
        node_id: "bus-main".into(),
        execution_class: GraphNodeExecutionClass::Stateful,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Bus),
            track_lane_id: None,
            bus_group_id: Some("mix:master".into()),
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
    });

    nodes.push(GraphNodeSpec {
        node_id: "console-main".into(),
        execution_class: GraphNodeExecutionClass::PureTransform,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("main:out", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::ConsoleNode),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: Some("console:main".into()),
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
    });

    runtime.engine.graph = Some(ExecutableGraph::new(graph_id, nodes));
    runtime
        .engine
        .refresh_planning(runtime.anticipative_enabled);
    runtime.refresh_scheduler_topology_summary();
}
