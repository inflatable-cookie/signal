#![allow(dead_code, unused_imports)]

#[path = "tests_support/forecast_helpers.rs"]
mod forecast_helpers;
#[path = "tests_support/graph_helpers.rs"]
mod graph_helpers;
#[path = "tests_support/media_fixtures.rs"]
mod media_fixtures;
#[path = "tests_support/runtime_fixtures.rs"]
mod runtime_fixtures;

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

use forecast_helpers::{
    apply_current_forecast_block_state, handshake_and_configure_with_disabled_forecast,
    seed_pending_prework_targets,
};
use graph_helpers::{
    apply_latency_runtime_graph, apply_plugin_continuity_graph,
    install_scheduler_topology_runtime_graph, record_ready_plugin_sandbox,
};
use media_fixtures::{
    temp_artifact_dir, temp_capture_path, temp_media_path, write_test_aiff, write_test_wav,
    write_transient_test_wav,
};
use runtime_fixtures::{
    filled_stereo_buffer, prepare_offline_render_engine_runtime,
    prepare_offline_render_engine_runtime_without_cached_plugin_render, prepare_sidechain_runtime,
    prepare_spatial_runtime,
};
