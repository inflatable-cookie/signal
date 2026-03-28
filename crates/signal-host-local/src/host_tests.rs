
    use super::host_test_support::{
        assert_local_plugin_topology, assert_plugin_dispatch_summary,
        assert_runtime_automation_continuity, assert_runtime_automation_values,
        assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
        RuntimeAutomationExpectations,
        prepare_local_host_for_offline_render, prepare_local_host_with_lifecycle,
        prepare_local_host_without_lifecycle, temp_artifact_dir, unique_test_path, write_test_wav,
    };
    use super::{
        LocalAudioStreamState, LocalAudioTransferPolicy, LocalRuntimeHost, LOCAL_DEMO_GRAPH_ID,
        LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES, LOCAL_DEMO_PLUGIN_NODE_ID,
        LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
    };
    use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
    use signal_hardware::{
        AudioDeviceDescriptor, AudioSampleFormat, AudioStreamDirection, BackendHealth,
        HardwareBackendIdentity, HardwareClockSource, HardwareClockTopology,
        HardwareLatencyProfile, HardwareLifecycleContract, HardwareLifecycleOwnership,
        HardwareRestartPolicy, HardwareStreamConfig,
    };
    use signal_plugin::{
        CompletionState, LoopRange, PluginEvent, PluginFormat, WatchdogTriggerReason,
    };
    use signal_plugin_clap::ClapSandboxLifecycleHarness;
    use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, SampleRate};
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
        GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
        GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
        PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
        RuntimeClipProcessingRegistration, RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind,
        RuntimeHostAudioStreamState, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
        RuntimeMediaPreviewState, RuntimeObservationApi, RuntimeOfflineFreezeArtifactRequest,
        RuntimeOfflinePluginExecutionBoundary, RuntimeOfflinePluginExecutionOwner,
        RuntimeOfflinePluginExecutionStageBoundary, RuntimeOfflinePluginOverrideState,
        RuntimeOfflineRenderArtifactKind, RuntimeOfflineRenderRequest,
        RuntimeOfflineRenderStemTarget, RuntimeOfflineRenderTargetKind, RuntimePluginHostPlatform,
        RuntimePluginRecallHandoffSelection, RuntimePluginRecallHandoffStageId,
        RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage,
        SignalRuntime, StopReason, TransportAttachIntent,
    };
    use signal_runtime::{
        RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
        RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
        RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    };

#[path = "host_tests/execution.rs"]
mod execution;
#[path = "host_tests/recovery.rs"]
mod recovery;
#[path = "host_tests/reports.rs"]
mod reports;
