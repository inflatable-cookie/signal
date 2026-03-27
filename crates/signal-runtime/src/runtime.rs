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
#[path = "tests_support.rs"]
mod tests_support;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
