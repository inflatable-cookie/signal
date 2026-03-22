use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::GraphNodeTopologyRole;
use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphStageSpec,
};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind,
};
use signal_plugin::{
    EventPacketSummary, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, HandshakeRequest,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxInstanceStateRecord,
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RestartRequest, RuntimeAuxiliaryPathKind, RuntimeBlockDeadlinePressure, RuntimeBusIntent,
    RuntimeBusRole, RuntimeCanonicalChannelLayout, RuntimeConfig, RuntimeConfigRequest,
    RuntimeDeferredServiceBackpressureSource, RuntimeDeferredServiceCancellationCause,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeDeploymentClass, RuntimeDeviceFaultBoundaryState,
    RuntimeDeviceRestartState, RuntimeDeviceSupervisionState, RuntimeDynamicBusNegotiationPosture,
    RuntimeError, RuntimeErrorKind, RuntimeEventRecorder, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeExternalMidiDiscoveryState, RuntimeExternalMidiGraphState,
    RuntimeFoldDownPolicy, RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState,
    RuntimeHostAudioTransferPolicy, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockSource,
    RuntimeHostClockTransitionState, RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState,
    RuntimeHostEndpointTopology, RuntimeHostHardwareSummary, RuntimeHostIoSummary,
    RuntimeHostLatencySummary, RuntimeHostLifecycleOwnership, RuntimeHostObservationReport,
    RuntimeHostRestartPolicy, RuntimeHostSupervisorReport, RuntimeImmersiveExportAuthority,
    RuntimeImmersiveExportClass, RuntimeImmersiveExportOutcome,
    RuntimeImmersiveObjectRenderingPosture, RuntimeImmersiveRoomOutcome, RuntimeInterruptionClass,
    RuntimeJackClientRole, RuntimeJackGraphCoordinationState, RuntimeJackGuardedCoordinationState,
    RuntimeJackTransportPosture, RuntimeLifecycleApi, RuntimeLv2ExtensionCapabilitySummary,
    RuntimeLv2ExtensionNegotiationState, RuntimeLv2PatchExchangePosture,
    RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass, RuntimeObservationApi,
    RuntimeObservationReport, RuntimeOfflineRenderContractPreview,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginNegotiationFallbackOutcome,
    RuntimePluginParityBand, RuntimePluginPinGroupIdentity, RuntimePluginPinMatrixPosture,
    RuntimePluginPlacementPolicy, RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher,
    RuntimePluginPresetDescriptor, RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass,
    RuntimeProjectionApi, RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeRendererCapabilityAuthority,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRoomPolicyAuthority,
    RuntimeRoomPolicyClass, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSpatialAdapterClass, RuntimeSpatialBedClass,
    RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, RuntimeSpatialRenderScope,
    RuntimeSpatialTargetEnvironment, RuntimeSupervisorReport, RuntimeWatchdogTrigger,
    SafeModeRequest, SignalRuntime, StopReason, TransportDispatchState,
    TransportHeartbeatFreshness, TransportSessionBoundaryMode, TransportSessionState,
    TransportSessionSummary, WatchdogRestartRecord,
};

fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:public-boundary".into(),
        plugin_id: "com.signal.public-boundary".into(),
        vendor: "Signal".into(),
        name: "Signal Public Boundary".into(),
        format: PluginFormat::Clap,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::AudioEffect, PluginFeature::Utility],
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
        ),
        audio_bus_count: 2,
        parameter_count: 8,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: true,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: true,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        },
        lv2_extension_capabilities: None,
        summary: "public boundary discovered plugin".into(),
    }
}

fn sample_backend_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        plugin_id: "com.signal.public-instrument".into(),
        vendor: "Signal".into(),
        name: "Signal Public Instrument".into(),
        format: PluginFormat::Vst3,
        version: Some("2.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 12,
        state_contract: PluginStateContract {
            supports_snapshot: false,
            supports_reset: true,
            supports_bypass: false,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: false,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: None,
        summary: "public boundary backend breadth plugin".into(),
    }
}

fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:public-multiout".into(),
        plugin_id: "com.signal.public-multiout".into(),
        vendor: "Signal".into(),
        name: "Signal Public Multi Output".into(),
        format: PluginFormat::Vst3,
        version: Some("2.1.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 24,
        state_contract: PluginStateContract {
            supports_snapshot: false,
            supports_reset: true,
            supports_bypass: false,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: None,
        summary: "public boundary complex multi-output instrument".into(),
    }
}

fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:public-bus-fx".into(),
        plugin_id: "com.signal.public-bus-fx".into(),
        vendor: "Signal".into(),
        name: "Signal Public Bus FX".into(),
        format: PluginFormat::Vst3,
        version: Some("2.1.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 4,
                audio_outputs: 4,
                midi_inputs: 0,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::AudioEffect, PluginFeature::Utility],
            PluginIoLayout {
                audio_inputs: 4,
                audio_outputs: 4,
                midi_inputs: 0,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 2,
        parameter_count: 18,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: true,
            accepts_midi: false,
            accepts_note_events: false,
            supports_note_expression: false,
            produces_midi: false,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        },
        lv2_extension_capabilities: None,
        summary: "public boundary bus-capable fx".into(),
    }
}

fn sample_public_clock_topology_host_io(
    clock_domain: RuntimeHostClockDomain,
    fallback_state: RuntimeHostClockFallbackState,
    transition_state: RuntimeHostClockTransitionState,
    drift_state: RuntimeHostClockDriftState,
    discontinuity_state: RuntimeHostClockDiscontinuityState,
    duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    endpoint_topology: RuntimeHostEndpointTopology,
    partial_availability: bool,
) -> RuntimeHostIoSummary {
    let linux_backend_identity =
        signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::CoreAudio,
        );
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio".into(),
            linux_backend_identity,
            linux_backend_portability:
                signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_portability(
                    HardwareBackendIdentity::CoreAudio,
                    false,
                    BackendHealth::Healthy,
                    0,
                    0,
                    0,
                ),
            device_id: "device:public-clock-topology".into(),
            device_name: "Public Clock Topology".into(),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated: false,
            backend_health: BackendHealth::Healthy,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
        },
        audio_pump: RuntimeHostAudioPumpSummary {
            stream_state: RuntimeHostAudioStreamState::Running,
            transfer_policy: RuntimeHostAudioTransferPolicy {
                max_callback_frames: 256,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            },
            callback_count: 12,
            total_callback_frames: 3_072,
            total_runtime_output_frames: 3_072,
            copied_output_samples: 6_144,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.35),
            last_runtime_graph_id: Some("graph:public-clock-topology".into()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source: RuntimeHostClockSource::Internal,
            ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
            restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
            processing_sample_rate_hz: 44_100,
            hardware_sample_rate_hz: 48_000,
            clock_domain,
            fallback_state,
            transition_state,
            drift_state,
            discontinuity_state,
            duplex_mismatch_state,
            endpoint_topology,
            linux_clocking_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_clocking_parity(
                    linux_backend_identity,
                    BackendHealth::Healthy,
                    RuntimeHostAudioStreamState::Running,
                    clock_domain,
                    fallback_state,
                    transition_state,
                    drift_state,
                    discontinuity_state,
                ),
            linux_duplex_parity: signal_runtime::RuntimeHostIoSummary::classify_linux_duplex_parity(
                linux_backend_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                clock_domain,
                fallback_state,
                transition_state,
                duplex_mismatch_state,
                endpoint_topology,
                partial_availability,
            ),
            linux_endpoint_topology_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                    linux_backend_identity,
                    BackendHealth::Healthy,
                    transition_state,
                    discontinuity_state,
                    duplex_mismatch_state,
                    endpoint_topology,
                    partial_availability,
                ),
            partial_availability,
            crossing_required: matches!(
                clock_domain,
                RuntimeHostClockDomain::CrossClock | RuntimeHostClockDomain::Aggregate
            ),
            callback_interval_ms: 5.333,
        },
        latency: RuntimeHostLatencySummary {
            input_latency_samples: Some(128),
            output_latency_samples: 256,
            round_trip_latency_samples: Some(384),
            graph_latency_samples: 24,
            estimated_output_latency_samples: 280,
            estimated_round_trip_latency_samples: Some(408),
            output_latency_ms: 5.333,
            graph_latency_ms: 0.5,
            estimated_output_latency_ms: 5.833,
            estimated_round_trip_latency_ms: Some(8.5),
        },
        runtime_graph_id_matches_pump: true,
    }
}

fn sample_public_linux_backend_host_io(
    backend_identity: HardwareBackendIdentity,
    backend_name: &str,
    device_id: &str,
    device_name: &str,
    simulated: bool,
    backend_health: BackendHealth,
    device_loss_count: u64,
    restart_attempt_count: u64,
    restart_failure_count: u64,
) -> RuntimeHostIoSummary {
    let linux_backend_identity =
        signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_identity(
            backend_identity,
        );
    let clock_source = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockSource::Internal
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack) => {
            RuntimeHostClockSource::ExternalWordClock
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockSource::Virtual
        }
        _ => RuntimeHostClockSource::Internal,
    };
    let clock_domain = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDomain::SameClock
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDomain::Aggregate
        }
        _ => RuntimeHostClockDomain::SameClock,
    };
    let fallback_state = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockFallbackState::Direct
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockFallbackState::RuntimeResampled
        }
        _ => RuntimeHostClockFallbackState::Direct,
    };
    let transition_state = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockTransitionState::Stable
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockTransitionState::EnteredAggregateClock
        }
        _ => RuntimeHostClockTransitionState::Stable,
    };
    let drift_state = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDriftState::Stable
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDriftState::AggregateManaged
        }
        _ => RuntimeHostClockDriftState::Stable,
    };
    let discontinuity_state = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostClockDiscontinuityState::Continuous
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostClockDiscontinuityState::Reconfigured
        }
        _ => RuntimeHostClockDiscontinuityState::Continuous,
    };
    let duplex_mismatch_state = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostDuplexMismatchState::Aligned
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostDuplexMismatchState::CrossClockDiverged
        }
        _ => RuntimeHostDuplexMismatchState::NotApplicable,
    };
    let endpoint_topology = match backend_identity {
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
            RuntimeHostEndpointTopology::Duplex
        }
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
        | HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
            RuntimeHostEndpointTopology::Aggregate
        }
        _ => RuntimeHostEndpointTopology::Duplex,
    };
    let partial_availability = false;
    let stream_state = RuntimeHostAudioStreamState::Running;
    RuntimeHostIoSummary {
        hardware: RuntimeHostHardwareSummary {
            backend_identity,
            backend_name: backend_name.into(),
            linux_backend_identity,
            linux_backend_portability:
                signal_runtime::RuntimeHostHardwareSummary::classify_linux_backend_portability(
                    backend_identity,
                    simulated,
                    backend_health,
                    device_loss_count,
                    restart_attempt_count,
                    restart_failure_count,
                ),
            device_id: device_id.into(),
            device_name: device_name.into(),
            sample_rate: 48_000,
            buffer_size: 256,
            input_channels: 2,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            simulated,
            backend_health,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count,
            restart_attempt_count,
            restart_failure_count,
        },
        audio_pump: RuntimeHostAudioPumpSummary {
            stream_state,
            transfer_policy: RuntimeHostAudioTransferPolicy {
                max_callback_frames: 256,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            },
            callback_count: 12,
            total_callback_frames: 3_072,
            total_runtime_output_frames: 3_072,
            copied_output_samples: 6_144,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.25),
            last_runtime_graph_id: Some("graph:public-linux-backend".into()),
        },
        clocking: RuntimeHostClockingSummary {
            clock_source,
            ownership: RuntimeHostLifecycleOwnership::BackendManagedCallback,
            restart_policy: RuntimeHostRestartPolicy::BackendMayRestart,
            processing_sample_rate_hz: 48_000,
            hardware_sample_rate_hz: 48_000,
            clock_domain,
            fallback_state,
            transition_state,
            drift_state,
            discontinuity_state,
            duplex_mismatch_state,
            endpoint_topology,
            linux_clocking_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_clocking_parity(
                    linux_backend_identity,
                    backend_health,
                    stream_state,
                    clock_domain,
                    fallback_state,
                    transition_state,
                    drift_state,
                    discontinuity_state,
                ),
            linux_duplex_parity: signal_runtime::RuntimeHostIoSummary::classify_linux_duplex_parity(
                linux_backend_identity,
                backend_health,
                stream_state,
                clock_domain,
                fallback_state,
                transition_state,
                duplex_mismatch_state,
                endpoint_topology,
                partial_availability,
            ),
            linux_endpoint_topology_parity:
                signal_runtime::RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                    linux_backend_identity,
                    backend_health,
                    transition_state,
                    discontinuity_state,
                    duplex_mismatch_state,
                    endpoint_topology,
                    partial_availability,
                ),
            partial_availability,
            crossing_required: matches!(
                clock_domain,
                RuntimeHostClockDomain::CrossClock | RuntimeHostClockDomain::Aggregate
            ),
            callback_interval_ms: 5.333,
        },
        latency: RuntimeHostLatencySummary {
            input_latency_samples: Some(128),
            output_latency_samples: 256,
            round_trip_latency_samples: Some(384),
            graph_latency_samples: 24,
            estimated_output_latency_samples: 280,
            estimated_round_trip_latency_samples: Some(408),
            output_latency_ms: 5.333,
            graph_latency_ms: 0.5,
            estimated_output_latency_ms: 5.833,
            estimated_round_trip_latency_ms: Some(8.5),
        },
        runtime_graph_id_matches_pump: true,
    }
}

fn sample_public_transport_session_summary(
    current_state: TransportSessionState,
    currently_attached: bool,
    heartbeat_freshness: TransportHeartbeatFreshness,
    dispatch_state: TransportDispatchState,
    attach_events: usize,
    detach_requested_events: usize,
    detached_events: usize,
) -> TransportSessionSummary {
    TransportSessionSummary {
        boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
        current_state,
        currently_attached,
        heartbeat_freshness,
        dispatch_state,
        current_attached_session_count: usize::from(currently_attached),
        max_concurrent_attached_sessions: usize::from(currently_attached),
        attach_events,
        detach_requested_events,
        detached_events,
        detach_fault_events: 0,
        heartbeat_requested_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Requested
                | TransportHeartbeatFreshness::Fresh
                | TransportHeartbeatFreshness::Missed
        )),
        heartbeat_responded_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        )),
        heartbeat_missed_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        )),
        dispatch_requested_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Requested
                | TransportDispatchState::Completed
                | TransportDispatchState::TimedOut
        )),
        dispatch_completed_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Completed
        )),
        dispatch_timed_out_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::TimedOut
        )),
        first_processing_epoch: None,
        last_processing_epoch: None,
        first_block_sequence: None,
        last_block_sequence: None,
        active_sandbox_id: None,
        active_lease_id: None,
        active_region_id: None,
        active_block_sequence: None,
        active_sessions: Vec::new(),
        last_sandbox_id: None,
        last_lease_id: None,
        last_region_id: None,
    }
}

fn public_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for test files")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-runtime-public-media-{label}-{}-{unique}.wav",
        std::process::id()
    ))
}

fn write_public_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 128u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample =
            (((index as f32 / (frame_count - 1) as f32) * 2.0) - 1.0) * i16::MAX as f32 * 0.5;
        bytes.extend_from_slice(&(sample as i16).to_le_bytes());
    }
    fs::write(path, bytes).expect("public media fixture should be written");
}

fn write_public_transient_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 48_000u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample = if index % 6_000 == 0 { i16::MAX } else { 0 };
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).expect("public transient media fixture should be written");
}

fn sample_au_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:au:public-instrument".into(),
        plugin_id: "com.signal.public-au-instrument".into(),
        vendor: "Signal".into(),
        name: "Signal Public AU Instrument".into(),
        format: PluginFormat::Au,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 10,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: None,
        summary: "public boundary au breadth plugin".into(),
    }
}

fn sample_lv2_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:lv2:public-linux-synth".into(),
        plugin_id: "com.signal.public-lv2-linux-synth".into(),
        vendor: "Signal".into(),
        name: "Signal Public LV2 Linux Synth".into(),
        format: PluginFormat::Lv2,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 10,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: false,
            produces_midi: false,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: Some(
            RuntimeLv2ExtensionCapabilitySummary::from_lv2_feature_uris(
                &[
                    "http://lv2plug.in/ns/ext/urid#map".into(),
                    "http://lv2plug.in/ns/ext/worker#schedule".into(),
                ],
                &["http://lv2plug.in/ns/ext/patch#Message".into()],
            ),
        ),
        summary: "public lv2 boundary plugin".into(),
    }
}

fn apply_public_capture_graph(runtime: &mut SignalRuntime, graph_id: &str) {
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
        .expect("public capture graph projection should succeed");
}

fn apply_public_render_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "offline-inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                },
                GraphNodeProjection {
                    node_id: "offline-latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .expect("public render graph projection should succeed");
}

fn apply_public_plugin_continuity_graph(
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
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                })
                .collect(),
        })
        .expect("public plugin continuity graph projection should succeed");
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
                        track_lane_id: Some("track:public:plugin-continuity".into()),
                        bus_group_id: Some("mix:public".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                })
                .collect(),
        })
        .expect("public plugin continuity contracts should succeed");
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
        .expect("public plugin continuity bindings should succeed");
}

fn apply_public_multichannel_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "surround-track".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "analysis-send".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                },
            ],
        })
        .expect("public multichannel graph projection should succeed");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "surround-track".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:surround".into()),
                        bus_group_id: Some("mix:public:surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "analysis-send".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Count(ChannelCount(4)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: Some("track:public:surround".into()),
                        bus_group_id: Some("mix:public:surround".into()),
                        console_group_id: None,
                        send_return_id: Some("send:return:public:analysis".into()),
                    },
                },
            ],
        })
        .expect("public multichannel graph contract should succeed");
}

fn apply_public_sidechain_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "program-track".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.92 }],
                },
                GraphNodeProjection {
                    node_id: "kick-sidechain".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "compressor".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.78 }],
                },
            ],
        })
        .expect("public sidechain graph projection should succeed");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 3,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "program-track".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:program".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:sidechain".into()),
                        bus_group_id: Some("mix:public:sidechain".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "kick-sidechain".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
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
                    node_id: "compressor".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:program".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        secondary_input: Some(RuntimeSecondaryInputContractProjection {
                            source_kind:
                                signal_runtime::RuntimeSecondaryInputSourceKind::NodeOutput,
                            source_id: "kick-sidechain".into(),
                            source_bus_id: Some("bus:sidechain:kick".into()),
                            target_bus_id: "plugin:compressor:sidechain".into(),
                            attachment_policy: RuntimeSecondaryInputAttachmentPolicy::Required,
                            fallback_outcome:
                                RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation,
                        }),
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:sidechain".into()),
                        bus_group_id: Some("mix:public:sidechain".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public sidechain graph contract should succeed");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "compressor".into(),
                sandbox_id: "sandbox:public:sidechain".into(),
            }],
        })
        .expect("public sidechain plugin binding should succeed");
}

fn apply_public_multi_bus_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 5,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track-input".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "bus-dry".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "send-fx".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.4 }],
                },
                GraphNodeProjection {
                    node_id: "return-fx".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                },
                GraphNodeProjection {
                    node_id: "output-main".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.1 }],
                },
            ],
        })
        .expect("public multi-bus graph projection should succeed");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 5,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "track-input".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:multi-bus".into()),
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "bus-dry".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
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
                    node_id: "send-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:track:lead".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx:plate".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: Some("fx:plate".into()),
                    },
                },
                GraphNodeContractProjection {
                    node_id: "return-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:fx:plate".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Return),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: Some("fx:plate".into()),
                    },
                },
                GraphNodeContractProjection {
                    node_id: "output-main".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:mix:master".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: Some("console:public:main".into()),
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public multi-bus graph contract should succeed");
}

fn apply_public_complex_io_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "plugin-multiout".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-bus-fx".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                },
            ],
        })
        .expect("public complex io graph should apply");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "plugin-multiout".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:complex-io".into()),
                        bus_group_id: Some("mix:public:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "plugin-bus-fx".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:complex-io".into()),
                        bus_group_id: Some("mix:public:complex-io".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public complex io contracts should apply");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:public:multiout".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:public:bus-fx".into(),
                },
            ],
        })
        .expect("public complex io bindings should apply");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:public:multiout".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-multiout".into()),
    });
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:public:bus-fx".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-bus-fx".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:multiout",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:public:multiout",
        "lease-public-multiout",
        "region-public-multiout",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public complex io multiout attached".into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:bus-fx",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:bus-fx",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(2),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:public:bus-fx",
        "lease-public-bus-fx",
        "region-public-bus-fx",
        PluginSandboxTransportStage::Attached,
        Some(2),
        Some("public complex io bus fx attached".into()),
    );
}

fn apply_public_spatial_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
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
        .expect("public spatial graph projection should succeed");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "spatial-stereo".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:stereo".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:spatial-stereo".into()),
                        bus_group_id: Some("bus:public:spatial-stereo".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "spatial-surround".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "main:surround-in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: signal_runtime::GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:surround".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:public:spatial-surround".into()),
                        bus_group_id: Some("bus:public:spatial-surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("public spatial contracts should succeed");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: "sandbox:public:spatial-stereo".into(),
                },
                PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: "sandbox:public:spatial-surround".into(),
                },
            ],
        })
        .expect("public spatial bindings should succeed");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:spatial-stereo",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:spatial-surround",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
}

fn record_public_plugin_sandbox_ready(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        &format!("lease-{sandbox_id}"),
        &format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(epoch),
        None,
    );
}

fn sample_public_preset_descriptor() -> RuntimePluginPresetDescriptor {
    RuntimePluginPresetDescriptor {
        preset_id: Some("preset:factory:init".into()),
        label: Some("Init".into()),
        origin: RuntimePluginPresetOrigin::Factory,
        summary: "public runtime preset descriptor".into(),
    }
}

fn sample_public_ara_context(
    portability_class: RuntimePluginRecallPortabilityClass,
    document_id: &str,
    source_id: &str,
    region_id: &str,
    timeline_start_samples: i64,
    duration_samples: u32,
) -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: document_id.into(),
            display_label: Some("Session".into()),
            summary: "public runtime ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: source_id.into(),
            display_label: Some("Lead Vocal".into()),
            summary: "public runtime ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: region_id.into(),
            display_label: Some("Verse".into()),
            timeline_start_samples: Some(timeline_start_samples),
            duration_samples: Some(duration_samples),
            summary: "public runtime ara region".into(),
        }),
        summary: "public runtime ara context".into(),
    }
}

#[test]
fn public_runtime_contract_boundary_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-boundary-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-boundary-sandbox",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();

    assert_eq!(profiling.sample_rate_hz, 48_000);
    assert_eq!(profiling.block_size, 512);
    assert_eq!(soak.event_stream_count, 0);
    assert_eq!(
        observation.fault_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Steady
    );
    assert!(!observation.interruption_summary.active);
    assert_eq!(observation.fault_diagnostic_receipt.primary_family, None);
    assert_eq!(observation.fault_diagnostic_receipt.contributions.len(), 4);
    assert_eq!(
        observation.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Idle)
    );
    assert_eq!(observation.plugin_discovery_snapshot.scan_count, 1);
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        2
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:clap:public-boundary"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].features,
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    );
    assert!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .multi_format_catalog
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .requires_main_thread_for_state_count,
        1
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.format_coverage[1].format,
        PluginFormat::Vst3
    );
    assert_eq!(
        observation.plugin_lifecycle_snapshot.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"sample_rate\":48000"));
    assert!(observation_json.contains("\"block_size\":512"));
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"fault_status\":{"));
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"interruption_summary\":{"));
    assert!(observation_json.contains("\"recording_capture_snapshot\":{"));
    assert!(observation_json.contains("\"class\":\"Steady\""));
    assert!(observation_json.contains("\"execution_topology_summary\":{"));
    assert!(observation_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:clap:public-boundary\""));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
    assert!(observation_json.contains("\"discovered_format_count\":2"));
    assert!(observation_json.contains("\"multi_format_catalog\":true"));
    assert!(observation_json.contains("\"supports_snapshot\":true"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"sample_rate\":48000"));
    assert!(supervisor_json.contains("\"block_size\":512"));
    assert!(supervisor_json.contains("\"event_stream\":0"));
    assert!(supervisor_json.contains("\"fault_status\":{"));
    assert!(supervisor_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(supervisor_json.contains("\"interruption_summary\":{"));
    assert!(supervisor_json.contains("\"recording_capture_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"discovered_type_count\":2"));
    assert!(supervisor_json.contains("\"format_coverage\":["));

    let profiling_json = profiling.render_json();
    assert!(profiling_json.contains("\"sample_rate_hz\":48000"));
    assert!(profiling_json.contains("\"block_size\":512"));
    assert!(profiling_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(profiling_json.contains("\"summary\":"));

    let soak_json = soak.render_json();
    assert!(soak_json.contains("\"event_stream_count\":0"));
    assert!(soak_json.contains("\"summary\":"));

    assert!(profiling
        .render_multiline()
        .contains("sample_rate_hz=48000"));
    assert!(soak.render_multiline().contains("event_stream_count=0"));
}

#[test]
fn public_runtime_plugin_discovery_coverage_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let coverage = &observation.plugin_discovery_snapshot.capability_coverage;
    let format_coverage = &observation.plugin_discovery_snapshot.format_coverage;

    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .discovered_format_count,
        2
    );
    assert!(coverage.multi_format_catalog);
    assert_eq!(coverage.audio_effect_count, 1);
    assert_eq!(coverage.instrument_count, 1);
    assert_eq!(coverage.requires_main_thread_for_state_count, 1);
    assert_eq!(coverage.max_parameter_count, 12);
    assert_eq!(format_coverage.len(), 2);
    assert_eq!(format_coverage[0].format, PluginFormat::Clap);
    assert_eq!(format_coverage[0].supports_activate_count, 1);
    assert_eq!(format_coverage[1].format, PluginFormat::Vst3);
    assert_eq!(format_coverage[1].instrument_count, 1);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"discovered_format_count\":2"));
    assert!(observation_json.contains("\"format_coverage\":["));
    assert!(observation_json.contains("\"multi_format_catalog\":true"));
    assert!(observation_json.contains("\"requires_main_thread_for_state_count\":1"));
}

#[test]
fn public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-plugin-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public plugin continuity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public plugin continuity configure should succeed");
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://public-shared".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:public".into()),
            }],
        })
        .expect("public plugin continuity policy should apply");
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:public:plugin-continuity",
        &[
            ("plugin-a", "sandbox-shared"),
            ("plugin-b", "sandbox-shared"),
            ("plugin-c", "sandbox-isolated"),
        ],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-shared",
        PluginFormat::Clap,
        "plugin://public-shared",
        1,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-isolated",
        PluginFormat::Clap,
        "plugin://public-isolated",
        1,
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        signal_runtime::PluginFaultKind::Crash,
        "shared public crash",
        Some(2),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        signal_runtime::PluginFaultKind::Timeout,
        "shared public timeout",
        Some(3),
    );

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let lifecycle = &supervisor.observation.plugin_lifecycle_snapshot;
    let shared = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared boundary should be visible on public runtime boundary");
    assert_eq!(
        shared.placement_outcome,
        RuntimePluginIsolationOutcome::SharedSandbox
    );
    assert_eq!(
        shared.placement_rule_id.as_deref(),
        Some("share-verified-clap")
    );
    assert_eq!(shared.sandbox_group_key, "shared:public");
    assert_eq!(shared.shared_boundary_member_count, 2);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);

    let isolated = lifecycle
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
        .expect("isolated boundary should remain visible on public runtime boundary");
    assert_eq!(
        isolated.placement_outcome,
        RuntimePluginIsolationOutcome::IsolatedSandbox
    );
    assert_eq!(isolated.continuity_class, RuntimeInterruptionClass::Steady);

    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
    assert!(rendered.contains("\"placement_rule_id\":\"share-verified-clap\""));
    assert!(rendered.contains("\"sandbox_group_key\":\"shared:public\""));
    assert!(rendered.contains("\"shared_boundary_member_count\":2"));
    assert!(rendered.contains("\"continuity_class\":\"Terminal\""));
}

#[test]
fn public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_backend_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-vst3-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-vst3-sandbox".into(),
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        instance_id: "instance:public:vst3".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-vst3-sandbox",
        "lease-public-vst3",
        "region-public-vst3",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public vst3 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Vst3])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:vst3:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Vst3
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-vst3-sandbox")
        .expect("public vst3 sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:vst3:public-instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Vst3\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:vst3:public-instrument\""));
}

#[test]
fn public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
        formats: vec![PluginFormat::Au],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_au_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-au-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-au-sandbox".into(),
        plugin_type_id: "plugin:au:public-instrument".into(),
        instance_id: "instance:public:au".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-au-sandbox",
        "lease-public-au",
        "region-public-au",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public au transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Au])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:au:public-instrument"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Au
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-au-sandbox")
        .expect("public au sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:au:public-instrument")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Au\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:au:public-instrument\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:au:public-instrument\""));
}

#[test]
fn public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime.record_plugin_format_platform_coverage(
        vec![RuntimePluginFormatPlatformCoverageRecord {
        format: PluginFormat::Lv2,
        supported_platforms: vec![RuntimePluginHostPlatform::Linux],
        unsupported_platforms: vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ],
        linux_parity_band: RuntimePluginParityBand::Portable,
        linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
        linux_strict_sandbox_default: true,
        summary:
            "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                .into(),
    }],
    );
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
        formats: vec![PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_lv2_breadth_record()]);
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:public-linux-synth".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-lv2-sandbox",
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-lv2-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-lv2-sandbox".into(),
        plugin_type_id: "plugin:lv2:public-linux-synth".into(),
        instance_id: "instance:public:lv2".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-lv2-sandbox",
        "lease-public-lv2",
        "region-public-lv2",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public lv2 transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_type_count,
        1
    );
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.formats.clone()),
        Some(vec![PluginFormat::Lv2])
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].plugin_type_id,
        "plugin:lv2:public-linux-synth"
    );
    assert_eq!(
        observation.plugin_discovery_snapshot.discovered_types[0].format,
        PluginFormat::Lv2
    );
    let parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public lv2 parity should be visible");
    assert_eq!(
        parity.supported_platforms,
        vec![RuntimePluginHostPlatform::Linux]
    );
    assert_eq!(
        parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let sandbox = observation
        .plugin_lifecycle_snapshot
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "public-lv2-sandbox")
        .expect("public lv2 sandbox should be visible");
    assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
    assert_eq!(
        sandbox.plugin_type_id.as_deref(),
        Some("plugin:lv2:public-linux-synth")
    );
    assert_eq!(
        sandbox.lifecycle_stage,
        Some(PluginSandboxLifecycleStage::InstancePrepared)
    );
    assert_eq!(
        sandbox.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(sandbox.readiness_state.as_deref(), Some("Ready"));
    assert!(sandbox.active_transport);
    assert_eq!(observation.lv2_extension_snapshot.plugin_type_count, 1);
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        1
    );
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .urid_negotiated_type_count,
        1
    );
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        1
    );
    let lv2_extension = observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:public-linux-synth")
        .expect("public lv2 extension record should be visible");
    assert_eq!(
        lv2_extension.worker_posture,
        RuntimeLv2WorkerPosture::WorkerRequiredAvailable
    );
    assert_eq!(
        lv2_extension.urid_negotiation_posture,
        RuntimeLv2UridNegotiationPosture::Negotiated
    );
    assert_eq!(
        lv2_extension.patch_exchange_posture,
        RuntimeLv2PatchExchangePosture::Supported
    );
    assert_eq!(
        lv2_extension.extension_negotiation_state,
        RuntimeLv2ExtensionNegotiationState::Negotiated
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"formats\":[\"Lv2\"]"));
    assert!(observation_json.contains("\"plugin_type_id\":\"plugin:lv2:public-linux-synth\""));
    assert!(observation_json.contains("\"transport_stage\":\"Attached\""));
    assert!(observation_json.contains("\"supported_platforms\":[\"Linux\"]"));
    assert!(observation_json.contains("\"lv2_extension_snapshot\":{"));
    assert!(observation_json.contains("\"worker_posture\":\"WorkerRequiredAvailable\""));
    assert!(observation_json.contains("\"patch_exchange_posture\":\"Supported\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_type_id\":\"plugin:lv2:public-linux-synth\""));
    assert!(supervisor_json.contains("\"lv2_extension_snapshot\":{"));
}

#[test]
fn public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime.record_plugin_format_platform_coverage(vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Au,
            supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Unsupported,
            linux_preferred_sandbox_outcome: None,
            linux_strict_sandbox_default: false,
            summary: "platforms=MacOs linux=Unsupported unsupported=Linux/Windows".into(),
        },
    ]);
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "~/.vst3".into(),
            "~/Library/Audio/Plug-Ins/Components".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
            sample_au_breadth_record(),
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-parity-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-parity-vst3-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-parity-vst3-sandbox".into(),
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        instance_id: "instance:public:parity:vst3".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-parity-vst3-sandbox",
        "lease-public-parity-vst3",
        "region-public-parity-vst3",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public parity vst3 transport attached".into()),
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-parity-au-sandbox".into(),
        plugin_format: PluginFormat::Au,
        plugin_type_id: Some("plugin:au:public-instrument".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-parity-au-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: "public-parity-au-sandbox".into(),
        plugin_type_id: "plugin:au:public-instrument".into(),
        instance_id: "instance:public:parity:au".into(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: Some(1),
        processing_sample_rate_hz: Some(48_000),
        processing_max_block_frames: Some(512),
        audio_inputs: Some(0),
        audio_outputs: Some(2),
        midi_inputs: Some(1),
        midi_outputs: Some(0),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_transport(
        "public-parity-au-sandbox",
        "lease-public-parity-au",
        "region-public-parity-au",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public parity au transport attached".into()),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(
        observation.plugin_discovery_snapshot.parity_coverage.len(),
        3
    );
    let clap_parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("clap parity should be exported on the public runtime boundary");
    assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(clap_parity.discovered_type_count, 1);
    assert_eq!(
        clap_parity.supported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let au_parity = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("au parity should be exported on the public runtime boundary");
    assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
    assert_eq!(au_parity.discovered_type_count, 1);
    assert_eq!(au_parity.sandbox_count, 1);
    assert_eq!(au_parity.ready_sandbox_count, 1);
    assert_eq!(au_parity.active_transport_count, 1);
    assert_eq!(
        au_parity.supported_platforms,
        vec![RuntimePluginHostPlatform::MacOs]
    );
    assert_eq!(
        au_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    let vst3_parity = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("vst3 lifecycle parity should be exported on the public runtime boundary");
    assert_eq!(vst3_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(vst3_parity.sandbox_count, 1);
    assert_eq!(vst3_parity.ready_sandbox_count, 1);
    assert_eq!(vst3_parity.active_transport_count, 1);

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"parity_coverage\":["));
    assert!(observation_json.contains("\"parity_band\":\"Portable\""));
    assert!(observation_json.contains("\"parity_band\":\"Guarded\""));
    assert!(observation_json.contains("\"supported_platforms\":[\"MacOs\",\"Linux\",\"Windows\"]"));
    assert!(observation_json.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"parity_coverage\":["));
}

#[test]
fn public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-plugin-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux parity configure should succeed");
    runtime.record_plugin_format_platform_coverage(vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                    .into(),
        },
    ]);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![
                RuntimePluginPlacementRule {
                    rule_id: "public-linux-share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("linux:clap".into()),
                },
                RuntimePluginPlacementRule {
                    rule_id: "public-linux-inline-vst3".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Vst3),
                    outcome: RuntimePluginIsolationOutcome::InProcess,
                    sandbox_group_key: None,
                },
            ],
        })
        .expect("public linux parity placement policy should apply");

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into(), "~/.lv2".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
            sample_lv2_breadth_record(),
        ],
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-clap-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:public-boundary".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "public-linux-clap-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "public-linux-clap-sandbox",
        "lease-public-linux-clap",
        "region-public-linux-clap",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:public-instrument".into()),
    });
    runtime.record_recovery_cycle(
        "public-linux-vst3-sandbox",
        signal_runtime::RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "public-linux-vst3-sandbox",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "public-linux-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:public-linux-synth".into()),
    });
    runtime.record_plugin_sandbox_fault(
        "public-linux-lv2-sandbox",
        signal_runtime::PluginFaultKind::Crash,
        "public linux lv2 sandbox fault",
        Some(3),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let clap_discovery = observation
        .plugin_discovery_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("public linux clap parity should be visible");
    assert_eq!(
        clap_discovery.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(clap_discovery.linux_supported);
    assert_eq!(
        clap_discovery.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap_discovery.linux_strict_sandbox_default);

    let vst3_lifecycle = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("public linux vst3 parity should be visible");
    assert_eq!(
        vst3_lifecycle.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(vst3_lifecycle.linux_supported);
    assert_eq!(vst3_lifecycle.in_process_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.restarting_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.rebindable_sandbox_count, 1);
    assert_eq!(vst3_lifecycle.prepare_capable_type_count, 1);

    let lv2_lifecycle = observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("public linux lv2 parity should be visible");
    assert_eq!(
        lv2_lifecycle.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert!(lv2_lifecycle.linux_supported);
    assert_eq!(lv2_lifecycle.faulted_sandbox_count, 1);
    assert_eq!(
        lv2_lifecycle.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"linux_parity_band\":\"Portable\""));
    assert!(observation_json.contains("\"linux_supported\":true"));
    assert!(observation_json.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(observation_json.contains("\"restarting_sandbox_count\":1"));
    assert!(observation_json.contains("\"faulted_sandbox_count\":1"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_discovery_snapshot\":{"));
    assert!(supervisor_json.contains("\"plugin_lifecycle_snapshot\":{"));
    assert!(supervisor_json.contains("\"linux_strict_sandbox_default\":true"));
}

#[test]
fn public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-audio-backend".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux audio backend handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux audio backend configure should succeed");

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );

    let alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:graph-main",
        "JACK Graph Main",
        true,
        BackendHealth::Recovering,
        1,
        1,
        0,
    );
    let pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        false,
        BackendHealth::Degraded,
        0,
        1,
        1,
    );

    let alsa_observation = baseline.clone().with_host_external_io(&alsa);
    let jack_observation = baseline.clone().with_host_external_io(&jack);
    let pipewire_observation = baseline.with_host_external_io(&pipewire);

    assert_eq!(
        alsa_observation.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Portable
    );
    assert_eq!(
        jack_observation.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_backend_identity\":\"PipeWire\""));
    assert!(observation_json.contains("\"linux_backend_portability\":\"Guarded\""));
    assert!(observation_json.contains("\"backend_name\":\"pipewire\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor.observation.clone().with_host_external_io(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"external_io_snapshot\":{"));
    assert!(supervisor_json.contains("\"linux_backend_identity\":\"Alsa\""));
    assert!(supervisor_json.contains("\"linux_backend_portability\":\"Portable\""));
}

#[test]
fn public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-live-ownership".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux live ownership handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux live ownership configure should succeed");

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.linux_backend_session_snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        baseline.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::Unavailable
    );

    let mut alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    alsa.clocking.ownership = RuntimeHostLifecycleOwnership::HostDrivenCallback;
    alsa.clocking.restart_policy = RuntimeHostRestartPolicy::HostMustRestart;
    alsa.clocking.clock_domain = RuntimeHostClockDomain::SameClock;
    alsa.clocking.fallback_state = RuntimeHostClockFallbackState::Direct;
    alsa.clocking.transition_state = RuntimeHostClockTransitionState::Stable;
    alsa.clocking.drift_state = RuntimeHostClockDriftState::Stable;
    alsa.clocking.discontinuity_state = RuntimeHostClockDiscontinuityState::Continuous;
    alsa.clocking.duplex_mismatch_state = RuntimeHostDuplexMismatchState::Aligned;
    alsa.clocking.endpoint_topology = RuntimeHostEndpointTopology::Duplex;

    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:graph-main",
        "JACK Graph Main",
        true,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let mut pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        true,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );
    pipewire.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let alsa_observation = baseline.clone().with_linux_backend_session_snapshot(&alsa);
    let jack_observation = baseline.clone().with_linux_backend_session_snapshot(&jack);
    let pipewire_observation = baseline.with_linux_backend_session_snapshot(&pipewire);

    assert_eq!(
        alsa_observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa_observation.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
    );
    assert_eq!(
        alsa_observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
    );
    assert_eq!(
        jack_observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack_observation.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        jack_observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Recovering
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::Lost
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::FallbackContinuation
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_backend_session_snapshot\":{"));
    assert!(observation_json.contains("\"backend_identity\":\"PipeWire\""));
    assert!(observation_json.contains("\"lifecycle_state\":\"Recovering\""));
    assert!(observation_json.contains("\"device_claim_posture\":\"Lost\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_linux_backend_session_snapshot(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"linux_backend_session_snapshot\":{"));
    assert!(supervisor_json.contains("\"backend_identity\":\"Alsa\""));
    assert!(supervisor_json.contains("\"ownership\":\"HostBrokeredCallback\""));
}

#[test]
fn public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-pipewire-alsa-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public pipewire/alsa parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public pipewire/alsa parity configure should succeed");

    let mut alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    alsa.clocking.ownership = RuntimeHostLifecycleOwnership::HostDrivenCallback;
    alsa.clocking.restart_policy = RuntimeHostRestartPolicy::HostMustRestart;
    alsa.clocking.clock_domain = RuntimeHostClockDomain::SameClock;
    alsa.clocking.fallback_state = RuntimeHostClockFallbackState::Direct;
    alsa.clocking.transition_state = RuntimeHostClockTransitionState::Stable;
    alsa.clocking.drift_state = RuntimeHostClockDriftState::Stable;
    alsa.clocking.discontinuity_state = RuntimeHostClockDiscontinuityState::Continuous;
    alsa.clocking.duplex_mismatch_state = RuntimeHostDuplexMismatchState::Aligned;
    alsa.clocking.endpoint_topology = RuntimeHostEndpointTopology::Duplex;

    let pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        true,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );

    let mut recovering_pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:recovering-graph",
        "PipeWire Recovering Graph",
        true,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );
    recovering_pipewire.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let alsa_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&alsa)
        .with_pipewire_alsa_parity_snapshot(&alsa);
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        signal_runtime::RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
    );
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
    );
    assert_eq!(
        alsa_observation.pipewire_alsa_parity_snapshot.guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::Direct
    );

    let pipewire_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&pipewire)
        .with_pipewire_alsa_parity_snapshot(&pipewire);
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::ClockGuarded
    );

    let recovering_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&recovering_pipewire)
        .with_pipewire_alsa_parity_snapshot(&recovering_pipewire);
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        signal_runtime::RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::Lost
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::Restarting
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
    );

    let observation_json = recovering_observation.render_json();
    assert!(observation_json.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(observation_json.contains("\"session_role_parity\":\"FallbackContinuation\""));
    assert!(observation_json.contains("\"device_claim_parity\":\"Lost\""));
    assert!(observation_json.contains("\"stream_policy_parity\":\"Restarting\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_linux_backend_session_snapshot(&alsa)
        .with_pipewire_alsa_parity_snapshot(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(supervisor_json.contains("\"stream_policy_parity\":\"DirectHostCallback\""));
}

#[test]
fn public_runtime_jack_coordination_boundary_reports_runtime_owned_transport_graph_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-jack-coordination".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public jack coordination handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public jack coordination configure should succeed");

    let alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:main-graph",
        "JACK Main Graph",
        true,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let mut recovering_jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:recovering-graph",
        "JACK Recovering Graph",
        true,
        BackendHealth::Recovering,
        1,
        1,
        0,
    );
    recovering_jack.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let not_jack_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&alsa)
        .with_linux_backend_session_snapshot(&alsa)
        .with_jack_coordination_snapshot(&alsa);
    assert_eq!(
        not_jack_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        not_jack_observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(
        not_jack_observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::NotJack
    );
    assert_eq!(
        not_jack_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::NotJack
    );

    let mut following_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&jack)
        .with_linux_backend_session_snapshot(&jack);
    following_observation.transport_session_summary = sample_public_transport_session_summary(
        TransportSessionState::AttachActive,
        true,
        TransportHeartbeatFreshness::Fresh,
        TransportDispatchState::Completed,
        1,
        0,
        0,
    );
    following_observation = following_observation.with_jack_coordination_snapshot(&jack);
    assert_eq!(
        following_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::FollowingExternal
    );
    assert_eq!(
        following_observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        following_observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::FallbackContinuation
    );
    assert_eq!(
        following_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::TransportGuarded
    );

    let mut recovering_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&recovering_jack)
        .with_linux_backend_session_snapshot(&recovering_jack);
    recovering_observation.transport_session_summary = sample_public_transport_session_summary(
        TransportSessionState::DetachFaulted,
        true,
        TransportHeartbeatFreshness::Missed,
        TransportDispatchState::TimedOut,
        2,
        1,
        1,
    );
    recovering_observation =
        recovering_observation.with_jack_coordination_snapshot(&recovering_jack);
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::Guarded
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .graph_state,
        RuntimeJackGraphCoordinationState::Recovering
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .client_role,
        RuntimeJackClientRole::FallbackContinuation
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::Recovering
    );

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = following_observation;
    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"FollowingExternal\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    assert!(rendered.contains("\"client_role\":\"FallbackContinuation\""));
    assert!(rendered.contains("\"guarded_state\":\"TransportGuarded\""));
}

#[test]
fn public_runtime_linux_backend_clock_topology_boundary_reports_runtime_owned_linux_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );

    let alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:graph-main",
        "JACK Graph Main",
        true,
        BackendHealth::Recovering,
        1,
        1,
        0,
    );
    let pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        false,
        BackendHealth::Degraded,
        0,
        1,
        1,
    );

    let alsa_observation = baseline.clone().with_host_external_io(&alsa);
    let jack_observation = baseline.clone().with_host_external_io(&jack);
    let pipewire_observation = baseline.with_host_external_io(&pipewire);

    assert_eq!(
        alsa_observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Portable
    );
    assert_eq!(
        alsa_observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Aligned
    );
    assert_eq!(
        alsa_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
    );

    assert_eq!(
        jack_observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        jack_observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        jack_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_clocking_parity\":\"Guarded\""));
    assert!(observation_json.contains("\"linux_duplex_parity\":\"Guarded\""));
    assert!(observation_json.contains("\"linux_endpoint_topology_parity\":\"Guarded\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor.observation.clone().with_host_external_io(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"linux_clocking_parity\":\"Portable\""));
    assert!(supervisor_json.contains("\"linux_duplex_parity\":\"Aligned\""));
    assert!(supervisor_json.contains("\"linux_endpoint_topology_parity\":\"Portable\""));
}

#[test]
fn public_runtime_external_midi_boundary_reports_runtime_owned_endpoint_graph_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Unavailable
    );
    assert_eq!(
        unavailable.external_midi_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.external_midi_snapshot.device_count, 0);
    assert_eq!(unavailable.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        unavailable
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        unavailable
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Unavailable
    );
    assert!(unavailable.external_midi_snapshot.devices.is_empty());
    assert!(unavailable.external_midi_snapshot.endpoints.is_empty());

    let empty = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty("public-runtime");
    let empty_observation = unavailable
        .clone()
        .with_external_midi_snapshot(empty.clone());
    assert_eq!(
        empty_observation.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Empty
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.provider_name,
        "public-runtime"
    );
    assert_eq!(empty_observation.external_midi_snapshot.device_count, 0);
    assert_eq!(empty_observation.external_midi_snapshot.endpoint_count, 0);
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .ownership_posture,
        signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::Unavailable
    );
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .attach_continuity,
        signal_runtime::RuntimeExternalMidiAttachContinuity::Unavailable
    );
    assert_eq!(
        empty_observation
            .external_midi_snapshot
            .live_ownership
            .backend_parity,
        signal_runtime::RuntimeExternalMidiBackendParity::Unavailable
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.active_route_count,
        0
    );
    assert_eq!(
        empty_observation.external_midi_snapshot.guarded_route_count,
        0
    );

    let observation_json = empty_observation.render_json();
    assert!(observation_json.contains("\"external_midi_snapshot\":{"));
    assert!(observation_json.contains("\"live_ownership\":{"));
    assert!(observation_json.contains("\"discovery_state\":\"Idle\""));
    assert!(observation_json.contains("\"graph_state\":\"Empty\""));
    assert!(observation_json.contains("\"ownership_posture\":\"Unavailable\""));
    assert!(observation_json.contains("\"backend_parity\":\"Unavailable\""));
    assert!(observation_json.contains("\"provider_name\":\"public-runtime\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(empty);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"external_midi_snapshot\":{"));
    assert!(supervisor_json.contains("\"discovery_state\":\"Idle\""));
    assert!(supervisor_json.contains("\"live_ownership\":{"));
    assert!(supervisor_json.contains("\"ownership_posture\":\"Unavailable\""));
    assert!(supervisor_json.contains("\"provider_name\":\"public-runtime\""));
}

#[test]
fn public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    runtime.record_plugin_event_summary(
        7,
        "lease-public-events",
        12,
        144,
        EventPacketSummary {
            total_events: 7,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 2,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 0,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let snapshot = &observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(7));
    assert_eq!(snapshot.last_block_sequence, Some(12));
    assert_eq!(snapshot.last_generated_event_bytes, 144);
    assert_eq!(snapshot.total_events, 7);
    assert_eq!(snapshot.note_expression_events, 2);
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 0);
    assert_eq!(snapshot.note_expression_tuning_events, 1);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );
    assert_eq!(snapshot.segment_count, 1);
    assert_eq!(snapshot.segment_epochs, vec![7]);
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count,
        2
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"plugin_events\":{"));
    assert!(observation_json.contains("\"note_expression_events\":2"));
    assert!(observation_json.contains("\"supports_note_expression_count\":2"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_events\":{"));
    assert!(supervisor_json.contains("\"last_generated_event_bytes\":144"));
    assert!(supervisor_json.contains("\"supports_note_expression_count\":2"));
}

#[test]
fn public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    runtime.record_plugin_event_summary(
        13,
        "lease-public-controller-expression",
        21,
        288,
        EventPacketSummary {
            total_events: 10,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: true,
        supports_note_tuning_expression: true,
        supports_mpe: true,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=true tuning=true mpe=true midi2=Guarded control-surface=guarded".into(),
    };
    let controller_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "controller-expression-runtime".into(),
        device_count: 1,
        endpoint_count: 1,
        input_endpoint_count: 1,
        output_endpoint_count: 0,
        duplex_endpoint_count: 0,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:surface:1".into(),
            device_name: "Surface One".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 1,
            summary: "device=Surface One endpoints=1".into(),
        }],
        endpoints: vec![signal_runtime::RuntimeExternalMidiEndpointDescriptor {
            endpoint_id: "endpoint:surface:1".into(),
            endpoint_name: "Surface One Input".into(),
            device_id: "device:surface:1".into(),
            direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
            lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
            route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
            capability: capability.clone(),
            summary: "direction=Input route=InputObserved pressure=true timbre=true tuning=true mpe=true midi2=Guarded".into(),
        }],
        summary: "discovery=Enumerated graph=Stable provider=controller-expression-runtime devices=1 endpoints=1 routes=1".into(),
    };

    let observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_external_midi_snapshot(controller_graph.clone());
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(controller_graph);

    let snapshot = &observation.plugin_event_snapshot;
    assert_eq!(snapshot.note_expression_events, 4);
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 1);
    assert_eq!(snapshot.note_expression_tuning_events, 2);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );

    let endpoint = &observation.external_midi_snapshot.endpoints[0];
    assert!(endpoint.capability.supports_note_pressure_expression);
    assert!(endpoint.capability.supports_note_timbre_expression);
    assert!(endpoint.capability.supports_note_tuning_expression);
    assert!(endpoint.capability.supports_mpe);
    assert_eq!(
        endpoint.capability.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"note_expression_pressure_events\":1"));
    assert!(observation_json.contains("\"note_expression_timbre_events\":1"));
    assert!(observation_json.contains("\"note_expression_tuning_events\":2"));
    assert!(observation_json.contains("\"mpe_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"midi2_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"supports_note_pressure_expression\":true"));
    assert!(observation_json.contains("\"supports_note_timbre_expression\":true"));
    assert!(observation_json.contains("\"supports_note_tuning_expression\":true"));
    assert!(observation_json.contains("\"supports_mpe\":true"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_events\":{"));
    assert!(supervisor_json.contains("\"external_midi_snapshot\":{"));
    assert!(supervisor_json.contains("\"midi2_posture\":\"Guarded\""));
}

#[test]
fn public_runtime_control_surface_boundary_reports_runtime_owned_transport_and_feedback_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.control_surface_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Unavailable
    );
    assert_eq!(
        unavailable.control_surface_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.control_surface_snapshot.device_count, 0);
    assert!(unavailable.control_surface_snapshot.devices.is_empty());

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
    };
    let control_surface_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-control-surface".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:control-surface:1".into(),
            device_name: "Control Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Control Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-surface:input".into(),
                endpoint_name: "Control Surface Input".into(),
                device_id: "device:control-surface:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:control-surface:output".into(),
                endpoint_name: "Control Surface Output".into(),
                device_id: "device:control-surface:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-control-surface state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    };

    let observation = unavailable
        .clone()
        .with_external_midi_snapshot(control_surface_graph.clone());
    assert_eq!(
        observation.control_surface_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Enumerated
    );
    assert_eq!(
        observation.control_surface_snapshot.graph_state,
        signal_runtime::RuntimeControlSurfaceGraphState::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.provider_name,
        "public-control-surface"
    );
    assert_eq!(observation.control_surface_snapshot.device_count, 1);
    assert_eq!(observation.control_surface_snapshot.mapped_device_count, 1);
    assert_eq!(
        observation
            .control_surface_snapshot
            .feedback_ready_device_count,
        0
    );
    assert_eq!(observation.control_surface_snapshot.guarded_device_count, 1);
    assert_eq!(
        observation.control_surface_snapshot.devices[0].transport_posture,
        signal_runtime::RuntimeControlSurfaceTransportPosture::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.devices[0].mapping_posture,
        signal_runtime::RuntimeControlSurfaceMappingPosture::Guarded
    );
    assert_eq!(
        observation.control_surface_snapshot.devices[0].feedback_readiness,
        signal_runtime::RuntimeControlSurfaceFeedbackReadiness::Guarded
    );
    assert!(
        observation.control_surface_snapshot.devices[0]
            .capability
            .supports_feedback_output
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"control_surface_snapshot\":{"));
    assert!(observation_json.contains("\"graph_state\":\"Guarded\""));
    assert!(observation_json.contains("\"provider_name\":\"public-control-surface\""));
    assert!(observation_json.contains("\"feedback_ready_device_count\":0"));
    assert!(observation_json.contains("\"mapping_posture\":\"Guarded\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(control_surface_graph);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"control_surface_snapshot\":{"));
    assert!(supervisor_json.contains("\"transport_posture\":\"Guarded\""));
    assert!(supervisor_json.contains("\"feedback_readiness\":\"Guarded\""));
}

#[test]
fn public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let unavailable = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        unavailable.advanced_hardware_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Unavailable
    );
    assert_eq!(
        unavailable.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Unavailable
    );
    assert_eq!(
        unavailable.advanced_hardware_snapshot.provider_name,
        "runtime-unavailable"
    );
    assert_eq!(unavailable.advanced_hardware_snapshot.device_count, 0);
    assert!(unavailable.advanced_hardware_snapshot.devices.is_empty());

    let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
        supports_bounded_midi_input: true,
        supports_bounded_midi_output: true,
        supports_transport_clock: true,
        supports_note_events: true,
        supports_controller_events: true,
        supports_note_pressure_expression: true,
        supports_note_timbre_expression: false,
        supports_note_tuning_expression: false,
        supports_mpe: false,
        midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Unsupported,
        control_surface_guarded: true,
        summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
    };
    let advanced_hardware_graph = signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
        graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
        live_ownership:
            signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
        provider_name: "public-advanced-hardware".into(),
        device_count: 1,
        endpoint_count: 2,
        input_endpoint_count: 1,
        output_endpoint_count: 1,
        duplex_endpoint_count: 1,
        active_route_count: 1,
        guarded_route_count: 1,
        devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
            device_id: "device:advanced-hardware:1".into(),
            device_name: "Advanced Surface".into(),
            lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
            endpoint_count: 2,
            summary: "device=Advanced Surface endpoints=2".into(),
        }],
        endpoints: vec![
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:input".into(),
                endpoint_name: "Advanced Surface Input".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                capability: capability.clone(),
                summary: "input".into(),
            },
            signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:advanced-hardware:output".into(),
                endpoint_name: "Advanced Surface Output".into(),
                device_id: "device:advanced-hardware:1".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                capability,
                summary: "output".into(),
            },
        ],
        summary: "provider=public-advanced-hardware state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
    };

    let observation = unavailable
        .clone()
        .with_external_midi_snapshot(advanced_hardware_graph.clone());
    assert_eq!(
        observation.advanced_hardware_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Enumerated
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.graph_state,
        signal_runtime::RuntimeAdvancedHardwareGraphState::Guarded
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.provider_name,
        "public-advanced-hardware"
    );
    assert_eq!(observation.advanced_hardware_snapshot.device_count, 1);
    assert_eq!(
        observation.advanced_hardware_snapshot.portable_device_count,
        0
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.guarded_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .feedback_channel_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .display_transport_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .motor_transport_device_count,
        0
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .haptic_transport_device_count,
        0
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .scene_mapping_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .feedback_page_device_count,
        1
    );
    assert_eq!(
        observation
            .advanced_hardware_snapshot
            .safe_action_graph_device_count,
        1
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].scripting_safe_posture,
        signal_runtime::RuntimeScriptingSafeDevicePolicyPosture::Guarded
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_channel_posture,
        signal_runtime::RuntimeGuardedFeedbackChannelPosture::Guarded
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_display_feedback
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_bank_navigation
    );
    assert!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .supports_macro_triggers
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].display_transport_posture,
        signal_runtime::RuntimeDisplayTransportPosture::GuardedDisplay
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].display_content_class,
        signal_runtime::RuntimeDisplayContentClass::GuardedVendorDisplay
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].motor_transport_posture,
        signal_runtime::RuntimeMotorTransportPosture::NoMotorTransport
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].haptic_transport_posture,
        signal_runtime::RuntimeHapticTransportPosture::NoHapticTransport
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_authority,
        signal_runtime::RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_outcome,
        signal_runtime::RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].scene_mapping_posture,
        signal_runtime::RuntimeSceneMappingPosture::GuardedSceneMapping
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_page_posture,
        signal_runtime::RuntimeFeedbackPagePosture::GuardedFeedbackPages
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].feedback_page_class,
        signal_runtime::RuntimeFeedbackPageClass::GuardedVendorPage
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].safe_action_graph_posture,
        signal_runtime::RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].action_authority,
        signal_runtime::RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0].safe_action_outcome,
        signal_runtime::RuntimeSafeActionOutcome::CollapseToGuardedAction
    );
    assert_eq!(
        observation.advanced_hardware_snapshot.devices[0]
            .capability
            .action_classes,
        vec![
            signal_runtime::RuntimeAdvancedHardwareActionClass::DisplayFeedback,
            signal_runtime::RuntimeAdvancedHardwareActionClass::BankNavigation,
            signal_runtime::RuntimeAdvancedHardwareActionClass::MacroTrigger,
            signal_runtime::RuntimeAdvancedHardwareActionClass::DeviceStateObservation,
        ]
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(observation_json.contains("\"graph_state\":\"Guarded\""));
    assert!(observation_json.contains("\"provider_name\":\"public-advanced-hardware\""));
    assert!(observation_json.contains("\"feedback_channel_device_count\":1"));
    assert!(observation_json.contains("\"display_transport_device_count\":1"));
    assert!(observation_json.contains("\"scene_mapping_device_count\":1"));
    assert!(observation_json.contains("\"feedback_page_device_count\":1"));
    assert!(observation_json.contains("\"safe_action_graph_device_count\":1"));
    assert!(observation_json.contains("\"scripting_safe_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"feedback_channel_posture\":\"Guarded\""));
    assert!(observation_json.contains("\"display_transport_posture\":\"GuardedDisplay\""));
    assert!(observation_json.contains("\"display_content_class\":\"GuardedVendorDisplay\""));
    assert!(observation_json.contains("\"feedback_authority\":\"RuntimeDefault\""));
    assert!(observation_json.contains("\"feedback_outcome\":\"CollapseToGuardedFeedback\""));
    assert!(observation_json.contains("\"scene_mapping_posture\":\"GuardedSceneMapping\""));
    assert!(observation_json.contains("\"feedback_page_posture\":\"GuardedFeedbackPages\""));
    assert!(observation_json.contains("\"feedback_page_class\":\"GuardedVendorPage\""));
    assert!(observation_json.contains("\"safe_action_graph_posture\":\"GuardedSafeActionGraph\""));
    assert!(observation_json.contains("\"action_authority\":\"RuntimeDefault\""));
    assert!(observation_json.contains("\"safe_action_outcome\":\"CollapseToGuardedAction\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_external_midi_snapshot(advanced_hardware_graph);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(supervisor_json.contains("\"supports_display_feedback\":true"));
    assert!(supervisor_json.contains("\"supports_macro_triggers\":true"));
    assert!(supervisor_json.contains("\"display_transport_posture\":\"GuardedDisplay\""));
    assert!(supervisor_json.contains("\"feedback_outcome\":\"CollapseToGuardedFeedback\""));
    assert!(supervisor_json.contains("\"scene_mapping_posture\":\"GuardedSceneMapping\""));
    assert!(supervisor_json.contains("\"feedback_page_posture\":\"GuardedFeedbackPages\""));
    assert!(supervisor_json.contains("\"safe_action_outcome\":\"CollapseToGuardedAction\""));
}

#[test]
fn public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-recall-portability".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime recall portability handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public runtime recall portability configure should succeed");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    apply_public_plugin_continuity_graph(
        &mut runtime,
        "graph:public:recall-portability",
        &[("node-clap", "sandbox-clap"), ("node-vst3", "sandbox-vst3")],
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-clap",
        PluginFormat::Clap,
        "plugin:clap:public-boundary",
        31,
    );
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox-vst3",
        PluginFormat::Vst3,
        "plugin:vst3:public-instrument",
        32,
    );
    runtime.record_plugin_preset_descriptor("sandbox-clap", sample_public_preset_descriptor());
    runtime.record_plugin_ara_context(
        "sandbox-clap",
        sample_public_ara_context(
            RuntimePluginRecallPortabilityClass::ContextOnly,
            "doc:public-runtime",
            "source:lead-vocal",
            "region:verse-a",
            1_024,
            4_096,
        ),
    );
    runtime.record_plugin_ara_context(
        "sandbox-vst3",
        sample_public_ara_context(
            RuntimePluginRecallPortabilityClass::ContextOnly,
            "doc:public-runtime",
            "source:synth-bus",
            "region:hook-b",
            8_192,
            2_048,
        ),
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let clap_stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "node-clap")
        .expect("public runtime recall boundary should export clap stage");
    assert_eq!(
        clap_stage.recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::Portable
    );
    assert!(
        clap_stage
            .recall
            .payload
            .interchange
            .shared_payload_available
    );
    assert!(
        !clap_stage
            .recall
            .payload
            .interchange
            .native_supplement_required
    );
    assert_eq!(
        clap_stage
            .recall
            .payload
            .interchange
            .preset_descriptor
            .as_ref()
            .and_then(|descriptor| descriptor.label.as_deref()),
        Some("Init")
    );
    assert_eq!(
        clap_stage
            .recall
            .payload
            .ara_context
            .as_ref()
            .and_then(|context| context.document_context.as_ref())
            .map(|context| context.document_id.as_str()),
        Some("doc:public-runtime")
    );
    let vst3_stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "node-vst3")
        .expect("public runtime recall boundary should export vst3 stage");
    assert_eq!(
        vst3_stage.recall.payload.interchange.portability_class,
        RuntimePluginRecallPortabilityClass::ContextOnly
    );
    assert!(
        !vst3_stage
            .recall
            .payload
            .interchange
            .shared_payload_available
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .nodes
            .iter()
            .find(|node| node.node_id == "node-clap")
            .and_then(|node| node.plugin_recall.as_ref())
            .map(|recall| recall.payload.interchange.portability_class),
        Some(RuntimePluginRecallPortabilityClass::Portable)
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"interchange\":{"));
    assert!(observation_json.contains("\"portability_class\":\"Portable\""));
    assert!(observation_json.contains("\"portability_class\":\"ContextOnly\""));
    assert!(observation_json.contains("\"preset_descriptor\":{"));
    assert!(observation_json.contains("\"document_context\":{"));
    assert!(observation_json.contains("\"region_id\":\"region:verse-a\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_chain_snapshot\":{"));
    assert!(supervisor_json.contains("\"execution_topology_summary\":{"));
    assert!(supervisor_json.contains("\"preset_id\":\"preset:factory:init\""));
}

#[test]
fn public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public fault diagnostic safe mode should enable");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("public fault diagnostic render queue should defer");
    assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let deferred_contribution = observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .find(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
        })
        .expect("public fault diagnostic deferred-work contribution should be present");

    assert_eq!(
        observation.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        observation.fault_diagnostic_receipt.interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(deferred_contribution.active);
    assert!(deferred_contribution.event_count >= 1);
    assert_eq!(
        supervisor
            .observation
            .fault_diagnostic_receipt
            .primary_family,
        observation.fault_diagnostic_receipt.primary_family
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"primary_family\":\"DeferredWorkPressure\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(supervisor_json.contains("\"primary_family\":\"DeferredWorkPressure\""));
}

#[test]
fn public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states() {
    let mut recovering = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    recovering
        .handshake(HandshakeRequest {
            client_version: "public-runtime-device-supervision-recovering".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public device supervision recovering handshake should succeed");
    recovering
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public device supervision recovering configure should succeed");
    recovering
        .start()
        .expect("public device supervision recovering start should succeed");
    recovering.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-device-supervision-watchdog".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 2,
    });

    let recovering_observation =
        RuntimeObservationReport::capture(&recovering, &RuntimeEventRecorder::default());
    assert_eq!(
        recovering_observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .interruption_class,
        RuntimeInterruptionClass::Steady
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(
        recovering_observation
            .device_supervision_snapshot
            .watchdog_restart_count,
        1
    );

    let mut faulted = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    faulted
        .handshake(HandshakeRequest {
            client_version: "public-runtime-device-supervision-faulted".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public device supervision faulted handshake should succeed");
    faulted
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public device supervision faulted configure should succeed");
    faulted
        .start()
        .expect("public device supervision faulted start should succeed");
    let readiness = faulted.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public runtime device supervision fault boundary",
    ));
    assert!(matches!(
        readiness,
        signal_runtime::RuntimeReadiness::Failed { .. }
    ));

    let faulted_observation =
        RuntimeObservationReport::capture(&faulted, &RuntimeEventRecorder::default());
    let faulted_supervisor =
        RuntimeSupervisorReport::capture(&faulted, &RuntimeEventRecorder::default());
    assert_eq!(
        faulted_observation.device_supervision_snapshot.state,
        RuntimeDeviceSupervisionState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .restart_state,
        RuntimeDeviceRestartState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .fault_boundary,
        RuntimeDeviceFaultBoundaryState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .recovery_state,
        RuntimeRecoveryState::Faulted
    );
    assert_eq!(
        faulted_observation
            .device_supervision_snapshot
            .primary_fault_cause,
        Some(signal_runtime::RuntimeFaultCause::RuntimeError)
    );

    let rendered = faulted_supervisor.render_json();
    assert!(rendered.contains("\"device_supervision_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Faulted\""));
    assert!(rendered.contains("\"fault_boundary\":\"Faulted\""));
}

#[test]
fn public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-clock-topology".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime clock topology handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(44_100, 256))
        .expect("public runtime clock topology configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let cross_clock_duplex = sample_public_clock_topology_host_io(
        RuntimeHostClockDomain::CrossClock,
        RuntimeHostClockFallbackState::RuntimeResampled,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback,
        RuntimeHostClockDriftState::CrossClockManaged,
        RuntimeHostClockDiscontinuityState::Reconfigured,
        RuntimeHostDuplexMismatchState::CrossClockDiverged,
        RuntimeHostEndpointTopology::Duplex,
        false,
    );
    let host_observation = RuntimeHostObservationReport::new(
        observation
            .clone()
            .with_host_device_supervision(&cross_clock_duplex),
        cross_clock_duplex.clone(),
    );

    assert_eq!(
        host_observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::CrossClockManaged
    );
    assert_eq!(
        host_observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert_eq!(
        host_observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::CrossClockDiverged
    );
    assert_eq!(
        host_observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert!(!host_observation.host_io.clocking.partial_availability);

    let partial_duplex = sample_public_clock_topology_host_io(
        RuntimeHostClockDomain::SameClock,
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::Stable,
        RuntimeHostClockDriftState::Stable,
        RuntimeHostClockDiscontinuityState::Continuous,
        RuntimeHostDuplexMismatchState::PartialAvailability,
        RuntimeHostEndpointTopology::Duplex,
        true,
    );
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_host_device_supervision(&partial_duplex);
    let host_supervisor = RuntimeHostSupervisorReport::new(supervisor, partial_duplex);

    assert_eq!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::PartialAvailability
    );
    assert_eq!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .partial_availability
    );

    let rendered = host_supervisor.render_json();
    assert!(rendered.contains("\"drift_state\":\"Stable\""));
    assert!(rendered.contains("\"discontinuity_state\":\"Continuous\""));
    assert!(rendered.contains("\"duplex_mismatch_state\":\"PartialAvailability\""));
    assert!(rendered.contains("\"endpoint_topology\":\"Duplex\""));
    assert!(rendered.contains("\"partial_availability\":true"));
}

#[test]
fn public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-external-io".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime external io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(44_100, 256))
        .expect("public runtime external io configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        baseline.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let cross_clock_duplex = sample_public_clock_topology_host_io(
        RuntimeHostClockDomain::CrossClock,
        RuntimeHostClockFallbackState::RuntimeResampled,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback,
        RuntimeHostClockDriftState::CrossClockManaged,
        RuntimeHostClockDiscontinuityState::Reconfigured,
        RuntimeHostDuplexMismatchState::CrossClockDiverged,
        RuntimeHostEndpointTopology::Duplex,
        false,
    );
    let observation = baseline.with_host_external_io(&cross_clock_duplex);
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_host_external_io(&cross_clock_duplex);

    assert_eq!(
        observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramDuplex
    );
    assert_eq!(
        observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Guarded
    );
    assert_eq!(
        supervisor.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        supervisor.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Guarded
    );

    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"external_io_snapshot\":{"));
    assert!(rendered.contains("\"primary_role\":\"ProgramDuplex\""));
    assert!(rendered.contains("\"monitoring_state\":\"Guarded\""));
    assert!(rendered.contains("\"monitoring_tap_point\":\"PostHardwareOutput\""));
    assert!(rendered.contains("\"loopback_state\":\"Guarded\""));
}

#[test]
fn public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multichannel-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multichannel handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multichannel configure should succeed");
    apply_public_multichannel_graph(&mut runtime, "graph:public:multichannel");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins".into()],
        formats: vec![PluginFormat::Clap],
    });
    runtime.record_plugin_scan_results(scan_handle, vec![sample_discovered_type_record()]);
    let recorder = RuntimeEventRecorder::default();

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    let track_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "surround-track")
        .expect("surround-track node should be present");
    assert_eq!(
        track_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        track_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(track_node.input_bus_intent, RuntimeBusIntent::MainProgram);
    assert_eq!(track_node.output_bus_intent, RuntimeBusIntent::MainProgram);

    let send_node = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "analysis-send")
        .expect("analysis-send node should be present");
    assert_eq!(
        send_node.input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(
        send_node.output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Quad)
    );
    assert_eq!(send_node.output_bus_intent, RuntimeBusIntent::AuxSend);

    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .input_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        discovery.discovered_types[0]
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );

    let rendered = observation.render_json();
    assert!(rendered.contains("\"execution_topology_summary\":{"));
    assert!(rendered.contains("\"canonical_layout\":\"Surround5_1\""));
    assert!(rendered.contains("\"output_bus_intent\":\"AuxSend\""));
    assert!(rendered.contains("\"default_multichannel_io\":{"));
}

#[test]
fn public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-sidechain-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:public:sidechain");
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox:public:sidechain".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:public-boundary".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox:public:sidechain",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox:public:sidechain",
        "lease-public-sidechain",
        "region-public-sidechain",
        PluginSandboxTransportStage::Attached,
        Some(1),
        Some("public sidechain transport attached".into()),
    );
    runtime
        .process_engine_block(
            2,
            3,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime sidechain block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.secondary_input_count, 1);
    assert_eq!(topology.required_secondary_input_count, 1);
    let route = &topology.secondary_inputs[0];
    assert_eq!(route.source_id, "kick-sidechain");
    assert_eq!(route.target_id, "compressor");
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let compressor = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "compressor")
        .expect("compressor node should be present");
    let node_secondary_input = compressor
        .secondary_input
        .as_ref()
        .expect("compressor should carry sidechain receipt");
    assert_eq!(node_secondary_input.source_id, "kick-sidechain");
    assert_eq!(
        node_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );

    let stage = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .find(|chain| chain.stage_count == 1)
        .and_then(|chain| chain.stages.first())
        .expect("plugin chain stage should be present");
    let stage_secondary_input = stage
        .secondary_input
        .as_ref()
        .expect("plugin chain stage should carry sidechain receipt");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "compressor");

    let rendered = observation.render_json();
    assert!(rendered.contains("\"secondary_input_count\":1"));
    assert!(rendered.contains("\"target_kind\":\"NodeInput\""));
    assert!(rendered.contains("\"target_kind\":\"PluginInput\""));
    assert!(rendered.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
}

#[test]
fn public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multi-bus-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:public:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime multi-bus block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.bus_connection_count, 5);
    assert_eq!(topology.auxiliary_path_count, 3);
    assert!(topology.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == RuntimeBusRole::AuxSend
            && connection.target_bus_role == RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(RuntimeAuxiliaryPathKind::SendReturn)
            && connection.auxiliary_path_id.as_deref() == Some("send_return:fx:plate")
    }));
    assert!(topology.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:master"
            && path.path_kind == RuntimeAuxiliaryPathKind::Submix
            && path.bus_role == RuntimeBusRole::Submix
    }));
    assert_eq!(observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(observation.metering_snapshot.auxiliary_path_count, 3);
    assert!(observation
        .metering_snapshot
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id == "return-fx:bus:mix:master->output-main:bus:mix:master"
        }));

    let rendered = observation.render_json();
    assert!(rendered.contains("\"bus_connection_count\":5"));
    assert!(rendered.contains("\"auxiliary_path_count\":3"));
    assert!(rendered.contains("\"connection_id\":\"send-fx:bus:fx:plate->return-fx:bus:fx:plate\""));
    assert!(rendered.contains("\"auxiliary_path_id\":\"send_return:fx:plate\""));
}

#[test]
fn public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-complex-io-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime complex io handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime complex io configure should succeed");
    apply_public_complex_io_graph(&mut runtime, "graph:public:complex-io");
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_complex_multi_output_record(),
            sample_complex_bus_fx_record(),
        ],
    );
    runtime
        .apply_plugin_node_render_batch(signal_runtime::PluginNodeRenderBatch {
            graph_id: "graph:public:complex-io".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-multiout".into(),
                    sandbox_id: "sandbox:public:multiout".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                },
                signal_runtime::PluginNodeRender {
                    node_id: "plugin-bus-fx".into(),
                    sandbox_id: "sandbox:public:bus-fx".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(8),
                    ),
                    latency_samples: 16,
                    tail_samples: 24,
                    bypassed: false,
                },
            ],
        })
        .expect("public complex io render batch should apply");
    runtime
        .process_engine_block(
            4,
            6,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3),
        )
        .expect("public runtime complex io block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let discovery = &observation.plugin_discovery_snapshot;
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.capability_coverage.complex_io_type_count, 2);
    assert_eq!(
        discovery.capability_coverage.multi_output_instrument_count,
        1
    );
    assert_eq!(discovery.capability_coverage.bus_capable_fx_count, 1);
    assert_eq!(
        discovery
            .capability_coverage
            .max_complex_io_port_group_count,
        4
    );
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-multiout"
            && record.complex_io_summary.multi_output_instrument
            && record.complex_io_summary.instrument_output_group_count == 2
    }));
    assert!(discovery.discovered_types.iter().any(|record| {
        record.plugin_type_id == "plugin:vst3:public-bus-fx"
            && record.complex_io_summary.bus_capable_fx_class
                == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            && record.complex_io_summary.secondary_input_group_count == 1
    }));
    let pin_matrix = &observation.plugin_pin_matrix_snapshot;
    assert_eq!(pin_matrix.plugin_type_count, 2);
    assert_eq!(pin_matrix.negotiated_type_count, 2);
    assert_eq!(pin_matrix.dynamic_negotiated_type_count, 2);
    let multiout_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-multiout")
        .expect("public multi-output pin matrix record should be visible");
    assert_eq!(
        multiout_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::RoutePrimaryOnly
    );
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::PrimaryProgramPath));
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SecondaryProgramPath));
    let bus_fx_pin_matrix = pin_matrix
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:public-bus-fx")
        .expect("public bus-fx pin matrix record should be visible");
    assert_eq!(
        bus_fx_pin_matrix.pin_matrix_posture,
        RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.dynamic_bus_negotiation_posture,
        RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert_eq!(
        bus_fx_pin_matrix.fallback_outcome,
        RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
    );
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::SidechainPath));
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&RuntimePluginPinGroupIdentity::AuxReturnPath));

    let plugin_chain = &observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.chain_count, 1);
    assert_eq!(plugin_chain.stage_count, 2);
    let multiout_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-multiout")
        .expect("multi-output stage should be present");
    assert!(multiout_stage.complex_io_summary.has_complex_topology);
    assert!(multiout_stage.complex_io_summary.multi_output_instrument);
    assert_eq!(
        multiout_stage
            .complex_io_summary
            .instrument_output_group_count,
        2
    );
    let bus_fx_stage = plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .find(|stage| stage.node_id == "plugin-bus-fx")
        .expect("bus fx stage should be present");
    assert_eq!(
        bus_fx_stage.complex_io_summary.bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert_eq!(
        bus_fx_stage.complex_io_summary.secondary_input_group_count,
        1
    );

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public:complex-io".into(),
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
        &handoff,
    )
    .expect("public complex io offline preview should build");
    assert_eq!(preview.chain_contract.complex_io_stage_count, 2);
    assert_eq!(
        preview.chain_contract.multi_output_instrument_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.bus_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.sidechain_capable_fx_stage_count, 1);
    assert!(preview
        .chain_contract
        .complex_io_stages
        .iter()
        .any(|stage| {
            stage.plugin_type_id.as_deref() == Some("plugin:vst3:public-multiout")
                && stage.topology.multi_output_instrument
        }));
    assert!(preview
        .chain_contract
        .complex_io_stages
        .iter()
        .any(|stage| {
            stage.plugin_type_id.as_deref() == Some("plugin:vst3:public-bus-fx")
                && stage.topology.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        }));

    let supervisor = signal_runtime::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"plugin_discovery_snapshot\":{"));
    assert!(rendered.contains("\"plugin_pin_matrix_snapshot\":{"));
    assert!(rendered.contains("\"complex_io_summary\":{"));
    assert!(rendered.contains("\"pin_matrix_posture\":\"Negotiated\""));
    assert!(rendered.contains("\"dynamic_bus_negotiation_posture\":\"Negotiated\""));
    assert!(rendered.contains("\"multi_output_instrument\":true"));
    assert!(rendered.contains("\"bus_capable_fx_class\":\"SendReturnCapableFx\""));
}

#[test]
fn public_runtime_spatial_boundary_reports_runtime_owned_execution_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-spatial-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime spatial handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime spatial configure should succeed");
    apply_public_spatial_graph(&mut runtime, "graph:public:spatial");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.spatial_node_count, 2);
    assert_eq!(topology.active_spatial_node_count, 1);
    assert_eq!(topology.bypassed_spatial_node_count, 1);
    assert_eq!(topology.fallback_spatial_node_count, 1);
    assert_eq!(topology.surround_bed_spatial_node_count, 1);
    assert_eq!(topology.object_aware_spatial_node_count, 0);
    assert_eq!(topology.expanded_fallback_spatial_node_count, 1);
    assert_eq!(topology.immersive_spatial_node_count, 1);
    assert_eq!(topology.room_policy_aware_spatial_node_count, 0);
    assert_eq!(topology.fallback_room_policy_spatial_node_count, 1);
    assert_eq!(topology.deployment_spatial_node_count, 1);
    assert_eq!(topology.folded_down_spatial_node_count, 1);
    assert_eq!(topology.fallback_monitoring_scene_spatial_node_count, 1);
    assert_eq!(topology.renderer_capability_spatial_node_count, 1);
    assert_eq!(topology.negotiated_renderer_spatial_node_count, 0);
    assert_eq!(topology.immersive_export_spatial_node_count, 1);
    assert_eq!(topology.fallback_immersive_export_spatial_node_count, 1);

    let stereo = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-stereo")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("public stereo node should carry spatial execution");
    assert_eq!(stereo.adapter_class, RuntimeSpatialAdapterClass::Balance);
    assert_eq!(
        stereo.execution_mode,
        RuntimeSpatialExecutionMode::BalanceGroups
    );
    assert_eq!(
        stereo.target_environment,
        RuntimeSpatialTargetEnvironment::SourceLayout
    );
    assert_eq!(stereo.fallback_outcome, None);
    assert_eq!(stereo.bed_class, RuntimeSpatialBedClass::StereoBed);
    assert_eq!(stereo.object_role, None);
    assert_eq!(stereo.object_count, 0);
    assert_eq!(stereo.mix_policy, RuntimeSpatialMixPolicy::BedOnly);
    assert_eq!(stereo.render_scope, RuntimeSpatialRenderScope::BedRender);
    assert_eq!(stereo.expanded_fallback_outcome, None);
    assert_eq!(stereo.balance.as_deref(), Some("-0.200"));

    let surround = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-surround")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("public surround node should carry spatial execution");
    assert_eq!(surround.adapter_class, RuntimeSpatialAdapterClass::Balance);
    assert_eq!(
        surround.execution_mode,
        RuntimeSpatialExecutionMode::Bypassed
    );
    assert_eq!(
        surround.fallback_outcome,
        Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
    );
    assert_eq!(
        surround.bed_class,
        RuntimeSpatialBedClass::CanonicalSurroundBed
    );
    assert_eq!(surround.object_role, None);
    assert_eq!(surround.object_count, 0);
    assert_eq!(
        surround.mix_policy,
        RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
    );
    assert_eq!(surround.render_scope, RuntimeSpatialRenderScope::BedRender);
    assert_eq!(
        surround.expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
    );
    let surround_immersive = surround
        .immersive_room_policy
        .as_ref()
        .expect("public surround node should carry immersive room policy");
    assert_eq!(
        surround_immersive.object_rendering_posture,
        RuntimeImmersiveObjectRenderingPosture::NotRequested
    );
    assert_eq!(
        surround_immersive.room_policy_class,
        RuntimeRoomPolicyClass::FallbackRoom
    );
    assert_eq!(
        surround_immersive.room_policy_authority,
        RuntimeRoomPolicyAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_immersive.room_outcome,
        RuntimeImmersiveRoomOutcome::BypassRoomPolicy
    );
    let surround_monitoring = surround
        .deployment_monitoring
        .as_ref()
        .expect("public surround node should carry deployment and monitoring summary");
    assert_eq!(
        surround_monitoring.deployment_class,
        RuntimeDeploymentClass::FallbackDeployment
    );
    assert_eq!(
        surround_monitoring.fold_down_policy,
        RuntimeFoldDownPolicy::FoldDownToReferenceBed
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_class,
        RuntimeMonitoringSceneClass::FallbackScene
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_authority,
        RuntimeMonitoringSceneAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_monitoring.monitoring_outcome,
        RuntimeMonitoringOutcome::BypassMonitoringScene
    );
    let surround_export = surround
        .renderer_export
        .as_ref()
        .expect("public surround node should carry renderer and export summary");
    assert_eq!(
        surround_export.renderer_capability_posture,
        RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
    );
    assert_eq!(
        surround_export.capability_authority,
        RuntimeRendererCapabilityAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.immersive_export_class,
        RuntimeImmersiveExportClass::FallbackExport
    );
    assert_eq!(
        surround_export.export_authority,
        RuntimeImmersiveExportAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.export_outcome,
        RuntimeImmersiveExportOutcome::BypassImmersiveExport
    );
    assert_eq!(surround.balance.as_deref(), Some("0.350"));

    let plugin_chain = &observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.stage_count, 2);
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "spatial-stereo"
                && stage.spatial_execution.as_ref().is_some_and(|spatial| {
                    spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
                        && spatial.bed_class == RuntimeSpatialBedClass::StereoBed
                        && spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
                })
        }));
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "spatial-surround"
                && stage.spatial_execution.as_ref().is_some_and(|spatial| {
                    spatial.fallback_outcome
                        == Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
                        && spatial.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed
                        && spatial.expanded_fallback_outcome
                            == Some(
                                RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial,
                            )
                })
        }));

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public:spatial".into(),
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
        &handoff,
    )
    .expect("public spatial render preview should build");
    assert_eq!(preview.chain_contract.spatial_stage_count, 2);
    assert_eq!(preview.chain_contract.active_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.bypassed_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.fallback_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.surround_bed_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.object_aware_spatial_stage_count, 0);
    assert_eq!(
        preview.chain_contract.expanded_fallback_spatial_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.immersive_spatial_stage_count, 1);
    assert_eq!(
        preview.chain_contract.room_policy_aware_spatial_stage_count,
        0
    );
    assert_eq!(
        preview
            .chain_contract
            .fallback_room_policy_spatial_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.deployment_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.folded_down_spatial_stage_count, 1);
    assert_eq!(
        preview
            .chain_contract
            .fallback_monitoring_scene_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .renderer_capability_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .negotiated_renderer_spatial_stage_count,
        0
    );
    assert_eq!(
        preview.chain_contract.immersive_export_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .fallback_immersive_export_spatial_stage_count,
        1
    );
    assert!(preview.chain_contract.spatial_stages.iter().any(|stage| {
        stage.node_id == "spatial-stereo"
            && stage.spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
            && stage.spatial.bed_class == RuntimeSpatialBedClass::StereoBed
            && stage.spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
    }));
    assert!(preview.chain_contract.spatial_stages.iter().any(|stage| {
        stage.node_id == "spatial-surround"
            && stage.spatial.fallback_outcome
                == Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
            && stage.spatial.expanded_fallback_outcome
                == Some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
            && stage.spatial.render_scope == RuntimeSpatialRenderScope::BedRender
            && stage
                .spatial
                .immersive_room_policy
                .as_ref()
                .is_some_and(|immersive| {
                    immersive.room_policy_class == RuntimeRoomPolicyClass::FallbackRoom
                        && immersive.room_outcome == RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                })
            && stage
                .spatial
                .deployment_monitoring
                .as_ref()
                .is_some_and(|monitoring| {
                    monitoring.deployment_class == RuntimeDeploymentClass::FallbackDeployment
                        && monitoring.fold_down_policy
                            == RuntimeFoldDownPolicy::FoldDownToReferenceBed
                        && monitoring.monitoring_scene_class
                            == RuntimeMonitoringSceneClass::FallbackScene
                        && monitoring.monitoring_outcome
                            == RuntimeMonitoringOutcome::BypassMonitoringScene
                })
            && stage
                .spatial
                .renderer_export
                .as_ref()
                .is_some_and(|renderer| {
                    renderer.renderer_capability_posture
                        == RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                        && renderer.immersive_export_class
                            == RuntimeImmersiveExportClass::FallbackExport
                        && renderer.export_outcome
                            == RuntimeImmersiveExportOutcome::BypassImmersiveExport
                })
    }));

    let rendered = observation.render_json();
    assert!(rendered.contains("\"spatial_node_count\":2"));
    assert!(rendered.contains("\"active_spatial_node_count\":1"));
    assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
    assert!(rendered.contains("\"immersive_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_room_policy_spatial_node_count\":1"));
    assert!(rendered.contains("\"deployment_spatial_node_count\":1"));
    assert!(rendered.contains("\"folded_down_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_monitoring_scene_spatial_node_count\":1"));
    assert!(rendered.contains("\"renderer_capability_spatial_node_count\":1"));
    assert!(rendered.contains("\"negotiated_renderer_spatial_node_count\":0"));
    assert!(rendered.contains("\"immersive_export_spatial_node_count\":1"));
    assert!(rendered.contains("\"fallback_immersive_export_spatial_node_count\":1"));
    assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
    assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"render_scope\":\"BedRender\""));
    assert!(rendered.contains("\"execution_mode\":\"BalanceGroups\""));
    assert!(rendered.contains("\"fallback_outcome\":\"BypassSpatialProcessing\""));
    assert!(rendered.contains("\"expanded_fallback_outcome\":\"CollapseToBaselineSpatial\""));
    assert!(rendered.contains("\"immersive_room_policy\":{"));
    assert!(rendered.contains("\"room_policy_class\":\"FallbackRoom\""));
    assert!(rendered.contains("\"room_outcome\":\"BypassRoomPolicy\""));
    assert!(rendered.contains("\"deployment_monitoring\":{"));
    assert!(rendered.contains("\"deployment_class\":\"FallbackDeployment\""));
    assert!(rendered.contains("\"fold_down_policy\":\"FoldDownToReferenceBed\""));
    assert!(rendered.contains("\"monitoring_scene_class\":\"FallbackScene\""));
    assert!(rendered.contains("\"monitoring_outcome\":\"BypassMonitoringScene\""));
    assert!(rendered.contains("\"renderer_export\":{"));
    assert!(rendered.contains("\"renderer_capability_posture\":\"FallbackNegotiation\""));
    assert!(rendered.contains("\"immersive_export_class\":\"FallbackExport\""));
    assert!(rendered.contains("\"export_outcome\":\"BypassImmersiveExport\""));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"spatial_node_count\":2"));
    assert!(supervisor_json.contains("\"fallback_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"surround_bed_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"expanded_fallback_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"immersive_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"fallback_room_policy_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"deployment_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"folded_down_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"fallback_monitoring_scene_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"renderer_capability_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"immersive_export_spatial_node_count\":1"));
    assert!(supervisor_json.contains("\"adapter_class\":\"Balance\""));
    assert!(supervisor_json.contains("\"bed_class\":\"CanonicalSurroundBed\""));
    assert!(supervisor_json.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
    assert!(supervisor_json.contains("\"room_policy_class\":\"FallbackRoom\""));
    assert!(supervisor_json.contains("\"deployment_class\":\"FallbackDeployment\""));
    assert!(supervisor_json.contains("\"monitoring_scene_class\":\"FallbackScene\""));
    assert!(supervisor_json.contains("\"renderer_capability_posture\":\"FallbackNegotiation\""));
    assert!(supervisor_json.contains("\"immersive_export_class\":\"FallbackExport\""));
}

#[test]
fn public_runtime_stretch_boundary_reports_runtime_owned_engine_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-stretch-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime stretch handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime stretch configure should succeed");

    let ready_path = public_media_fixture_path("stretch-ready");
    write_public_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-stretch-ready".into(),
            content_hash: "public-stretch-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-stretch-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .expect("public stretch media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-stretch".into(),
            media_asset_id: Some("asset:sha256:public-stretch-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public stretch warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-stretch".into(),
            media_asset_id: Some("asset:sha256:public-stretch-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public stretch clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public stretch transport projection should apply");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(observation.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(observation.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation.stretch_engine_snapshot.sample_domain_clip_count,
        1
    );
    assert_eq!(observation.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].engine_class,
        signal_runtime::RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].readiness,
        signal_runtime::RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        observation.stretch_engine_snapshot.clips[0].fallback_kind,
        signal_runtime::RuntimeStretchFallbackKind::None
    );
    assert!(observation
        .render_json()
        .contains("\"stretch_engine_snapshot\":{\"clip_count\":1"));
    assert!(observation
        .render_compact()
        .contains("stretch_clips=1/1/1/0/0/0/0/0"));

    let rendered = runtime
        .render_clip_processing_buffer(signal_runtime::RuntimeClipRenderRequest {
            clip_id: "clip:public-stretch".into(),
            timeline_start_samples: 0,
            input_stage: signal_runtime::RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .expect("public stretch clip render should succeed");
    assert_eq!(
        rendered.stretch_engine_snapshot.engine_class,
        signal_runtime::RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.readiness,
        signal_runtime::RuntimeStretchReadiness::Ready
    );
    assert_eq!(
        rendered.stretch_engine_snapshot.fallback_kind,
        signal_runtime::RuntimeStretchFallbackKind::None
    );
    assert!(rendered.summary.contains("stretch=SampleDomain/Ready/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public-stretch-preview".into(),
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
    .expect("public stretch preview should build");
    assert_eq!(preview.stretch_engine_snapshot.clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.ready_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.sample_domain_clip_count, 1);
    assert_eq!(preview.stretch_engine_snapshot.fallback_clip_count, 0);
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].engine_class,
        signal_runtime::RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(
        preview.stretch_engine_snapshot.clips[0].readiness,
        signal_runtime::RuntimeStretchReadiness::Ready
    );
    assert!(preview.summary.contains("stretch=1/fallback=0"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"stretch_engine_snapshot\":{"));
    assert!(supervisor_json.contains("\"sample_domain_clip_count\":1"));
    assert!(supervisor_json.contains("\"engine_class\":\"SampleDomain\""));

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
fn public_runtime_marker_analysis_boundary_reports_runtime_owned_analysis_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-marker-analysis-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime marker-analysis handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime marker-analysis configure should succeed");

    let ready_path = public_media_fixture_path("marker-analysis-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-marker-analysis-ready".into(),
            content_hash: "public-marker-analysis-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-marker-analysis-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public marker-analysis media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:public-marker-analysis-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public marker-analysis warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:public-marker-analysis-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public marker-analysis clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public marker-analysis transport projection should apply");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(observation.marker_analysis_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(observation.marker_analysis_snapshot.warp_marker_count > 0);
    assert!(observation.marker_analysis_snapshot.transient_anchor_count > 0);
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_posture,
        signal_runtime::RuntimeTempoAssistPosture::Ready
    );
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_hint_source,
        signal_runtime::RuntimeTempoAssistHintSource::SourceTempo
    );
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_hint_bpm,
        Some(120.0)
    );
    assert!(observation
        .render_json()
        .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));
    assert!(observation
        .render_compact()
        .contains("marker_analysis_clips=1/1/0/0/0"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));
    assert!(supervisor_json.contains("\"tempo_assist_ready_clip_count\":1"));
    assert!(supervisor_json.contains("\"tempo_assist_posture\":\"Ready\""));
    assert!(supervisor_json.contains("\"tempo_assist_hint_source\":\"SourceTempo\""));

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
fn public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-transform-artifact-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime transform-artifact handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime transform-artifact configure should succeed");

    let ready_path = public_media_fixture_path("transform-artifact-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-transform-artifact-ready".into(),
            content_hash: "public-transform-artifact-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-transform-artifact-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public transform-artifact media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:public-transform-artifact-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public transform-artifact warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-transform-artifact".into(),
            media_asset_id: Some("asset:sha256:public-transform-artifact-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public transform-artifact clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public transform-artifact transport projection should apply");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
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
        signal_runtime::RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        signal_runtime::RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        observation
            .transform_artifact_snapshot
            .transform_persistence
            .cache_placement_outcome,
        signal_runtime::RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot
    );
    assert_eq!(
        observation.transform_artifact_snapshot.clips[0].readiness,
        signal_runtime::RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        observation.transform_artifact_snapshot.clips[0].reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(observation.transform_artifact_snapshot.clips[0].cached_media_ready);
    assert!(observation
        .render_json()
        .contains("\"transform_artifact_snapshot\":{\"clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"transform_persistence\":{\"persistence_posture\":\"AssetScopedTransformPersistence\""
    ));
    assert!(observation
        .render_compact()
        .contains("transform_artifacts=1/1/0/0/0"));

    let rendered = runtime
        .render_clip_processing_buffer(signal_runtime::RuntimeClipRenderRequest {
            clip_id: "clip:public-transform-artifact".into(),
            timeline_start_samples: 0,
            input_stage: signal_runtime::RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .expect("public transform-artifact clip render should succeed");
    assert_eq!(
        rendered.transform_artifact_snapshot.readiness,
        signal_runtime::RuntimeTransformArtifactReadiness::Ready
    );
    assert_eq!(
        rendered.transform_artifact_snapshot.reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(rendered.transform_artifact_snapshot.cached_media_ready);
    assert!(rendered
        .summary
        .contains("transform=Ready/Reusable/cached_media=true"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public-transform-artifact-preview".into(),
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
    .expect("public transform-artifact preview should build");
    assert_eq!(preview.transform_artifact_snapshot.clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.ready_clip_count, 1);
    assert_eq!(preview.transform_artifact_snapshot.reusable_clip_count, 1);
    assert_eq!(
        preview
            .transform_artifact_snapshot
            .transform_persistence
            .retention_outcome,
        signal_runtime::RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
    );
    assert_eq!(
        preview.transform_artifact_snapshot.clips[0].reuse_state,
        signal_runtime::RuntimeTransformArtifactReuseState::Reusable
    );
    assert!(preview.summary.contains("transform_artifacts=1/reusable=1"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"transform_artifact_snapshot\":{\"clip_count\":1"));
    assert!(supervisor_json.contains("\"reusable_clip_count\":1"));
    assert!(supervisor_json.contains("\"reuse_state\":\"Reusable\""));
    assert!(supervisor_json.contains("\"persistence_posture\":\"AssetScopedTransformPersistence\""));

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
fn public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-preview-transform-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime preview-transform handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime preview-transform configure should succeed");

    let ready_path = public_media_fixture_path("preview-transform-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-preview-transform-ready".into(),
            content_hash: "public-preview-transform-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-preview-transform-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public preview-transform media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-preview-transform".into(),
            media_asset_id: Some("asset:sha256:public-preview-transform-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public preview-transform warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-preview-transform".into(),
            media_asset_id: Some("asset:sha256:public-preview-transform-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public preview-transform clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public preview-transform transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:public-preview-transform-ready")
        .expect("public preview-transform media preview should start");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
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
        signal_runtime::RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .audition_sink_class,
        signal_runtime::RuntimeAuditionSinkClass::GuardedPreviewSink
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .audition_sink_authority,
        signal_runtime::RuntimeAuditionSinkAuthority::RuntimeDefault
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .low_latency_device_policy_class,
        signal_runtime::RuntimeLowLatencyDevicePolicyClass::GuardedLowLatencyDevicePolicy
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_device_policy
            .low_latency_device_policy_outcome,
        signal_runtime::RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        signal_runtime::RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_class,
        signal_runtime::RuntimePreviewBrowserQueueClass::SingleAssetAuditionQueue
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .queue_outcome,
        signal_runtime::RuntimePreviewBrowserQueueOutcome::PreserveActivePreviewRequest
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .audition_posture,
        signal_runtime::RuntimeMediaAuditionOrchestrationPosture::DirectRuntimeAuditionOrchestration
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .audition_authority,
        signal_runtime::RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .audition_continuity_outcome,
        signal_runtime::RuntimeMediaAuditionContinuityOutcome::PreserveActiveAudition
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_posture,
        signal_runtime::RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_authority,
        signal_runtime::RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        signal_runtime::RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert_eq!(
        observation
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        observation.preview_transform_snapshot.clips[0].service_class,
        signal_runtime::RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        observation.preview_transform_snapshot.clips[0].readiness,
        signal_runtime::RuntimePreviewTransformReadiness::Ready
    );
    assert!(observation.preview_transform_snapshot.clips[0].audition_active);
    assert!(observation
        .render_json()
        .contains("\"preview_transform_snapshot\":{\"clip_count\":1"));
    assert!(observation
        .render_json()
        .contains("\"active_audition_clip_count\":1"));
    assert!(observation.render_json().contains(
        "\"preview_device_policy\":{\"routing_posture\":\"GuardedPreviewOutputRouting\""
    ));
    assert!(observation
        .render_json()
        .contains("\"preview_workflow\":{\"queue_posture\":\"SingleActivePreviewQueue\""));

    let rendered = runtime
        .render_clip_processing_buffer(signal_runtime::RuntimeClipRenderRequest {
            clip_id: "clip:public-preview-transform".into(),
            timeline_start_samples: 0,
            input_stage: signal_runtime::RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .expect("public preview-transform clip render should succeed");
    assert_eq!(
        rendered.preview_transform_snapshot.service_class,
        signal_runtime::RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        rendered.preview_transform_snapshot.readiness,
        signal_runtime::RuntimePreviewTransformReadiness::Ready
    );
    assert!(rendered.preview_transform_snapshot.audition_active);
    assert!(rendered
        .summary
        .contains("preview=ArtifactBacked/Ready/None/None"));

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public-preview-transform-preview".into(),
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
    .expect("public preview-transform preview should build");
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
        signal_runtime::RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        signal_runtime::RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_class,
        signal_runtime::RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_outcome,
        signal_runtime::RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .audition_continuity_outcome,
        signal_runtime::RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        signal_runtime::RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
    assert!(preview
        .summary
        .contains("preview_transform=1/artifact_backed=1/fallback=0"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"preview_transform_snapshot\":{\"clip_count\":1"));
    assert!(supervisor_json.contains("\"active_audition_clip_count\":1"));
    assert!(supervisor_json.contains("\"service_class\":\"ArtifactBacked\""));
    assert!(supervisor_json.contains("\"routing_posture\":\"GuardedPreviewOutputRouting\""));
    assert!(supervisor_json.contains("\"queue_posture\":\"SingleActivePreviewQueue\""));

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
fn public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime media-service configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let ready_path = public_media_fixture_path("ready");
    let missing_path = public_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-ready".into(),
                content_hash: "public-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "public-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-missing".into(),
                content_hash: "public-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "public-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("public runtime media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:public-media-ready")
        .expect("public runtime media preview should start");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(observation.media_pipeline_snapshot.ready_asset_count, 1);
    assert_eq!(observation.media_pipeline_snapshot.invalid_asset_count, 1);
    assert_eq!(observation.media_service_snapshot.indexed_asset_count, 2);
    assert_eq!(
        observation
            .media_service_snapshot
            .analysis_ready_asset_count,
        1
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.previewable_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.invalidated_asset_count,
        1
    );
    assert!(observation.media_service_snapshot.invalidation_active);
    assert_eq!(
        observation.media_service_snapshot.preview_state,
        signal_runtime::RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-ready")
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-missing")
    );
    assert!(observation
        .media_service_snapshot
        .last_invalidation_error
        .is_some());
    assert_eq!(
        supervisor.observation.media_pipeline_snapshot.asset_count,
        observation.media_pipeline_snapshot.asset_count
    );
    assert_eq!(
        supervisor.observation.media_service_snapshot.preview_state,
        signal_runtime::RuntimeMediaPreviewState::Previewing
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"media_pipeline_snapshot\":{"));
    assert!(observation_json.contains("\"media_service_snapshot\":{"));
    assert!(observation_json.contains("\"invalidated_asset_count\":1"));
    assert!(observation_json.contains("\"preview_state\":\"Previewing\""));
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"media_pipeline_snapshot\":{"));
    assert!(supervisor_json.contains("\"media_service_snapshot\":{"));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:public-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-analysis-metadata".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime analysis-metadata handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime analysis-metadata configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let ready_path = public_media_fixture_path("analysis-ready");
    let missing_path = public_media_fixture_path("analysis-missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-analysis-ready".into(),
                content_hash: "public-analysis-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "public-analysis-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public analysis fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-analysis-missing".into(),
                content_hash: "public-analysis-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "public-analysis-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("public runtime analysis metadata assets should reconcile");

    let library_snapshot = runtime.get_media_library_service_snapshot();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(library_snapshot.indexed_asset_count, 2);
    assert_eq!(library_snapshot.ready_descriptor_count, 1);
    assert_eq!(library_snapshot.invalidated_descriptor_count, 1);
    assert_eq!(library_snapshot.unavailable_descriptor_count, 0);
    assert_eq!(library_snapshot.loudness_ready_descriptor_count, 1);
    assert_eq!(library_snapshot.character_ready_descriptor_count, 1);
    let ready = library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:public-analysis-ready")
        .expect("public ready analysis descriptor");
    assert_eq!(
        ready.metadata_state,
        signal_runtime::RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert_eq!(
        ready.loudness_state,
        signal_runtime::RuntimeMediaAnalysisFamilyState::Ready
    );
    assert_eq!(
        ready.character_state,
        signal_runtime::RuntimeMediaAnalysisFamilyState::Ready
    );
    assert_eq!(
        ready.rhythm_state,
        signal_runtime::RuntimeMediaAnalysisFamilyState::Deferred
    );
    assert_eq!(
        ready.tonal_state,
        signal_runtime::RuntimeMediaAnalysisFamilyState::Deferred
    );
    assert_eq!(
        ready.embedding_state,
        signal_runtime::RuntimeMediaAnalysisFamilyState::Deferred
    );
    assert!(ready.loudness.is_some());
    assert!(ready.character.is_some());
    let loudness = ready.loudness.as_ref().expect("public loudness descriptor");
    assert!(loudness.integrated_lufs.is_finite());
    assert!(loudness.true_peak_dbtp.is_finite());
    let character = ready
        .character
        .as_ref()
        .expect("public character descriptor");
    assert!(character.centroid_hz.is_finite());
    assert!(character.dynamic_range.is_finite());

    let invalidated = library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:public-analysis-missing")
        .expect("public invalidated analysis descriptor");
    assert_eq!(
        invalidated.metadata_state,
        signal_runtime::RuntimeMediaAnalysisDescriptorState::Invalidated
    );
    assert!(invalidated.last_error.is_some());

    assert_eq!(
        observation.media_library_snapshot.ready_descriptor_count,
        library_snapshot.ready_descriptor_count
    );
    assert_eq!(
        supervisor
            .observation
            .media_library_snapshot
            .invalidated_descriptor_count,
        library_snapshot.invalidated_descriptor_count
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"media_library_snapshot\":{"));
    assert!(observation_json.contains("\"ready_descriptor_count\":1"));
    assert!(observation_json.contains("\"invalidated_descriptor_count\":1"));
    assert!(observation_json.contains("\"loudness_ready_descriptor_count\":1"));
    assert!(observation_json.contains("\"character_ready_descriptor_count\":1"));
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"media_library_snapshot\":{"));
    assert!(supervisor_json.contains("\"metadata_state\":\"Ready\""));
    assert!(supervisor_json.contains("\"metadata_state\":\"Invalidated\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:public-analysis-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn public_runtime_block_timing_boundary_reports_bounded_runtime_measurements() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 47),
        )
        .expect("public block timing block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[observation.clone()]);

    assert_eq!(
        observation.engine_block_snapshot.last_block_sequence,
        Some(1)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert!(
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
            .expect("public block timing should expose latest execution time")
            > 0
    );
    assert_eq!(
        performance.last_block_execution_time_ns,
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
    );
    assert_eq!(
        performance.last_block_deadline_budget_ns,
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns
    );
    assert_eq!(
        performance.last_block_deadline_pressure,
        observation
            .engine_block_snapshot
            .last_block_deadline_pressure
    );
    assert!(matches!(
        performance.last_block_deadline_pressure,
        RuntimeBlockDeadlinePressure::Normal
            | RuntimeBlockDeadlinePressure::Elevated
            | RuntimeBlockDeadlinePressure::Critical
            | RuntimeBlockDeadlinePressure::Overrun
    ));
    assert_eq!(
        supervisor
            .performance_snapshot()
            .last_block_execution_time_ns,
        performance.last_block_execution_time_ns
    );
    assert_eq!(trace.observation_count, 1);
    assert_eq!(
        trace.peak_block_execution_time_ns,
        performance
            .last_block_execution_time_ns
            .expect("trace should preserve the public latest block timing")
    );
    assert_eq!(
        trace.peak_block_budget_utilization_percent,
        performance
            .last_block_budget_utilization_percent
            .expect("trace should preserve public budget utilization")
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"last_block_execution_time_ns\":"));
    assert!(observation_json.contains("\"last_block_deadline_pressure\":"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"engine_block_snapshot\":{"));
    assert!(supervisor_json.contains("\"last_block_deadline_budget_ns\":1000000"));

    let performance_json = performance.render_json();
    assert!(performance_json.contains("\"last_block_execution_time_ns\":"));
    assert!(performance_json.contains("\"last_block_deadline_pressure\":"));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"peak_block_execution_time_ns\":"));
    assert!(trace_json.contains("\"peak_block_budget_utilization_percent\":"));
}

#[test]
fn public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 31),
        )
        .expect("public critical-path block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[observation.clone()]);

    assert!(performance.hot_latency_node_id.is_some());
    assert!(performance.hot_latency_group_node_count > 0);
    assert!(matches!(
        performance.critical_path_lane.as_deref(),
        Some("Realtime") | Some("Anticipative")
    ));
    assert!(!performance.worker_lane_summaries.is_empty());

    let critical_lane_summary = performance
        .worker_lane_summaries
        .iter()
        .find(|summary| {
            Some(match summary.lane {
                GraphExecutionLane::Realtime => "Realtime",
                GraphExecutionLane::Anticipative => "Anticipative",
            }) == performance.critical_path_lane.as_deref()
        })
        .expect("public critical-path lane should resolve to a typed worker-lane summary");
    assert_eq!(
        performance.critical_path_lane_node_count,
        critical_lane_summary.node_count
    );
    assert_eq!(
        performance.critical_path_lane_plugin_backed_node_count,
        critical_lane_summary.plugin_backed_node_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        critical_lane_summary.total_latency_samples
    );
    assert_eq!(
        supervisor.performance_snapshot().critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_hot_latency_group_node_count,
        performance.hot_latency_group_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_critical_path_lane_total_latency_samples,
        performance.critical_path_lane_total_latency_samples
    );

    let performance_json = performance.render_json();
    assert!(performance_json.contains("\"hot_latency_group_node_count\":"));
    assert!(performance_json.contains("\"worker_lane_summaries\":["));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"peak_critical_path_lane\":"));
    assert!(trace_json.contains("\"peak_hot_latency_group_node_count\":"));
}

#[test]
fn public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-deferred-work-policy".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public deferred-work configure should succeed");
    apply_public_render_graph(&mut runtime, "graph:public:deferred-work-policy");

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode for deferred-work proof");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public:deferred-work:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 96,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer the public render queue request");
    let deferred_observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let deferred_supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let deferred_receipt = deferred_observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("deferred observation should carry a scheduler-policy receipt");
    assert_eq!(
        deferred_receipt.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    assert_eq!(
        deferred_receipt.reason,
        RuntimeDeferredServiceReason::SafeMode
    );
    assert_eq!(
        deferred_receipt.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        deferred_receipt.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        deferred_receipt.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(deferred_receipt.starvation_risk);
    assert_eq!(deferred_receipt.starved_work_item_count, 1);
    assert_eq!(deferred_receipt.cancellation_cause, None);

    let deferred_performance = deferred_supervisor.performance_snapshot();
    assert_eq!(
        deferred_performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert_eq!(
        deferred_performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::UserVisible)
    );
    assert_eq!(
        deferred_performance.background_service_blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        deferred_performance.background_service_backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(deferred_performance.background_service_starvation_risk);
    assert_eq!(
        deferred_performance.background_service_starved_work_item_count,
        1
    );

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode before abort proof");
    let abort_error = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: String::new(),
            artifact_root_path: None,
            report_path: None,
        })
        .expect_err("empty purge request id should record a typed cancellation policy");
    assert!(abort_error.message.contains("requires a request id"));

    let abort_observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let abort_supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let abort_receipt = abort_observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("abort observation should carry a scheduler-policy receipt");
    assert_eq!(
        abort_receipt.decision,
        RuntimeDeferredServiceDecision::Abort
    );
    assert_eq!(
        abort_receipt.reason,
        RuntimeDeferredServiceReason::InvalidRequest
    );
    assert_eq!(
        abort_receipt.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
    );
    assert_eq!(abort_receipt.blocking_priority_band, None);
    assert_eq!(abort_receipt.backpressure_source, None);
    assert!(!abort_receipt.starvation_risk);
    assert_eq!(
        abort_receipt.cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(abort_receipt.cancelled_work_item_count, 1);

    let abort_performance = abort_supervisor.performance_snapshot();
    assert_eq!(
        abort_performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Abort)
    );
    assert_eq!(
        abort_performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::Maintenance)
    );
    assert_eq!(
        abort_performance.background_service_cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(
        abort_performance.background_service_cancelled_work_item_count,
        1
    );

    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[
        deferred_observation.clone(),
        abort_observation.clone(),
    ]);
    assert_eq!(trace.observation_count, 2);
    assert_eq!(trace.background_service_defer_count, 1);
    assert_eq!(trace.background_service_abort_count, 1);
    assert_eq!(trace.background_starvation_observation_count, 1);
    assert_eq!(trace.peak_background_starved_work_item_count, 1);
    assert_eq!(trace.background_cancellation_observation_count, 1);
    assert_eq!(trace.peak_background_cancelled_work_item_count, 1);
    assert_eq!(trace.background_realtime_backpressure_observation_count, 0);
    assert_eq!(trace.background_recovery_backpressure_observation_count, 1);

    let deferred_json = deferred_supervisor.render_json();
    assert!(deferred_json.contains("\"last_deferred_service\":{"));
    assert!(deferred_json.contains("\"backpressure_source\":\"SafeMode\""));
    assert!(deferred_json.contains("\"starvation_risk\":true"));

    let abort_json = abort_supervisor.render_json();
    assert!(abort_json.contains("\"cancellation_cause\":\"InvalidRequest\""));
    assert!(abort_json.contains("\"cancelled_work_item_count\":1"));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"background_cancellation_observation_count\":1"));
    assert!(trace_json.contains("\"peak_background_cancelled_work_item_count\":1"));
}

#[test]
fn public_runtime_interruption_boundary_reports_restartable_runtime_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-interruption".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public interruption boundary handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public interruption boundary configure should succeed");
    runtime
        .start()
        .expect("public interruption boundary start should succeed");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-boundary-sandbox".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "public-runtime-boundary-sandbox".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

    assert_eq!(
        observation.fault_status.recovery_state,
        RuntimeRecoveryState::Recovering
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(!observation.interruption_summary.rebindable);

    let rendered = observation.render_json();
    assert!(rendered.contains("\"fault_status\":{"));
    assert!(rendered.contains("\"interruption_summary\":{"));
    assert!(rendered.contains("\"class\":\"Restartable\""));
    assert!(rendered.contains("\"primary_fault_cause\":\"WatchdogRestart\""));
}

#[test]
fn public_runtime_interruption_boundary_reports_resumable_deferred_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public interruption boundary safe mode should enable");

    let queue = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public-interruption:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer public interruption boundary queue");

    assert_eq!(
        queue.orchestration.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(!queue.orchestration.interruption_rebindable);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let deferred = observation
        .last_deferred_service_receipt
        .expect("deferred receipt should be exported on the public observation boundary");
    assert_eq!(
        deferred.interruption_class,
        RuntimeInterruptionClass::Resumable
    );
    assert!(!deferred.interruption_rebindable);
}

#[test]
fn public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states()
{
    let recorder = RuntimeEventRecorder::default();

    let mut resumable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    resumable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    resumable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut resumable, "graph:public:recording-resumable");
    resumable
        .start()
        .expect("public recording continuity start should succeed");
    resumable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:resumable".into(),
            track_id: "track:public:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .expect("public recording capture should start");
    resumable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 44),
        )
        .expect("public recording block should process");
    resumable
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("public recording safe mode should enable");
    let resumable_report = RuntimeObservationReport::capture(&resumable, &recorder);
    assert_eq!(
        resumable_report
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let mut restartable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    restartable
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    restartable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut restartable, "graph:public:recording-restartable");
    restartable
        .start()
        .expect("public recording start should succeed");
    restartable
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:restartable".into(),
            track_id: "track:public:restartable".into(),
            start_samples: 3_072,
            capture_path: std::env::temp_dir()
                .join("signal-public-recording-restartable.wav")
                .display()
                .to_string(),
        })
        .expect("public restartable capture should start");
    restartable
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 45),
        )
        .expect("public restartable block should process");
    restartable
        .stop(signal_runtime::StopReason::DeviceReconfigure)
        .expect("public restartable stop should succeed");
    let restartable_report = RuntimeObservationReport::capture(&restartable, &recorder);
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );
    assert_eq!(
        restartable_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
    );

    let mut terminal = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    terminal
        .handshake(HandshakeRequest {
            client_version: "public-recording-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public recording continuity handshake should succeed");
    terminal
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public recording continuity configure should succeed");
    apply_public_capture_graph(&mut terminal, "graph:public:recording-terminal");
    terminal
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:public:terminal".into(),
            track_id: "track:public:terminal".into(),
            start_samples: 4_096,
            capture_path: "/dev/null/signal-public-recording-terminal.wav".into(),
        })
        .expect("public terminal capture should start");
    terminal
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 46),
        )
        .expect("public terminal block should process");
    let terminal_error = terminal.finish_recording_capture().unwrap_err();
    assert_eq!(
        terminal_error.kind,
        signal_runtime::RuntimeErrorKind::ResourceUnavailable
    );
    let terminal_report = RuntimeObservationReport::capture(&terminal, &recorder);
    assert_eq!(
        terminal_report.recording_capture_snapshot.state,
        Some(signal_runtime::RuntimeRecordingCaptureState::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_class),
        Some(RuntimeRecordingCaptureCheckpointClass::Failed)
    );
    assert_eq!(
        terminal_report
            .recording_capture_snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
}

#[test]
fn public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states(
) {
    let recorder = RuntimeEventRecorder::default();

    let mut resumable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    resumable
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    resumable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut resumable, "graph:public:render-resumable");
    resumable
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:resumable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public resumable render should begin");
    resumable
        .advance_offline_render_execution("render:public:resumable")
        .expect("public resumable render should advance");
    resumable
        .pause_offline_render_execution("render:public:resumable")
        .expect("public resumable render should pause");
    let resumable_report = RuntimeObservationReport::capture(&resumable, &recorder);
    assert_eq!(
        resumable_report
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );

    let mut restartable = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    restartable
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    restartable
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut restartable, "graph:public:render-restartable");
    restartable
        .start()
        .expect("public render runtime should start");
    restartable
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public restartable render should begin");
    restartable
        .advance_offline_render_execution("render:public:restartable")
        .expect("public restartable render should advance");
    restartable
        .stop(StopReason::DeviceReconfigure)
        .expect("public restartable render should stop");
    restartable
        .restart(RestartRequest { reconfigure: None })
        .expect("public restartable render should restart");
    let restartable_report = RuntimeObservationReport::capture(&restartable, &recorder);
    assert_eq!(
        restartable_report
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    let mut terminal = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    terminal
        .handshake(HandshakeRequest {
            client_version: "public-render-continuity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public render continuity handshake should succeed");
    terminal
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public render continuity configure should succeed");
    apply_public_render_graph(&mut terminal, "graph:public:render-terminal");
    terminal
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:public:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-public-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .expect("public terminal render should begin");
    let mut terminal_error_observed = false;
    for _ in 0..16 {
        match terminal.advance_offline_render_execution("render:public:terminal") {
            Ok(_) => continue,
            Err(_) => {
                terminal_error_observed = true;
                break;
            }
        }
    }
    assert!(terminal_error_observed);
    let terminal_report = RuntimeObservationReport::capture(&terminal, &recorder);
    assert_eq!(
        terminal_report
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        terminal_report
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Failed\""));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}
