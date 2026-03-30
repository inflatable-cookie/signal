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

#[path = "tests/fixtures.rs"]
mod fixtures;
use fixtures::*;
#[path = "tests/clip_processing.rs"]
mod clip_processing;
#[path = "tests/core_runtime.rs"]
mod core_runtime;
#[path = "tests/discovery_parity.rs"]
mod discovery_parity;
#[path = "tests/engine_execution.rs"]
mod engine_execution;
#[path = "tests/event_lifecycle.rs"]
mod event_lifecycle;
#[path = "tests/forecast_override_lifecycle.rs"]
mod forecast_override_lifecycle;
#[path = "tests/forecast_profile_queueing.rs"]
mod forecast_profile_queueing;
#[path = "tests/forecast_profile_rebuilds.rs"]
mod forecast_profile_rebuilds;
#[path = "tests/forecast_profile_selection.rs"]
mod forecast_profile_selection;
#[path = "tests/forecast_windows.rs"]
mod forecast_windows;
#[path = "tests/graph_projection.rs"]
mod graph_projection;
#[path = "tests/lifecycle_guards.rs"]
mod lifecycle_guards;
#[path = "tests/media_service.rs"]
mod media_service;
#[path = "tests/metering_automation.rs"]
mod metering_automation;
#[path = "tests/observation_transform_receipts.rs"]
mod observation_transform_receipts;
#[path = "tests/offline_contracts.rs"]
mod offline_contracts;
#[path = "tests/offline_delegated_boundary.rs"]
mod offline_delegated_boundary;
#[path = "tests/offline_delegated_receipts.rs"]
mod offline_delegated_receipts;
#[path = "tests/offline_execution_recovery.rs"]
mod offline_execution_recovery;
#[path = "tests/offline_queue_purge.rs"]
mod offline_queue_purge;
#[path = "tests/offline_render_flow.rs"]
mod offline_render_flow;
#[path = "tests/offline_stage_model_fallback.rs"]
mod offline_stage_model_fallback;
#[path = "tests/performance_receipts.rs"]
mod performance_receipts;
#[path = "tests/plugin_binding.rs"]
mod plugin_binding;
#[path = "tests/plugin_chain_receipts.rs"]
mod plugin_chain_receipts;
#[path = "tests/plugin_chain_recovery.rs"]
mod plugin_chain_recovery;
#[path = "tests/plugin_placement.rs"]
mod plugin_placement;
#[path = "tests/pressure_policies.rs"]
mod pressure_policies;
#[path = "tests/preview_transform_reports.rs"]
mod preview_transform_reports;
#[path = "tests/prework_cache_invalidation.rs"]
mod prework_cache_invalidation;
#[path = "tests/prework_queue.rs"]
mod prework_queue;
#[path = "tests/realtime_prework_service.rs"]
mod realtime_prework_service;
#[path = "tests/realtime_scheduler_recovery.rs"]
mod realtime_scheduler_recovery;
#[path = "tests/recall_handoff.rs"]
mod recall_handoff;
#[path = "tests/recording_capture.rs"]
mod recording_capture;
#[path = "tests/routing_receipts.rs"]
mod routing_receipts;
#[path = "tests/scheduler_state.rs"]
mod scheduler_state;
#[path = "tests/scheduler_topology.rs"]
mod scheduler_topology;
#[path = "tests/transport_sessions.rs"]
mod transport_sessions;
#[path = "tests/transport_state.rs"]
mod transport_state;
#[path = "tests/watchdog_faults.rs"]
mod watchdog_faults;
