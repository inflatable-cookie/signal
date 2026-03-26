//! Runtime configuration and shell implementation for Signal.
#[path = "offline_render_delivery.rs"]
mod offline_render_delivery;
#[path = "offline_render_maintenance.rs"]
mod offline_render_maintenance;
#[path = "runtime_audio_file_io.rs"]
mod runtime_audio_file_io;
#[path = "runtime_automation_state.rs"]
mod runtime_automation_state;
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
#[path = "runtime_plugin_event_state.rs"]
mod runtime_plugin_event_state;
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
#[path = "runtime_supervision_state.rs"]
mod runtime_supervision_state;
#[path = "runtime_tempo_warp_state.rs"]
mod runtime_tempo_warp_state;
#[path = "runtime_timeline_state.rs"]
mod runtime_timeline_state;
#[path = "runtime_transport_concurrency.rs"]
mod runtime_transport_concurrency;
#[path = "runtime_transport_sessions.rs"]
mod runtime_transport_sessions;

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::interfaces::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    DegradedReason, EffectiveRuntimeConfig, GraphContractProjection, GraphProjection,
    HandshakeRequest, HandshakeResponse, HeartbeatCycleStage, LeaseRolloverRecord,
    LingeringCleanupMode, LingeringCleanupQueueReceipt, LingeringCleanupTrigger, ParameterBatch,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginNodeRenderBatch,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, ProjectionReceipt, RecoveryRestartIntent,
    RestartRequest, RuntimeAcceptanceReceipt, RuntimeAutomationInterpolation,
    RuntimeAutomationProjection, RuntimeAutomationSnapshot, RuntimeBlockDeadlinePressure,
    RuntimeClipFadeShape, RuntimeClipGainShape, RuntimeClipProcessingPipelineSnapshot,
    RuntimeClipProcessingReadiness, RuntimeClipProcessingRegistration,
    RuntimeClipProcessingSnapshot, RuntimeClipProcessingStage, RuntimeClipRenderInputStage,
    RuntimeClipRenderRequest, RuntimeClipRenderResult, RuntimeConfigRequest,
    RuntimeControlSnapshot, RuntimeControllerExpressionMidi2Posture,
    RuntimeControllerExpressionMpePosture, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceClass,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeDeferredServiceReceipt, RuntimeDiagnosticsSnapshot,
    RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventSink, RuntimeExecutionPhase, RuntimeExecutionTopologySummary,
    RuntimeInterruptionClass, RuntimeLifecycleApi, RuntimeMarkerAnalysisReadiness,
    RuntimeMarkerAnalysisSnapshot, RuntimeMediaAnalysisDescriptorState,
    RuntimeMediaAnalysisFamilyState, RuntimeMediaAssetRegistration, RuntimeMediaAssetSnapshot,
    RuntimeMediaAssetState, RuntimeMediaCharacterDescriptor, RuntimeMediaIndexingState,
    RuntimeMediaLibraryAssetDescriptor, RuntimeMediaLibraryServiceSnapshot,
    RuntimeMediaLoudnessDescriptor, RuntimeMediaPipelineSnapshot, RuntimeMediaPreviewState,
    RuntimeMediaServiceSnapshot, RuntimeMeterSourceRole, RuntimeMeterSourceSnapshot,
    RuntimeMeteringSnapshot, RuntimeObservationApi, RuntimeOfflineFreezeArtifactResult,
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
    RuntimePluginEventSnapshot, RuntimePluginExecutionChainSummary,
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
    SafeModeRequest, SandboxOperationFailureStage, ScanHandle, ScheduleProjection, StopReason,
    SubscriptionHandle, TransportAttachIntent, TransportProjection, TransportSessionProvenance,
    TransportSessionState, WatchdogRestartRecord,
};
use offline_render_delivery::{materialize_offline_render_delivery, offline_render_manifest};
use offline_render_maintenance::{
    purge_offline_render_directory, purge_offline_render_file, refresh_offline_render_result,
};
use runtime_audio_file_io::write_audio_buffer_wav;
use runtime_automation_state::{
    graph_parameter_target_from_runtime_target, graph_stage_parameter_sort_key,
    RuntimeAutomationBatchMetrics, RuntimeAutomationState,
};
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
    RuntimeMeteringStateModel,
};
use runtime_offline_render_session::RuntimeOfflineRenderExecutionSession;
use runtime_plugin_event_state::RuntimePluginEventState;
use runtime_plugin_lifecycle::{
    runtime_plugin_boundary_counts, runtime_plugin_stage_assignment,
    RuntimePluginLifecycleStateModel,
};
use runtime_plugin_recording::{
    plugin_format_sort_key, runtime_plugin_capability_coverage, runtime_plugin_format_coverage,
};
use runtime_recording_capture::RuntimeRecordingCaptureStateModel;
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
use signal_plugin::{
    AutomationContinuityReport, BlockSequenceContinuityReport, CompletionState,
    EventPacketContinuityReport, EventPacketSummary, ParameterAutomationSummary, PluginFeature,
    PluginFormat,
};
use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};

const PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES: u32 = 64;
const OFFLINE_RENDER_PROGRESS_CHECKPOINT_TARGET_COUNT: usize = 6;
const BLOCK_DEADLINE_ELEVATED_UTILIZATION_PERCENT: f32 = 75.0;
const BLOCK_DEADLINE_CRITICAL_UTILIZATION_PERCENT: f32 = 95.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub sample_rate: SampleRate,
    pub graph: GraphConfig,
    pub profile: RuntimeProfile,
}

impl RuntimeConfig {
    pub fn local(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Local,
        }
    }

    pub fn server(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Server,
        }
    }
}

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
    plugin_events: RuntimePluginEventState,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeSupervisionPolicy {
    safe_mode_restart_threshold: u32,
    safe_mode_xrun_threshold: u64,
}

impl Default for RuntimeSupervisionPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
            safe_mode_xrun_threshold: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeSupervisionState {
    policy: RuntimeSupervisionPolicy,
    watchdog_restart_count: u32,
    xrun_overload_active: bool,
    last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    last_sandbox_id: Option<String>,
    last_processing_epoch: Option<u64>,
}

impl RuntimeSupervisionState {
    fn snapshot(&self, safe_mode_enabled: bool) -> RuntimeSupervisionSnapshot {
        RuntimeSupervisionSnapshot {
            watchdog_restart_count: self.watchdog_restart_count,
            safe_mode_enabled,
            xrun_overload_active: self.xrun_overload_active,
            last_watchdog_trigger: self.last_watchdog_trigger,
            last_sandbox_id: self.last_sandbox_id.clone(),
            last_processing_epoch: self.last_processing_epoch,
        }
    }

    fn record_watchdog_restart(&mut self, record: WatchdogRestartRecord) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        self.last_watchdog_trigger = Some(record.trigger);
        self.last_sandbox_id = Some(record.sandbox_id);
        self.last_processing_epoch = Some(record.processing_epoch);
        self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold
    }

    fn record_xrun_overload(&mut self, processing_epoch: Option<u64>, xruns: u64) -> bool {
        if let Some(processing_epoch) = processing_epoch {
            self.last_processing_epoch = Some(processing_epoch);
        }
        if xruns >= self.policy.safe_mode_xrun_threshold {
            self.xrun_overload_active = true;
        }
        self.xrun_overload_active
    }

    fn clear_xrun_overload_recovery(&mut self) {
        self.xrun_overload_active = false;
    }
}

impl Default for RuntimeSupervisionState {
    fn default() -> Self {
        Self {
            policy: RuntimeSupervisionPolicy::default(),
            watchdog_restart_count: 0,
            xrun_overload_active: false,
            last_watchdog_trigger: None,
            last_sandbox_id: None,
            last_processing_epoch: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RuntimePluginDiscoveryStateModel {
    scan_count: usize,
    format_filtered_scan_count: usize,
    next_scan_handle: u64,
    last_scan: Option<RuntimePluginScanReceipt>,
    discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    platform_coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
}

impl RuntimePluginDiscoveryStateModel {
    fn record_platform_coverage(
        &mut self,
        coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
    ) {
        self.platform_coverage = coverage;
    }

    fn record_scan(&mut self, request: &PluginScanRequest) -> ScanHandle {
        self.next_scan_handle = self.next_scan_handle.saturating_add(1);
        self.scan_count = self.scan_count.saturating_add(1);
        if !request.formats.is_empty() {
            self.format_filtered_scan_count = self.format_filtered_scan_count.saturating_add(1);
        }
        let scan_handle = ScanHandle(self.next_scan_handle);
        self.last_scan = Some(RuntimePluginScanReceipt {
            scan_handle,
            roots: request.roots.clone(),
            formats: request.formats.clone(),
            targeted_format_count: request.formats.len(),
            discovered_type_count: 0,
            discovered_format_count: 0,
            format_coverage: Vec::new(),
            parity_coverage: Vec::new(),
            capability_coverage: RuntimePluginCapabilityCoverageSummary {
                summary: "formats=0 multi_format=false types=0".into(),
                ..RuntimePluginCapabilityCoverageSummary::default()
            },
            summary: format!(
                "scan={} roots={} formats={:?} discovered_types=0 discovered_formats=0",
                scan_handle.0,
                request.roots.len(),
                request.formats,
            ),
        });
        scan_handle
    }

    fn record_scan_results(
        &mut self,
        scan_handle: ScanHandle,
        discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
        parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    ) {
        let format_coverage = runtime_plugin_format_coverage(&discovered_types);
        let capability_coverage = runtime_plugin_capability_coverage(&discovered_types);
        if let Some(last_scan) = self.last_scan.as_mut() {
            if last_scan.scan_handle == scan_handle {
                last_scan.discovered_type_count = discovered_types.len();
                last_scan.discovered_format_count = format_coverage.len();
                last_scan.format_coverage = format_coverage;
                last_scan.parity_coverage = parity_coverage;
                last_scan.capability_coverage = capability_coverage;
                last_scan.summary = format!(
                    "scan={} roots={} formats={:?} discovered_types={} discovered_formats={}",
                    last_scan.scan_handle.0,
                    last_scan.roots.len(),
                    last_scan.formats,
                    last_scan.discovered_type_count,
                    last_scan.discovered_format_count,
                );
                self.discovered_types = discovered_types;
            }
        }
    }

    fn snapshot(
        &self,
        parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    ) -> RuntimePluginDiscoverySnapshot {
        let format_coverage = runtime_plugin_format_coverage(&self.discovered_types);
        let capability_coverage = runtime_plugin_capability_coverage(&self.discovered_types);
        RuntimePluginDiscoverySnapshot {
            scan_count: self.scan_count,
            format_filtered_scan_count: self.format_filtered_scan_count,
            discovered_type_count: self.discovered_types.len(),
            discovered_format_count: format_coverage.len(),
            last_scan: self.last_scan.clone(),
            format_coverage,
            parity_coverage,
            capability_coverage,
            discovered_types: self.discovered_types.clone(),
            summary: format!(
                "scans={} filtered_scans={} discovered_types={} discovered_formats={} last_scan={} capability={}",
                self.scan_count,
                self.format_filtered_scan_count,
                self.discovered_types.len(),
                {
                    let mut formats = self
                        .discovered_types
                        .iter()
                        .map(|record| record.format)
                        .collect::<Vec<_>>();
                    formats.sort_by_key(|format| plugin_format_sort_key(*format));
                    formats.dedup();
                    formats.len()
                },
                self.last_scan
                    .as_ref()
                    .map(|scan| scan.summary.as_str())
                    .unwrap_or("none"),
                runtime_plugin_capability_coverage(&self.discovered_types).summary,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeRecordingCapturePolicy {
    pressure_threshold_frames: u64,
}

impl Default for RuntimeRecordingCapturePolicy {
    fn default() -> Self {
        Self {
            pressure_threshold_frames: 16_384,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeMediaPipelinePolicy {
    cache_root: PathBuf,
}

impl Default for RuntimeMediaPipelinePolicy {
    fn default() -> Self {
        Self {
            cache_root: std::env::temp_dir().join("loophole-signal-media-cache"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeMediaAnalysisStateModel {
    descriptor_state: RuntimeMediaAnalysisDescriptorState,
    loudness: Option<RuntimeMediaLoudnessDescriptor>,
    character: Option<RuntimeMediaCharacterDescriptor>,
    last_error: Option<String>,
}

impl Default for RuntimeMediaAnalysisStateModel {
    fn default() -> Self {
        Self {
            descriptor_state: RuntimeMediaAnalysisDescriptorState::Missing,
            loudness: None,
            character: None,
            last_error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeMediaPipelineAsset {
    registration: RuntimeMediaAssetRegistration,
    state: RuntimeMediaAssetState,
    cache_path: Option<String>,
    cache_byte_size: Option<u64>,
    rebuild_count: u32,
    last_error: Option<String>,
    analysis: RuntimeMediaAnalysisStateModel,
}

fn transport_session_provenance(intent: TransportAttachIntent) -> TransportSessionProvenance {
    match intent {
        TransportAttachIntent::SteadyState => TransportSessionProvenance::SteadyOrigin,
        TransportAttachIntent::RecoveryOverlap => TransportSessionProvenance::RecoveryReplacement,
    }
}

fn offline_render_plugin_override_status<'a>(
    latest: Option<&'a RuntimePluginRenderedNodeState>,
    bound_sandbox_id: Option<&String>,
    sandboxes: &BTreeMap<String, RuntimePluginSandboxSnapshot>,
    last_processing_epoch: Option<u64>,
    last_block_sequence: Option<u64>,
) -> (
    RuntimeOfflinePluginOverrideState,
    Option<&'a RuntimePluginRenderedNodeState>,
) {
    let Some(latest) = latest else {
        return (RuntimeOfflinePluginOverrideState::NotAvailable, None);
    };
    let fresh = Some(latest.processing_epoch) == last_processing_epoch
        && Some(latest.block_sequence) == last_block_sequence
        && bound_sandbox_id.is_none_or(|sandbox_id| sandbox_id == &latest.sandbox_id)
        && bound_sandbox_id
            .and_then(|sandbox_id| sandboxes.get(sandbox_id))
            .is_none_or(|sandbox| sandbox.state == RuntimePluginLifecycleState::Ready);
    if fresh {
        (
            RuntimeOfflinePluginOverrideState::FreshLatestBlock,
            Some(latest),
        )
    } else {
        (RuntimeOfflinePluginOverrideState::StaleLatestBlock, None)
    }
}

fn runtime_meter_source_role(role: GraphNodeTopologyRole) -> RuntimeMeterSourceRole {
    match role {
        GraphNodeTopologyRole::Utility => RuntimeMeterSourceRole::Utility,
        GraphNodeTopologyRole::TrackLane => RuntimeMeterSourceRole::TrackLane,
        GraphNodeTopologyRole::Bus => RuntimeMeterSourceRole::Bus,
        GraphNodeTopologyRole::Send => RuntimeMeterSourceRole::Send,
        GraphNodeTopologyRole::Return => RuntimeMeterSourceRole::Return,
        GraphNodeTopologyRole::ConsoleNode => RuntimeMeterSourceRole::ConsoleNode,
    }
}

fn unique_string<'a>(values: impl Iterator<Item = &'a String>) -> Option<String> {
    let mut values = values.cloned().collect::<BTreeSet<_>>().into_iter();
    let first = values.next()?;
    if values.next().is_none() {
        Some(first)
    } else {
        None
    }
}

fn sanitize_asset_id(asset_id: &str) -> String {
    asset_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

impl core::fmt::Debug for SignalRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalRuntime")
            .field("config", &self.config)
            .field("readiness", &self.readiness)
            .field("safe_mode_enabled", &self.safe_mode_enabled)
            .field("anticipative_enabled", &self.anticipative_enabled)
            .field("active_output_device", &self.active_output_device)
            .field("applied_graph", &self.applied_graph)
            .field("applied_schedule", &self.applied_schedule)
            .field("applied_transport", &self.applied_transport)
            .field("latest_parameter_epoch", &self.latest_parameter_epoch)
            .field("projection_epoch", &self.projection_epoch)
            .field("control", &self.control)
            .field("timeline", &self.timeline)
            .field("automation", &self.automation)
            .field("engine", &self.engine)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        let mut runtime = Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            applied_graph: None,
            applied_schedule: None,
            applied_transport: None,
            applied_parameter_batch: None,
            prework_forecast_requested_mode: RuntimePreworkForecastMode::RuntimeRoleDefault,
            prework_forecast_mode: RuntimePreworkForecastMode::Disabled,
            prework_forecast_policy: None,
            prework_forecast_profile: None,
            prework_forecast_profile_source: None,
            latest_parameter_epoch: 0,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            timeline: RuntimeTimelineState::default(),
            automation: RuntimeAutomationState::default(),
            plugin_events: RuntimePluginEventState::default(),
            engine: RuntimeEngineState::default(),
            transport_concurrency: RuntimeTransportConcurrencyState::default(),
            plugin_discovery: RuntimePluginDiscoveryStateModel::default(),
            plugin_lifecycle: RuntimePluginLifecycleStateModel::default(),
            plugin_placement_policy: RuntimePluginPlacementPolicy::default(),
            recording_capture: RuntimeRecordingCaptureStateModel::default(),
            metering: RuntimeMeteringStateModel::default(),
            media_pipeline: RuntimeMediaPipelineStateModel::default(),
            tempo_map: RuntimeTempoMapStateModel::default(),
            warp_pipeline: RuntimeWarpPipelineStateModel::default(),
            clip_processing_pipeline: RuntimeClipProcessingPipelineStateModel::default(),
            diagnostics: RuntimeDiagnosticsSnapshot {
                cpu_load_percent: 0.0,
                xruns: 0,
                graph_latency_ms: 0.0,
                active_plugin_sandboxes: 0,
                backend_policy_tier: BackendPolicyTier::Tier0InHost,
                topology_compatible: false,
                topology_issue_count: 0,
                degraded_bound_plugin_sandboxes: 0,
                missing_bound_plugin_sandboxes: 0,
                last_output_peak: None,
                last_output_rms: None,
                momentary_loudness_lufs: None,
                short_term_loudness_lufs: None,
                integrated_loudness_lufs: None,
            },
            supervision: RuntimeSupervisionState::default(),
            last_deferred_service_receipt: RefCell::new(None),
            last_offline_render_session_snapshot: RefCell::new(None),
            last_offline_render_cancellation_receipt: RefCell::new(None),
            last_offline_render_purge_receipt: RefCell::new(None),
            offline_render_executions: HashMap::new(),
            next_subscription: 1,
            sinks: Vec::new(),
        };
        runtime.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RuntimeRoleDefault,
        );
        runtime.set_prework_forecast_mode_internal(RuntimePreworkForecastMode::Disabled);
        runtime.recompute_prework_service_policy_snapshot();
        runtime
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    fn reconcile_prework_service_state(&mut self, processing_epoch: Option<u64>) {
        let state = if !self.engine.snapshot.prework_cache_enabled
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            RuntimePreworkServiceState::Disabled
        } else if !self.control.running {
            RuntimePreworkServiceState::Paused
        } else if !self.engine.pending_prework_targets.is_empty()
            && (self.engine.snapshot.prework_service_plugin_gate_active
                || self.engine.snapshot.prework_service_transport_gate_active)
        {
            RuntimePreworkServiceState::Yielding
        } else if !self.engine.pending_prework_targets.is_empty() {
            RuntimePreworkServiceState::Pending
        } else {
            RuntimePreworkServiceState::Idle
        };
        self.engine
            .transition_prework_service_state(state, processing_epoch);
    }

    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.plugin_lifecycle.set_active_sandbox_count(count);
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::PluginSandboxChanged {
            active_sandboxes: self.diagnostics.active_plugin_sandboxes,
        });
    }

    pub fn apply_plugin_node_render_batch(
        &mut self,
        batch: PluginNodeRenderBatch,
    ) -> Result<(), RuntimeError> {
        self.engine.apply_plugin_node_render_batch(batch)
    }

    pub fn set_backend_policy_tier(&mut self, tier: BackendPolicyTier) {
        self.diagnostics.backend_policy_tier = tier;
    }

    pub fn set_cpu_load_percent(&mut self, cpu_load_percent: f32) {
        self.diagnostics.cpu_load_percent = cpu_load_percent.max(0.0);
    }

    pub fn set_graph_latency_ms(&mut self, graph_latency_ms: f32) {
        self.diagnostics.graph_latency_ms = graph_latency_ms.max(0.0);
    }

    pub fn projection_epoch(&self) -> u64 {
        self.projection_epoch
    }

    pub fn reset_block_timeline(&mut self) {
        self.timeline.reset();
    }

    pub fn reset_automation_tracking(&mut self) {
        self.automation.reset();
    }

    pub fn reset_plugin_event_tracking(&mut self) {
        self.plugin_events.reset();
    }

    pub fn process_engine_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before processing engine blocks",
            ));
        }
        let block_start = Instant::now();
        let transport = self.applied_transport;
        let context = self.build_engine_execution_context(processing_epoch, block_sequence);
        let (parameter_batch, automation_metrics) =
            self.current_graph_parameter_batch(&context, buffer.frames().0);
        let pending_transition = self
            .timeline
            .consume_pending_transport_transition(block_sequence);
        let mut result = self
            .engine
            .process_block(context, transport, buffer, parameter_batch)?;
        self.apply_engine_transport_update(
            processing_epoch,
            block_sequence,
            pending_transition,
            &mut result,
        );
        self.finalize_engine_block_result(
            processing_epoch,
            block_sequence,
            automation_metrics,
            block_start,
            &mut result,
        )?;
        Ok(result)
    }
}

impl RuntimeLifecycleApi for SignalRuntime {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError> {
        if request.client_version.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "client_version must not be empty",
            ));
        }
        if matches!(request.max_sample_rate_hint, Some(0)) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "max_sample_rate_hint must be positive when provided",
            ));
        }

        self.control.handshaken = true;
        self.control.handshake_count = self.control.handshake_count.saturating_add(1);
        self.control.last_client_version = Some(request.client_version.clone());

        Ok(HandshakeResponse {
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            supports_anticipative: true,
            supports_dynamic_reconfigure: true,
            max_channels: 2048,
            max_sample_rate: request.max_sample_rate_hint.unwrap_or(192_000),
        })
    }

    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError> {
        self.configure_runtime_state(request)
    }

    fn start(&mut self) -> Result<(), RuntimeError> {
        self.start_runtime_state()
    }

    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        self.stop_runtime_state(reason)
    }

    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.reconfigure.is_none() {
            self.require_configured()?;
        }
        if self.control.running {
            self.stop(StopReason::DeviceReconfigure)?;
        }
        if let Some(config) = request.reconfigure {
            self.configure(config)?;
        }
        self.control.restart_count = self.control.restart_count.saturating_add(1);
        self.start()
    }

    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError> {
        self.set_safe_mode_state(request)
    }
}

impl RuntimeProjectionApi for SignalRuntime {
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.engine.set_prework_service_pressure(pressure);
        self.refresh_prework_service_policy_and_state(None);
        Ok(())
    }

    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_mode(self, mode)
    }

    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_profile(self, selection)
    }

    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_policy(self, policy)
    }

    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        SignalRuntime::service_prework_lane(self, processing_epoch, cycles)
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_plugin_backed_node_bindings_projection(projection)
    }

    fn apply_plugin_placement_policy(
        &mut self,
        policy: RuntimePluginPlacementPolicy,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.plugin_placement_policy = policy;
        Ok(())
    }

    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_graph_contract_projection_state(projection)
    }

    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_graph_projection_state(projection)
    }

    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_schedule_projection_state(projection)
    }

    fn apply_automation_projection(
        &mut self,
        projection: RuntimeAutomationProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        Self::validate_automation_projection_request(&projection)?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.automation.apply_projection(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_tempo_map_projection(
        &mut self,
        projection: RuntimeTempoMapProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        Self::validate_tempo_map_projection_request(&projection)?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.tempo_map.apply_projection(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        self.apply_transport_projection_state(projection)
    }

    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError> {
        self.apply_parameter_batch_state(batch)
    }

    fn apply_hardware_config(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.apply_hardware_config_state(request)
    }
}

fn runtime_plugin_chain_id(
    track_lane_id: Option<&str>,
    bus_group_id: Option<&str>,
    console_group_id: Option<&str>,
    send_return_id: Option<&str>,
) -> String {
    track_lane_id
        .map(str::to_string)
        .or_else(|| bus_group_id.map(str::to_string))
        .or_else(|| console_group_id.map(str::to_string))
        .or_else(|| send_return_id.map(str::to_string))
        .unwrap_or_else(|| "global".into())
}

fn runtime_plugin_discovered_type_for_recall<'a>(
    plugin_type_id: Option<&str>,
    discovered_types: &'a [RuntimePluginDiscoveredTypeRecord],
) -> Option<&'a RuntimePluginDiscoveredTypeRecord> {
    let plugin_type_id = plugin_type_id?;
    discovered_types
        .iter()
        .find(|record| record.plugin_type_id == plugin_type_id)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RuntimePluginCompensationObservation {
    state: RuntimePluginCompensationState,
    realized_latency_samples: Option<u32>,
    tail_samples: Option<u32>,
}

#[cfg(test)]
mod tests {
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
        GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
        GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        LingeringCleanupTrigger, ParameterBatch, ParameterEvent, PluginBackedNodeBinding,
        PluginBackedNodeBindingProjection, PluginFaultKind, PluginNodeRender,
        PluginNodeRenderBatch, PluginSandboxLifecycleStage, PluginSandboxSpec,
        PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent, RestartRequest,
        RuntimeAuditionSinkAuthority, RuntimeAuditionSinkClass, RuntimeAutomationInterpolation,
        RuntimeAutomationLaneProjection, RuntimeAutomationPointProjection,
        RuntimeAutomationProjection, RuntimeAutomationResolution,
        RuntimeAutomationTargetProjection, RuntimeBlockDeadlinePressure, RuntimeClipFadeEnvelope,
        RuntimeClipFadeShape, RuntimeClipGainEnvelope, RuntimeClipGainShape,
        RuntimeClipProcessingReadiness, RuntimeClipProcessingRegistration,
        RuntimeClipProcessingStage, RuntimeClipRenderInputStage, RuntimeClipRenderRequest,
        RuntimeConfigRequest, RuntimeControllerExpressionMidi2Posture,
        RuntimeControllerExpressionMpePosture, RuntimeDeferredServiceBackpressureSource,
        RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceClass,
        RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
        RuntimeDeferredServiceReason, RuntimeError, RuntimeErrorKind, RuntimeEvent,
        RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionPhase, RuntimeFaultCause,
        RuntimeFaultStatusSnapshot, RuntimeInterruptionClass, RuntimeLifecycleApi,
        RuntimeLowLatencyDevicePolicyClass, RuntimeLowLatencyDevicePolicyOutcome,
        RuntimeMarkerAnalysisReadiness, RuntimeMediaAssetRegistration, RuntimeMediaAssetState,
        RuntimeMediaAuditionContinuityOutcome, RuntimeMediaAuditionOrchestrationAuthority,
        RuntimeMediaAuditionOrchestrationPosture, RuntimeMediaPreviewState, RuntimeMeterSourceRole,
        RuntimeMeterSourceSnapshot, RuntimeObservationApi, RuntimeObservationReport,
        RuntimeOfflineFreezeArtifactRequest, RuntimeOfflinePluginDelegatedExecutionMerge,
        RuntimeOfflinePluginDelegatedExecutionOutcome,
        RuntimeOfflinePluginDelegatedExecutionReceipt,
        RuntimeOfflinePluginDelegatedExecutionStageReceipt,
        RuntimeOfflinePluginDelegatedExecutionStatus,
        RuntimeOfflinePluginDelegatedFreezeArtifactOutput, RuntimeOfflinePluginDelegatedStemOutput,
        RuntimeOfflinePluginExecutionBoundary, RuntimeOfflinePluginExecutionOwner,
        RuntimeOfflinePluginExecutionStageBoundary, RuntimeOfflinePluginOverrideState,
        RuntimeOfflineRenderArtifactKind, RuntimeOfflineRenderCheckpointStage,
        RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderExecutionState,
        RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderRequest,
        RuntimeOfflineRenderStemTarget, RuntimeOfflineRenderTargetKind,
        RuntimePluginBusCapableFxClass, RuntimePluginCompensationState,
        RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
        RuntimePluginIsolationOutcome, RuntimePluginLifecycleState, RuntimePluginParityBand,
        RuntimePluginPlacementPolicy, RuntimePluginPlacementRule,
        RuntimePluginPlacementRuleMatcher, RuntimePluginRecallHandoffSelection,
        RuntimePluginRecallHandoffStageId, RuntimePluginRecallPayload, RuntimePluginRecallState,
        RuntimePreviewBrowserQueueClass, RuntimePreviewBrowserQueueOutcome,
        RuntimePreviewBrowserQueuePosture, RuntimePreviewOutputRoutingPosture,
        RuntimePreviewTransformFallbackKind, RuntimePreviewTransformReadiness,
        RuntimePreviewTransformSchedulingAuthority, RuntimePreviewTransformSchedulingOutcome,
        RuntimePreviewTransformSchedulingPosture, RuntimePreviewTransformServiceClass,
        RuntimePreworkBacklogClass, RuntimePreworkCacheState, RuntimePreworkForecastMode,
        RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
        RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
        RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason,
        RuntimePreworkRetirementReason, RuntimePreworkServicePressure,
        RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
        RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
        RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
        RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState, RuntimeRecoveryState,
        RuntimeSchedulerState, RuntimeSchedulerTopologyIssue,
        RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputTargetKind,
        RuntimeStretchEngineClass, RuntimeStretchFallbackKind, RuntimeStretchReadiness,
        RuntimeSupervisorReport, RuntimeTempoAssistHintSource, RuntimeTempoAssistPosture,
        RuntimeTempoMapInterpolation, RuntimeTempoMapProjection, RuntimeTempoSource,
        RuntimeTransformArtifactReadiness, RuntimeTransformArtifactReuseState,
        RuntimeTransformCachePlacementAuthority, RuntimeTransformCachePlacementOutcome,
        RuntimeTransformCachePlacementPosture, RuntimeTransformPersistencePosture,
        RuntimeTransformRetentionAuthority, RuntimeTransformRetentionOutcome,
        RuntimeTransformRetentionPolicyClass, RuntimeWarpClipRegistration, RuntimeWarpMode,
        RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest,
        SandboxOperationFailureStage, ScheduleProjection, StopReason, TransportAttachIntent,
        TransportProjection, TransportSessionProvenance, WatchdogRestartRecord,
    };
    use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};
    use signal_graph::{
        synthetic_stereo_block, ExecutableGraph, GraphExecutionLane, GraphNodeBufferContract,
        GraphNodeBusEndpoint, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeSpec,
        GraphNodeTopologyMetadata, GraphNodeTopologyRole, GraphStageSpec,
    };
    use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
    use signal_plugin::{
        CompletionState, EventPacketSummary, ParameterAutomationSummary, PluginFeature,
        PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
        PluginStateContract,
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
            &format!("lease-{sandbox_id}"),
            &format!("region-{sandbox_id}"),
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
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(32),
                    ),
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

    fn prepare_offline_render_engine_runtime_without_cached_plugin_render(
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
            .apply_parameter_batch(
                runtime.forecast_parameter_batch_for_block(block_sequence, &policy),
            )
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

    #[test]
    fn runtime_starts_and_reports_ready() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
        assert_eq!(runtime.config().profile, RuntimeProfile::Local);
    }

    #[test]
    fn configure_updates_effective_config() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(96_000, 256))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
    }

    #[test]
    fn configure_resets_runtime_block_timeline() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 0);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 0);
    }

    #[test]
    fn runtime_timeline_tracks_sequences_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let first = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first);
        let second = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second);
        let third = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 2, "lease-b", third);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 3);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 2);
        assert_eq!(timeline.block_sequence_continuity.lease_rollovers, 1);
        assert_eq!(
            timeline.block_sequence_continuity.first_block_sequence(),
            Some(0)
        );
        assert_eq!(
            timeline.block_sequence_continuity.last_block_sequence(),
            Some(2)
        );
    }

    #[test]
    fn configure_resets_runtime_automation_tracking() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 0);
        assert_eq!(automation.segment_count, 0);
        assert_eq!(automation.first_epoch, None);
    }

    #[test]
    fn runtime_automation_tracking_rolls_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 4096);
        assert_eq!(automation.value_events, 4);
        assert_eq!(automation.segment_count, 2);
        assert_eq!(automation.segment_epochs, vec![1, 2]);
        assert_eq!(automation.lease_rollovers, 1);
        assert_eq!(automation.first_epoch, Some(1));
        assert_eq!(automation.last_epoch, Some(2));
    }

    #[test]
    fn runtime_plugin_event_tracking_rolls_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_event_summary(
            1,
            "lease-a",
            7,
            96,
            EventPacketSummary {
                total_events: 6,
                parameter_value_events: 1,
                parameter_modulation_events: 1,
                parameter_gesture_events: 1,
                note_events: 1,
                note_expression_events: 1,
                note_expression_pressure_events: 1,
                note_expression_timbre_events: 0,
                note_expression_tuning_events: 0,
                midi_events: 1,
            },
        );
        runtime.record_plugin_event_summary(
            2,
            "lease-b",
            8,
            64,
            EventPacketSummary {
                total_events: 5,
                parameter_value_events: 1,
                parameter_modulation_events: 0,
                parameter_gesture_events: 1,
                note_events: 1,
                note_expression_events: 1,
                note_expression_pressure_events: 0,
                note_expression_timbre_events: 0,
                note_expression_tuning_events: 1,
                midi_events: 1,
            },
        );

        let snapshot = runtime.get_plugin_event_snapshot();
        assert_eq!(snapshot.last_processing_epoch, Some(2));
        assert_eq!(snapshot.last_block_sequence, Some(8));
        assert_eq!(snapshot.last_generated_event_bytes, 64);
        assert_eq!(snapshot.last_batch_total_events, 5);
        assert_eq!(snapshot.last_batch_note_expression_events, 1);
        assert_eq!(snapshot.last_batch_note_expression_pressure_events, 0);
        assert_eq!(snapshot.last_batch_note_expression_timbre_events, 0);
        assert_eq!(snapshot.last_batch_note_expression_tuning_events, 1);
        assert_eq!(snapshot.total_events, 11);
        assert_eq!(snapshot.parameter_value_events, 2);
        assert_eq!(snapshot.parameter_modulation_events, 1);
        assert_eq!(snapshot.parameter_gesture_events, 2);
        assert_eq!(snapshot.note_events, 2);
        assert_eq!(snapshot.note_expression_events, 2);
        assert_eq!(snapshot.note_expression_pressure_events, 1);
        assert_eq!(snapshot.note_expression_timbre_events, 0);
        assert_eq!(snapshot.note_expression_tuning_events, 1);
        assert_eq!(snapshot.midi_events, 2);
        assert_eq!(
            snapshot.mpe_posture,
            RuntimeControllerExpressionMpePosture::Guarded
        );
        assert_eq!(
            snapshot.midi2_posture,
            RuntimeControllerExpressionMidi2Posture::Guarded
        );
        assert_eq!(snapshot.first_epoch, Some(1));
        assert_eq!(snapshot.last_epoch, Some(2));
        assert_eq!(snapshot.segment_count, 2);
        assert_eq!(snapshot.segment_epochs, vec![1, 2]);
        assert_eq!(snapshot.lease_rollovers, 1);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(observation
            .render_json()
            .contains("\"plugin_events\":{\"last_processing_epoch\":2"));
        assert!(observation
            .render_compact()
            .contains("plugin_events_total=11/2/1/2/2/2/2"));
        assert!(observation
            .render_json()
            .contains("\"note_expression_tuning_events\":1"));
        assert!(observation
            .render_json()
            .contains("\"mpe_posture\":\"Guarded\""));
        assert!(observation
            .render_json()
            .contains("\"midi2_posture\":\"Guarded\""));
    }

    #[test]
    fn runtime_plugin_event_tracking_resets_on_reconfigure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime.record_plugin_event_summary(
            1,
            "lease-a",
            4,
            80,
            EventPacketSummary {
                total_events: 4,
                parameter_value_events: 1,
                parameter_modulation_events: 1,
                parameter_gesture_events: 0,
                note_events: 1,
                note_expression_events: 1,
                note_expression_pressure_events: 1,
                note_expression_timbre_events: 0,
                note_expression_tuning_events: 0,
                midi_events: 0,
            },
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let snapshot = runtime.get_plugin_event_snapshot();
        assert_eq!(snapshot.total_events, 0);
        assert_eq!(snapshot.segment_count, 0);
        assert_eq!(snapshot.first_epoch, None);
        assert_eq!(snapshot.last_processing_epoch, None);
    }

    #[test]
    fn automation_projection_requires_explicit_targets_and_positive_linear_resolution() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));
        handshake_and_configure(&mut runtime);

        let error = runtime
            .apply_automation_projection(RuntimeAutomationProjection {
                lane_count: 1,
                point_count: 1,
                lanes: vec![RuntimeAutomationLaneProjection {
                    automation_lane_id: "lane:invalid".into(),
                    target: RuntimeAutomationTargetProjection {
                        node_id: String::new(),
                        parameter_id: "gain".into(),
                    },
                    base_normalized_value: 0.0,
                    interpolation: RuntimeAutomationInterpolation::Linear,
                    resolution: RuntimeAutomationResolution {
                        ramp_step_samples: 0,
                        max_sub_blocks: 0,
                    },
                    point_count: 1,
                    points: vec![RuntimeAutomationPointProjection {
                        time_samples: 0,
                        normalized_value: 0.0,
                    }],
                }],
            })
            .expect_err("invalid automation projection should be rejected");

        assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
    }

    #[test]
    fn tempo_map_projection_requires_bounded_non_overlapping_segments() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));
        handshake_and_configure(&mut runtime);

        let error = runtime
            .apply_tempo_map_projection(RuntimeTempoMapProjection {
                segment_count: 2,
                segments: vec![
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:intro".into(),
                        start_samples: 0,
                        end_samples: None,
                        start_tempo_bpm: 120.0,
                        end_tempo_bpm: None,
                        interpolation: RuntimeTempoMapInterpolation::Hold,
                    },
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:lift".into(),
                        start_samples: 4_800,
                        end_samples: Some(9_600),
                        start_tempo_bpm: 132.0,
                        end_tempo_bpm: None,
                        interpolation: RuntimeTempoMapInterpolation::Hold,
                    },
                ],
            })
            .expect_err("invalid tempo map projection should be rejected");

        assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
        assert!(error.message.contains("open-ended tempo map segments"));
    }

    #[test]
    fn runtime_linear_automation_projection_drives_multi_block_gain_playback() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 8));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 8))
            .unwrap();
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:automation-linear".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "gain".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.0 }],
                }],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .apply_automation_projection(RuntimeAutomationProjection {
                lane_count: 1,
                point_count: 3,
                lanes: vec![RuntimeAutomationLaneProjection {
                    automation_lane_id: "lane:gain:linear".into(),
                    target: RuntimeAutomationTargetProjection {
                        node_id: "gain".into(),
                        parameter_id: "gain".into(),
                    },
                    base_normalized_value: 0.0,
                    interpolation: RuntimeAutomationInterpolation::Linear,
                    resolution: RuntimeAutomationResolution {
                        ramp_step_samples: 2,
                        max_sub_blocks: 8,
                    },
                    point_count: 3,
                    points: vec![
                        RuntimeAutomationPointProjection {
                            time_samples: 0,
                            normalized_value: 0.0,
                        },
                        RuntimeAutomationPointProjection {
                            time_samples: 8,
                            normalized_value: 1.0,
                        },
                        RuntimeAutomationPointProjection {
                            time_samples: 16,
                            normalized_value: 0.0,
                        },
                    ],
                }],
            })
            .unwrap();

        let first = runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![1.0; 8],
                ),
            )
            .expect("first automation block should process");
        let second = runtime
            .process_engine_block(
                2,
                2,
                AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![1.0; 8],
                ),
            )
            .expect("second automation block should process");

        assert_eq!(
            first.output.samples(),
            &[
                0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 0.25, 0.25, 0.5, 0.5, 0.5, 0.5, 0.75, 0.75, 0.75,
                0.75,
            ]
        );
        assert_eq!(
            second.output.samples(),
            &[
                1.0, 1.0, 1.0, 1.0, 0.75, 0.75, 0.75, 0.75, 0.5, 0.5, 0.5, 0.5, 0.25, 0.25, 0.25,
                0.25,
            ]
        );
        assert_eq!(first.snapshot.parameter_event_count, 4);
        assert_eq!(first.snapshot.parameter_sub_block_count, 4);
        assert_eq!(second.snapshot.parameter_event_count, 4);
        assert_eq!(second.snapshot.parameter_sub_block_count, 4);

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.lane_count, 1);
        assert_eq!(automation.point_count, 3);
        assert_eq!(automation.projected_segment_count, 2);
        assert_eq!(automation.mapped_lane_count, 1);
        assert_eq!(automation.unmapped_lane_count, 0);
        assert_eq!(automation.hold_lane_count, 0);
        assert_eq!(automation.linear_lane_count, 1);
        assert_eq!(automation.last_batch_event_count, 4);
        assert_eq!(automation.last_batch_sub_block_count, 4);
        assert_eq!(automation.last_batch_strategy_max_sub_blocks, 8);
        assert_eq!(automation.last_batch_min_ramp_step_samples, Some(2));
        assert_eq!(automation.last_batch_max_sample_offset, Some(6));
        assert_eq!(automation.last_block_sequence, Some(2));
        assert_eq!(automation.last_timeline_position_samples, Some(8));
        assert_eq!(automation.transport_playing, Some(true));

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(observation
            .render_compact()
            .contains("automation_projection=1/3/2"));
        assert!(observation
            .render_compact()
            .contains("automation_shapes=0/1"));
        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(supervisor
            .render_multiline()
            .contains("automation_linear_lanes=1"));
        assert!(supervisor
            .render_multiline()
            .contains("automation_last_batch_min_ramp_step_samples=Some(2)"));
        assert!(supervisor
            .render_json()
            .contains("\"automation\":{\"lane_count\":1"));
        assert!(supervisor
            .render_json()
            .contains("\"last_batch_min_ramp_step_samples\":2"));
    }

    #[test]
    fn runtime_hold_automation_projection_drives_plugin_backed_threshold_fixture() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 4));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 4))
            .unwrap();
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:automation-plugin".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 1.0 }],
                }],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .apply_automation_projection(RuntimeAutomationProjection {
                lane_count: 1,
                point_count: 1,
                lanes: vec![RuntimeAutomationLaneProjection {
                    automation_lane_id: "lane:plugin:threshold".into(),
                    target: RuntimeAutomationTargetProjection {
                        node_id: "plugin".into(),
                        parameter_id: "threshold".into(),
                    },
                    base_normalized_value: 1.0,
                    interpolation: RuntimeAutomationInterpolation::Hold,
                    resolution: RuntimeAutomationResolution::default(),
                    point_count: 1,
                    points: vec![RuntimeAutomationPointProjection {
                        time_samples: 2,
                        normalized_value: 0.5,
                    }],
                }],
            })
            .unwrap();

        let result = runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![0.7, 0.7, 0.7, 0.7],
                ),
            )
            .expect("plugin-backed automation block should process");

        assert_eq!(
            result.output.samples(),
            &[0.7, 0.7, 0.7, 0.7, 0.5, 0.5, 0.5, 0.5]
        );
        assert_eq!(result.snapshot.plugin_backed_node_count, 1);
        assert_eq!(result.snapshot.parameter_event_count, 2);
        assert_eq!(result.snapshot.parameter_sub_block_count, 2);

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.hold_lane_count, 1);
        assert_eq!(automation.linear_lane_count, 0);
        assert_eq!(automation.mapped_lane_count, 1);
        assert_eq!(automation.projected_segment_count, 0);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(observation
            .render_compact()
            .contains("automation_shapes=1/0"));
        assert!(
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default())
                .render_json()
                .contains("\"linear_lane_count\":0")
        );
    }

    #[test]
    fn handshake_requires_client_version() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .handshake(HandshakeRequest {
                client_version: String::new(),
                anticipative_preferred: true,
                max_sample_rate_hint: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn schedule_projection_advances_epoch() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let receipt = runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-1".into(),
                stream_count: 2,
            })
            .unwrap();

        assert_eq!(receipt.accepted_epoch, 1);
        assert!(receipt.applied_at_block_boundary);
    }

    #[test]
    fn schedule_projection_refreshes_running_prework_window_with_widened_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set widened refresh policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:schedule-refresh");
        runtime.start().expect("start runtime");

        let before = runtime.get_engine_block_snapshot();
        assert_eq!(before.prework_cache_queue_depth, 2);
        assert!(before.prework_pending_target_count > 0);

        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:refresh-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert!(snapshot.scheduler_topology.compatible);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(snapshot.last_prework_service_cycle_count, 3);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
        assert!(snapshot.prework_cache_queue_depth > before.prework_cache_queue_depth);
        assert_eq!(snapshot.prework_pending_target_count, 0);
    }

    #[test]
    fn runtime_scheduler_topology_summary_validates_track_bus_console_groups() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology",
            &["track:drums", "track:bass"],
            false,
        );

        let missing_schedule = runtime.get_engine_block_snapshot();
        let scheduler_topology = runtime.get_scheduler_topology_summary();
        assert_eq!(missing_schedule.scheduler_topology.track_lane_node_count, 2);
        assert_eq!(scheduler_topology.track_lane_node_count, 2);
        assert_eq!(
            missing_schedule.scheduler_topology.track_lane_group_count,
            2
        );
        assert_eq!(missing_schedule.scheduler_topology.bus_node_count, 1);
        assert_eq!(missing_schedule.scheduler_topology.bus_group_count, 2);
        assert_eq!(missing_schedule.scheduler_topology.console_node_count, 1);
        assert_eq!(missing_schedule.scheduler_topology.console_group_count, 1);
        assert_eq!(
            missing_schedule.scheduler_topology.schedule_stream_count,
            None
        );
        assert!(!missing_schedule.scheduler_topology.compatible);
        assert!(
            missing_schedule
                .scheduler_topology
                .requires_host_reinterpretation
        );
        assert!(matches!(
            missing_schedule.scheduler_topology.issues.as_slice(),
            [
                RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
                    required_streams: 2
                }
            ]
        ));

        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-topology".into(),
                stream_count: 2,
            })
            .expect("apply matching schedule projection");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        let result = runtime
            .process_engine_block(1, 1, block)
            .expect("process topology-aware block");
        let execution_topology = runtime.get_execution_topology_summary();

        assert_eq!(result.snapshot.lane_order.len(), 2);
        assert_eq!(
            result.snapshot.lane_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(
            result.snapshot.dispatch_order.last().copied(),
            Some(signal_graph::GraphExecutionLane::Realtime)
        );
        assert!(result.snapshot.scheduler_topology.compatible);
        assert!(
            !result
                .snapshot
                .scheduler_topology
                .requires_host_reinterpretation
        );
        assert!(result.snapshot.scheduler_topology.issues.is_empty());
        assert_eq!(
            result.snapshot.scheduler_topology.schedule_stream_count,
            Some(2)
        );
        assert_eq!(execution_topology.node_count, result.snapshot.node_count);
        assert_eq!(execution_topology.track_lane_group_count, 2);
        assert_eq!(execution_topology.bus_group_count, 2);
        assert_eq!(execution_topology.console_group_count, 1);
    }

    #[test]
    fn runtime_scheduler_topology_summary_flags_insufficient_schedule_streams() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-insufficient",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-too-small".into(),
                stream_count: 1,
            })
            .expect("apply undersized schedule projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert!(!snapshot.scheduler_topology.compatible);
        assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
        assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
            matches!(
                issue,
                RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
                    required_streams: 2,
                    actual_streams: 1
                }
            )
        }));
    }

    #[test]
    fn runtime_scheduler_topology_summary_flags_missing_track_lane_metadata() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-missing-metadata",
            &["track:drums"],
            true,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-metadata".into(),
                stream_count: 2,
            })
            .expect("apply schedule projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert!(!snapshot.scheduler_topology.compatible);
        assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
        assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
            matches!(
                issue,
                RuntimeSchedulerTopologyIssue::MissingTrackLaneIds { node_count: 1 }
            )
        }));
    }

    #[test]
    fn runtime_scheduler_topology_projects_into_runtime_reports() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-report",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-topology-report".into(),
                stream_count: 2,
            })
            .expect("apply matching schedule projection");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        runtime
            .process_engine_block(1, 1, block)
            .expect("process topology report block");

        let metering = runtime.get_metering_snapshot();
        assert!(metering.meter_count > 0);
        assert!(metering.main_output_peak_level.is_some());
        assert!(metering.main_output_rms_level.is_some());
        assert!(metering
            .meters
            .iter()
            .any(|meter| meter.bus_id == "main:out"));
        assert_eq!(metering.track_lanes.len(), 2);
        assert_eq!(metering.bus_groups.len(), 2);
        assert_eq!(metering.console_groups.len(), 1);
        assert!(metering.send_returns.is_empty());
        assert!(metering
            .track_lanes
            .iter()
            .any(|track_lane| track_lane.track_lane_id == "track:drums"));
        assert!(metering
            .bus_groups
            .iter()
            .any(|bus_group| bus_group.bus_group_id == "mix:master"));
        assert!(metering.console_groups.iter().any(|console_group| {
            console_group.console_group_id == "console:main"
                && console_group.aggregate.meter_count > 0
        }));

        let diagnostics = runtime.get_diagnostics_snapshot();
        assert!(diagnostics.topology_compatible);
        assert_eq!(
            diagnostics.last_output_peak,
            metering.main_output_peak_level
        );
        assert_eq!(diagnostics.last_output_rms, metering.main_output_rms_level);
        assert_eq!(
            diagnostics.momentary_loudness_lufs,
            metering.momentary_loudness_lufs
        );
        assert_eq!(
            diagnostics.integrated_loudness_lufs,
            metering.integrated_loudness_lufs
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.execution_topology_summary.node_count, 5);
        assert_eq!(
            observation.execution_topology_summary.track_lane_node_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
        assert_eq!(observation.execution_topology_summary.console_node_count, 1);
        assert_eq!(
            observation
                .execution_topology_summary
                .track_lane_group_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_group_count, 2);
        assert_eq!(
            observation.execution_topology_summary.console_group_count,
            1
        );
        assert_eq!(observation.execution_topology_summary.track_lanes.len(), 2);
        assert_eq!(observation.execution_topology_summary.bus_groups.len(), 2);
        assert_eq!(
            observation.execution_topology_summary.console_groups.len(),
            1
        );
        assert_eq!(observation.execution_topology_summary.lanes.len(), 2);
        assert_eq!(observation.metering_snapshot.track_lanes.len(), 2);
        assert_eq!(observation.metering_snapshot.bus_groups.len(), 2);
        assert_eq!(observation.metering_snapshot.console_groups.len(), 1);
        assert!(observation.metering_snapshot.send_returns.is_empty());
        assert!(observation
            .render_compact()
            .contains("engine_scheduler_topology_compatible=true"));
        assert!(observation
            .render_compact()
            .contains("engine_scheduler_topology_track_lanes=2/2"));
        assert!(observation
            .render_compact()
            .contains("execution_topology_summary_roles=1/2/1/0/1"));
        assert!(observation
            .render_compact()
            .contains("execution_topology_summary_lane_shapes=Anticipative:1|Realtime:4"));
        assert!(observation
            .render_compact()
            .contains("metering_snapshot_routes=2/2/0/1"));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_bus_groups=2"));
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_console_groups=1"));
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_issue_count=0"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_lane_0=Anticipative"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_lane_1=Realtime"));
        assert!(supervisor
            .render_multiline()
            .contains("metering_snapshot_meter_count="));
        assert!(supervisor
            .render_multiline()
            .contains("metering_snapshot_track_lane_count=2"));
        assert!(supervisor
            .render_multiline()
            .contains("metering_snapshot_console_group_0=console:main"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_node_2=track-1/Realtime/StatefulRealtime/TrackLane/track_lane_id=Some(\"track:bass\")"));
        assert!(supervisor.render_multiline().contains(
            "execution_topology_summary_node_4=console-main/Realtime/InlineRealtime/ConsoleNode"
        ));

        let json = supervisor.render_json();
        assert!(json.contains("\"scheduler_topology\":{\"track_lane_node_count\":2"));
        assert!(json.contains("\"track_lane_group_count\":2"));
        assert!(json.contains("\"schedule_stream_count\":2"));
        assert!(json.contains("\"compatible\":true"));
        assert!(json.contains("\"metering_snapshot\":{\"meter_count\":"));
        assert!(json.contains("\"track_lanes\":["));
        assert!(json.contains("\"console_groups\":["));
        assert!(json.contains("\"execution_topology_summary\":{\"node_count\":5"));
        assert!(json.contains("\"track_lane_node_count\":2"));
        assert!(json.contains("\"lane\":\"Anticipative\""));
        assert!(json.contains("\"lane\":\"Realtime\""));
        assert!(json.contains("\"node_id\":\"track-0\""));
        assert!(json.contains("\"track_lane_id\":\"track:drums\""));
        assert!(json.contains("\"bus_group_id\":\"mix:master\""));
        assert!(json.contains("\"console_group_id\":\"console:main\""));
        assert!(json.contains("\"track_lanes\":["));
        assert!(json.contains("\"bus_groups\":["));
        assert!(json.contains("\"console_groups\":["));
        assert!(json.contains("\"node_id\":\"console-main\""));
        assert!(json.contains("\"output_bus_id\":\"main:out\""));
    }

    #[test]
    fn runtime_metering_snapshot_reports_loudness_for_non_silent_output() {
        let mut metering = RuntimeMeteringStateModel::default();
        let output = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![0.5, -0.5, 0.25, -0.25, 0.75, -0.75, 0.125, -0.125],
        );

        metering.capture(
            48_000,
            &output,
            vec![RuntimeMeterSourceSnapshot {
                bus_id: "main:out".into(),
                topology_role: RuntimeMeterSourceRole::Bus,
                track_lane_id: None,
                bus_group_id: Some("mix:master".into()),
                console_group_id: None,
                send_return_id: None,
                producer_node_ids: vec!["bus-main".into()],
                peak_level: 0.75,
                rms_level: 0.4677072,
                latency_samples: 0,
                tail_samples: 0,
                summary: "main output".into(),
            }],
        );

        let snapshot = metering.snapshot();
        assert_eq!(snapshot.meter_count, 1);
        assert_eq!(snapshot.main_output_peak_level, Some(0.75));
        assert_eq!(snapshot.main_output_rms_level, Some(0.4677072));
        assert!(snapshot.momentary_loudness_lufs.is_some());
        assert!(snapshot.integrated_loudness_lufs.is_some());
        assert_eq!(snapshot.clipped_sample_count, 0);
        assert!(snapshot
            .meters
            .iter()
            .any(|meter| meter.bus_id == "main:out"));
    }

    #[test]
    fn runtime_automation_projection_drives_within_block_parameter_events() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 6));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:automation-playback".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "gain".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
                }],
            })
            .expect("apply automation playback graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:automation-playback".into(),
                contract_count: 1,
                nodes: vec![GraphNodeContractProjection {
                    node_id: "gain".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Mono,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "main:out".into(),
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
                }],
            })
            .expect("apply automation playback contract");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:automation-playback".into(),
                stream_count: 1,
            })
            .expect("apply automation playback schedule");
        let receipt = runtime
            .apply_automation_projection(RuntimeAutomationProjection {
                lane_count: 1,
                point_count: 2,
                lanes: vec![RuntimeAutomationLaneProjection {
                    automation_lane_id: "automation-lane:gain".into(),
                    target: RuntimeAutomationTargetProjection {
                        node_id: "gain".into(),
                        parameter_id: "gain".into(),
                    },
                    base_normalized_value: 0.0,
                    interpolation: crate::interfaces::RuntimeAutomationInterpolation::Hold,
                    resolution: RuntimeAutomationResolution::default(),
                    point_count: 2,
                    points: vec![
                        RuntimeAutomationPointProjection {
                            time_samples: 2,
                            normalized_value: 0.5,
                        },
                        RuntimeAutomationPointProjection {
                            time_samples: 4,
                            normalized_value: 1.0,
                        },
                    ],
                }],
            })
            .expect("apply automation projection");
        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: receipt.accepted_epoch,
                events: Vec::new(),
            })
            .expect("apply automation epoch batch");
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .expect("apply transport");

        let input =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 6]);
        let result = runtime
            .process_engine_block(1, 1, input)
            .expect("process automated block");

        assert_eq!(
            result.snapshot.parameter_epoch,
            Some(receipt.accepted_epoch)
        );
        assert_eq!(result.snapshot.parameter_event_count, 3);
        assert_eq!(result.snapshot.parameter_sub_block_count, 3);
        assert_eq!(result.snapshot.parameter_ignored_event_count, 0);
        let expected = [0.0_f32, 0.0, 0.5, 0.5, 1.0, 1.0];
        for (actual, expected) in result.output.samples().iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1.0e-6);
        }

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.lane_count, 1);
        assert_eq!(automation.point_count, 2);
        assert_eq!(automation.mapped_lane_count, 1);
        assert_eq!(automation.unmapped_lane_count, 0);
        assert_eq!(automation.last_batch_epoch, Some(receipt.accepted_epoch));
        assert_eq!(automation.last_batch_event_count, 3);
        assert_eq!(automation.last_batch_sub_block_count, 3);
        assert_eq!(automation.last_batch_ignored_event_count, 0);
        assert_eq!(automation.last_batch_coalesced_event_count, 0);
        assert_eq!(automation.last_batch_max_sample_offset, Some(4));
        assert_eq!(automation.last_block_sequence, Some(1));
        assert_eq!(automation.last_timeline_position_samples, Some(0));
        assert_eq!(automation.transport_playing, Some(true));
    }

    #[test]
    fn runtime_graph_contract_projection_updates_execution_topology_for_projected_graphs() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:projected-topology".into(),
                node_count: 4,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "track-input".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-insert".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                    },
                    GraphNodeProjection {
                        node_id: "bus-main".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                    },
                    GraphNodeProjection {
                        node_id: "output-main".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.15 }],
                    },
                ],
            })
            .expect("apply projected graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:projected-topology".into(),
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
                        node_id: "plugin-insert".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:track:lead".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:mix:tracks".into(),
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
                                bus_id: "bus:mix:tracks".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:console:main".into(),
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
                        node_id: "output-main".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:console:main".into(),
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
            .expect("apply projected graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:projected-topology".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin-insert".into(),
                    sandbox_id: "sandbox:lead".into(),
                }],
            })
            .expect("apply plugin bindings");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        runtime
            .process_engine_block(1, 1, block)
            .expect("process projected topology block");

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.execution_topology_summary.node_count, 4);
        assert_eq!(
            observation.execution_topology_summary.track_lane_node_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
        assert_eq!(observation.execution_topology_summary.console_node_count, 1);
        assert_eq!(
            observation
                .execution_topology_summary
                .track_lane_group_count,
            1
        );
        assert_eq!(observation.execution_topology_summary.bus_group_count, 2);
        assert_eq!(
            observation.execution_topology_summary.console_group_count,
            1
        );
        assert_eq!(observation.execution_topology_summary.track_lanes.len(), 1);
        assert_eq!(observation.execution_topology_summary.bus_groups.len(), 2);
        assert_eq!(
            observation.execution_topology_summary.console_groups.len(),
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .plugin_chain
                .chain_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .plugin_chain
                .stage_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .plugin_chain
                .pending_render_stage_count,
            1
        );
        assert!(observation
            .execution_topology_summary
            .track_lanes
            .iter()
            .any(|track_lane| {
                track_lane.track_lane_id == "track:lead"
                    && track_lane.bus_group_ids == vec!["mix:tracks".to_string()]
                    && track_lane.plugin_chain.chain_count == 1
                    && track_lane.plugin_chain.pending_render_stage_count == 1
                    && track_lane
                        .output_bus_ids
                        .contains(&"bus:track:lead".to_string())
                    && track_lane
                        .output_bus_ids
                        .contains(&"bus:mix:tracks".to_string())
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "track-input"
                    && node.topology_role == GraphNodeTopologyRole::TrackLane
                    && node.track_lane_id.as_deref() == Some("track:lead")
                    && node.output_bus_id == "bus:track:lead"
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "plugin-insert"
                    && node.plugin_sandbox_id.as_deref() == Some("sandbox:lead")
                    && node.plugin_recall_state == Some(RuntimePluginRecallState::Cold)
                    && node.plugin_compensation_state
                        == Some(RuntimePluginCompensationState::PendingRender)
                    && node.plugin_realized_latency_samples.is_none()
                    && node.input_bus_id == "bus:track:lead"
                    && node.output_bus_id == "bus:mix:tracks"
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "bus-main"
                    && node.topology_role == GraphNodeTopologyRole::Bus
                    && node.bus_group_id.as_deref() == Some("mix:master")
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "output-main"
                    && node.topology_role == GraphNodeTopologyRole::ConsoleNode
                    && node.console_group_id.as_deref() == Some("console:main")
                    && node.input_bus_id == "bus:console:main"
                    && node.output_bus_id == "main:out"
            }));
        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_plugin_chain=1/1/1/0/0/0/0/0/0/0/0"));
        let json = supervisor.render_json();
        assert!(json.contains("\"plugin_chain\":{\"chain_count\":1"));
        assert!(json.contains("\"plugin_recall_state\":\"Cold\""));
        assert!(json.contains("\"plugin_compensation_state\":\"PendingRender\""));
    }

    #[test]
    fn runtime_execution_topology_summary_carries_sidechain_routing_and_fallback_receipts() {
        let mut runtime = prepare_sidechain_runtime();
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(128), 2);
        runtime
            .process_engine_block(5, 8, block)
            .expect("process sidechain routing block");

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.execution_topology_summary.secondary_input_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .required_secondary_input_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .terminal_fallback_secondary_input_count,
            0
        );
        let route = &observation.execution_topology_summary.secondary_inputs[0];
        assert_eq!(route.source_id, "sidechain-feed");
        assert_eq!(route.source_bus_id.as_deref(), Some("bus:sidechain:kick"));
        assert_eq!(
            route.target_kind,
            RuntimeSecondaryInputTargetKind::NodeInput
        );
        assert_eq!(route.target_id, "plugin-compressor");
        assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
        assert_eq!(
            route.attachment_policy,
            crate::RuntimeSecondaryInputAttachmentPolicy::Required
        );
        assert_eq!(
            route.fallback_outcome,
            crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
        );
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "plugin-compressor"
                    && node
                        .secondary_input
                        .as_ref()
                        .is_some_and(|secondary_input| {
                            secondary_input.target_kind
                                == RuntimeSecondaryInputTargetKind::NodeInput
                                && secondary_input.source_id == "sidechain-feed"
                        })
            }));
        let stage = &observation.plugin_chain_snapshot.chains[0].stages[0];
        let stage_secondary_input = stage
            .secondary_input
            .as_ref()
            .expect("plugin stage should carry sidechain route");
        assert_eq!(
            stage_secondary_input.target_kind,
            RuntimeSecondaryInputTargetKind::PluginInput
        );
        assert_eq!(stage_secondary_input.target_id, "plugin-compressor");
        assert_eq!(
            stage_secondary_input.fallback_outcome,
            crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
        );

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let json = supervisor.render_json();
        assert!(json.contains("\"secondary_input_count\":1"));
        assert!(json.contains("\"target_kind\":\"PluginInput\""));
        assert!(json.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
    }

    #[test]
    fn runtime_observation_and_render_preview_surface_spatial_execution_receipts() {
        let runtime = prepare_spatial_runtime();

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.execution_topology_summary.spatial_node_count, 2);
        assert_eq!(
            observation
                .execution_topology_summary
                .active_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .bypassed_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .fallback_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .surround_bed_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .object_aware_spatial_node_count,
            0
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .expanded_fallback_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .immersive_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .room_policy_aware_spatial_node_count,
            0
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .fallback_room_policy_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .deployment_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .folded_down_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .fallback_monitoring_scene_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .renderer_capability_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .negotiated_renderer_spatial_node_count,
            0
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .immersive_export_spatial_node_count,
            1
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .fallback_immersive_export_spatial_node_count,
            1
        );

        let stereo = observation
            .execution_topology_summary
            .nodes
            .iter()
            .find(|node| node.node_id == "spatial-stereo")
            .and_then(|node| node.spatial_execution.as_ref())
            .expect("stereo node should carry spatial execution summary");
        assert_eq!(
            stereo.adapter_class,
            crate::RuntimeSpatialAdapterClass::Balance
        );
        assert_eq!(
            stereo.execution_mode,
            crate::RuntimeSpatialExecutionMode::BalanceGroups
        );
        assert_eq!(stereo.fallback_outcome, None);
        assert_eq!(
            stereo.target_environment,
            crate::RuntimeSpatialTargetEnvironment::SourceLayout
        );
        assert_eq!(stereo.bed_class, crate::RuntimeSpatialBedClass::StereoBed);
        assert_eq!(stereo.object_role, None);
        assert_eq!(stereo.object_count, 0);
        assert_eq!(stereo.mix_policy, crate::RuntimeSpatialMixPolicy::BedOnly);
        assert_eq!(
            stereo.render_scope,
            crate::RuntimeSpatialRenderScope::BedRender
        );
        assert_eq!(stereo.expanded_fallback_outcome, None);
        assert_eq!(stereo.balance.as_deref(), Some("-0.200"));
        assert_eq!(stereo.immersive_room_policy, None);
        assert_eq!(stereo.deployment_monitoring, None);
        assert_eq!(stereo.renderer_export, None);

        let surround = observation
            .execution_topology_summary
            .nodes
            .iter()
            .find(|node| node.node_id == "spatial-surround")
            .and_then(|node| node.spatial_execution.as_ref())
            .expect("surround node should carry spatial execution summary");
        assert_eq!(
            surround.execution_mode,
            crate::RuntimeSpatialExecutionMode::Bypassed
        );
        assert_eq!(
            surround.fallback_outcome,
            Some(crate::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
        );
        assert_eq!(
            surround.bed_class,
            crate::RuntimeSpatialBedClass::CanonicalSurroundBed
        );
        assert_eq!(surround.object_role, None);
        assert_eq!(surround.object_count, 0);
        assert_eq!(
            surround.mix_policy,
            crate::RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
        );
        assert_eq!(
            surround.render_scope,
            crate::RuntimeSpatialRenderScope::BedRender
        );
        assert_eq!(
            surround.expanded_fallback_outcome,
            Some(crate::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
        );
        let surround_immersive = surround
            .immersive_room_policy
            .as_ref()
            .expect("surround node should carry immersive room policy summary");
        assert_eq!(
            surround_immersive.object_rendering_posture,
            crate::RuntimeImmersiveObjectRenderingPosture::NotRequested
        );
        assert_eq!(
            surround_immersive.room_policy_class,
            crate::RuntimeRoomPolicyClass::FallbackRoom
        );
        assert_eq!(
            surround_immersive.room_policy_authority,
            crate::RuntimeRoomPolicyAuthority::RuntimeDefault
        );
        assert_eq!(
            surround_immersive.room_outcome,
            crate::RuntimeImmersiveRoomOutcome::BypassRoomPolicy
        );
        let surround_monitoring = surround
            .deployment_monitoring
            .as_ref()
            .expect("surround node should carry deployment and monitoring summary");
        assert_eq!(
            surround_monitoring.deployment_class,
            crate::RuntimeDeploymentClass::FallbackDeployment
        );
        assert_eq!(
            surround_monitoring.fold_down_policy,
            crate::RuntimeFoldDownPolicy::FoldDownToReferenceBed
        );
        assert_eq!(
            surround_monitoring.monitoring_scene_class,
            crate::RuntimeMonitoringSceneClass::FallbackScene
        );
        assert_eq!(
            surround_monitoring.monitoring_scene_authority,
            crate::RuntimeMonitoringSceneAuthority::RuntimeDefault
        );
        assert_eq!(
            surround_monitoring.monitoring_outcome,
            crate::RuntimeMonitoringOutcome::BypassMonitoringScene
        );
        let surround_export = surround
            .renderer_export
            .as_ref()
            .expect("surround node should carry renderer and export summary");
        assert_eq!(
            surround_export.renderer_capability_posture,
            crate::RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
        );
        assert_eq!(
            surround_export.capability_authority,
            crate::RuntimeRendererCapabilityAuthority::RuntimeDefault
        );
        assert_eq!(
            surround_export.immersive_export_class,
            crate::RuntimeImmersiveExportClass::FallbackExport
        );
        assert_eq!(
            surround_export.export_authority,
            crate::RuntimeImmersiveExportAuthority::RuntimeDefault
        );
        assert_eq!(
            surround_export.export_outcome,
            crate::RuntimeImmersiveExportOutcome::BypassImmersiveExport
        );
        assert_eq!(surround.balance.as_deref(), Some("0.350"));
        assert_eq!(
            surround.output_layout.canonical_layout,
            Some(crate::RuntimeCanonicalChannelLayout::Surround5_1)
        );

        let plugin_stage_count = observation
            .plugin_chain_snapshot
            .chains
            .iter()
            .flat_map(|chain| chain.stages.iter())
            .filter(|stage| stage.spatial_execution.is_some())
            .count();
        assert_eq!(plugin_stage_count, 2);

        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &RuntimeOfflineRenderRequest {
                request_id: "render:spatial-preview".into(),
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
        .expect("build offline render spatial preview");
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
        assert!(preview
            .chain_contract
            .spatial_stages
            .iter()
            .any(|stage| stage.node_id == "spatial-stereo"
                && stage.spatial.execution_mode
                    == crate::RuntimeSpatialExecutionMode::BalanceGroups
                && stage.spatial.bed_class == crate::RuntimeSpatialBedClass::StereoBed
                && stage.spatial.mix_policy == crate::RuntimeSpatialMixPolicy::BedOnly));
        assert!(preview
            .chain_contract
            .spatial_stages
            .iter()
            .any(|stage| stage.node_id == "spatial-surround"
                && stage.spatial.fallback_outcome
                    == Some(crate::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
                && stage.spatial.expanded_fallback_outcome
                    == Some(
                        crate::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                    )
                && stage.spatial.bed_class == crate::RuntimeSpatialBedClass::CanonicalSurroundBed
                && stage
                    .spatial
                    .immersive_room_policy
                    .as_ref()
                    .is_some_and(|immersive| {
                        immersive.room_policy_class == crate::RuntimeRoomPolicyClass::FallbackRoom
                            && immersive.room_outcome
                                == crate::RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                    })
                && stage
                    .spatial
                    .deployment_monitoring
                    .as_ref()
                    .is_some_and(|monitoring| {
                        monitoring.deployment_class
                            == crate::RuntimeDeploymentClass::FallbackDeployment
                            && monitoring.fold_down_policy
                                == crate::RuntimeFoldDownPolicy::FoldDownToReferenceBed
                            && monitoring.monitoring_scene_class
                                == crate::RuntimeMonitoringSceneClass::FallbackScene
                            && monitoring.monitoring_outcome
                                == crate::RuntimeMonitoringOutcome::BypassMonitoringScene
                    })
                && stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.renderer_capability_posture
                            == crate::RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                            && renderer.immersive_export_class
                                == crate::RuntimeImmersiveExportClass::FallbackExport
                            && renderer.export_outcome
                                == crate::RuntimeImmersiveExportOutcome::BypassImmersiveExport
                    })));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let json = supervisor.render_json();
        assert!(json.contains("\"spatial_node_count\":2"));
        assert!(json.contains("\"active_spatial_node_count\":1"));
        assert!(json.contains("\"bypassed_spatial_node_count\":1"));
        assert!(json.contains("\"surround_bed_spatial_node_count\":1"));
        assert!(json.contains("\"expanded_fallback_spatial_node_count\":1"));
        assert!(json.contains("\"immersive_spatial_node_count\":1"));
        assert!(json.contains("\"fallback_room_policy_spatial_node_count\":1"));
        assert!(json.contains("\"deployment_spatial_node_count\":1"));
        assert!(json.contains("\"folded_down_spatial_node_count\":1"));
        assert!(json.contains("\"fallback_monitoring_scene_spatial_node_count\":1"));
        assert!(json.contains("\"renderer_capability_spatial_node_count\":1"));
        assert!(json.contains("\"negotiated_renderer_spatial_node_count\":0"));
        assert!(json.contains("\"immersive_export_spatial_node_count\":1"));
        assert!(json.contains("\"fallback_immersive_export_spatial_node_count\":1"));
        assert!(json.contains("\"adapter_class\":\"Balance\""));
        assert!(json.contains("\"bed_class\":\"CanonicalSurroundBed\""));
        assert!(json.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
        assert!(json.contains("\"render_scope\":\"BedRender\""));
        assert!(json.contains("\"execution_mode\":\"Bypassed\""));
        assert!(json.contains("\"fallback_outcome\":\"BypassSpatialProcessing\""));
        assert!(json.contains("\"expanded_fallback_outcome\":\"CollapseToBaselineSpatial\""));
        assert!(json.contains("\"immersive_room_policy\":{"));
        assert!(json.contains("\"room_policy_class\":\"FallbackRoom\""));
        assert!(json.contains("\"room_outcome\":\"BypassRoomPolicy\""));
        assert!(json.contains("\"deployment_monitoring\":{"));
        assert!(json.contains("\"deployment_class\":\"FallbackDeployment\""));
        assert!(json.contains("\"fold_down_policy\":\"FoldDownToReferenceBed\""));
        assert!(json.contains("\"monitoring_scene_class\":\"FallbackScene\""));
        assert!(json.contains("\"monitoring_outcome\":\"BypassMonitoringScene\""));
        assert!(json.contains("\"renderer_export\":{"));
        assert!(json.contains("\"renderer_capability_posture\":\"FallbackNegotiation\""));
        assert!(json.contains("\"immersive_export_class\":\"FallbackExport\""));
        assert!(json.contains("\"export_outcome\":\"BypassImmersiveExport\""));
    }

    #[test]
    fn runtime_execution_topology_summarizes_send_return_routes_explicitly() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:send-return-summary".into(),
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
            .expect("apply projected graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:send-return-summary".into(),
                contract_count: 5,
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
                        node_id: "bus-dry".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:track:lead".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
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
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:track:lead".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
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
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:fx:plate".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
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
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:mix:master".into(),
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
            .expect("apply projected graph contracts");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        runtime
            .process_engine_block(3, 5, block)
            .expect("process send return topology block");

        let metering = runtime.get_metering_snapshot();
        assert_eq!(metering.send_returns.len(), 1);
        assert_eq!(metering.bus_connection_count, 5);
        assert_eq!(metering.auxiliary_path_count, 3);
        assert!(metering.send_returns.iter().any(|send_return| {
            send_return.send_return_id == "fx:plate"
                && send_return.aggregate.meter_count == 2
                && send_return
                    .aggregate
                    .metered_bus_ids
                    .contains(&"bus:fx:plate".to_string())
                && send_return
                    .aggregate
                    .metered_bus_ids
                    .contains(&"bus:mix:master".to_string())
        }));
        assert!(metering.bus_connections.iter().any(|connection| {
            connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
                && connection.source_bus_role == crate::RuntimeBusRole::AuxSend
                && connection.target_bus_role == crate::RuntimeBusRole::AuxReturn
                && connection.auxiliary_path_kind
                    == Some(crate::RuntimeAuxiliaryPathKind::SendReturn)
                && connection.auxiliary_path_id.as_deref() == Some("send_return:fx:plate")
        }));
        assert!(metering.auxiliary_paths.iter().any(|path| {
            path.auxiliary_path_id == "send_return:fx:plate"
                && path.path_kind == crate::RuntimeAuxiliaryPathKind::SendReturn
                && path.bus_role == crate::RuntimeBusRole::AuxSend
                && path
                    .connection_ids
                    .contains(&"send-fx:bus:fx:plate->return-fx:bus:fx:plate".to_string())
        }));
        assert!(metering.auxiliary_paths.iter().any(|path| {
            path.auxiliary_path_id == "bus_group:mix:master"
                && path.path_kind == crate::RuntimeAuxiliaryPathKind::Submix
                && path.bus_role == crate::RuntimeBusRole::Submix
                && path.source_node_ids.contains(&"bus-dry".to_string())
                && path.target_node_ids.contains(&"output-main".to_string())
        }));

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation
                .execution_topology_summary
                .send_return_node_count,
            2
        );
        assert_eq!(
            observation
                .execution_topology_summary
                .send_return_group_count,
            1
        );
        assert_eq!(
            observation.execution_topology_summary.bus_connection_count,
            5
        );
        assert_eq!(
            observation.execution_topology_summary.auxiliary_path_count,
            3
        );
        assert_eq!(observation.execution_topology_summary.send_returns.len(), 1);
        assert_eq!(observation.metering_snapshot.send_returns.len(), 1);
        assert_eq!(observation.metering_snapshot.bus_connection_count, 5);
        assert_eq!(observation.metering_snapshot.auxiliary_path_count, 3);
        assert!(observation
            .execution_topology_summary
            .send_returns
            .iter()
            .any(|send_return| {
                send_return.send_return_id == "fx:plate"
                    && send_return.send_node_ids == vec!["send-fx".to_string()]
                    && send_return.return_node_ids == vec!["return-fx".to_string()]
                    && send_return
                        .input_bus_ids
                        .contains(&"bus:track:lead".to_string())
                    && send_return
                        .input_bus_ids
                        .contains(&"bus:fx:plate".to_string())
                    && send_return
                        .output_bus_ids
                        .contains(&"bus:fx:plate".to_string())
                    && send_return
                        .output_bus_ids
                        .contains(&"bus:mix:master".to_string())
            }));
        assert!(observation
            .execution_topology_summary
            .bus_connections
            .iter()
            .any(|connection| {
                connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
                    && connection.source_bus_role == crate::RuntimeBusRole::AuxSend
                    && connection.target_bus_role == crate::RuntimeBusRole::AuxReturn
            }));
        assert!(observation
            .execution_topology_summary
            .auxiliary_paths
            .iter()
            .any(|path| {
                path.auxiliary_path_id == "send_return:fx:plate"
                    && path
                        .connection_ids
                        .contains(&"send-fx:bus:fx:plate->return-fx:bus:fx:plate".to_string())
                    && path.connection_ids.contains(
                        &"return-fx:bus:mix:master->output-main:bus:mix:master".to_string(),
                    )
            }));
        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(supervisor
            .render_multiline()
            .contains("metering_snapshot_send_return_0=fx:plate"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_bus_connection_count=5"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_auxiliary_path_0="));
        let json = supervisor.render_json();
        assert!(json.contains("\"metering_snapshot\":{\"meter_count\":"));
        assert!(json.contains("\"send_return_group_count\":1"));
        assert!(json.contains("\"send_returns\":["));
        assert!(json.contains("\"send_return_id\":\"fx:plate\""));
        assert!(json.contains("\"bus_connection_count\":5"));
        assert!(json.contains("\"auxiliary_path_count\":3"));
        assert!(json.contains("\"connection_id\":\"send-fx:bus:fx:plate->return-fx:bus:fx:plate\""));
        assert!(json.contains("\"auxiliary_path_id\":\"send_return:fx:plate\""));
    }

    #[test]
    fn hardware_config_updates_runtime_and_backend_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime
            .apply_hardware_config(HardwareConfigRequest::new(
                96_000,
                256,
                BackendPolicyTier::Tier1Brokered,
            ))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
        assert_eq!(
            runtime.get_diagnostics_snapshot().backend_policy_tier,
            BackendPolicyTier::Tier1Brokered
        );
    }

    #[test]
    fn runtime_executes_applied_graph_block_and_updates_snapshot() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:test".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![
                            GraphStageSpec::Gain { linear: 0.5 },
                            GraphStageSpec::Bias { amount: 0.2 },
                        ],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 96,
                tempo_bpm: 120.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 64,
                    end_samples: 128,
                }),
            })
            .unwrap();
        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch(),
                events: vec![ParameterEvent {
                    target: "engine.runtime.test".into(),
                    sample_offset: 0,
                    normalized_value: 0.5,
                }],
            })
            .unwrap();

        let result = runtime
            .process_engine_block(
                1,
                42,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 3),
            )
            .unwrap();

        assert_eq!(
            result.snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(result.snapshot.node_count, 2);
        assert_eq!(result.snapshot.stateful_node_count, 1);
        assert_eq!(result.snapshot.latency_node_count, 1);
        assert!(result.snapshot.anticipative_planning_enabled);
        assert_eq!(result.snapshot.inline_realtime_node_count, 1);
        assert_eq!(result.snapshot.stateful_realtime_node_count, 0);
        assert_eq!(result.snapshot.anticipative_eligible_node_count, 1);
        assert_eq!(result.snapshot.phase_count, 2);
        assert_eq!(result.snapshot.anticipative_phase_count, 1);
        assert_eq!(result.snapshot.lane_count, 2);
        assert_eq!(result.snapshot.anticipative_lane_count, 1);
        assert_eq!(
            result.snapshot.lane_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.dispatch_count, 2);
        assert_eq!(result.snapshot.dispatch_boundary_count, 1);
        assert_eq!(
            result.snapshot.dispatch_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.prepared_dispatch_count, 1);
        assert_eq!(result.snapshot.realtime_dispatch_count, 1);
        assert_eq!(result.snapshot.dispatch_handoff_count, 1);
        assert!(result.snapshot.prework_cache_enabled);
        assert_eq!(
            result.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(result.snapshot.prework_cache_admissions, 1);
        assert_eq!(result.snapshot.prework_cache_consumptions, 1);
        assert_eq!(result.snapshot.prework_cache_hits, 0);
        assert_eq!(result.snapshot.prework_cache_misses, 1);
        assert_eq!(result.snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(result.snapshot.prework_cache_retirement_count, 0);
        assert_eq!(
            result.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(result.snapshot.prework_cache_block_freshness_window, 2);
        assert_eq!(
            result.snapshot.prework_cache_remaining_valid_blocks,
            Some(2)
        );
        assert!(!result.snapshot.last_prework_cache_hit);
        assert_eq!(result.snapshot.last_prework_invalidation_reason, None);
        assert_eq!(
            result.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            result.snapshot.prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            result.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(result.snapshot.last_prework_source_block_sequence, Some(42));
        assert_eq!(
            result.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            result.snapshot.last_prework_admission_block_sequence,
            Some(42)
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_processing_epoch,
            Some(1)
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_block_sequence,
            Some(42)
        );
        assert_eq!(
            result.snapshot.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::AnticipativeEligible,
            ]
        );
        assert_eq!(result.snapshot.planned_nodes.len(), 2);
        assert_eq!(result.snapshot.stage_count, 3);
        assert_eq!(result.snapshot.total_latency_samples, 16);
        assert_eq!(result.snapshot.max_node_latency_samples, 16);
        assert_eq!(result.snapshot.processed_blocks, 1);
        assert_eq!(result.snapshot.last_processing_epoch, Some(1));
        assert_eq!(result.snapshot.last_block_sequence, Some(42));
        assert_eq!(result.snapshot.last_frame_count, 8);
        assert_eq!(result.snapshot.last_channel_count, 2);
        assert!(result.snapshot.last_prework_output_peak.is_some());
        assert_eq!(
            result.snapshot.last_prework_output_peak,
            result.snapshot.last_realtime_input_peak
        );
        assert!(result.snapshot.last_output_peak.unwrap_or_default() <= 0.7);
        assert!(result.snapshot.last_output_rms.unwrap_or_default() > 0.0);
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(96)
        );
        assert!(result.output.samples().first().is_some());
        assert_eq!(
            runtime
                .applied_transport
                .map(|transport| transport.timeline_position_samples),
            Some(104)
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.engine_block_snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(observation.engine_block_snapshot.node_count, 2);
        assert_eq!(observation.engine_block_snapshot.stateful_node_count, 1);
        assert!(
            observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            observation.engine_block_snapshot.inline_realtime_node_count,
            1
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            0
        );
        assert_eq!(observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            observation.engine_block_snapshot.anticipative_phase_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.lane_count, 2);
        assert_eq!(observation.engine_block_snapshot.anticipative_lane_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_count, 2);
        assert_eq!(observation.engine_block_snapshot.dispatch_boundary_count, 1);
        assert_eq!(observation.engine_block_snapshot.prepared_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.realtime_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_handoff_count, 1);
        assert_eq!(observation.scheduler_summary.phase_count, 2);
        assert_eq!(observation.scheduler_summary.lane_count, 2);
        assert_eq!(observation.scheduler_summary.dispatch_count, 2);
        assert_eq!(
            observation.scheduler_snapshot.state,
            RuntimeSchedulerState::Configured
        );
        assert_eq!(
            observation.scheduler_snapshot.phase,
            RuntimeExecutionPhase::Idle
        );
        assert!(observation.scheduler_snapshot.graph_applied);
        assert!(!observation.scheduler_snapshot.schedule_applied);
        assert!(observation.scheduler_snapshot.transport_projected);
        assert_eq!(
            observation.scheduler_summary.prework_service_state,
            RuntimePreworkServiceState::Disabled
        );
        assert_eq!(observation.block_summary.processed_blocks, 1);
        assert_eq!(observation.block_summary.transport_epoch, 1);
        assert_eq!(
            observation.block_summary.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert!(!observation.degradation_summary.readiness_degraded);
        assert_eq!(observation.degradation_summary.xrun_count, 0);
        assert!(observation.engine_block_snapshot.prework_cache_enabled);
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_admissions,
            1
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_consumptions,
            1
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(observation.engine_block_snapshot.prework_cache_hits, 0);
        assert_eq!(observation.engine_block_snapshot.prework_cache_misses, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_retirement_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_invalidation_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.processed_blocks, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn scheduler_snapshot_tracks_state_and_phase_transitions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let configured = runtime.get_scheduler_snapshot();
        assert_eq!(configured.state, RuntimeSchedulerState::Configured);
        assert_eq!(configured.phase, RuntimeExecutionPhase::Idle);
        assert!(!configured.graph_applied);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:scheduler".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "track".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                    },
                    GraphNodeProjection {
                        node_id: "master".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().unwrap();

        let primed = runtime.get_scheduler_snapshot();
        assert_eq!(primed.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(primed.phase, RuntimeExecutionPhase::Prework);
        assert!(primed.graph_applied);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(2),
            })
            .unwrap();
        seed_pending_prework_targets(&mut runtime, 1, &[2, 3]);

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime.service_prework_lane(1, 1).unwrap();

        let prework = runtime.get_scheduler_snapshot();
        assert_eq!(prework.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(prework.phase, RuntimeExecutionPhase::Prework);
        assert!(prework.transport_projected);

        runtime
            .process_engine_block(
                2,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(256)),
            )
            .unwrap();

        let realtime = runtime.get_scheduler_snapshot();
        assert_eq!(realtime.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(realtime.phase, RuntimeExecutionPhase::Realtime);
        assert_eq!(realtime.processed_block_count, 1);
    }

    #[test]
    fn scheduler_snapshot_surfaces_realtime_only_and_degraded_runtime_states() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, false);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:realtime-only".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "track".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                }],
            })
            .unwrap();
        runtime.start().unwrap();

        let realtime_only = runtime.get_scheduler_snapshot();
        assert_eq!(realtime_only.state, RuntimeSchedulerState::RealtimeOnly);
        assert_eq!(realtime_only.phase, RuntimeExecutionPhase::Priming);

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();

        let degraded = runtime.get_scheduler_snapshot();
        assert_eq!(degraded.state, RuntimeSchedulerState::Degraded);
        assert_eq!(degraded.phase, RuntimeExecutionPhase::Degraded);
    }

    #[test]
    fn runtime_replans_graph_when_anticipative_mode_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:planning".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "drive".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::TanhDrive { drive: 1.4 }],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 32,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                    },
                ],
            })
            .unwrap();

        let initial = runtime.get_engine_block_snapshot();
        assert!(initial.anticipative_planning_enabled);
        assert_eq!(initial.inline_realtime_node_count, 1);
        assert_eq!(initial.stateful_realtime_node_count, 1);
        assert_eq!(initial.anticipative_eligible_node_count, 1);
        assert_eq!(initial.prepared_dispatch_count, 1);
        assert_eq!(initial.realtime_dispatch_count, 1);
        assert_eq!(initial.dispatch_handoff_count, 1);
        assert!(initial.prework_cache_enabled);
        assert_eq!(initial.prework_cache_state, RuntimePreworkCacheState::Empty);
        assert_eq!(
            initial.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Empty
        );
        assert_eq!(initial.prework_cache_admissions, 0);
        assert_eq!(initial.prework_cache_consumptions, 0);
        assert_eq!(initial.prework_cache_hits, 0);
        assert_eq!(initial.prework_cache_misses, 0);
        assert_eq!(initial.prework_cache_invalidation_count, 0);
        assert_eq!(initial.prework_cache_retirement_count, 0);

        let mut request = RuntimeConfigRequest::new(48_000, 256);
        request.anticipative_enabled = false;
        runtime.configure(request).unwrap();

        let replanned = runtime.get_engine_block_snapshot();
        assert!(!replanned.anticipative_planning_enabled);
        assert_eq!(replanned.inline_realtime_node_count, 1);
        assert_eq!(replanned.stateful_realtime_node_count, 2);
        assert_eq!(replanned.anticipative_eligible_node_count, 0);
        assert_eq!(replanned.phase_count, 2);
        assert_eq!(replanned.anticipative_phase_count, 0);
        assert_eq!(replanned.lane_count, 1);
        assert_eq!(replanned.anticipative_lane_count, 0);
        assert_eq!(
            replanned.lane_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(replanned.dispatch_count, 1);
        assert_eq!(replanned.dispatch_boundary_count, 0);
        assert_eq!(replanned.prepared_dispatch_count, 0);
        assert_eq!(replanned.realtime_dispatch_count, 1);
        assert_eq!(replanned.dispatch_handoff_count, 0);
        assert!(!replanned.prework_cache_enabled);
        assert_eq!(
            replanned.prework_cache_state,
            RuntimePreworkCacheState::Disabled
        );
        assert_eq!(
            replanned.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Disabled
        );
        assert_eq!(replanned.prework_cache_admissions, 0);
        assert_eq!(replanned.prework_cache_consumptions, 0);
        assert_eq!(replanned.prework_cache_valid_until_processing_epoch, None);
        assert_eq!(
            replanned.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(replanned.prework_cache_invalidation_count, 0);
        assert_eq!(replanned.prework_cache_retirement_count, 0);
        assert_eq!(
            replanned.dispatch_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(
            replanned.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::StatefulRealtime,
            ]
        );
        assert_eq!(replanned.planned_nodes.len(), 3);
        assert_eq!(
            replanned
                .planned_nodes
                .iter()
                .map(|node| (node.node_id.as_str(), format!("{:?}", node.group)))
                .collect::<Vec<_>>(),
            vec![
                ("input", "InlineRealtime".into()),
                ("drive", "StatefulRealtime".into()),
                ("output", "StatefulRealtime".into()),
            ]
        );
    }

    #[test]
    fn safe_mode_sets_degraded_readiness() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();

        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn runtime_reuses_prework_cache_for_matching_adjacent_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:cache".into(),
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

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 11);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(2, 2, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_hits, 0);
        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(first.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(first.snapshot.prework_cache_queued_consumptions, 0);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(first.snapshot.prework_cache_remaining_valid_blocks, Some(2));
        assert!(!first.snapshot.last_prework_cache_hit);
        assert_eq!(
            first.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_misses, 1);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 1);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(second.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Expiring
        );
        assert_eq!(
            second.snapshot.prework_cache_remaining_valid_blocks,
            Some(1)
        );
        assert!(second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(second.snapshot.last_prework_source_block_sequence, Some(1));
        assert_eq!(
            second.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_admission_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prepared_dispatch_count, 1);
        assert_eq!(second.snapshot.realtime_dispatch_count, 1);
    }

    #[test]
    fn runtime_consumes_primed_prework_for_the_next_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework".into(),
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
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let next_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let next_batch = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                sample_offset: 0,
                normalized_value: 0.5,
            }],
        };
        let next_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                next_block.clone(),
                Some(next_batch.epoch),
                Some(next_transport),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_admissions, 1);
        assert_eq!(primed.prework_cache_queued_admissions, 1);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(2));
        assert_eq!(primed.last_prework_admitted_from_block_sequence, Some(1));

        runtime.apply_parameter_batch(next_batch).unwrap();
        runtime.apply_transport_projection(next_transport).unwrap();
        let consumed = runtime.process_engine_block(1, 2, next_block).unwrap();
        assert_eq!(consumed.snapshot.prework_cache_hits, 1);
        assert_eq!(consumed.snapshot.prework_cache_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_consumptions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_consumptions, 1);
        assert!(consumed.snapshot.last_prework_cache_hit);
        assert_eq!(consumed.snapshot.last_prework_invalidation_reason, None);
        assert_eq!(
            consumed.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(72)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn runtime_prework_queue_consumes_multiple_future_blocks_in_order() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-pipeline".into(),
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
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let block2 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let block3 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 13);
        let batch2 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                sample_offset: 0,
                normalized_value: 0.5,
            }],
        };
        let batch3 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(4),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                sample_offset: 0,
                normalized_value: 0.65,
            }],
        };
        let transport2 = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let transport3 = TransportProjection {
            playing: true,
            timeline_position_samples: 80,
            tempo_bpm: 120.0,
            loop_state: None,
        };

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block2.clone(),
                Some(batch2.epoch),
                Some(transport2),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                3,
                1,
                block3.clone(),
                Some(batch3.epoch),
                Some(transport3),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 2);
        assert_eq!(primed.prework_cache_peak_queue_depth, 2);
        assert_eq!(primed.prework_cache_queued_admissions, 2);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(3));

        runtime.apply_parameter_batch(batch2).unwrap();
        runtime.apply_transport_projection(transport2).unwrap();
        let second = runtime.process_engine_block(1, 2, block2).unwrap();
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );

        runtime.apply_parameter_batch(batch3).unwrap();
        runtime.apply_transport_projection(transport3).unwrap();
        let third = runtime.process_engine_block(1, 3, block3).unwrap();
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_queued_consumptions, 2);
        assert_eq!(third.snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            third.snapshot.last_prework_consumption_block_sequence,
            Some(3)
        );
        assert_eq!(
            third.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
    }

    #[test]
    fn runtime_prework_queue_evicts_oldest_future_entry_when_capacity_is_exceeded() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-eviction".into(),
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
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        for offset in 0..4 {
            let target_block_sequence = 2 + offset;
            let block = synthetic_stereo_block(
                SampleRate(48_000),
                FrameCount(8),
                12 + target_block_sequence,
            );
            let batch_epoch = runtime
                .projection_epoch()
                .saturating_add(3)
                .saturating_add(offset);
            let transport = TransportProjection {
                playing: true,
                timeline_position_samples: 72 + (offset as i64 * 8),
                tempo_bpm: 120.0,
                loop_state: None,
            };
            assert!(runtime
                .prepare_engine_prework_for_block_with_future_state(
                    1,
                    target_block_sequence,
                    1,
                    block,
                    Some(batch_epoch),
                    Some(transport),
                )
                .unwrap());
        }

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 3);
        assert_eq!(primed.prework_cache_peak_queue_depth, 3);
        assert_eq!(primed.prework_cache_queued_admissions, 4);
        assert_eq!(primed.prework_cache_invalidation_count, 1);
        assert_eq!(
            primed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::QueueCapacityExceeded)
        );
        assert_eq!(
            primed.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::QueueCapacityExceeded)
        );
        assert_eq!(primed.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_reuses_existing_future_queue_entry_when_target_state_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-reuse".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block.clone(),
                Some(9),
                Some(transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                block,
                Some(9),
                Some(transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 1);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(1));
    }

    #[test]
    fn runtime_replaces_future_queue_entry_when_target_state_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-replace".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let first_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let replacement_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 104,
            tempo_bpm: 121.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 42),
                Some(9),
                Some(first_transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43),
                Some(10),
                Some(replacement_transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 2);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::SupersededByAdmission)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::SupersededByAdmission)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(2));
    }

    #[test]
    fn runtime_planning_window_retires_future_entries_not_in_revised_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-revision".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: 2,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 52),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: 3,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(1, targets)
                .expect("initial planning window"),
            2
        );

        let revised_targets = vec![RuntimePreworkWindowTarget {
            target_block_sequence: 3,
            admitted_from_block_sequence: 2,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
            parameter_epoch_override: Some(10),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 104,
                tempo_bpm: 121.0,
                loop_state: None,
            }),
        }];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(2, revised_targets)
                .expect("revised planning window"),
            1
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningWindowRevised)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::PlanningWindowRevised)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_planning_window_reuses_existing_future_sequences_and_allocates_missing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-sequences".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let first_future_sequence = runtime.allocate_block_sequence();
        let second_future_sequence = runtime.allocate_block_sequence();

        let initial_targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: first_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: second_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        runtime
            .prepare_engine_prework_window(1, initial_targets)
            .expect("initial planning window");

        let revised_sequences =
            runtime.plan_prework_window_block_sequences(first_future_sequence, 2);
        assert_eq!(
            revised_sequences,
            vec![
                second_future_sequence,
                second_future_sequence.saturating_add(1)
            ]
        );
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_window_target_count, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![first_future_sequence, second_future_sequence]
        );
    }

    #[test]
    fn runtime_primes_prework_window_from_forecast_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-prework".into(),
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

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 2,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 17,
            transport_playing: true,
            transport_tempo_bpm: 122.0,
            transport_loop_length_blocks: 24,
            parameter_target: "engine.server.balance".into(),
            parameter_cycle_length: 6,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime forecast window");
        assert_eq!(admitted, 2);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));

        let transport = runtime.forecast_transport_projection_for_block(2, &policy);
        assert_eq!(transport.tempo_bpm, 122.0);
        assert_eq!(transport.timeline_position_samples, 512);

        let batch = runtime.forecast_parameter_batch_for_block(2, &policy);
        assert_eq!(batch.epoch, 4);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].target, "engine.server.balance");
        assert!((batch.events[0].normalized_value - 0.4).abs() < 1.0e-6);
    }

    #[test]
    fn runtime_forecast_policy_limits_prework_window_depth() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-limit".into(),
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

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 1,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime limited forecast window");
        assert_eq!(admitted, 1);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
    }

    #[test]
    fn runtime_constrained_anticipative_window_caps_widened_service_realization() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 1,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set constrained widened forecast policy");
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:constrained-window-widened",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:constrained-window-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened constrained schedule");
        runtime.start().expect("start runtime");

        for block_sequence in 1..=3u64 {
            let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), block_sequence);
            apply_current_forecast_block_state(&mut runtime, block_sequence);
            let snapshot = runtime
                .process_engine_block(block_sequence, block_sequence, block)
                .expect("process constrained widened block")
                .snapshot;

            assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
            assert!(snapshot.scheduler_topology.compatible);
            assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
            assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
            assert_eq!(snapshot.last_prework_service_cycle_count, 1);
            assert_eq!(snapshot.last_prework_service_prepared_targets, 1);
            assert!(snapshot.prework_cache_window_target_count <= 2);
            assert_eq!(snapshot.prework_pending_target_count, 0);
            assert!(snapshot.prework_cache_peak_queue_depth <= 2);
        }
    }

    #[test]
    fn runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded raw forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-budget".into(),
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
        assert_eq!(runtime.engine.prework_queue.len(), 1);
        assert!(runtime.engine.pending_prework_targets.len() > 1);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert!(snapshot.prework_pending_target_count > 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 8);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );

        runtime.start().expect("start runtime");
        let started = runtime.get_engine_block_snapshot();
        assert_eq!(started.prework_cache_queue_depth, 2);
        assert!(started.prework_pending_target_count > 0);

        let serviced_once = runtime
            .service_prework_lane(1, 1)
            .expect("service pending prework once");
        assert_eq!(serviced_once, 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_cycle_count >= 1);
        assert!(snapshot.prework_service_prepared_targets >= 1);
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(1));
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
        assert!(snapshot.last_prework_service_prepared_targets >= 1);

        let serviced_again = runtime
            .service_prework_lane(1, 2)
            .expect("service pending prework until idle");
        assert!(serviced_again >= 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert!(snapshot.prework_cache_queue_depth >= 3);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_cycle_count >= 2);
        assert!(snapshot.prework_service_prepared_targets >= 3);
        assert_eq!(snapshot.last_prework_service_cycle_count, 2);
        assert_eq!(snapshot.last_prework_service_prepared_targets, 2);
    }

    #[test]
    fn runtime_prework_service_lane_enters_starved_state_when_budget_is_zero() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 0,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set zero-budget forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-starved".into(),
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

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert!(paused.prework_pending_target_count > 0);

        runtime.start().expect("start runtime");
        runtime
            .service_prework_lane(1, 1)
            .expect("service prework lane with zero effective budget");
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Starved
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_starvation_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_resumes_after_start() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-resume".into(),
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

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert!(paused.prework_pending_target_count > 0);

        runtime.start().expect("start runtime");

        let resumed = runtime.get_engine_block_snapshot();
        assert!(matches!(
            resumed.prework_service_state,
            RuntimePreworkServiceState::Pending | RuntimePreworkServiceState::Idle
        ));
        assert!(resumed.prework_service_pause_count >= 1);
        assert!(resumed.prework_service_resume_count >= 1);
        assert!(resumed.prework_service_prepared_targets >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_yields_under_critical_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-critical".into(),
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
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Critical)
            .expect("set critical prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under critical pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Critical
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0)
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_throttles_under_elevated_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-elevated".into(),
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
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under elevated pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert!(snapshot.last_prework_service_effective_cycles <= 1);
        assert!(matches!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0 | 1)
        ));
        assert!(snapshot.prework_service_throttle_count >= 1);
        assert!(
            snapshot.prework_service_prepared_targets >= 1
                || snapshot.prework_service_yield_count >= 1
        );
    }

    #[test]
    fn runtime_elevated_pressure_preserves_deferred_prework_targets() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set elevated forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-backlog-classes".into(),
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
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");
        runtime
            .service_prework_lane(2, 3)
            .expect("service elevated lane second cycle");
        runtime
            .service_prework_lane(3, 3)
            .expect("service elevated lane third cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
        assert_eq!(snapshot.prework_pending_near_term_target_count, 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert_eq!(
            snapshot.prework_pending_target_count,
            snapshot.prework_pending_deferred_target_count
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_latency_focused_graph_expands_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set latency-focused forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:latency-focused-prework-priority".into(),
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
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(2)
        );
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle
        );
        assert_eq!(
            snapshot.last_prework_serviced_backlog_class,
            Some(RuntimePreworkBacklogClass::Deferred)
        );
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-constrained-prework-priority".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(1);
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.plugin_backed_node_count, 1);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_policy_tracks_active_plugin_sandbox_count() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-policy-tracking".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        runtime.set_active_plugin_sandboxes(1);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        runtime.set_active_plugin_sandboxes(0);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
    }

    #[test]
    fn runtime_plugin_constrained_lane_yields_when_multiple_plugin_sandboxes_are_active() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(2);
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_active_plugin_sandboxes, 2);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_schedule_widened_plugin_gate_yields_without_servicing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 6,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 32,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };
        runtime
            .set_prework_forecast_policy(policy.clone())
            .expect("set widened plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-gate-schedule-widened".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(2);
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:plugin-gate-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        let current_sequence = runtime.allocate_block_sequence();

        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime widened plugin-gated window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(admitted, 0);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0)
        );
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-bound".into(),
                }],
            })
            .expect("apply plugin-backed bindings");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 1);
        assert!(snapshot.planned_nodes.iter().any(|node| {
            node.node_id == "plugin" && node.plugin_sandbox_id.as_deref() == Some("sandbox-bound")
        }));

        runtime
            .begin_transport_session(
                "sandbox-bound",
                "lease-bound",
                "region-bound",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin bound transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-bound",
            "lease-bound",
            "region-bound",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
    }

    #[test]
    fn runtime_consumes_plugin_node_render_batch_on_matching_engine_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-render".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.2 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-render".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox:render".into(),
                }],
            })
            .expect("apply bindings");
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-render".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![PluginNodeRender {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox:render".into(),
                    output: AudioBuffer::from_interleaved(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        vec![0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625],
                    ),
                    latency_samples: 24,
                    tail_samples: 40,
                    bypassed: false,
                }],
            })
            .expect("apply plugin node render batch");

        let first = runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process plugin render block");
        let second = runtime
            .process_engine_block(
                1,
                2,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process fallback block");

        assert_eq!(
            first.output.samples(),
            &[0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625]
        );
        assert_eq!(first.snapshot.output_tail_samples, 40);
        assert_eq!(second.output.samples(), &[0.0; 8]);
        assert_eq!(second.snapshot.output_tail_samples, 0);
    }

    #[test]
    fn runtime_plugin_chain_snapshot_reports_compensation_and_recall() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
            formats: vec![PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:vst3:multiout-instrument".into(),
                    plugin_id: "com.signal.multiout".into(),
                    vendor: "Signal".into(),
                    name: "Signal Multi Output Instrument".into(),
                    format: PluginFormat::Vst3,
                    version: Some("1.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::Instrument,
                        signal_plugin::PluginFeature::Analyzer,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 0,
                            audio_outputs: 6,
                            midi_inputs: 1,
                            midi_outputs: 0,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::Instrument,
                                signal_plugin::PluginFeature::Analyzer,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 0,
                                audio_outputs: 6,
                                midi_inputs: 1,
                                midi_outputs: 0,
                            },
                        ),
                    audio_bus_count: 1,
                    parameter_count: 24,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: false,
                        supports_reset: true,
                        supports_bypass: false,
                        exposes_latency: false,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 2048,
                        sample_accurate_automation: false,
                        accepts_midi: true,
                        accepts_note_events: true,
                        supports_note_expression: true,
                        produces_midi: false,
                        silence_aware: false,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: true,
                        supports_prepare: true,
                        supports_activate: true,
                        supports_reset_while_active: false,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:vst3:multiout-instrument".into(),
                },
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:vst3:bus-fx".into(),
                    plugin_id: "com.signal.bus-fx".into(),
                    vendor: "Signal".into(),
                    name: "Signal Bus FX".into(),
                    format: PluginFormat::Vst3,
                    version: Some("1.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::AudioEffect,
                        signal_plugin::PluginFeature::Utility,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 4,
                        audio_outputs: 4,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 4,
                            audio_outputs: 4,
                            midi_inputs: 0,
                            midi_outputs: 0,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::AudioEffect,
                                signal_plugin::PluginFeature::Utility,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 4,
                                audio_outputs: 4,
                                midi_inputs: 0,
                                midi_outputs: 0,
                            },
                        ),
                    audio_bus_count: 2,
                    parameter_count: 18,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: true,
                        supports_reset: true,
                        supports_bypass: true,
                        exposes_latency: true,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 4096,
                        sample_accurate_automation: true,
                        accepts_midi: false,
                        accepts_note_events: false,
                        supports_note_expression: false,
                        produces_midi: false,
                        silence_aware: true,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: false,
                        supports_prepare: true,
                        supports_activate: true,
                        supports_reset_while_active: true,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:vst3:bus-fx".into(),
                },
            ],
        );
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-chain".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                    },
                ],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-chain".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-chain".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                    },
                ],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-a".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:multiout-instrument".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-b".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:bus-fx".into()),
        });

        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        runtime.record_recovery_cycle(
            "sandbox-b",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(1),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(1),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(2),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-b",
            "lease-b",
            "region-b",
            PluginSandboxTransportStage::Attached,
            Some(2),
            None,
        );

        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-chain".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![
                    PluginNodeRender {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                        output: AudioBuffer::new(
                            SampleRate(48_000),
                            ChannelLayout::Stereo,
                            FrameCount(4),
                        ),
                        latency_samples: 32,
                        tail_samples: 48,
                        bypassed: false,
                    },
                    PluginNodeRender {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                        output: AudioBuffer::new(
                            SampleRate(48_000),
                            ChannelLayout::Stereo,
                            FrameCount(4),
                        ),
                        latency_samples: 16,
                        tail_samples: 24,
                        bypassed: true,
                    },
                ],
            })
            .expect("apply render batch");
        runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process block");

        let snapshot = runtime.get_plugin_chain_snapshot();
        assert_eq!(snapshot.chain_count, 1);
        assert_eq!(snapshot.stage_count, 2);
        assert_eq!(snapshot.compensated_stage_count, 1);
        assert_eq!(snapshot.bypassed_stage_count, 1);
        assert_eq!(snapshot.total_realized_latency_samples, 48);
        assert_eq!(snapshot.total_tail_samples, 72);
        assert_eq!(snapshot.chains[0].chain_id, "track:lead");
        assert_eq!(snapshot.chains[0].stages[0].node_id, "plugin-a");
        assert_eq!(snapshot.chains[0].stages[1].node_id, "plugin-b");
        assert!(
            snapshot.chains[0].stages[0]
                .complex_io_summary
                .has_complex_topology
        );
        assert!(
            snapshot.chains[0].stages[0]
                .complex_io_summary
                .multi_output_instrument
        );
        assert_eq!(
            snapshot.chains[0].stages[0]
                .complex_io_summary
                .instrument_output_group_count,
            2
        );
        assert_eq!(
            snapshot.chains[0].stages[1]
                .complex_io_summary
                .bus_capable_fx_class,
            Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        );
        assert_eq!(
            snapshot.chains[0].stages[1]
                .complex_io_summary
                .secondary_input_group_count,
            1
        );
        let observation = crate::RuntimeObservationReport::capture(
            &runtime,
            &crate::RuntimeEventRecorder::default(),
        );
        assert_eq!(observation.plugin_pin_matrix_snapshot.plugin_type_count, 2);
        assert_eq!(
            observation.plugin_pin_matrix_snapshot.negotiated_type_count,
            2
        );
        assert_eq!(
            observation
                .plugin_pin_matrix_snapshot
                .dynamic_negotiated_type_count,
            2
        );
        let multiout_pin_matrix = observation
            .plugin_pin_matrix_snapshot
            .records
            .iter()
            .find(|record| record.plugin_type_id == "plugin:vst3:multiout-instrument")
            .expect("multi-output pin matrix record should exist");
        assert_eq!(
            multiout_pin_matrix.pin_matrix_posture,
            crate::RuntimePluginPinMatrixPosture::Negotiated
        );
        assert_eq!(
            multiout_pin_matrix.dynamic_bus_negotiation_posture,
            crate::RuntimeDynamicBusNegotiationPosture::Negotiated
        );
        assert!(multiout_pin_matrix
            .pin_group_identities
            .contains(&crate::RuntimePluginPinGroupIdentity::PrimaryProgramPath));
        assert!(multiout_pin_matrix
            .pin_group_identities
            .contains(&crate::RuntimePluginPinGroupIdentity::SecondaryProgramPath));
        let bus_fx_pin_matrix = observation
            .plugin_pin_matrix_snapshot
            .records
            .iter()
            .find(|record| record.plugin_type_id == "plugin:vst3:bus-fx")
            .expect("bus-fx pin matrix record should exist");
        assert_eq!(
            bus_fx_pin_matrix.fallback_outcome,
            crate::RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
        );
        assert!(bus_fx_pin_matrix
            .pin_group_identities
            .contains(&crate::RuntimePluginPinGroupIdentity::SidechainPath));
        assert!(bus_fx_pin_matrix
            .pin_group_identities
            .contains(&crate::RuntimePluginPinGroupIdentity::AuxReturnPath));
        assert_eq!(
            snapshot.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::Compensated
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall_state,
            RuntimePluginRecallState::Warm
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall.state,
            RuntimePluginRecallState::Warm
        );
        assert_eq!(
            snapshot.chains[0].stages[0]
                .recall
                .payload
                .sandbox_id
                .as_deref(),
            Some("sandbox-a")
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall.payload.lifecycle_state,
            Some(RuntimePluginLifecycleState::Ready)
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall.payload.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert_eq!(
            snapshot.chains[0].stages[1].compensation_state,
            RuntimePluginCompensationState::Bypassed
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall_state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall.state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall.payload.recovery_count,
            1
        );
        assert_eq!(snapshot.chains[0].stages[1].recall.payload.restart_count, 1);
        assert_eq!(
            snapshot.chains[0].stages[1]
                .recall
                .payload
                .last_restart_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall.payload.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );

        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(handoff.stage_count, 2);
        assert_eq!(handoff.warm_stage_count, 1);
        assert_eq!(handoff.recovered_stage_count, 1);
        assert_eq!(handoff.unavailable_stage_count, 0);
        assert_eq!(handoff.stages[1].chain_id, "track:lead");
        assert_eq!(handoff.stages[1].node_id, "plugin-b");
        assert_eq!(
            handoff.stages[1].recall_state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(
            handoff.stages[1].recall_payload,
            snapshot.chains[0].stages[1].recall.payload
        );

        let recorder = RuntimeEventRecorder::default();
        let observation = RuntimeObservationReport::capture(&runtime, &recorder);
        let compact = observation.render_compact();
        assert!(compact.contains("plugin_chains=1/2"));
        assert!(compact.contains("plugin_chain_compensated=1"));
        assert!(compact.contains("plugin_chain_bypassed=1"));

        let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("plugin_chain_count=1"));
        assert!(multiline.contains("plugin_chain_0_stage_0=plugin-a"));
        assert!(multiline.contains("plugin_chain_0_stage_1=plugin-b"));
        assert!(multiline.contains("recall=Recovered/sandbox=Some(\"sandbox-b\")"));
        let json = supervisor.render_json();
        assert!(json.contains("\"plugin_chain_snapshot\":{\"chain_count\":1"));
        assert!(json.contains("\"recall\":{\"state\":\"Recovered\""));
        assert!(json.contains("\"payload\":{\"sandbox_id\":\"sandbox-b\""));
        assert!(json.contains("\"last_restart_intent\":\"CrashRecovery\""));
        assert!(json.contains("\"compensation_state\":\"Bypassed\""));
    }

    #[test]
    fn runtime_plugin_chain_snapshot_settles_tail_before_returning_to_pending_render() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-chain-settling".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-chain-settling".into(),
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
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-chain-settling".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-chain-settling".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![PluginNodeRender {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                }],
            })
            .expect("apply render batch");
        runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process first block");

        let compensated = runtime.get_plugin_chain_snapshot();
        assert_eq!(compensated.compensated_stage_count, 1);
        assert_eq!(compensated.settling_stage_count, 0);
        assert_eq!(
            compensated.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::Compensated
        );
        assert_eq!(compensated.chains[0].stages[0].tail_samples, Some(48));

        runtime
            .process_engine_block(
                1,
                2,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process settling block");

        let settling = runtime.get_plugin_chain_snapshot();
        assert_eq!(settling.pending_render_stage_count, 0);
        assert_eq!(settling.settling_stage_count, 1);
        assert_eq!(settling.compensated_stage_count, 0);
        assert_eq!(settling.total_realized_latency_samples, 32);
        assert_eq!(settling.total_tail_samples, 44);
        assert_eq!(
            settling.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::Settling
        );
        assert_eq!(
            settling.chains[0].stages[0].realized_latency_samples,
            Some(32)
        );
        assert_eq!(settling.chains[0].stages[0].tail_samples, Some(44));

        for block_sequence in 3..=13 {
            runtime
                .process_engine_block(
                    1,
                    block_sequence,
                    AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
                )
                .expect("process tail retirement block");
        }

        let pending = runtime.get_plugin_chain_snapshot();
        assert_eq!(pending.pending_render_stage_count, 1);
        assert_eq!(pending.settling_stage_count, 0);
        assert_eq!(pending.compensated_stage_count, 0);
        assert_eq!(pending.total_realized_latency_samples, 0);
        assert_eq!(pending.total_tail_samples, 0);
        assert_eq!(
            pending.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::PendingRender
        );
        assert_eq!(pending.chains[0].stages[0].realized_latency_samples, None);
        assert_eq!(pending.chains[0].stages[0].tail_samples, None);
    }

    #[test]
    fn runtime_plugin_chain_snapshot_tracks_mixed_settling_and_pending_stages_in_multi_stage_chain()
    {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 8,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-c".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                    },
                ],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
                contract_count: 3,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-c".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-c".into(),
                        sandbox_id: "sandbox-c".into(),
                    },
                ],
            })
            .expect("apply bindings");
        for sandbox_id in ["sandbox-a", "sandbox-b", "sandbox-c"] {
            runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::InstancePrepared,
                Some(1),
            );
        }
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-chain-multi-stage-settling".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![
                    PluginNodeRender {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                        output: AudioBuffer::new(
                            SampleRate(48_000),
                            ChannelLayout::Stereo,
                            FrameCount(4),
                        ),
                        latency_samples: 8,
                        tail_samples: 0,
                        bypassed: false,
                    },
                    PluginNodeRender {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                        output: AudioBuffer::new(
                            SampleRate(48_000),
                            ChannelLayout::Stereo,
                            FrameCount(4),
                        ),
                        latency_samples: 16,
                        tail_samples: 16,
                        bypassed: false,
                    },
                    PluginNodeRender {
                        node_id: "plugin-c".into(),
                        sandbox_id: "sandbox-c".into(),
                        output: AudioBuffer::new(
                            SampleRate(48_000),
                            ChannelLayout::Stereo,
                            FrameCount(4),
                        ),
                        latency_samples: 24,
                        tail_samples: 40,
                        bypassed: false,
                    },
                ],
            })
            .expect("apply render batch");
        runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process first block");
        runtime
            .process_engine_block(
                1,
                2,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process settling block");

        let snapshot = runtime.get_plugin_chain_snapshot();
        assert_eq!(snapshot.chain_count, 1);
        assert_eq!(snapshot.stage_count, 3);
        assert_eq!(snapshot.pending_render_stage_count, 1);
        assert_eq!(snapshot.settling_stage_count, 2);
        assert_eq!(snapshot.compensated_stage_count, 0);
        assert_eq!(snapshot.total_realized_latency_samples, 40);
        assert_eq!(snapshot.total_tail_samples, 48);
        assert_eq!(
            snapshot.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::PendingRender
        );
        assert_eq!(
            snapshot.chains[0].stages[1].compensation_state,
            RuntimePluginCompensationState::Settling
        );
        assert_eq!(snapshot.chains[0].stages[1].tail_samples, Some(12));
        assert_eq!(
            snapshot.chains[0].stages[2].compensation_state,
            RuntimePluginCompensationState::Settling
        );
        assert_eq!(snapshot.chains[0].stages[2].tail_samples, Some(36));
    }

    #[test]
    fn runtime_recovery_cycle_invalidates_stale_compensation_for_restarted_sandbox() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-recovery-invalidates-render".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-recovery-invalidates-render".into(),
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
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-recovery-invalidates-render".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-recovery-invalidates-render".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![PluginNodeRender {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                }],
            })
            .expect("apply render batch");
        runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process first block");

        let compensated = runtime.get_plugin_chain_snapshot();
        assert_eq!(
            compensated.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::Compensated
        );

        runtime.record_recovery_cycle(
            "sandbox-a",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(3),
        );

        let recovered = runtime.get_plugin_chain_snapshot();
        assert_eq!(recovered.pending_render_stage_count, 1);
        assert_eq!(recovered.settling_stage_count, 0);
        assert_eq!(recovered.compensated_stage_count, 0);
        assert_eq!(
            recovered.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::PendingRender
        );
        assert_eq!(recovered.chains[0].stages[0].realized_latency_samples, None);
        assert_eq!(recovered.chains[0].stages[0].tail_samples, None);
        assert_eq!(
            recovered.chains[0].stages[0].recall_state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(
            recovered.chains[0].stages[0]
                .recall
                .payload
                .last_restart_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );
    }

    #[test]
    fn runtime_plugin_chain_snapshot_preserves_degraded_and_missing_binding_states() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-chain-degraded".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                    },
                ],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-chain-degraded".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-chain-degraded".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-faulted".into(),
                }],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-faulted",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-faulted",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_fault(
            "sandbox-faulted",
            PluginFaultKind::Crash,
            "sandbox faulted before render",
            Some(2),
        );

        let snapshot = runtime.get_plugin_chain_snapshot();
        assert_eq!(snapshot.chain_count, 1);
        assert_eq!(snapshot.stage_count, 2);
        assert_eq!(snapshot.degraded_stage_count, 1);
        assert_eq!(snapshot.missing_binding_stage_count, 1);
        assert_eq!(
            snapshot.chains[0].stages[0].compensation_state,
            RuntimePluginCompensationState::Degraded
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall_state,
            RuntimePluginRecallState::Unavailable
        );
        assert_eq!(
            snapshot.chains[0].stages[0].recall.payload.lifecycle_state,
            Some(RuntimePluginLifecycleState::Faulted)
        );
        assert_eq!(snapshot.chains[0].stages[0].recall.payload.fault_count, 1);
        assert_eq!(
            snapshot.chains[0].stages[0].recall.payload.last_fault_kind,
            Some(PluginFaultKind::Crash)
        );
        assert_eq!(
            snapshot.chains[0].stages[0]
                .recall
                .payload
                .last_fault_detail
                .as_deref(),
            Some("sandbox faulted before render")
        );
        assert_eq!(
            snapshot.chains[0].stages[1].compensation_state,
            RuntimePluginCompensationState::MissingBinding
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall_state,
            RuntimePluginRecallState::Unbound
        );
        assert_eq!(
            snapshot.chains[0].stages[1].recall.state,
            RuntimePluginRecallState::Unbound
        );
        assert_eq!(snapshot.chains[0].stages[1].recall.payload.sandbox_id, None);

        let unavailable_handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(unavailable_handoff.stage_count, 2);
        assert_eq!(unavailable_handoff.unavailable_stage_count, 1);
        assert_eq!(unavailable_handoff.unbound_stage_count, 1);
        assert_eq!(
            unavailable_handoff.stages[0].recall_payload.lifecycle_state,
            Some(RuntimePluginLifecycleState::Faulted)
        );

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let unavailable_json = supervisor.render_json();
        assert!(unavailable_json.contains("\"recall\":{\"state\":\"Unavailable\""));
        assert!(unavailable_json.contains("\"payload\":{\"sandbox_id\":\"sandbox-faulted\""));
        assert!(unavailable_json.contains("\"lifecycle_state\":\"Faulted\""));

        runtime.record_plugin_sandbox_fault(
            "sandbox-faulted",
            PluginFaultKind::Timeout,
            "sandbox missed heartbeat twice",
            Some(3),
        );

        let quarantined = runtime.get_plugin_chain_snapshot();
        assert_eq!(
            quarantined.chains[0].stages[0].recall.state,
            RuntimePluginRecallState::Unavailable
        );
        assert_eq!(
            quarantined.chains[0].stages[0]
                .recall
                .payload
                .lifecycle_state,
            Some(RuntimePluginLifecycleState::Quarantined)
        );
        assert_eq!(
            quarantined.chains[0].stages[0].recall.payload.fault_count,
            2
        );
        assert_eq!(
            quarantined.chains[0].stages[0]
                .recall
                .payload
                .last_fault_kind,
            Some(PluginFaultKind::Timeout)
        );

        let quarantined_handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(quarantined_handoff.unavailable_stage_count, 1);
        assert_eq!(
            quarantined_handoff.stages[0].recall_payload.lifecycle_state,
            Some(RuntimePluginLifecycleState::Quarantined)
        );

        let quarantined_supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let quarantined_multiline = quarantined_supervisor.render_multiline();
        assert!(
            quarantined_multiline.contains("recall=Unavailable/sandbox=Some(\"sandbox-faulted\")")
        );
        let quarantined_json = quarantined_supervisor.render_json();
        assert!(quarantined_json.contains("\"lifecycle_state\":\"Quarantined\""));
    }

    #[test]
    fn runtime_execution_topology_summary_clears_stale_plugin_chain_state_on_rebind_and_refresh() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-rebind-refresh".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-rebind-refresh".into(),
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
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-rebind-refresh".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-rebind-refresh".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![PluginNodeRender {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(4),
                    ),
                    latency_samples: 32,
                    tail_samples: 48,
                    bypassed: false,
                }],
            })
            .expect("apply render batch");
        runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process first block");

        let realized = runtime.get_execution_topology_summary();
        assert_eq!(realized.plugin_chain.total_realized_latency_samples, 32);
        assert_eq!(
            realized.nodes[0].plugin_compensation_state,
            Some(RuntimePluginCompensationState::Compensated)
        );
        assert_eq!(
            realized.nodes[0].plugin_recall_state,
            Some(RuntimePluginRecallState::Warm)
        );
        assert_eq!(
            realized.nodes[0]
                .plugin_recall
                .as_ref()
                .map(|recall| recall.state),
            Some(RuntimePluginRecallState::Warm)
        );
        assert_eq!(
            realized.nodes[0]
                .plugin_recall
                .as_ref()
                .and_then(|recall| recall.payload.sandbox_id.as_deref()),
            Some("sandbox-a")
        );

        let realized_handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(realized_handoff.stage_count, 1);
        assert_eq!(
            realized_handoff.stages[0]
                .recall_payload
                .sandbox_id
                .as_deref(),
            Some("sandbox-a")
        );
        assert_eq!(realized.nodes[0].plugin_realized_latency_samples, Some(32));
        assert_eq!(realized.nodes[0].plugin_tail_samples, Some(48));

        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-rebind-refresh".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-b".into(),
                }],
            })
            .expect("rebind plugin");

        let rebound = runtime.get_execution_topology_summary();
        assert_eq!(rebound.plugin_chain.total_realized_latency_samples, 0);
        assert_eq!(rebound.plugin_chain.pending_render_stage_count, 1);
        assert_eq!(
            rebound.nodes[0].plugin_compensation_state,
            Some(RuntimePluginCompensationState::PendingRender)
        );
        assert_eq!(
            rebound.nodes[0].plugin_recall_state,
            Some(RuntimePluginRecallState::Cold)
        );
        assert_eq!(
            rebound.nodes[0]
                .plugin_recall
                .as_ref()
                .map(|recall| recall.state),
            Some(RuntimePluginRecallState::Cold)
        );
        assert_eq!(
            rebound.nodes[0]
                .plugin_recall
                .as_ref()
                .and_then(|recall| recall.payload.sandbox_id.as_deref()),
            Some("sandbox-b")
        );
        assert_eq!(
            rebound.nodes[0]
                .plugin_recall
                .as_ref()
                .and_then(|recall| recall.payload.lifecycle_state),
            None
        );

        let rebound_handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(rebound_handoff.stage_count, 1);
        assert_eq!(rebound_handoff.cold_stage_count, 1);
        assert_eq!(
            rebound_handoff.stages[0]
                .recall_payload
                .sandbox_id
                .as_deref(),
            Some("sandbox-b")
        );
        assert_eq!(rebound.nodes[0].plugin_realized_latency_samples, None);
        assert_eq!(rebound.nodes[0].plugin_tail_samples, None);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:utility-refresh".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "utility".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                }],
            })
            .expect("apply refreshed graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:utility-refresh".into(),
                contract_count: 1,
                nodes: vec![GraphNodeContractProjection {
                    node_id: "utility".into(),
                    buffer_contract: GraphNodeBufferContractProjection::default(),
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::Utility),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                }],
            })
            .expect("apply refreshed contracts");

        let refreshed = runtime.get_execution_topology_summary();
        assert_eq!(refreshed.plugin_chain.chain_count, 0);
        assert_eq!(refreshed.plugin_chain.stage_count, 0);
        assert_eq!(refreshed.track_lanes.len(), 0);
        assert_eq!(refreshed.nodes.len(), 1);
        assert_eq!(refreshed.nodes[0].node_id, "utility");
        assert_eq!(refreshed.nodes[0].plugin_recall_state, None);
        assert_eq!(refreshed.nodes[0].plugin_recall, None);
        assert_eq!(refreshed.nodes[0].plugin_compensation_state, None);
        assert_eq!(refreshed.nodes[0].plugin_realized_latency_samples, None);

        let refreshed_handoff = runtime.get_plugin_recall_handoff_snapshot();
        assert_eq!(refreshed_handoff.stage_count, 0);
    }

    #[test]
    fn runtime_plugin_recall_handoff_snapshot_resolves_consumer_selection_without_export_parsing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-recall-selection".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                    },
                ],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-recall-selection".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-recall-selection".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                    },
                ],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_recovery_cycle(
            "sandbox-b",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(3),
        );

        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: 2,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };

        let resolved = handoff
            .resolve_selection(&selection)
            .expect("resolve recall handoff selection");
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].stage_id, selection.stage_ids[0]);
        assert_eq!(resolved[0].recall_payload, handoff.stages[0].recall_payload);
        assert_eq!(resolved[1].stage_id, selection.stage_ids[1]);
        assert_eq!(
            resolved[1].recall_state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(
            resolved[1].recall_payload.last_restart_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );

        let mut missing_selection = selection.clone();
        missing_selection
            .stage_ids
            .push(crate::interfaces::RuntimePluginRecallHandoffStageId {
                chain_id: "track:lead".into(),
                stage_index: 99,
                node_id: "plugin-missing".into(),
            });
        missing_selection.stage_count = missing_selection.stage_ids.len();
        assert!(handoff.resolve_selection(&missing_selection).is_none());
    }

    #[test]
    fn runtime_plugin_discovery_snapshot_and_reports_surface_typed_scan_filters() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure(&mut runtime);
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

        let first_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        });
        let second_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec![
                "~/Library/Audio/Plug-Ins".into(),
                "/Library/Audio/Plug-Ins".into(),
            ],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            second_handle,
            vec![
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:clap:default".into(),
                    plugin_id: "com.signal.default".into(),
                    vendor: "Signal".into(),
                    name: "Signal Default".into(),
                    format: PluginFormat::Clap,
                    version: Some("1.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::AudioEffect,
                        signal_plugin::PluginFeature::Utility,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 1,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 2,
                            audio_outputs: 2,
                            midi_inputs: 1,
                            midi_outputs: 1,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::AudioEffect,
                                signal_plugin::PluginFeature::Utility,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 2,
                                audio_outputs: 2,
                                midi_inputs: 1,
                                midi_outputs: 1,
                            },
                        ),
                    audio_bus_count: 2,
                    parameter_count: 16,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: true,
                        supports_reset: true,
                        supports_bypass: true,
                        exposes_latency: true,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 4_096,
                        sample_accurate_automation: true,
                        accepts_midi: true,
                        accepts_note_events: true,
                        supports_note_expression: true,
                        produces_midi: true,
                        silence_aware: true,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: false,
                        supports_prepare: true,
                        supports_activate: true,
                        supports_reset_while_active: true,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:clap:default plugin_id=com.signal.default format=Clap features=2 io=PluginIoLayout { audio_inputs: 2, audio_outputs: 2, midi_inputs: 1, midi_outputs: 1 } parameters=16".into(),
                },
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:vst3:instrument".into(),
                    plugin_id: "com.signal.instrument".into(),
                    vendor: "Signal".into(),
                    name: "Signal Instrument".into(),
                    format: PluginFormat::Vst3,
                    version: Some("2.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::Instrument,
                        signal_plugin::PluginFeature::Analyzer,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 0,
                            audio_outputs: 2,
                            midi_inputs: 1,
                            midi_outputs: 0,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::Instrument,
                                signal_plugin::PluginFeature::Analyzer,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 0,
                                audio_outputs: 2,
                                midi_inputs: 1,
                                midi_outputs: 0,
                            },
                        ),
                    audio_bus_count: 1,
                    parameter_count: 24,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: false,
                        supports_reset: true,
                        supports_bypass: false,
                        exposes_latency: false,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 2_048,
                        sample_accurate_automation: false,
                        accepts_midi: true,
                        accepts_note_events: true,
                        supports_note_expression: true,
                        produces_midi: false,
                        silence_aware: false,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: true,
                        supports_prepare: true,
                        supports_activate: false,
                        supports_reset_while_active: false,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:vst3:instrument plugin_id=com.signal.instrument format=Vst3 features=2 io=PluginIoLayout { audio_inputs: 0, audio_outputs: 2, midi_inputs: 1, midi_outputs: 0 } parameters=24".into(),
                },
            ],
        );
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-a".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: None,
        });
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );

        let discovery = runtime.get_plugin_discovery_snapshot();
        assert_eq!(first_handle.0, 1);
        assert_eq!(second_handle.0, 2);
        assert_eq!(discovery.scan_count, 2);
        assert_eq!(discovery.format_filtered_scan_count, 2);
        let last_scan = discovery.last_scan.expect("last scan receipt should exist");
        assert_eq!(last_scan.scan_handle, second_handle);
        assert_eq!(
            last_scan.formats,
            vec![PluginFormat::Clap, PluginFormat::Vst3]
        );
        assert_eq!(last_scan.targeted_format_count, 2);
        assert_eq!(last_scan.discovered_type_count, 2);
        assert_eq!(last_scan.discovered_format_count, 2);
        assert_eq!(last_scan.format_coverage.len(), 2);
        assert_eq!(last_scan.parity_coverage.len(), 3);
        assert!(last_scan.capability_coverage.multi_format_catalog);
        assert_eq!(last_scan.capability_coverage.supports_snapshot_count, 1);
        assert_eq!(last_scan.capability_coverage.supports_activate_count, 1);
        assert_eq!(discovery.discovered_type_count, 2);
        assert_eq!(discovery.discovered_format_count, 2);
        assert_eq!(discovery.format_coverage.len(), 2);
        assert_eq!(discovery.parity_coverage.len(), 3);
        assert_eq!(discovery.capability_coverage.instrument_count, 1);
        assert_eq!(discovery.capability_coverage.audio_effect_count, 1);
        assert_eq!(
            discovery
                .capability_coverage
                .requires_main_thread_for_state_count,
            1
        );
        assert_eq!(discovery.capability_coverage.max_parameter_count, 24);
        assert_eq!(discovery.discovered_types.len(), 2);
        let discovered_type = &discovery.discovered_types[0];
        assert_eq!(discovered_type.plugin_type_id, "plugin:clap:default");
        assert_eq!(discovered_type.plugin_id, "com.signal.default");
        assert_eq!(discovered_type.format, PluginFormat::Clap);
        assert_eq!(
            discovered_type.features,
            vec![
                signal_plugin::PluginFeature::AudioEffect,
                signal_plugin::PluginFeature::Utility,
            ]
        );
        assert_eq!(discovered_type.audio_bus_count, 2);
        assert_eq!(discovered_type.parameter_count, 16);
        assert_eq!(
            discovered_type
                .default_multichannel_io
                .input_layout
                .canonical_layout,
            Some(crate::RuntimeCanonicalChannelLayout::Stereo)
        );
        assert_eq!(
            discovered_type
                .default_multichannel_io
                .output_layout
                .canonical_layout,
            Some(crate::RuntimeCanonicalChannelLayout::Stereo)
        );
        assert!(discovered_type.state_contract.supports_snapshot);
        assert!(discovered_type.processing_contract.produces_midi);
        assert!(discovered_type.lifecycle_contract.supports_activate);
        let clap_parity = discovery
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Clap)
            .expect("clap parity should be present");
        assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
        assert_eq!(
            clap_parity.linux_parity_band,
            RuntimePluginParityBand::Portable
        );
        assert_eq!(
            clap_parity.supported_platforms,
            vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert!(clap_parity.linux_supported);
        assert_eq!(
            clap_parity.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(clap_parity.linux_strict_sandbox_default);
        assert_eq!(clap_parity.discovered_type_count, 1);
        assert_eq!(clap_parity.prepare_capable_type_count, 1);
        assert_eq!(clap_parity.activate_capable_type_count, 1);
        assert_eq!(clap_parity.sandbox_count, 1);
        assert_eq!(clap_parity.in_process_sandbox_count, 0);
        assert_eq!(clap_parity.explicit_placement_rule_count, 0);
        let au_parity = discovery
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Au)
            .expect("au parity should be present even before discovery");
        assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
        assert_eq!(
            au_parity.linux_parity_band,
            RuntimePluginParityBand::Unsupported
        );
        assert!(!au_parity.linux_supported);
        assert_eq!(au_parity.linux_preferred_sandbox_outcome, None);
        assert!(!au_parity.linux_strict_sandbox_default);
        assert_eq!(
            au_parity.unsupported_platforms,
            vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert_eq!(au_parity.discovered_type_count, 0);

        let lifecycle = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(lifecycle.sandbox_count, 1);
        assert_eq!(
            lifecycle.sandboxes[0].plugin_format,
            Some(PluginFormat::Clap)
        );
        assert_eq!(lifecycle.parity_coverage.len(), 3);
        assert_eq!(
            lifecycle
                .parity_coverage
                .iter()
                .find(|record| record.format == PluginFormat::Clap)
                .map(|record| record.active_transport_count),
            Some(0)
        );

        let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(report.plugin_discovery_snapshot.scan_count, 2);
        assert_eq!(report.plugin_discovery_snapshot.discovered_type_count, 2);
        assert_eq!(report.plugin_discovery_snapshot.discovered_format_count, 2);
        assert!(report
            .render_json()
            .contains("\"plugin_discovery_snapshot\":{"));
        assert!(report
            .render_json()
            .contains("\"formats\":[\"Clap\",\"Vst3\"]"));
        assert!(report.render_json().contains("\"discovered_type_count\":2"));
        assert!(report
            .render_json()
            .contains("\"discovered_format_count\":2"));
        assert!(report
            .render_json()
            .contains("\"plugin_type_id\":\"plugin:clap:default\""));
        assert!(report
            .render_json()
            .contains("\"default_multichannel_io\":{"));
        assert!(report
            .render_json()
            .contains("\"plugin_type_id\":\"plugin:vst3:instrument\""));
        assert!(report
            .render_json()
            .contains("\"multi_format_catalog\":true"));
        assert!(report
            .render_json()
            .contains("\"supports_activate_count\":1"));
        assert!(report.render_json().contains("\"format_coverage\":["));
        assert!(report.render_json().contains("\"parity_coverage\":["));
        assert!(report
            .render_json()
            .contains("\"parity_band\":\"Portable\""));
        assert!(report
            .render_json()
            .contains("\"linux_parity_band\":\"Portable\""));
        assert!(report
            .render_json()
            .contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
        assert!(report
            .render_json()
            .contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
        assert!(report.render_json().contains("\"supports_snapshot\":true"));
    }

    #[test]
    fn runtime_linux_plugin_parity_coverage_tracks_policy_render_failure_and_restart_receipts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure(&mut runtime);
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
                linux_preferred_sandbox_outcome: Some(
                    RuntimePluginIsolationOutcome::IsolatedSandbox,
                ),
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
                linux_preferred_sandbox_outcome: Some(
                    RuntimePluginIsolationOutcome::IsolatedSandbox,
                ),
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
                linux_preferred_sandbox_outcome: Some(
                    RuntimePluginIsolationOutcome::IsolatedSandbox,
                ),
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
                        rule_id: "share-clap-linux".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(
                            PluginFormat::Clap,
                        ),
                        outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                        sandbox_group_key: Some("linux:clap".into()),
                    },
                    RuntimePluginPlacementRule {
                        rule_id: "inline-vst3-linux".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(
                            PluginFormat::Vst3,
                        ),
                        outcome: RuntimePluginIsolationOutcome::InProcess,
                        sandbox_group_key: None,
                    },
                ],
            })
            .expect("apply linux placement policy");

        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/.clap".into(), "~/.vst3".into(), "~/.lv2".into()],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
        });
        let sample_record =
            |plugin_type_id: &str,
             format: PluginFormat,
             features: Vec<PluginFeature>,
             io: PluginIoLayout,
             supports_prepare: bool,
             supports_activate: bool| crate::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: plugin_type_id.into(),
                plugin_id: format!("com.signal.{}", plugin_type_id.replace(':', ".")),
                vendor: "Signal".into(),
                name: plugin_type_id.into(),
                format,
                version: Some("1.0.0".into()),
                features: features.clone(),
                default_io_layout: io,
                default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(io),
                complex_io_summary:
                    crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                        &features, io,
                    ),
                audio_bus_count: 1,
                parameter_count: 8,
                state_contract: PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                },
                processing_contract: PluginProcessingContract {
                    max_block_frames: 2048,
                    sample_accurate_automation: true,
                    accepts_midi: io.midi_inputs > 0,
                    accepts_note_events: io.midi_inputs > 0,
                    supports_note_expression: io.midi_inputs > 0,
                    produces_midi: io.midi_outputs > 0,
                    silence_aware: true,
                },
                lifecycle_contract: PluginLifecycleContract {
                    requires_main_thread_for_state: false,
                    supports_prepare,
                    supports_activate,
                    supports_reset_while_active: supports_activate,
                },
                lv2_extension_capabilities: (format == PluginFormat::Lv2).then(|| {
                    crate::RuntimeLv2ExtensionCapabilitySummary::from_lv2_feature_uris(
                        &["http://lv2plug.in/ns/ext/urid#map".into()],
                        &["http://lv2plug.in/ns/ext/patch#Message".into()],
                    )
                }),
                summary: format!("plugin_type={plugin_type_id} format={format:?}"),
            };
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_record(
                    "plugin:clap:linux-parity",
                    PluginFormat::Clap,
                    vec![PluginFeature::AudioEffect],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                    true,
                    true,
                ),
                sample_record(
                    "plugin:vst3:linux-parity",
                    PluginFormat::Vst3,
                    vec![PluginFeature::Instrument],
                    PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                    true,
                    true,
                ),
                sample_record(
                    "plugin:lv2:linux-parity",
                    PluginFormat::Lv2,
                    vec![PluginFeature::Utility],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                    true,
                    true,
                ),
            ],
        );

        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "linux-clap-sandbox".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("plugin:clap:linux-parity".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "linux-clap-sandbox",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "linux-clap-sandbox",
            "lease-clap",
            "region-clap",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "linux-vst3-sandbox".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:linux-parity".into()),
        });
        runtime.record_recovery_cycle(
            "linux-vst3-sandbox",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "linux-vst3-sandbox",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(2),
        );

        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "linux-lv2-sandbox".into(),
            plugin_format: PluginFormat::Lv2,
            plugin_type_id: Some("plugin:lv2:linux-parity".into()),
        });
        runtime.record_plugin_sandbox_fault(
            "linux-lv2-sandbox",
            PluginFaultKind::Crash,
            "linux lv2 sandbox fault",
            Some(3),
        );

        let lifecycle = runtime.get_plugin_lifecycle_snapshot();
        let clap = lifecycle
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Clap)
            .expect("clap linux parity should be present");
        assert_eq!(clap.linux_parity_band, RuntimePluginParityBand::Portable);
        assert!(clap.linux_supported);
        assert_eq!(
            clap.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(clap.linux_strict_sandbox_default);
        assert_eq!(clap.prepare_capable_type_count, 1);
        assert_eq!(clap.activate_capable_type_count, 1);
        assert_eq!(clap.shared_sandbox_count, 1);
        assert_eq!(clap.active_transport_count, 1);

        let vst3 = lifecycle
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Vst3)
            .expect("vst3 linux parity should be present");
        assert_eq!(vst3.linux_parity_band, RuntimePluginParityBand::Portable);
        assert!(vst3.linux_supported);
        assert_eq!(vst3.in_process_sandbox_count, 1);
        assert_eq!(vst3.restarting_sandbox_count, 1);
        assert_eq!(vst3.rebindable_sandbox_count, 1);
        assert_eq!(vst3.prepare_capable_type_count, 1);

        let lv2 = lifecycle
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Lv2)
            .expect("lv2 linux parity should be present");
        assert_eq!(lv2.parity_band, RuntimePluginParityBand::Guarded);
        assert_eq!(lv2.linux_parity_band, RuntimePluginParityBand::Portable);
        assert!(lv2.linux_supported);
        assert_eq!(lv2.faulted_sandbox_count, 1);
        assert_eq!(
            lv2.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.lv2_extension_snapshot.plugin_type_count, 1);
        assert_eq!(
            observation
                .lv2_extension_snapshot
                .worker_required_type_count,
            0
        );
        assert_eq!(
            observation
                .lv2_extension_snapshot
                .patch_supported_type_count,
            0
        );
        assert_eq!(observation.lv2_extension_snapshot.unavailable_type_count, 1);
        let lv2_extension = observation
            .lv2_extension_snapshot
            .records
            .iter()
            .find(|record| record.plugin_type_id == "plugin:lv2:linux-parity")
            .expect("lv2 extension snapshot should be present");
        assert_eq!(
            lv2_extension.worker_posture,
            crate::RuntimeLv2WorkerPosture::WorkerAbsent
        );
        assert_eq!(
            lv2_extension.urid_negotiation_posture,
            crate::RuntimeLv2UridNegotiationPosture::Unavailable
        );
        assert_eq!(
            lv2_extension.patch_exchange_posture,
            crate::RuntimeLv2PatchExchangePosture::Unavailable
        );
        assert_eq!(
            lv2_extension.extension_negotiation_state,
            crate::RuntimeLv2ExtensionNegotiationState::Unavailable
        );

        let rendered = observation.render_json();
        assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
        assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
        assert!(rendered.contains("\"linux_strict_sandbox_default\":true"));
        assert!(rendered.contains("\"restarting_sandbox_count\":1"));
        assert!(rendered.contains("\"faulted_sandbox_count\":1"));
        assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
        assert!(rendered.contains("\"urid_negotiation_posture\":\"Unavailable\""));
    }

    #[test]
    fn runtime_offline_render_contract_preview_reuses_runtime_topology_tempo_clip_and_recall_contracts(
    ) {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
            formats: vec![PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:vst3:multiout-instrument".into(),
                    plugin_id: "com.signal.multiout".into(),
                    vendor: "Signal".into(),
                    name: "Signal Multi Output Instrument".into(),
                    format: PluginFormat::Vst3,
                    version: Some("1.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::Instrument,
                        signal_plugin::PluginFeature::Analyzer,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 6,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 0,
                            audio_outputs: 6,
                            midi_inputs: 1,
                            midi_outputs: 0,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::Instrument,
                                signal_plugin::PluginFeature::Analyzer,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 0,
                                audio_outputs: 6,
                                midi_inputs: 1,
                                midi_outputs: 0,
                            },
                        ),
                    audio_bus_count: 1,
                    parameter_count: 24,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: false,
                        supports_reset: true,
                        supports_bypass: false,
                        exposes_latency: false,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 2048,
                        sample_accurate_automation: false,
                        accepts_midi: true,
                        accepts_note_events: true,
                        supports_note_expression: true,
                        produces_midi: false,
                        silence_aware: false,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: true,
                        supports_prepare: true,
                        supports_activate: true,
                        supports_reset_while_active: false,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:vst3:multiout-instrument".into(),
                },
                crate::RuntimePluginDiscoveredTypeRecord {
                    plugin_type_id: "plugin:vst3:bus-fx".into(),
                    plugin_id: "com.signal.bus-fx".into(),
                    vendor: "Signal".into(),
                    name: "Signal Bus FX".into(),
                    format: PluginFormat::Vst3,
                    version: Some("1.0.0".into()),
                    features: vec![
                        signal_plugin::PluginFeature::AudioEffect,
                        signal_plugin::PluginFeature::Utility,
                    ],
                    default_io_layout: signal_plugin::PluginIoLayout {
                        audio_inputs: 4,
                        audio_outputs: 4,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                    default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 4,
                            audio_outputs: 4,
                            midi_inputs: 0,
                            midi_outputs: 0,
                        },
                    ),
                    complex_io_summary:
                        crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                            &[
                                signal_plugin::PluginFeature::AudioEffect,
                                signal_plugin::PluginFeature::Utility,
                            ],
                            signal_plugin::PluginIoLayout {
                                audio_inputs: 4,
                                audio_outputs: 4,
                                midi_inputs: 0,
                                midi_outputs: 0,
                            },
                        ),
                    audio_bus_count: 2,
                    parameter_count: 18,
                    state_contract: signal_plugin::PluginStateContract {
                        supports_snapshot: true,
                        supports_reset: true,
                        supports_bypass: true,
                        exposes_latency: true,
                        exposes_tail: true,
                    },
                    processing_contract: signal_plugin::PluginProcessingContract {
                        max_block_frames: 4096,
                        sample_accurate_automation: true,
                        accepts_midi: false,
                        accepts_note_events: false,
                        supports_note_expression: false,
                        produces_midi: false,
                        silence_aware: true,
                    },
                    lifecycle_contract: signal_plugin::PluginLifecycleContract {
                        requires_main_thread_for_state: false,
                        supports_prepare: true,
                        supports_activate: true,
                        supports_reset_while_active: true,
                    },
                    lv2_extension_capabilities: None,
                    summary: "plugin_type=plugin:vst3:bus-fx".into(),
                },
            ],
        );
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:offline-render-preview".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                    },
                ],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:offline-render-preview".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:offline-render-preview".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-a".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-b".into(),
                    },
                ],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-a".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:multiout-instrument".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-b".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:bus-fx".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_recovery_cycle(
            "sandbox-b",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(2),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-b",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(3),
        );
        runtime
            .apply_tempo_map_projection(RuntimeTempoMapProjection {
                segment_count: 1,
                segments: vec![crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:offline-render".into(),
                    start_samples: 0,
                    end_samples: Some(48_000),
                    start_tempo_bpm: 132.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                }],
            })
            .expect("apply tempo map");
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 24_000,
                tempo_bpm: 90.0,
                loop_state: None,
            })
            .expect("apply transport");
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:offline-render".into(),
                media_asset_id: None,
                warp_mode: RuntimeWarpMode::Off,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                clip_gain: RuntimeClipGainEnvelope::default(),
            }])
            .expect("reconcile clip processing");

        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: 2,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };
        let request = RuntimeOfflineRenderRequest {
            request_id: "render:preview".into(),
            timeline_start_samples: 0,
            duration_samples: 48_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: vec![RuntimeOfflineRenderStemTarget {
                stem_id: "stem:track:lead".into(),
                target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                target_id: Some("track:lead".into()),
            }],
            freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                artifact_id: "freeze:track:lead".into(),
                source_stem_id: "stem:track:lead".into(),
                recall_selection: selection.clone(),
            }],
        };

        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &request,
            &runtime.get_execution_topology_summary(),
            &runtime.get_clip_processing_pipeline_snapshot(),
            &runtime.get_media_pipeline_snapshot(),
            &runtime.get_tempo_map_snapshot(),
            &runtime.get_marker_analysis_snapshot(),
            &handoff,
        )
        .expect("build offline render contract preview");

        assert_eq!(preview.request_id, "render:preview");
        assert_eq!(preview.timeline_end_samples, 48_000);
        assert_eq!(preview.export_sample_rate_hz, 48_000);
        assert_eq!(preview.clip_count, 1);
        assert_eq!(preview.ready_clip_count, 1);
        assert_eq!(preview.stem_count, 1);
        assert_eq!(preview.freeze_artifact_count, 1);
        assert_eq!(preview.resolved_tempo_bpm, 132.0);
        assert_eq!(
            preview.resolved_tempo_source,
            RuntimeTempoSource::TempoMapSegment
        );
        assert_eq!(preview.stem_targets[0].stem_id, "stem:track:lead");
        assert_eq!(
            preview.stem_targets[0].target_kind,
            RuntimeOfflineRenderTargetKind::TrackLane
        );
        assert_eq!(
            preview.stem_targets[0].target_id.as_deref(),
            Some("track:lead")
        );
        assert_eq!(
            preview.stem_targets[0].resolved_node_ids,
            vec!["plugin-a".to_string(), "plugin-b".to_string()]
        );
        assert_eq!(preview.freeze_artifacts[0].artifact_id, "freeze:track:lead");
        assert_eq!(preview.freeze_artifacts[0].recall_stage_count, 2);
        assert_eq!(
            preview.freeze_artifacts[0].recall_stage_ids,
            selection.stage_ids
        );
        assert_eq!(
            preview.freeze_artifacts[0].recall_states,
            vec![
                RuntimePluginRecallState::Warm,
                RuntimePluginRecallState::Recovered
            ]
        );
        assert_eq!(preview.chain_contract.chain_count, 1);
        assert_eq!(preview.chain_contract.stage_count, 2);
        assert_eq!(preview.chain_contract.pending_render_stage_count, 2);
        assert_eq!(preview.chain_contract.settling_stage_count, 0);
        assert_eq!(preview.chain_contract.compensated_stage_count, 0);
        assert_eq!(preview.chain_contract.total_planned_latency_samples, 36);
        assert_eq!(preview.chain_contract.total_realized_latency_samples, 0);
        assert_eq!(preview.chain_contract.total_tail_samples, 0);
        assert_eq!(preview.chain_contract.complex_io_stage_count, 2);
        assert_eq!(
            preview.chain_contract.multi_output_instrument_stage_count,
            1
        );
        assert_eq!(preview.chain_contract.bus_capable_fx_stage_count, 1);
        assert_eq!(preview.chain_contract.sidechain_capable_fx_stage_count, 1);
        assert_eq!(preview.chain_contract.recall_stage_count, 2);
        assert_eq!(preview.chain_contract.warm_recall_stage_count, 1);
        assert_eq!(preview.chain_contract.recovered_recall_stage_count, 1);
        assert_eq!(preview.chain_contract.cold_recall_stage_count, 0);
        assert_eq!(preview.chain_contract.unavailable_recall_stage_count, 0);
        assert_eq!(preview.chain_contract.complex_io_stages.len(), 2);
        assert_eq!(
            preview.chain_contract.complex_io_stages[0].plugin_type_id,
            Some("plugin:vst3:multiout-instrument".to_string())
        );
        assert!(
            preview.chain_contract.complex_io_stages[0]
                .topology
                .multi_output_instrument
        );
        assert_eq!(
            preview.chain_contract.complex_io_stages[0]
                .topology
                .instrument_output_group_count,
            2
        );
        assert_eq!(
            preview.chain_contract.complex_io_stages[1]
                .topology
                .bus_capable_fx_class,
            Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        );
        assert!(preview.chain_contract.summary.contains("pending=2"));
        assert!(preview
            .chain_contract
            .summary
            .contains("complex_io_stages=2"));
        assert!(preview.chain_contract.summary.contains("recall=2/"));
        assert!(preview.summary.contains("stems=1"));
        assert!(preview.summary.contains("freeze_artifacts=1"));
        assert!(preview.summary.contains("chain_contract=chains=1"));
    }

    #[test]
    fn runtime_offline_render_contract_preview_rejects_misaligned_chain_and_recall_contracts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
                contract_count: 1,
                nodes: vec![GraphNodeContractProjection {
                    node_id: "plugin-a".into(),
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
            .expect("apply graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply bindings");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );

        let mut handoff = runtime.get_plugin_recall_handoff_snapshot();
        handoff.stage_count = 0;
        handoff.stages.clear();
        handoff.summary = "stages=0".into();
        let request = RuntimeOfflineRenderRequest {
            request_id: "render:misaligned".into(),
            timeline_start_samples: 0,
            duration_samples: 48_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        };

        let error = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &request,
            &runtime.get_execution_topology_summary(),
            &runtime.get_clip_processing_pipeline_snapshot(),
            &runtime.get_media_pipeline_snapshot(),
            &runtime.get_tempo_map_snapshot(),
            &runtime.get_marker_analysis_snapshot(),
            &handoff,
        )
        .expect_err("misaligned chain and recall contracts should fail");
        assert_eq!(error.kind, RuntimeErrorKind::InvalidState);
        assert!(error
            .message
            .contains("aligned plugin chain and recall handoff"));
    }

    #[test]
    fn runtime_offline_render_contract_preview_carries_sidechain_dependency_receipts() {
        let runtime = prepare_sidechain_runtime();
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let request = RuntimeOfflineRenderRequest {
            request_id: "render:sidechain-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        };

        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &request,
            &runtime.get_execution_topology_summary(),
            &runtime.get_clip_processing_pipeline_snapshot(),
            &runtime.get_media_pipeline_snapshot(),
            &runtime.get_tempo_map_snapshot(),
            &runtime.get_marker_analysis_snapshot(),
            &handoff,
        )
        .expect("build offline render sidechain preview");

        assert_eq!(preview.chain_contract.secondary_input_count, 1);
        assert_eq!(preview.chain_contract.required_secondary_input_count, 1);
        assert_eq!(preview.chain_contract.optional_secondary_input_count, 0);
        assert_eq!(preview.chain_contract.disabled_secondary_input_count, 0);
        assert_eq!(
            preview
                .chain_contract
                .terminal_fallback_secondary_input_count,
            0
        );
        assert_eq!(preview.chain_contract.bus_connection_count, 2);
        assert_eq!(preview.chain_contract.auxiliary_path_count, 1);
        let route = &preview.chain_contract.secondary_inputs[0];
        assert_eq!(route.source_id, "sidechain-feed");
        assert_eq!(
            route.target_kind,
            RuntimeSecondaryInputTargetKind::RenderInput
        );
        assert_eq!(route.target_id, "offline-render");
        assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
        assert_eq!(
            route.fallback_outcome,
            crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
        );
        assert!(preview
            .chain_contract
            .bus_connections
            .iter()
            .any(|connection| {
                connection.connection_id
                    == "track-input:bus:track:lead->plugin-compressor:bus:track:lead"
                    && connection.source_bus_role == crate::RuntimeBusRole::ProgramMain
                    && connection.target_bus_role == crate::RuntimeBusRole::ProgramMain
            }));
        assert!(preview.chain_contract.auxiliary_paths.iter().any(|path| {
            path.auxiliary_path_id == "bus_group:mix:tracks"
                && path.path_kind == crate::RuntimeAuxiliaryPathKind::Submix
        }));
        assert!(preview
            .chain_contract
            .summary
            .contains("secondary_inputs=1"));
        assert!(preview
            .chain_contract
            .summary
            .contains("bus_connections=2 auxiliary_paths=1"));
        assert!(preview.summary.contains("chain_contract=chains=1"));
    }

    #[test]
    fn runtime_offline_render_renders_main_mix_stem_and_freeze_from_runtime_owned_state() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();

        let processed_before = runtime.get_engine_block_snapshot().processed_blocks;
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: handoff.stage_count,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };

        let result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:engine-proof".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: selection,
                }],
            })
            .expect("offline render should succeed");

        assert_eq!(
            runtime.get_engine_block_snapshot().processed_blocks,
            processed_before
        );
        assert_eq!(result.rendered_frame_count, 64);
        assert_eq!(result.block_count, 1);
        assert_eq!(result.stems.len(), 1);
        assert_eq!(result.freeze_artifacts.len(), 1);
        assert_eq!(result.manifest.artifact_count, 0);
        assert!(result.manifest.artifacts.is_empty());
        assert!(result.manifest.report.is_none());
        assert!(!result.manifest.materialized);
        assert_eq!(result.manifest.delegated_execution_request.stage_count, 0);
        assert!(result.manifest.delegated_execution_receipt.is_none());
        assert_eq!(result.plugin_execution_boundary.stage_count, 1);
        assert_eq!(
            result
                .plugin_execution_boundary
                .signal_stage_model_stage_count,
            1
        );
        assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 64);
        assert_eq!(result.stems[0].output.frames().0, 64);
        assert_eq!(
            result.freeze_artifacts[0].recall_states,
            vec![RuntimePluginRecallState::Recovered]
        );
        assert_eq!(
            result.freeze_artifacts[0].output.samples(),
            result.stems[0].output.samples()
        );
        assert_eq!(
            result.main_mix.as_ref().unwrap().samples(),
            result.stems[0].output.samples()
        );
        assert!((result.main_mix_peak_level.unwrap() - 0.5).abs() < 1.0e-6);
        assert!(result.main_mix_rms_level.unwrap() > 0.15);
        assert!(result.main_mix_rms_level.unwrap() < 0.5);
        let rendered = result.main_mix.as_ref().unwrap().samples();
        assert!((rendered[0] + 0.5).abs() < 1.0e-6);
        assert!((rendered[1] + 0.5).abs() < 1.0e-6);
        assert!((rendered[2] + 0.492_187_5).abs() < 1.0e-6);
        assert!(result.summary.contains("stems=1"));
        assert!(result.summary.contains("freeze_artifacts=1"));

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
    fn runtime_offline_render_writes_artifact_receipts_and_resamples_export_rate() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-artifacts");
        let handoff = runtime.get_plugin_recall_handoff_snapshot();

        let result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:artifact-proof".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 24_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: RuntimePluginRecallHandoffSelection {
                        stage_count: handoff.stage_count,
                        stage_ids: handoff
                            .stages
                            .iter()
                            .map(|stage| stage.stage_id.clone())
                            .collect(),
                    },
                }],
            })
            .expect("offline render with artifacts should succeed");

        assert_eq!(result.runtime_frame_count, 64);
        assert_eq!(result.rendered_frame_count, 32);
        assert_eq!(result.main_mix.as_ref().unwrap().sample_rate().0, 24_000);
        assert_eq!(result.main_mix.as_ref().unwrap().frames().0, 32);
        assert_eq!(result.stems[0].output.sample_rate().0, 24_000);
        assert_eq!(result.freeze_artifacts[0].output.sample_rate().0, 24_000);
        assert_eq!(result.manifest.artifact_count, 3);
        assert!(result.manifest.materialized);
        assert_eq!(result.manifest.delegated_execution_request.stage_count, 0);
        assert!(result.manifest.delegated_execution_receipt.is_none());
        assert_eq!(
            result.manifest.artifact_root_path.as_deref(),
            Some(
                artifact_dir
                    .to_str()
                    .expect("artifact dir should be valid utf-8")
            )
        );
        assert_eq!(
            result
                .manifest
                .report
                .as_ref()
                .map(|receipt| receipt.artifact_count),
            Some(3)
        );
        assert!(result
            .manifest
            .artifacts
            .iter()
            .all(|receipt| receipt.sample_rate_hz == 24_000));

        let main_mix_receipt = result
            .manifest
            .artifacts
            .iter()
            .find(|receipt| receipt.artifact_kind == RuntimeOfflineRenderArtifactKind::MainMix)
            .expect("main mix receipt should exist");
        let main_mix_reader =
            hound::WavReader::open(&main_mix_receipt.output_path).expect("main mix wav readable");
        assert_eq!(main_mix_reader.spec().sample_rate, 24_000);

        let report_receipt = result
            .manifest
            .report
            .as_ref()
            .expect("report receipt should exist");
        let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
        assert!(report_body.contains("\"artifact_count\":3"));
        assert!(report_body.contains("\"delegated_stage_count\":0"));
        assert!(report_body.contains("\"rendered_frame_count\":32"));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &result.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &result.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_queue_executes_requests_in_order_and_tracks_queue_completion_progress(
    ) {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let first_artifact_dir = temp_artifact_dir("offline-render-queue-first");
        let second_artifact_dir = temp_artifact_dir("offline-render-queue-second");
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: handoff.stage_count,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };

        let queue_result = runtime
            .render_offline_queue(vec![
                RuntimeOfflineRenderRequest {
                    request_id: "render:queue:0001".into(),
                    timeline_start_samples: 0,
                    duration_samples: 64,
                    export_sample_rate_hz: 48_000,
                    include_main_mix: true,
                    artifact_root_path: Some(first_artifact_dir.display().to_string()),
                    stem_targets: vec![RuntimeOfflineRenderStemTarget {
                        stem_id: "stem:track:lead".into(),
                        target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                        target_id: Some("track:lead".into()),
                    }],
                    freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                        artifact_id: "freeze:track:lead".into(),
                        source_stem_id: "stem:track:lead".into(),
                        recall_selection: selection.clone(),
                    }],
                },
                RuntimeOfflineRenderRequest {
                    request_id: "render:queue:0002".into(),
                    timeline_start_samples: 32,
                    duration_samples: 64,
                    export_sample_rate_hz: 24_000,
                    include_main_mix: true,
                    artifact_root_path: Some(second_artifact_dir.display().to_string()),
                    stem_targets: vec![RuntimeOfflineRenderStemTarget {
                        stem_id: "stem:track:lead".into(),
                        target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                        target_id: Some("track:lead".into()),
                    }],
                    freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                        artifact_id: "freeze:track:lead".into(),
                        source_stem_id: "stem:track:lead".into(),
                        recall_selection: selection,
                    }],
                },
            ])
            .expect("offline render queue should succeed");

        assert_eq!(queue_result.queue_count, 2);
        assert_eq!(queue_result.completed_job_count, 2);
        assert_eq!(
            queue_result.orchestration.decision,
            RuntimeDeferredServiceDecision::Run
        );
        assert_eq!(
            queue_result.orchestration.reason,
            RuntimeDeferredServiceReason::Ready
        );
        assert_eq!(
            queue_result.orchestration.priority_band,
            RuntimeDeferredServicePriorityBand::UserVisible
        );
        assert_eq!(queue_result.orchestration.blocking_priority_band, None);
        assert_eq!(queue_result.orchestration.backpressure_source, None);
        assert!(!queue_result.orchestration.starvation_risk);
        assert_eq!(queue_result.orchestration.starved_work_item_count, 0);
        assert_eq!(queue_result.orchestration.cancellation_cause, None);
        assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
        assert_eq!(queue_result.orchestration.admitted_work_item_count, 2);
        assert_eq!(queue_result.orchestration.completed_work_item_count, 2);
        assert_eq!(queue_result.orchestration.deferred_work_item_count, 0);
        assert_eq!(queue_result.progress.len(), 2);
        assert_eq!(queue_result.results.len(), 2);
        assert!(queue_result.deferred_requests.is_empty());
        assert_eq!(queue_result.progress[0].request_id, "render:queue:0001");
        assert_eq!(queue_result.progress[0].queue_index, 0);
        assert_eq!(queue_result.progress[0].completed_job_count, 1);
        assert_eq!(queue_result.progress[0].progress_percent, 50);
        assert_eq!(queue_result.progress[1].request_id, "render:queue:0002");
        assert_eq!(queue_result.progress[1].queue_index, 1);
        assert_eq!(queue_result.progress[1].completed_job_count, 2);
        assert_eq!(queue_result.progress[1].progress_percent, 100);
        assert_eq!(queue_result.results[0].request_id, "render:queue:0001");
        assert_eq!(queue_result.results[1].request_id, "render:queue:0002");
        assert_eq!(
            queue_result.results[0]
                .manifest
                .artifact_root_path
                .as_deref(),
            Some(
                first_artifact_dir
                    .to_str()
                    .expect("first artifact dir should be valid utf-8")
            )
        );
        assert_eq!(
            queue_result.results[1]
                .manifest
                .artifact_root_path
                .as_deref(),
            Some(
                second_artifact_dir
                    .to_str()
                    .expect("second artifact dir should be valid utf-8")
            )
        );
        assert_eq!(queue_result.results[0].manifest.artifact_count, 3);
        assert_eq!(queue_result.results[1].manifest.artifact_count, 3);
        assert!(queue_result.results[0].manifest.report.is_some());
        assert!(queue_result.results[1].manifest.report.is_some());
        assert_eq!(
            queue_result.results[1]
                .main_mix
                .as_ref()
                .expect("second main mix should exist")
                .sample_rate()
                .0,
            24_000
        );
        assert!(queue_result.summary.contains("queue_count=2"));
        assert!(queue_result.summary.contains("completed_job_count=2"));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for result in &queue_result.results {
            for receipt in &result.manifest.artifacts {
                let _ = fs::remove_file(&receipt.output_path);
            }
            if let Some(report_receipt) = &result.manifest.report {
                let _ = fs::remove_file(&report_receipt.report_path);
            }
        }
        let _ = fs::remove_dir(&first_artifact_dir);
        let _ = fs::remove_dir(&second_artifact_dir);
    }

    #[test]
    fn runtime_offline_render_with_checkpoints_reports_runtime_owned_progress_stages() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-checkpoints");
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: handoff.stage_count,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };

        let execution = runtime
            .render_offline_with_checkpoints(RuntimeOfflineRenderRequest {
                request_id: "render:checkpoint:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: selection,
                }],
            })
            .expect("offline render with checkpoints should succeed");

        assert_eq!(execution.request_id, "render:checkpoint:0001");
        assert_eq!(execution.result.request_id, "render:checkpoint:0001");
        assert_eq!(execution.checkpoint_count, execution.checkpoints.len());
        assert!(execution.checkpoint_count >= 4);
        assert_eq!(
            execution
                .checkpoints
                .first()
                .map(|checkpoint| checkpoint.stage),
            Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
        );
        assert!(execution.checkpoints.iter().any(|checkpoint| {
            checkpoint.stage == RuntimeOfflineRenderCheckpointStage::RenderingGraph
                && checkpoint.progress_percent >= 10
                && checkpoint.progress_percent <= 90
        }));
        assert_eq!(
            execution
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.stage),
            Some(RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts)
        );
        assert_eq!(
            execution
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.progress_percent),
            Some(99)
        );
        assert!(execution
            .checkpoints
            .windows(2)
            .all(|window| window[0].checkpoint_index < window[1].checkpoint_index));
        assert_eq!(
            execution
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.checkpoint_count),
            Some(execution.checkpoint_count)
        );
        assert!(execution.summary.contains("checkpoints="));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &execution.result.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &execution.result.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_execution_streams_checkpoints_before_delivery_completion() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-streaming");
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let selection = RuntimePluginRecallHandoffSelection {
            stage_count: handoff.stage_count,
            stage_ids: handoff
                .stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
        };

        let begin = runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:stream:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: selection,
                }],
            })
            .expect("offline render execution should begin");

        assert_eq!(begin.state, RuntimeOfflineRenderExecutionState::Running);
        assert_eq!(begin.emitted_checkpoint_count, 1);
        assert_eq!(
            begin.checkpoint.as_ref().map(|checkpoint| checkpoint.stage),
            Some(RuntimeOfflineRenderCheckpointStage::PreparingInput)
        );
        assert!(!artifact_dir.exists());

        let mut observed_stages = vec![
            begin
                .checkpoint
                .as_ref()
                .expect("begin checkpoint should exist")
                .stage,
        ];
        let mut completed_result = None;
        for _ in 0..32 {
            let receipt = runtime
                .advance_offline_render_execution("render:stream:0001")
                .expect("offline render execution step should succeed");
            if let Some(checkpoint) = receipt.checkpoint.as_ref() {
                observed_stages.push(checkpoint.stage);
                assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Running);
                assert!(!artifact_dir.exists());
            }
            if let Some(result) = receipt.result {
                assert_eq!(receipt.state, RuntimeOfflineRenderExecutionState::Completed);
                completed_result = Some(result);
                break;
            }
        }

        let completed_result = completed_result
            .expect("offline render execution should complete within the step budget");
        assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::RenderingGraph));
        assert!(
            observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::MaterializingOutputs)
        );
        assert!(observed_stages.contains(&RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts));
        assert!(artifact_dir.exists());
        assert_eq!(completed_result.request_id, "render:stream:0001");
        assert!(completed_result.manifest.report.is_some());

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &completed_result.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &completed_result.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_execution_cancels_without_persisted_artifacts() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-cancel");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:cancel:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render execution should begin");
        runtime
            .advance_offline_render_execution("render:cancel:0001")
            .expect("offline render execution should advance");

        let cancelled = runtime
            .cancel_offline_render_execution("render:cancel:0001")
            .expect("offline render execution should cancel");

        assert_eq!(cancelled.request_id, "render:cancel:0001");
        assert!(cancelled.cancelled_after_checkpoint_count >= 1);
        assert!(cancelled.rendered_frame_count > 0);
        assert!(!artifact_dir.exists());
        assert!(runtime
            .advance_offline_render_execution("render:cancel:0001")
            .is_err());

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_execution_pauses_and_resumes_without_early_delivery() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-pause-resume");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:pause:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render execution should begin");
        runtime
            .advance_offline_render_execution("render:pause:0001")
            .expect("offline render execution should advance");

        let paused = runtime
            .pause_offline_render_execution("render:pause:0001")
            .expect("offline render execution should pause");
        assert_eq!(paused.state, RuntimeOfflineRenderExecutionState::Paused);
        assert_eq!(
            paused.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert!(!paused.interruption_rebindable);
        assert!(paused.summary.contains("state=paused"));
        assert!(!artifact_dir.exists());

        let still_paused = runtime
            .advance_offline_render_execution("render:pause:0001")
            .expect("paused offline render execution should not advance");
        assert_eq!(
            still_paused.state,
            RuntimeOfflineRenderExecutionState::Paused
        );
        assert_eq!(
            still_paused.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert!(still_paused.checkpoint.is_none());
        assert!(!artifact_dir.exists());

        let resumed = runtime
            .resume_offline_render_execution("render:pause:0001")
            .expect("offline render execution should resume");
        assert_eq!(resumed.state, RuntimeOfflineRenderExecutionState::Running);
        assert_eq!(resumed.interruption_class, RuntimeInterruptionClass::Steady);

        let mut completed = None;
        for _ in 0..32 {
            let receipt = runtime
                .advance_offline_render_execution("render:pause:0001")
                .expect("resumed offline render execution should advance");
            if let Some(result) = receipt.result {
                completed = Some(result);
                break;
            }
        }
        let completed = completed.expect("paused session should resume to completion");
        assert!(artifact_dir.exists());
        assert!(completed.manifest.report.is_some());

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &completed.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &completed.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_execution_becomes_recoverable_and_resumes_after_interrupt() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-recoverable");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:recover:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render execution should begin");
        runtime
            .advance_offline_render_execution("render:recover:0001")
            .expect("offline render execution should advance");

        let recoverable = runtime
            .interrupt_offline_render_execution(
                "render:recover:0001",
                "runtime restart boundary".to_string(),
            )
            .expect("offline render execution should become recoverable");
        assert_eq!(
            recoverable.state,
            RuntimeOfflineRenderExecutionState::Recoverable
        );
        assert_eq!(
            recoverable.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert!(recoverable.summary.contains("state=recoverable"));
        assert!(!artifact_dir.exists());

        let still_recoverable = runtime
            .advance_offline_render_execution("render:recover:0001")
            .expect("recoverable execution should not advance until resumed");
        assert_eq!(
            still_recoverable.state,
            RuntimeOfflineRenderExecutionState::Recoverable
        );
        assert_eq!(
            still_recoverable.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert!(still_recoverable.checkpoint.is_none());

        runtime
            .resume_offline_render_execution("render:recover:0001")
            .expect("recoverable execution should resume");
        let mut completed = None;
        for _ in 0..32 {
            let receipt = runtime
                .advance_offline_render_execution("render:recover:0001")
                .expect("resumed recoverable execution should advance");
            if let Some(result) = receipt.result {
                completed = Some(result);
                break;
            }
        }
        let completed = completed.expect("recoverable session should resume to completion");
        assert!(artifact_dir.exists());
        assert!(completed.manifest.report.is_some());

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &completed.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &completed.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_session_snapshot_preserves_checkpoint_through_pause_and_recoverable_states(
    ) {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-session-snapshot");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:session:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render execution should begin");
        runtime
            .advance_offline_render_execution("render:session:0001")
            .expect("offline render execution should advance");

        let running_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(running_snapshot.active_session_count, 1);
        assert_eq!(
            running_snapshot.active_sessions[0].request_id,
            "render:session:0001"
        );
        assert!(running_snapshot.active_sessions[0]
            .last_checkpoint
            .as_ref()
            .is_some());

        runtime
            .pause_offline_render_execution("render:session:0001")
            .expect("offline render execution should pause");
        let paused_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(paused_snapshot.active_session_count, 1);
        assert_eq!(paused_snapshot.paused_session_count, 1);
        assert_eq!(paused_snapshot.recoverable_session_count, 0);
        assert_eq!(
            paused_snapshot.active_sessions[0].state,
            RuntimeOfflineRenderExecutionState::Paused
        );
        assert_eq!(
            paused_snapshot.active_sessions[0].interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert!(paused_snapshot.active_sessions[0]
            .active_checkpoint
            .is_some());
        assert!(paused_snapshot.active_sessions[0].last_checkpoint.is_some());
        assert_eq!(
            paused_snapshot
                .last_session
                .as_ref()
                .map(|session| session.state),
            Some(RuntimeOfflineRenderExecutionState::Paused)
        );

        runtime
            .resume_offline_render_execution("render:session:0001")
            .expect("paused execution should resume");
        runtime
            .interrupt_offline_render_execution(
                "render:session:0001",
                "recoverable interruption".into(),
            )
            .expect("running execution should become recoverable");
        let recoverable_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(recoverable_snapshot.active_session_count, 1);
        assert_eq!(recoverable_snapshot.paused_session_count, 0);
        assert_eq!(recoverable_snapshot.recoverable_session_count, 1);
        assert_eq!(
            recoverable_snapshot.active_sessions[0].state,
            RuntimeOfflineRenderExecutionState::Recoverable
        );
        assert_eq!(
            recoverable_snapshot.active_sessions[0].interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert_eq!(
            recoverable_snapshot.active_sessions[0].interruption_count,
            1
        );
        assert!(recoverable_snapshot.active_sessions[0]
            .active_checkpoint
            .is_some());
        assert!(recoverable_snapshot.active_sessions[0]
            .last_checkpoint
            .is_some());
        assert_eq!(
            recoverable_snapshot
                .last_session
                .as_ref()
                .map(|session| session.state),
            Some(RuntimeOfflineRenderExecutionState::Recoverable)
        );

        runtime
            .cancel_offline_render_execution("render:session:0001")
            .expect("recoverable execution should cancel for cleanup");

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_session_snapshot_tracks_completed_cancellation_and_purge_receipts() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let completed_artifact_dir = temp_artifact_dir("offline-render-session-completed");
        let cancelled_artifact_dir = temp_artifact_dir("offline-render-session-cancelled");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:session:completed".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(completed_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("completed session should begin");
        let mut completed_result = None;
        for _ in 0..32 {
            let receipt = runtime
                .advance_offline_render_execution("render:session:completed")
                .expect("completed session should advance");
            if let Some(result) = receipt.result {
                completed_result = Some(result);
                break;
            }
        }
        let completed_result = completed_result.expect("completed session should finish");
        let completed_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(completed_snapshot.active_session_count, 0);
        assert_eq!(
            completed_snapshot
                .last_session
                .as_ref()
                .map(|session| session.state),
            Some(RuntimeOfflineRenderExecutionState::Completed)
        );
        assert_eq!(
            completed_snapshot
                .last_session
                .as_ref()
                .map(|session| session.request_id.as_str()),
            Some("render:session:completed")
        );
        assert_eq!(
            completed_snapshot
                .last_session
                .as_ref()
                .map(|session| session.materialized),
            Some(true)
        );
        assert_eq!(
            completed_snapshot
                .last_session
                .as_ref()
                .map(|session| session.artifact_count),
            Some(completed_result.manifest.artifact_count)
        );
        assert_eq!(
            completed_snapshot
                .last_session
                .as_ref()
                .and_then(|session| session.report_path.as_deref()),
            completed_result
                .manifest
                .report
                .as_ref()
                .map(|report| report.report_path.as_str())
        );

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:session:cancelled".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(cancelled_artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("cancelled session should begin");
        runtime
            .advance_offline_render_execution("render:session:cancelled")
            .expect("cancelled session should advance");
        runtime
            .cancel_offline_render_execution("render:session:cancelled")
            .expect("cancelled session should cancel");
        let cancelled_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(
            cancelled_snapshot
                .last_session
                .as_ref()
                .map(|session| session.state),
            Some(RuntimeOfflineRenderExecutionState::Cancelled)
        );
        assert_eq!(
            cancelled_snapshot
                .last_cancellation
                .as_ref()
                .map(|receipt| receipt.request_id.as_str()),
            Some("render:session:cancelled")
        );

        let completed_report_path = completed_result
            .manifest
            .report
            .as_ref()
            .map(|receipt| receipt.report_path.clone())
            .expect("completed session should materialize report");
        runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: completed_result.request_id.clone(),
                artifact_root_path: completed_result.manifest.artifact_root_path.clone(),
                report_path: Some(completed_report_path.clone()),
            })
            .expect("purge should succeed");
        let purged_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(
            purged_snapshot
                .last_purge
                .as_ref()
                .map(|receipt| receipt.request_id.as_str()),
            Some("render:session:completed")
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        assert!(report
            .render_json()
            .contains("\"offline_render_session_snapshot\":{"));
        assert!(report
            .render_json()
            .contains("\"request_id\":\"render:session:cancelled\""));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&completed_artifact_dir);
        let _ = fs::remove_dir(&cancelled_artifact_dir);
    }

    #[test]
    fn runtime_offline_render_session_snapshot_reports_restartable_state_across_stop_restart_and_resume(
    ) {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-session-restartable");
        runtime.start().expect("runtime should start");

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:session:restartable".into(),
                timeline_start_samples: 0,
                duration_samples: 2048,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("restartable session should begin");
        runtime
            .advance_offline_render_execution("render:session:restartable")
            .expect("restartable session should advance");

        runtime
            .stop(StopReason::DeviceReconfigure)
            .expect("runtime stop should succeed");
        let stopped_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(stopped_snapshot.active_session_count, 1);
        assert_eq!(
            stopped_snapshot.active_sessions[0].state,
            RuntimeOfflineRenderExecutionState::Recoverable
        );
        assert_eq!(
            stopped_snapshot.active_sessions[0].interruption_class,
            RuntimeInterruptionClass::Restartable
        );
        assert_eq!(
            stopped_snapshot
                .last_session
                .as_ref()
                .map(|session| session.interruption_class),
            Some(RuntimeInterruptionClass::Restartable)
        );

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("runtime restart should succeed");
        let restarted_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(restarted_snapshot.active_session_count, 1);
        assert_eq!(
            restarted_snapshot.active_sessions[0].interruption_class,
            RuntimeInterruptionClass::Restartable
        );

        runtime
            .resume_offline_render_execution("render:session:restartable")
            .expect("restartable session should resume");
        let resumed_snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(resumed_snapshot.active_session_count, 1);
        assert_eq!(
            resumed_snapshot.active_sessions[0].state,
            RuntimeOfflineRenderExecutionState::Running
        );
        assert_eq!(
            resumed_snapshot.active_sessions[0].interruption_class,
            RuntimeInterruptionClass::Steady
        );

        let mut completed = None;
        for _ in 0..32 {
            let receipt = runtime
                .advance_offline_render_execution("render:session:restartable")
                .expect("resumed restartable session should advance");
            if let Some(result) = receipt.result {
                completed = Some(result);
                break;
            }
        }
        let completed = completed.expect("restartable session should complete after resume");
        assert!(completed.manifest.report.is_some());

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &completed.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &completed.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_session_snapshot_reports_failed_terminal_state_on_delivery_error() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();

        runtime
            .begin_offline_render_execution(RuntimeOfflineRenderRequest {
                request_id: "render:session:terminal".into(),
                timeline_start_samples: 0,
                duration_samples: 256,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some("/dev/null/signal-runtime-offline-render-terminal".into()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("terminal session should begin");

        let mut failure = None;
        for _ in 0..16 {
            match runtime.advance_offline_render_execution("render:session:terminal") {
                Ok(_) => continue,
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        let failure = failure.expect("terminal session should fail during delivery");
        assert!(matches!(
            failure.kind,
            RuntimeErrorKind::ResourceUnavailable | RuntimeErrorKind::Fatal
        ));

        let snapshot = runtime.get_offline_render_session_snapshot();
        assert_eq!(snapshot.active_session_count, 0);
        assert_eq!(
            snapshot.last_session.as_ref().map(|session| session.state),
            Some(RuntimeOfflineRenderExecutionState::Failed)
        );
        assert_eq!(
            snapshot
                .last_session
                .as_ref()
                .map(|session| session.interruption_class),
            Some(RuntimeInterruptionClass::Terminal)
        );
        assert_eq!(
            snapshot
                .last_session
                .as_ref()
                .and_then(|session| session.last_checkpoint.as_ref())
                .map(|checkpoint| checkpoint.stage),
            Some(RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts)
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        assert!(report
            .render_json()
            .contains("\"offline_render_session_snapshot\":{"));
        assert!(report.render_json().contains("\"state\":\"Failed\""));
        assert!(report
            .render_json()
            .contains("\"interruption_class\":\"Terminal\""));

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
    fn runtime_offline_render_queue_throttles_when_runtime_is_running() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        runtime.start().expect("start runtime");

        let first_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-first");
        let second_artifact_dir = temp_artifact_dir("offline-render-queue-throttle-second");
        let queue_result = runtime
            .render_offline_queue(vec![
                RuntimeOfflineRenderRequest {
                    request_id: "render:queue:throttle:0001".into(),
                    timeline_start_samples: 0,
                    duration_samples: 64,
                    export_sample_rate_hz: 48_000,
                    include_main_mix: true,
                    artifact_root_path: Some(first_artifact_dir.display().to_string()),
                    stem_targets: Vec::new(),
                    freeze_artifacts: Vec::new(),
                },
                RuntimeOfflineRenderRequest {
                    request_id: "render:queue:throttle:0002".into(),
                    timeline_start_samples: 32,
                    duration_samples: 64,
                    export_sample_rate_hz: 48_000,
                    include_main_mix: true,
                    artifact_root_path: Some(second_artifact_dir.display().to_string()),
                    stem_targets: Vec::new(),
                    freeze_artifacts: Vec::new(),
                },
            ])
            .expect("running runtime should throttle offline render queue");

        assert_eq!(
            queue_result.orchestration.decision,
            RuntimeDeferredServiceDecision::Throttle
        );
        assert_eq!(
            queue_result.orchestration.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert_eq!(
            queue_result.orchestration.reason,
            RuntimeDeferredServiceReason::RealtimeActive
        );
        assert_eq!(
            queue_result.orchestration.priority_band,
            RuntimeDeferredServicePriorityBand::UserVisible
        );
        assert_eq!(
            queue_result.orchestration.blocking_priority_band,
            Some(RuntimeDeferredServicePriorityBand::RealtimeCritical)
        );
        assert_eq!(
            queue_result.orchestration.backpressure_source,
            Some(RuntimeDeferredServiceBackpressureSource::RealtimeAudio)
        );
        assert!(queue_result.orchestration.starvation_risk);
        assert_eq!(queue_result.orchestration.starved_work_item_count, 1);
        assert_eq!(queue_result.orchestration.cancellation_cause, None);
        assert_eq!(queue_result.orchestration.cancelled_work_item_count, 0);
        assert_eq!(queue_result.orchestration.admitted_work_item_count, 1);
        assert_eq!(queue_result.orchestration.completed_work_item_count, 1);
        assert_eq!(queue_result.orchestration.deferred_work_item_count, 1);
        assert_eq!(queue_result.completed_job_count, 1);
        assert_eq!(queue_result.progress.len(), 1);
        assert_eq!(queue_result.results.len(), 1);
        assert_eq!(queue_result.deferred_requests.len(), 1);
        assert_eq!(
            queue_result.results[0].request_id,
            "render:queue:throttle:0001"
        );
        assert_eq!(
            queue_result.deferred_requests[0].request_id,
            "render:queue:throttle:0002"
        );
        assert!(queue_result.summary.contains("deferred_job_count=1"));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &queue_result.results[0].manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &queue_result.results[0].manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&first_artifact_dir);
        let _ = fs::remove_dir(&second_artifact_dir);
    }

    #[test]
    fn runtime_offline_render_queue_defers_and_resumes_after_safe_mode_clears() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");

        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:queue:safe-mode:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("safe mode should defer offline render queue");

        assert_eq!(
            deferred.orchestration.decision,
            RuntimeDeferredServiceDecision::Defer
        );
        assert_eq!(
            deferred.orchestration.interruption_class,
            RuntimeInterruptionClass::Resumable
        );
        assert_eq!(
            deferred.orchestration.reason,
            RuntimeDeferredServiceReason::SafeMode
        );
        assert_eq!(
            deferred.orchestration.priority_band,
            RuntimeDeferredServicePriorityBand::UserVisible
        );
        assert_eq!(
            deferred.orchestration.blocking_priority_band,
            Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
        );
        assert_eq!(
            deferred.orchestration.backpressure_source,
            Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
        );
        assert!(deferred.orchestration.starvation_risk);
        assert_eq!(deferred.orchestration.starved_work_item_count, 1);
        assert_eq!(deferred.orchestration.cancellation_cause, None);
        assert_eq!(deferred.orchestration.cancelled_work_item_count, 0);
        assert_eq!(deferred.completed_job_count, 0);
        assert!(deferred.progress.is_empty());
        assert!(deferred.results.is_empty());
        assert_eq!(deferred.deferred_requests.len(), 1);

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("disable safe mode");
        let resumed = runtime
            .render_offline_queue(deferred.deferred_requests)
            .expect("cleared safe mode should resume deferred queue");

        assert_eq!(
            resumed.orchestration.decision,
            RuntimeDeferredServiceDecision::Run
        );
        assert_eq!(
            resumed.orchestration.interruption_class,
            RuntimeInterruptionClass::Steady
        );
        assert_eq!(resumed.completed_job_count, 1);
        assert_eq!(resumed.results.len(), 1);
        assert!(resumed.deferred_requests.is_empty());
        assert_eq!(resumed.results[0].request_id, "render:queue:safe-mode:0001");

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
    fn runtime_offline_render_purge_removes_report_and_artifact_root() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-purge");

        let result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:purge-proof".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render should materialize purge proof artifacts");
        let report_path = result
            .manifest
            .report
            .as_ref()
            .map(|receipt| receipt.report_path.clone())
            .expect("report receipt should exist");
        assert!(PathBuf::from(&report_path).exists());
        assert!(artifact_dir.exists());

        let purge_receipt = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: result.request_id.clone(),
                artifact_root_path: result.manifest.artifact_root_path.clone(),
                report_path: Some(report_path.clone()),
            })
            .expect("offline render purge should succeed");

        assert_eq!(purge_receipt.request_id, "render:purge-proof");
        assert_eq!(
            purge_receipt.orchestration.decision,
            RuntimeDeferredServiceDecision::Run
        );
        assert_eq!(
            purge_receipt.orchestration.reason,
            RuntimeDeferredServiceReason::Ready
        );
        assert_eq!(
            purge_receipt.orchestration.priority_band,
            RuntimeDeferredServicePriorityBand::Maintenance
        );
        assert_eq!(purge_receipt.orchestration.blocking_priority_band, None);
        assert_eq!(purge_receipt.orchestration.backpressure_source, None);
        assert!(!purge_receipt.orchestration.starvation_risk);
        assert_eq!(purge_receipt.orchestration.starved_work_item_count, 0);
        assert_eq!(purge_receipt.orchestration.cancellation_cause, None);
        assert_eq!(purge_receipt.orchestration.cancelled_work_item_count, 0);
        assert!(purge_receipt.purged_report);
        assert!(purge_receipt.purged_artifact_root);
        assert!(purge_receipt.purged_report_byte_count > 0);
        assert!(purge_receipt.purged_artifact_file_count > 0);
        assert!(purge_receipt.purged_artifact_byte_count > 0);
        assert!(purge_receipt.summary.contains("artifact_files="));
        assert!(!PathBuf::from(&report_path).exists());
        assert!(!artifact_dir.exists());

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
    fn runtime_purge_defers_in_safe_mode_and_observation_export_surfaces_last_decision() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-purge-deferred");

        let result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:purge-deferred".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render should materialize deferred purge proof artifacts");
        let report_path = result
            .manifest
            .report
            .as_ref()
            .map(|receipt| receipt.report_path.clone())
            .expect("report receipt should exist");

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");
        let deferred = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: result.request_id.clone(),
                artifact_root_path: result.manifest.artifact_root_path.clone(),
                report_path: Some(report_path.clone()),
            })
            .expect("safe mode should defer purge");

        assert_eq!(
            deferred.orchestration.decision,
            RuntimeDeferredServiceDecision::Defer
        );
        assert_eq!(
            deferred.orchestration.reason,
            RuntimeDeferredServiceReason::SafeMode
        );
        assert_eq!(
            deferred.orchestration.priority_band,
            RuntimeDeferredServicePriorityBand::Maintenance
        );
        assert_eq!(
            deferred.orchestration.blocking_priority_band,
            Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
        );
        assert_eq!(
            deferred.orchestration.backpressure_source,
            Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
        );
        assert!(deferred.orchestration.starvation_risk);
        assert_eq!(deferred.orchestration.starved_work_item_count, 1);
        assert_eq!(deferred.orchestration.cancellation_cause, None);
        assert_eq!(deferred.orchestration.cancelled_work_item_count, 0);
        assert!(!deferred.purged_report);
        assert!(!deferred.purged_artifact_root);
        assert!(PathBuf::from(&report_path).exists());
        assert!(artifact_dir.exists());

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        assert_eq!(
            report
                .observation
                .last_deferred_service_receipt
                .as_ref()
                .map(|receipt| receipt.decision),
            Some(RuntimeDeferredServiceDecision::Defer)
        );
        assert!(report.render_json().contains("\"last_deferred_service\":{"));
        assert!(report
            .render_json()
            .contains("\"work_class\":\"OfflineRenderPurge\""));
        assert!(report.render_json().contains("\"decision\":\"Defer\""));

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("disable safe mode");
        let resumed = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: result.request_id,
                artifact_root_path: result.manifest.artifact_root_path,
                report_path: Some(report_path.clone()),
            })
            .expect("cleared safe mode should allow purge");
        assert_eq!(
            resumed.orchestration.decision,
            RuntimeDeferredServiceDecision::Run
        );
        assert!(resumed.purged_report);
        assert!(resumed.purged_artifact_root);
        assert!(!PathBuf::from(&report_path).exists());
        assert!(!artifact_dir.exists());

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
    fn runtime_offline_render_invalid_request_abort_surfaces_typed_cancellation_policy() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));

        let error = runtime
            .render_offline_queue(Vec::new())
            .expect_err("empty offline render queue should be rejected");

        assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
        let report = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let receipt = report
            .observation
            .last_deferred_service_receipt
            .as_ref()
            .expect("invalid request should record a deferred-service receipt");
        assert_eq!(
            receipt.work_class,
            RuntimeDeferredServiceClass::OfflineRenderQueue
        );
        assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Abort);
        assert_eq!(receipt.reason, RuntimeDeferredServiceReason::InvalidRequest);
        assert_eq!(
            receipt.priority_band,
            RuntimeDeferredServicePriorityBand::UserVisible
        );
        assert_eq!(receipt.blocking_priority_band, None);
        assert_eq!(receipt.backpressure_source, None);
        assert!(!receipt.starvation_risk);
        assert_eq!(receipt.starved_work_item_count, 0);
        assert_eq!(
            receipt.cancellation_cause,
            Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
        );
        assert_eq!(receipt.cancelled_work_item_count, 0);
        assert!(report
            .render_json()
            .contains("\"cancellation_cause\":\"InvalidRequest\""));
    }

    #[test]
    fn runtime_prepare_offline_plugin_execution_boundary_surfaces_runtime_owned_stage_contracts() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let boundary = runtime
            .prepare_offline_plugin_execution_boundary(&RuntimeOfflineRenderRequest {
                request_id: "render:boundary".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline plugin boundary should build");

        assert_eq!(boundary.stage_count, 1);
        assert_eq!(boundary.block_count, 1);
        assert_eq!(boundary.signal_stage_model_stage_count, 1);
        assert_eq!(boundary.host_delegate_stage_count, 0);
        assert_eq!(boundary.fresh_override_stage_count, 1);
        assert_eq!(boundary.stale_override_stage_count, 0);
        assert_eq!(
            boundary.stages[0].execution_owner,
            RuntimeOfflinePluginExecutionOwner::SignalStageModel
        );
        assert!(!boundary.stages[0].host_delegate_required);
        assert_eq!(
            boundary.stages[0].override_state,
            RuntimeOfflinePluginOverrideState::FreshLatestBlock
        );
        assert_eq!(boundary.stages[0].sandbox_id.as_deref(), Some("sandbox-a"));
        assert_eq!(
            boundary.stages[0].recall_state,
            RuntimePluginRecallState::Recovered
        );
        assert_eq!(boundary.stages[0].plugin_type_id.as_deref(), None);
        assert_eq!(boundary.stages[0].plugin_format, Some(PluginFormat::Clap));
        assert_eq!(
            boundary.stages[0].recall_payload.plugin_type_id.as_deref(),
            None
        );
        assert_eq!(
            boundary.stages[0].recall_payload.plugin_format,
            Some(PluginFormat::Clap)
        );
        let delegated_request = runtime
            .prepare_offline_plugin_delegated_execution_request(&RuntimeOfflineRenderRequest {
                request_id: "render:boundary".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("delegated execution request should build");
        assert_eq!(delegated_request.stage_count, 0);
        assert!(delegated_request.stages.is_empty());

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
    fn runtime_offline_plugin_delegated_execution_request_filters_host_stages() {
        let boundary = RuntimeOfflinePluginExecutionBoundary {
            request_id: "render:delegated-boundary".into(),
            timeline_start_samples: 0,
            duration_samples: 128,
            runtime_sample_rate_hz: 48_000,
            export_sample_rate_hz: 48_000,
            block_size: 32,
            block_count: 4,
            stage_count: 2,
            signal_stage_model_stage_count: 1,
            host_delegate_stage_count: 1,
            fresh_override_stage_count: 0,
            stale_override_stage_count: 1,
            stages: vec![
                RuntimeOfflinePluginExecutionStageBoundary {
                    stage_id: RuntimePluginRecallHandoffStageId {
                        chain_id: "track:lead".into(),
                        stage_index: 0,
                        node_id: "plugin-a".into(),
                    },
                    node_id: "plugin-a".into(),
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    sandbox_id: Some("sandbox-a".into()),
                    plugin_type_id: None,
                    plugin_format: None,
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                    recall_state: RuntimePluginRecallState::Recovered,
                    recall_payload: RuntimePluginRecallPayload {
                        sandbox_id: Some("sandbox-a".into()),
                        recovery_count: 1,
                        ..RuntimePluginRecallPayload::default()
                    },
                    execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
                    host_delegate_required: true,
                    override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
                    latest_override_processing_epoch: Some(7),
                    latest_override_block_sequence: Some(12),
                    summary: "delegated".into(),
                },
                RuntimeOfflinePluginExecutionStageBoundary {
                    stage_id: RuntimePluginRecallHandoffStageId {
                        chain_id: "track:lead".into(),
                        stage_index: 1,
                        node_id: "plugin-b".into(),
                    },
                    node_id: "plugin-b".into(),
                    chain_id: "track:lead".into(),
                    stage_index: 1,
                    sandbox_id: Some("sandbox-b".into()),
                    plugin_type_id: None,
                    plugin_format: None,
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                    recall_state: RuntimePluginRecallState::Warm,
                    recall_payload: RuntimePluginRecallPayload {
                        sandbox_id: Some("sandbox-b".into()),
                        ..RuntimePluginRecallPayload::default()
                    },
                    execution_owner: RuntimeOfflinePluginExecutionOwner::SignalStageModel,
                    host_delegate_required: false,
                    override_state: RuntimeOfflinePluginOverrideState::NotAvailable,
                    latest_override_processing_epoch: None,
                    latest_override_block_sequence: None,
                    summary: "signal".into(),
                },
            ],
            summary: "boundary".into(),
        };

        let delegated_request = boundary.delegated_execution_request();

        assert_eq!(delegated_request.request_id, "render:delegated-boundary");
        assert_eq!(delegated_request.stage_count, 1);
        assert_eq!(delegated_request.stages[0].node_id, "plugin-a");
        assert_eq!(delegated_request.stages[0].plugin_format, None);
        assert_eq!(
            delegated_request.stages[0].override_state,
            RuntimeOfflinePluginOverrideState::StaleLatestBlock
        );
        assert_eq!(
            delegated_request.stages[0]
                .latest_override_processing_epoch
                .unwrap(),
            7
        );
    }

    #[test]
    fn runtime_applies_delegated_execution_receipt_into_manifest_bundle() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-delegated-receipt");
        let mut result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:delegated-receipt".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render should succeed");
        result.plugin_execution_boundary = RuntimeOfflinePluginExecutionBoundary {
            request_id: result.request_id.clone(),
            timeline_start_samples: 0,
            duration_samples: 64,
            runtime_sample_rate_hz: 48_000,
            export_sample_rate_hz: 48_000,
            block_size: 32,
            block_count: 2,
            stage_count: 1,
            signal_stage_model_stage_count: 0,
            host_delegate_stage_count: 1,
            fresh_override_stage_count: 0,
            stale_override_stage_count: 1,
            stages: vec![RuntimeOfflinePluginExecutionStageBoundary {
                stage_id: RuntimePluginRecallHandoffStageId {
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    node_id: "plugin-a".into(),
                },
                node_id: "plugin-a".into(),
                chain_id: "track:lead".into(),
                stage_index: 0,
                sandbox_id: Some("sandbox-a".into()),
                plugin_type_id: None,
                plugin_format: None,
                track_lane_id: Some("track:lead".into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
                recall_state: RuntimePluginRecallState::Recovered,
                recall_payload: RuntimePluginRecallPayload {
                    sandbox_id: Some("sandbox-a".into()),
                    recovery_count: 1,
                    ..RuntimePluginRecallPayload::default()
                },
                execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
                host_delegate_required: true,
                override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
                latest_override_processing_epoch: Some(4),
                latest_override_block_sequence: Some(9),
                summary: "delegated".into(),
            }],
            summary: "boundary".into(),
        };

        let updated = runtime
            .apply_offline_plugin_delegated_execution_receipt(
                &result,
                RuntimeOfflinePluginDelegatedExecutionReceipt {
                    request_id: result.request_id.clone(),
                    stage_count: 1,
                    completed_stage_count: 1,
                    rejected_stage_count: 0,
                    unavailable_stage_count: 0,
                    stages: vec![RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                        stage_id: RuntimePluginRecallHandoffStageId {
                            chain_id: "track:lead".into(),
                            stage_index: 0,
                            node_id: "plugin-a".into(),
                        },
                        node_id: "plugin-a".into(),
                        chain_id: "track:lead".into(),
                        stage_index: 0,
                        status: RuntimeOfflinePluginDelegatedExecutionStatus::Completed,
                        delegate_label: Some("host:offline-sandbox".into()),
                        detail: Some("rendered by delegated sandbox".into()),
                        summary: "completed".into(),
                    }],
                    summary: "receipt".into(),
                },
            )
            .expect("delegated execution receipt should apply");

        assert_eq!(updated.manifest.delegated_execution_request.stage_count, 1);
        assert_eq!(
            updated.manifest.delegated_execution_request.stages[0].node_id,
            "plugin-a"
        );
        assert_eq!(
            updated
                .manifest
                .delegated_execution_receipt
                .as_ref()
                .unwrap()
                .completed_stage_count,
            1
        );
        assert!(updated
            .manifest
            .summary
            .contains("delegated_request_stages=1"));
        assert!(updated.manifest.summary.contains("delegated_receipt=true"));
        let report_receipt = updated
            .manifest
            .report
            .as_ref()
            .expect("materialized report receipt should exist");
        let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
        assert!(report_body.contains("\"delegated_receipt_stage_count\":1"));
        assert!(report_body.contains("\"delegate_label\":\"host:offline-sandbox\""));
        assert!(report_body.contains("\"status\":\"Completed\""));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &updated.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &updated.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_offline_render_receipts_pin_delegated_unavailable_boundary() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-receipt-unavailable");
        let mut result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:delegated-unavailable".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            })
            .expect("offline render should succeed");
        result.plugin_execution_boundary = RuntimeOfflinePluginExecutionBoundary {
            request_id: result.request_id.clone(),
            timeline_start_samples: 0,
            duration_samples: 64,
            runtime_sample_rate_hz: 48_000,
            export_sample_rate_hz: 48_000,
            block_size: 32,
            block_count: 2,
            stage_count: 1,
            signal_stage_model_stage_count: 0,
            host_delegate_stage_count: 1,
            fresh_override_stage_count: 0,
            stale_override_stage_count: 1,
            stages: vec![RuntimeOfflinePluginExecutionStageBoundary {
                stage_id: RuntimePluginRecallHandoffStageId {
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    node_id: "plugin-a".into(),
                },
                node_id: "plugin-a".into(),
                chain_id: "track:lead".into(),
                stage_index: 0,
                sandbox_id: Some("sandbox-a".into()),
                plugin_type_id: None,
                plugin_format: None,
                track_lane_id: Some("track:lead".into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
                recall_state: RuntimePluginRecallState::Recovered,
                recall_payload: RuntimePluginRecallPayload {
                    sandbox_id: Some("sandbox-a".into()),
                    recovery_count: 1,
                    ..RuntimePluginRecallPayload::default()
                },
                execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
                host_delegate_required: true,
                override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
                latest_override_processing_epoch: Some(4),
                latest_override_block_sequence: Some(9),
                summary: "delegated".into(),
            }],
            summary: "boundary".into(),
        };

        let updated = runtime
            .apply_offline_plugin_delegated_execution_receipt(
                &result,
                RuntimeOfflinePluginDelegatedExecutionReceipt {
                    request_id: result.request_id.clone(),
                    stage_count: 1,
                    completed_stage_count: 0,
                    rejected_stage_count: 0,
                    unavailable_stage_count: 1,
                    stages: vec![RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                        stage_id: RuntimePluginRecallHandoffStageId {
                            chain_id: "track:lead".into(),
                            stage_index: 0,
                            node_id: "plugin-a".into(),
                        },
                        node_id: "plugin-a".into(),
                        chain_id: "track:lead".into(),
                        stage_index: 0,
                        status: RuntimeOfflinePluginDelegatedExecutionStatus::Unavailable,
                        delegate_label: Some("host:offline-sandbox".into()),
                        detail: Some("delegate not available during degraded recovery".into()),
                        summary: "unavailable".into(),
                    }],
                    summary: "receipt".into(),
                },
            )
            .expect("delegated unavailable receipt should apply");

        let profiling = updated.profiling_receipt();
        let soak = updated.soak_receipt();
        assert_eq!(profiling.delegated_stage_count, 1);
        assert_eq!(profiling.stale_override_stage_count, 1);
        assert_eq!(profiling.artifact_count, 1);
        assert!(profiling.report_materialized);
        assert!(profiling
            .render_json()
            .contains("\"delegated_stage_count\":1"));
        assert_eq!(soak.delegated_stage_count, 1);
        assert_eq!(soak.delegated_completed_stage_count, 0);
        assert_eq!(soak.delegated_rejected_stage_count, 0);
        assert_eq!(soak.delegated_unavailable_stage_count, 1);
        assert!(soak
            .render_json()
            .contains("\"delegated_unavailable_stage_count\":1"));

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &updated.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &updated.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

    #[test]
    fn runtime_applies_delegated_execution_outcome_into_runtime_owned_finalization() {
        let (runtime, imported_path) = prepare_offline_render_engine_runtime();
        let artifact_dir = temp_artifact_dir("offline-render-delegated-outcome");
        let handoff = runtime.get_plugin_recall_handoff_snapshot();
        let mut result = runtime
            .render_offline(RuntimeOfflineRenderRequest {
                request_id: "render:delegated-outcome".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: Some(artifact_dir.display().to_string()),
                stem_targets: vec![RuntimeOfflineRenderStemTarget {
                    stem_id: "stem:track:lead".into(),
                    target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
                    target_id: Some("track:lead".into()),
                }],
                freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
                    artifact_id: "freeze:track:lead".into(),
                    source_stem_id: "stem:track:lead".into(),
                    recall_selection: RuntimePluginRecallHandoffSelection {
                        stage_count: handoff.stage_count,
                        stage_ids: handoff
                            .stages
                            .iter()
                            .map(|stage| stage.stage_id.clone())
                            .collect(),
                    },
                }],
            })
            .expect("offline render should succeed");
        result.plugin_execution_boundary = RuntimeOfflinePluginExecutionBoundary {
            request_id: result.request_id.clone(),
            timeline_start_samples: 0,
            duration_samples: 64,
            runtime_sample_rate_hz: 48_000,
            export_sample_rate_hz: 48_000,
            block_size: 32,
            block_count: 2,
            stage_count: 1,
            signal_stage_model_stage_count: 0,
            host_delegate_stage_count: 1,
            fresh_override_stage_count: 0,
            stale_override_stage_count: 1,
            stages: vec![RuntimeOfflinePluginExecutionStageBoundary {
                stage_id: RuntimePluginRecallHandoffStageId {
                    chain_id: "track:lead".into(),
                    stage_index: 0,
                    node_id: "plugin-a".into(),
                },
                node_id: "plugin-a".into(),
                chain_id: "track:lead".into(),
                stage_index: 0,
                sandbox_id: Some("sandbox-a".into()),
                plugin_type_id: None,
                plugin_format: None,
                track_lane_id: Some("track:lead".into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
                recall_state: RuntimePluginRecallState::Recovered,
                recall_payload: RuntimePluginRecallPayload {
                    sandbox_id: Some("sandbox-a".into()),
                    recovery_count: 1,
                    ..RuntimePluginRecallPayload::default()
                },
                execution_owner: RuntimeOfflinePluginExecutionOwner::HostDelegated,
                host_delegate_required: true,
                override_state: RuntimeOfflinePluginOverrideState::StaleLatestBlock,
                latest_override_processing_epoch: Some(4),
                latest_override_block_sequence: Some(9),
                summary: "delegated".into(),
            }],
            summary: "boundary".into(),
        };

        let updated = runtime
            .apply_offline_plugin_delegated_execution_outcome(
                &result,
                RuntimeOfflinePluginDelegatedExecutionOutcome {
                    receipt: RuntimeOfflinePluginDelegatedExecutionReceipt {
                        request_id: result.request_id.clone(),
                        stage_count: 1,
                        completed_stage_count: 1,
                        rejected_stage_count: 0,
                        unavailable_stage_count: 0,
                        stages: vec![RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                            stage_id: RuntimePluginRecallHandoffStageId {
                                chain_id: "track:lead".into(),
                                stage_index: 0,
                                node_id: "plugin-a".into(),
                            },
                            node_id: "plugin-a".into(),
                            chain_id: "track:lead".into(),
                            stage_index: 0,
                            status: RuntimeOfflinePluginDelegatedExecutionStatus::Completed,
                            delegate_label: Some("host:offline-sandbox".into()),
                            detail: Some("rendered by delegated sandbox".into()),
                            summary: "completed".into(),
                        }],
                        summary: "receipt".into(),
                    },
                    merge: RuntimeOfflinePluginDelegatedExecutionMerge {
                        request_id: result.request_id.clone(),
                        main_mix: Some(filled_stereo_buffer(48_000, 64, 0.2)),
                        stems: vec![RuntimeOfflinePluginDelegatedStemOutput {
                            stem_id: "stem:track:lead".into(),
                            output: filled_stereo_buffer(48_000, 64, 0.1),
                            summary: "stem override".into(),
                        }],
                        freeze_artifacts: vec![RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
                            artifact_id: "freeze:track:lead".into(),
                            output: filled_stereo_buffer(48_000, 64, 0.05),
                            summary: "freeze override".into(),
                        }],
                        summary: "merge".into(),
                    },
                    summary: "outcome".into(),
                },
            )
            .expect("delegated execution outcome should apply");

        assert!((updated.main_mix_peak_level.unwrap() - 0.2).abs() < 1.0e-6);
        assert!((updated.stems[0].peak_level - 0.1).abs() < 1.0e-6);
        assert!((updated.freeze_artifacts[0].peak_level - 0.05).abs() < 1.0e-6);
        assert_eq!(updated.main_mix.as_ref().unwrap().samples()[0], 0.2);
        assert_eq!(updated.stems[0].output.samples()[0], 0.1);
        assert_eq!(updated.freeze_artifacts[0].output.samples()[0], 0.05);
        let report_receipt = updated
            .manifest
            .report
            .as_ref()
            .expect("materialized report receipt should exist");
        let report_body = fs::read_to_string(&report_receipt.report_path).expect("read report");
        assert!(report_body.contains("\"delegate_label\":\"host:offline-sandbox\""));
        assert!(report_body.contains("\"peak_level\":0.200000"));
        assert!(report_body.contains("\"peak_level\":0.100000"));
        assert!(report_body.contains("\"peak_level\":0.050000"));

        let main_mix_receipt = updated
            .manifest
            .artifacts
            .iter()
            .find(|receipt| receipt.artifact_kind == RuntimeOfflineRenderArtifactKind::MainMix)
            .expect("main mix receipt should exist");
        let mut main_mix_reader =
            hound::WavReader::open(&main_mix_receipt.output_path).expect("main mix wav readable");
        let first_sample = main_mix_reader
            .samples::<f32>()
            .next()
            .expect("main mix wav should contain samples")
            .expect("main mix wav sample should decode");
        assert!((first_sample - 0.2).abs() < 1.0e-6);

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
        for receipt in &updated.manifest.artifacts {
            let _ = fs::remove_file(&receipt.output_path);
        }
        if let Some(report_receipt) = &updated.manifest.report {
            let _ = fs::remove_file(&report_receipt.report_path);
        }
        let _ = fs::remove_dir(&artifact_dir);
    }

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
                    output: AudioBuffer::new(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        FrameCount(32),
                    ),
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

    #[test]
    fn runtime_degraded_bound_plugin_session_gates_prework_lane() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-bound forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply plugin-backed bindings");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late detach fault".into()),
        );

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated prework lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_service_yield_count >= 1);

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        let profiling = supervisor.profiling_receipt();
        let soak = supervisor.soak_receipt();
        assert!(profiling.plugin_gate_active);
        assert_eq!(profiling.degraded_bound_plugin_sandboxes, 1);
        assert_eq!(profiling.missing_bound_plugin_sandboxes, 0);
        assert_eq!(profiling.plugin_chain_stage_count, 1);
        assert!(profiling
            .render_json()
            .contains("\"plugin_gate_active\":true"));
        assert!(profiling
            .render_json()
            .contains("\"degraded_bound_plugin_sandboxes\":1"));
        assert_eq!(soak.plugin_fault_count, 0);
        assert_eq!(soak.plugin_quarantined_sandbox_count, 0);
    }

    #[test]
    fn runtime_realtime_block_services_prework_window_under_normal_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set realtime-driven forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-normal");
        runtime.start().expect("start runtime");

        let before = runtime.get_engine_block_snapshot();
        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert_eq!(
            first.snapshot.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert!(snapshot.prework_service_cycle_count > before.prework_service_cycle_count);
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(4)
        );
        assert!(snapshot.last_prework_service_prepared_targets >= 1);
        assert!(snapshot
            .last_prework_serviced_target_block_sequence
            .is_some_and(|block_sequence| block_sequence >= 5));
        assert_eq!(
            snapshot.last_prework_serviced_backlog_class,
            Some(RuntimePreworkBacklogClass::Deferred)
        );
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle
        );
        assert!(snapshot
            .prework_cache_window_target_block_sequences
            .contains(&5));
    }

    #[test]
    fn runtime_compatible_schedule_projection_widens_normal_prework_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set widened multicore realtime policy");
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:realtime-scheduler-widened-budget",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:widened-budget".into(),
                stream_count: 3,
            })
            .expect("apply widened compatible schedule");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process widened realtime block");

        assert_eq!(
            first.snapshot.scheduler_topology.schedule_stream_count,
            Some(3)
        );
        assert!(first.snapshot.scheduler_topology.compatible);
        assert_eq!(first.snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(first.snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(first.snapshot.last_prework_service_cycle_count, 3);
        assert_eq!(
            first.snapshot.last_prework_service_budget_per_cycle,
            Some(1)
        );
        assert_eq!(
            first
                .snapshot
                .last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
        assert!(first.snapshot.last_prework_service_prepared_targets >= 7);
        assert!(first.snapshot.prework_service_prepared_targets >= 7);
        assert_eq!(first.snapshot.prework_pending_target_count, 0);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation
                .engine_block_snapshot
                .scheduler_topology
                .schedule_stream_count,
            Some(3)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .last_prework_service_requested_cycles,
            3
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
        assert!(observation
            .render_json()
            .contains("\"last_prework_service_requested_cycles\":3"));
        assert!(observation
            .render_json()
            .contains("\"last_prework_service_effective_budget_per_cycle\":3"));
    }

    #[test]
    fn runtime_missing_schedule_projection_does_not_widen_prework_service_budget() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set single-budget realtime policy");
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:realtime-scheduler-no-schedule",
            &["track:drums", "track:bass"],
            false,
        );
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process no-schedule realtime block");

        assert_eq!(
            first.snapshot.scheduler_topology.schedule_stream_count,
            None
        );
        assert!(!first.snapshot.scheduler_topology.compatible);
        assert_eq!(first.snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(
            first.snapshot.last_prework_service_budget_per_cycle,
            Some(1)
        );
        assert_eq!(
            first
                .snapshot
                .last_prework_service_effective_budget_per_cycle,
            Some(1)
        );
        assert!(first.snapshot.last_prework_service_prepared_targets <= 1);
    }

    #[test]
    fn runtime_elevated_pressure_clamps_schedule_widened_prework_cycles() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };
        runtime
            .set_prework_forecast_policy(policy.clone())
            .expect("set elevated widened realtime policy");
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:realtime-scheduler-elevated-widened-cycles",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:elevated-widened-cycles".into(),
                stream_count: 3,
            })
            .expect("apply elevated compatible schedule");
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");
        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime elevated widened forecast window");
        assert_eq!(admitted, 1);

        let snapshot = runtime.get_engine_block_snapshot();

        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert!(snapshot.scheduler_topology.compatible);
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(1)
        );
        assert!(snapshot.last_prework_service_prepared_targets <= 1);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(snapshot.prework_service_throttle_count >= 1);
        assert!(snapshot.prework_pending_target_count > 0);
    }

    #[test]
    fn runtime_realtime_block_respects_elevated_pressure_backlog_limits() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set elevated realtime-driven forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-elevated");
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert_eq!(
            first.snapshot.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
        assert!(snapshot
            .prework_next_pending_target_block_sequence
            .is_some());
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_recovery_overlap_throttles_realtime_scheduler_under_normal_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set recovery-overlap realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-overlap");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin steady session");
        runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .expect("begin overlap session");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Normal
        );
        assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 1);
        assert_eq!(snapshot.prework_service_lingering_sessions, 0);
        assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
        assert!(!snapshot.prework_service_transport_gate_active);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(1)
        );
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert!(snapshot.prework_service_throttle_count >= 1);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(report.degradation_summary.recovery_overlap_sessions, 1);
        assert_eq!(report.degradation_summary.lingering_sessions, 0);
        assert!(!report.degradation_summary.transport_gate_active);
        assert_eq!(
            report.scheduler_summary.prework_pending_target_count,
            snapshot.prework_pending_target_count
        );
        assert!(report
            .render_compact()
            .contains("degradation_summary_sessions=1/0/0/0"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("degradation_summary_recovery_overlap_sessions=1"));
        let json = supervisor.render_json();
        assert!(json.contains("\"degradation_summary\":{"));
        assert!(json.contains("\"recovery_overlap_sessions\":1"));
        assert!(json.contains("\"lingering_sessions\":0"));
    }

    #[test]
    fn runtime_lingering_transport_enters_yielding_scheduler_state_under_elevated_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set lingering realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-lingering");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin steady session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");
        seed_pending_prework_targets(&mut runtime, 1, &[2, 3, 4]);
        runtime.refresh_prework_service_policy_and_state(None);
        let snapshot = runtime.get_engine_block_snapshot();

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 0);
        assert_eq!(snapshot.prework_service_lingering_sessions, 1);
        assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
        assert!(snapshot.prework_service_transport_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
    }

    #[test]
    fn runtime_schedule_widened_transport_gate_yields_without_servicing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 6,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };
        runtime
            .set_prework_forecast_policy(policy.clone())
            .expect("set widened transport policy");
        apply_latency_runtime_graph(
            &mut runtime,
            "graph:runtime:transport-gate-schedule-widened",
        );
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin steady session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:transport-gate-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        let current_sequence = runtime.allocate_block_sequence();

        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime widened transport-gated window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(admitted, 0);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0)
        );
        assert!(snapshot.prework_service_transport_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_restart_and_reconfigure_keep_realtime_scheduler_window_coherent() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set restart forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-restart");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert!(first
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&4));

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart runtime");
        let restarted = runtime.get_engine_block_snapshot();
        assert_eq!(
            restarted.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let restart_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let after_restart = runtime
            .process_engine_block(2, 2, restart_block)
            .expect("process realtime block after restart");
        assert!(after_restart
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&5));
        assert_eq!(
            after_restart.snapshot.last_prework_service_processing_epoch,
            Some(2)
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure runtime");
        let reconfigured = runtime.get_engine_block_snapshot();
        assert_eq!(
            reconfigured.prework_cache_window_target_block_sequences,
            vec![3, 4, 5]
        );
        assert_eq!(
            reconfigured.prework_service_state,
            RuntimePreworkServiceState::Paused
        );

        let reconfigured_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3);
        runtime.start().expect("restart after reconfigure");
        apply_current_forecast_block_state(&mut runtime, 3);
        let after_reconfigure = runtime
            .process_engine_block(3, 3, reconfigured_block)
            .expect("process realtime block after reconfigure");
        assert!(after_reconfigure
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&6));
        assert_eq!(
            after_reconfigure
                .snapshot
                .last_prework_service_processing_epoch,
            Some(3)
        );

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(report.control_snapshot.restart_count, 1);
        assert!(report.scheduler_summary.prework_pending_target_count > 0);
        assert!(report.render_compact().contains("restarts=1"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor.render_multiline().contains("restart_count=1"));
        assert!(supervisor
            .render_multiline()
            .contains("scheduler_summary_pending_targets="));
        let json = supervisor.render_json();
        assert!(json.contains("\"restart_count\":1"));
        assert!(json.contains("\"scheduler_summary\":{"));
    }

    #[test]
    fn runtime_schedule_width_survives_restart_and_reconfigure_transitions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set widened restart policy");
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:restart-reconfigure-schedule-widened",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:restart-reconfigure-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        runtime.start().expect("start runtime");

        let started = runtime.get_engine_block_snapshot();
        assert_eq!(started.scheduler_topology.schedule_stream_count, Some(3));
        assert!(started.scheduler_topology.compatible);
        assert_eq!(started.last_prework_service_requested_cycles, 3);
        assert_eq!(started.last_prework_service_effective_cycles, 3);

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart runtime");
        let restarted = runtime.get_engine_block_snapshot();
        assert_eq!(restarted.scheduler_topology.schedule_stream_count, Some(3));
        assert!(restarted.scheduler_topology.compatible);
        assert_eq!(restarted.last_prework_service_requested_cycles, 3);
        assert_eq!(restarted.last_prework_service_effective_cycles, 3);
        assert_eq!(runtime.get_control_snapshot().restart_count, 1);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure runtime");
        let reconfigured = runtime.get_engine_block_snapshot();
        assert_eq!(
            reconfigured.scheduler_topology.schedule_stream_count,
            Some(3)
        );
        assert!(reconfigured.scheduler_topology.compatible);
        assert_eq!(
            reconfigured.prework_service_state,
            RuntimePreworkServiceState::Paused
        );

        runtime.start().expect("restart after reconfigure");
        let restarted_after_reconfigure = runtime.get_engine_block_snapshot();
        assert_eq!(
            restarted_after_reconfigure
                .scheduler_topology
                .schedule_stream_count,
            Some(3)
        );
        assert!(restarted_after_reconfigure.scheduler_topology.compatible);
        assert_eq!(
            restarted_after_reconfigure.last_prework_service_requested_cycles,
            3
        );
        assert_eq!(
            restarted_after_reconfigure.last_prework_service_effective_cycles,
            3
        );
        assert_eq!(
            restarted_after_reconfigure.last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
    }

    #[test]
    fn runtime_supervisor_report_derives_profiling_and_soak_receipts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:profiling-receipt");
        runtime.set_cpu_load_percent(7.25);
        runtime.set_graph_latency_ms(3.5);
        runtime.start().expect("start runtime");
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1),
            )
            .expect("process profiling block");

        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-profile".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(1),
            },
        );

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(&runtime, &recorder);
        let profiling = supervisor.profiling_receipt();
        let soak = supervisor.soak_receipt();

        assert_eq!(profiling.sample_rate_hz, 48_000);
        assert_eq!(profiling.block_size, 256);
        assert_eq!(profiling.engine_processed_blocks, 1);
        assert_eq!(profiling.engine_node_count, 2);
        assert_eq!(profiling.engine_stage_count, 2);
        assert!(!profiling.readiness_degraded);
        assert!(!profiling.transport_gate_active);
        assert!(!profiling.plugin_gate_active);
        assert_eq!(profiling.plugin_chain_stage_count, 0);
        assert_eq!(profiling.plugin_chain_degraded_stage_count, 0);
        assert!((profiling.runtime_cpu_load_percent - 7.25).abs() < 1.0e-6);
        assert!((profiling.runtime_graph_latency_ms - 3.5).abs() < 1.0e-6);
        assert_eq!(profiling.host_callback_count, None);
        assert!(profiling
            .render_json()
            .contains("\"runtime_graph_latency_ms\":3.5"));

        assert_eq!(soak.event_stream_count, 1);
        assert!(!soak.readiness_degraded);
        assert_eq!(soak.recovery_event_count, 1);
        assert_eq!(soak.plugin_quarantined_sandbox_count, 0);
        assert_eq!(soak.recall_stage_count, 0);
        assert_eq!(
            soak.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(soak.last_stop_reason, None);
        assert!(soak.render_json().contains("\"recovery_event_count\":1"));
    }

    #[test]
    fn runtime_performance_snapshot_captures_scheduler_pressure_and_background_policy() {
        let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
        runtime.set_cpu_load_percent(11.5);
        runtime.set_graph_latency_ms(4.25);
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");

        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:queue:performance:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("safe mode should defer offline render queue");

        assert_eq!(
            deferred.orchestration.decision,
            RuntimeDeferredServiceDecision::Defer
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let performance = report.performance_snapshot();

        assert_eq!(performance.sample_rate_hz, 48_000);
        assert_eq!(performance.block_size, 256);
        assert!((performance.cpu_load_percent - 11.5).abs() < 1.0e-6);
        assert!((performance.graph_latency_ms - 4.25).abs() < 1.0e-6);
        let engine_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            performance.prework_service_state,
            engine_snapshot.prework_service_state
        );
        assert_eq!(
            performance.prework_service_pressure,
            engine_snapshot.prework_service_pressure
        );
        assert_eq!(
            performance.scheduler_prepared_dispatch_count,
            engine_snapshot.prepared_dispatch_count
        );
        assert_eq!(
            performance.scheduler_realtime_dispatch_count,
            engine_snapshot.realtime_dispatch_count
        );
        assert_eq!(
            performance.scheduler_dispatch_handoff_count,
            engine_snapshot.dispatch_handoff_count
        );
        assert_eq!(
            performance.scheduler_topology_compatible,
            engine_snapshot.scheduler_topology.compatible
        );
        assert_eq!(
            performance.scheduler_topology_requires_host_reinterpretation,
            engine_snapshot
                .scheduler_topology
                .requires_host_reinterpretation
        );
        assert_eq!(
            performance.scheduler_topology_issue_count,
            engine_snapshot.scheduler_topology.issues.len()
        );
        assert_eq!(
            performance.prework_service_starvation_count,
            engine_snapshot.prework_service_starvation_count
        );
        assert_eq!(
            performance.prework_service_throttle_count,
            engine_snapshot.prework_service_throttle_count
        );
        assert_eq!(
            performance.prework_service_yield_count,
            engine_snapshot.prework_service_yield_count
        );
        assert_eq!(
            performance.last_prework_service_effective_cycles,
            engine_snapshot.last_prework_service_effective_cycles
        );
        assert_eq!(
            performance.last_prework_service_budget_per_cycle,
            engine_snapshot.last_prework_service_budget_per_cycle
        );
        assert_eq!(
            performance.last_prework_service_effective_budget_per_cycle,
            engine_snapshot.last_prework_service_effective_budget_per_cycle
        );
        assert_eq!(
            performance.last_prework_serviced_backlog_class,
            engine_snapshot
                .last_prework_serviced_backlog_class
                .map(|class| format!("{class:?}"))
        );
        let expected_hot_node = engine_snapshot
            .planned_nodes
            .iter()
            .max_by_key(|node| node.latency_samples)
            .filter(|node| node.latency_samples > 0)
            .expect("prepared runtime should expose a latency-bearing hot node");
        assert_eq!(
            performance.hot_latency_node_id.as_deref(),
            Some(expected_hot_node.node_id.as_str())
        );
        assert_eq!(
            performance.hot_latency_node_group.as_deref(),
            Some(match expected_hot_node.group {
                GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
            })
        );
        assert_eq!(
            performance.hot_latency_node_topology_role.as_deref(),
            Some(match expected_hot_node.topology_role {
                GraphNodeTopologyRole::Utility => "Utility",
                GraphNodeTopologyRole::TrackLane => "TrackLane",
                GraphNodeTopologyRole::Bus => "Bus",
                GraphNodeTopologyRole::Send => "Send",
                GraphNodeTopologyRole::Return => "Return",
                GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
            })
        );
        assert_eq!(
            performance.hot_latency_node_samples,
            expected_hot_node.latency_samples
        );
        let expected_group_total = engine_snapshot
            .planned_nodes
            .iter()
            .filter(|node| node.group == expected_hot_node.group)
            .map(|node| node.latency_samples)
            .sum::<u32>();
        assert_eq!(
            performance.hot_latency_group.as_deref(),
            performance.hot_latency_node_group.as_deref()
        );
        assert_eq!(
            performance.hot_latency_group_node_count,
            engine_snapshot
                .planned_nodes
                .iter()
                .filter(|node| node.group == expected_hot_node.group)
                .count()
        );
        assert_eq!(
            performance.hot_latency_group_total_samples,
            expected_group_total
        );
        let expected_lane = performance
            .worker_lane_summaries
            .iter()
            .max_by_key(|summary| summary.total_latency_samples)
            .expect("prepared runtime should export at least one worker-lane summary");
        assert_eq!(
            performance.critical_path_lane.as_deref(),
            Some(match expected_lane.lane {
                GraphExecutionLane::Realtime => "Realtime",
                GraphExecutionLane::Anticipative => "Anticipative",
            })
        );
        assert_eq!(
            performance.critical_path_lane_node_count,
            expected_lane.node_count
        );
        assert_eq!(
            performance.critical_path_lane_plugin_backed_node_count,
            expected_lane.plugin_backed_node_count
        );
        assert_eq!(
            performance.critical_path_lane_planning_group_count,
            expected_lane.planning_group_count
        );
        assert_eq!(
            performance.critical_path_lane_total_latency_samples,
            expected_lane.total_latency_samples
        );
        assert_eq!(
            performance.worker_lane_summaries.len(),
            engine_snapshot.lane_order.len()
        );
        assert!(performance.worker_lane_summaries.iter().all(|summary| {
            summary.node_count > 0
                && summary.total_latency_samples >= summary.max_node_latency_samples
        }));
        assert_eq!(
            performance.background_service_class,
            Some(RuntimeDeferredServiceClass::OfflineRenderQueue)
        );
        assert_eq!(
            performance.background_service_decision,
            Some(RuntimeDeferredServiceDecision::Defer)
        );
        assert_eq!(
            performance.background_service_reason,
            Some(RuntimeDeferredServiceReason::SafeMode)
        );
        assert_eq!(
            performance.background_service_priority_band,
            Some(RuntimeDeferredServicePriorityBand::UserVisible)
        );
        assert_eq!(
            performance.background_service_blocking_priority_band,
            Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
        );
        assert_eq!(
            performance.background_service_backpressure_source,
            Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
        );
        assert!(performance.background_service_starvation_risk);
        assert_eq!(performance.background_service_starved_work_item_count, 1);
        assert_eq!(performance.background_service_cancellation_cause, None);
        assert_eq!(performance.background_service_cancelled_work_item_count, 0);
        assert_eq!(performance.background_queued_work_item_count, 1);
        assert_eq!(performance.background_deferred_work_item_count, 1);
        assert!(performance
            .render_json()
            .contains("\"background_service_decision\":\"Defer\""));
        assert!(performance
            .render_json()
            .contains("\"background_service_backpressure_source\":\"SafeMode\""));
        assert!(performance
            .render_json()
            .contains("\"scheduler_dispatch_handoff_count\":"));
        assert!(performance
            .render_json()
            .contains("\"critical_path_lane\":"));
        assert!(performance
            .render_json()
            .contains("\"worker_lane_summaries\":["));

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("disable safe mode");

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
    fn runtime_process_engine_block_records_bounded_timing_and_budget_fields() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:block-timing");

        let result = runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 21),
            )
            .expect("process runtime block with timing instrumentation");
        let snapshot = runtime.get_engine_block_snapshot();
        let diagnostics = runtime.get_diagnostics_snapshot();

        assert_eq!(snapshot.last_block_sequence, Some(1));
        assert_eq!(snapshot.last_block_deadline_budget_ns, Some(1_000_000));
        assert_eq!(
            result.snapshot.last_block_deadline_budget_ns,
            Some(1_000_000)
        );
        assert_eq!(
            snapshot.last_block_execution_time_ns,
            result.snapshot.last_block_execution_time_ns
        );
        let execution_time_ns = snapshot
            .last_block_execution_time_ns
            .expect("runtime should capture a block execution time");
        assert!(execution_time_ns > 0);
        assert_eq!(
            snapshot.last_block_budget_overrun_ns.is_some(),
            snapshot.last_block_deadline_pressure == RuntimeBlockDeadlinePressure::Overrun
        );
        assert!(snapshot.peak_block_execution_time_ns >= execution_time_ns);
        assert!(
            (diagnostics.graph_latency_ms - (execution_time_ns as f32 / 1_000_000.0)).abs() < 0.01
        );
        assert_eq!(
            diagnostics.cpu_load_percent,
            snapshot
                .last_block_budget_utilization_percent
                .expect("timing instrumentation should derive utilization")
        );
    }

    #[test]
    fn runtime_block_timing_pressure_rolls_into_performance_snapshot_and_trace_receipt() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:block-timing-trace");

        runtime.record_block_execution_timing_ns(48, 500_000);
        let normal = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

        runtime.record_block_execution_timing_ns(48, 800_000);
        let elevated = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

        runtime.record_block_execution_timing_ns(48, 950_000);
        let critical = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

        runtime.record_block_execution_timing_ns(48, 1_250_000);
        let overrun = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

        let performance = overrun.performance_snapshot();
        assert_eq!(performance.last_block_deadline_budget_ns, Some(1_000_000));
        assert_eq!(performance.last_block_execution_time_ns, Some(1_250_000));
        assert_eq!(
            performance.last_block_deadline_pressure,
            RuntimeBlockDeadlinePressure::Overrun
        );
        assert_eq!(performance.last_block_budget_overrun_ns, Some(250_000));
        assert_eq!(performance.budget_overrun_count, 1);
        assert_eq!(performance.peak_block_execution_time_ns, 1_250_000);
        assert_eq!(performance.peak_block_budget_overrun_ns, 250_000);
        assert!(performance
            .render_json()
            .contains("\"last_block_deadline_pressure\":\"Overrun\""));

        let trace = RuntimeSupervisorReport::build_performance_trace_receipt(&[
            normal.clone(),
            elevated,
            critical,
            overrun.clone(),
        ]);
        assert_eq!(trace.elevated_deadline_pressure_observation_count, 1);
        assert_eq!(trace.critical_deadline_pressure_observation_count, 1);
        assert_eq!(trace.overrun_deadline_pressure_observation_count, 1);
        assert_eq!(trace.budget_overrun_count_delta, 1);
        assert_eq!(trace.peak_block_execution_time_ns, 1_250_000);
        assert_eq!(trace.peak_block_budget_overrun_ns, 250_000);
        assert!(trace
            .render_json()
            .contains("\"budget_overrun_count_delta\":1"));
        assert!(overrun
            .render_json()
            .contains("\"last_block_deadline_pressure\":\"Overrun\""));
        assert!(overrun
            .render_json()
            .contains("\"last_block_deadline_budget_ns\":1000000"));
    }

    #[test]
    fn runtime_performance_trace_receipt_summarizes_playback_recording_and_deferred_work_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:performance-trace");
        runtime.set_cpu_load_percent(13.5);
        runtime.set_graph_latency_ms(5.25);

        let capture_path = temp_capture_path("performance-trace");
        let mut reports = Vec::new();
        reports.push(RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        ));

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 4_096,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:performance-trace".to_string(),
                track_id: "track:test:performance-trace".to_string(),
                start_samples: 4_096,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(16), 19),
            )
            .unwrap();
        reports.push(RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        ));

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");
        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:queue:performance-trace:0001".into(),
                timeline_start_samples: 0,
                duration_samples: 128,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("safe mode should defer offline render queue");
        assert_eq!(
            deferred.orchestration.decision,
            RuntimeDeferredServiceDecision::Defer
        );
        reports.push(RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        ));

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("disable safe mode");
        runtime
            .process_engine_block(
                2,
                2,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(12), 20),
            )
            .unwrap();
        reports.push(RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        ));

        let trace = RuntimeSupervisorReport::build_performance_trace_receipt(&reports);
        let performance_snapshots = reports
            .iter()
            .map(|report| report.performance_snapshot())
            .collect::<Vec<_>>();
        let expected_peak_cpu = reports
            .iter()
            .map(|report| report.performance_snapshot().cpu_load_percent)
            .fold(0.0f32, f32::max);
        let expected_peak_graph_latency = reports
            .iter()
            .map(|report| report.performance_snapshot().graph_latency_ms)
            .fold(0.0f32, f32::max);
        assert_eq!(trace.observation_count, reports.len());
        assert_eq!(trace.first_block_sequence, None);
        assert_eq!(trace.last_block_sequence, Some(2));
        assert_eq!(trace.processed_block_span, 2);
        assert_eq!(trace.peak_cpu_load_percent, expected_peak_cpu);
        assert_eq!(trace.peak_graph_latency_ms, expected_peak_graph_latency);
        assert!(trace.peak_block_execution_time_ns > 0);
        assert!(trace.playback_active_observation_count >= 3);
        assert!(trace.recording_active_observation_count >= 3);
        assert!(trace.background_service_defer_count >= 1);
        assert!(trace.background_service_while_playing_count >= 1);
        assert!(trace.background_service_while_recording_count >= 1);
        assert!(trace.background_starvation_observation_count >= 1);
        assert_eq!(trace.peak_background_starved_work_item_count, 1);
        assert_eq!(trace.background_cancellation_observation_count, 0);
        assert_eq!(trace.peak_background_cancelled_work_item_count, 0);
        assert_eq!(trace.background_realtime_backpressure_observation_count, 0);
        assert!(trace.background_recovery_backpressure_observation_count >= 1);
        assert_eq!(trace.peak_background_queued_work_item_count, 1);
        assert_eq!(trace.peak_background_deferred_work_item_count, 1);
        assert_eq!(trace.peak_hot_latency_node_id.as_deref(), Some("latency"));
        assert_eq!(trace.peak_hot_latency_node_samples, 24);
        let expected_peak_lane = performance_snapshots
            .iter()
            .max_by_key(|snapshot| snapshot.critical_path_lane_total_latency_samples)
            .expect("trace should have at least one performance snapshot");
        assert_eq!(
            trace.peak_hot_latency_group_node_count,
            expected_peak_lane.hot_latency_group_node_count
        );
        assert_eq!(
            trace.peak_critical_path_lane.as_deref(),
            expected_peak_lane.critical_path_lane.as_deref()
        );
        assert_eq!(
            trace.peak_critical_path_lane_node_count,
            expected_peak_lane.critical_path_lane_node_count
        );
        assert_eq!(
            trace.peak_critical_path_lane_plugin_backed_node_count,
            expected_peak_lane.critical_path_lane_plugin_backed_node_count
        );
        assert_eq!(
            trace.peak_critical_path_lane_total_latency_samples,
            expected_peak_lane.critical_path_lane_total_latency_samples
        );
        assert!(trace.summary.contains("recording_active="));
        assert!(trace.summary.contains("deadline="));
        assert!(trace.summary.contains("background="));
        assert!(trace.summary.contains("backpressure="));
        assert!(trace.summary.contains("starvation="));
        assert!(trace.summary.contains("critical_lane="));
        assert!(trace
            .render_json()
            .contains("\"peak_hot_latency_node_id\":\"latency\""));
        assert!(trace.render_json().contains("\"peak_critical_path_lane\":"));
        assert!(trace
            .render_json()
            .contains("\"peak_block_execution_time_ns\":"));
        assert!(trace
            .render_json()
            .contains("\"peak_background_starved_work_item_count\":1"));

        runtime.cancel_recording_capture().unwrap();
        let _ = fs::remove_file(capture_path);
    }

    #[test]
    fn runtime_forecast_profile_change_keeps_realtime_scheduler_coherent() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set initial realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-profile-change");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("switch forecast profile while running");

        let reprofiled = runtime.get_engine_block_snapshot();
        assert_eq!(
            reprofiled.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            reprofiled.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            reprofiled.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            reprofiled.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
        assert_eq!(
            reprofiled.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(reprofiled.prework_pending_target_count > 0);

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block after profile change")
            .snapshot;

        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
        assert!(snapshot
            .prework_cache_window_target_block_sequences
            .contains(&6));
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
        assert!(matches!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle | RuntimePreworkServiceState::Pending
        ));
    }

    #[test]
    fn runtime_mixed_execution_class_graph_transition_reuses_schedule_widened_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set widened mixed-graph policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:mixed-graph-before");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:mixed-graph-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        runtime.start().expect("start runtime");

        let before = runtime.get_engine_block_snapshot();
        assert_eq!(before.last_prework_service_requested_cycles, 3);
        assert_eq!(before.last_prework_service_effective_cycles, 3);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:mixed-graph-after".into(),
                node_count: 4,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "state".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .expect("apply mixed execution-class graph");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert!(snapshot.scheduler_topology.compatible);
        assert_eq!(snapshot.node_count, 4);
        assert_eq!(snapshot.plugin_backed_node_count, 1);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
        assert!(snapshot.prework_cache_queue_depth >= before.prework_cache_queue_depth);
    }

    #[test]
    fn runtime_mixed_execution_class_graph_churn_preserves_widened_scheduler_contract() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set mixed graph churn policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:mixed-graph-churn-a");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:mixed-graph-churn".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        runtime.start().expect("start runtime");

        let projections = vec![
            GraphProjection {
                graph_id: "graph:runtime:mixed-graph-churn-b".into(),
                node_count: 4,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "state".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 48,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            },
            GraphProjection {
                graph_id: "graph:runtime:mixed-graph-churn-c".into(),
                node_count: 5,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "state-a".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "inline-a".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.85 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency-a".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            },
        ];

        let mut last_invalidation_count = runtime
            .get_engine_block_snapshot()
            .prework_cache_invalidation_count;
        for projection in projections {
            let expected_node_count = projection.node_count;
            let expected_plugin_count = projection
                .nodes
                .iter()
                .filter(|node| node.execution_class == GraphNodeExecutionClass::PluginBacked)
                .count();
            runtime
                .apply_graph_projection(projection)
                .expect("apply mixed execution-class graph projection");
            let snapshot = runtime.get_engine_block_snapshot();

            assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
            assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
            assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
            assert_eq!(
                snapshot.last_prework_service_effective_budget_per_cycle,
                Some(3)
            );
            assert_eq!(snapshot.node_count, expected_node_count);
            assert_eq!(snapshot.plugin_backed_node_count, expected_plugin_count);
            assert!(snapshot.prework_cache_invalidation_count >= last_invalidation_count);
            last_invalidation_count = snapshot.prework_cache_invalidation_count;
        }
    }

    #[test]
    fn runtime_apply_forecast_state_primes_window_and_applies_current_block_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-advance".into(),
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

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: None,
            })
            .expect("set prework forecast profile");

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 2);

        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.tempo_bpm),
            Some(122.0)
        );
        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.timeline_position_samples),
            Some(0)
        );
        assert_eq!(
            runtime.latest_parameter_epoch,
            runtime
                .forecast_parameter_batch_for_block(
                    current_sequence,
                    &SignalRuntime::prework_forecast_policy_for_profile(
                        RuntimePreworkForecastProfileSelection {
                            profile: RuntimePreworkForecastProfile::Server,
                            target_window_blocks_override: None,
                        },
                    ),
                )
                .epoch
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));
    }

    #[test]
    fn runtime_reconfigure_uses_role_default_after_requested_mode_is_reset() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set prework forecast profile");
        assert_eq!(
            runtime.get_engine_block_snapshot().prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("reset requested mode to runtime role default");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-role-default-after-reconfigure".into(),
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

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("forecast apply should use runtime-role default");
        assert_eq!(admitted, 2);
    }

    #[test]
    fn runtime_selects_forecast_profile_with_target_window_override() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-profile-override".into(),
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

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set prework forecast profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 4);
    }

    #[test]
    fn runtime_configure_seeds_default_forecast_profile_from_runtime_role() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
    }

    #[test]
    fn runtime_can_disable_and_restore_role_default_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-mode-toggle".into(),
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

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
            .expect("disable prework forecast mode");
        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(disabled_snapshot.prework_forecast_policy_configured);
        assert_eq!(
            disabled_snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningDisabled)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state while disabled");
        assert_eq!(admitted, 0);

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("restore role-default forecast mode");
        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        let next_block_sequence = runtime.allocate_block_sequence();
        let restored_admitted = runtime
            .apply_forecast_state_for_block(2, next_block_sequence)
            .expect("apply forecast state after restore");
        assert_eq!(restored_admitted, 2);
    }

    #[test]
    fn runtime_retires_queued_prework_when_forecast_profile_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-change".into(),
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

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("switch explicit profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_pending_target_count, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
        assert_eq!(snapshot.prework_cache_queued_admissions, 4);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_rebuilds_missing_queued_prework_when_forecast_window_expands() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-expand".into(),
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

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("expand local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.prework_cache_retirement_count, 0);
    }

    #[test]
    fn runtime_forecast_plan_change_rebuild_uses_schedule_widened_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-change-schedule-widened".into(),
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
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:forecast-plan-change-widened".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        runtime.start().expect("start runtime");

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime role-default prework");
        assert!(admitted >= 2);
        let before = runtime.get_engine_block_snapshot();

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(6),
            })
            .expect("switch widened forecast profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert!((1..=3).contains(&snapshot.last_prework_service_cycle_count));
        assert!(
            snapshot.prework_cache_window_target_count > before.prework_cache_window_target_count
        );
        assert!(snapshot.prework_cache_invalidation_count >= 1);
        assert!(snapshot.prework_cache_retirement_count >= 1);
        assert!(snapshot.prework_cache_queue_depth >= before.prework_cache_queue_depth);
        assert!(snapshot.prework_pending_target_count <= before.prework_pending_target_count);
    }

    #[test]
    fn runtime_preserves_compatible_queued_prework_when_forecast_mode_changes_but_plan_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-compatible".into(),
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

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: None,
            })
            .expect("switch to explicit profile with matching plan");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_selectively_trims_queued_prework_when_forecast_window_shrinks() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-shrink".into(),
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

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(1),
            })
            .expect("shrink local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
    }

    #[test]
    fn runtime_configure_with_anticipative_disabled_enters_disabled_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, false);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("store explicit profile while anticipative planning is off");
        let explicit_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            explicit_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_retires_queued_prework_when_effective_mode_drops_to_disabled() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:disable-retire".into(),
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

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::RuntimeReconfigured)
        );
    }

    #[test]
    fn runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:auto-prime-on-graph-apply".into(),
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
            .expect("apply graph projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_start_rebuilds_prework_window_after_runtime_stop() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:restart-rebuild".into(),
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
            .expect("apply graph projection");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime.start().expect("start runtime");
        runtime
            .stop(StopReason::UserRequested)
            .expect("stop runtime");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            0
        );

        runtime.start().expect("restart runtime");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_reconfigure_preserves_explicit_forecast_profile_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set explicit forecast profile");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
    }

    #[test]
    fn runtime_restores_requested_explicit_forecast_mode_after_anticipative_reenable() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set explicit forecast profile");

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reenable anticipative");

        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_restart_preserves_raw_forecast_override_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime.start().expect("start runtime");

        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 5,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 11,
                transport_playing: true,
                transport_tempo_bpm: 130.0,
                transport_loop_length_blocks: 12,
                parameter_target: "engine.test.raw".into(),
                parameter_cycle_length: 9,
            })
            .expect("set raw forecast policy");

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart without reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(5)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride)
        );
    }

    #[test]
    fn runtime_prework_cache_expires_by_block_sequence_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:block-expiry".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(1, 2, block.clone()).unwrap();
        let third = runtime.process_engine_block(1, 3, block.clone()).unwrap();
        let fourth = runtime.process_engine_block(1, 4, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_consumptions, 3);
        assert_eq!(
            third.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Exhausted
        );
        assert_eq!(third.snapshot.prework_cache_remaining_valid_blocks, Some(0));
        assert_eq!(
            fourth.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::BlockSequenceExpired)
        );
        assert_eq!(
            fourth.snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::BlockSequenceExpired)
        );
        assert_eq!(fourth.snapshot.last_prework_retired_unconsumed, Some(false));
        assert_eq!(fourth.snapshot.prework_cache_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_consumed_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(fourth.snapshot.prework_cache_misses, 2);
        assert_eq!(
            fourth.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(fourth.snapshot.prework_cache_consumptions, 4);
        assert_eq!(
            fourth.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(
            fourth.snapshot.prework_cache_valid_until_block_sequence,
            Some(6)
        );
        assert_eq!(fourth.snapshot.last_prework_source_block_sequence, Some(4));
    }

    #[test]
    fn runtime_invalidates_prework_cache_on_parameter_and_transport_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:invalidate");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 21);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(1, 2, 1, block.clone(), None, None)
            .unwrap());

        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch().saturating_add(1),
                events: vec![ParameterEvent {
                    target: "invalidate.param".into(),
                    sample_offset: 0,
                    normalized_value: 0.25,
                }],
            })
            .unwrap();
        let after_parameter = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_parameter.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(after_parameter.last_prework_invalidation_reason, None);

        let second = runtime.process_engine_block(2, 2, block.clone()).unwrap();
        assert_eq!(second.snapshot.prework_cache_misses, 2);
        assert!(!second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 2);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(2, 3, 2, block.clone(), None, None)
            .unwrap());

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 130.0,
                loop_state: None,
            })
            .unwrap();
        let after_transport = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_transport.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        );
        assert_eq!(
            after_transport.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStarted)
        );
        assert_eq!(after_transport.prework_cache_invalidation_count, 2);
        assert_eq!(after_transport.prework_cache_retirement_count, 2);
        assert_eq!(
            after_transport.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::TransportStarted)
        );
        assert_eq!(after_transport.last_prework_retired_unconsumed, Some(false));
        assert_eq!(after_transport.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(after_transport.prework_cache_consumed_retirement_count, 2);
        assert_eq!(
            after_transport.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Invalidated
        );
        assert_eq!(
            after_transport.prework_cache_valid_until_processing_epoch,
            None
        );
    }

    #[test]
    fn runtime_invalidation_heavy_transition_stress_preserves_widened_scheduler_receipts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set transition stress policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:transition-stress");
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched:runtime:transition-stress".into(),
                stream_count: 3,
            })
            .expect("apply widened schedule projection");
        runtime.start().expect("start runtime");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 91);
        let transitions = vec![
            TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            },
            TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 120.0,
                loop_state: None,
            },
            TransportProjection {
                playing: true,
                timeline_position_samples: 520,
                tempo_bpm: 130.0,
                loop_state: None,
            },
            TransportProjection {
                playing: true,
                timeline_position_samples: 528,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            },
            TransportProjection {
                playing: false,
                timeline_position_samples: 536,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            },
        ];

        for (index, projection) in transitions.into_iter().enumerate() {
            runtime
                .apply_parameter_batch(ParameterBatch {
                    epoch: runtime.projection_epoch().saturating_add(50 + index as u64),
                    events: vec![ParameterEvent {
                        target: format!("stress.param.{index}"),
                        sample_offset: 0,
                        normalized_value: (index as f32) * 0.1,
                    }],
                })
                .expect("apply stress parameter batch");
            runtime
                .apply_transport_projection(projection)
                .expect("apply stress transport projection");

            let result = runtime
                .process_engine_block((index + 1) as u64, (index + 1) as u64, block.clone())
                .expect("process stress transition block");

            assert_eq!(
                result.snapshot.scheduler_topology.schedule_stream_count,
                Some(3)
            );
            assert_eq!(result.snapshot.last_prework_service_requested_cycles, 3);
            assert_eq!(result.snapshot.last_prework_service_effective_cycles, 3);
            assert_eq!(
                result
                    .snapshot
                    .last_prework_service_effective_budget_per_cycle,
                Some(3)
            );
        }

        let snapshot = runtime.get_engine_block_snapshot();
        assert!(snapshot.prework_cache_invalidation_count >= 5);
        assert!(snapshot.prework_cache_retirement_count >= 5);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Stopped)
        );
    }

    #[test]
    fn restart_reconfigures_runtime() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .restart(RestartRequest {
                reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
            })
            .unwrap();

        assert_eq!(runtime.get_effective_config().sample_rate.0, 44_100);
        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
    }

    #[test]
    fn transport_projection_rejects_non_positive_tempo() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 0.0,
                loop_state: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn runtime_classifies_transport_invalidation_boundaries() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-boundaries");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let started = runtime.get_engine_block_snapshot();
        assert_eq!(
            started.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStarted)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );

        runtime.process_engine_block(2, 2, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let seeked = runtime.get_engine_block_snapshot();
        assert_eq!(
            seeked.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportSeeked)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );

        runtime.process_engine_block(3, 3, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 520,
                tempo_bpm: 130.0,
                loop_state: None,
            })
            .unwrap();
        let tempo_changed = runtime.get_engine_block_snapshot();
        assert_eq!(
            tempo_changed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportTempoChanged)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::TempoChanged)
        );

        runtime.process_engine_block(4, 4, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 528,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            })
            .unwrap();
        let loop_state_changed = runtime.get_engine_block_snapshot();
        assert_eq!(
            loop_state_changed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportLoopStateChanged)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopStateChanged)
        );

        runtime.process_engine_block(5, 5, block).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 536,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            })
            .unwrap();
        let stopped = runtime.get_engine_block_snapshot();
        assert_eq!(
            stopped.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStopped)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Stopped)
        );
    }

    #[test]
    fn runtime_records_transport_progression_in_timeline_and_engine_snapshot() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-progression");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let result = runtime.process_engine_block(2, 2, block).unwrap();
        assert_eq!(result.snapshot.transport_epoch, 1);
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(result.snapshot.transport_block_start_samples, Some(64));
        assert_eq!(result.snapshot.transport_block_end_samples, Some(72));
        assert!(!result.snapshot.transport_loop_wrapped);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.transport_epoch, 1);
        assert_eq!(
            timeline.last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(timeline.last_transport_transition_block_sequence, Some(2));
        assert_eq!(timeline.last_transport_playing, Some(true));
        assert_eq!(timeline.last_transport_tempo_bpm, Some(120.0));
        assert_eq!(timeline.last_transport_timeline_position_samples, Some(72));
        assert_eq!(timeline.last_engine_block_start_samples, Some(64));
        assert_eq!(timeline.last_engine_block_end_samples, Some(72));
        assert_eq!(timeline.loop_wrap_count, 0);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        let compact = report.render_compact();
        assert!(compact.contains("transport_epoch=1"));
        assert!(compact.contains("engine_transport_transition=Some(Started)"));
        let json = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        )
        .render_json();
        assert!(json.contains("\"transport_epoch\":1"));
        assert!(json.contains("\"transport_transition\":\"Started\""));

        let transport = runtime.get_transport_observation_snapshot();
        assert_eq!(transport.transport_epoch, 1);
        assert_eq!(transport.projected_playing, Some(true));
        assert_eq!(transport.projected_tempo_bpm, Some(120.0));
        assert_eq!(transport.projected_timeline_position_samples, Some(72));
        assert_eq!(transport.observed_playing, Some(true));
        assert_eq!(transport.observed_tempo_bpm, Some(120.0));
        assert_eq!(transport.observed_timeline_position_samples, Some(72));
        assert_eq!(
            transport.last_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(transport.last_transition_block_sequence, Some(2));
        assert_eq!(transport.last_engine_block_start_samples, Some(64));
        assert_eq!(transport.last_engine_block_end_samples, Some(72));
        assert_eq!(transport.loop_wrap_count, 0);
    }

    #[test]
    fn runtime_seek_invalidation_projects_into_export_summaries_on_real_engine_path() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:seek-export");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime.process_engine_block(2, 2, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let boundary_report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(
            boundary_report
                .block_summary
                .last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportSeeked)
        );

        let result = runtime.process_engine_block(3, 3, block).unwrap();
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );
        assert_eq!(
            result.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
        );

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(
            report.block_summary.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );
        assert_eq!(
            report.block_summary.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
        );

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_transition=Some(Seeked)"));
        let json = supervisor.render_json();
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"transport_transition\":\"Seeked\""));
        assert!(json.contains("\"last_prework_invalidation_reason\":\"ProcessingEpochExpired\""));
    }

    #[test]
    fn runtime_records_loop_wrap_as_transport_boundary() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:loop-wrap");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 51);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 60,
                tempo_bpm: 120.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 32,
                    end_samples: 68,
                }),
            })
            .unwrap();

        let result = runtime.process_engine_block(2, 2, block).unwrap();
        assert_eq!(result.snapshot.transport_epoch, 2);
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(result.snapshot.transport_block_start_samples, Some(60));
        assert_eq!(result.snapshot.transport_block_end_samples, Some(32));
        assert!(result.snapshot.transport_loop_wrapped);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.transport_epoch, 2);
        assert_eq!(
            timeline.last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopWrapped)
        );
        assert_eq!(timeline.last_transport_transition_processing_epoch, Some(2));
        assert_eq!(timeline.last_transport_transition_block_sequence, Some(2));
        assert_eq!(timeline.last_transport_timeline_position_samples, Some(32));
        assert_eq!(timeline.last_engine_block_start_samples, Some(60));
        assert_eq!(timeline.last_engine_block_end_samples, Some(32));
        assert_eq!(timeline.loop_wrap_count, 1);

        let transport = runtime.get_transport_observation_snapshot();
        assert_eq!(transport.transport_epoch, 2);
        assert_eq!(transport.projected_timeline_position_samples, Some(32));
        assert_eq!(transport.observed_timeline_position_samples, Some(32));
        assert_eq!(
            transport.last_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopWrapped)
        );
        assert_eq!(transport.last_transition_processing_epoch, Some(2));
        assert_eq!(transport.last_transition_block_sequence, Some(2));
        assert_eq!(transport.last_engine_block_start_samples, Some(60));
        assert_eq!(transport.last_engine_block_end_samples, Some(32));
        assert_eq!(transport.loop_wrap_count, 1);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(report.block_summary.transport_loop_wrapped);
        assert_eq!(report.block_summary.transport_epoch, 2);
        assert!(report
            .render_compact()
            .contains("block_summary_transport=2/Some(Started)/true"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_loop_wrapped=true"));
        let json = supervisor.render_json();
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"transport_loop_wrapped\":true"));
    }

    #[test]
    fn runtime_recording_capture_buffers_output_and_commits_wav() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-capture");

        let capture_path = temp_capture_path("recording-capture");
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 2_048,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:0001".to_string(),
                track_id: "track:test:0001".to_string(),
                start_samples: 2_048,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();

        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(16), 77),
            )
            .unwrap();

        let recording = runtime.get_recording_capture_snapshot();
        assert!(recording.capture_ready);
        assert_eq!(
            recording.state,
            Some(RuntimeRecordingCaptureState::Capturing)
        );
        assert_eq!(
            recording.capture_kind,
            Some(RuntimeRecordingCaptureKind::Audio)
        );
        assert_eq!(recording.active_take_id.as_deref(), Some("take:test:0001"));
        assert_eq!(recording.buffered_block_count, 1);
        assert_eq!(recording.buffered_frame_count, 16);
        assert_eq!(recording.buffered_event_count, 0);
        assert_eq!(recording.captured_channel_count, 2);
        assert_eq!(
            recording
                .active_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class),
            Some(RuntimeRecordingCaptureCheckpointClass::Streaming)
        );

        let receipt = runtime.finish_recording_capture().unwrap();
        assert_eq!(receipt.capture_kind, RuntimeRecordingCaptureKind::Audio);
        assert_eq!(receipt.take_id, "take:test:0001");
        assert_eq!(receipt.duration_samples, 16);
        assert_eq!(receipt.channel_count, 2);
        assert_eq!(
            receipt.committed_checkpoint.checkpoint_class,
            RuntimeRecordingCaptureCheckpointClass::Committed
        );
        assert!(capture_path.exists());

        let committed = runtime.get_recording_capture_snapshot();
        assert_eq!(committed.state, Some(RuntimeRecordingCaptureState::Idle));
        assert_eq!(
            committed.last_committed_path.as_deref(),
            Some(capture_path.to_string_lossy().as_ref())
        );
        assert_eq!(committed.last_committed_duration_samples, Some(16));
        assert_eq!(
            committed
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class),
            Some(RuntimeRecordingCaptureCheckpointClass::Committed)
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.recording_capture_snapshot.capture_kind,
            Some(RuntimeRecordingCaptureKind::Audio)
        );
        let observation_json = observation.render_json();
        assert!(observation_json.contains("\"recording_capture_snapshot\":{"));
        assert!(observation_json.contains("\"checkpoint_class\":\"Committed\""));

        let _ = fs::remove_file(capture_path);
    }

    #[test]
    fn runtime_recording_capture_cancels_without_committing_file() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-cancel");

        let capture_path = temp_capture_path("recording-cancel");
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:cancel".to_string(),
                track_id: "track:test:cancel".to_string(),
                start_samples: 512,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 33),
            )
            .unwrap();
        runtime.cancel_recording_capture().unwrap();

        let recording = runtime.get_recording_capture_snapshot();
        assert_eq!(recording.state, Some(RuntimeRecordingCaptureState::Idle));
        assert_eq!(recording.active_take_id, None);
        assert_eq!(recording.last_committed_path, None);
        assert_eq!(
            recording
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class),
            Some(RuntimeInterruptionClass::Restartable)
        );
        assert!(!capture_path.exists());
    }

    #[test]
    fn runtime_recording_capture_preserves_restartable_checkpoint_across_stop_and_reconfigure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-restartable");
        runtime.start().unwrap();

        let capture_path = temp_capture_path("recording-restartable");
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:restartable".to_string(),
                track_id: "track:test:restartable".to_string(),
                start_samples: 1_024,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(12), 91),
            )
            .unwrap();

        runtime.stop(StopReason::DeviceReconfigure).unwrap();
        runtime
            .configure(RuntimeConfigRequest {
                sample_rate: SampleRate(48_000),
                block_size: 256,
                anticipative_enabled: true,
                realtime_safe_mode: false,
                max_graph_latency_ms: None,
                max_background_load_percent: None,
            })
            .unwrap();

        let recording = runtime.get_recording_capture_snapshot();
        assert_eq!(recording.state, Some(RuntimeRecordingCaptureState::Idle));
        assert_eq!(
            recording.capture_kind,
            Some(RuntimeRecordingCaptureKind::Audio)
        );
        assert_eq!(
            recording
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class),
            Some(RuntimeRecordingCaptureCheckpointClass::Buffered)
        );
        assert_eq!(
            recording
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class),
            Some(RuntimeInterruptionClass::Restartable)
        );
        assert_eq!(
            recording
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.buffered_frame_count),
            Some(12)
        );
        assert_eq!(recording.last_committed_path, None);
    }

    #[test]
    fn runtime_recording_capture_resumes_same_identity_after_safe_mode_clears() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-resumable");
        runtime.start().unwrap();

        let capture_path = temp_capture_path("recording-resumable");
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:resumable".to_string(),
                track_id: "track:test:resumable".to_string(),
                start_samples: 4_096,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(10), 55),
            )
            .unwrap();

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();
        let resumable =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            resumable
                .recording_capture_snapshot
                .active_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class),
            Some(RuntimeInterruptionClass::Resumable)
        );
        assert_eq!(
            resumable
                .recording_capture_snapshot
                .active_take_id
                .as_deref(),
            Some("take:test:resumable")
        );

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .unwrap();
        runtime
            .process_engine_block(
                2,
                2,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(6), 56),
            )
            .unwrap();
        let receipt = runtime.finish_recording_capture().unwrap();
        assert_eq!(receipt.take_id, "take:test:resumable");
        assert_eq!(receipt.duration_samples, 16);

        let recording = runtime.get_recording_capture_snapshot();
        assert_eq!(
            recording
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class),
            Some(RuntimeRecordingCaptureCheckpointClass::Committed)
        );
        assert_eq!(
            recording.last_committed_take_id.as_deref(),
            Some("take:test:resumable")
        );

        let _ = fs::remove_file(capture_path);
    }

    #[test]
    fn runtime_recording_capture_reports_terminal_checkpoint_on_commit_failure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-terminal");

        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:test:terminal".to_string(),
                track_id: "track:test:terminal".to_string(),
                start_samples: 2_560,
                capture_path: "/dev/null/signal-runtime-recording-terminal.wav".to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 71),
            )
            .unwrap();

        let error = runtime.finish_recording_capture().unwrap_err();
        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);

        let failed = runtime.get_recording_capture_snapshot();
        assert_eq!(failed.state, Some(RuntimeRecordingCaptureState::Failed));
        assert_eq!(failed.active_take_id, None);
        assert_eq!(
            failed
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class),
            Some(RuntimeRecordingCaptureCheckpointClass::Failed)
        );
        assert_eq!(
            failed
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class),
            Some(RuntimeInterruptionClass::Terminal)
        );
    }

    #[test]
    fn runtime_reconciles_media_assets_into_shared_ready_cache_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("media-imported");
        let recorded_path = temp_capture_path("media-recorded");
        write_test_wav(&imported_path);
        write_test_wav(&recorded_path);

        runtime
            .reconcile_media_assets(vec![
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:imported".to_string(),
                    content_hash: "imported".to_string(),
                    source_path: imported_path.display().to_string(),
                    file_name: "imported.wav".to_string(),
                    byte_size: fs::metadata(&imported_path).unwrap().len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:recorded".to_string(),
                    content_hash: "recorded".to_string(),
                    source_path: recorded_path.display().to_string(),
                    file_name: "recorded.wav".to_string(),
                    byte_size: fs::metadata(&recorded_path).unwrap().len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
            ])
            .unwrap();

        let snapshot = runtime.get_media_pipeline_snapshot();
        assert_eq!(snapshot.asset_count, 2);
        assert_eq!(snapshot.ready_asset_count, 2);
        assert_eq!(snapshot.invalid_asset_count, 0);
        assert!(snapshot.assets.iter().all(|asset| {
            asset.state == Some(RuntimeMediaAssetState::Ready)
                && asset.cache_path.as_deref().is_some()
        }));

        let cached_path = PathBuf::from(
            snapshot.assets[0]
                .cache_path
                .as_deref()
                .expect("cached media should exist"),
        );
        fs::remove_file(&cached_path).unwrap();

        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:imported".to_string(),
                content_hash: "imported".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "imported.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();

        let rebuilt = runtime.get_media_pipeline_snapshot();
        assert_eq!(rebuilt.asset_count, 1);
        assert_eq!(rebuilt.ready_asset_count, 1);
        assert_eq!(rebuilt.assets[0].state, Some(RuntimeMediaAssetState::Ready));
        assert!(rebuilt.assets[0].rebuild_count >= 1);

        let _ = fs::remove_file(imported_path);
        let _ = fs::remove_file(recorded_path);
        if let Some(path) = rebuilt.assets[0].cache_path.as_deref() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn runtime_media_service_snapshot_tracks_ready_previewable_and_invalidated_assets() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let ready_path = temp_capture_path("media-service-ready");
        let missing_path = temp_capture_path("media-service-missing");
        write_test_wav(&ready_path);

        runtime
            .reconcile_media_assets(vec![
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:ready".to_string(),
                    content_hash: "ready".to_string(),
                    source_path: ready_path.display().to_string(),
                    file_name: "ready.wav".to_string(),
                    byte_size: fs::metadata(&ready_path).unwrap().len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:missing".to_string(),
                    content_hash: "missing".to_string(),
                    source_path: missing_path.display().to_string(),
                    file_name: "missing.wav".to_string(),
                    byte_size: 0,
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
            ])
            .unwrap();

        let service = runtime.get_media_service_snapshot();
        assert_eq!(service.indexed_asset_count, 2);
        assert_eq!(service.analysis_ready_asset_count, 1);
        assert_eq!(service.waveform_ready_asset_count, 1);
        assert_eq!(service.waveform_pending_asset_count, 0);
        assert_eq!(service.previewable_asset_count, 1);
        assert_eq!(service.invalidated_asset_count, 1);
        assert!(service.invalidation_active);
        assert_eq!(
            service.indexing_state,
            crate::interfaces::RuntimeMediaIndexingState::Invalidated
        );
        assert_eq!(
            service.preview_state,
            crate::interfaces::RuntimeMediaPreviewState::Ready
        );
        assert_eq!(
            service.last_invalidated_asset_id.as_deref(),
            Some("asset:sha256:missing")
        );
        assert!(service.last_invalidation_error.is_some());

        let library = runtime.get_media_library_service_snapshot();
        assert_eq!(library.indexed_asset_count, 2);
        assert_eq!(library.ready_descriptor_count, 1);
        assert_eq!(library.invalidated_descriptor_count, 1);
        assert_eq!(library.unavailable_descriptor_count, 0);
        assert_eq!(library.loudness_ready_descriptor_count, 1);
        assert_eq!(library.character_ready_descriptor_count, 1);
        let ready = library
            .descriptors
            .iter()
            .find(|descriptor| descriptor.asset_id == "asset:sha256:ready")
            .expect("ready descriptor");
        assert_eq!(
            ready.metadata_state,
            crate::RuntimeMediaAnalysisDescriptorState::Ready
        );
        assert_eq!(
            ready.loudness_state,
            crate::RuntimeMediaAnalysisFamilyState::Ready
        );
        assert_eq!(
            ready.character_state,
            crate::RuntimeMediaAnalysisFamilyState::Ready
        );
        assert!(ready.loudness.is_some());
        assert!(ready.character.is_some());
        let missing = library
            .descriptors
            .iter()
            .find(|descriptor| descriptor.asset_id == "asset:sha256:missing")
            .expect("missing descriptor");
        assert_eq!(
            missing.metadata_state,
            crate::RuntimeMediaAnalysisDescriptorState::Invalidated
        );

        let _ = fs::remove_file(ready_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "asset:sha256:ready")
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn runtime_media_preview_clears_when_previewed_asset_is_invalidated() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let ready_path = temp_capture_path("media-preview-ready");
        write_test_wav(&ready_path);

        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:previewed".to_string(),
                content_hash: "previewed".to_string(),
                source_path: ready_path.display().to_string(),
                file_name: "previewed.wav".to_string(),
                byte_size: fs::metadata(&ready_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();

        runtime
            .start_media_preview("asset:sha256:previewed")
            .expect("preview should start for ready media");
        let previewing = runtime.get_media_service_snapshot();
        assert_eq!(
            previewing.preview_state,
            crate::RuntimeMediaPreviewState::Previewing
        );
        assert_eq!(
            previewing.previewing_asset_id.as_deref(),
            Some("asset:sha256:previewed")
        );

        fs::remove_file(&ready_path).unwrap();
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:previewed".to_string(),
                content_hash: "previewed".to_string(),
                source_path: ready_path.display().to_string(),
                file_name: "previewed.wav".to_string(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();

        let invalidated = runtime.get_media_service_snapshot();
        assert_eq!(
            invalidated.preview_state,
            crate::RuntimeMediaPreviewState::Invalidated
        );
        assert_eq!(invalidated.previewing_asset_id, None);
        assert!(invalidated.last_preview_error.is_some());
    }

    #[test]
    fn runtime_media_service_recovers_after_invalidation_and_supports_preview_again() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let ready_path = temp_capture_path("media-preview-recovered");
        write_test_wav(&ready_path);

        let registration = RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:recoverable".to_string(),
            content_hash: "recoverable".to_string(),
            source_path: ready_path.display().to_string(),
            file_name: "recoverable.wav".to_string(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        };

        runtime
            .reconcile_media_assets(vec![registration.clone()])
            .expect("ready media should reconcile");
        runtime
            .start_media_preview("asset:sha256:recoverable")
            .expect("preview should start for ready media");
        assert_eq!(
            runtime.get_media_service_snapshot().preview_state,
            crate::RuntimeMediaPreviewState::Previewing
        );

        fs::remove_file(&ready_path).expect("source media should be removable");
        runtime
            .reconcile_media_assets(vec![registration.clone()])
            .expect("missing media should reconcile as invalid");

        let invalidated = runtime.get_media_service_snapshot();
        assert_eq!(
            invalidated.preview_state,
            crate::RuntimeMediaPreviewState::Invalidated
        );
        assert_eq!(invalidated.previewing_asset_id, None);
        assert_eq!(invalidated.invalidated_asset_count, 1);

        write_test_wav(&ready_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                byte_size: fs::metadata(&ready_path).unwrap().len(),
                ..registration
            }])
            .expect("restored media should reconcile");

        let recovered = runtime.get_media_service_snapshot();
        assert_eq!(
            recovered.indexing_state,
            crate::RuntimeMediaIndexingState::Ready
        );
        assert_eq!(
            recovered.preview_state,
            crate::RuntimeMediaPreviewState::Ready
        );
        assert_eq!(recovered.invalidated_asset_count, 0);
        assert_eq!(recovered.previewing_asset_id, None);
        assert_eq!(recovered.last_invalidated_asset_id, None);

        runtime
            .start_media_preview("asset:sha256:recoverable")
            .expect("preview should restart after recovery");
        let previewing_again = runtime.get_media_service_snapshot();
        assert_eq!(
            previewing_again.preview_state,
            crate::RuntimeMediaPreviewState::Previewing
        );
        assert_eq!(
            previewing_again.previewing_asset_id.as_deref(),
            Some("asset:sha256:recoverable")
        );

        let _ = fs::remove_file(ready_path);
    }

    #[test]
    fn runtime_observation_and_supervisor_reports_surface_media_service_baseline() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let ready_path = temp_capture_path("media-observation-preview");
        write_test_wav(&ready_path);

        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:observation".to_string(),
                content_hash: "observation".to_string(),
                source_path: ready_path.display().to_string(),
                file_name: "observation.wav".to_string(),
                byte_size: fs::metadata(&ready_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            }])
            .expect("ready media should reconcile");
        runtime
            .start_media_preview("asset:sha256:observation")
            .expect("preview should start for ready media");

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.media_pipeline_snapshot.asset_count, 1);
        assert_eq!(observation.media_pipeline_snapshot.ready_asset_count, 1);
        assert_eq!(observation.media_service_snapshot.indexed_asset_count, 1);
        assert_eq!(
            observation
                .media_service_snapshot
                .waveform_ready_asset_count,
            1
        );
        assert_eq!(
            observation.media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Previewing
        );
        assert_eq!(
            observation
                .media_service_snapshot
                .previewing_asset_id
                .as_deref(),
            Some("asset:sha256:observation")
        );
        assert_eq!(observation.media_library_snapshot.indexed_asset_count, 1);
        assert_eq!(observation.media_library_snapshot.ready_descriptor_count, 1);
        assert_eq!(
            observation
                .media_library_snapshot
                .loudness_ready_descriptor_count,
            1
        );
        assert_eq!(
            observation
                .media_library_snapshot
                .character_ready_descriptor_count,
            1
        );
        assert_eq!(
            observation.media_library_snapshot.descriptors[0].metadata_state,
            crate::RuntimeMediaAnalysisDescriptorState::Ready
        );
        assert!(observation.media_library_snapshot.descriptors[0]
            .loudness
            .is_some());
        assert!(observation.media_library_snapshot.descriptors[0]
            .character
            .is_some());

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("media_asset_count=1"));
        assert!(multiline.contains("media_preview_state=Previewing"));
        assert!(multiline.contains("media_library_ready_descriptor_count=1"));

        let json = supervisor.render_json();
        assert!(json.contains("\"media_pipeline_snapshot\":{"));
        assert!(json.contains("\"media_service_snapshot\":{"));
        assert!(json.contains("\"media_library_snapshot\":{"));
        assert!(json.contains("\"preview_state\":\"Previewing\""));
        assert!(json.contains("\"waveform_ready_asset_count\":1"));
        assert!(json.contains("\"ready_descriptor_count\":1"));

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
    fn runtime_observation_and_supervisor_reports_surface_external_midi_endpoint_baseline() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.external_midi_snapshot.discovery_state,
            crate::RuntimeExternalMidiDiscoveryState::Unavailable
        );
        assert_eq!(
            observation.external_midi_snapshot.graph_state,
            crate::RuntimeExternalMidiGraphState::Unavailable
        );
        assert_eq!(observation.external_midi_snapshot.device_count, 0);
        assert_eq!(observation.external_midi_snapshot.endpoint_count, 0);

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("external_midi_discovery_state=Unavailable"));
        assert!(multiline.contains("external_midi_graph_state=Unavailable"));

        let json = supervisor.render_json();
        assert!(json.contains("\"external_midi_snapshot\":{"));
        assert!(json.contains("\"discovery_state\":\"Unavailable\""));
        assert!(json.contains("\"graph_state\":\"Unavailable\""));
        assert!(json.contains("\"provider_name\":\"runtime-unavailable\""));
    }

    #[test]
    fn runtime_acceptance_receipt_scopes_integrated_runtime_lanes_and_targets() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:acceptance-scope");

        let ready_path = temp_capture_path("acceptance-media-ready");
        write_test_wav(&ready_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:acceptance".to_string(),
                content_hash: "acceptance".to_string(),
                source_path: ready_path.display().to_string(),
                file_name: "acceptance.wav".to_string(),
                byte_size: fs::metadata(&ready_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .expect("ready media should reconcile");
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                capture_kind: RuntimeRecordingCaptureKind::Audio,
                take_id: "take:acceptance".to_string(),
                track_id: "track:acceptance".to_string(),
                start_samples: 0,
                capture_path: temp_capture_path("acceptance-take").display().to_string(),
            })
            .expect("recording capture should start");

        let receipt = runtime.get_acceptance_receipt();
        assert_eq!(receipt.runtime_lane_count, 6);
        assert!(receipt.playback_ready);
        assert!(receipt.recording_ready);
        assert!(receipt.media_ready);
        assert!(!receipt.clip_processing_ready);
        assert!(!receipt.plugin_ready);
        assert!(receipt.recovery_ready);
        assert_eq!(receipt.minimum_trace_observation_count, 128);
        assert_eq!(receipt.minimum_soak_event_count, 64);
        assert_eq!(receipt.runtime_ready_lane_count, 4);

        let _ = fs::remove_file(ready_path);
    }

    #[test]
    fn runtime_reconciles_warp_clips_against_media_readiness_and_project_tempo() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("warp-ready");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:warp-ready".to_string(),
                content_hash: "warp-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "warp-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:warp-ready".to_string(),
                media_asset_id: Some("asset:sha256:warp-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let ready = runtime.get_warp_pipeline_snapshot();
        assert_eq!(ready.clip_count, 1);
        assert_eq!(ready.ready_clip_count, 1);
        assert_eq!(ready.degraded_clip_count, 0);
        assert_eq!(
            ready.resolved_project_tempo_source,
            RuntimeTempoSource::TransportProjection
        );
        assert_eq!(ready.clips[0].readiness, RuntimeWarpReadiness::Ready);
        assert_eq!(
            ready.clips[0].project_tempo_source,
            RuntimeTempoSource::TransportProjection
        );
        assert!((ready.clips[0].realized_ratio - 1.5).abs() < 0.000_1);

        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 300.0,
                loop_state: None,
            })
            .unwrap();
        let degraded = runtime.get_warp_pipeline_snapshot();
        assert_eq!(degraded.ready_clip_count, 0);
        assert_eq!(degraded.degraded_clip_count, 1);
        assert_eq!(
            degraded.resolved_project_tempo_source,
            RuntimeTempoSource::TransportProjection
        );
        assert_eq!(degraded.clips[0].readiness, RuntimeWarpReadiness::Degraded);
        assert!(degraded.clips[0]
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("outside baseline support"));

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
    fn runtime_reconciles_clip_processing_against_media_and_warp_readiness() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("clip-processing-ready");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:clip-processing-ready".to_string(),
                content_hash: "clip-processing-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "clip-processing-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:processing-ready".to_string(),
                media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:processing-ready".to_string(),
                media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope {
                    duration_samples: 2_048,
                    shape: RuntimeClipFadeShape::SmoothStep,
                },
                fade_out: RuntimeClipFadeEnvelope {
                    duration_samples: 4_096,
                    shape: RuntimeClipFadeShape::EqualPower,
                },
                clip_gain: RuntimeClipGainEnvelope {
                    start_linear: 0.82,
                    end_linear: 0.64,
                    shape: RuntimeClipGainShape::Linear,
                },
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let ready = runtime.get_clip_processing_pipeline_snapshot();
        assert_eq!(ready.clip_count, 1);
        assert_eq!(ready.ready_clip_count, 1);
        assert_eq!(ready.pending_media_clip_count, 0);
        assert_eq!(ready.pending_warp_clip_count, 0);
        assert_eq!(ready.invalid_clip_count, 0);
        assert_eq!(ready.faded_clip_count, 1);
        assert_eq!(ready.gain_shaped_clip_count, 1);
        assert_eq!(ready.warped_clip_count, 1);
        assert_eq!(ready.treatment_stage_count, 4);
        assert_eq!(
            ready.clips[0].readiness,
            RuntimeClipProcessingReadiness::Ready
        );
        assert_eq!(ready.clips[0].fade_in_end_samples, 2_048);
        assert_eq!(ready.clips[0].fade_out_start_samples, 43_904);
        assert_eq!(
            ready.clips[0].treatment_stages,
            vec![
                RuntimeClipProcessingStage::Warp,
                RuntimeClipProcessingStage::FadeIn,
                RuntimeClipProcessingStage::GainShape,
                RuntimeClipProcessingStage::FadeOut,
            ]
        );
        assert_eq!(
            ready.clips[0].fade_in.shape,
            RuntimeClipFadeShape::SmoothStep
        );
        assert_eq!(
            ready.clips[0].fade_out.shape,
            RuntimeClipFadeShape::EqualPower
        );
        assert_eq!(ready.clips[0].clip_gain.shape, RuntimeClipGainShape::Linear);
        assert!((ready.clips[0].clip_gain.start_linear - 0.82).abs() < f32::EPSILON);
        assert!((ready.clips[0].clip_gain.end_linear - 0.64).abs() < f32::EPSILON);
        assert_eq!(
            ready.clips[0].project_tempo_source,
            Some(RuntimeTempoSource::TransportProjection)
        );
        assert!((ready.clips[0].realized_warp_ratio.unwrap_or_default() - 1.5).abs() < 0.000_1);

        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 300.0,
                loop_state: None,
            })
            .unwrap();

        let invalid = runtime.get_clip_processing_pipeline_snapshot();
        assert_eq!(invalid.clip_count, 1);
        assert_eq!(invalid.ready_clip_count, 0);
        assert_eq!(invalid.invalid_clip_count, 1);
        assert_eq!(
            invalid.clips[0].readiness,
            RuntimeClipProcessingReadiness::Invalid
        );
        assert!(invalid.clips[0]
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("outside baseline support"));

        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:processing-ready".to_string(),
                media_asset_id: Some("asset:sha256:clip-processing-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope {
                    duration_samples: 2_048,
                    shape: RuntimeClipFadeShape::Linear,
                },
                fade_out: RuntimeClipFadeEnvelope {
                    duration_samples: 4_096,
                    shape: RuntimeClipFadeShape::Linear,
                },
                clip_gain: RuntimeClipGainEnvelope {
                    start_linear: 0.82,
                    end_linear: 0.64,
                    shape: RuntimeClipGainShape::Hold,
                },
            }])
            .unwrap();
        let invalid_gain_shape = runtime.get_clip_processing_pipeline_snapshot();
        assert_eq!(invalid_gain_shape.invalid_clip_count, 1);
        assert_eq!(
            invalid_gain_shape.clips[0].readiness,
            RuntimeClipProcessingReadiness::Invalid
        );
        assert!(invalid_gain_shape.clips[0]
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("hold clip gain shape requires identical start and end gain"));

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
    fn runtime_observation_clip_render_and_offline_render_preview_surface_stretch_engine_receipts()
    {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("stretch-engine-ready");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:stretch-engine-ready".to_string(),
                content_hash: "stretch-engine-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "stretch-engine-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:stretch-engine-ready".to_string(),
                media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:stretch-engine-ready".to_string(),
                media_asset_id: Some("asset:sha256:stretch-engine-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                clip_gain: RuntimeClipGainEnvelope::default(),
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.stretch_engine_snapshot.clip_count, 1);
        assert_eq!(observation.stretch_engine_snapshot.ready_clip_count, 1);
        assert_eq!(
            observation.stretch_engine_snapshot.sample_domain_clip_count,
            1
        );
        assert_eq!(observation.stretch_engine_snapshot.fallback_clip_count, 0);
        assert_eq!(
            observation.stretch_engine_snapshot.clips[0].engine_class,
            RuntimeStretchEngineClass::SampleDomain
        );
        assert_eq!(
            observation.stretch_engine_snapshot.clips[0].readiness,
            RuntimeStretchReadiness::Ready
        );
        assert_eq!(
            observation.stretch_engine_snapshot.clips[0].fallback_kind,
            RuntimeStretchFallbackKind::None
        );
        assert!(observation
            .render_compact()
            .contains("stretch_clips=1/1/1/0/0/0/0/0"));
        assert!(observation
            .render_json()
            .contains("\"stretch_engine_snapshot\":{\"clip_count\":1"));

        let rendered = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:stretch-engine-ready".to_string(),
                timeline_start_samples: 0,
                input_stage: RuntimeClipRenderInputStage::PostWarp,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![0.5; 8],
                ),
            })
            .unwrap();
        assert_eq!(
            rendered.stretch_engine_snapshot.engine_class,
            RuntimeStretchEngineClass::SampleDomain
        );
        assert_eq!(
            rendered.stretch_engine_snapshot.readiness,
            RuntimeStretchReadiness::Ready
        );
        assert_eq!(
            rendered.stretch_engine_snapshot.fallback_kind,
            RuntimeStretchFallbackKind::None
        );
        assert!(rendered.summary.contains("stretch=SampleDomain/Ready/None"));

        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &RuntimeOfflineRenderRequest {
                request_id: "render:stretch-engine-preview".into(),
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
        .expect("build stretch engine offline render preview");
        assert_eq!(preview.stretch_engine_snapshot.clip_count, 1);
        assert_eq!(preview.stretch_engine_snapshot.ready_clip_count, 1);
        assert_eq!(preview.stretch_engine_snapshot.sample_domain_clip_count, 1);
        assert_eq!(preview.stretch_engine_snapshot.fallback_clip_count, 0);
        assert_eq!(
            preview.stretch_engine_snapshot.clips[0].engine_class,
            RuntimeStretchEngineClass::SampleDomain
        );
        assert_eq!(
            preview.stretch_engine_snapshot.clips[0].readiness,
            RuntimeStretchReadiness::Ready
        );
        assert!(preview.summary.contains("stretch=1/fallback=0"));

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
    fn runtime_marker_analysis_snapshot_derives_from_stretch_and_media_baselines() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("marker-analysis-ready");
        write_transient_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:marker-analysis-ready".to_string(),
                content_hash: "marker-analysis-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "marker-analysis-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 48_000,
                waveform_bin_count: 32,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:marker-analysis-ready".to_string(),
                media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:marker-analysis-ready".to_string(),
                media_asset_id: Some("asset:sha256:marker-analysis-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                clip_gain: RuntimeClipGainEnvelope::default(),
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let marker_analysis = runtime.get_marker_analysis_snapshot();
        assert_eq!(marker_analysis.clip_count, 1);
        assert_eq!(marker_analysis.ready_clip_count, 1);
        assert_eq!(marker_analysis.pending_media_clip_count, 0);
        assert_eq!(marker_analysis.degraded_clip_count, 0);
        assert_eq!(marker_analysis.invalidated_clip_count, 0);
        assert_eq!(marker_analysis.unsupported_clip_count, 0);
        assert_eq!(marker_analysis.tempo_assist_ready_clip_count, 1);
        assert!(marker_analysis.warp_marker_count > 0);
        assert!(marker_analysis.transient_anchor_count > 0);
        assert_eq!(
            marker_analysis.clips[0].readiness,
            RuntimeMarkerAnalysisReadiness::Ready
        );
        assert_eq!(
            marker_analysis.clips[0].tempo_assist_posture,
            RuntimeTempoAssistPosture::Ready
        );
        assert_eq!(
            marker_analysis.clips[0].tempo_assist_hint_source,
            RuntimeTempoAssistHintSource::SourceTempo
        );
        assert_eq!(marker_analysis.clips[0].tempo_assist_hint_bpm, Some(120.0));

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.marker_analysis_snapshot.clip_count, 1);
        assert_eq!(observation.marker_analysis_snapshot.ready_clip_count, 1);
        assert_eq!(
            observation
                .marker_analysis_snapshot
                .tempo_assist_ready_clip_count,
            1
        );
        assert!(observation
            .render_compact()
            .contains("marker_analysis_clips=1/1/0/0/0"));
        assert!(observation
            .render_json()
            .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("marker_analysis_clip_count=1"));
        assert!(multiline.contains("marker_analysis_tempo_assist_ready_clip_count=1"));
        assert!(supervisor
            .render_json()
            .contains("\"marker_analysis_snapshot\":{\"clip_count\":1"));

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
    fn runtime_transform_artifact_snapshot_derives_from_stretch_and_marker_analysis_baselines() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("transform-artifact-ready");
        write_transient_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:transform-artifact-ready".to_string(),
                content_hash: "transform-artifact-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "transform-artifact-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 48_000,
                waveform_bin_count: 32,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:transform-artifact-ready".to_string(),
                media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:transform-artifact-ready".to_string(),
                media_asset_id: Some("asset:sha256:transform-artifact-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                clip_gain: RuntimeClipGainEnvelope::default(),
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let transform_artifact = runtime.get_transform_artifact_snapshot();
        assert_eq!(transform_artifact.clip_count, 1);
        assert_eq!(transform_artifact.ready_clip_count, 1);
        assert_eq!(transform_artifact.pending_media_clip_count, 0);
        assert_eq!(transform_artifact.degraded_clip_count, 0);
        assert_eq!(transform_artifact.invalidated_clip_count, 0);
        assert_eq!(transform_artifact.unsupported_clip_count, 0);
        assert_eq!(transform_artifact.cached_media_ready_clip_count, 1);
        assert_eq!(transform_artifact.reusable_clip_count, 1);
        assert_eq!(transform_artifact.requires_render_clip_count, 0);
        assert_eq!(transform_artifact.guarded_reuse_clip_count, 0);
        assert_eq!(
            transform_artifact.transform_persistence.persistence_posture,
            RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .retention_policy_class,
            RuntimeTransformRetentionPolicyClass::AssetLifetimeRetentionPolicy
        );
        assert_eq!(
            transform_artifact.transform_persistence.retention_authority,
            RuntimeTransformRetentionAuthority::RuntimeDefault
        );
        assert_eq!(
            transform_artifact.transform_persistence.retention_outcome,
            RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .cache_placement_posture,
            RuntimeTransformCachePlacementPosture::RuntimeCacheRootPlacement
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .cache_placement_authority,
            RuntimeTransformCachePlacementAuthority::RuntimeDefault
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .cache_placement_outcome,
            RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .persistent_clip_count,
            1
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .guarded_persistence_clip_count,
            0
        );
        assert_eq!(
            transform_artifact
                .transform_persistence
                .invalidated_persistence_clip_count,
            0
        );
        assert!(!transform_artifact
            .transform_persistence
            .cache_root_path
            .is_empty());
        assert_eq!(
            transform_artifact.clips[0].readiness,
            RuntimeTransformArtifactReadiness::Ready
        );
        assert_eq!(
            transform_artifact.clips[0].reuse_state,
            RuntimeTransformArtifactReuseState::Reusable
        );
        assert!(transform_artifact.clips[0].cached_media_ready);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
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
            RuntimeTransformPersistencePosture::AssetScopedTransformPersistence
        );
        assert!(observation
            .render_compact()
            .contains("transform_artifacts=1/1/0/0/0"));
        assert!(observation
            .render_json()
            .contains("\"transform_artifact_snapshot\":{\"clip_count\":1"));
        assert!(observation.render_json().contains(
            "\"transform_persistence\":{\"persistence_posture\":\"AssetScopedTransformPersistence\""
        ));

        let rendered = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:transform-artifact-ready".to_string(),
                timeline_start_samples: 0,
                input_stage: RuntimeClipRenderInputStage::PostWarp,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![0.5; 8],
                ),
            })
            .unwrap();
        assert_eq!(
            rendered.transform_artifact_snapshot.readiness,
            RuntimeTransformArtifactReadiness::Ready
        );
        assert_eq!(
            rendered.transform_artifact_snapshot.reuse_state,
            RuntimeTransformArtifactReuseState::Reusable
        );
        assert!(rendered.transform_artifact_snapshot.cached_media_ready);
        assert!(rendered
            .summary
            .contains("transform=Ready/Reusable/cached_media=true"));

        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &RuntimeOfflineRenderRequest {
                request_id: "render:transform-artifact-preview".into(),
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
        .expect("build transform artifact offline render preview");
        assert_eq!(preview.transform_artifact_snapshot.clip_count, 1);
        assert_eq!(preview.transform_artifact_snapshot.ready_clip_count, 1);
        assert_eq!(preview.transform_artifact_snapshot.reusable_clip_count, 1);
        assert_eq!(
            preview
                .transform_artifact_snapshot
                .transform_persistence
                .retention_outcome,
            RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms
        );
        assert!(preview.summary.contains("transform_artifacts=1/reusable=1"));

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
    fn runtime_preview_transform_snapshot_derives_from_stretch_and_artifact_baselines() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("preview-transform-ready");
        write_transient_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:preview-transform-ready".to_string(),
                content_hash: "preview-transform-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "preview-transform-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 48_000,
                waveform_bin_count: 32,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:preview-transform-ready".to_string(),
                media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:preview-transform-ready".to_string(),
                media_asset_id: Some("asset:sha256:preview-transform-ready".to_string()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                clip_gain: RuntimeClipGainEnvelope::default(),
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .start_media_preview("asset:sha256:preview-transform-ready")
            .expect("preview transform media preview should start");

        let preview_transform = runtime.get_preview_transform_snapshot();
        assert_eq!(preview_transform.clip_count, 1);
        assert_eq!(preview_transform.active_audition_clip_count, 1);
        assert_eq!(preview_transform.scrub_supported_clip_count, 1);
        assert_eq!(preview_transform.ready_clip_count, 1);
        assert_eq!(preview_transform.pending_clip_count, 0);
        assert_eq!(preview_transform.degraded_clip_count, 0);
        assert_eq!(preview_transform.invalidated_clip_count, 0);
        assert_eq!(preview_transform.unsupported_clip_count, 0);
        assert_eq!(preview_transform.stretch_aligned_clip_count, 0);
        assert_eq!(preview_transform.artifact_backed_clip_count, 1);
        assert_eq!(preview_transform.fallback_clip_count, 0);
        assert_eq!(
            preview_transform.preview_device_policy.routing_posture,
            RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
        );
        assert_eq!(
            preview_transform.preview_device_policy.audition_sink_class,
            RuntimeAuditionSinkClass::GuardedPreviewSink
        );
        assert_eq!(
            preview_transform
                .preview_device_policy
                .audition_sink_authority,
            RuntimeAuditionSinkAuthority::RuntimeDefault
        );
        assert_eq!(
            preview_transform
                .preview_device_policy
                .low_latency_device_policy_class,
            RuntimeLowLatencyDevicePolicyClass::GuardedLowLatencyDevicePolicy
        );
        assert_eq!(
            preview_transform
                .preview_device_policy
                .low_latency_device_policy_outcome,
            RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview
        );
        assert_eq!(
            preview_transform.preview_workflow.queue_posture,
            RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
        );
        assert_eq!(
            preview_transform.preview_workflow.queue_class,
            RuntimePreviewBrowserQueueClass::SingleAssetAuditionQueue
        );
        assert_eq!(
            preview_transform.preview_workflow.queue_outcome,
            RuntimePreviewBrowserQueueOutcome::PreserveActivePreviewRequest
        );
        assert_eq!(
            preview_transform.preview_workflow.audition_posture,
            RuntimeMediaAuditionOrchestrationPosture::DirectRuntimeAuditionOrchestration
        );
        assert_eq!(
            preview_transform.preview_workflow.audition_authority,
            RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .audition_continuity_outcome,
            RuntimeMediaAuditionContinuityOutcome::PreserveActiveAudition
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .transform_scheduling_posture,
            RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .transform_scheduling_authority,
            RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .transform_scheduling_outcome,
            RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .queued_preview_request_count,
            1
        );
        assert_eq!(
            preview_transform.preview_workflow.previewable_asset_count,
            1
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .active_audition_clip_count,
            1
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .pending_transform_clip_count,
            0
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .ready_transform_clip_count,
            1
        );
        assert_eq!(
            preview_transform
                .preview_workflow
                .fallback_transform_clip_count,
            0
        );
        assert_eq!(
            preview_transform.clips[0].service_class,
            RuntimePreviewTransformServiceClass::ArtifactBacked
        );
        assert_eq!(
            preview_transform.clips[0].readiness,
            RuntimePreviewTransformReadiness::Ready
        );
        assert_eq!(
            preview_transform.clips[0].fallback_kind,
            RuntimePreviewTransformFallbackKind::None
        );
        assert!(preview_transform.clips[0].audition_active);
        assert!(preview_transform.clips[0].scrub_supported);

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
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
            RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting
        );
        assert_eq!(
            observation
                .preview_transform_snapshot
                .preview_workflow
                .queue_posture,
            RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue
        );
        assert!(observation
            .render_json()
            .contains("\"preview_transform_snapshot\":{\"clip_count\":1"));
        assert!(observation.render_json().contains(
            "\"preview_device_policy\":{\"routing_posture\":\"GuardedPreviewOutputRouting\""
        ));
        assert!(observation
            .render_json()
            .contains("\"preview_workflow\":{\"queue_posture\":\"SingleActivePreviewQueue\""));

        let rendered = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:preview-transform-ready".to_string(),
                timeline_start_samples: 0,
                input_stage: RuntimeClipRenderInputStage::PostWarp,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![0.5; 8],
                ),
            })
            .unwrap();
        assert_eq!(
            rendered.preview_transform_snapshot.service_class,
            RuntimePreviewTransformServiceClass::ArtifactBacked
        );
        assert_eq!(
            rendered.preview_transform_snapshot.readiness,
            RuntimePreviewTransformReadiness::Ready
        );
        assert!(rendered.preview_transform_snapshot.audition_active);
        assert!(rendered
            .summary
            .contains("preview=ArtifactBacked/Ready/None/None"));

        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &RuntimeOfflineRenderRequest {
                request_id: "render:preview-transform-preview".into(),
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
        .expect("build preview transform offline render preview");
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
            RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting
        );
        assert_eq!(
            preview
                .preview_transform_snapshot
                .preview_workflow
                .queue_posture,
            RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue
        );
        assert_eq!(
            preview
                .preview_transform_snapshot
                .preview_workflow
                .queue_class,
            RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue
        );
        assert_eq!(
            preview
                .preview_transform_snapshot
                .preview_workflow
                .queue_outcome,
            RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview
        );
        assert_eq!(
            preview
                .preview_transform_snapshot
                .preview_workflow
                .audition_continuity_outcome,
            RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition
        );
        assert_eq!(
            preview
                .preview_transform_snapshot
                .preview_workflow
                .transform_scheduling_outcome,
            RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
        );
        assert!(preview
            .summary
            .contains("preview_transform=1/artifact_backed=1/fallback=0"));

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
    fn runtime_tempo_map_projection_drives_warp_ratio_and_export_reports() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("warp-tempo-map");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:warp-tempo-map".to_string(),
                content_hash: "warp-tempo-map".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "warp-tempo-map.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .apply_tempo_map_projection(RuntimeTempoMapProjection {
                segment_count: 2,
                segments: vec![
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:intro".to_string(),
                        start_samples: 0,
                        end_samples: Some(48_000),
                        start_tempo_bpm: 120.0,
                        end_tempo_bpm: None,
                        interpolation: RuntimeTempoMapInterpolation::Hold,
                    },
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:lift".to_string(),
                        start_samples: 48_000,
                        end_samples: Some(96_000),
                        start_tempo_bpm: 120.0,
                        end_tempo_bpm: Some(180.0),
                        interpolation: RuntimeTempoMapInterpolation::Linear,
                    },
                ],
            })
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:warp-tempo-map".to_string(),
                media_asset_id: Some("asset:sha256:warp-tempo-map".to_string()),
                mode: RuntimeWarpMode::Repitch,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 72_000,
                tempo_bpm: 90.0,
                loop_state: None,
            })
            .unwrap();

        let tempo_map = runtime.get_tempo_map_snapshot();
        assert_eq!(tempo_map.segment_count, 2);
        assert_eq!(tempo_map.active_segment_id.as_deref(), Some("tempo:lift"));
        assert_eq!(tempo_map.active_segment_index, Some(1));
        assert_eq!(tempo_map.tempo_source, RuntimeTempoSource::TempoMapSegment);
        assert!((tempo_map.resolved_tempo_bpm - 150.0).abs() < 0.000_1);

        let warp = runtime.get_warp_pipeline_snapshot();
        assert_eq!(warp.clip_count, 1);
        assert_eq!(warp.ready_clip_count, 1);
        assert_eq!(warp.degraded_clip_count, 0);
        assert_eq!(
            warp.resolved_project_tempo_source,
            RuntimeTempoSource::TempoMapSegment
        );
        assert_eq!(
            warp.resolved_project_tempo_segment_id.as_deref(),
            Some("tempo:lift")
        );
        assert!((warp.resolved_project_tempo_bpm - 150.0).abs() < 0.000_1);
        assert_eq!(
            warp.clips[0].project_tempo_source,
            RuntimeTempoSource::TempoMapSegment
        );
        assert_eq!(
            warp.clips[0].project_tempo_segment_id.as_deref(),
            Some("tempo:lift")
        );
        assert!((warp.clips[0].project_tempo_bpm - 150.0).abs() < 0.000_1);
        assert!((warp.clips[0].realized_ratio - 1.25).abs() < 0.000_1);

        let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            report.tempo_map_snapshot.tempo_source,
            RuntimeTempoSource::TempoMapSegment
        );
        assert_eq!(
            report.warp_pipeline_snapshot.resolved_project_tempo_source,
            RuntimeTempoSource::TempoMapSegment
        );
        assert!(report.render_compact().contains("tempo_map_segments=2"));
        assert!(report
            .render_compact()
            .contains("tempo_map_source=TempoMapSegment"));
        assert!(report.render_compact().contains("warp_clips=1/1/0/0"));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("tempo_map_source=TempoMapSegment"));
        assert!(multiline.contains("warp_resolved_project_tempo_source=TempoMapSegment"));
        let json = supervisor.render_json();
        assert!(json.contains("\"tempo_map_snapshot\":{\"segment_count\":2"));
        assert!(json.contains("\"resolved_project_tempo_source\":\"TempoMapSegment\""));

        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 120_000,
                tempo_bpm: 90.0,
                loop_state: None,
            })
            .unwrap();
        let fallback_tempo_map = runtime.get_tempo_map_snapshot();
        assert_eq!(fallback_tempo_map.active_segment_id, None);
        assert_eq!(
            fallback_tempo_map.tempo_source,
            RuntimeTempoSource::TransportProjection
        );
        assert!((fallback_tempo_map.resolved_tempo_bpm - 90.0).abs() < 0.000_1);
        let fallback_warp = runtime.get_warp_pipeline_snapshot();
        assert_eq!(
            fallback_warp.resolved_project_tempo_source,
            RuntimeTempoSource::TransportProjection
        );
        assert_eq!(fallback_warp.resolved_project_tempo_segment_id, None);
        assert!((fallback_warp.clips[0].realized_ratio - 0.75).abs() < 0.000_1);

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
    fn runtime_clip_render_path_applies_fade_gain_and_clip_bounds() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:render-envelope".to_string(),
                media_asset_id: None,
                warp_mode: RuntimeWarpMode::Off,
                start_samples: 10,
                duration_samples: 5,
                fade_in: RuntimeClipFadeEnvelope {
                    duration_samples: 2,
                    shape: RuntimeClipFadeShape::Linear,
                },
                fade_out: RuntimeClipFadeEnvelope {
                    duration_samples: 2,
                    shape: RuntimeClipFadeShape::Linear,
                },
                clip_gain: RuntimeClipGainEnvelope {
                    start_linear: 1.0,
                    end_linear: 0.5,
                    shape: RuntimeClipGainShape::Linear,
                },
            }])
            .unwrap();

        let result = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:render-envelope".to_string(),
                timeline_start_samples: 8,
                input_stage: RuntimeClipRenderInputStage::PostWarp,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![1.0; 7],
                ),
            })
            .unwrap();

        assert_eq!(
            result.clip_processing_snapshot.treatment_stages,
            vec![
                RuntimeClipProcessingStage::FadeIn,
                RuntimeClipProcessingStage::GainShape,
                RuntimeClipProcessingStage::FadeOut,
            ]
        );
        assert_eq!(result.timeline_start_samples, 8);
        assert_eq!(result.timeline_end_samples, 15);
        assert_eq!(result.first_frame_gain, Some(0.0));
        assert_eq!(result.last_frame_gain, Some(0.0));
        assert!((result.peak_applied_gain.unwrap_or_default() - 0.875).abs() < 1.0e-6);
        let expected = [0.0_f32, 0.0, 0.0, 0.875, 0.75, 0.625, 0.0];
        for (actual, expected) in result.output.samples().iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1.0e-6);
        }
        assert!(result
            .summary
            .contains("clip_render clip=clip:render-envelope"));
        assert!(result.summary.contains("input_stage=PostWarp"));
    }

    #[test]
    fn runtime_clip_render_path_requires_post_warp_input_for_warp_enabled_clips() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("clip-render-post-warp");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:clip-render-post-warp".to_string(),
                content_hash: "clip-render-post-warp".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "clip-render-post-warp.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:render-post-warp".to_string(),
                media_asset_id: Some("asset:sha256:clip-render-post-warp".to_string()),
                mode: RuntimeWarpMode::Repitch,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 8,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:render-post-warp".to_string(),
                media_asset_id: Some("asset:sha256:clip-render-post-warp".to_string()),
                warp_mode: RuntimeWarpMode::Repitch,
                start_samples: 0,
                duration_samples: 8,
                fade_in: RuntimeClipFadeEnvelope {
                    duration_samples: 0,
                    shape: RuntimeClipFadeShape::Linear,
                },
                fade_out: RuntimeClipFadeEnvelope {
                    duration_samples: 0,
                    shape: RuntimeClipFadeShape::Linear,
                },
                clip_gain: RuntimeClipGainEnvelope {
                    start_linear: 1.0,
                    end_linear: 1.0,
                    shape: RuntimeClipGainShape::Hold,
                },
            }])
            .unwrap();

        let raw_input_error = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:render-post-warp".to_string(),
                timeline_start_samples: 0,
                input_stage: RuntimeClipRenderInputStage::RawClip,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![1.0; 8],
                ),
            })
            .expect_err("warp-enabled clip render should require post-warp input");
        assert_eq!(
            raw_input_error.kind,
            RuntimeErrorKind::UnsupportedCapability
        );
        assert!(raw_input_error.message.contains("require post-warp input"));

        let rendered = runtime
            .render_clip_processing_buffer(RuntimeClipRenderRequest {
                clip_id: "clip:render-post-warp".to_string(),
                timeline_start_samples: 0,
                input_stage: RuntimeClipRenderInputStage::PostWarp,
                buffer: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![0.25; 8],
                ),
            })
            .unwrap();
        assert_eq!(
            rendered.clip_processing_snapshot.treatment_stages,
            vec![RuntimeClipProcessingStage::Warp]
        );
        assert_eq!(
            rendered.clip_processing_snapshot.project_tempo_source,
            Some(RuntimeTempoSource::DefaultFallback)
        );
        assert_eq!(rendered.output.samples(), &[0.25; 8]);

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
    fn runtime_clip_processing_exports_treatment_surface_with_warp_and_automation() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("clip-processing-export");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:clip-processing-export".to_string(),
                content_hash: "clip-processing-export".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "clip-processing-export.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .apply_tempo_map_projection(RuntimeTempoMapProjection {
                segment_count: 2,
                segments: vec![
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:intro".to_string(),
                        start_samples: 0,
                        end_samples: Some(48_000),
                        start_tempo_bpm: 120.0,
                        end_tempo_bpm: None,
                        interpolation: RuntimeTempoMapInterpolation::Hold,
                    },
                    crate::interfaces::RuntimeTempoMapSegmentProjection {
                        segment_id: "tempo:lift".to_string(),
                        start_samples: 48_000,
                        end_samples: Some(96_000),
                        start_tempo_bpm: 120.0,
                        end_tempo_bpm: Some(180.0),
                        interpolation: RuntimeTempoMapInterpolation::Linear,
                    },
                ],
            })
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:processing-export".to_string(),
                media_asset_id: Some("asset:sha256:clip-processing-export".to_string()),
                mode: RuntimeWarpMode::Repitch,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
                clip_id: "clip:processing-export".to_string(),
                media_asset_id: Some("asset:sha256:clip-processing-export".to_string()),
                warp_mode: RuntimeWarpMode::Repitch,
                start_samples: 0,
                duration_samples: 48_000,
                fade_in: RuntimeClipFadeEnvelope {
                    duration_samples: 1_024,
                    shape: RuntimeClipFadeShape::SmoothStep,
                },
                fade_out: RuntimeClipFadeEnvelope {
                    duration_samples: 2_048,
                    shape: RuntimeClipFadeShape::EqualPower,
                },
                clip_gain: RuntimeClipGainEnvelope {
                    start_linear: 1.0,
                    end_linear: 0.5,
                    shape: RuntimeClipGainShape::Linear,
                },
            }])
            .unwrap();
        runtime
            .apply_automation_projection(RuntimeAutomationProjection {
                lane_count: 1,
                point_count: 2,
                lanes: vec![RuntimeAutomationLaneProjection {
                    automation_lane_id: "lane:clip:gain".into(),
                    target: RuntimeAutomationTargetProjection {
                        node_id: "node:clip:gain".into(),
                        parameter_id: "gain".into(),
                    },
                    base_normalized_value: 1.0,
                    interpolation: RuntimeAutomationInterpolation::Linear,
                    resolution: RuntimeAutomationResolution {
                        ramp_step_samples: 4,
                        max_sub_blocks: 8,
                    },
                    point_count: 2,
                    points: vec![
                        RuntimeAutomationPointProjection {
                            time_samples: 0,
                            normalized_value: 1.0,
                        },
                        RuntimeAutomationPointProjection {
                            time_samples: 48_000,
                            normalized_value: 0.5,
                        },
                    ],
                }],
            })
            .unwrap();
        runtime.record_automation_summary(
            1,
            "lease:clip-processing-export",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 0,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(1.0),
                last_value: Some(0.5),
                last_modulation: None,
            },
        );
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 72_000,
                tempo_bpm: 90.0,
                loop_state: None,
            })
            .unwrap();

        let clip_processing = runtime.get_clip_processing_pipeline_snapshot();
        assert_eq!(clip_processing.clip_count, 1);
        assert_eq!(clip_processing.ready_clip_count, 1);
        assert_eq!(clip_processing.faded_clip_count, 1);
        assert_eq!(clip_processing.gain_shaped_clip_count, 1);
        assert_eq!(clip_processing.warped_clip_count, 1);
        assert_eq!(clip_processing.treatment_stage_count, 4);
        assert_eq!(
            clip_processing.clips[0].project_tempo_source,
            Some(RuntimeTempoSource::TempoMapSegment)
        );
        assert_eq!(
            clip_processing.clips[0].project_tempo_segment_id.as_deref(),
            Some("tempo:lift")
        );
        assert_eq!(
            clip_processing.clips[0].treatment_stages,
            vec![
                RuntimeClipProcessingStage::Warp,
                RuntimeClipProcessingStage::FadeIn,
                RuntimeClipProcessingStage::GainShape,
                RuntimeClipProcessingStage::FadeOut,
            ]
        );
        assert!(
            (clip_processing.clips[0]
                .realized_warp_ratio
                .unwrap_or_default()
                - 1.25)
                .abs()
                < 0.000_1
        );

        let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        let compact = report.render_compact();
        assert!(compact.contains("clip_processing_clips=1/1/0/0"));
        assert!(compact.contains("clip_processing_shapes=1/1/1"));
        assert!(compact.contains("clip_processing_treatment_stages=4"));
        assert!(compact.contains("automation_param=4096"));
        assert!(compact.contains("tempo_map_source=TempoMapSegment"));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let multiline = supervisor.render_multiline();
        assert!(multiline.contains("clip_processing_clip_count=1"));
        assert!(multiline.contains(
            "clip_processing_clip_0=clip:processing-export/readiness=Ready/warp=Repitch/Some(1.25)/Some(TempoMapSegment)"
        ));
        assert!(multiline.contains("stages=[Warp, FadeIn, GainShape, FadeOut]"));
        let json = supervisor.render_json();
        assert!(json.contains("\"clip_processing_pipeline_snapshot\":{\"clip_count\":1"));
        assert!(
            json.contains("\"treatment_stages\":[\"Warp\",\"FadeIn\",\"GainShape\",\"FadeOut\"]")
        );

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
    fn runtime_emits_events_to_subscribers() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let sink = Box::new(TestSink::default());
        runtime.subscribe(sink);

        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();
        runtime.start().unwrap();
        runtime.set_active_output_device("coreaudio:default");
        runtime.set_active_plugin_sandboxes(2);

        let readiness = runtime.get_readiness();
        assert_eq!(readiness, RuntimeReadiness::Ready);
        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            2
        );
    }

    #[test]
    fn runtime_records_plugin_fault_events() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::interfaces::PluginFaultKind::ProtocolViolation,
            "epoch mismatch",
            Some(3),
        );

        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            0
        );
    }

    #[test]
    fn runtime_tracks_plugin_lifecycle_recovery_and_quarantine_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        let recorder = RuntimeEventRecorder::default();
        runtime.subscribe(Box::new(recorder.clone()));
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:lifecycle-receipts".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                }],
            })
            .expect("apply lifecycle receipt graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:lifecycle-receipts".into(),
                contract_count: 1,
                nodes: vec![GraphNodeContractProjection {
                    node_id: "plugin-a".into(),
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
            .expect("apply lifecycle receipt contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:lifecycle-receipts".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin-a".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply lifecycle receipt binding");

        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );
        runtime.set_active_plugin_sandboxes(1);

        let ready = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(ready.active_sandbox_count, 1);
        assert_eq!(ready.ready_sandbox_count, 1);
        assert_eq!(ready.sandboxes[0].state, RuntimePluginLifecycleState::Ready);
        assert_eq!(
            ready.sandboxes[0].active_lease_id.as_deref(),
            Some("lease-a")
        );

        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::interfaces::PluginFaultKind::Crash,
            "sandbox crashed during process block",
            Some(2),
        );
        runtime.set_active_plugin_sandboxes(0);

        let faulted = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(faulted.faulted_sandbox_count, 1);
        assert_eq!(
            faulted.sandboxes[0].state,
            RuntimePluginLifecycleState::Faulted
        );
        assert_eq!(
            faulted.sandboxes[0].last_fault_detail.as_deref(),
            Some("sandbox crashed during process block")
        );

        runtime.record_recovery_cycle(
            "sandbox-a",
            RecoveryRestartIntent::CrashRecovery,
            StopReason::DegradedModeRecovery,
            Some(3),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::SandboxRestarted,
            Some(3),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-a",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(4),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-b",
            "region-b",
            PluginSandboxTransportStage::Attached,
            Some(4),
            None,
        );
        runtime.set_active_plugin_sandboxes(1);

        let recovered = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(recovered.ready_sandbox_count, 1);
        assert_eq!(
            recovered.sandboxes[0].state,
            RuntimePluginLifecycleState::Ready
        );
        assert_eq!(recovered.sandboxes[0].restart_count, 1);
        assert_eq!(recovered.sandboxes[0].recovery_count, 1);
        assert_eq!(
            recovered.sandboxes[0].active_lease_id.as_deref(),
            Some("lease-b")
        );

        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::interfaces::PluginFaultKind::Timeout,
            "sandbox missed heartbeat twice",
            Some(5),
        );

        let quarantined = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(quarantined.quarantined_sandbox_count, 1);
        assert_eq!(
            quarantined.sandboxes[0].state,
            RuntimePluginLifecycleState::Quarantined
        );
        assert_eq!(quarantined.sandboxes[0].fault_count, 2);

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(&runtime, &recorder);
        let profiling = supervisor.profiling_receipt();
        let soak = supervisor.soak_receipt();
        assert_eq!(profiling.plugin_chain_stage_count, 1);
        assert_eq!(profiling.plugin_chain_degraded_stage_count, 1);
        assert_eq!(soak.plugin_fault_count, 2);
        assert_eq!(soak.recovery_event_count, 1);
        assert_eq!(soak.plugin_quarantined_sandbox_count, 1);
        assert_eq!(soak.recall_stage_count, 1);
        assert_eq!(soak.recovered_recall_stage_count, 0);
        assert_eq!(soak.unavailable_recall_stage_count, 1);
        assert_eq!(
            soak.last_recovery_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );
        assert!(soak
            .render_json()
            .contains("\"plugin_quarantined_sandbox_count\":1"));
    }

    #[test]
    fn runtime_plugin_placement_policy_drives_shared_and_isolated_assignment_receipts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
                default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
                rules: vec![
                    RuntimePluginPlacementRule {
                        rule_id: "isolate-instrument".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                            "plugin://instrument".into(),
                        ),
                        outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
                        sandbox_group_key: None,
                    },
                    RuntimePluginPlacementRule {
                        rule_id: "share-clap".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(
                            PluginFormat::Clap,
                        ),
                        outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                        sandbox_group_key: Some("format:clap".into()),
                    },
                ],
            })
            .expect("apply plugin placement policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-placement".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-c".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                ],
            })
            .expect("apply plugin placement graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:plugin-placement".into(),
                contract_count: 3,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-c".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply plugin placement contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-placement".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-shared".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-shared".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-c".into(),
                        sandbox_id: "sandbox-isolated".into(),
                    },
                ],
            })
            .expect("apply plugin placement bindings");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-shared".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("plugin://shared-effect".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-isolated".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("plugin://instrument".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-shared",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-shared",
            "lease-shared",
            "region-shared",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-isolated",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-isolated",
            "lease-isolated",
            "region-isolated",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        let lifecycle = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(lifecycle.shared_sandbox_count, 1);
        assert_eq!(lifecycle.isolated_sandbox_count, 1);
        let shared = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("shared sandbox");
        assert_eq!(
            shared.placement_outcome,
            RuntimePluginIsolationOutcome::SharedSandbox
        );
        assert_eq!(shared.placement_rule_id.as_deref(), Some("share-clap"));
        assert_eq!(shared.sandbox_group_key, "format:clap");
        assert_eq!(shared.shared_boundary_member_count, 2);
        assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Steady);
        let isolated = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-isolated")
            .expect("isolated sandbox");
        assert_eq!(
            isolated.placement_outcome,
            RuntimePluginIsolationOutcome::IsolatedSandbox
        );
        assert_eq!(
            isolated.placement_rule_id.as_deref(),
            Some("isolate-instrument")
        );
        assert_eq!(isolated.shared_boundary_member_count, 1);

        let chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(chain.shared_sandbox_stage_count, 2);
        assert_eq!(chain.isolated_sandbox_stage_count, 1);
        assert_eq!(chain.rebindable_stage_count, 0);
        assert_eq!(chain.terminal_stage_count, 0);
        assert!(chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.placement_outcome == RuntimePluginIsolationOutcome::SharedSandbox)
            .all(|stage| {
                stage.sandbox_group_key.as_deref() == Some("format:clap")
                    && stage.shared_boundary_member_count == 2
                    && stage.continuity_class == RuntimeInterruptionClass::Steady
            }));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let rendered = supervisor.render_json();
        assert!(rendered.contains("\"plugin_lifecycle_snapshot\":{"));
        assert!(rendered.contains("\"placement_outcome\":\"SharedSandbox\""));
        assert!(rendered.contains("\"sandbox_group_key\":\"format:clap\""));
    }

    #[test]
    fn runtime_shared_sandbox_rebind_receipts_track_restartable_and_terminal_boundaries() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
                default_outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                rules: vec![RuntimePluginPlacementRule {
                    rule_id: "share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("format:clap".into()),
                }],
            })
            .expect("apply shared plugin placement policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:shared-rebind".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin-a".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-b".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                ],
            })
            .expect("apply shared rebind graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:shared-rebind".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "plugin-a".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-b".into(),
                        buffer_contract: GraphNodeBufferContractProjection::default(),
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply shared rebind contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:shared-rebind".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "plugin-a".into(),
                        sandbox_id: "sandbox-shared".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "plugin-b".into(),
                        sandbox_id: "sandbox-shared".into(),
                    },
                ],
            })
            .expect("apply shared rebind bindings");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-shared".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("plugin://shared-effect".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox-shared",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_transport(
            "sandbox-shared",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        runtime.record_plugin_sandbox_transport(
            "sandbox-shared",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(2),
            Some("replacement attach requested".into()),
        );

        let restartable = runtime.get_plugin_lifecycle_snapshot();
        let shared = restartable
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("shared sandbox");
        assert_eq!(
            shared.continuity_class,
            RuntimeInterruptionClass::Restartable
        );
        assert!(shared.rebindable);
        assert_eq!(
            shared.transport_stage,
            Some(PluginSandboxTransportStage::DetachRequested)
        );

        let restartable_chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(restartable_chain.rebindable_stage_count, 2);
        assert!(restartable_chain.chains[0]
            .stages
            .iter()
            .all(
                |stage| stage.continuity_class == RuntimeInterruptionClass::Restartable
                    && stage.rebindable
                    && stage.transport_stage == Some(PluginSandboxTransportStage::DetachRequested)
            ));

        let restartable_supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let restartable_json = restartable_supervisor.render_json();
        assert!(restartable_json.contains("\"plugin_lifecycle_snapshot\":{"));
        assert!(restartable_json.contains("\"continuity_class\":\"Restartable\""));

        runtime.record_plugin_sandbox_fault(
            "sandbox-shared",
            PluginFaultKind::Crash,
            "shared sandbox crash",
            Some(3),
        );
        runtime.record_plugin_sandbox_fault(
            "sandbox-shared",
            PluginFaultKind::Timeout,
            "shared sandbox timeout",
            Some(4),
        );

        let terminal = runtime.get_plugin_lifecycle_snapshot();
        let shared = terminal
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("terminal shared sandbox");
        assert_eq!(shared.state, RuntimePluginLifecycleState::Quarantined);
        assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
        assert!(!shared.rebindable);

        let terminal_chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(terminal_chain.terminal_stage_count, 2);
        assert!(terminal_chain.chains[0]
            .stages
            .iter()
            .all(
                |stage| stage.continuity_class == RuntimeInterruptionClass::Terminal
                    && !stage.rebindable
            ));

        let terminal_supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let terminal_json = terminal_supervisor.render_json();
        assert!(terminal_json.contains("\"terminal_stage_count\":2"));
        assert!(terminal_json.contains("\"continuity_class\":\"Terminal\""));
    }

    #[test]
    fn runtime_shared_sandbox_blast_radius_stays_boundary_local_across_recovery_and_terminal_states(
    ) {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
                default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
                rules: vec![RuntimePluginPlacementRule {
                    rule_id: "share-verified-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                        "plugin://shared-verified".into(),
                    ),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("shared:verified".into()),
                }],
            })
            .expect("apply plugin continuity placement policy");
        apply_plugin_continuity_graph(
            &mut runtime,
            "graph:runtime:plugin-continuity:shared-boundary",
            &[
                ("plugin-a", "sandbox-shared"),
                ("plugin-b", "sandbox-shared"),
                ("plugin-c", "sandbox-shared"),
                ("plugin-d", "sandbox-steady"),
            ],
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-shared",
            PluginFormat::Clap,
            "plugin://shared-verified",
            1,
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-steady",
            PluginFormat::Clap,
            "plugin://steady-utility",
            1,
        );

        let steady = runtime.get_plugin_chain_snapshot();
        assert_eq!(steady.shared_sandbox_stage_count, 3);
        assert_eq!(steady.isolated_sandbox_stage_count, 1);
        assert_eq!(steady.rebindable_stage_count, 0);
        assert_eq!(steady.terminal_stage_count, 0);

        runtime.record_plugin_sandbox_transport(
            "sandbox-shared",
            "lease-sandbox-shared",
            "region-sandbox-shared",
            PluginSandboxTransportStage::DetachRequested,
            Some(2),
            Some("shared boundary rebind".into()),
        );

        let restartable = runtime.get_plugin_lifecycle_snapshot();
        let shared = restartable
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("shared boundary should remain exported");
        assert_eq!(shared.shared_boundary_member_count, 3);
        assert_eq!(
            shared.continuity_class,
            RuntimeInterruptionClass::Restartable
        );
        assert!(shared.rebindable);
        let steady_boundary = restartable
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-steady")
            .expect("steady boundary should remain exported");
        assert_eq!(
            steady_boundary.continuity_class,
            RuntimeInterruptionClass::Steady
        );
        assert!(!steady_boundary.rebindable);

        let restartable_chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(restartable_chain.rebindable_stage_count, 3);
        assert_eq!(restartable_chain.terminal_stage_count, 0);
        assert_eq!(
            restartable_chain.chains[0]
                .stages
                .iter()
                .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
                .count(),
            3
        );
        assert!(restartable_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
            .all(|stage| {
                stage.continuity_class == RuntimeInterruptionClass::Restartable
                    && stage.rebindable
                    && stage.shared_boundary_member_count == 3
            }));
        assert!(restartable_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-steady"))
            .all(|stage| {
                stage.continuity_class == RuntimeInterruptionClass::Steady && !stage.rebindable
            }));

        runtime.record_plugin_sandbox_transport(
            "sandbox-shared",
            "lease-sandbox-shared",
            "region-sandbox-shared",
            PluginSandboxTransportStage::Attached,
            Some(3),
            None,
        );

        let recovered = runtime.get_plugin_lifecycle_snapshot();
        let shared = recovered
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("shared boundary should recover");
        assert_eq!(shared.state, RuntimePluginLifecycleState::Ready);
        assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Steady);
        assert!(!shared.rebindable);

        let recovered_chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(recovered_chain.rebindable_stage_count, 0);
        assert_eq!(recovered_chain.terminal_stage_count, 0);
        assert!(recovered_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
            .all(|stage| {
                stage.continuity_class == RuntimeInterruptionClass::Steady && !stage.rebindable
            }));

        runtime.record_plugin_sandbox_fault(
            "sandbox-shared",
            PluginFaultKind::Crash,
            "shared boundary crash",
            Some(4),
        );
        runtime.record_plugin_sandbox_fault(
            "sandbox-shared",
            PluginFaultKind::Timeout,
            "shared boundary timeout",
            Some(5),
        );

        let terminal = runtime.get_plugin_lifecycle_snapshot();
        let shared = terminal
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
            .expect("shared boundary should remain visible after terminal fault");
        assert_eq!(shared.state, RuntimePluginLifecycleState::Quarantined);
        assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
        assert!(!shared.rebindable);
        let steady_boundary = terminal
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-steady")
            .expect("steady boundary should remain visible after sibling failure");
        assert_eq!(
            steady_boundary.continuity_class,
            RuntimeInterruptionClass::Steady
        );

        let terminal_chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(terminal_chain.terminal_stage_count, 3);
        assert_eq!(terminal_chain.rebindable_stage_count, 0);
        assert!(terminal_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
            .all(|stage| {
                stage.continuity_class == RuntimeInterruptionClass::Terminal
                    && !stage.rebindable
                    && stage.shared_boundary_member_count == 3
            }));
        assert!(terminal_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-steady"))
            .all(|stage| stage.continuity_class == RuntimeInterruptionClass::Steady));

        let terminal_json =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default())
                .render_json();
        assert!(terminal_json.contains("\"plugin_lifecycle_snapshot\":{"));
        assert!(terminal_json.contains("\"sandbox_group_key\":\"shared:verified\""));
        assert!(terminal_json.contains("\"shared_boundary_member_count\":3"));
        assert!(terminal_json.contains("\"continuity_class\":\"Terminal\""));
    }

    #[test]
    fn runtime_plugin_placement_policy_exports_allowlist_denylist_and_by_format_receipts() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
                default_outcome: RuntimePluginIsolationOutcome::InProcess,
                rules: vec![
                    RuntimePluginPlacementRule {
                        rule_id: "deny-risky".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                            "plugin://risky-plugin".into(),
                        ),
                        outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
                        sandbox_group_key: None,
                    },
                    RuntimePluginPlacementRule {
                        rule_id: "allow-verified-clap".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                            "plugin://safe-shared".into(),
                        ),
                        outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                        sandbox_group_key: Some("allow:verified".into()),
                    },
                    RuntimePluginPlacementRule {
                        rule_id: "share-vst3".into(),
                        matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(
                            PluginFormat::Vst3,
                        ),
                        outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                        sandbox_group_key: Some("format:vst3".into()),
                    },
                ],
            })
            .expect("apply allowlist denylist by-format policy");
        apply_plugin_continuity_graph(
            &mut runtime,
            "graph:runtime:plugin-continuity:policy",
            &[
                ("plugin-default", "sandbox-default"),
                ("plugin-safe-a", "sandbox-allow"),
                ("plugin-safe-b", "sandbox-allow"),
                ("plugin-risky", "sandbox-deny"),
                ("plugin-vst3-a", "sandbox-format"),
                ("plugin-vst3-b", "sandbox-format"),
            ],
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-default",
            PluginFormat::Clap,
            "plugin://default-utility",
            1,
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-allow",
            PluginFormat::Clap,
            "plugin://safe-shared",
            1,
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-deny",
            PluginFormat::Clap,
            "plugin://risky-plugin",
            1,
        );
        record_ready_plugin_sandbox(
            &mut runtime,
            "sandbox-format",
            PluginFormat::Vst3,
            "plugin://vst3-effect",
            1,
        );

        let lifecycle = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(lifecycle.sandbox_count, 4);
        assert_eq!(lifecycle.shared_sandbox_count, 2);
        assert_eq!(lifecycle.isolated_sandbox_count, 1);

        let default_boundary = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-default")
            .expect("default boundary should be exported");
        assert_eq!(
            default_boundary.placement_outcome,
            RuntimePluginIsolationOutcome::InProcess
        );
        assert_eq!(default_boundary.placement_rule_id, None);
        assert_eq!(default_boundary.sandbox_group_key, "in-process:default");

        let allow_boundary = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-allow")
            .expect("allowlisted boundary should be exported");
        assert_eq!(
            allow_boundary.placement_outcome,
            RuntimePluginIsolationOutcome::SharedSandbox
        );
        assert_eq!(
            allow_boundary.placement_rule_id.as_deref(),
            Some("allow-verified-clap")
        );
        assert_eq!(allow_boundary.sandbox_group_key, "allow:verified");
        assert_eq!(allow_boundary.shared_boundary_member_count, 2);

        let deny_boundary = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-deny")
            .expect("denylisted boundary should be exported");
        assert_eq!(
            deny_boundary.placement_outcome,
            RuntimePluginIsolationOutcome::IsolatedSandbox
        );
        assert_eq!(
            deny_boundary.placement_rule_id.as_deref(),
            Some("deny-risky")
        );
        assert_eq!(deny_boundary.shared_boundary_member_count, 1);

        let format_boundary = lifecycle
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-format")
            .expect("format boundary should be exported");
        assert_eq!(
            format_boundary.placement_outcome,
            RuntimePluginIsolationOutcome::SharedSandbox
        );
        assert_eq!(
            format_boundary.placement_rule_id.as_deref(),
            Some("share-vst3")
        );
        assert_eq!(format_boundary.sandbox_group_key, "format:vst3");
        assert_eq!(format_boundary.shared_boundary_member_count, 2);

        let chain = runtime.get_plugin_chain_snapshot();
        assert_eq!(chain.in_process_stage_count, 1);
        assert_eq!(chain.shared_sandbox_stage_count, 4);
        assert_eq!(chain.isolated_sandbox_stage_count, 1);
        assert!(chain.chains[0]
            .stages
            .iter()
            .any(|stage| stage.node_id == "plugin-default"
                && stage.placement_outcome == RuntimePluginIsolationOutcome::InProcess
                && stage.sandbox_group_key.as_deref() == Some("in-process:default")));
        assert!(chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-allow"))
            .all(|stage| {
                stage.placement_rule_id.as_deref() == Some("allow-verified-clap")
                    && stage.sandbox_group_key.as_deref() == Some("allow:verified")
                    && stage.shared_boundary_member_count == 2
            }));
        assert!(chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-format"))
            .all(|stage| {
                stage.placement_rule_id.as_deref() == Some("share-vst3")
                    && stage.sandbox_group_key.as_deref() == Some("format:vst3")
                    && stage.shared_boundary_member_count == 2
            }));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        let rendered = supervisor.render_json();
        assert!(rendered.contains("\"placement_outcome\":\"InProcess\""));
        assert!(rendered.contains("\"placement_rule_id\":\"allow-verified-clap\""));
        assert!(rendered.contains("\"placement_rule_id\":\"deny-risky\""));
        assert!(rendered.contains("\"placement_rule_id\":\"share-vst3\""));
        assert!(rendered.contains("\"sandbox_group_key\":\"format:vst3\""));
    }

    #[test]
    fn runtime_owns_watchdog_restart_escalation() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        let first = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        assert_eq!(first.watchdog_restart_count, 1);
        assert!(!first.safe_mode_enabled);

        let second = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });
        assert_eq!(second.watchdog_restart_count, 2);
        assert!(second.safe_mode_enabled);
        assert_eq!(
            second.last_watchdog_trigger,
            Some(RuntimeWatchdogTrigger::DeadlineMisses)
        );
        assert_eq!(second.last_processing_epoch, Some(2));
        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn runtime_fault_status_snapshot_classifies_watchdog_plugin_fault_and_xrun_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");
        runtime.record_xrun_overload(Some(1));
        runtime.record_xrun_overload(Some(2));
        runtime.record_xrun_overload(Some(3));
        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            PluginFaultKind::Crash,
            "sandbox crashed during process block",
            Some(2),
        );
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 3,
        });
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 4,
        });

        let status = RuntimeFaultStatusSnapshot::capture(
            runtime.get_readiness(),
            &runtime.get_control_snapshot(),
            &runtime.get_diagnostics_snapshot(),
            &runtime.get_supervision_snapshot(),
            &runtime.get_engine_block_snapshot(),
            &runtime.get_transport_concurrency_snapshot(),
            &runtime.get_plugin_lifecycle_snapshot(),
            false,
            0,
        );

        assert_eq!(status.recovery_state, RuntimeRecoveryState::Recovering);
        assert_eq!(
            status.primary_fault_cause,
            Some(RuntimeFaultCause::WatchdogRestart)
        );
        assert_eq!(status.active_fault_count, 3);
        assert!(status.xrun_overload_active);
        assert!(status.plugin_fault_active);
        assert!(status.watchdog_active);
        assert!(status.safe_mode_enabled);
        assert_eq!(status.plugin_fault_count, 1);
        assert_eq!(status.watchdog_restart_count, 2);
        assert!(status.summary.contains("primary=Some(WatchdogRestart)"));
    }

    #[test]
    fn runtime_fault_status_snapshot_clears_watchdog_active_after_safe_mode_recovery() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });
        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("safe mode should clear after watchdog recovery");

        let status = RuntimeFaultStatusSnapshot::capture(
            runtime.get_readiness(),
            &runtime.get_control_snapshot(),
            &runtime.get_diagnostics_snapshot(),
            &runtime.get_supervision_snapshot(),
            &runtime.get_engine_block_snapshot(),
            &runtime.get_transport_concurrency_snapshot(),
            &runtime.get_plugin_lifecycle_snapshot(),
            false,
            0,
        );

        assert_eq!(status.recovery_state, RuntimeRecoveryState::Steady);
        assert_eq!(status.primary_fault_cause, None);
        assert_eq!(status.active_fault_count, 0);
        assert!(!status.watchdog_active);
        assert!(!status.safe_mode_enabled);
        assert_eq!(status.watchdog_restart_count, 2);
    }

    #[test]
    fn runtime_observation_report_surfaces_restartable_interruption_summary() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

        assert_eq!(
            observation.fault_status.primary_fault_cause,
            Some(RuntimeFaultCause::WatchdogRestart)
        );
        assert_eq!(
            observation.interruption_summary.class,
            RuntimeInterruptionClass::Restartable
        );
        assert!(observation.interruption_summary.active);
        assert!(!observation.interruption_summary.rebindable);

        let observation_json = observation.render_json();
        assert!(observation_json.contains("\"fault_status\":{"));
        assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
        assert!(observation_json.contains("\"interruption_summary\":{"));
        assert!(observation_json.contains("\"class\":\"Restartable\""));
    }

    #[test]
    fn runtime_fault_diagnostic_receipt_maps_xrun_pressure_into_runtime_owned_primary_family() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");
        runtime.record_xrun_overload(Some(1));
        runtime.record_xrun_overload(Some(2));
        runtime.record_xrun_overload(Some(3));

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        let receipt = &observation.fault_diagnostic_receipt;
        let xrun = receipt
            .contributions
            .iter()
            .find(|entry| {
                entry.family == crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure
            })
            .expect("xrun contribution should be present");

        assert_eq!(
            receipt.primary_family,
            Some(crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure)
        );
        assert_eq!(
            receipt.primary_fault_cause,
            Some(crate::interfaces::RuntimeFaultCause::XrunOverload)
        );
        assert_eq!(
            receipt.interruption_class,
            crate::interfaces::RuntimeInterruptionClass::Recoverable
        );
        assert!(xrun.active);
        assert_eq!(xrun.event_count, 3);
        assert_eq!(
            xrun.authority,
            crate::interfaces::RuntimeFaultDiagnosticAuthority::RuntimeCanonical
        );

        let observation_json = observation.render_json();
        assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
        assert!(observation_json.contains("\"primary_family\":\"XrunPressure\""));
    }

    #[test]
    fn runtime_fault_diagnostic_receipt_maps_deferred_work_pressure_without_faulting_runtime() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");

        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:queue:fault-diagnostic:deferred".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("safe mode should defer offline render queue");
        assert_eq!(
            deferred.orchestration.decision,
            RuntimeDeferredServiceDecision::Defer
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        let receipt = &observation.fault_diagnostic_receipt;
        let deferred_entry = receipt
            .contributions
            .iter()
            .find(|entry| {
                entry.family
                    == crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
            })
            .expect("deferred-work contribution should be present");

        assert_eq!(
            receipt.primary_family,
            Some(crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
        );
        assert_eq!(receipt.primary_fault_cause, None);
        assert_eq!(
            receipt.interruption_class,
            crate::interfaces::RuntimeInterruptionClass::Recoverable
        );
        assert!(deferred_entry.active);
        assert!(deferred_entry.event_count >= 1);
        assert!(deferred_entry
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("decision=Some(Defer)"));
    }

    #[test]
    fn runtime_xrun_overload_escalates_into_safe_mode_and_clears_after_recovery() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");

        let first = runtime.record_xrun_overload(Some(1));
        assert!(!first.safe_mode_enabled);
        assert!(!first.xrun_overload_active);

        let second = runtime.record_xrun_overload(Some(2));
        assert!(!second.safe_mode_enabled);
        assert!(!second.xrun_overload_active);

        let third = runtime.record_xrun_overload(Some(3));
        assert!(third.safe_mode_enabled);
        assert!(third.xrun_overload_active);
        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));

        let active_status = RuntimeFaultStatusSnapshot::capture(
            runtime.get_readiness(),
            &runtime.get_control_snapshot(),
            &runtime.get_diagnostics_snapshot(),
            &runtime.get_supervision_snapshot(),
            &runtime.get_engine_block_snapshot(),
            &runtime.get_transport_concurrency_snapshot(),
            &runtime.get_plugin_lifecycle_snapshot(),
            false,
            0,
        );
        assert_eq!(
            active_status.recovery_state,
            RuntimeRecoveryState::Recovering
        );
        assert_eq!(
            active_status.primary_fault_cause,
            Some(RuntimeFaultCause::XrunOverload)
        );
        assert_eq!(active_status.active_fault_count, 1);
        assert!(active_status.xrun_overload_active);
        assert!(active_status.safe_mode_enabled);

        runtime
            .set_safe_mode(SafeModeRequest { enabled: false })
            .expect("safe mode should clear");

        let recovered_status = RuntimeFaultStatusSnapshot::capture(
            runtime.get_readiness(),
            &runtime.get_control_snapshot(),
            &runtime.get_diagnostics_snapshot(),
            &runtime.get_supervision_snapshot(),
            &runtime.get_engine_block_snapshot(),
            &runtime.get_transport_concurrency_snapshot(),
            &runtime.get_plugin_lifecycle_snapshot(),
            false,
            0,
        );
        assert_eq!(
            recovered_status.recovery_state,
            RuntimeRecoveryState::Steady
        );
        assert_eq!(recovered_status.primary_fault_cause, None);
        assert_eq!(recovered_status.active_fault_count, 0);
        assert!(!recovered_status.xrun_overload_active);
        assert_eq!(runtime.get_diagnostics_snapshot().xruns, 3);
    }

    #[test]
    fn runtime_fail_runtime_marks_faulted_recovery_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().expect("start runtime");

        let readiness = runtime.fail_runtime(RuntimeError::new(
            RuntimeErrorKind::HardwareFailure,
            "simulated output recovery exhaustion",
        ));
        assert!(matches!(readiness, RuntimeReadiness::Failed { .. }));

        let status = RuntimeFaultStatusSnapshot::capture(
            runtime.get_readiness(),
            &runtime.get_control_snapshot(),
            &runtime.get_diagnostics_snapshot(),
            &runtime.get_supervision_snapshot(),
            &runtime.get_engine_block_snapshot(),
            &runtime.get_transport_concurrency_snapshot(),
            &runtime.get_plugin_lifecycle_snapshot(),
            false,
            0,
        );
        assert_eq!(status.recovery_state, RuntimeRecoveryState::Faulted);
        assert_eq!(
            status.primary_fault_cause,
            Some(RuntimeFaultCause::RuntimeError)
        );
        assert_eq!(status.active_fault_count, 1);
        assert!(runtime.get_effective_config().safe_mode_enabled);
    }

    #[test]
    fn runtime_event_recorder_builds_reusable_observation_diagnostics() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SupervisionChanged(crate::interfaces::RuntimeSupervisionSnapshot {
                watchdog_restart_count: 2,
                safe_mode_enabled: true,
                xrun_overload_active: false,
                last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
                last_sandbox_id: Some("sandbox-a".into()),
                last_processing_epoch: Some(4),
            }),
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "heartbeat watchdog missed twice".into(),
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "block deadline missed twice".into(),
                processing_epoch: Some(3),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxInstanceState {
                state: crate::interfaces::PluginSandboxInstanceStateRecord {
                    sandbox_id: "sandbox-a".into(),
                    plugin_type_id: "plugin:clap:default".into(),
                    instance_id: "instance:runtime:default".into(),
                    lifecycle_state: "Active".into(),
                    readiness_state: "Ready".into(),
                    degraded_reasons: Vec::new(),
                    active: true,
                    processing_epoch: Some(4),
                    processing_sample_rate_hz: Some(48_000),
                    processing_max_block_frames: Some(512),
                    audio_inputs: Some(2),
                    audio_outputs: Some(2),
                    midi_inputs: Some(1),
                    midi_outputs: Some(0),
                    last_fault: None,
                },
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-a".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-a".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-a".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 12,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(12),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        assert_eq!(diagnostics.total_events, 18);
        assert_eq!(diagnostics.supervision_update_count(), 1);
        assert_eq!(diagnostics.plugin_fault_count(), 2);
        assert_eq!(diagnostics.plugin_instance_state_event_count(), 1);
        assert_eq!(diagnostics.recovery_event_count(), 1);
        assert_eq!(diagnostics.lifecycle_event_count(), 1);
        assert_eq!(diagnostics.transport_event_count(), 4);
        assert_eq!(diagnostics.heartbeat_event_count(), 1);
        assert_eq!(diagnostics.block_dispatch_event_count(), 1);
        assert_eq!(diagnostics.lease_rollover_event_count(), 1);
        assert_eq!(diagnostics.invalidation_event_count(), 1);
        assert_eq!(diagnostics.completion_slot_event_count(), 2);
        assert_eq!(diagnostics.transport_fault_event_count(), 8);
        assert_eq!(diagnostics.broker_failure_event_count(), 1);
        assert_eq!(diagnostics.sandbox_operation_failure_event_count(), 1);
        assert_eq!(diagnostics.fault_detail_count_containing("watchdog"), 1);
        assert_eq!(
            diagnostics.fault_detail_count_containing("block deadline"),
            1
        );
        assert_eq!(
            diagnostics.last_plugin_instance_state().map(|state| (
                state.instance_id.as_str(),
                state.lifecycle_state.as_str(),
                state.readiness_state.as_str(),
                state.processing_sample_rate_hz,
            )),
            Some(("instance:runtime:default", "Active", "Ready", Some(48_000)))
        );
        assert_eq!(
            diagnostics
                .last_supervision_update()
                .and_then(|snapshot| snapshot.last_processing_epoch),
            Some(4)
        );
        assert_eq!(
            diagnostics
                .last_recovery_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics
                .last_lifecycle_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics.last_transport_event().map(|event| event.stage),
            Some(PluginSandboxTransportStage::DetachFault)
        );
        assert_eq!(
            diagnostics
                .transport_events
                .first()
                .map(|event| event.region_id.as_str()),
            Some("region-4")
        );
        assert_eq!(
            diagnostics
                .last_heartbeat_event()
                .map(|event| event.block_sequence),
            Some(Some(12))
        );
        assert_eq!(
            diagnostics
                .last_block_dispatch_event()
                .map(|event| event.completion_state),
            Some(Some(CompletionState::Completed))
        );
        assert_eq!(
            diagnostics
                .last_lease_rollover_event()
                .map(|event| event.previous_lease_id.as_str()),
            Some("lease-3")
        );
        assert_eq!(
            diagnostics
                .last_invalidation_event()
                .map(|event| event.reason.as_str()),
            Some("watchdog recovery teardown")
        );
        assert_eq!(
            diagnostics
                .last_completion_slot_event()
                .map(|event| event.stage),
            Some(CompletionSlotStage::FallbackApplied)
        );
        assert_eq!(
            diagnostics.last_transport_fault_event().map(|event| (
                event.source,
                event.stage,
                event.phase,
                event.resource
            )),
            Some((
                crate::interfaces::TransportFaultSource::SandboxOperation,
                crate::interfaces::TransportFaultStage::ProcessAttach,
                crate::interfaces::TransportFaultPhase::Dispatch,
                crate::interfaces::TransportFaultResource::SharedMemoryLease,
            ))
        );
        assert_eq!(
            diagnostics
                .last_broker_failure_event()
                .map(|event| event.stage),
            Some(BrokerFailureStage::PayloadRead)
        );
        assert_eq!(
            diagnostics
                .last_sandbox_operation_failure_event()
                .map(|event| event.stage),
            Some(SandboxOperationFailureStage::ProcessAttach)
        );
        assert!(diagnostics.render_compact().contains("plugin_faults=2"));
        assert!(diagnostics
            .render_compact()
            .contains("plugin_instance_states=1"));
        assert!(diagnostics.render_compact().contains("recovery_events=1"));
        assert!(diagnostics.render_compact().contains("lifecycle_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("block_dispatch_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("lease_rollover_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("invalidation_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("completion_slot_events=2"));
        assert!(diagnostics
            .render_compact()
            .contains("transport_fault_events=8"));
        assert!(diagnostics
            .render_compact()
            .contains("broker_failure_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("sandbox_operation_failure_events=1"));

        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);
        let second_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second_sequence);
        let report = RuntimeObservationReport::capture(&runtime, &recorder);
        assert!(report.render_compact().contains("readiness=Ready"));
        assert!(report.render_compact().contains("handshaken=true"));
        assert!(report.render_compact().contains("configures=1"));
        assert!(report.render_compact().contains("plugin_faults=2"));
        assert!(report.render_compact().contains("plugin_instance_states=1"));
        assert!(report.render_compact().contains("next_block_sequence=2"));
        assert!(report
            .render_compact()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert!(report
            .render_compact()
            .contains("degradation_summary_faults=2/8/1/1"));
        assert_eq!(report.scheduler_summary.topology_issue_count, 0);
        assert_eq!(report.scheduler_summary.dispatch_count, 0);
        assert!(
            !report
                .scheduler_summary
                .topology_requires_host_reinterpretation
        );
        assert_eq!(report.degradation_summary.plugin_fault_count, 2);
        assert_eq!(report.degradation_summary.transport_fault_event_count, 8);
        assert_eq!(
            report.degradation_summary.last_watchdog_trigger,
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert_eq!(
            report.transport_fault_summary.boundary_mode,
            crate::interfaces::TransportFaultBoundaryMode::FaultAdjacentOnly
        );
        assert_eq!(report.transport_fault_summary.total_events, 8);
        assert_eq!(report.transport_fault_summary.host_broker_events, 4);
        assert_eq!(report.transport_fault_summary.sandbox_operation_events, 1);
        assert_eq!(report.transport_fault_summary.runtime_dispatch_events, 3);
        assert_eq!(report.transport_fault_summary.prepare_events, 0);
        assert_eq!(report.transport_fault_summary.dispatch_events, 4);
        assert_eq!(report.transport_fault_summary.teardown_events, 4);
        assert_eq!(report.transport_fault_summary.control_events, 0);
        assert_eq!(
            report.transport_fault_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(report.transport_fault_summary.last_block_sequence, Some(12));
        assert_eq!(
            report.transport_session_summary.boundary_mode,
            crate::interfaces::TransportSessionBoundaryMode::HealthyPathVisible
        );
        assert_eq!(
            report.transport_session_summary.current_state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert!(!report.transport_session_summary.currently_attached);
        assert_eq!(
            report.transport_session_summary.heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            report.transport_session_summary.dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(report.transport_session_summary.attach_events, 1);
        assert_eq!(report.transport_session_summary.detach_requested_events, 1);
        assert_eq!(report.transport_session_summary.detached_events, 1);
        assert_eq!(report.transport_session_summary.detach_fault_events, 1);
        assert_eq!(
            report.transport_session_summary.heartbeat_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.heartbeat_responded_events,
            1
        );
        assert_eq!(report.transport_session_summary.heartbeat_missed_events, 0);
        assert_eq!(
            report.transport_session_summary.dispatch_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.dispatch_completed_events,
            1
        );
        assert_eq!(
            report.transport_session_summary.dispatch_timed_out_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(
            report.transport_session_summary.last_block_sequence,
            Some(12)
        );
        assert_eq!(
            report
                .transport_session_summary
                .active_sandbox_id
                .as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_lease_id.as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_region_id.as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_block_sequence,
            Some(12)
        );
        assert_eq!(
            report
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(
            report
                .transport_session_summary
                .max_concurrent_attached_sessions,
            1
        );
        assert!(report.transport_session_summary.active_sessions.is_empty());
        assert_eq!(
            report.transport_session_summary.last_sandbox_id.as_deref(),
            Some("sandbox-a")
        );
        assert_eq!(
            report.transport_session_summary.last_lease_id.as_deref(),
            Some("lease-4")
        );
        assert_eq!(
            report.transport_session_summary.last_region_id.as_deref(),
            Some("region-4")
        );
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(supervisor.event_count(), 18);
        assert_eq!(supervisor.supervision_update_count(), 1);
        assert_eq!(supervisor.plugin_fault_count(), 2);
        assert_eq!(supervisor.plugin_instance_state_event_count(), 1);
        assert_eq!(supervisor.recovery_event_count(), 1);
        assert_eq!(supervisor.lifecycle_event_count(), 1);
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(supervisor.render_compact().contains("event_stream=18"));
        assert!(supervisor
            .render_compact()
            .contains("plugin_instance_states=1"));
        assert!(supervisor.render_compact().contains("recovery_events=1"));
        assert!(supervisor.render_compact().contains("lifecycle_events=1"));
        assert!(supervisor.render_multiline().contains("plugin_faults=2"));
        assert!(supervisor
            .render_multiline()
            .contains("recovery_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("lifecycle_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("sequence_segments=1"));
        assert!(supervisor
            .render_multiline()
            .contains("automation_param=4096"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_host_broker_events=4"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_boundary=HealthyPathVisible"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_attach_events=1"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_state=DetachFaulted"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_heartbeat_state=Fresh"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_dispatch_state=Completed"));
        assert!(supervisor
            .render_multiline()
            .contains("scheduler_summary_topology_issue_count=0"));
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_transition=None"));
        assert!(supervisor
            .render_multiline()
            .contains("degradation_summary_transport_fault_events=8"));
        let json = supervisor.render_json();
        assert!(json.contains("\"readiness\":\"Ready\""));
        assert!(json.contains("\"control\":{\"handshaken\":true"));
        assert!(json.contains("\"next_block_sequence\":2"));
        assert!(json.contains("\"sequence_segments\":1"));
        assert!(json.contains("\"plugin_faults\":2"));
        assert!(json.contains("\"recovery_events\":1"));
        assert!(json.contains("\"recovery_sequence\":[{"));
        assert!(json.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(json.contains("\"lifecycle_events\":1"));
        assert!(json.contains("\"lifecycle_sequence\":[{"));
        assert!(json.contains("\"stage\":\"TransportAttached\""));
        assert!(json.contains("\"transport_fault_events\":8"));
        assert!(json.contains("\"last_transport_fault\":{"));
        assert!(json.contains("\"transport_fault_sequence\":[{"));
        assert!(json.contains("\"source\":\"HostBroker\""));
        assert!(json.contains("\"source\":\"SandboxOperation\""));
        assert!(json.contains("\"source\":\"RuntimeDispatch\""));
        assert!(json.contains("\"phase\":\"Dispatch\""));
        assert!(json.contains("\"phase\":\"Teardown\""));
        assert!(json.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(json.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(json.contains("\"resource\":\"CompletionSlot\""));
        assert!(json.contains("\"operation\":\"block_payload.read\""));
        assert!(json.contains("\"operation\":\"transport.detach_request\""));
        assert!(json.contains("\"operation\":\"transport.detached\""));
        assert!(json.contains("\"operation\":\"transport.detach_fault\""));
        assert!(json.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(json.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(json.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(json.contains("\"operation\":\"processBlock\""));
        assert!(json.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(json.contains("\"stage\":\"TransportDetached\""));
        assert!(json.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(json.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(json.contains("\"stage\":\"FallbackApplied\""));
        assert!(json.contains("\"stage\":\"PayloadRead\""));
        assert!(json.contains("\"stage\":\"ProcessAttach\""));
        assert!(json.contains("\"transport_fault_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(json.contains("\"host_broker_events\":4"));
        assert!(json.contains("\"sandbox_operation_events\":1"));
        assert!(json.contains("\"runtime_dispatch_events\":3"));
        assert!(json.contains("\"scheduler_summary\":{"));
        assert!(json.contains("\"topology_issue_count\":0"));
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"degradation_summary\":{"));
        assert!(json.contains("\"plugin_fault_count\":2"));
        assert!(json.contains("\"transport_fault_event_count\":8"));
        assert!(json.contains("\"dispatch_events\":4"));
        assert!(json.contains("\"teardown_events\":4"));
        assert!(json.contains("\"transport_session_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(json.contains("\"current_state\":\"DetachFaulted\""));
        assert!(json.contains("\"currently_attached\":false"));
        assert!(json.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(json.contains("\"dispatch_state\":\"Completed\""));
        assert!(json.contains("\"current_attached_session_count\":0"));
        assert!(json.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(json.contains("\"attach_events\":1"));
        assert!(json.contains("\"detach_requested_events\":1"));
        assert!(json.contains("\"detached_events\":1"));
        assert!(json.contains("\"detach_fault_events\":1"));
        assert!(json.contains("\"heartbeat_responded_events\":1"));
        assert!(json.contains("\"dispatch_completed_events\":1"));
        assert!(json.contains("\"active_sandbox_id\":null"));
        assert!(json.contains("\"active_lease_id\":null"));
        assert!(json.contains("\"active_region_id\":null"));
        assert!(json.contains("\"active_block_sequence\":12"));
        assert!(json.contains("\"active_sessions\":[]"));
        assert!(json.contains("\"last_region_id\":\"region-4\""));
        assert!(json.contains("\"automation\":{\"lane_count\":0"));
        assert!(json.contains("\"parameter_id\":4096"));
    }

    #[test]
    fn transport_session_summary_tracks_concurrent_active_sessions() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        let summary = crate::interfaces::TransportSessionSummary::from_diagnostics(&diagnostics);
        assert_eq!(summary.current_attached_session_count, 2);
        assert_eq!(summary.max_concurrent_attached_sessions, 2);
        assert_eq!(
            summary.current_state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.currently_attached);
        assert_eq!(summary.active_sessions.len(), 2);
        assert_eq!(summary.active_sandbox_id.as_deref(), Some("sandbox-a"));
        assert_eq!(summary.active_lease_id.as_deref(), Some("lease-a"));
        assert_eq!(summary.active_region_id.as_deref(), Some("region-a"));
        assert_eq!(summary.active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[0].sandbox_id.as_str(), "sandbox-a");
        assert_eq!(
            summary.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.active_sessions[0].currently_attached);
        assert_eq!(
            summary.active_sessions[0].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            summary.active_sessions[0].dispatch_state,
            crate::interfaces::TransportDispatchState::TimedOut
        );
        assert_eq!(summary.active_sessions[0].processing_epoch, Some(4));
        assert_eq!(summary.active_sessions[0].active_block_sequence, Some(11));
        assert_eq!(summary.active_sessions[0].transport_fault_count, 2);
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::RuntimeDispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::CompletionSlotTimedOut)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_processing_epoch,
            Some(4)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_block_sequence,
            Some(11)
        );
        assert_eq!(summary.active_sessions[1].sandbox_id.as_str(), "sandbox-b");
        assert_eq!(
            summary.active_sessions[1].state,
            crate::interfaces::TransportSessionState::AttachActive
        );
        assert!(summary.active_sessions[1].currently_attached);
        assert_eq!(
            summary.active_sessions[1].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            summary.active_sessions[1].dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(summary.active_sessions[1].processing_epoch, Some(5));
        assert_eq!(summary.active_sessions[1].active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[1].transport_fault_count, 1);
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::HostBroker)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::PayloadRead)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_processing_epoch,
            Some(5)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_block_sequence,
            Some(12)
        );
    }

    #[test]
    fn runtime_owns_transport_session_admission_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        let first = runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        assert_eq!(first.current_attached_sessions, 1);
        assert_eq!(first.peak_attached_sessions, 1);
        assert_eq!(first.current_recovery_overlap_sessions, 0);
        assert_eq!(first.current_lingering_sessions, 0);
        assert_eq!(
            first.active_sessions[0].state,
            crate::interfaces::TransportSessionState::AttachActive
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.peak_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.peak_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 0);

        let overlap_reject = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap_err();
        assert_eq!(overlap_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(overlap_reject
            .message
            .contains("recovery overlap session limit 1"));

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.current_attached_sessions, 2);
        assert_eq!(snapshot.peak_attached_sessions, 2);
        assert_eq!(snapshot.current_lingering_sessions, 0);
        assert_eq!(
            snapshot.last_admitted_sandbox_id.as_deref(),
            Some("sandbox-b")
        );
        assert_eq!(
            snapshot.last_rejected_sandbox_id.as_deref(),
            Some("sandbox-c")
        );
        assert!(snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));

        let after_end = runtime.end_transport_session("sandbox-a", "lease-a", "region-a");
        assert_eq!(after_end.current_attached_sessions, 1);
        assert_eq!(after_end.current_recovery_overlap_sessions, 1);
        assert_eq!(after_end.current_lingering_sessions, 0);

        let promoted =
            runtime.promote_transport_session_to_steady_state("sandbox-b", "lease-b", "region-b");
        assert_eq!(promoted.current_attached_sessions, 1);
        assert_eq!(promoted.current_recovery_overlap_sessions, 0);
        assert_eq!(promoted.current_lingering_sessions, 0);
        assert_eq!(
            promoted.active_sessions[0].provenance,
            TransportSessionProvenance::RecoveryReplacement
        );

        let re_admit = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(re_admit.current_attached_sessions, 2);
        assert_eq!(re_admit.current_recovery_overlap_sessions, 1);

        let after_overlap_end = runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        assert_eq!(after_overlap_end.current_attached_sessions, 1);
        assert_eq!(after_overlap_end.current_recovery_overlap_sessions, 1);
        assert_eq!(after_overlap_end.current_lingering_sessions, 0);

        let after_final_end = runtime.end_transport_session("sandbox-c", "lease-c", "region-c");
        assert_eq!(after_final_end.current_attached_sessions, 0);
        assert_eq!(after_final_end.current_recovery_overlap_sessions, 0);
        assert_eq!(after_final_end.current_lingering_sessions, 0);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .unwrap();
        let reset = runtime.get_transport_concurrency_snapshot();
        assert_eq!(reset.current_attached_sessions, 0);
        assert_eq!(reset.current_lingering_sessions, 0);
        assert!(reset.active_sessions.is_empty());
        assert_eq!(reset.peak_attached_sessions, 0);
        assert_eq!(reset.peak_lingering_sessions, 0);
    }

    #[test]
    fn runtime_transport_session_limits_can_be_widened_for_multiple_steady_sessions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        let widened = runtime
            .set_transport_session_limits(4, 6)
            .expect("set widened transport session policy");
        assert_eq!(widened.steady_session_limit, 4);
        assert_eq!(widened.recovery_session_limit, 6);

        let first = runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin first steady session");
        assert_eq!(first.current_attached_sessions, 1);

        let second = runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin second steady session");
        assert_eq!(second.current_attached_sessions, 2);
        assert_eq!(second.steady_session_limit, 4);
        assert_eq!(second.recovery_session_limit, 6);
    }

    #[test]
    fn runtime_tracks_lingering_transport_sessions_as_first_class_admission_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(2),
            None,
        );

        let requested = runtime.get_transport_concurrency_snapshot();
        assert_eq!(requested.current_attached_sessions, 1);
        assert_eq!(requested.current_lingering_sessions, 1);
        assert_eq!(requested.peak_lingering_sessions, 1);
        assert_eq!(requested.current_detach_requested_sessions, 1);
        assert_eq!(requested.current_detach_faulted_sessions, 0);
        assert_eq!(
            requested.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(steady_reject.message.contains("lingering session"));

        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("teardown fault".into()),
        );

        let faulted = runtime.get_transport_concurrency_snapshot();
        assert_eq!(faulted.current_attached_sessions, 1);
        assert_eq!(faulted.current_lingering_sessions, 1);
        assert_eq!(faulted.current_detach_requested_sessions, 0);
        assert_eq!(faulted.current_detach_faulted_sessions, 1);
        assert_eq!(
            faulted.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 1);
        assert_eq!(overlap.current_detach_faulted_sessions, 1);
        assert_eq!(overlap.peak_lingering_sessions, 1);

        runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        runtime.end_transport_session("sandbox-a", "lease-a", "region-a");

        let cleared = runtime.get_transport_concurrency_snapshot();
        assert_eq!(cleared.current_attached_sessions, 0);
        assert_eq!(cleared.current_lingering_sessions, 0);
        assert_eq!(cleared.current_detach_requested_sessions, 0);
        assert_eq!(cleared.current_detach_faulted_sessions, 0);
    }

    #[test]
    fn runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-origin",
                "region-origin",
                TransportAttachIntent::SteadyState,
                Some(2),
                TransportSessionProvenance::SteadyOrigin,
                Some("/tmp/signal-origin".into()),
                Some(4096),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("origin detach fault".into()),
        );

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-replacement",
                "region-replacement",
                TransportAttachIntent::RecoveryOverlap,
                Some(3),
                TransportSessionProvenance::RecoveryReplacement,
                Some("/tmp/signal-replacement".into()),
                Some(8192),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            PluginSandboxTransportStage::DetachRequested,
            Some(3),
            None,
        );

        let cleanup_receipt = runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::StrictPreAttach,
                LingeringCleanupTrigger::RecoveryPreAttach,
                4,
                None,
                None,
            )
            .expect("cleanup work should be queued");
        let queued = runtime.get_transport_concurrency_snapshot();
        assert_eq!(queued.pending_cleanup_work_items, 1);
        assert_eq!(queued.pending_deferred_retry_work_items, 0);
        assert_eq!(queued.next_cleanup_epoch, 2);
        assert_eq!(queued.oldest_pending_cleanup_ready_epoch, Some(4));
        assert_eq!(queued.pending_cleanup_waves.len(), 1);
        assert_eq!(queued.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            queued.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(
            queued.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );

        let cleanup_plan = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 4)
            .expect("cleanup plan should dequeue");
        assert_eq!(cleanup_plan.work_id, cleanup_receipt.work_id);
        assert_eq!(cleanup_plan.cleanup_epoch, cleanup_receipt.cleanup_epoch);
        assert_eq!(cleanup_plan.cleanup_wave, cleanup_receipt.cleanup_wave);
        assert_eq!(cleanup_plan.sandbox_id, "sandbox-a");
        assert_eq!(cleanup_plan.mode, LingeringCleanupMode::StrictPreAttach);
        assert_eq!(
            cleanup_plan.trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(cleanup_plan.retry_count, 0);
        assert_eq!(cleanup_plan.processing_epoch, 4);
        assert_eq!(cleanup_plan.ready_at_processing_epoch, 4);
        assert_eq!(cleanup_plan.exclude_lease_id, None);
        assert_eq!(cleanup_plan.exclude_region_id, None);
        let cleanup_candidates = cleanup_plan.candidates;
        assert_eq!(cleanup_candidates.len(), 2);
        assert!(cleanup_candidates[0].attach_sequence < cleanup_candidates[1].attach_sequence);

        assert_eq!(
            cleanup_candidates[0].provenance,
            TransportSessionProvenance::SteadyOrigin
        );
        assert_eq!(cleanup_candidates[0].attach_processing_epoch, Some(2));
        assert_eq!(
            cleanup_candidates[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert_eq!(cleanup_candidates[0].lease_id, "lease-origin");
        assert_eq!(cleanup_candidates[0].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[0].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[0].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[0].cleanup_in_progress);
        assert_eq!(cleanup_candidates[0].last_cleanup_epoch, Some(4));
        assert_eq!(cleanup_candidates[0].last_cleanup_error, None);

        assert_eq!(
            cleanup_candidates[1].provenance,
            TransportSessionProvenance::RecoveryReplacement
        );
        assert_eq!(cleanup_candidates[1].attach_processing_epoch, Some(3));
        assert_eq!(
            cleanup_candidates[1].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert_eq!(cleanup_candidates[1].lease_id, "lease-replacement");
        assert_eq!(cleanup_candidates[1].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[1].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[1].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[1].cleanup_in_progress);

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.active_sessions.len(), 2);
        assert!(snapshot
            .active_sessions
            .iter()
            .all(|session| session.cleanup_in_progress));

        let failed = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            LingeringCleanupMode::StrictPreAttach,
            4,
            "cleanup failed",
        );
        let origin = failed
            .active_sessions
            .iter()
            .find(|session| session.lease_id == "lease-origin")
            .unwrap();
        assert!(!origin.cleanup_in_progress);
        assert_eq!(origin.cleanup_attempt_count, 1);
        assert_eq!(
            origin.last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(origin.last_cleanup_epoch, Some(4));
        assert_eq!(origin.last_cleanup_error.as_deref(), Some("cleanup failed"));

        let retried = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            LingeringCleanupMode::BestEffortPostStart,
            5,
            "late cleanup failed",
        );
        assert_eq!(retried.pending_cleanup_work_items, 1);
        assert_eq!(retried.pending_deferred_retry_work_items, 1);
        assert_eq!(retried.next_cleanup_epoch, 3);
        assert_eq!(retried.oldest_pending_cleanup_ready_epoch, Some(6));
        assert_eq!(retried.pending_cleanup_waves.len(), 1);
        assert_eq!(retried.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            retried.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert!(runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 5)
            .is_none());
        let deferred_retry = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 6)
            .expect("deferred retry should dequeue");
        assert_eq!(deferred_retry.cleanup_epoch, 2);
        assert_eq!(deferred_retry.cleanup_wave, 1);
        assert_eq!(
            deferred_retry.trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert_eq!(deferred_retry.retry_count, 1);
        assert_eq!(
            deferred_retry.mode,
            LingeringCleanupMode::BestEffortPostStart
        );
        assert_eq!(deferred_retry.ready_at_processing_epoch, 6);
        assert_eq!(
            deferred_retry.exclude_lease_id.as_deref(),
            Some("lease-replacement")
        );
        assert_eq!(
            deferred_retry.exclude_region_id.as_deref(),
            Some("region-replacement")
        );

        runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::BestEffortPostStart,
                LingeringCleanupTrigger::PostStartReconciliation,
                7,
                None,
                None,
            )
            .expect("second cleanup wave should queue");
        let next_wave = runtime.get_transport_concurrency_snapshot();
        assert_eq!(next_wave.pending_cleanup_waves.len(), 1);
        assert_eq!(next_wave.pending_cleanup_waves[0].cleanup_wave, 2);
        assert_eq!(
            next_wave.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::PostStartReconciliation
        );
    }

    #[test]
    fn configure_requires_prior_handshake() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidState
        );
    }

    #[test]
    fn start_requires_prior_configuration() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let error = runtime.start().unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidState
        );
    }

    #[test]
    fn control_snapshot_tracks_handshake_configure_and_restart_history() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();
        runtime.start().unwrap();
        runtime
            .restart(RestartRequest {
                reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
            })
            .unwrap();

        let control = runtime.get_control_snapshot();
        assert!(control.handshaken);
        assert!(control.configured);
        assert!(control.running);
        assert_eq!(control.handshake_count, 1);
        assert_eq!(control.configure_count, 2);
        assert_eq!(control.start_count, 2);
        assert_eq!(control.stop_count, 1);
        assert_eq!(control.restart_count, 1);
        assert_eq!(control.last_client_version.as_deref(), Some("runtime-test"));
        assert_eq!(
            control.last_stop_reason,
            Some(StopReason::DeviceReconfigure)
        );
        assert_eq!(
            control
                .last_reconfigure
                .map(|request| request.sample_rate.0),
            Some(44_100)
        );
    }
}
