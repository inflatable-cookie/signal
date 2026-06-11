//! Runtime configuration and shell implementation for Signal.
#[path = "offline_render_delivery.rs"]
mod offline_render_delivery;
#[path = "offline_render_maintenance.rs"]
mod offline_render_maintenance;
#[path = "runtime_api.rs"]
mod runtime_api;
#[path = "runtime_audio_file_io.rs"]
mod runtime_audio_file_io;
#[path = "runtime_automation_state.rs"]
mod runtime_automation_state;
#[path = "runtime_contract.rs"]
mod runtime_contract;
#[path = "runtime_deferred_service.rs"]
mod runtime_deferred_service;
#[path = "runtime_engine_state.rs"]
mod runtime_engine_state;
#[path = "runtime_event_recording.rs"]
mod runtime_event_recording;
#[path = "runtime_event_surface.rs"]
mod runtime_event_surface;
#[path = "runtime_graph_projection.rs"]
mod runtime_graph_projection;
#[path = "runtime_lifecycle_state.rs"]
mod runtime_lifecycle_state;
#[path = "runtime_media_processing.rs"]
mod runtime_media_processing;
#[path = "runtime_media_services.rs"]
mod runtime_media_services;
#[path = "runtime_media_state.rs"]
mod runtime_media_state;
#[path = "runtime_observation_surface.rs"]
mod runtime_observation_surface;
#[path = "runtime_offline_render_session.rs"]
mod runtime_offline_render_session;
#[path = "runtime_planning_snapshot.rs"]
mod runtime_planning_snapshot;
#[path = "runtime_plugin_lifecycle.rs"]
mod runtime_plugin_lifecycle;
#[path = "runtime_plugin_recording.rs"]
mod runtime_plugin_recording;
#[path = "runtime_prework_admission.rs"]
mod runtime_prework_admission;
#[path = "runtime_prework_forecast.rs"]
mod runtime_prework_forecast;
#[path = "runtime_prework_service.rs"]
mod runtime_prework_service;
#[path = "runtime_prework_state.rs"]
mod runtime_prework_state;
#[path = "runtime_projection_guards.rs"]
mod runtime_projection_guards;
#[path = "runtime_recording_capture.rs"]
mod runtime_recording_capture;
#[path = "runtime_shell.rs"]
mod runtime_shell;
#[path = "runtime_supervision_state.rs"]
mod runtime_supervision_state;
#[path = "runtime_support_models.rs"]
mod runtime_support_models;
#[path = "runtime_tempo_warp_state.rs"]
mod runtime_tempo_warp_state;
#[path = "runtime_timeline_state.rs"]
mod runtime_timeline_state;
#[path = "runtime_transport_concurrency.rs"]
mod runtime_transport_concurrency;
#[path = "runtime_transport_sessions.rs"]
mod runtime_transport_sessions;
#[path = "runtime_utils.rs"]
pub(crate) mod runtime_utils;

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::interfaces::{
    BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage, DegradedReason,
    EffectiveRuntimeConfig, GraphContractProjection, GraphProjection, HandshakeRequest,
    HandshakeResponse, HeartbeatCycleStage, LeaseRolloverRecord, LingeringCleanupMode,
    LingeringCleanupQueueReceipt, LingeringCleanupTrigger, ParameterBatch,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginNodeRenderBatch,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, ProjectionReceipt, RecoveryRestartIntent,
    RestartRequest, RuntimeAcceptanceReceipt, RuntimeAutomationInterpolation,
    RuntimeAutomationProjection, RuntimeAutomationSnapshot, RuntimeBlockDeadlinePressure,
    RuntimeClipFadeShape, RuntimeClipGainShape, RuntimeClipProcessingPipelineSnapshot,
    RuntimeClipProcessingReadiness, RuntimeClipProcessingRegistration,
    RuntimeClipProcessingSnapshot, RuntimeClipProcessingStage, RuntimeClipRenderInputStage,
    RuntimeClipRenderRequest, RuntimeClipRenderResult, RuntimeConfigRequest,
    RuntimeControlSnapshot, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceClass,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeDeferredServiceReceipt, RuntimeDiagnosticsSnapshot,
    RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventSink, RuntimeExecutionPhase, RuntimeExecutionTopologySummary,
    RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeLv2PreparedNegotiationRecord,
    RuntimeMarkerAnalysisReadiness, RuntimeMarkerAnalysisSnapshot,
    RuntimeMediaAnalysisDescriptorState, RuntimeMediaAnalysisFamilyState,
    RuntimeMediaAssetRegistration, RuntimeMediaAssetSnapshot, RuntimeMediaAssetState,
    RuntimeMediaCharacterDescriptor, RuntimeMediaIndexingState, RuntimeMediaLibraryAssetDescriptor,
    RuntimeMediaLibraryServiceSnapshot, RuntimeMediaLoudnessDescriptor,
    RuntimeMediaPipelineSnapshot, RuntimeMediaPreviewState, RuntimeMediaServiceSnapshot,
    RuntimeMeterSourceRole, RuntimeMeterSourceSnapshot, RuntimeMeteringSnapshot,
    RuntimeObservationApi, RuntimeOfflineFreezeArtifactResult,
    RuntimeOfflinePluginDelegatedExecutionMerge, RuntimeOfflinePluginDelegatedExecutionOutcome,
    RuntimeOfflinePluginDelegatedExecutionReceipt, RuntimeOfflinePluginDelegatedExecutionRequest,
    RuntimeOfflinePluginExecutionBoundary, RuntimeOfflinePluginExecutionOwner,
    RuntimeOfflinePluginExecutionStageBoundary, RuntimeOfflinePluginOverrideState,
    RuntimeOfflineRenderArtifactKind, RuntimeOfflineRenderArtifactReceipt,
    RuntimeOfflineRenderCheckpointReceipt, RuntimeOfflineRenderCheckpointStage,
    RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderExecutionCancellationReceipt,
    RuntimeOfflineRenderExecutionProgressReceipt, RuntimeOfflineRenderExecutionReceipt,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderManifest,
    RuntimeOfflineRenderPurgeReceipt, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderQueueProgressReceipt, RuntimeOfflineRenderQueueResult,
    RuntimeOfflineRenderReportReceipt, RuntimeOfflineRenderRequest, RuntimeOfflineRenderResult,
    RuntimeOfflineRenderStemPreview, RuntimeOfflineRenderStemResult,
    RuntimePluginAraContextSnapshot, RuntimePluginBusCapableFxClass,
    RuntimePluginCapabilityCoverageSummary, RuntimePluginChainSnapshot,
    RuntimePluginChainStageSnapshot, RuntimePluginCompensationState,
    RuntimePluginDiscoveredTypeRecord, RuntimePluginDiscoverySnapshot, RuntimePluginDispatchState,
    RuntimePluginExecutionChainSummary,
    RuntimePluginFormatCoverageRecord, RuntimePluginFormatParityRecord,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginInterchangeSnapshot, RuntimePluginIsolationOutcome,
    RuntimePluginLifecycleSnapshot, RuntimePluginLifecycleState, RuntimePluginParityBand,
    RuntimePluginPlacementPolicy, RuntimePluginPlacementRuleMatcher, RuntimePluginPresetDescriptor,
    RuntimePluginRecallHandoffSnapshot, RuntimePluginRecallPayload,
    RuntimePluginRecallPortabilityClass, RuntimePluginRecallSnapshot, RuntimePluginRecallState,
    RuntimePluginSandboxSnapshot, RuntimePluginScanReceipt, RuntimePreviewTransformClipSnapshot,
    RuntimePreviewTransformDegradedState, RuntimePreviewTransformFallbackKind,
    RuntimePreviewTransformReadiness, RuntimePreviewTransformServiceClass,
    RuntimePreviewTransformServiceSnapshot, RuntimePreworkBacklogClass, RuntimePreworkCacheState,
    RuntimePreworkForecastMode, RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureCheckpointSnapshot,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureSnapshot, RuntimeRecordingCaptureStartRequest,
    RuntimeRecordingCaptureState, RuntimeSchedulerSnapshot, RuntimeSchedulerState,
    RuntimeSchedulerTopologyIssue, RuntimeSchedulerTopologySummary,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputRouteSummary,
    RuntimeSecondaryInputTargetKind, RuntimeStretchClipSnapshot, RuntimeStretchEngineClass,
    RuntimeStretchEngineSnapshot, RuntimeStretchReadiness, RuntimeSupervisionSnapshot,
    RuntimeTempoMapInterpolation, RuntimeTempoMapProjection, RuntimeTempoMapSegmentProjection,
    RuntimeTempoMapSegmentSnapshot, RuntimeTempoMapSnapshot, RuntimeTempoSource,
    RuntimeTimelineSnapshot, RuntimeTransformArtifactClipSnapshot,
    RuntimeTransformArtifactInvalidationState, RuntimeTransformArtifactReadiness,
    RuntimeTransformArtifactReuseState, RuntimeTransformArtifactSnapshot,
    RuntimeTransportConcurrencySnapshot, RuntimeTransportObservationSnapshot,
    RuntimeTransportTransitionKind, RuntimeWarpClipRegistration, RuntimeWarpClipSnapshot,
    RuntimeWarpMode, RuntimeWarpPipelineSnapshot, RuntimeWarpReadiness, RuntimeWatchdogTrigger,
    SafeModeRequest, ScanHandle, ScheduleProjection, StopReason, SubscriptionHandle,
    TransportAttachIntent, TransportProjection, TransportSessionProvenance, TransportSessionState,
    WatchdogRestartRecord,
};
use offline_render_delivery::{materialize_offline_render_delivery, offline_render_manifest};
use offline_render_maintenance::{
    purge_offline_render_directory, purge_offline_render_file, refresh_offline_render_result,
};
use runtime_audio_file_io::write_audio_buffer_wav;
use runtime_automation_state::{
    graph_parameter_target_from_runtime_target, graph_stage_parameter_sort_key,
    RuntimeAutomationBatchMetrics, RuntimeAutomationExecutionRecord, RuntimeAutomationState,
};
pub use runtime_contract::{RuntimeConfig, RuntimeProfile};
pub(crate) use runtime_engine_state::{
    RuntimeEngineState, RuntimePluginBackedBindingSummary, RuntimePluginRenderedNodeState,
    RuntimePreworkTransportCondition, PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW, PREWORK_QUEUE_CAPACITY,
};
use runtime_media_processing::{
    adapt_audio_buffer_layout, analyze_runtime_media_asset, decode_runtime_media_asset,
    hash_audio_buffer, mix_audio_buffer, peak_abs, resample_audio_buffer_linear, rms,
    sample_audio_buffer_linear, write_offline_render_block,
};
pub(crate) use runtime_media_state::{
    RuntimeClipProcessingPipelineStateModel, RuntimeMediaPipelineStateModel,
    RuntimeMeterContractMetadata, RuntimeMeteringStateModel,
};
use runtime_offline_render_session::RuntimeOfflineRenderExecutionSession;
use runtime_plugin_lifecycle::{
    runtime_plugin_boundary_counts, runtime_plugin_stage_assignment,
    RuntimePluginLifecycleStateModel,
};
use runtime_plugin_recording::{
    runtime_plugin_capability_coverage, runtime_plugin_format_coverage,
};
use runtime_recording_capture::RuntimeRecordingCaptureStateModel;
pub(crate) use runtime_support_models::{
    runtime_plugin_chain_id, runtime_plugin_discovered_type_for_recall,
    RuntimeMediaAnalysisStateModel, RuntimeMediaPipelineAsset, RuntimeMediaPipelinePolicy,
    RuntimePluginCompensationObservation, RuntimeRecordingCapturePolicy,
};
use runtime_support_models::{RuntimePluginDiscoveryStateModel, RuntimeSupervisionState};
pub(crate) use runtime_tempo_warp_state::{
    RuntimeResolvedTempo, RuntimeTempoMapStateModel, RuntimeWarpPipelineStateModel,
};
use runtime_timeline_state::{
    classify_transport_invalidation_reason, classify_transport_transition,
    transport_projection_from_context, RuntimeTimelineState,
};
use runtime_transport_concurrency::RuntimeTransportConcurrencyState;
use signal_graph::{
    synthetic_stereo_block, ExecutableGraph, GraphBlockReport, GraphCapturedBusOutput, GraphConfig,
    GraphExecutionContext, GraphNodeBufferContract, GraphNodeExecutionClass,
    GraphNodeRenderOverride, GraphNodeSpec, GraphNodeTopologyMetadata, GraphNodeTopologyRole,
    GraphParameterApplicationStrategy, GraphParameterBatch, GraphParameterEvent,
    GraphPreparedDispatch,
};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{PluginFeature, PluginFormat};
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};

const PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES: u32 = 64;
const OFFLINE_RENDER_PROGRESS_CHECKPOINT_TARGET_COUNT: usize = 6;
const BLOCK_DEADLINE_ELEVATED_UTILIZATION_PERCENT: f32 = 75.0;
const BLOCK_DEADLINE_CRITICAL_UTILIZATION_PERCENT: f32 = 95.0;

/// Central control-plane and observation-plane for Signal graph execution.
///
/// `SignalRuntime` owns the graph scheduler, plugin sandbox lifecycle, prework
/// service, transport and parameter state, media pipeline, recording capture,
/// and offline render queue.
///
/// Construct with [`SignalRuntime::new`], then drive through the lifecycle with
/// the [`RuntimeLifecycleApi`] trait methods (`handshake → configure → start →
/// … → stop`).  Use [`RuntimeProjectionApi`] to feed the engine a graph and
/// transport state, and [`RuntimeSupervisorApi`](crate::RuntimeSupervisorApi) to manage plugin sandboxes and
/// offline rendering.  Read state without mutating via [`RuntimeObservationApi`]
/// and event subscriptions via `RuntimeEventSink`.
pub struct SignalRuntime {
    config: RuntimeConfig,
    readiness: RuntimeReadiness,
    safe_mode_enabled: bool,
    anticipative_enabled: bool,
    active_output_device: Option<String>,
    applied_graph: Option<GraphProjection>,
    applied_schedule: Option<ScheduleProjection>,
    applied_transport: Option<TransportProjection>,
    applied_parameter_batch: Option<ParameterBatch>,
    prework_forecast_requested_mode: RuntimePreworkForecastMode,
    prework_forecast_mode: RuntimePreworkForecastMode,
    prework_forecast_policy: Option<RuntimePreworkForecastPolicy>,
    prework_forecast_profile: Option<RuntimePreworkForecastProfileSelection>,
    prework_forecast_profile_source: Option<RuntimePreworkForecastProfileSource>,
    latest_parameter_epoch: u64,
    projection_epoch: u64,
    control: RuntimeControlSnapshot,
    timeline: RuntimeTimelineState,
    automation: RuntimeAutomationState,
    engine: RuntimeEngineState,
    transport_concurrency: RuntimeTransportConcurrencyState,
    plugin_discovery: RuntimePluginDiscoveryStateModel,
    plugin_lifecycle: RuntimePluginLifecycleStateModel,
    plugin_placement_policy: RuntimePluginPlacementPolicy,
    recording_capture: RuntimeRecordingCaptureStateModel,
    metering: RuntimeMeteringStateModel,
    media_pipeline: RuntimeMediaPipelineStateModel,
    tempo_map: RuntimeTempoMapStateModel,
    warp_pipeline: RuntimeWarpPipelineStateModel,
    clip_processing_pipeline: RuntimeClipProcessingPipelineStateModel,
    diagnostics: RuntimeDiagnosticsSnapshot,
    supervision: RuntimeSupervisionState,
    last_deferred_service_receipt: RefCell<Option<RuntimeDeferredServiceReceipt>>,
    last_offline_render_session_snapshot:
        RefCell<Option<crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot>>,
    last_offline_render_cancellation_receipt:
        RefCell<Option<crate::interfaces::RuntimeOfflineRenderExecutionCancellationReceipt>>,
    last_offline_render_purge_receipt:
        RefCell<Option<crate::interfaces::RuntimeOfflineRenderPurgeReceipt>>,
    offline_render_executions: HashMap<String, RuntimeOfflineRenderExecutionSession>,
    next_subscription: u64,
    sinks: Vec<Box<dyn RuntimeEventSink>>,
}

#[cfg(test)]
#[path = "tests_support.rs"]
mod tests_support;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
