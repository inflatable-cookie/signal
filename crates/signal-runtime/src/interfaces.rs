//! Typed runtime-host interfaces for embedded Signal assemblies.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use signal_graph::{
    GraphChannelAdaptationMode, GraphDynamicStageStateModel, GraphExecutionContext,
    GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, BackendPolicyTier, HardwareClockSource,
    HardwareConfigRequest, HardwareLifecycleOwnership, HardwareRestartPolicy,
};
use signal_plugin::{BlockSequenceContinuityReport, CompletionState};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    InvalidRequest,
    UnsupportedCapability,
    InvalidState,
    ResourceUnavailable,
    PluginFailure,
    HardwareFailure,
    Timeout,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeRequest {
    pub client_version: String,
    pub anticipative_preferred: bool,
    pub max_sample_rate_hint: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub runtime_version: String,
    pub protocol_version: u32,
    pub supports_anticipative: bool,
    pub supports_dynamic_reconfigure: bool,
    pub max_channels: u32,
    pub max_sample_rate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfigRequest {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub realtime_safe_mode: bool,
    pub max_graph_latency_ms: Option<u32>,
    pub max_background_load_percent: Option<u8>,
}

impl RuntimeConfigRequest {
    pub fn new(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            block_size,
            anticipative_enabled: true,
            realtime_safe_mode: false,
            max_graph_latency_ms: None,
            max_background_load_percent: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    UserRequested,
    DeviceReconfigure,
    DegradedModeRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryRestartIntent {
    CrashRecovery,
    WatchdogRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartRequest {
    pub reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeModeRequest {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeProjection {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub latency_samples: u32,
    pub stages: Vec<GraphStageSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphProjection {
    pub graph_id: String,
    pub node_count: usize,
    pub nodes: Vec<GraphNodeProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpointProjection {
    pub bus_id: String,
    pub channels: ChannelLayout,
}

impl Default for GraphNodeBusEndpointProjection {
    fn default() -> Self {
        Self {
            bus_id: "main:in".into(),
            channels: ChannelLayout::Stereo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContractProjection {
    pub input: GraphNodeBusEndpointProjection,
    pub output: GraphNodeBusEndpointProjection,
    pub scratch_buffers: usize,
    pub silence_policy: GraphNodeSilencePolicy,
    pub channel_adaptation: GraphChannelAdaptationMode,
    pub reset_policy: GraphNodeResetPolicy,
}

impl Default for GraphNodeBufferContractProjection {
    fn default() -> Self {
        Self {
            input: GraphNodeBusEndpointProjection::default(),
            output: GraphNodeBusEndpointProjection {
                bus_id: "main:out".into(),
                channels: ChannelLayout::Stereo,
            },
            scratch_buffers: 0,
            silence_policy: GraphNodeSilencePolicy::Process,
            channel_adaptation: GraphChannelAdaptationMode::AdaptiveMonoStereo,
            reset_policy: GraphNodeResetPolicy::RetainAcrossBlocks,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyProjection {
    pub role: Option<GraphNodeTopologyRole>,
    pub lane_id: Option<String>,
    pub bus_group_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractProjection {
    pub node_id: String,
    pub buffer_contract: GraphNodeBufferContractProjection,
    pub topology: GraphNodeTopologyProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphContractProjection {
    pub graph_id: String,
    pub contract_count: usize,
    pub nodes: Vec<GraphNodeContractProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBinding {
    pub node_id: String,
    pub sandbox_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBindingProjection {
    pub graph_id: String,
    pub bindings: Vec<PluginBackedNodeBinding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRender {
    pub node_id: String,
    pub sandbox_id: String,
    pub output: AudioBuffer,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub bypassed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRenderBatch {
    pub graph_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub renders: Vec<PluginNodeRender>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePluginDispatchState {
    pub transport: Option<TransportProjection>,
    pub parameter_batch: Option<ParameterBatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkCacheState {
    Disabled,
    Empty,
    Admitted,
    Consumed,
    Invalidated,
}

impl Default for RuntimePreworkCacheState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkInvalidationReason {
    RuntimeReconfigured,
    RuntimeStopped,
    PlanningDisabled,
    ForecastPlanChanged,
    GraphProjectionChanged,
    TransportStarted,
    TransportStopped,
    TransportSeeked,
    TransportTempoChanged,
    TransportLoopStateChanged,
    TransportLoopWrapped,
    ParameterBatchApplied,
    InputSignatureChanged,
    ProcessingEpochExpired,
    BlockSequenceExpired,
    SupersededByAdmission,
    PlanningWindowRevised,
    QueueCapacityExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkRetirementReason {
    RuntimeReconfigured,
    RuntimeStopped,
    ForecastPlanChanged,
    GraphProjectionChanged,
    TransportStarted,
    TransportStopped,
    TransportSeeked,
    TransportTempoChanged,
    TransportLoopStateChanged,
    TransportLoopWrapped,
    ParameterBatchApplied,
    InputSignatureChanged,
    ProcessingEpochExpired,
    BlockSequenceExpired,
    PlanningDisabled,
    SupersededByAdmission,
    PlanningWindowRevised,
    QueueCapacityExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkFreshnessState {
    Disabled,
    Empty,
    Fresh,
    Expiring,
    Exhausted,
    Invalidated,
}

impl Default for RuntimePreworkFreshnessState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServiceState {
    Disabled,
    Idle,
    Pending,
    Servicing,
    Yielding,
    Paused,
    Starved,
}

impl Default for RuntimePreworkServiceState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServicePressure {
    Normal,
    Elevated,
    Critical,
}

impl Default for RuntimePreworkServicePressure {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServiceSemanticPolicy {
    Balanced,
    LatencyFocused,
    PluginConstrained,
}

impl Default for RuntimePreworkServiceSemanticPolicy {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeSchedulerTopologyIssue {
    MissingTrackLaneIds {
        node_count: usize,
    },
    MissingBusGroupIds {
        node_count: usize,
    },
    MissingConsoleGroupIds {
        node_count: usize,
    },
    MissingRealtimeLaneForTopology,
    AnticipativeLaneMustPrecedeRealtime,
    RealtimeDispatchMustTerminateTopology,
    MissingScheduleProjectionForTrackLanes {
        required_streams: usize,
    },
    InsufficientScheduleStreams {
        required_streams: usize,
        actual_streams: usize,
    },
}

/// Scheduler-facing topology summary derived from the active graph projection
/// and schedule view.
///
/// This tells hosts whether the runtime-owned planning shape lines up with the
/// declared track/bus/send/console topology or whether a host would need to
/// reinterpret the current plan boundary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerTopologySummary {
    pub track_lane_node_count: usize,
    pub track_lane_group_count: usize,
    pub bus_node_count: usize,
    pub bus_group_count: usize,
    pub send_return_node_count: usize,
    pub send_return_group_count: usize,
    pub console_node_count: usize,
    pub console_group_count: usize,
    pub schedule_stream_count: Option<usize>,
    pub compatible: bool,
    pub requires_host_reinterpretation: bool,
    pub issues: Vec<RuntimeSchedulerTopologyIssue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTransportTransitionKind {
    Initial,
    Started,
    Stopped,
    Seeked,
    TempoChanged,
    LoopStateChanged,
    LoopWrapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePreworkBacklogClass {
    Immediate,
    NearTerm,
    Deferred,
}

impl Default for RuntimePreworkBacklogClass {
    fn default() -> Self {
        Self::Immediate
    }
}

/// Supervisor-facing snapshot of the most recent engine block.
///
/// This is the primary integration surface for understanding how runtime is
/// currently executing the graph: planning shape, dispatch behavior, prework
/// service state, forecast policy, and per-block output telemetry all land
/// here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeEngineBlockSnapshot {
    pub graph_id: Option<String>,
    pub node_count: usize,
    pub stateful_node_count: usize,
    pub latency_node_count: usize,
    pub plugin_backed_node_count: usize,
    pub anticipative_planning_enabled: bool,
    pub inline_realtime_node_count: usize,
    pub stateful_realtime_node_count: usize,
    pub anticipative_eligible_node_count: usize,
    pub phase_count: usize,
    pub anticipative_phase_count: usize,
    pub phase_order: Vec<GraphNodePlanningGroup>,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub lane_order: Vec<GraphExecutionLane>,
    pub scheduler_topology: RuntimeSchedulerTopologySummary,
    pub dispatch_count: usize,
    pub dispatch_boundary_count: usize,
    pub dispatch_order: Vec<GraphExecutionLane>,
    pub prepared_dispatch_count: usize,
    pub realtime_dispatch_count: usize,
    pub dispatch_handoff_count: usize,
    pub prework_cache_enabled: bool,
    pub prework_cache_state: RuntimePreworkCacheState,
    pub prework_cache_queue_capacity: usize,
    pub prework_cache_queue_depth: usize,
    pub prework_cache_peak_queue_depth: usize,
    pub prework_pending_target_count: usize,
    pub prework_pending_immediate_target_count: usize,
    pub prework_pending_near_term_target_count: usize,
    pub prework_pending_deferred_target_count: usize,
    pub prework_next_pending_target_block_sequence: Option<u64>,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub prework_service_active_plugin_sandboxes: u32,
    pub prework_service_bound_plugin_sandboxes: usize,
    pub prework_service_active_bound_plugin_sandboxes: usize,
    pub prework_service_degraded_bound_plugin_sandboxes: usize,
    pub prework_service_missing_bound_plugin_sandboxes: usize,
    pub prework_service_plugin_gate_active: bool,
    pub prework_service_recovery_overlap_sessions: usize,
    pub prework_service_lingering_sessions: usize,
    pub prework_service_detach_faulted_sessions: usize,
    pub prework_service_transport_gate_active: bool,
    pub prework_service_cycle_count: u64,
    pub prework_service_prepared_targets: u64,
    pub prework_service_pause_count: u64,
    pub prework_service_resume_count: u64,
    pub prework_service_starvation_count: u64,
    pub prework_service_throttle_count: u64,
    pub prework_service_yield_count: u64,
    pub last_prework_service_processing_epoch: Option<u64>,
    pub last_prework_service_requested_cycles: usize,
    pub last_prework_service_effective_cycles: usize,
    pub last_prework_service_cycle_count: usize,
    pub last_prework_service_budget_per_cycle: Option<usize>,
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    pub last_prework_service_prepared_targets: usize,
    pub last_prework_serviced_target_block_sequence: Option<u64>,
    pub last_prework_serviced_backlog_class: Option<RuntimePreworkBacklogClass>,
    pub prework_forecast_requested_mode: RuntimePreworkForecastMode,
    pub prework_forecast_mode: RuntimePreworkForecastMode,
    pub prework_forecast_policy_configured: bool,
    pub prework_forecast_profile: Option<RuntimePreworkForecastProfile>,
    pub prework_forecast_profile_source: Option<RuntimePreworkForecastProfileSource>,
    pub prework_forecast_profile_target_window_override: Option<usize>,
    pub prework_forecast_policy_target_window_blocks: Option<usize>,
    pub prework_cache_window_target_count: usize,
    pub prework_cache_window_target_block_sequences: Vec<u64>,
    pub prework_cache_admissions: u64,
    pub prework_cache_consumptions: u64,
    pub prework_cache_queued_admissions: u64,
    pub prework_cache_queued_consumptions: u64,
    pub prework_cache_hits: u64,
    pub prework_cache_misses: u64,
    pub prework_cache_invalidation_count: u64,
    pub prework_cache_retirement_count: u64,
    pub prework_cache_unconsumed_retirement_count: u64,
    pub prework_cache_consumed_retirement_count: u64,
    pub last_prework_cache_hit: bool,
    pub last_prework_invalidation_reason: Option<RuntimePreworkInvalidationReason>,
    pub last_prework_retirement_reason: Option<RuntimePreworkRetirementReason>,
    pub last_prework_retired_unconsumed: Option<bool>,
    pub prework_cache_freshness_state: RuntimePreworkFreshnessState,
    pub prework_cache_block_freshness_window: u64,
    pub prework_cache_remaining_valid_blocks: Option<u64>,
    pub prework_cache_valid_until_processing_epoch: Option<u64>,
    pub prework_cache_valid_until_block_sequence: Option<u64>,
    pub last_prework_source_processing_epoch: Option<u64>,
    pub last_prework_source_block_sequence: Option<u64>,
    pub last_prework_admission_processing_epoch: Option<u64>,
    pub last_prework_admission_block_sequence: Option<u64>,
    pub last_prework_admitted_from_block_sequence: Option<u64>,
    pub last_prework_consumption_processing_epoch: Option<u64>,
    pub last_prework_consumption_block_sequence: Option<u64>,
    pub last_prework_consumed_from_block_sequence: Option<u64>,
    pub last_prework_retirement_processing_epoch: Option<u64>,
    pub last_prework_retirement_block_sequence: Option<u64>,
    pub planned_nodes: Vec<RuntimePlannedGraphNode>,
    pub stage_count: usize,
    pub dynamic_kernel_stage_count: usize,
    pub dynamic_stage_state_model: GraphDynamicStageStateModel,
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
    pub total_tail_samples: u32,
    pub max_node_tail_samples: u32,
    pub output_tail_samples: u32,
    pub max_bus_tail_samples: u32,
    pub processed_blocks: u64,
    pub last_processing_epoch: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub last_frame_count: usize,
    pub last_channel_count: usize,
    pub last_input_peak: Option<f32>,
    pub last_prework_output_peak: Option<f32>,
    pub last_realtime_input_peak: Option<f32>,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub last_first_output_sample: Option<f32>,
    pub transport_epoch: u64,
    pub transport_transition: Option<RuntimeTransportTransitionKind>,
    pub transport_block_start_samples: Option<i64>,
    pub transport_block_end_samples: Option<i64>,
    pub transport_loop_wrapped: bool,
    pub last_execution_context: Option<GraphExecutionContext>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkWindowTarget {
    pub target_block_sequence: u64,
    pub admitted_from_block_sequence: u64,
    pub buffer: AudioBuffer,
    pub parameter_epoch_override: Option<u64>,
    pub transport_override: Option<TransportProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkForecastPolicy {
    pub target_window_blocks: usize,
    pub prepare_budget_per_cycle: usize,
    pub buffer_seed_offset: u64,
    pub transport_playing: bool,
    pub transport_tempo_bpm: f64,
    pub transport_loop_length_blocks: usize,
    pub parameter_target: String,
    pub parameter_cycle_length: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkForecastMode {
    #[default]
    Disabled,
    RuntimeRoleDefault,
    ExplicitProfile,
    RawPolicyOverride,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfileSource {
    RuntimeRoleDefault,
    ExplicitSelection,
    RawPolicyOverride,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePreworkForecastProfileSelection {
    pub profile: RuntimePreworkForecastProfile,
    pub target_window_blocks_override: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePlannedGraphNode {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub group: GraphNodePlanningGroup,
    pub latency_samples: u32,
    pub topology_role: GraphNodeTopologyRole,
    pub lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub plugin_sandbox_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEngineBlockResult {
    pub snapshot: RuntimeEngineBlockSnapshot,
    pub output: AudioBuffer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleProjection {
    pub schedule_id: String,
    pub stream_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportProjection {
    pub playing: bool,
    pub timeline_position_samples: i64,
    pub tempo_bpm: f64,
    pub loop_state: Option<LoopRegion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterEvent {
    pub target: String,
    pub normalized_value: f32,
}

/// Runtime-owned batch of parameter changes accepted for one automation epoch.
///
/// Runtime stays authoritative for epoch assignment and block-boundary
/// application; callers supply only the logical event payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    pub epoch: u64,
    pub events: Vec<ParameterEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReceipt {
    pub accepted_epoch: u64,
    pub applied_at_block_boundary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedReason(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    Timeout,
    Crash,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWatchdogTrigger {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogRestartRecord {
    pub sandbox_id: String,
    pub trigger: RuntimeWatchdogTrigger,
    pub processing_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupervisionSnapshot {
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub last_sandbox_id: Option<String>,
    pub last_processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTimelineSnapshot {
    pub next_block_sequence: u64,
    pub block_sequence_continuity: BlockSequenceContinuityReport,
    pub transport_epoch: u64,
    pub last_transport_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transport_transition_processing_epoch: Option<u64>,
    pub last_transport_transition_block_sequence: Option<u64>,
    pub last_transport_playing: Option<bool>,
    pub last_transport_tempo_bpm: Option<f64>,
    pub last_transport_timeline_position_samples: Option<i64>,
    pub last_transport_loop_start_samples: Option<i64>,
    pub last_transport_loop_end_samples: Option<i64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTransportObservationSnapshot {
    pub transport_epoch: u64,
    pub projected_playing: Option<bool>,
    pub projected_tempo_bpm: Option<f64>,
    pub projected_timeline_position_samples: Option<i64>,
    pub projected_loop_start_samples: Option<i64>,
    pub projected_loop_end_samples: Option<i64>,
    pub observed_playing: Option<bool>,
    pub observed_tempo_bpm: Option<f64>,
    pub observed_timeline_position_samples: Option<i64>,
    pub observed_loop_start_samples: Option<i64>,
    pub observed_loop_end_samples: Option<i64>,
    pub last_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transition_processing_epoch: Option<u64>,
    pub last_transition_block_sequence: Option<u64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureState {
    Idle,
    Capturing,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordingCaptureStartRequest {
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub capture_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCommitReceipt {
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub channel_count: usize,
    pub peak_level: f32,
    pub capture_path: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRecordingCaptureSnapshot {
    pub capture_ready: bool,
    pub state: Option<RuntimeRecordingCaptureState>,
    pub active_take_id: Option<String>,
    pub active_track_id: Option<String>,
    pub capture_start_samples: Option<i64>,
    pub active_capture_path: Option<String>,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub last_committed_take_id: Option<String>,
    pub last_committed_path: Option<String>,
    pub last_committed_duration_samples: Option<u32>,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMediaAssetState {
    Ingesting,
    Conforming,
    Ready,
    Invalid,
    Rebuilding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMediaAssetRegistration {
    pub asset_id: String,
    pub content_hash: String,
    pub source_path: String,
    pub file_name: String,
    pub byte_size: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub duration_samples: u64,
    pub waveform_bin_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaAssetSnapshot {
    pub asset_id: String,
    pub content_hash: String,
    pub source_path: String,
    pub file_name: String,
    pub byte_size: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub duration_samples: u64,
    pub waveform_bin_count: usize,
    pub state: Option<RuntimeMediaAssetState>,
    pub cache_path: Option<String>,
    pub cache_byte_size: Option<u64>,
    pub rebuild_count: u32,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaPipelineSnapshot {
    pub cache_root_path: String,
    pub asset_count: usize,
    pub ready_asset_count: usize,
    pub invalid_asset_count: usize,
    pub ingesting_asset_count: usize,
    pub conforming_asset_count: usize,
    pub rebuilding_asset_count: usize,
    pub assets: Vec<RuntimeMediaAssetSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpMode {
    Off,
    Repitch,
    ElastiqueDraft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpReadiness {
    Bypassed,
    Ready,
    Degraded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipRegistration {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub mode: RuntimeWarpMode,
    pub source_tempo_bpm: Option<f64>,
    pub anchor_timeline_samples: i64,
    pub start_samples: i64,
    pub duration_samples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub mode: RuntimeWarpMode,
    pub source_tempo_bpm: Option<f64>,
    pub project_tempo_bpm: f64,
    pub realized_ratio: f64,
    pub anchor_timeline_samples: i64,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub readiness: RuntimeWarpReadiness,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeWarpPipelineSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub degraded_clip_count: usize,
    pub bypassed_clip_count: usize,
    pub active_warp_count: usize,
    pub clips: Vec<RuntimeWarpClipSnapshot>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    pub parameter_id: u32,
    pub value_events: usize,
    pub modulation_events: usize,
    pub gesture_begin_events: usize,
    pub gesture_end_events: usize,
    pub first_value: Option<f32>,
    pub last_value: Option<f32>,
    pub last_modulation: Option<f32>,
    pub first_epoch: Option<u64>,
    pub last_epoch: Option<u64>,
    pub segment_count: usize,
    pub segment_epochs: Vec<u64>,
    pub lease_rollovers: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportAttachIntent {
    SteadyState,
    RecoveryOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionProvenance {
    SteadyOrigin,
    RecoveryReplacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupMode {
    StrictPreAttach,
    BestEffortPostStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupTrigger {
    RecoveryPreAttach,
    PostStartReconciliation,
    DeferredRetry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportConcurrencySession {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub intent: TransportAttachIntent,
    pub provenance: TransportSessionProvenance,
    pub attach_sequence: u64,
    pub attach_processing_epoch: Option<u64>,
    pub state: TransportSessionState,
    pub backing_path: Option<String>,
    pub total_bytes: Option<u32>,
    pub cleanup_attempt_count: u32,
    pub last_cleanup_mode: Option<LingeringCleanupMode>,
    pub last_cleanup_wave: Option<u64>,
    pub cleanup_in_progress: bool,
    pub last_cleanup_epoch: Option<u64>,
    pub last_cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupQueueReceipt {
    pub work_id: u64,
    pub cleanup_epoch: u64,
    pub cleanup_wave: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupPlan {
    pub work_id: u64,
    pub cleanup_epoch: u64,
    pub cleanup_wave: u64,
    pub sandbox_id: String,
    pub mode: LingeringCleanupMode,
    pub trigger: LingeringCleanupTrigger,
    pub retry_count: u32,
    pub processing_epoch: u64,
    pub ready_at_processing_epoch: u64,
    pub exclude_lease_id: Option<String>,
    pub exclude_region_id: Option<String>,
    pub candidates: Vec<ActiveTransportConcurrencySession>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingLingeringCleanupWaveSummary {
    pub sandbox_id: String,
    pub cleanup_wave: u64,
    pub mode: LingeringCleanupMode,
    pub first_trigger: LingeringCleanupTrigger,
    pub latest_trigger: LingeringCleanupTrigger,
    pub pending_work_items: usize,
    pub deferred_retry_work_items: usize,
    pub first_cleanup_epoch: u64,
    pub latest_cleanup_epoch: u64,
    pub first_processing_epoch: u64,
    pub latest_processing_epoch: u64,
    pub oldest_ready_at_processing_epoch: u64,
    pub newest_ready_at_processing_epoch: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransportConcurrencySnapshot {
    pub steady_session_limit: usize,
    pub recovery_session_limit: usize,
    pub current_attached_sessions: usize,
    pub peak_attached_sessions: usize,
    pub current_recovery_overlap_sessions: usize,
    pub peak_recovery_overlap_sessions: usize,
    pub current_lingering_sessions: usize,
    pub peak_lingering_sessions: usize,
    pub current_detach_requested_sessions: usize,
    pub current_detach_faulted_sessions: usize,
    pub pending_cleanup_work_items: usize,
    pub pending_deferred_retry_work_items: usize,
    pub next_cleanup_epoch: u64,
    pub oldest_pending_cleanup_ready_epoch: Option<u64>,
    pub pending_cleanup_waves: Vec<PendingLingeringCleanupWaveSummary>,
    pub active_sessions: Vec<ActiveTransportConcurrencySession>,
    pub last_admitted_sandbox_id: Option<String>,
    pub last_rejected_sandbox_id: Option<String>,
    pub last_rejection_reason: Option<String>,
}

/// Runtime control-plane summary.
///
/// Callers typically pair this with `RuntimeReadiness` and
/// `EffectiveRuntimeConfig` to decide whether the runtime has been handshaken,
/// configured, started, or restarted and which control request most recently
/// changed that state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeControlSnapshot {
    pub handshaken: bool,
    pub configured: bool,
    pub running: bool,
    pub handshake_count: u64,
    pub configure_count: u64,
    pub start_count: u64,
    pub stop_count: u64,
    pub restart_count: u64,
    pub last_client_version: Option<String>,
    pub last_stop_reason: Option<StopReason>,
    pub last_reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSchedulerState {
    #[default]
    Stopped,
    Configured,
    ReadyIdle,
    RealtimeOnly,
    Anticipative,
    Degraded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeExecutionPhase {
    #[default]
    Idle,
    Priming,
    Prework,
    Realtime,
    Degraded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerSnapshot {
    pub state: RuntimeSchedulerState,
    pub phase: RuntimeExecutionPhase,
    pub graph_applied: bool,
    pub schedule_applied: bool,
    pub transport_projected: bool,
    pub anticipative_enabled: bool,
    pub active_graph_id: Option<String>,
    pub phase_count: usize,
    pub lane_count: usize,
    pub dispatch_count: usize,
    pub pending_prework_target_count: usize,
    pub processed_block_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<DegradedReason> },
    Stopped,
    Failed { fatal: RuntimeError },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRuntimeConfig {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub safe_mode_enabled: bool,
    pub active_output_device: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub cpu_load_percent: f32,
    pub xruns: u64,
    pub graph_latency_ms: f32,
    pub active_plugin_sandboxes: u32,
    pub backend_policy_tier: BackendPolicyTier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryRecord {
    pub sandbox_id: String,
    pub intent: RecoveryRestartIntent,
    pub stop_reason: StopReason,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxLifecycleStage {
    SandboxEnsured,
    SandboxHandshaken,
    PluginTypeLoaded,
    InstanceCreated,
    InstancePrepared,
    TransportAttached,
    InstanceActivated,
    InstanceDeactivated,
    InstanceReset,
    InstanceDestroyed,
    SandboxTeardown,
    TransportTornDown,
    SandboxRestarted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxLifecycleRecord {
    pub sandbox_id: String,
    pub stage: PluginSandboxLifecycleStage,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxTransportStage {
    Attached,
    DetachRequested,
    Detached,
    DetachFault,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxTransportRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub stage: PluginSandboxTransportStage,
    pub processing_epoch: Option<u64>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceFaultRecord {
    pub kind: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceStateRecord {
    pub sandbox_id: String,
    pub plugin_type_id: String,
    pub instance_id: String,
    pub lifecycle_state: String,
    pub readiness_state: String,
    pub degraded_reasons: Vec<String>,
    pub active: bool,
    pub processing_epoch: Option<u64>,
    pub processing_sample_rate_hz: Option<u32>,
    pub processing_max_block_frames: Option<u32>,
    pub audio_inputs: Option<u16>,
    pub audio_outputs: Option<u16>,
    pub midi_inputs: Option<u16>,
    pub midi_outputs: Option<u16>,
    pub last_fault: Option<PluginSandboxInstanceFaultRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatCycleStage {
    Requested,
    Responded,
    Missed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatCycleRecord {
    pub sandbox_id: String,
    pub stage: HeartbeatCycleStage,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockDispatchStage {
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDispatchRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub frame_count: u32,
    pub stage: BlockDispatchStage,
    pub completion_state: Option<CompletionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseRolloverRecord {
    pub sandbox_id: String,
    pub previous_lease_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub first_block_sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerInvalidationStage {
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerInvalidationRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: Option<u64>,
    pub stage: BrokerInvalidationStage,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionSlotStage {
    ReadyForProcessing,
    Processing,
    Completed,
    TimedOut,
    Invalidated,
    FallbackApplied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionSlotRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub stage: CompletionSlotStage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerFailureStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub stage: BrokerFailureStage,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxOperationFailureStage {
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxOperationFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub operation: String,
    pub error_kind: String,
    pub stage: SandboxOperationFailureStage,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultSource {
    HostBroker,
    SandboxOperation,
    RuntimeDispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
    TransportDetachRequested,
    TransportDetached,
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
    TransportDetachFault,
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
    CompletionSlotTimedOut,
    CompletionSlotInvalidated,
    FallbackApplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultPhase {
    Prepare,
    Dispatch,
    Teardown,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultResource {
    PreparePlan,
    SharedMemoryPayload,
    SharedMemoryLease,
    CompletionSlot,
    ProcessProtocol,
    ControlProtocol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub source: TransportFaultSource,
    pub stage: TransportFaultStage,
    pub phase: TransportFaultPhase,
    pub resource: TransportFaultResource,
    pub operation: String,
    pub error_kind: Option<String>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultBoundaryMode {
    FaultAdjacentOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultSummary {
    pub boundary_mode: TransportFaultBoundaryMode,
    pub total_events: usize,
    pub host_broker_events: usize,
    pub sandbox_operation_events: usize,
    pub runtime_dispatch_events: usize,
    pub prepare_events: usize,
    pub dispatch_events: usize,
    pub teardown_events: usize,
    pub control_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
}

impl TransportFaultSummary {
    pub fn from_records(records: &[TransportFaultRecord]) -> Self {
        let mut summary = Self {
            boundary_mode: TransportFaultBoundaryMode::FaultAdjacentOnly,
            total_events: records.len(),
            host_broker_events: 0,
            sandbox_operation_events: 0,
            runtime_dispatch_events: 0,
            prepare_events: 0,
            dispatch_events: 0,
            teardown_events: 0,
            control_events: 0,
            first_processing_epoch: None,
            last_processing_epoch: None,
            first_block_sequence: None,
            last_block_sequence: None,
        };

        for record in records {
            match record.source {
                TransportFaultSource::HostBroker => {
                    summary.host_broker_events = summary.host_broker_events.saturating_add(1)
                }
                TransportFaultSource::SandboxOperation => {
                    summary.sandbox_operation_events =
                        summary.sandbox_operation_events.saturating_add(1)
                }
                TransportFaultSource::RuntimeDispatch => {
                    summary.runtime_dispatch_events =
                        summary.runtime_dispatch_events.saturating_add(1)
                }
            }
            match record.phase {
                TransportFaultPhase::Prepare => {
                    summary.prepare_events = summary.prepare_events.saturating_add(1)
                }
                TransportFaultPhase::Dispatch => {
                    summary.dispatch_events = summary.dispatch_events.saturating_add(1)
                }
                TransportFaultPhase::Teardown => {
                    summary.teardown_events = summary.teardown_events.saturating_add(1)
                }
                TransportFaultPhase::Control => {
                    summary.control_events = summary.control_events.saturating_add(1)
                }
            }

            if let Some(epoch) = record.processing_epoch {
                summary.first_processing_epoch = Some(
                    summary
                        .first_processing_epoch
                        .map_or(epoch, |current| current.min(epoch)),
                );
                summary.last_processing_epoch = Some(
                    summary
                        .last_processing_epoch
                        .map_or(epoch, |current| current.max(epoch)),
                );
            }
            if let Some(block_sequence) = record.block_sequence {
                summary.first_block_sequence = Some(
                    summary
                        .first_block_sequence
                        .map_or(block_sequence, |current| current.min(block_sequence)),
                );
                summary.last_block_sequence = Some(
                    summary
                        .last_block_sequence
                        .map_or(block_sequence, |current| current.max(block_sequence)),
                );
            }
        }

        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionBoundaryMode {
    HealthyPathVisible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionState {
    Detached,
    AttachActive,
    DetachRequested,
    DetachFaulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportHeartbeatFreshness {
    Unknown,
    Requested,
    Fresh,
    Missed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportDispatchState {
    Idle,
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportSessionRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub processing_epoch: Option<u64>,
    pub active_block_sequence: Option<u64>,
    pub transport_fault_count: usize,
    pub last_transport_fault_source: Option<TransportFaultSource>,
    pub last_transport_fault_stage: Option<TransportFaultStage>,
    pub last_transport_fault_phase: Option<TransportFaultPhase>,
    pub last_transport_fault_processing_epoch: Option<u64>,
    pub last_transport_fault_block_sequence: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportSessionSummary {
    pub boundary_mode: TransportSessionBoundaryMode,
    pub current_state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub current_attached_session_count: usize,
    pub max_concurrent_attached_sessions: usize,
    pub attach_events: usize,
    pub detach_requested_events: usize,
    pub detached_events: usize,
    pub detach_fault_events: usize,
    pub heartbeat_requested_events: usize,
    pub heartbeat_responded_events: usize,
    pub heartbeat_missed_events: usize,
    pub dispatch_requested_events: usize,
    pub dispatch_completed_events: usize,
    pub dispatch_timed_out_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub active_sandbox_id: Option<String>,
    pub active_lease_id: Option<String>,
    pub active_region_id: Option<String>,
    pub active_block_sequence: Option<u64>,
    pub active_sessions: Vec<ActiveTransportSessionRecord>,
    pub last_sandbox_id: Option<String>,
    pub last_lease_id: Option<String>,
    pub last_region_id: Option<String>,
}

impl TransportSessionSummary {
    pub fn from_diagnostics(diagnostics: &RuntimeObservationDiagnostics) -> Self {
        let mut summary = Self {
            boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
            current_state: TransportSessionState::Detached,
            currently_attached: false,
            heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
            dispatch_state: TransportDispatchState::Idle,
            current_attached_session_count: 0,
            max_concurrent_attached_sessions: 0,
            attach_events: 0,
            detach_requested_events: 0,
            detached_events: 0,
            detach_fault_events: 0,
            heartbeat_requested_events: 0,
            heartbeat_responded_events: 0,
            heartbeat_missed_events: 0,
            dispatch_requested_events: 0,
            dispatch_completed_events: 0,
            dispatch_timed_out_events: 0,
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
        };
        let mut active_sessions: BTreeMap<(String, String, String), ActiveTransportSessionRecord> =
            BTreeMap::new();
        let mut last_transport_key: Option<(String, String, String)> = None;

        for record in &diagnostics.transport_events {
            match record.stage {
                PluginSandboxTransportStage::Attached => {
                    summary.attach_events = summary.attach_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachRequested => {
                    summary.detach_requested_events =
                        summary.detach_requested_events.saturating_add(1)
                }
                PluginSandboxTransportStage::Detached => {
                    summary.detached_events = summary.detached_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachFault => {
                    summary.detach_fault_events = summary.detach_fault_events.saturating_add(1)
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            last_transport_key = Some(apply_transport_session_state(
                &mut summary,
                &mut active_sessions,
                record,
            ));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.last_region_id = Some(record.region_id.clone());
        }

        for record in &diagnostics.heartbeat_events {
            match record.stage {
                HeartbeatCycleStage::Requested => {
                    summary.heartbeat_requested_events =
                        summary.heartbeat_requested_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Requested;
                }
                HeartbeatCycleStage::Responded => {
                    summary.heartbeat_responded_events =
                        summary.heartbeat_responded_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Fresh;
                }
                HeartbeatCycleStage::Missed => {
                    summary.heartbeat_missed_events =
                        summary.heartbeat_missed_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Missed;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            update_transport_session_block_bounds(&mut summary, record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                None,
                last_transport_key.as_ref(),
            ) {
                session.heartbeat_freshness = summary.heartbeat_freshness;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
            }
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
        }

        for record in &diagnostics.block_dispatch_events {
            match record.stage {
                BlockDispatchStage::Requested => {
                    summary.dispatch_requested_events =
                        summary.dispatch_requested_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Requested;
                }
                BlockDispatchStage::Completed => {
                    summary.dispatch_completed_events =
                        summary.dispatch_completed_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Completed;
                }
                BlockDispatchStage::TimedOut => {
                    summary.dispatch_timed_out_events =
                        summary.dispatch_timed_out_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::TimedOut;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, Some(record.processing_epoch));
            update_transport_session_block_bounds(&mut summary, Some(record.block_sequence));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.active_block_sequence = Some(record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                Some(&record.lease_id),
                last_transport_key.as_ref(),
            ) {
                session.dispatch_state = summary.dispatch_state;
                session.processing_epoch = Some(record.processing_epoch);
                session.active_block_sequence = Some(record.block_sequence);
            }
        }

        for record in &diagnostics.transport_fault_events {
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                record.lease_id.as_deref(),
                last_transport_key.as_ref(),
            ) {
                session.transport_fault_count = session.transport_fault_count.saturating_add(1);
                session.last_transport_fault_source = Some(record.source);
                session.last_transport_fault_stage = Some(record.stage);
                session.last_transport_fault_phase = Some(record.phase);
                session.last_transport_fault_processing_epoch = record.processing_epoch;
                session.last_transport_fault_block_sequence = record.block_sequence;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
                if let Some(block_sequence) = record.block_sequence {
                    session.active_block_sequence = Some(block_sequence);
                }
            }
        }

        summary.current_attached_session_count = active_sessions.len();
        summary.active_sessions = active_sessions.into_values().collect();

        summary
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSchedulerExportSummary {
    pub phase_count: usize,
    pub anticipative_phase_count: usize,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub dispatch_count: usize,
    pub prepared_dispatch_count: usize,
    pub realtime_dispatch_count: usize,
    pub dispatch_handoff_count: usize,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub prework_pending_target_count: usize,
    pub prework_pending_deferred_target_count: usize,
    pub topology_compatible: bool,
    pub topology_requires_host_reinterpretation: bool,
    pub topology_issue_count: usize,
    pub lane_order: Vec<GraphExecutionLane>,
    pub dispatch_order: Vec<GraphExecutionLane>,
}

impl RuntimeSchedulerExportSummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        Self {
            phase_count: snapshot.phase_count,
            anticipative_phase_count: snapshot.anticipative_phase_count,
            lane_count: snapshot.lane_count,
            anticipative_lane_count: snapshot.anticipative_lane_count,
            dispatch_count: snapshot.dispatch_count,
            prepared_dispatch_count: snapshot.prepared_dispatch_count,
            realtime_dispatch_count: snapshot.realtime_dispatch_count,
            dispatch_handoff_count: snapshot.dispatch_handoff_count,
            prework_service_state: snapshot.prework_service_state,
            prework_service_pressure: snapshot.prework_service_pressure,
            prework_service_semantic_policy: snapshot.prework_service_semantic_policy,
            prework_pending_target_count: snapshot.prework_pending_target_count,
            prework_pending_deferred_target_count: snapshot.prework_pending_deferred_target_count,
            topology_compatible: snapshot.scheduler_topology.compatible,
            topology_requires_host_reinterpretation: snapshot
                .scheduler_topology
                .requires_host_reinterpretation,
            topology_issue_count: snapshot.scheduler_topology.issues.len(),
            lane_order: snapshot.lane_order.clone(),
            dispatch_order: snapshot.dispatch_order.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeBlockExecutionSummary {
    pub processed_blocks: u64,
    pub last_processing_epoch: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub last_frame_count: usize,
    pub last_channel_count: usize,
    pub prework_cache_state: RuntimePreworkCacheState,
    pub prework_cache_freshness_state: RuntimePreworkFreshnessState,
    pub last_prework_invalidation_reason: Option<RuntimePreworkInvalidationReason>,
    pub total_latency_samples: u32,
    pub total_tail_samples: u32,
    pub output_tail_samples: u32,
    pub max_bus_tail_samples: u32,
    pub last_input_peak: Option<f32>,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub transport_epoch: u64,
    pub transport_transition: Option<RuntimeTransportTransitionKind>,
    pub transport_loop_wrapped: bool,
    pub context_anticipative: Option<bool>,
    pub transport_playing: Option<bool>,
    pub transport_tempo_bpm: Option<f64>,
    pub timeline_position_samples: Option<i64>,
}

impl RuntimeBlockExecutionSummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        Self {
            processed_blocks: snapshot.processed_blocks,
            last_processing_epoch: snapshot.last_processing_epoch,
            last_block_sequence: snapshot.last_block_sequence,
            last_frame_count: snapshot.last_frame_count,
            last_channel_count: snapshot.last_channel_count,
            prework_cache_state: snapshot.prework_cache_state,
            prework_cache_freshness_state: snapshot.prework_cache_freshness_state,
            last_prework_invalidation_reason: snapshot.last_prework_invalidation_reason,
            total_latency_samples: snapshot.total_latency_samples,
            total_tail_samples: snapshot.total_tail_samples,
            output_tail_samples: snapshot.output_tail_samples,
            max_bus_tail_samples: snapshot.max_bus_tail_samples,
            last_input_peak: snapshot.last_input_peak,
            last_output_peak: snapshot.last_output_peak,
            last_output_rms: snapshot.last_output_rms,
            transport_epoch: snapshot.transport_epoch,
            transport_transition: snapshot.transport_transition,
            transport_loop_wrapped: snapshot.transport_loop_wrapped,
            context_anticipative: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            transport_playing: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            transport_tempo_bpm: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            timeline_position_samples: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeDegradationSummary {
    pub readiness_degraded: bool,
    pub safe_mode_enabled: bool,
    pub xrun_count: u64,
    pub plugin_fault_count: usize,
    pub transport_fault_event_count: usize,
    pub broker_failure_event_count: usize,
    pub sandbox_operation_failure_event_count: usize,
    pub recovery_event_count: usize,
    pub active_plugin_sandboxes: u32,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub recovery_overlap_sessions: usize,
    pub lingering_sessions: usize,
    pub detach_faulted_sessions: usize,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
}

impl RuntimeDegradationSummary {
    pub fn capture(
        readiness: &RuntimeReadiness,
        diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
        supervision_snapshot: &RuntimeSupervisionSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        transport_concurrency_snapshot: &RuntimeTransportConcurrencySnapshot,
        observation: &RuntimeObservationDiagnostics,
    ) -> Self {
        Self {
            readiness_degraded: matches!(readiness, RuntimeReadiness::Degraded { .. }),
            safe_mode_enabled: supervision_snapshot.safe_mode_enabled,
            xrun_count: diagnostics_snapshot.xruns,
            plugin_fault_count: observation.plugin_fault_count(),
            transport_fault_event_count: observation.transport_fault_event_count(),
            broker_failure_event_count: observation.broker_failure_event_count(),
            sandbox_operation_failure_event_count: observation
                .sandbox_operation_failure_event_count(),
            recovery_event_count: observation.recovery_event_count(),
            active_plugin_sandboxes: diagnostics_snapshot.active_plugin_sandboxes,
            degraded_bound_plugin_sandboxes: engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            missing_bound_plugin_sandboxes: engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            recovery_overlap_sessions: engine_block_snapshot
                .prework_service_recovery_overlap_sessions,
            lingering_sessions: engine_block_snapshot.prework_service_lingering_sessions,
            detach_faulted_sessions: transport_concurrency_snapshot.current_detach_faulted_sessions,
            transport_gate_active: engine_block_snapshot.prework_service_transport_gate_active,
            plugin_gate_active: engine_block_snapshot.prework_service_plugin_gate_active,
            last_watchdog_trigger: observation
                .last_supervision_update()
                .and_then(|snapshot| snapshot.last_watchdog_trigger)
                .or(supervision_snapshot.last_watchdog_trigger),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostAudioStreamState {
    Stopped,
    Running,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeHostAudioTransferPolicy {
    pub max_callback_frames: usize,
    pub max_transfer_channels: u16,
    pub zero_fill_unwritten_output: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostLifecycleOwnership {
    HostDrivenCallback,
    BackendManagedCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostRestartPolicy {
    HostMustRestart,
    BackendMayRestart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockSource {
    Internal,
    ExternalWordClock,
    DigitalInput,
    Virtual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostClockingSummary {
    pub clock_source: RuntimeHostClockSource,
    pub ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub callback_interval_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostLatencySummary {
    pub input_latency_samples: Option<u32>,
    pub output_latency_samples: u32,
    pub round_trip_latency_samples: Option<u32>,
    pub graph_latency_samples: u32,
    pub estimated_output_latency_samples: u32,
    pub estimated_round_trip_latency_samples: Option<u32>,
    pub output_latency_ms: f32,
    pub graph_latency_ms: f32,
    pub estimated_output_latency_ms: f32,
    pub estimated_round_trip_latency_ms: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHostHardwareSummary {
    pub backend_name: String,
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub simulated: bool,
    pub backend_health: BackendHealth,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostAudioPumpSummary {
    pub stream_state: RuntimeHostAudioStreamState,
    pub transfer_policy: RuntimeHostAudioTransferPolicy,
    pub callback_count: u64,
    pub total_callback_frames: u64,
    pub total_runtime_output_frames: u64,
    pub copied_output_samples: u64,
    pub zero_filled_output_samples: u64,
    pub dropped_output_samples: u64,
    pub last_callback_output_peak: Option<f32>,
    pub last_runtime_graph_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostIoSummary {
    pub hardware: RuntimeHostHardwareSummary,
    pub audio_pump: RuntimeHostAudioPumpSummary,
    pub clocking: RuntimeHostClockingSummary,
    pub latency: RuntimeHostLatencySummary,
    pub runtime_graph_id_matches_pump: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostObservationReport {
    pub observation: RuntimeObservationReport,
    pub host_io: RuntimeHostIoSummary,
}

impl RuntimeHostObservationReport {
    pub fn new(observation: RuntimeObservationReport, host_io: RuntimeHostIoSummary) -> Self {
        Self {
            observation,
            host_io,
        }
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} host_backend={} host_device={} host_stream_state={:?} host_clock_source={:?} host_clock_ownership={:?} host_clock_restart_policy={:?} host_callback_interval_ms={:.3} host_output_latency_samples={} host_graph_latency_samples={} host_estimated_output_latency_samples={} host_backend_health={:?} host_backend_xruns={} host_backend_device_losses={} host_backend_restart_attempts={} host_backend_restart_failures={} host_audio_callbacks={} host_audio_frames={} host_audio_copied_samples={} host_audio_zero_filled_samples={} host_audio_dropped_samples={} host_audio_peak={:?} host_audio_graph={:?} host_audio_graph_matches_runtime={}",
            self.observation.render_compact(),
            self.host_io.hardware.backend_name,
            self.host_io.hardware.device_id,
            self.host_io.audio_pump.stream_state,
            self.host_io.clocking.clock_source,
            self.host_io.clocking.ownership,
            self.host_io.clocking.restart_policy,
            self.host_io.clocking.callback_interval_ms,
            self.host_io.latency.output_latency_samples,
            self.host_io.latency.graph_latency_samples,
            self.host_io.latency.estimated_output_latency_samples,
            self.host_io.hardware.backend_health,
            self.host_io.hardware.xrun_count,
            self.host_io.hardware.device_loss_count,
            self.host_io.hardware.restart_attempt_count,
            self.host_io.hardware.restart_failure_count,
            self.host_io.audio_pump.callback_count,
            self.host_io.audio_pump.total_callback_frames,
            self.host_io.audio_pump.copied_output_samples,
            self.host_io.audio_pump.zero_filled_output_samples,
            self.host_io.audio_pump.dropped_output_samples,
            self.host_io.audio_pump.last_callback_output_peak,
            self.host_io.audio_pump.last_runtime_graph_id,
            self.host_io.runtime_graph_id_matches_pump,
        )
    }

    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "observation={}",
                "\nhost_backend={}",
                "\nhost_device_id={}",
                "\nhost_device_name={}",
                "\nhost_sample_rate={}",
                "\nhost_buffer_size={}",
                "\nhost_output_channels={}",
                "\nhost_sample_format={:?}",
                "\nhost_simulated={}",
                "\nhost_clock_source={:?}",
                "\nhost_clock_ownership={:?}",
                "\nhost_clock_restart_policy={:?}",
                "\nhost_callback_interval_ms={:.3}",
                "\nhost_input_latency_samples={:?}",
                "\nhost_output_latency_samples={}",
                "\nhost_round_trip_latency_samples={:?}",
                "\nhost_graph_latency_samples={}",
                "\nhost_estimated_output_latency_samples={}",
                "\nhost_estimated_round_trip_latency_samples={:?}",
                "\nhost_output_latency_ms={:.3}",
                "\nhost_graph_latency_ms={:.3}",
                "\nhost_estimated_output_latency_ms={:.3}",
                "\nhost_estimated_round_trip_latency_ms={:?}",
                "\nhost_backend_health={:?}",
                "\nhost_backend_xruns={}",
                "\nhost_backend_callback_overruns={}",
                "\nhost_backend_device_losses={}",
                "\nhost_backend_restart_attempts={}",
                "\nhost_backend_restart_failures={}",
                "\nhost_audio_stream_state={:?}",
                "\nhost_audio_callback_count={}",
                "\nhost_audio_total_callback_frames={}",
                "\nhost_audio_total_runtime_output_frames={}",
                "\nhost_audio_copied_output_samples={}",
                "\nhost_audio_zero_filled_output_samples={}",
                "\nhost_audio_dropped_output_samples={}",
                "\nhost_audio_last_callback_output_peak={:?}",
                "\nhost_audio_last_runtime_graph_id={:?}",
                "\nhost_audio_graph_matches_runtime={}",
            ),
            self.observation.render_compact(),
            self.host_io.hardware.backend_name,
            self.host_io.hardware.device_id,
            self.host_io.hardware.device_name,
            self.host_io.hardware.sample_rate,
            self.host_io.hardware.buffer_size,
            self.host_io.hardware.output_channels,
            self.host_io.hardware.sample_format,
            self.host_io.hardware.simulated,
            self.host_io.clocking.clock_source,
            self.host_io.clocking.ownership,
            self.host_io.clocking.restart_policy,
            self.host_io.clocking.callback_interval_ms,
            self.host_io.latency.input_latency_samples,
            self.host_io.latency.output_latency_samples,
            self.host_io.latency.round_trip_latency_samples,
            self.host_io.latency.graph_latency_samples,
            self.host_io.latency.estimated_output_latency_samples,
            self.host_io.latency.estimated_round_trip_latency_samples,
            self.host_io.latency.output_latency_ms,
            self.host_io.latency.graph_latency_ms,
            self.host_io.latency.estimated_output_latency_ms,
            self.host_io.latency.estimated_round_trip_latency_ms,
            self.host_io.hardware.backend_health,
            self.host_io.hardware.xrun_count,
            self.host_io.hardware.callback_overrun_count,
            self.host_io.hardware.device_loss_count,
            self.host_io.hardware.restart_attempt_count,
            self.host_io.hardware.restart_failure_count,
            self.host_io.audio_pump.stream_state,
            self.host_io.audio_pump.callback_count,
            self.host_io.audio_pump.total_callback_frames,
            self.host_io.audio_pump.total_runtime_output_frames,
            self.host_io.audio_pump.copied_output_samples,
            self.host_io.audio_pump.zero_filled_output_samples,
            self.host_io.audio_pump.dropped_output_samples,
            self.host_io.audio_pump.last_callback_output_peak,
            self.host_io.audio_pump.last_runtime_graph_id,
            self.host_io.runtime_graph_id_matches_pump,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"observation\":{{",
                "\"readiness\":{},",
                "\"xruns\":{},",
                "\"engine_graph_id\":{},",
                "\"degradation_summary\":{},",
                "\"execution_topology_summary\":{}",
                "}},",
                "\"host_io\":{{",
                "\"hardware\":{{",
                "\"backend_name\":{},",
                "\"device_id\":{},",
                "\"device_name\":{},",
                "\"sample_rate\":{},",
                "\"buffer_size\":{},",
                "\"output_channels\":{},",
                "\"sample_format\":{},",
                "\"simulated\":{},",
                "\"clocking\":{{",
                "\"clock_source\":{},",
                "\"ownership\":{},",
                "\"restart_policy\":{},",
                "\"callback_interval_ms\":{}",
                "}},",
                "\"latency\":{{",
                "\"input_latency_samples\":{},",
                "\"output_latency_samples\":{},",
                "\"round_trip_latency_samples\":{},",
                "\"graph_latency_samples\":{},",
                "\"estimated_output_latency_samples\":{},",
                "\"estimated_round_trip_latency_samples\":{},",
                "\"output_latency_ms\":{},",
                "\"graph_latency_ms\":{},",
                "\"estimated_output_latency_ms\":{},",
                "\"estimated_round_trip_latency_ms\":{}",
                "}},",
                "\"backend_health\":{},",
                "\"xrun_count\":{},",
                "\"callback_overrun_count\":{},",
                "\"device_loss_count\":{},",
                "\"restart_attempt_count\":{},",
                "\"restart_failure_count\":{}",
                "}},",
                "\"audio_pump\":{{",
                "\"stream_state\":{},",
                "\"max_callback_frames\":{},",
                "\"max_transfer_channels\":{},",
                "\"zero_fill_unwritten_output\":{},",
                "\"callback_count\":{},",
                "\"total_callback_frames\":{},",
                "\"total_runtime_output_frames\":{},",
                "\"copied_output_samples\":{},",
                "\"zero_filled_output_samples\":{},",
                "\"dropped_output_samples\":{},",
                "\"last_callback_output_peak\":{},",
                "\"last_runtime_graph_id\":{}",
                "}},",
                "\"runtime_graph_id_matches_pump\":{}",
                "}}",
                "}}"
            ),
            json_option_string(Some(match self.observation.readiness {
                RuntimeReadiness::Starting => "Starting",
                RuntimeReadiness::Ready => "Ready",
                RuntimeReadiness::Degraded { .. } => "Degraded",
                RuntimeReadiness::Stopped => "Stopped",
                RuntimeReadiness::Failed { .. } => "Failed",
            })),
            self.observation.diagnostics_snapshot.xruns,
            json_option_string(self.observation.engine_block_snapshot.graph_id.as_deref()),
            json_runtime_degradation_summary(&self.observation.degradation_summary),
            json_runtime_execution_topology_summary(&self.observation.execution_topology_summary),
            json_option_string(Some(self.host_io.hardware.backend_name.as_str())),
            json_option_string(Some(self.host_io.hardware.device_id.as_str())),
            json_option_string(Some(self.host_io.hardware.device_name.as_str())),
            self.host_io.hardware.sample_rate,
            self.host_io.hardware.buffer_size,
            self.host_io.hardware.output_channels,
            json_option_string(Some(match self.host_io.hardware.sample_format {
                AudioSampleFormat::F32 => "F32",
                AudioSampleFormat::I16 => "I16",
                AudioSampleFormat::I32 => "I32",
            })),
            self.host_io.hardware.simulated,
            json_option_string(Some(match self.host_io.clocking.clock_source {
                RuntimeHostClockSource::Internal => "Internal",
                RuntimeHostClockSource::ExternalWordClock => "ExternalWordClock",
                RuntimeHostClockSource::DigitalInput => "DigitalInput",
                RuntimeHostClockSource::Virtual => "Virtual",
            })),
            json_option_string(Some(match self.host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => "HostDrivenCallback",
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    "BackendManagedCallback"
                }
            })),
            json_option_string(Some(match self.host_io.clocking.restart_policy {
                RuntimeHostRestartPolicy::HostMustRestart => "HostMustRestart",
                RuntimeHostRestartPolicy::BackendMayRestart => "BackendMayRestart",
            })),
            self.host_io.clocking.callback_interval_ms,
            json_option_u32(self.host_io.latency.input_latency_samples),
            self.host_io.latency.output_latency_samples,
            json_option_u32(self.host_io.latency.round_trip_latency_samples),
            self.host_io.latency.graph_latency_samples,
            self.host_io.latency.estimated_output_latency_samples,
            json_option_u32(self.host_io.latency.estimated_round_trip_latency_samples),
            self.host_io.latency.output_latency_ms,
            self.host_io.latency.graph_latency_ms,
            self.host_io.latency.estimated_output_latency_ms,
            json_option_f32(self.host_io.latency.estimated_round_trip_latency_ms),
            json_option_string(Some(match self.host_io.hardware.backend_health {
                BackendHealth::Healthy => "Healthy",
                BackendHealth::Degraded => "Degraded",
                BackendHealth::Recovering => "Recovering",
            })),
            self.host_io.hardware.xrun_count,
            self.host_io.hardware.callback_overrun_count,
            self.host_io.hardware.device_loss_count,
            self.host_io.hardware.restart_attempt_count,
            self.host_io.hardware.restart_failure_count,
            json_option_string(Some(match self.host_io.audio_pump.stream_state {
                RuntimeHostAudioStreamState::Stopped => "Stopped",
                RuntimeHostAudioStreamState::Running => "Running",
                RuntimeHostAudioStreamState::Faulted => "Faulted",
            })),
            self.host_io.audio_pump.transfer_policy.max_callback_frames,
            self.host_io
                .audio_pump
                .transfer_policy
                .max_transfer_channels,
            self.host_io
                .audio_pump
                .transfer_policy
                .zero_fill_unwritten_output,
            self.host_io.audio_pump.callback_count,
            self.host_io.audio_pump.total_callback_frames,
            self.host_io.audio_pump.total_runtime_output_frames,
            self.host_io.audio_pump.copied_output_samples,
            self.host_io.audio_pump.zero_filled_output_samples,
            self.host_io.audio_pump.dropped_output_samples,
            json_option_f32(self.host_io.audio_pump.last_callback_output_peak),
            json_option_string(self.host_io.audio_pump.last_runtime_graph_id.as_deref()),
            self.host_io.runtime_graph_id_matches_pump,
        )
    }
}

impl From<HardwareLifecycleOwnership> for RuntimeHostLifecycleOwnership {
    fn from(value: HardwareLifecycleOwnership) -> Self {
        match value {
            HardwareLifecycleOwnership::HostDrivenCallback => Self::HostDrivenCallback,
            HardwareLifecycleOwnership::BackendManagedCallback => Self::BackendManagedCallback,
        }
    }
}

impl From<HardwareRestartPolicy> for RuntimeHostRestartPolicy {
    fn from(value: HardwareRestartPolicy) -> Self {
        match value {
            HardwareRestartPolicy::HostMustRestart => Self::HostMustRestart,
            HardwareRestartPolicy::BackendMayRestart => Self::BackendMayRestart,
        }
    }
}

impl From<HardwareClockSource> for RuntimeHostClockSource {
    fn from(value: HardwareClockSource) -> Self {
        match value {
            HardwareClockSource::Internal => Self::Internal,
            HardwareClockSource::ExternalWordClock => Self::ExternalWordClock,
            HardwareClockSource::DigitalInput => Self::DigitalInput,
            HardwareClockSource::Virtual => Self::Virtual,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostSupervisorReport {
    pub observation: RuntimeHostObservationReport,
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeHostSupervisorReport {
    pub fn new(supervisor: RuntimeSupervisorReport, host_io: RuntimeHostIoSummary) -> Self {
        Self {
            observation: RuntimeHostObservationReport::new(supervisor.observation, host_io),
            events: supervisor.events,
        }
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} event_stream={}",
            self.observation.render_compact(),
            self.events.len()
        )
    }

    pub fn render_multiline(&self) -> String {
        format!(
            "{}\nevent_stream={}",
            self.observation.render_multiline(),
            self.events.len()
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"observation\":{},\"event_stream\":{}}}",
            self.observation.render_json(),
            self.events.len()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionLaneSummary {
    pub lane: GraphExecutionLane,
    pub groups: Vec<GraphNodePlanningGroup>,
    pub node_ids: Vec<String>,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub track_lane_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    pub node_id: String,
    pub lane: GraphExecutionLane,
    pub group: GraphNodePlanningGroup,
    pub execution_class: GraphNodeExecutionClass,
    pub topology_role: GraphNodeTopologyRole,
    pub lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub plugin_sandbox_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeExecutionTopologySummary {
    pub node_count: usize,
    pub utility_node_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub lane_count: usize,
    pub track_lane_group_count: usize,
    pub bus_group_count: usize,
    pub console_group_count: usize,
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
}

impl RuntimeExecutionTopologySummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        let mut track_lane_ids = std::collections::BTreeSet::new();
        let mut bus_group_ids = std::collections::BTreeSet::new();
        let mut console_group_ids = std::collections::BTreeSet::new();
        let mut lanes = Vec::new();

        for lane in &snapshot.lane_order {
            let mut groups = Vec::new();
            let mut node_ids = Vec::new();
            let mut topology_roles = Vec::new();
            let mut lane_ids = Vec::new();
            let mut bus_groups = Vec::new();

            for node in snapshot
                .planned_nodes
                .iter()
                .filter(|node| runtime_lane_for_group(node.group) == *lane)
            {
                if !groups.contains(&node.group) {
                    groups.push(node.group);
                }
                node_ids.push(node.node_id.clone());
                if !topology_roles.contains(&node.topology_role) {
                    topology_roles.push(node.topology_role);
                }
                if let Some(lane_id) = &node.lane_id {
                    if !lane_ids.contains(lane_id) {
                        lane_ids.push(lane_id.clone());
                    }
                    track_lane_ids.insert(lane_id.clone());
                }
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !bus_groups.contains(bus_group_id) {
                        bus_groups.push(bus_group_id.clone());
                    }
                    match node.topology_role {
                        GraphNodeTopologyRole::Bus
                        | GraphNodeTopologyRole::Send
                        | GraphNodeTopologyRole::Return => {
                            bus_group_ids.insert(bus_group_id.clone());
                        }
                        GraphNodeTopologyRole::ConsoleNode => {
                            console_group_ids.insert(bus_group_id.clone());
                        }
                        GraphNodeTopologyRole::Utility | GraphNodeTopologyRole::TrackLane => {}
                    }
                }
            }

            lanes.push(RuntimeExecutionLaneSummary {
                lane: *lane,
                groups,
                node_ids,
                topology_roles,
                track_lane_ids: lane_ids,
                bus_group_ids: bus_groups,
            });
        }

        let mut nodes = Vec::with_capacity(snapshot.planned_nodes.len());
        let mut utility_node_count = 0usize;
        let mut track_lane_node_count = 0usize;
        let mut bus_node_count = 0usize;
        let mut send_return_node_count = 0usize;
        let mut console_node_count = 0usize;

        for node in &snapshot.planned_nodes {
            match node.topology_role {
                GraphNodeTopologyRole::Utility => utility_node_count += 1,
                GraphNodeTopologyRole::TrackLane => track_lane_node_count += 1,
                GraphNodeTopologyRole::Bus => bus_node_count += 1,
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                    send_return_node_count += 1;
                }
                GraphNodeTopologyRole::ConsoleNode => console_node_count += 1,
            }
            nodes.push(RuntimeExecutionNodeSummary {
                node_id: node.node_id.clone(),
                lane: runtime_lane_for_group(node.group),
                group: node.group,
                execution_class: node.execution_class,
                topology_role: node.topology_role,
                lane_id: node.lane_id.clone(),
                bus_group_id: node.bus_group_id.clone(),
                input_bus_id: node.input_bus_id.clone(),
                output_bus_id: node.output_bus_id.clone(),
                plugin_sandbox_id: node.plugin_sandbox_id.clone(),
            });
        }

        Self {
            node_count: snapshot.planned_nodes.len(),
            utility_node_count,
            track_lane_node_count,
            bus_node_count,
            send_return_node_count,
            console_node_count,
            lane_count: lanes.len(),
            track_lane_group_count: track_lane_ids.len(),
            bus_group_count: bus_group_ids.len(),
            console_group_count: console_group_ids.len(),
            lanes,
            nodes,
        }
    }
}

fn runtime_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
    }
}

fn apply_transport_session_state(
    summary: &mut TransportSessionSummary,
    active_sessions: &mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    record: &PluginSandboxTransportRecord,
) -> (String, String, String) {
    let key = (
        record.sandbox_id.clone(),
        record.lease_id.clone(),
        record.region_id.clone(),
    );
    let prior_session = active_sessions.remove(&key);
    match record.stage {
        PluginSandboxTransportStage::Attached => {
            summary.current_state = TransportSessionState::AttachActive;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::AttachActive,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::DetachRequested => {
            summary.current_state = TransportSessionState::DetachRequested;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::DetachRequested,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::Detached => {
            summary.current_state = TransportSessionState::Detached;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
        PluginSandboxTransportStage::DetachFault => {
            summary.current_state = TransportSessionState::DetachFaulted;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
    }
    summary.max_concurrent_attached_sessions = summary
        .max_concurrent_attached_sessions
        .max(active_sessions.len());
    key
}

fn resolve_active_session_mut<'a>(
    active_sessions: &'a mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    sandbox_id: &str,
    lease_id: Option<&str>,
    last_transport_key: Option<&(String, String, String)>,
) -> Option<&'a mut ActiveTransportSessionRecord> {
    if let Some(lease_id) = lease_id {
        if let Some(key) = active_sessions
            .keys()
            .find(|(sandbox, lease, _)| sandbox == sandbox_id && lease == lease_id)
            .cloned()
        {
            return active_sessions.get_mut(&key);
        }
    }

    if let Some(key) = last_transport_key {
        if key.0 == sandbox_id {
            return active_sessions.get_mut(key);
        }
    }

    let fallback_key = active_sessions
        .keys()
        .rev()
        .find(|(sandbox, _, _)| sandbox == sandbox_id)
        .cloned()?;
    active_sessions.get_mut(&fallback_key)
}

fn update_transport_session_epoch_bounds(
    summary: &mut TransportSessionSummary,
    epoch: Option<u64>,
) {
    if let Some(epoch) = epoch {
        summary.first_processing_epoch = Some(
            summary
                .first_processing_epoch
                .map_or(epoch, |current| current.min(epoch)),
        );
        summary.last_processing_epoch = Some(
            summary
                .last_processing_epoch
                .map_or(epoch, |current| current.max(epoch)),
        );
    }
}

fn update_transport_session_block_bounds(
    summary: &mut TransportSessionSummary,
    block_sequence: Option<u64>,
) {
    if let Some(block_sequence) = block_sequence {
        summary.first_block_sequence = Some(
            summary
                .first_block_sequence
                .map_or(block_sequence, |current| current.min(block_sequence)),
        );
        summary.last_block_sequence = Some(
            summary
                .last_block_sequence
                .map_or(block_sequence, |current| current.max(block_sequence)),
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    ReadinessChanged(RuntimeReadiness),
    EffectiveConfigChanged(EffectiveRuntimeConfig),
    SupervisionChanged(RuntimeSupervisionSnapshot),
    PluginSandboxChanged {
        active_sandboxes: u32,
    },
    PluginSandboxFault {
        sandbox_id: String,
        kind: PluginFaultKind,
        detail: String,
        processing_epoch: Option<u64>,
    },
    RecoveryCycle {
        sandbox_id: String,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    },
    PluginSandboxLifecycle {
        sandbox_id: String,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    },
    PluginSandboxInstanceState {
        state: PluginSandboxInstanceStateRecord,
    },
    PluginSandboxTransport {
        sandbox_id: String,
        lease_id: String,
        region_id: String,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    },
    HeartbeatCycle {
        sandbox_id: String,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    },
    BlockDispatch {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    },
    LeaseRollover {
        sandbox_id: String,
        previous_lease_id: String,
        lease_id: String,
        processing_epoch: u64,
        first_block_sequence: u64,
    },
    BrokerInvalidation {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: String,
    },
    CompletionSlotTransition {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    },
    BrokerFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: String,
    },
    SandboxOperationFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: String,
        error_kind: String,
        stage: SandboxOperationFailureStage,
        detail: String,
    },
    HardwareDeviceChanged {
        device_id: Option<String>,
    },
}

pub trait RuntimeEventSink: Send {
    fn push(&mut self, event: RuntimeEvent);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultRecord {
    pub sandbox_id: String,
    pub kind: PluginFaultKind,
    pub detail: String,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservationDiagnostics {
    pub total_events: usize,
    pub supervision_updates: Vec<RuntimeSupervisionSnapshot>,
    pub plugin_faults: Vec<PluginFaultRecord>,
    pub plugin_instance_states: Vec<PluginSandboxInstanceStateRecord>,
    pub recovery_events: Vec<RecoveryRecord>,
    pub lifecycle_events: Vec<PluginSandboxLifecycleRecord>,
    pub transport_events: Vec<PluginSandboxTransportRecord>,
    pub heartbeat_events: Vec<HeartbeatCycleRecord>,
    pub block_dispatch_events: Vec<BlockDispatchRecord>,
    pub lease_rollover_events: Vec<LeaseRolloverRecord>,
    pub invalidation_events: Vec<BrokerInvalidationRecord>,
    pub completion_slot_events: Vec<CompletionSlotRecord>,
    pub transport_fault_events: Vec<TransportFaultRecord>,
    pub broker_failure_events: Vec<BrokerFailureRecord>,
    pub sandbox_operation_failure_events: Vec<SandboxOperationFailureRecord>,
}

impl RuntimeObservationDiagnostics {
    pub fn supervision_update_count(&self) -> usize {
        self.supervision_updates.len()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.plugin_faults.len()
    }

    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.plugin_instance_states.len()
    }

    pub fn recovery_event_count(&self) -> usize {
        self.recovery_events.len()
    }

    pub fn lifecycle_event_count(&self) -> usize {
        self.lifecycle_events.len()
    }

    pub fn transport_event_count(&self) -> usize {
        self.transport_events.len()
    }

    pub fn heartbeat_event_count(&self) -> usize {
        self.heartbeat_events.len()
    }

    pub fn block_dispatch_event_count(&self) -> usize {
        self.block_dispatch_events.len()
    }

    pub fn lease_rollover_event_count(&self) -> usize {
        self.lease_rollover_events.len()
    }

    pub fn invalidation_event_count(&self) -> usize {
        self.invalidation_events.len()
    }

    pub fn completion_slot_event_count(&self) -> usize {
        self.completion_slot_events.len()
    }

    pub fn transport_fault_event_count(&self) -> usize {
        self.transport_fault_events.len()
    }

    pub fn broker_failure_event_count(&self) -> usize {
        self.broker_failure_events.len()
    }

    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.sandbox_operation_failure_events.len()
    }

    pub fn fault_detail_count_containing(&self, needle: &str) -> usize {
        self.plugin_faults
            .iter()
            .filter(|fault| fault.detail.contains(needle))
            .count()
    }

    pub fn last_supervision_update(&self) -> Option<&RuntimeSupervisionSnapshot> {
        self.supervision_updates.last()
    }

    pub fn last_recovery_event(&self) -> Option<&RecoveryRecord> {
        self.recovery_events.last()
    }

    pub fn last_plugin_instance_state(&self) -> Option<&PluginSandboxInstanceStateRecord> {
        self.plugin_instance_states.last()
    }

    pub fn last_lifecycle_event(&self) -> Option<&PluginSandboxLifecycleRecord> {
        self.lifecycle_events.last()
    }

    pub fn last_transport_event(&self) -> Option<&PluginSandboxTransportRecord> {
        self.transport_events.last()
    }

    pub fn last_heartbeat_event(&self) -> Option<&HeartbeatCycleRecord> {
        self.heartbeat_events.last()
    }

    pub fn last_block_dispatch_event(&self) -> Option<&BlockDispatchRecord> {
        self.block_dispatch_events.last()
    }

    pub fn last_lease_rollover_event(&self) -> Option<&LeaseRolloverRecord> {
        self.lease_rollover_events.last()
    }

    pub fn last_invalidation_event(&self) -> Option<&BrokerInvalidationRecord> {
        self.invalidation_events.last()
    }

    pub fn last_completion_slot_event(&self) -> Option<&CompletionSlotRecord> {
        self.completion_slot_events.last()
    }

    pub fn last_transport_fault_event(&self) -> Option<&TransportFaultRecord> {
        self.transport_fault_events.last()
    }

    pub fn last_broker_failure_event(&self) -> Option<&BrokerFailureRecord> {
        self.broker_failure_events.last()
    }

    pub fn last_sandbox_operation_failure_event(&self) -> Option<&SandboxOperationFailureRecord> {
        self.sandbox_operation_failure_events.last()
    }

    pub fn render_compact(&self) -> String {
        let last_trigger = self
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
            .map(|trigger| format!("{trigger:?}"))
            .unwrap_or_else(|| "none".into());
        let last_fault = self
            .plugin_faults
            .last()
            .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
            .unwrap_or_else(|| "none".into());
        let last_recovery = self
            .last_recovery_event()
            .map(|recovery| {
                format!(
                    "{}:{:?}:{:?}@{:?}",
                    recovery.sandbox_id,
                    recovery.intent,
                    recovery.stop_reason,
                    recovery.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_plugin_instance_state = self
            .last_plugin_instance_state()
            .map(|state| {
                format!(
                    "{}:{}:{}/{}/active={}@{:?}",
                    state.sandbox_id,
                    state.instance_id,
                    state.lifecycle_state,
                    state.readiness_state,
                    state.active,
                    state.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_lifecycle = self
            .last_lifecycle_event()
            .map(|lifecycle| {
                format!(
                    "{}:{:?}@{:?}",
                    lifecycle.sandbox_id, lifecycle.stage, lifecycle.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport = self
            .last_transport_event()
            .map(|transport| {
                format!(
                    "{}:{}:{}:{:?}@{:?}",
                    transport.sandbox_id,
                    transport.lease_id,
                    transport.region_id,
                    transport.stage,
                    transport.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_heartbeat = self
            .last_heartbeat_event()
            .map(|heartbeat| {
                format!(
                    "{}:{:?}@{:?}/block={:?}",
                    heartbeat.sandbox_id,
                    heartbeat.stage,
                    heartbeat.processing_epoch,
                    heartbeat.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_dispatch = self
            .last_block_dispatch_event()
            .map(|dispatch| {
                format!(
                    "{}:{}:{:?}/block={}@{}",
                    dispatch.sandbox_id,
                    dispatch.lease_id,
                    dispatch.stage,
                    dispatch.block_sequence,
                    dispatch.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_rollover = self
            .last_lease_rollover_event()
            .map(|rollover| {
                format!(
                    "{}:{}->{}@{}/block={}",
                    rollover.sandbox_id,
                    rollover.previous_lease_id,
                    rollover.lease_id,
                    rollover.processing_epoch,
                    rollover.first_block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_invalidation = self
            .last_invalidation_event()
            .map(|invalidation| {
                format!(
                    "{}:{}:{:?}@{}/block={:?}",
                    invalidation.sandbox_id,
                    invalidation.lease_id,
                    invalidation.stage,
                    invalidation.processing_epoch,
                    invalidation.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_completion_slot = self
            .last_completion_slot_event()
            .map(|completion| {
                format!(
                    "{}:{}:{:?}@{}/block={}",
                    completion.sandbox_id,
                    completion.lease_id,
                    completion.stage,
                    completion.processing_epoch,
                    completion.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport_fault = self
            .last_transport_fault_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:{:?}:{:?}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.source,
                    failure.stage,
                    failure.phase,
                    failure.resource,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_broker_failure = self
            .last_broker_failure_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_sandbox_operation_failure = self
            .last_sandbox_operation_failure_event()
            .map(|failure| {
                format!(
                    "{}:{}:{:?}:lease={:?}@{:?}",
                    failure.sandbox_id,
                    failure.operation,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());

        format!(
            "events={} supervision_updates={} plugin_faults={} plugin_instance_states={} recovery_events={} lifecycle_events={} transport_events={} heartbeat_events={} block_dispatch_events={} lease_rollover_events={} invalidation_events={} completion_slot_events={} transport_fault_events={} broker_failure_events={} sandbox_operation_failure_events={} last_watchdog={} last_fault={} last_plugin_instance_state={} last_recovery={} last_lifecycle={} last_transport={} last_heartbeat={} last_dispatch={} last_rollover={} last_invalidation={} last_completion_slot={} last_transport_fault={} last_broker_failure={} last_sandbox_operation_failure={}",
            self.total_events,
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.plugin_instance_state_event_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            last_trigger,
            last_fault,
            last_plugin_instance_state,
            last_recovery,
            last_lifecycle,
            last_transport,
            last_heartbeat,
            last_dispatch,
            last_rollover,
            last_invalidation,
            last_completion_slot,
            last_transport_fault,
            last_broker_failure,
            last_sandbox_operation_failure,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObservationReport {
    pub readiness: RuntimeReadiness,
    pub effective_config: EffectiveRuntimeConfig,
    pub control_snapshot: RuntimeControlSnapshot,
    pub scheduler_snapshot: RuntimeSchedulerSnapshot,
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    pub automation_snapshot: RuntimeAutomationSnapshot,
    pub engine_block_snapshot: RuntimeEngineBlockSnapshot,
    pub transport_concurrency_snapshot: RuntimeTransportConcurrencySnapshot,
    pub scheduler_summary: RuntimeSchedulerExportSummary,
    pub block_summary: RuntimeBlockExecutionSummary,
    pub degradation_summary: RuntimeDegradationSummary,
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    pub transport_fault_summary: TransportFaultSummary,
    pub transport_session_summary: TransportSessionSummary,
    pub observation: RuntimeObservationDiagnostics,
}

impl RuntimeObservationReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        let observation = recorder.diagnostics();
        let readiness = runtime.get_readiness();
        let effective_config = runtime.get_effective_config();
        let control_snapshot = runtime.get_control_snapshot();
        let scheduler_snapshot = runtime.get_scheduler_snapshot();
        let diagnostics_snapshot = runtime.get_diagnostics_snapshot();
        let supervision_snapshot = runtime.get_supervision_snapshot();
        let timeline_snapshot = runtime.get_timeline_snapshot();
        let automation_snapshot = runtime.get_automation_snapshot();
        let engine_block_snapshot = runtime.get_engine_block_snapshot();
        let transport_concurrency_snapshot = runtime.get_transport_concurrency_snapshot();
        Self {
            readiness: readiness.clone(),
            effective_config,
            control_snapshot,
            scheduler_snapshot,
            diagnostics_snapshot,
            supervision_snapshot: supervision_snapshot.clone(),
            timeline_snapshot,
            automation_snapshot,
            scheduler_summary: RuntimeSchedulerExportSummary::from_snapshot(&engine_block_snapshot),
            block_summary: RuntimeBlockExecutionSummary::from_snapshot(&engine_block_snapshot),
            degradation_summary: RuntimeDegradationSummary::capture(
                &readiness,
                diagnostics_snapshot,
                &supervision_snapshot,
                &engine_block_snapshot,
                &transport_concurrency_snapshot,
                &observation,
            ),
            execution_topology_summary: RuntimeExecutionTopologySummary::from_snapshot(
                &engine_block_snapshot,
            ),
            engine_block_snapshot,
            transport_concurrency_snapshot,
            transport_fault_summary: TransportFaultSummary::from_records(
                &observation.transport_fault_events,
            ),
            transport_session_summary: TransportSessionSummary::from_diagnostics(&observation),
            observation,
        }
    }

    pub fn render_compact(&self) -> String {
        let automation = (self.automation_snapshot.parameter_id != 0)
            .then(|| {
                let snapshot = &self.automation_snapshot;
                format!(
                    " automation_param={} automation_segments={} automation_first_epoch={:?} automation_last_epoch={:?} automation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.segment_count,
                    snapshot.first_epoch,
                    snapshot.last_epoch,
                    snapshot.lease_rollovers
                )
            })
            .unwrap_or_default();
        let transport_timeline = format!(
            " transport_epoch={} transport_transition={:?} transport_transition_epoch={:?} transport_transition_block={:?} transport_playing={:?} transport_tempo_bpm={:?} transport_timeline_position_samples={:?} transport_loop_start_samples={:?} transport_loop_end_samples={:?} transport_last_block_start_samples={:?} transport_last_block_end_samples={:?} transport_loop_wrap_count={}",
            self.timeline_snapshot.transport_epoch,
            self.timeline_snapshot.last_transport_transition,
            self.timeline_snapshot.last_transport_transition_processing_epoch,
            self.timeline_snapshot.last_transport_transition_block_sequence,
            self.timeline_snapshot.last_transport_playing,
            self.timeline_snapshot.last_transport_tempo_bpm,
            self.timeline_snapshot.last_transport_timeline_position_samples,
            self.timeline_snapshot.last_transport_loop_start_samples,
            self.timeline_snapshot.last_transport_loop_end_samples,
            self.timeline_snapshot.last_engine_block_start_samples,
            self.timeline_snapshot.last_engine_block_end_samples,
            self.timeline_snapshot.loop_wrap_count,
        );
        let engine_transport = format!(
            " engine_transport_epoch={} engine_transport_transition={:?} engine_transport_block_start={:?} engine_transport_block_end={:?} engine_transport_loop_wrapped={}",
            self.engine_block_snapshot.transport_epoch,
            self.engine_block_snapshot.transport_transition,
            self.engine_block_snapshot.transport_block_start_samples,
            self.engine_block_snapshot.transport_block_end_samples,
            self.engine_block_snapshot.transport_loop_wrapped,
        );
        let scheduler_topology =
            format_scheduler_topology_compact(&self.engine_block_snapshot.scheduler_topology);
        let scheduler_snapshot =
            format_runtime_scheduler_snapshot_compact(&self.scheduler_snapshot);
        let scheduler_summary = format_runtime_scheduler_summary_compact(&self.scheduler_summary);
        let block_summary = format_runtime_block_summary_compact(&self.block_summary);
        let degradation_summary =
            format_runtime_degradation_summary_compact(&self.degradation_summary);
        let execution_topology_summary =
            format_runtime_execution_topology_summary_compact(&self.execution_topology_summary);
        let compact = format!(
            "readiness={:?} sample_rate={} block_size={} handshaken={} configured={} running={} handshakes={} configures={} starts={} stops={} restarts={} xruns={} active_sandboxes={} safe_mode={} next_block_sequence={} sequence_segments={} sequence_first_block={:?} sequence_last_block={:?}{}{}{}{}{}{} engine_graph_id={:?} engine_node_count={} engine_stateful_nodes={} engine_latency_nodes={} engine_plugin_backed_nodes={} engine_planning_anticipative={} engine_inline_realtime_nodes={} engine_stateful_realtime_nodes={} engine_anticipative_eligible_nodes={} engine_phase_count={} engine_anticipative_phases={} engine_phase_order={:?} engine_lane_count={} engine_anticipative_lanes={} engine_lane_order={:?} engine_dispatch_count={} engine_dispatch_boundaries={} engine_dispatch_order={:?} engine_prepared_dispatches={} engine_realtime_dispatches={} engine_dispatch_handoffs={}{} engine_prework_cache_enabled={} engine_prework_cache_state={:?} engine_prework_service_state={:?} engine_prework_service_pressure={:?} engine_prework_service_semantic_policy={:?} engine_prework_service_active_plugin_sandboxes={} engine_prework_service_bound_plugin_sandboxes={} engine_prework_service_active_bound_plugin_sandboxes={} engine_prework_service_degraded_bound_plugin_sandboxes={} engine_prework_service_missing_bound_plugin_sandboxes={} engine_prework_service_plugin_gate_active={} engine_prework_pending_targets={} engine_prework_pending_immediate_targets={} engine_prework_pending_near_term_targets={} engine_prework_pending_deferred_targets={} engine_prework_next_pending_target_block={:?} engine_prework_service_cycles={} engine_prework_service_prepared_targets={} engine_prework_service_pauses={} engine_prework_service_resumes={} engine_prework_service_starvations={} engine_prework_service_throttles={} engine_prework_service_yields={} engine_last_prework_service_epoch={:?} engine_last_prework_serviced_target_block={:?} engine_last_prework_serviced_backlog_class={:?} engine_prework_requested_mode={:?} engine_prework_mode={:?} engine_prework_policy_configured={} engine_prework_profile={:?} engine_prework_profile_source={:?} engine_prework_profile_window_override={:?} engine_prework_policy_window_blocks={:?} engine_prework_queue_capacity={} engine_prework_queue_depth={} engine_prework_peak_queue_depth={} engine_prework_window_targets={} engine_prework_window_blocks={:?} engine_prework_freshness_state={:?} engine_prework_block_window={} engine_prework_remaining_valid_blocks={:?} engine_prework_cache_admissions={} engine_prework_cache_consumptions={} engine_prework_queued_admissions={} engine_prework_queued_consumptions={} engine_prework_cache_hits={} engine_prework_cache_misses={} engine_prework_cache_invalidations={} engine_prework_cache_retirements={} engine_prework_unconsumed_retirements={} engine_prework_consumed_retirements={} engine_last_prework_cache_hit={} engine_last_prework_invalidation={:?} engine_last_prework_retirement={:?} engine_last_prework_retired_unconsumed={:?} engine_prework_cache_valid_until={:?} engine_prework_cache_valid_until_block={:?} engine_last_prework_source_epoch={:?} engine_last_prework_source_block={:?} engine_last_prework_admission_epoch={:?} engine_last_prework_admission_block={:?} engine_last_prework_admitted_from_block={:?} engine_last_prework_consumption_epoch={:?} engine_last_prework_consumption_block={:?} engine_last_prework_consumed_from_block={:?} engine_last_prework_retirement_epoch={:?} engine_last_prework_retirement_block={:?} engine_stage_count={} engine_dynamic_kernel_stages={} engine_dynamic_stage_state_model={:?} engine_total_latency_samples={} engine_max_node_latency_samples={} engine_total_tail_samples={} engine_max_node_tail_samples={} engine_output_tail_samples={} engine_max_bus_tail_samples={} engine_processed_blocks={} engine_last_block={:?} engine_prework_output_peak={:?} engine_realtime_input_peak={:?} engine_output_peak={:?} engine_output_rms={:?} engine_projection_epoch={:?} engine_parameter_epoch={:?} engine_context_anticipative={:?} engine_transport_playing={:?} engine_transport_tempo={:?} engine_timeline_position={:?}{} transport_concurrency_limits={}/{} transport_concurrency_current={} transport_concurrency_peak={} transport_concurrency_recovery_current={} transport_concurrency_recovery_peak={} transport_concurrency_cleanup_pending={} transport_concurrency_deferred_retries={} transport_concurrency_next_cleanup_epoch={} transport_concurrency_oldest_ready_epoch={:?} transport_fault_boundary={:?} transport_fault_sources={}/{}/{} transport_fault_phases={}/{}/{}/{} transport_session_boundary={:?} transport_session_state={:?} transport_session_attached={} transport_session_heartbeat_state={:?} transport_session_dispatch_state={:?} transport_session_attached_sessions={} transport_session_max_attached_sessions={} transport_session_attach={} transport_session_detach={}/{}/{} transport_session_heartbeat={}/{}/{} transport_session_dispatch={}/{}/{} {}",
            self.readiness,
            self.effective_config.sample_rate.0,
            self.effective_config.block_size,
            self.control_snapshot.handshaken,
            self.control_snapshot.configured,
            self.control_snapshot.running,
            self.control_snapshot.handshake_count,
            self.control_snapshot.configure_count,
            self.control_snapshot.start_count,
            self.control_snapshot.stop_count,
            self.control_snapshot.restart_count,
            self.diagnostics_snapshot.xruns,
            self.diagnostics_snapshot.active_plugin_sandboxes,
            self.supervision_snapshot.safe_mode_enabled,
            self.timeline_snapshot.next_block_sequence,
            self.timeline_snapshot.block_sequence_continuity.segment_count(),
            self.timeline_snapshot
                .block_sequence_continuity
                .first_block_sequence(),
            self.timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence(),
            automation,
            transport_timeline,
            scheduler_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            self.engine_block_snapshot.graph_id,
            self.engine_block_snapshot.node_count,
            self.engine_block_snapshot.stateful_node_count,
            self.engine_block_snapshot.latency_node_count,
            self.engine_block_snapshot.plugin_backed_node_count,
            self.engine_block_snapshot.anticipative_planning_enabled,
            self.engine_block_snapshot.inline_realtime_node_count,
            self.engine_block_snapshot.stateful_realtime_node_count,
            self.engine_block_snapshot.anticipative_eligible_node_count,
            self.engine_block_snapshot.phase_count,
            self.engine_block_snapshot.anticipative_phase_count,
            self.engine_block_snapshot.phase_order,
            self.engine_block_snapshot.lane_count,
            self.engine_block_snapshot.anticipative_lane_count,
            self.engine_block_snapshot.lane_order,
            self.engine_block_snapshot.dispatch_count,
            self.engine_block_snapshot.dispatch_boundary_count,
            self.engine_block_snapshot.dispatch_order,
            self.engine_block_snapshot.prepared_dispatch_count,
            self.engine_block_snapshot.realtime_dispatch_count,
            self.engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology,
            self.engine_block_snapshot.prework_cache_enabled,
            self.engine_block_snapshot.prework_cache_state,
            self.engine_block_snapshot.prework_service_state,
            self.engine_block_snapshot.prework_service_pressure,
            self.engine_block_snapshot.prework_service_semantic_policy,
            self.engine_block_snapshot.prework_service_active_plugin_sandboxes,
            self.engine_block_snapshot.prework_service_bound_plugin_sandboxes,
            self.engine_block_snapshot
                .prework_service_active_bound_plugin_sandboxes,
            self.engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            self.engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            self.engine_block_snapshot.prework_service_plugin_gate_active,
            self.engine_block_snapshot.prework_pending_target_count,
            self.engine_block_snapshot
                .prework_pending_immediate_target_count,
            self.engine_block_snapshot
                .prework_pending_near_term_target_count,
            self.engine_block_snapshot
                .prework_pending_deferred_target_count,
            self.engine_block_snapshot
                .prework_next_pending_target_block_sequence,
            self.engine_block_snapshot.prework_service_cycle_count,
            self.engine_block_snapshot.prework_service_prepared_targets,
            self.engine_block_snapshot.prework_service_pause_count,
            self.engine_block_snapshot.prework_service_resume_count,
            self.engine_block_snapshot.prework_service_starvation_count,
            self.engine_block_snapshot.prework_service_throttle_count,
            self.engine_block_snapshot.prework_service_yield_count,
            self.engine_block_snapshot.last_prework_service_processing_epoch,
            self.engine_block_snapshot
                .last_prework_serviced_target_block_sequence,
            self.engine_block_snapshot.last_prework_serviced_backlog_class,
            self.engine_block_snapshot.prework_forecast_requested_mode,
            self.engine_block_snapshot.prework_forecast_mode,
            self.engine_block_snapshot.prework_forecast_policy_configured,
            self.engine_block_snapshot.prework_forecast_profile,
            self.engine_block_snapshot.prework_forecast_profile_source,
            self.engine_block_snapshot
                .prework_forecast_profile_target_window_override,
            self.engine_block_snapshot
                .prework_forecast_policy_target_window_blocks,
            self.engine_block_snapshot.prework_cache_queue_capacity,
            self.engine_block_snapshot.prework_cache_queue_depth,
            self.engine_block_snapshot.prework_cache_peak_queue_depth,
            self.engine_block_snapshot.prework_cache_window_target_count,
            self.engine_block_snapshot.prework_cache_window_target_block_sequences,
            self.engine_block_snapshot.prework_cache_freshness_state,
            self.engine_block_snapshot.prework_cache_block_freshness_window,
            self.engine_block_snapshot.prework_cache_remaining_valid_blocks,
            self.engine_block_snapshot.prework_cache_admissions,
            self.engine_block_snapshot.prework_cache_consumptions,
            self.engine_block_snapshot.prework_cache_queued_admissions,
            self.engine_block_snapshot.prework_cache_queued_consumptions,
            self.engine_block_snapshot.prework_cache_hits,
            self.engine_block_snapshot.prework_cache_misses,
            self.engine_block_snapshot.prework_cache_invalidation_count,
            self.engine_block_snapshot.prework_cache_retirement_count,
            self.engine_block_snapshot.prework_cache_unconsumed_retirement_count,
            self.engine_block_snapshot.prework_cache_consumed_retirement_count,
            self.engine_block_snapshot.last_prework_cache_hit,
            self.engine_block_snapshot.last_prework_invalidation_reason,
            self.engine_block_snapshot.last_prework_retirement_reason,
            self.engine_block_snapshot.last_prework_retired_unconsumed,
            self.engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            self.engine_block_snapshot.prework_cache_valid_until_block_sequence,
            self.engine_block_snapshot
                .last_prework_source_processing_epoch,
            self.engine_block_snapshot.last_prework_source_block_sequence,
            self.engine_block_snapshot
                .last_prework_admission_processing_epoch,
            self.engine_block_snapshot
                .last_prework_admission_block_sequence,
            self.engine_block_snapshot
                .last_prework_admitted_from_block_sequence,
            self.engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            self.engine_block_snapshot
                .last_prework_consumption_block_sequence,
            self.engine_block_snapshot
                .last_prework_consumed_from_block_sequence,
            self.engine_block_snapshot
                .last_prework_retirement_processing_epoch,
            self.engine_block_snapshot
                .last_prework_retirement_block_sequence,
            self.engine_block_snapshot.stage_count,
            self.engine_block_snapshot.dynamic_kernel_stage_count,
            self.engine_block_snapshot.dynamic_stage_state_model,
            self.engine_block_snapshot.total_latency_samples,
            self.engine_block_snapshot.max_node_latency_samples,
            self.engine_block_snapshot.total_tail_samples,
            self.engine_block_snapshot.max_node_tail_samples,
            self.engine_block_snapshot.output_tail_samples,
            self.engine_block_snapshot.max_bus_tail_samples,
            self.engine_block_snapshot.processed_blocks,
            self.engine_block_snapshot.last_block_sequence,
            self.engine_block_snapshot.last_prework_output_peak,
            self.engine_block_snapshot.last_realtime_input_peak,
            self.engine_block_snapshot.last_output_peak,
            self.engine_block_snapshot.last_output_rms,
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            self.engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            engine_transport,
            self.transport_concurrency_snapshot.steady_session_limit,
            self.transport_concurrency_snapshot.recovery_session_limit,
            self.transport_concurrency_snapshot.current_attached_sessions,
            self.transport_concurrency_snapshot.peak_attached_sessions,
            self.transport_concurrency_snapshot
                .current_recovery_overlap_sessions,
            self.transport_concurrency_snapshot
                .peak_recovery_overlap_sessions,
            self.transport_concurrency_snapshot.pending_cleanup_work_items,
            self.transport_concurrency_snapshot
                .pending_deferred_retry_work_items,
            self.transport_concurrency_snapshot.next_cleanup_epoch,
            self.transport_concurrency_snapshot
                .oldest_pending_cleanup_ready_epoch,
            self.transport_fault_summary.boundary_mode,
            self.transport_fault_summary.host_broker_events,
            self.transport_fault_summary.sandbox_operation_events,
            self.transport_fault_summary.runtime_dispatch_events,
            self.transport_fault_summary.prepare_events,
            self.transport_fault_summary.dispatch_events,
            self.transport_fault_summary.teardown_events,
            self.transport_fault_summary.control_events,
            self.transport_session_summary.boundary_mode,
            self.transport_session_summary.current_state,
            self.transport_session_summary.currently_attached,
            self.transport_session_summary.heartbeat_freshness,
            self.transport_session_summary.dispatch_state,
            self.transport_session_summary.current_attached_session_count,
            self.transport_session_summary.max_concurrent_attached_sessions,
            self.transport_session_summary.attach_events,
            self.transport_session_summary.detach_requested_events,
            self.transport_session_summary.detached_events,
            self.transport_session_summary.detach_fault_events,
            self.transport_session_summary.heartbeat_requested_events,
            self.transport_session_summary.heartbeat_responded_events,
            self.transport_session_summary.heartbeat_missed_events,
            self.transport_session_summary.dispatch_requested_events,
            self.transport_session_summary.dispatch_completed_events,
            self.transport_session_summary.dispatch_timed_out_events,
            format!(
                "{} transport_concurrency_cleanup_waves={}",
                self.observation.render_compact(),
                self.transport_concurrency_snapshot.pending_cleanup_waves.len()
            )
        );
        format!("{compact}{execution_topology_summary}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSupervisorReport {
    pub observation: RuntimeObservationReport,
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeSupervisorReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        Self {
            observation: RuntimeObservationReport::capture(runtime, recorder),
            events: recorder.snapshot(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn supervision_update_count(&self) -> usize {
        self.observation.observation.supervision_update_count()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.observation.observation.plugin_fault_count()
    }

    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.observation
            .observation
            .plugin_instance_state_event_count()
    }

    pub fn recovery_event_count(&self) -> usize {
        self.observation.observation.recovery_event_count()
    }

    pub fn lifecycle_event_count(&self) -> usize {
        self.observation.observation.lifecycle_event_count()
    }

    pub fn transport_event_count(&self) -> usize {
        self.observation.observation.transport_event_count()
    }

    pub fn heartbeat_event_count(&self) -> usize {
        self.observation.observation.heartbeat_event_count()
    }

    pub fn block_dispatch_event_count(&self) -> usize {
        self.observation.observation.block_dispatch_event_count()
    }

    pub fn lease_rollover_event_count(&self) -> usize {
        self.observation.observation.lease_rollover_event_count()
    }

    pub fn invalidation_event_count(&self) -> usize {
        self.observation.observation.invalidation_event_count()
    }

    pub fn completion_slot_event_count(&self) -> usize {
        self.observation.observation.completion_slot_event_count()
    }

    pub fn transport_fault_event_count(&self) -> usize {
        self.observation.observation.transport_fault_event_count()
    }

    pub fn broker_failure_event_count(&self) -> usize {
        self.observation.observation.broker_failure_event_count()
    }

    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.observation
            .observation
            .sandbox_operation_failure_event_count()
    }

    pub fn last_watchdog_trigger(&self) -> Option<RuntimeWatchdogTrigger> {
        self.observation
            .observation
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} event_stream={}",
            self.observation.render_compact(),
            self.event_count()
        )
    }

    pub fn render_multiline(&self) -> String {
        let automation = (self.observation.automation_snapshot.parameter_id != 0)
            .then(|| {
                let snapshot = &self.observation.automation_snapshot;
                format!(
                    "\nautomation_param={}\nautomation_value_events={}\nautomation_modulation_events={}\nautomation_gesture_begin_events={}\nautomation_gesture_end_events={}\nautomation_first_value={:?}\nautomation_last_value={:?}\nautomation_last_modulation={:?}\nautomation_first_epoch={:?}\nautomation_last_epoch={:?}\nautomation_segments={}\nautomation_segment_epochs={:?}\nautomation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.value_events,
                    snapshot.modulation_events,
                    snapshot.gesture_begin_events,
                    snapshot.gesture_end_events,
                    snapshot.first_value,
                    snapshot.last_value,
                    snapshot.last_modulation,
                    snapshot.first_epoch,
                    snapshot.last_epoch,
                    snapshot.segment_count,
                    snapshot.segment_epochs,
                    snapshot.lease_rollovers,
                )
            })
            .unwrap_or_default();
        let transport_timeline = format!(
            "\ntransport_epoch={}\ntransport_transition={:?}\ntransport_transition_epoch={:?}\ntransport_transition_block={:?}\ntransport_playing={:?}\ntransport_tempo_bpm={:?}\ntransport_timeline_position_samples={:?}\ntransport_loop_start_samples={:?}\ntransport_loop_end_samples={:?}\ntransport_last_block_start_samples={:?}\ntransport_last_block_end_samples={:?}\ntransport_loop_wrap_count={}",
            self.observation.timeline_snapshot.transport_epoch,
            self.observation.timeline_snapshot.last_transport_transition,
            self.observation
                .timeline_snapshot
                .last_transport_transition_processing_epoch,
            self.observation
                .timeline_snapshot
                .last_transport_transition_block_sequence,
            self.observation.timeline_snapshot.last_transport_playing,
            self.observation.timeline_snapshot.last_transport_tempo_bpm,
            self.observation
                .timeline_snapshot
                .last_transport_timeline_position_samples,
            self.observation.timeline_snapshot.last_transport_loop_start_samples,
            self.observation.timeline_snapshot.last_transport_loop_end_samples,
            self.observation.timeline_snapshot.last_engine_block_start_samples,
            self.observation.timeline_snapshot.last_engine_block_end_samples,
            self.observation.timeline_snapshot.loop_wrap_count,
        );
        let engine_transport = format!(
            "\nengine_transport_epoch={}\nengine_transport_transition={:?}\nengine_transport_block_start_samples={:?}\nengine_transport_block_end_samples={:?}\nengine_transport_loop_wrapped={}",
            self.observation.engine_block_snapshot.transport_epoch,
            self.observation.engine_block_snapshot.transport_transition,
            self.observation
                .engine_block_snapshot
                .transport_block_start_samples,
            self.observation.engine_block_snapshot.transport_block_end_samples,
            self.observation.engine_block_snapshot.transport_loop_wrapped,
        );
        let scheduler_topology = format_scheduler_topology_multiline(
            &self.observation.engine_block_snapshot.scheduler_topology,
        );
        let scheduler_snapshot =
            format_runtime_scheduler_snapshot_multiline(&self.observation.scheduler_snapshot);
        let scheduler_summary =
            format_runtime_scheduler_summary_multiline(&self.observation.scheduler_summary);
        let block_summary = format_runtime_block_summary_multiline(&self.observation.block_summary);
        let degradation_summary =
            format_runtime_degradation_summary_multiline(&self.observation.degradation_summary);
        let execution_topology_summary = format_runtime_execution_topology_summary_multiline(
            &self.observation.execution_topology_summary,
        );
        let multiline = format!(
            "readiness={:?}\nsample_rate={}\nblock_size={}\nhandshaken={}\nconfigured={}\nrunning={}\nhandshake_count={}\nconfigure_count={}\nstart_count={}\nstop_count={}\nrestart_count={:?}\nlast_client_version={:?}\nlast_stop_reason={:?}\nlast_reconfigure={:?}\nxruns={}\nactive_sandboxes={}\nsafe_mode={}\nnext_block_sequence={}\nsequence_segments={}\nsequence_segment_epochs={:?}\nsequence_first_block={:?}\nsequence_last_block={:?}\nsequence_gaps={}\nsequence_lease_rollovers={}{}{}{}{}{}{}\nengine_graph_id={:?}\nengine_node_count={}\nengine_stateful_nodes={}\nengine_latency_nodes={}\nengine_plugin_backed_nodes={}\nengine_planning_anticipative={}\nengine_inline_realtime_nodes={}\nengine_stateful_realtime_nodes={}\nengine_anticipative_eligible_nodes={}\nengine_phase_count={}\nengine_anticipative_phases={}\nengine_phase_order={:?}\nengine_lane_count={}\nengine_anticipative_lanes={}\nengine_lane_order={:?}\nengine_dispatch_count={}\nengine_dispatch_boundaries={}\nengine_dispatch_order={:?}\nengine_prepared_dispatches={}\nengine_realtime_dispatches={}\nengine_dispatch_handoffs={}{}\nengine_prework_cache_enabled={}\nengine_prework_cache_state={:?}\nengine_prework_service_state={:?}\nengine_prework_service_pressure={:?}\nengine_prework_service_semantic_policy={:?}\nengine_prework_service_active_plugin_sandboxes={}\nengine_prework_service_bound_plugin_sandboxes={}\nengine_prework_service_active_bound_plugin_sandboxes={}\nengine_prework_service_degraded_bound_plugin_sandboxes={}\nengine_prework_service_missing_bound_plugin_sandboxes={}\nengine_prework_service_plugin_gate_active={}\nengine_prework_pending_targets={}\nengine_prework_pending_immediate_targets={}\nengine_prework_pending_near_term_targets={}\nengine_prework_pending_deferred_targets={}\nengine_prework_next_pending_target_block={:?}\nengine_prework_service_cycles={}\nengine_prework_service_prepared_targets={}\nengine_prework_service_pauses={}\nengine_prework_service_resumes={}\nengine_prework_service_starvations={}\nengine_prework_service_throttles={}\nengine_prework_service_yields={}\nengine_last_prework_service_epoch={:?}\nengine_last_prework_service_requested_cycles={}\nengine_last_prework_service_effective_cycles={}\nengine_last_prework_service_cycle_count={}\nengine_last_prework_service_budget={:?}\nengine_last_prework_service_effective_budget={:?}\nengine_last_prework_service_prepared_targets={}\nengine_last_prework_serviced_target_block={:?}\nengine_last_prework_serviced_backlog_class={:?}\nengine_prework_requested_mode={:?}\nengine_prework_mode={:?}\nengine_prework_policy_configured={}\nengine_prework_profile={:?}\nengine_prework_profile_source={:?}\nengine_prework_profile_window_override={:?}\nengine_prework_policy_window_blocks={:?}\nengine_prework_queue_capacity={}\nengine_prework_queue_depth={}\nengine_prework_peak_queue_depth={}\nengine_prework_window_targets={}\nengine_prework_window_blocks={:?}\nengine_prework_freshness_state={:?}\nengine_prework_block_window={}\nengine_prework_remaining_valid_blocks={:?}\nengine_prework_cache_admissions={}\nengine_prework_cache_consumptions={}\nengine_prework_queued_admissions={}\nengine_prework_queued_consumptions={}\nengine_prework_cache_hits={}\nengine_prework_cache_misses={}\nengine_prework_cache_invalidations={}\nengine_last_prework_cache_hit={}\nengine_last_prework_invalidation={:?}\nengine_prework_cache_valid_until={:?}\nengine_prework_cache_valid_until_block={:?}\nengine_last_prework_source_epoch={:?}\nengine_last_prework_source_block={:?}\nengine_last_prework_admission_epoch={:?}\nengine_last_prework_admission_block={:?}\nengine_last_prework_admitted_from_block={:?}\nengine_last_prework_consumption_epoch={:?}\nengine_last_prework_consumption_block={:?}\nengine_last_prework_consumed_from_block={:?}\nengine_planned_nodes={:?}\nengine_stage_count={}\nengine_dynamic_kernel_stages={}\nengine_dynamic_stage_state_model={:?}\nengine_total_latency_samples={}\nengine_max_node_latency_samples={}\nengine_total_tail_samples={}\nengine_max_node_tail_samples={}\nengine_output_tail_samples={}\nengine_max_bus_tail_samples={}\nengine_processed_blocks={}\nengine_last_processing_epoch={:?}\nengine_last_block_sequence={:?}\nengine_last_frame_count={}\nengine_last_channel_count={}\nengine_last_input_peak={:?}\nengine_last_prework_output_peak={:?}\nengine_last_realtime_input_peak={:?}\nengine_last_output_peak={:?}\nengine_last_output_rms={:?}\nengine_last_first_output_sample={:?}\nengine_projection_epoch={:?}\nengine_parameter_epoch={:?}\nengine_context_anticipative={:?}\nengine_transport_playing={:?}\nengine_transport_tempo_bpm={:?}\nengine_timeline_position_samples={:?}{}{}\ntransport_concurrency_steady_limit={}\ntransport_concurrency_recovery_limit={}\ntransport_concurrency_current_attached={}\ntransport_concurrency_peak_attached={}\ntransport_concurrency_current_recovery_overlap={}\ntransport_concurrency_peak_recovery_overlap={}\ntransport_concurrency_current_lingering={}\ntransport_concurrency_peak_lingering={}\ntransport_concurrency_current_detach_requested={}\ntransport_concurrency_current_detach_faulted={}\ntransport_concurrency_active_sessions={:?}\ntransport_concurrency_pending_cleanup_waves={:?}\ntransport_concurrency_last_admitted_sandbox_id={:?}\ntransport_concurrency_last_rejected_sandbox_id={:?}\ntransport_concurrency_last_rejection_reason={:?}\ntransport_fault_boundary={:?}\ntransport_fault_host_broker_events={}\ntransport_fault_sandbox_operation_events={}\ntransport_fault_runtime_dispatch_events={}\ntransport_fault_prepare_events={}\ntransport_fault_dispatch_events={}\ntransport_fault_teardown_events={}\ntransport_fault_control_events={}\ntransport_fault_first_epoch={:?}\ntransport_fault_last_epoch={:?}\ntransport_fault_first_block={:?}\ntransport_fault_last_block={:?}\ntransport_session_boundary={:?}\ntransport_session_state={:?}\ntransport_session_currently_attached={}\ntransport_session_heartbeat_state={:?}\ntransport_session_dispatch_state={:?}\ntransport_session_current_attached_sessions={}\ntransport_session_max_attached_sessions={}\ntransport_session_attach_events={}\ntransport_session_detach_requested_events={}\ntransport_session_detached_events={}\ntransport_session_detach_fault_events={}\ntransport_session_heartbeat_requested_events={}\ntransport_session_heartbeat_responded_events={}\ntransport_session_heartbeat_missed_events={}\ntransport_session_dispatch_requested_events={}\ntransport_session_dispatch_completed_events={}\ntransport_session_dispatch_timed_out_events={}\ntransport_session_first_epoch={:?}\ntransport_session_last_epoch={:?}\ntransport_session_first_block={:?}\ntransport_session_last_block={:?}\ntransport_session_active_sandbox_id={:?}\ntransport_session_active_lease_id={:?}\ntransport_session_active_region_id={:?}\ntransport_session_active_block_sequence={:?}\ntransport_session_active_sessions={:?}\ntransport_session_last_sandbox_id={:?}\ntransport_session_last_lease_id={:?}\ntransport_session_last_region_id={:?}\nevent_stream={}\nsupervision_updates={}\nplugin_faults={}\nrecovery_events={}\nlifecycle_events={}\ntransport_events={}\nheartbeat_events={}\nblock_dispatch_events={}\nlease_rollover_events={}\ninvalidation_events={}\ncompletion_slot_events={}\ntransport_fault_events={}\nbroker_failure_events={}\nsandbox_operation_failure_events={}\nlast_watchdog={}\nlast_fault={}\nlast_recovery={:?}\nlast_lifecycle={:?}\nlast_transport={:?}\nlast_heartbeat={:?}\nlast_dispatch={:?}\nlast_rollover={:?}\nlast_invalidation={:?}\nlast_completion_slot={:?}\nlast_transport_fault={:?}\nlast_broker_failure={:?}\nlast_sandbox_operation_failure={:?}\nrecovery_sequence={:?}\nlifecycle_sequence={:?}\ntransport_sequence={:?}\nheartbeat_sequence={:?}\nblock_dispatch_sequence={:?}\nlease_rollover_sequence={:?}\ninvalidation_sequence={:?}\ncompletion_slot_sequence={:?}\ntransport_fault_sequence={:?}\nbroker_failure_sequence={:?}\nsandbox_operation_failure_sequence={:?}",
            self.observation.readiness,
            self.observation.effective_config.sample_rate.0,
            self.observation.effective_config.block_size,
            self.observation.control_snapshot.handshaken,
            self.observation.control_snapshot.configured,
            self.observation.control_snapshot.running,
            self.observation.control_snapshot.handshake_count,
            self.observation.control_snapshot.configure_count,
            self.observation.control_snapshot.start_count,
            self.observation.control_snapshot.stop_count,
            self.observation.control_snapshot.restart_count,
            self.observation.control_snapshot.last_client_version,
            self.observation.control_snapshot.last_stop_reason,
            self.observation.control_snapshot.last_reconfigure,
            self.observation.diagnostics_snapshot.xruns,
            self.observation.diagnostics_snapshot.active_plugin_sandboxes,
            self.observation.supervision_snapshot.safe_mode_enabled,
            self.observation.timeline_snapshot.next_block_sequence,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_count(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_epochs(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .first_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .sequence_gaps,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .lease_rollovers,
            automation,
            transport_timeline,
            scheduler_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            self.observation.engine_block_snapshot.graph_id,
            self.observation.engine_block_snapshot.node_count,
            self.observation.engine_block_snapshot.stateful_node_count,
            self.observation.engine_block_snapshot.latency_node_count,
            self.observation.engine_block_snapshot.plugin_backed_node_count,
            self.observation
                .engine_block_snapshot
                .anticipative_planning_enabled,
            self.observation.engine_block_snapshot.inline_realtime_node_count,
            self.observation.engine_block_snapshot.stateful_realtime_node_count,
            self.observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            self.observation.engine_block_snapshot.phase_count,
            self.observation
                .engine_block_snapshot
                .anticipative_phase_count,
            self.observation.engine_block_snapshot.phase_order,
            self.observation.engine_block_snapshot.lane_count,
            self.observation.engine_block_snapshot.anticipative_lane_count,
            self.observation.engine_block_snapshot.lane_order,
            self.observation.engine_block_snapshot.dispatch_count,
            self.observation
                .engine_block_snapshot
                .dispatch_boundary_count,
            self.observation.engine_block_snapshot.dispatch_order,
            self.observation.engine_block_snapshot.prepared_dispatch_count,
            self.observation.engine_block_snapshot.realtime_dispatch_count,
            self.observation.engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology,
            self.observation.engine_block_snapshot.prework_cache_enabled,
            self.observation.engine_block_snapshot.prework_cache_state,
            self.observation.engine_block_snapshot.prework_service_state,
            self.observation.engine_block_snapshot.prework_service_pressure,
            self.observation
                .engine_block_snapshot
                .prework_service_semantic_policy,
            self.observation
                .engine_block_snapshot
                .prework_service_active_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_active_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_plugin_gate_active,
            self.observation
                .engine_block_snapshot
                .prework_pending_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_immediate_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_near_term_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_deferred_target_count,
            self.observation
                .engine_block_snapshot
                .prework_next_pending_target_block_sequence,
            self.observation.engine_block_snapshot.prework_service_cycle_count,
            self.observation
                .engine_block_snapshot
                .prework_service_prepared_targets,
            self.observation.engine_block_snapshot.prework_service_pause_count,
            self.observation.engine_block_snapshot.prework_service_resume_count,
            self.observation
                .engine_block_snapshot
                .prework_service_starvation_count,
            self.observation
                .engine_block_snapshot
                .prework_service_throttle_count,
            self.observation.engine_block_snapshot.prework_service_yield_count,
            self.observation
                .engine_block_snapshot
                .last_prework_service_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_service_requested_cycles,
            self.observation
                .engine_block_snapshot
                .last_prework_service_effective_cycles,
            self.observation
                .engine_block_snapshot
                .last_prework_service_cycle_count,
            self.observation
                .engine_block_snapshot
                .last_prework_service_budget_per_cycle,
            self.observation
                .engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            self.observation
                .engine_block_snapshot
                .last_prework_service_prepared_targets,
            self.observation
                .engine_block_snapshot
                .last_prework_serviced_target_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_serviced_backlog_class,
            self.observation
                .engine_block_snapshot
                .prework_forecast_requested_mode,
            self.observation.engine_block_snapshot.prework_forecast_mode,
            self.observation
                .engine_block_snapshot
                .prework_forecast_policy_configured,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile_source,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile_target_window_override,
            self.observation
                .engine_block_snapshot
                .prework_forecast_policy_target_window_blocks,
            self.observation.engine_block_snapshot.prework_cache_queue_capacity,
            self.observation.engine_block_snapshot.prework_cache_queue_depth,
            self.observation.engine_block_snapshot.prework_cache_peak_queue_depth,
            self.observation.engine_block_snapshot.prework_cache_window_target_count,
            self.observation
                .engine_block_snapshot
                .prework_cache_window_target_block_sequences,
            self.observation.engine_block_snapshot.prework_cache_freshness_state,
            self.observation
                .engine_block_snapshot
                .prework_cache_block_freshness_window,
            self.observation
                .engine_block_snapshot
                .prework_cache_remaining_valid_blocks,
            self.observation.engine_block_snapshot.prework_cache_admissions,
            self.observation.engine_block_snapshot.prework_cache_consumptions,
            self.observation
                .engine_block_snapshot
                .prework_cache_queued_admissions,
            self.observation
                .engine_block_snapshot
                .prework_cache_queued_consumptions,
            self.observation.engine_block_snapshot.prework_cache_hits,
            self.observation.engine_block_snapshot.prework_cache_misses,
            self.observation
                .engine_block_snapshot
                .prework_cache_invalidation_count,
            self.observation.engine_block_snapshot.last_prework_cache_hit,
            self.observation
                .engine_block_snapshot
                .last_prework_invalidation_reason,
            self.observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            self.observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_source_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_source_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_admission_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_admission_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_admitted_from_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_consumption_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_consumed_from_block_sequence,
            self.observation.engine_block_snapshot.planned_nodes,
            self.observation.engine_block_snapshot.stage_count,
            self.observation.engine_block_snapshot.dynamic_kernel_stage_count,
            self.observation.engine_block_snapshot.dynamic_stage_state_model,
            self.observation.engine_block_snapshot.total_latency_samples,
            self.observation.engine_block_snapshot.max_node_latency_samples,
            self.observation.engine_block_snapshot.total_tail_samples,
            self.observation.engine_block_snapshot.max_node_tail_samples,
            self.observation.engine_block_snapshot.output_tail_samples,
            self.observation.engine_block_snapshot.max_bus_tail_samples,
            self.observation.engine_block_snapshot.processed_blocks,
            self.observation.engine_block_snapshot.last_processing_epoch,
            self.observation.engine_block_snapshot.last_block_sequence,
            self.observation.engine_block_snapshot.last_frame_count,
            self.observation.engine_block_snapshot.last_channel_count,
            self.observation.engine_block_snapshot.last_input_peak,
            self.observation
                .engine_block_snapshot
                .last_prework_output_peak,
            self.observation
                .engine_block_snapshot
                .last_realtime_input_peak,
            self.observation.engine_block_snapshot.last_output_peak,
            self.observation.engine_block_snapshot.last_output_rms,
            self.observation.engine_block_snapshot.last_first_output_sample,
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            engine_transport,
            scheduler_topology,
            self.observation
                .transport_concurrency_snapshot
                .steady_session_limit,
            self.observation
                .transport_concurrency_snapshot
                .recovery_session_limit,
            self.observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            self.observation.transport_concurrency_snapshot.peak_attached_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_recovery_overlap_sessions,
            self.observation
                .transport_concurrency_snapshot
                .peak_recovery_overlap_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            self.observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_detach_requested_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            self.observation.transport_concurrency_snapshot.active_sessions,
            self.observation
                .transport_concurrency_snapshot
                .pending_cleanup_waves,
            self.observation
                .transport_concurrency_snapshot
                .last_admitted_sandbox_id,
            self.observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id,
            self.observation
                .transport_concurrency_snapshot
                .last_rejection_reason,
            self.observation.transport_fault_summary.boundary_mode,
            self.observation.transport_fault_summary.host_broker_events,
            self.observation.transport_fault_summary.sandbox_operation_events,
            self.observation.transport_fault_summary.runtime_dispatch_events,
            self.observation.transport_fault_summary.prepare_events,
            self.observation.transport_fault_summary.dispatch_events,
            self.observation.transport_fault_summary.teardown_events,
            self.observation.transport_fault_summary.control_events,
            self.observation.transport_fault_summary.first_processing_epoch,
            self.observation.transport_fault_summary.last_processing_epoch,
            self.observation.transport_fault_summary.first_block_sequence,
            self.observation.transport_fault_summary.last_block_sequence,
            self.observation.transport_session_summary.boundary_mode,
            self.observation.transport_session_summary.current_state,
            self.observation.transport_session_summary.currently_attached,
            self.observation.transport_session_summary.heartbeat_freshness,
            self.observation.transport_session_summary.dispatch_state,
            self.observation.transport_session_summary.current_attached_session_count,
            self.observation.transport_session_summary.max_concurrent_attached_sessions,
            self.observation.transport_session_summary.attach_events,
            self.observation.transport_session_summary.detach_requested_events,
            self.observation.transport_session_summary.detached_events,
            self.observation.transport_session_summary.detach_fault_events,
            self.observation.transport_session_summary.heartbeat_requested_events,
            self.observation.transport_session_summary.heartbeat_responded_events,
            self.observation.transport_session_summary.heartbeat_missed_events,
            self.observation.transport_session_summary.dispatch_requested_events,
            self.observation.transport_session_summary.dispatch_completed_events,
            self.observation.transport_session_summary.dispatch_timed_out_events,
            self.observation.transport_session_summary.first_processing_epoch,
            self.observation.transport_session_summary.last_processing_epoch,
            self.observation.transport_session_summary.first_block_sequence,
            self.observation.transport_session_summary.last_block_sequence,
            self.observation.transport_session_summary.active_sandbox_id,
            self.observation.transport_session_summary.active_lease_id,
            self.observation.transport_session_summary.active_region_id,
            self.observation.transport_session_summary.active_block_sequence,
            self.observation.transport_session_summary.active_sessions,
            self.observation.transport_session_summary.last_sandbox_id,
            self.observation.transport_session_summary.last_lease_id,
            self.observation.transport_session_summary.last_region_id,
            self.event_count(),
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            self.last_watchdog_trigger()
                .map(|trigger| format!("{trigger:?}"))
                .unwrap_or_else(|| "none".into()),
            self.observation
                .observation
                .plugin_faults
                .last()
                .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
                .unwrap_or_else(|| "none".into()),
            self.observation.observation.last_recovery_event(),
            self.observation.observation.last_lifecycle_event(),
            self.observation.observation.last_transport_event(),
            self.observation.observation.last_heartbeat_event(),
            self.observation.observation.last_block_dispatch_event(),
            self.observation.observation.last_lease_rollover_event(),
            self.observation.observation.last_invalidation_event(),
            self.observation.observation.last_completion_slot_event(),
            self.observation.observation.last_transport_fault_event(),
            self.observation.observation.last_broker_failure_event(),
            self.observation.observation.last_sandbox_operation_failure_event(),
            self.observation.observation.recovery_events,
            self.observation.observation.lifecycle_events,
            self.observation.observation.transport_events,
            self.observation.observation.heartbeat_events,
            self.observation.observation.block_dispatch_events,
            self.observation.observation.lease_rollover_events,
            self.observation.observation.invalidation_events,
            self.observation.observation.completion_slot_events,
            self.observation.observation.transport_fault_events,
            self.observation.observation.broker_failure_events,
            self.observation.observation.sandbox_operation_failure_events,
        );
        format!("{multiline}{execution_topology_summary}")
    }

    pub fn render_json(&self) -> String {
        let timeline = &self.observation.timeline_snapshot.block_sequence_continuity;
        let last_fault = self.observation.observation.plugin_faults.last();
        let last_plugin_instance_state = self.observation.observation.last_plugin_instance_state();
        let automation = &self.observation.automation_snapshot;
        let automation = if automation.parameter_id == 0 {
            "null".into()
        } else {
            json_runtime_automation_snapshot(automation)
        };
        format!(
            concat!(
                "{{",
                "\"readiness\":{},",
                "\"sample_rate\":{},",
                "\"block_size\":{},",
                "\"control\":{},",
                "\"xruns\":{},",
                "\"active_sandboxes\":{},",
                "\"safe_mode\":{},",
                "\"next_block_sequence\":{},",
                "\"sequence_segments\":{},",
                "\"sequence_segment_epochs\":{},",
                "\"sequence_first_block\":{},",
                "\"sequence_last_block\":{},",
                "\"sequence_gaps\":{},",
                "\"sequence_lease_rollovers\":{},",
                "\"transport_epoch\":{},",
                "\"transport_transition\":{},",
                "\"transport_transition_epoch\":{},",
                "\"transport_transition_block\":{},",
                "\"transport_playing\":{},",
                "\"transport_tempo_bpm\":{},",
                "\"transport_timeline_position_samples\":{},",
                "\"transport_loop_start_samples\":{},",
                "\"transport_loop_end_samples\":{},",
                "\"transport_last_block_start_samples\":{},",
                "\"transport_last_block_end_samples\":{},",
                "\"transport_loop_wrap_count\":{},",
                "\"engine_block_snapshot\":{},",
                "\"scheduler_snapshot\":{},",
                "\"scheduler_summary\":{},",
                "\"block_summary\":{},",
                "\"degradation_summary\":{},",
                "\"execution_topology_summary\":{},",
                "\"transport_concurrency_snapshot\":{},",
                "\"transport_fault_summary\":{},",
                "\"transport_session_summary\":{},",
                "\"event_stream\":{},",
                "\"supervision_updates\":{},",
                "\"plugin_faults\":{},",
                "\"plugin_instance_state_events\":{},",
                "\"recovery_events\":{},",
                "\"lifecycle_events\":{},",
                "\"transport_events\":{},",
                "\"heartbeat_events\":{},",
                "\"block_dispatch_events\":{},",
                "\"lease_rollover_events\":{},",
                "\"invalidation_events\":{},",
                "\"completion_slot_events\":{},",
                "\"transport_fault_events\":{},",
                "\"broker_failure_events\":{},",
                "\"sandbox_operation_failure_events\":{},",
                "\"last_watchdog\":{},",
                "\"last_fault\":{},",
                "\"last_plugin_instance_state\":{},",
                "\"last_recovery\":{},",
                "\"last_lifecycle\":{},",
                "\"last_transport\":{},",
                "\"last_heartbeat\":{},",
                "\"last_dispatch\":{},",
                "\"last_rollover\":{},",
                "\"last_invalidation\":{},",
                "\"last_completion_slot\":{},",
                "\"last_transport_fault\":{},",
                "\"last_broker_failure\":{},",
                "\"last_sandbox_operation_failure\":{},",
                "\"recovery_sequence\":{},",
                "\"lifecycle_sequence\":{},",
                "\"transport_sequence\":{},",
                "\"heartbeat_sequence\":{},",
                "\"block_dispatch_sequence\":{},",
                "\"lease_rollover_sequence\":{},",
                "\"invalidation_sequence\":{},",
                "\"completion_slot_sequence\":{},",
                "\"transport_fault_sequence\":{},",
                "\"broker_failure_sequence\":{},",
                "\"sandbox_operation_failure_sequence\":{},",
                "\"plugin_instance_state_sequence\":{},",
                "\"automation\":{}",
                "}}"
            ),
            json_escape_string(&format!("{:?}", self.observation.readiness)),
            self.observation.effective_config.sample_rate.0,
            self.observation.effective_config.block_size,
            json_runtime_control_snapshot(&self.observation.control_snapshot),
            self.observation.diagnostics_snapshot.xruns,
            self.observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            self.observation.supervision_snapshot.safe_mode_enabled,
            self.observation.timeline_snapshot.next_block_sequence,
            timeline.segment_count(),
            json_u64_vec(&timeline.segment_epochs()),
            json_option_u64(timeline.first_block_sequence()),
            json_option_u64(timeline.last_block_sequence()),
            timeline.sequence_gaps,
            timeline.lease_rollovers,
            self.observation.timeline_snapshot.transport_epoch,
            json_option_string(
                self.observation
                    .timeline_snapshot
                    .last_transport_transition
                    .map(|transition| format!("{transition:?}"))
                    .as_deref(),
            ),
            json_option_u64(
                self.observation
                    .timeline_snapshot
                    .last_transport_transition_processing_epoch,
            ),
            json_option_u64(
                self.observation
                    .timeline_snapshot
                    .last_transport_transition_block_sequence,
            ),
            match self.observation.timeline_snapshot.last_transport_playing {
                Some(value) => value.to_string(),
                None => "null".into(),
            },
            json_option_f64(self.observation.timeline_snapshot.last_transport_tempo_bpm),
            json_option_i64(
                self.observation
                    .timeline_snapshot
                    .last_transport_timeline_position_samples,
            ),
            json_option_i64(
                self.observation
                    .timeline_snapshot
                    .last_transport_loop_start_samples,
            ),
            json_option_i64(
                self.observation
                    .timeline_snapshot
                    .last_transport_loop_end_samples,
            ),
            json_option_i64(
                self.observation
                    .timeline_snapshot
                    .last_engine_block_start_samples,
            ),
            json_option_i64(
                self.observation
                    .timeline_snapshot
                    .last_engine_block_end_samples,
            ),
            self.observation.timeline_snapshot.loop_wrap_count,
            json_runtime_engine_block_snapshot(&self.observation.engine_block_snapshot),
            json_runtime_scheduler_snapshot(&self.observation.scheduler_snapshot),
            json_runtime_scheduler_export_summary(&self.observation.scheduler_summary),
            json_runtime_block_execution_summary(&self.observation.block_summary),
            json_runtime_degradation_summary(&self.observation.degradation_summary),
            json_runtime_execution_topology_summary(&self.observation.execution_topology_summary,),
            json_runtime_transport_concurrency_snapshot(
                &self.observation.transport_concurrency_snapshot,
            ),
            json_transport_fault_summary(&self.observation.transport_fault_summary),
            json_transport_session_summary(&self.observation.transport_session_summary),
            self.event_count(),
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.plugin_instance_state_event_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            json_option_string(
                self.last_watchdog_trigger()
                    .map(|trigger| format!("{trigger:?}"))
                    .as_deref(),
            ),
            json_option_string(
                last_fault
                    .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
                    .as_deref(),
            ),
            json_plugin_instance_state_record(last_plugin_instance_state),
            json_recovery_record(self.observation.observation.last_recovery_event()),
            json_lifecycle_record(self.observation.observation.last_lifecycle_event()),
            json_transport_record(self.observation.observation.last_transport_event()),
            json_heartbeat_record(self.observation.observation.last_heartbeat_event()),
            json_block_dispatch_record(self.observation.observation.last_block_dispatch_event()),
            json_lease_rollover_record(self.observation.observation.last_lease_rollover_event()),
            json_broker_invalidation_record(self.observation.observation.last_invalidation_event()),
            json_completion_slot_record(self.observation.observation.last_completion_slot_event(),),
            json_transport_fault_record(self.observation.observation.last_transport_fault_event()),
            json_broker_failure_record(self.observation.observation.last_broker_failure_event()),
            json_sandbox_operation_failure_record(
                self.observation
                    .observation
                    .last_sandbox_operation_failure_event(),
            ),
            json_recovery_record_vec(&self.observation.observation.recovery_events),
            json_lifecycle_record_vec(&self.observation.observation.lifecycle_events),
            json_transport_record_vec(&self.observation.observation.transport_events),
            json_heartbeat_record_vec(&self.observation.observation.heartbeat_events),
            json_block_dispatch_record_vec(&self.observation.observation.block_dispatch_events),
            json_lease_rollover_record_vec(&self.observation.observation.lease_rollover_events),
            json_broker_invalidation_record_vec(&self.observation.observation.invalidation_events),
            json_completion_slot_record_vec(&self.observation.observation.completion_slot_events,),
            json_transport_fault_record_vec(&self.observation.observation.transport_fault_events),
            json_broker_failure_record_vec(&self.observation.observation.broker_failure_events),
            json_sandbox_operation_failure_record_vec(
                &self
                    .observation
                    .observation
                    .sandbox_operation_failure_events,
            ),
            json_plugin_instance_state_record_vec(
                &self.observation.observation.plugin_instance_states,
            ),
            automation,
        )
    }
}

fn json_escape_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_string(value: Option<&str>) -> String {
    match value {
        Some(value) => json_escape_string(value),
        None => "null".into(),
    }
}

fn json_option_u64(value: Option<u64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_option_u32(value: Option<u32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_option_usize(value: Option<usize>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_option_i64(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_option_f64(value: Option<f64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_u64_vec(values: &[u64]) -> String {
    let joined = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_option_f32(value: Option<f32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn format_scheduler_topology_compact(summary: &RuntimeSchedulerTopologySummary) -> String {
    format!(
        " engine_scheduler_topology_compatible={} engine_scheduler_topology_requires_host_reinterpretation={} engine_scheduler_topology_track_lanes={}/{} engine_scheduler_topology_buses={}/{} engine_scheduler_topology_send_returns={}/{} engine_scheduler_topology_consoles={}/{} engine_scheduler_topology_schedule_streams={:?} engine_scheduler_topology_issue_count={} engine_scheduler_topology_issues={:?}",
        summary.compatible,
        summary.requires_host_reinterpretation,
        summary.track_lane_node_count,
        summary.track_lane_group_count,
        summary.bus_node_count,
        summary.bus_group_count,
        summary.send_return_node_count,
        summary.send_return_group_count,
        summary.console_node_count,
        summary.console_group_count,
        summary.schedule_stream_count,
        summary.issues.len(),
        summary.issues,
    )
}

fn format_scheduler_topology_multiline(summary: &RuntimeSchedulerTopologySummary) -> String {
    format!(
        "\nengine_scheduler_topology_compatible={}\nengine_scheduler_topology_requires_host_reinterpretation={}\nengine_scheduler_topology_track_lane_nodes={}\nengine_scheduler_topology_track_lane_groups={}\nengine_scheduler_topology_bus_nodes={}\nengine_scheduler_topology_bus_groups={}\nengine_scheduler_topology_send_return_nodes={}\nengine_scheduler_topology_send_return_groups={}\nengine_scheduler_topology_console_nodes={}\nengine_scheduler_topology_console_groups={}\nengine_scheduler_topology_schedule_streams={:?}\nengine_scheduler_topology_issue_count={}\nengine_scheduler_topology_issues={:?}",
        summary.compatible,
        summary.requires_host_reinterpretation,
        summary.track_lane_node_count,
        summary.track_lane_group_count,
        summary.bus_node_count,
        summary.bus_group_count,
        summary.send_return_node_count,
        summary.send_return_group_count,
        summary.console_node_count,
        summary.console_group_count,
        summary.schedule_stream_count,
        summary.issues.len(),
        summary.issues,
    )
}

fn format_runtime_scheduler_summary_compact(summary: &RuntimeSchedulerExportSummary) -> String {
    format!(
        " scheduler_summary_phases={}/{} scheduler_summary_lanes={}/{} scheduler_summary_dispatches={}/{}/{} scheduler_summary_handoffs={} scheduler_summary_prework={:?}/{:?}/{:?} scheduler_summary_pending={}/{} scheduler_summary_topology={}/{}/{} scheduler_summary_lane_order={:?} scheduler_summary_dispatch_order={:?}",
        summary.phase_count,
        summary.anticipative_phase_count,
        summary.lane_count,
        summary.anticipative_lane_count,
        summary.dispatch_count,
        summary.prepared_dispatch_count,
        summary.realtime_dispatch_count,
        summary.dispatch_handoff_count,
        summary.prework_service_state,
        summary.prework_service_pressure,
        summary.prework_service_semantic_policy,
        summary.prework_pending_target_count,
        summary.prework_pending_deferred_target_count,
        summary.topology_compatible,
        summary.topology_requires_host_reinterpretation,
        summary.topology_issue_count,
        summary.lane_order,
        summary.dispatch_order,
    )
}

fn format_runtime_scheduler_snapshot_compact(snapshot: &RuntimeSchedulerSnapshot) -> String {
    format!(
        " scheduler_snapshot_state={:?} scheduler_snapshot_phase={:?} scheduler_snapshot_graph_applied={} scheduler_snapshot_schedule_applied={} scheduler_snapshot_transport_projected={} scheduler_snapshot_anticipative_enabled={} scheduler_snapshot_graph_id={:?} scheduler_snapshot_phase_count={} scheduler_snapshot_lane_count={} scheduler_snapshot_dispatch_count={} scheduler_snapshot_pending_prework_targets={} scheduler_snapshot_processed_blocks={}",
        snapshot.state,
        snapshot.phase,
        snapshot.graph_applied,
        snapshot.schedule_applied,
        snapshot.transport_projected,
        snapshot.anticipative_enabled,
        snapshot.active_graph_id,
        snapshot.phase_count,
        snapshot.lane_count,
        snapshot.dispatch_count,
        snapshot.pending_prework_target_count,
        snapshot.processed_block_count,
    )
}

fn format_runtime_scheduler_summary_multiline(summary: &RuntimeSchedulerExportSummary) -> String {
    format!(
        "\nscheduler_summary_phase_count={}\nscheduler_summary_anticipative_phase_count={}\nscheduler_summary_lane_count={}\nscheduler_summary_anticipative_lane_count={}\nscheduler_summary_dispatch_count={}\nscheduler_summary_prepared_dispatch_count={}\nscheduler_summary_realtime_dispatch_count={}\nscheduler_summary_dispatch_handoffs={}\nscheduler_summary_prework_state={:?}\nscheduler_summary_prework_pressure={:?}\nscheduler_summary_prework_policy={:?}\nscheduler_summary_pending_targets={}\nscheduler_summary_pending_deferred_targets={}\nscheduler_summary_topology_compatible={}\nscheduler_summary_topology_requires_host_reinterpretation={}\nscheduler_summary_topology_issue_count={}\nscheduler_summary_lane_order={:?}\nscheduler_summary_dispatch_order={:?}",
        summary.phase_count,
        summary.anticipative_phase_count,
        summary.lane_count,
        summary.anticipative_lane_count,
        summary.dispatch_count,
        summary.prepared_dispatch_count,
        summary.realtime_dispatch_count,
        summary.dispatch_handoff_count,
        summary.prework_service_state,
        summary.prework_service_pressure,
        summary.prework_service_semantic_policy,
        summary.prework_pending_target_count,
        summary.prework_pending_deferred_target_count,
        summary.topology_compatible,
        summary.topology_requires_host_reinterpretation,
        summary.topology_issue_count,
        summary.lane_order,
        summary.dispatch_order,
    )
}

fn format_runtime_scheduler_snapshot_multiline(snapshot: &RuntimeSchedulerSnapshot) -> String {
    format!(
        "\nscheduler_snapshot_state={:?}\nscheduler_snapshot_phase={:?}\nscheduler_snapshot_graph_applied={}\nscheduler_snapshot_schedule_applied={}\nscheduler_snapshot_transport_projected={}\nscheduler_snapshot_anticipative_enabled={}\nscheduler_snapshot_graph_id={:?}\nscheduler_snapshot_phase_count={}\nscheduler_snapshot_lane_count={}\nscheduler_snapshot_dispatch_count={}\nscheduler_snapshot_pending_prework_target_count={}\nscheduler_snapshot_processed_block_count={}",
        snapshot.state,
        snapshot.phase,
        snapshot.graph_applied,
        snapshot.schedule_applied,
        snapshot.transport_projected,
        snapshot.anticipative_enabled,
        snapshot.active_graph_id,
        snapshot.phase_count,
        snapshot.lane_count,
        snapshot.dispatch_count,
        snapshot.pending_prework_target_count,
        snapshot.processed_block_count,
    )
}

fn format_runtime_block_summary_compact(summary: &RuntimeBlockExecutionSummary) -> String {
    format!(
        " block_summary_processed={} block_summary_last={:?}/{:?}/{}ch@{} block_summary_prework={:?}/{:?}/{:?} block_summary_latency_tail={}/{}/{} block_summary_levels={:?}/{:?}/{:?} block_summary_transport={}/{:?}/{} block_summary_context={:?}/{:?}/{:?}/{:?}",
        summary.processed_blocks,
        summary.last_processing_epoch,
        summary.last_block_sequence,
        summary.last_channel_count,
        summary.last_frame_count,
        summary.prework_cache_state,
        summary.prework_cache_freshness_state,
        summary.last_prework_invalidation_reason,
        summary.total_latency_samples,
        summary.total_tail_samples,
        summary.output_tail_samples,
        summary.last_input_peak,
        summary.last_output_peak,
        summary.last_output_rms,
        summary.transport_epoch,
        summary.transport_transition,
        summary.transport_loop_wrapped,
        summary.context_anticipative,
        summary.transport_playing,
        summary.transport_tempo_bpm,
        summary.timeline_position_samples,
    )
}

fn format_runtime_block_summary_multiline(summary: &RuntimeBlockExecutionSummary) -> String {
    format!(
        "\nblock_summary_processed_blocks={}\nblock_summary_last_processing_epoch={:?}\nblock_summary_last_block_sequence={:?}\nblock_summary_last_frame_count={}\nblock_summary_last_channel_count={}\nblock_summary_prework_cache_state={:?}\nblock_summary_prework_cache_freshness_state={:?}\nblock_summary_last_prework_invalidation_reason={:?}\nblock_summary_total_latency_samples={}\nblock_summary_total_tail_samples={}\nblock_summary_output_tail_samples={}\nblock_summary_max_bus_tail_samples={}\nblock_summary_last_input_peak={:?}\nblock_summary_last_output_peak={:?}\nblock_summary_last_output_rms={:?}\nblock_summary_transport_epoch={}\nblock_summary_transport_transition={:?}\nblock_summary_transport_loop_wrapped={}\nblock_summary_context_anticipative={:?}\nblock_summary_transport_playing={:?}\nblock_summary_transport_tempo_bpm={:?}\nblock_summary_timeline_position_samples={:?}",
        summary.processed_blocks,
        summary.last_processing_epoch,
        summary.last_block_sequence,
        summary.last_frame_count,
        summary.last_channel_count,
        summary.prework_cache_state,
        summary.prework_cache_freshness_state,
        summary.last_prework_invalidation_reason,
        summary.total_latency_samples,
        summary.total_tail_samples,
        summary.output_tail_samples,
        summary.max_bus_tail_samples,
        summary.last_input_peak,
        summary.last_output_peak,
        summary.last_output_rms,
        summary.transport_epoch,
        summary.transport_transition,
        summary.transport_loop_wrapped,
        summary.context_anticipative,
        summary.transport_playing,
        summary.transport_tempo_bpm,
        summary.timeline_position_samples,
    )
}

fn format_runtime_degradation_summary_compact(summary: &RuntimeDegradationSummary) -> String {
    format!(
        " degradation_summary_state={}/{} degradation_summary_faults={}/{}/{}/{} degradation_summary_recovery={} degradation_summary_sessions={}/{}/{}/{}/{} degradation_summary_gates={}/{} degradation_summary_last_watchdog={:?}",
        summary.readiness_degraded,
        summary.safe_mode_enabled,
        summary.plugin_fault_count,
        summary.transport_fault_event_count,
        summary.broker_failure_event_count,
        summary.sandbox_operation_failure_event_count,
        summary.recovery_event_count,
        summary.recovery_overlap_sessions,
        summary.lingering_sessions,
        summary.degraded_bound_plugin_sandboxes,
        summary.missing_bound_plugin_sandboxes,
        summary.detach_faulted_sessions,
        summary.plugin_gate_active,
        summary.transport_gate_active,
        summary.last_watchdog_trigger,
    )
}

fn format_runtime_degradation_summary_multiline(summary: &RuntimeDegradationSummary) -> String {
    format!(
        "\ndegradation_summary_readiness_degraded={}\ndegradation_summary_safe_mode_enabled={}\ndegradation_summary_xruns={}\ndegradation_summary_plugin_faults={}\ndegradation_summary_transport_fault_events={}\ndegradation_summary_broker_failure_events={}\ndegradation_summary_sandbox_operation_failure_events={}\ndegradation_summary_recovery_events={}\ndegradation_summary_active_plugin_sandboxes={}\ndegradation_summary_recovery_overlap_sessions={}\ndegradation_summary_lingering_sessions={}\ndegradation_summary_degraded_bound_plugin_sandboxes={}\ndegradation_summary_missing_bound_plugin_sandboxes={}\ndegradation_summary_detach_faulted_sessions={}\ndegradation_summary_plugin_gate_active={}\ndegradation_summary_transport_gate_active={}\ndegradation_summary_last_watchdog_trigger={:?}",
        summary.readiness_degraded,
        summary.safe_mode_enabled,
        summary.xrun_count,
        summary.plugin_fault_count,
        summary.transport_fault_event_count,
        summary.broker_failure_event_count,
        summary.sandbox_operation_failure_event_count,
        summary.recovery_event_count,
        summary.active_plugin_sandboxes,
        summary.recovery_overlap_sessions,
        summary.lingering_sessions,
        summary.degraded_bound_plugin_sandboxes,
        summary.missing_bound_plugin_sandboxes,
        summary.detach_faulted_sessions,
        summary.plugin_gate_active,
        summary.transport_gate_active,
        summary.last_watchdog_trigger,
    )
}

fn format_runtime_execution_topology_summary_compact(
    summary: &RuntimeExecutionTopologySummary,
) -> String {
    let lane_shapes = summary
        .lanes
        .iter()
        .map(|lane| format!("{:?}:{}", lane.lane, lane.node_ids.len()))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        " execution_topology_summary_nodes={} execution_topology_summary_roles={}/{}/{}/{}/{} execution_topology_summary_groups={}/{}/{} execution_topology_summary_lanes={} execution_topology_summary_lane_shapes={}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.console_group_count,
        summary.lane_count,
        lane_shapes,
    )
}

fn format_runtime_execution_topology_summary_multiline(
    summary: &RuntimeExecutionTopologySummary,
) -> String {
    let lane_lines = summary
        .lanes
        .iter()
        .enumerate()
        .map(|(index, lane)| {
            format!(
                "\nexecution_topology_summary_lane_{}={:?}/groups={:?}/nodes={:?}/roles={:?}/track_lanes={:?}/bus_groups={:?}",
                index,
                lane.lane,
                lane.groups,
                lane.node_ids,
                lane.topology_roles,
                lane.track_lane_ids,
                lane.bus_group_ids,
            )
        })
        .collect::<String>();
    let node_lines = summary
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            format!(
                "\nexecution_topology_summary_node_{}={}/{:?}/{:?}/{:?}/lane_id={:?}/bus_group_id={:?}/input={}/output={}/plugin={:?}",
                index,
                node.node_id,
                node.lane,
                node.group,
                node.topology_role,
                node.lane_id,
                node.bus_group_id,
                node.input_bus_id,
                node.output_bus_id,
                node.plugin_sandbox_id,
            )
        })
        .collect::<String>();
    format!(
        "\nexecution_topology_summary_node_count={}\nexecution_topology_summary_utility_nodes={}\nexecution_topology_summary_track_lane_nodes={}\nexecution_topology_summary_bus_nodes={}\nexecution_topology_summary_send_return_nodes={}\nexecution_topology_summary_console_nodes={}\nexecution_topology_summary_lane_count={}\nexecution_topology_summary_track_lane_groups={}\nexecution_topology_summary_bus_groups={}\nexecution_topology_summary_console_groups={}{}{}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.lane_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.console_group_count,
        lane_lines,
        node_lines,
    )
}

fn json_runtime_scheduler_export_summary(summary: &RuntimeSchedulerExportSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"phase_count\":{},",
            "\"anticipative_phase_count\":{},",
            "\"lane_count\":{},",
            "\"anticipative_lane_count\":{},",
            "\"dispatch_count\":{},",
            "\"prepared_dispatch_count\":{},",
            "\"realtime_dispatch_count\":{},",
            "\"dispatch_handoff_count\":{},",
            "\"prework_service_state\":{},",
            "\"prework_service_pressure\":{},",
            "\"prework_service_semantic_policy\":{},",
            "\"prework_pending_target_count\":{},",
            "\"prework_pending_deferred_target_count\":{},",
            "\"topology_compatible\":{},",
            "\"topology_requires_host_reinterpretation\":{},",
            "\"topology_issue_count\":{},",
            "\"lane_order\":{},",
            "\"dispatch_order\":{}",
            "}}"
        ),
        summary.phase_count,
        summary.anticipative_phase_count,
        summary.lane_count,
        summary.anticipative_lane_count,
        summary.dispatch_count,
        summary.prepared_dispatch_count,
        summary.realtime_dispatch_count,
        summary.dispatch_handoff_count,
        json_escape_string(&format!("{:?}", summary.prework_service_state)),
        json_escape_string(&format!("{:?}", summary.prework_service_pressure)),
        json_escape_string(&format!("{:?}", summary.prework_service_semantic_policy)),
        summary.prework_pending_target_count,
        summary.prework_pending_deferred_target_count,
        summary.topology_compatible,
        summary.topology_requires_host_reinterpretation,
        summary.topology_issue_count,
        json_runtime_execution_lane_order(&summary.lane_order),
        json_runtime_execution_lane_order(&summary.dispatch_order),
    )
}

fn json_runtime_scheduler_snapshot(snapshot: &RuntimeSchedulerSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"state\":{},",
            "\"phase\":{},",
            "\"graph_applied\":{},",
            "\"schedule_applied\":{},",
            "\"transport_projected\":{},",
            "\"anticipative_enabled\":{},",
            "\"active_graph_id\":{},",
            "\"phase_count\":{},",
            "\"lane_count\":{},",
            "\"dispatch_count\":{},",
            "\"pending_prework_target_count\":{},",
            "\"processed_block_count\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", snapshot.state)),
        json_escape_string(&format!("{:?}", snapshot.phase)),
        snapshot.graph_applied,
        snapshot.schedule_applied,
        snapshot.transport_projected,
        snapshot.anticipative_enabled,
        json_option_string(snapshot.active_graph_id.as_deref()),
        snapshot.phase_count,
        snapshot.lane_count,
        snapshot.dispatch_count,
        snapshot.pending_prework_target_count,
        snapshot.processed_block_count,
    )
}

fn json_runtime_block_execution_summary(summary: &RuntimeBlockExecutionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"processed_blocks\":{},",
            "\"last_processing_epoch\":{},",
            "\"last_block_sequence\":{},",
            "\"last_frame_count\":{},",
            "\"last_channel_count\":{},",
            "\"prework_cache_state\":{},",
            "\"prework_cache_freshness_state\":{},",
            "\"last_prework_invalidation_reason\":{},",
            "\"total_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"output_tail_samples\":{},",
            "\"max_bus_tail_samples\":{},",
            "\"last_input_peak\":{},",
            "\"last_output_peak\":{},",
            "\"last_output_rms\":{},",
            "\"transport_epoch\":{},",
            "\"transport_transition\":{},",
            "\"transport_loop_wrapped\":{},",
            "\"context_anticipative\":{},",
            "\"transport_playing\":{},",
            "\"transport_tempo_bpm\":{},",
            "\"timeline_position_samples\":{}",
            "}}"
        ),
        summary.processed_blocks,
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.last_block_sequence),
        summary.last_frame_count,
        summary.last_channel_count,
        json_escape_string(&format!("{:?}", summary.prework_cache_state)),
        json_escape_string(&format!("{:?}", summary.prework_cache_freshness_state)),
        json_option_string(
            summary
                .last_prework_invalidation_reason
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        summary.total_latency_samples,
        summary.total_tail_samples,
        summary.output_tail_samples,
        summary.max_bus_tail_samples,
        json_option_f32(summary.last_input_peak),
        json_option_f32(summary.last_output_peak),
        json_option_f32(summary.last_output_rms),
        summary.transport_epoch,
        json_option_string(
            summary
                .transport_transition
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
        summary.transport_loop_wrapped,
        match summary.context_anticipative {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        match summary.transport_playing {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        json_option_f64(summary.transport_tempo_bpm),
        json_option_i64(summary.timeline_position_samples),
    )
}

fn json_runtime_degradation_summary(summary: &RuntimeDegradationSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"readiness_degraded\":{},",
            "\"safe_mode_enabled\":{},",
            "\"xrun_count\":{},",
            "\"plugin_fault_count\":{},",
            "\"transport_fault_event_count\":{},",
            "\"broker_failure_event_count\":{},",
            "\"sandbox_operation_failure_event_count\":{},",
            "\"recovery_event_count\":{},",
            "\"active_plugin_sandboxes\":{},",
            "\"recovery_overlap_sessions\":{},",
            "\"lingering_sessions\":{},",
            "\"degraded_bound_plugin_sandboxes\":{},",
            "\"missing_bound_plugin_sandboxes\":{},",
            "\"detach_faulted_sessions\":{},",
            "\"transport_gate_active\":{},",
            "\"plugin_gate_active\":{},",
            "\"last_watchdog_trigger\":{}",
            "}}"
        ),
        summary.readiness_degraded,
        summary.safe_mode_enabled,
        summary.xrun_count,
        summary.plugin_fault_count,
        summary.transport_fault_event_count,
        summary.broker_failure_event_count,
        summary.sandbox_operation_failure_event_count,
        summary.recovery_event_count,
        summary.active_plugin_sandboxes,
        summary.recovery_overlap_sessions,
        summary.lingering_sessions,
        summary.degraded_bound_plugin_sandboxes,
        summary.missing_bound_plugin_sandboxes,
        summary.detach_faulted_sessions,
        summary.transport_gate_active,
        summary.plugin_gate_active,
        json_option_string(
            summary
                .last_watchdog_trigger
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
    )
}

fn json_runtime_execution_topology_summary(summary: &RuntimeExecutionTopologySummary) -> String {
    format!(
        concat!(
            "{{",
            "\"node_count\":{},",
            "\"utility_node_count\":{},",
            "\"track_lane_node_count\":{},",
            "\"bus_node_count\":{},",
            "\"send_return_node_count\":{},",
            "\"console_node_count\":{},",
            "\"lane_count\":{},",
            "\"track_lane_group_count\":{},",
            "\"bus_group_count\":{},",
            "\"console_group_count\":{},",
            "\"lanes\":{},",
            "\"nodes\":{}",
            "}}"
        ),
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.lane_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.console_group_count,
        json_runtime_execution_topology_lanes(&summary.lanes),
        json_runtime_execution_topology_nodes(&summary.nodes),
    )
}

fn json_runtime_scheduler_topology_issue(issue: &RuntimeSchedulerTopologyIssue) -> String {
    match issue {
        RuntimeSchedulerTopologyIssue::MissingTrackLaneIds { node_count } => format!(
            "{{\"kind\":\"MissingTrackLaneIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingBusGroupIds { node_count } => format!(
            "{{\"kind\":\"MissingBusGroupIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingConsoleGroupIds { node_count } => format!(
            "{{\"kind\":\"MissingConsoleGroupIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingRealtimeLaneForTopology => {
            "{\"kind\":\"MissingRealtimeLaneForTopology\"}".into()
        }
        RuntimeSchedulerTopologyIssue::AnticipativeLaneMustPrecedeRealtime => {
            "{\"kind\":\"AnticipativeLaneMustPrecedeRealtime\"}".into()
        }
        RuntimeSchedulerTopologyIssue::RealtimeDispatchMustTerminateTopology => {
            "{\"kind\":\"RealtimeDispatchMustTerminateTopology\"}".into()
        }
        RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
            required_streams,
        } => format!(
            "{{\"kind\":\"MissingScheduleProjectionForTrackLanes\",\"required_streams\":{}}}",
            required_streams
        ),
        RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
            required_streams,
            actual_streams,
        } => format!(
            "{{\"kind\":\"InsufficientScheduleStreams\",\"required_streams\":{},\"actual_streams\":{}}}",
            required_streams, actual_streams
        ),
    }
}

fn json_runtime_scheduler_topology_summary(summary: &RuntimeSchedulerTopologySummary) -> String {
    let issues = summary
        .issues
        .iter()
        .map(json_runtime_scheduler_topology_issue)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"track_lane_node_count\":{},",
            "\"track_lane_group_count\":{},",
            "\"bus_node_count\":{},",
            "\"bus_group_count\":{},",
            "\"send_return_node_count\":{},",
            "\"send_return_group_count\":{},",
            "\"console_node_count\":{},",
            "\"console_group_count\":{},",
            "\"schedule_stream_count\":{},",
            "\"compatible\":{},",
            "\"requires_host_reinterpretation\":{},",
            "\"issues\":[{}]",
            "}}"
        ),
        summary.track_lane_node_count,
        summary.track_lane_group_count,
        summary.bus_node_count,
        summary.bus_group_count,
        summary.send_return_node_count,
        summary.send_return_group_count,
        summary.console_node_count,
        summary.console_group_count,
        json_option_usize(summary.schedule_stream_count),
        summary.compatible,
        summary.requires_host_reinterpretation,
        issues,
    )
}

fn json_runtime_automation_snapshot(snapshot: &RuntimeAutomationSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"parameter_id\":{},",
            "\"value_events\":{},",
            "\"modulation_events\":{},",
            "\"gesture_begin_events\":{},",
            "\"gesture_end_events\":{},",
            "\"first_value\":{},",
            "\"last_value\":{},",
            "\"last_modulation\":{},",
            "\"first_epoch\":{},",
            "\"last_epoch\":{},",
            "\"segment_count\":{},",
            "\"segment_epochs\":{},",
            "\"lease_rollovers\":{}",
            "}}"
        ),
        snapshot.parameter_id,
        snapshot.value_events,
        snapshot.modulation_events,
        snapshot.gesture_begin_events,
        snapshot.gesture_end_events,
        json_option_f32(snapshot.first_value),
        json_option_f32(snapshot.last_value),
        json_option_f32(snapshot.last_modulation),
        json_option_u64(snapshot.first_epoch),
        json_option_u64(snapshot.last_epoch),
        snapshot.segment_count,
        json_u64_vec(&snapshot.segment_epochs),
        snapshot.lease_rollovers,
    )
}

fn json_runtime_engine_block_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"graph_id\":{},",
            "\"node_count\":{},",
            "\"stateful_node_count\":{},",
            "\"latency_node_count\":{},",
            "\"plugin_backed_node_count\":{},",
            "\"anticipative_planning_enabled\":{},",
            "\"inline_realtime_node_count\":{},",
            "\"stateful_realtime_node_count\":{},",
            "\"anticipative_eligible_node_count\":{},",
            "\"phase_count\":{},",
            "\"anticipative_phase_count\":{},",
            "\"phase_order\":{},",
            "\"lane_count\":{},",
            "\"anticipative_lane_count\":{},",
            "\"lane_order\":{},",
            "\"scheduler_topology\":{},",
            "\"dispatch_count\":{},",
            "\"dispatch_boundary_count\":{},",
            "\"dispatch_order\":{},",
            "\"prepared_dispatch_count\":{},",
            "\"realtime_dispatch_count\":{},",
            "\"dispatch_handoff_count\":{},",
            "\"prework_cache_enabled\":{},",
            "\"prework_cache_state\":\"{:?}\",",
            "\"prework_cache_queue_capacity\":{},",
            "\"prework_cache_queue_depth\":{},",
            "\"prework_cache_peak_queue_depth\":{},",
            "\"prework_pending_target_count\":{},",
            "\"prework_pending_immediate_target_count\":{},",
            "\"prework_pending_near_term_target_count\":{},",
            "\"prework_pending_deferred_target_count\":{},",
            "\"prework_next_pending_target_block_sequence\":{},",
            "\"prework_service_state\":\"{:?}\",",
            "\"prework_service_pressure\":\"{:?}\",",
            "\"prework_service_semantic_policy\":\"{:?}\",",
            "\"prework_service_active_plugin_sandboxes\":{},",
            "\"prework_service_bound_plugin_sandboxes\":{},",
            "\"prework_service_active_bound_plugin_sandboxes\":{},",
            "\"prework_service_degraded_bound_plugin_sandboxes\":{},",
            "\"prework_service_missing_bound_plugin_sandboxes\":{},",
            "\"prework_service_plugin_gate_active\":{},",
            "\"prework_service_cycle_count\":{},",
            "\"prework_service_prepared_targets\":{},",
            "\"prework_service_pause_count\":{},",
            "\"prework_service_resume_count\":{},",
            "\"prework_service_starvation_count\":{},",
            "\"prework_service_throttle_count\":{},",
            "\"prework_service_yield_count\":{},",
            "\"last_prework_service_processing_epoch\":{},",
            "\"last_prework_service_requested_cycles\":{},",
            "\"last_prework_service_effective_cycles\":{},",
            "\"last_prework_service_cycle_count\":{},",
            "\"last_prework_service_budget_per_cycle\":{},",
            "\"last_prework_service_effective_budget_per_cycle\":{},",
            "\"last_prework_service_prepared_targets\":{},",
            "\"last_prework_serviced_target_block_sequence\":{},",
            "\"last_prework_serviced_backlog_class\":{},",
            "\"prework_forecast_requested_mode\":\"{:?}\",",
            "\"prework_forecast_mode\":\"{:?}\",",
            "\"prework_forecast_policy_configured\":{},",
            "\"prework_forecast_profile\":{},",
            "\"prework_forecast_profile_source\":{},",
            "\"prework_forecast_profile_target_window_override\":{},",
            "\"prework_forecast_policy_target_window_blocks\":{},",
            "\"prework_cache_freshness_state\":\"{:?}\",",
            "\"prework_cache_block_freshness_window\":{},",
            "\"prework_cache_remaining_valid_blocks\":{},",
            "\"prework_cache_admissions\":{},",
            "\"prework_cache_consumptions\":{},",
            "\"prework_cache_queued_admissions\":{},",
            "\"prework_cache_queued_consumptions\":{},",
            "\"prework_cache_hits\":{},",
            "\"prework_cache_misses\":{},",
            "\"prework_cache_invalidation_count\":{},",
            "\"prework_cache_retirement_count\":{},",
            "\"prework_cache_unconsumed_retirement_count\":{},",
            "\"prework_cache_consumed_retirement_count\":{},",
            "\"last_prework_cache_hit\":{},",
            "\"last_prework_invalidation_reason\":{},",
            "\"last_prework_retirement_reason\":{},",
            "\"last_prework_retired_unconsumed\":{},",
            "\"prework_cache_valid_until_processing_epoch\":{},",
            "\"prework_cache_valid_until_block_sequence\":{},",
            "\"last_prework_source_processing_epoch\":{},",
            "\"last_prework_source_block_sequence\":{},",
            "\"last_prework_admission_processing_epoch\":{},",
            "\"last_prework_admission_block_sequence\":{},",
            "\"last_prework_admitted_from_block_sequence\":{},",
            "\"last_prework_consumption_processing_epoch\":{},",
            "\"last_prework_consumption_block_sequence\":{},",
            "\"last_prework_consumed_from_block_sequence\":{},",
            "\"last_prework_retirement_processing_epoch\":{},",
            "\"last_prework_retirement_block_sequence\":{},",
            "\"planned_nodes\":{},",
            "\"stage_count\":{},",
            "\"dynamic_kernel_stage_count\":{},",
            "\"dynamic_stage_state_model\":\"{:?}\",",
            "\"total_latency_samples\":{},",
            "\"max_node_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"max_node_tail_samples\":{},",
            "\"output_tail_samples\":{},",
            "\"max_bus_tail_samples\":{},",
            "\"processed_blocks\":{},",
            "\"last_processing_epoch\":{},",
            "\"last_block_sequence\":{},",
            "\"last_frame_count\":{},",
            "\"last_channel_count\":{},",
            "\"last_input_peak\":{},",
            "\"last_prework_output_peak\":{},",
            "\"last_realtime_input_peak\":{},",
            "\"last_output_peak\":{},",
            "\"last_output_rms\":{},",
            "\"last_first_output_sample\":{},",
            "\"transport_epoch\":{},",
            "\"transport_transition\":{},",
            "\"transport_block_start_samples\":{},",
            "\"transport_block_end_samples\":{},",
            "\"transport_loop_wrapped\":{},",
            "\"last_execution_context\":{}",
            "}}"
        ),
        json_option_string(snapshot.graph_id.as_deref()),
        snapshot.node_count,
        snapshot.stateful_node_count,
        snapshot.latency_node_count,
        snapshot.plugin_backed_node_count,
        snapshot.anticipative_planning_enabled,
        snapshot.inline_realtime_node_count,
        snapshot.stateful_realtime_node_count,
        snapshot.anticipative_eligible_node_count,
        snapshot.phase_count,
        snapshot.anticipative_phase_count,
        json_runtime_planning_group_order(&snapshot.phase_order),
        snapshot.lane_count,
        snapshot.anticipative_lane_count,
        json_runtime_execution_lane_order(&snapshot.lane_order),
        json_runtime_scheduler_topology_summary(&snapshot.scheduler_topology),
        snapshot.dispatch_count,
        snapshot.dispatch_boundary_count,
        json_runtime_execution_lane_order(&snapshot.dispatch_order),
        snapshot.prepared_dispatch_count,
        snapshot.realtime_dispatch_count,
        snapshot.dispatch_handoff_count,
        snapshot.prework_cache_enabled,
        snapshot.prework_cache_state,
        snapshot.prework_cache_queue_capacity,
        snapshot.prework_cache_queue_depth,
        snapshot.prework_cache_peak_queue_depth,
        snapshot.prework_pending_target_count,
        snapshot.prework_pending_immediate_target_count,
        snapshot.prework_pending_near_term_target_count,
        snapshot.prework_pending_deferred_target_count,
        json_option_u64(snapshot.prework_next_pending_target_block_sequence),
        snapshot.prework_service_state,
        snapshot.prework_service_pressure,
        snapshot.prework_service_semantic_policy,
        snapshot.prework_service_active_plugin_sandboxes,
        snapshot.prework_service_bound_plugin_sandboxes,
        snapshot.prework_service_active_bound_plugin_sandboxes,
        snapshot.prework_service_degraded_bound_plugin_sandboxes,
        snapshot.prework_service_missing_bound_plugin_sandboxes,
        snapshot.prework_service_plugin_gate_active,
        snapshot.prework_service_cycle_count,
        snapshot.prework_service_prepared_targets,
        snapshot.prework_service_pause_count,
        snapshot.prework_service_resume_count,
        snapshot.prework_service_starvation_count,
        snapshot.prework_service_throttle_count,
        snapshot.prework_service_yield_count,
        json_option_u64(snapshot.last_prework_service_processing_epoch),
        snapshot.last_prework_service_requested_cycles,
        snapshot.last_prework_service_effective_cycles,
        snapshot.last_prework_service_cycle_count,
        match snapshot.last_prework_service_budget_per_cycle {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        match snapshot.last_prework_service_effective_budget_per_cycle {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        snapshot.last_prework_service_prepared_targets,
        json_option_u64(snapshot.last_prework_serviced_target_block_sequence),
        json_option_string(
            snapshot
                .last_prework_serviced_backlog_class
                .map(|backlog| format!("{backlog:?}"))
                .as_deref(),
        ),
        snapshot.prework_forecast_requested_mode,
        snapshot.prework_forecast_mode,
        snapshot.prework_forecast_policy_configured,
        json_option_string(
            snapshot
                .prework_forecast_profile
                .map(|profile| format!("{profile:?}"))
                .as_deref(),
        ),
        json_option_string(
            snapshot
                .prework_forecast_profile_source
                .map(|source| format!("{source:?}"))
                .as_deref(),
        ),
        match snapshot.prework_forecast_profile_target_window_override {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        match snapshot.prework_forecast_policy_target_window_blocks {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        snapshot.prework_cache_freshness_state,
        snapshot.prework_cache_block_freshness_window,
        json_option_u64(snapshot.prework_cache_remaining_valid_blocks),
        snapshot.prework_cache_admissions,
        snapshot.prework_cache_consumptions,
        snapshot.prework_cache_queued_admissions,
        snapshot.prework_cache_queued_consumptions,
        snapshot.prework_cache_hits,
        snapshot.prework_cache_misses,
        snapshot.prework_cache_invalidation_count,
        snapshot.prework_cache_retirement_count,
        snapshot.prework_cache_unconsumed_retirement_count,
        snapshot.prework_cache_consumed_retirement_count,
        snapshot.last_prework_cache_hit,
        json_option_string(
            snapshot
                .last_prework_invalidation_reason
                .map(|reason| format!("{reason:?}"))
                .as_deref(),
        ),
        json_option_string(
            snapshot
                .last_prework_retirement_reason
                .map(|reason| format!("{reason:?}"))
                .as_deref(),
        ),
        match snapshot.last_prework_retired_unconsumed {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        json_option_u64(snapshot.prework_cache_valid_until_processing_epoch),
        json_option_u64(snapshot.prework_cache_valid_until_block_sequence),
        json_option_u64(snapshot.last_prework_source_processing_epoch),
        json_option_u64(snapshot.last_prework_source_block_sequence),
        json_option_u64(snapshot.last_prework_admission_processing_epoch),
        json_option_u64(snapshot.last_prework_admission_block_sequence),
        json_option_u64(snapshot.last_prework_admitted_from_block_sequence),
        json_option_u64(snapshot.last_prework_consumption_processing_epoch),
        json_option_u64(snapshot.last_prework_consumption_block_sequence),
        json_option_u64(snapshot.last_prework_consumed_from_block_sequence),
        json_option_u64(snapshot.last_prework_retirement_processing_epoch),
        json_option_u64(snapshot.last_prework_retirement_block_sequence),
        json_runtime_planned_graph_nodes(&snapshot.planned_nodes),
        snapshot.stage_count,
        snapshot.dynamic_kernel_stage_count,
        snapshot.dynamic_stage_state_model,
        snapshot.total_latency_samples,
        snapshot.max_node_latency_samples,
        snapshot.total_tail_samples,
        snapshot.max_node_tail_samples,
        snapshot.output_tail_samples,
        snapshot.max_bus_tail_samples,
        snapshot.processed_blocks,
        json_option_u64(snapshot.last_processing_epoch),
        json_option_u64(snapshot.last_block_sequence),
        snapshot.last_frame_count,
        snapshot.last_channel_count,
        json_option_f32(snapshot.last_input_peak),
        json_option_f32(snapshot.last_prework_output_peak),
        json_option_f32(snapshot.last_realtime_input_peak),
        json_option_f32(snapshot.last_output_peak),
        json_option_f32(snapshot.last_output_rms),
        json_option_f32(snapshot.last_first_output_sample),
        snapshot.transport_epoch,
        json_option_string(
            snapshot
                .transport_transition
                .map(|transition| format!("{transition:?}"))
                .as_deref(),
        ),
        json_option_i64(snapshot.transport_block_start_samples),
        json_option_i64(snapshot.transport_block_end_samples),
        snapshot.transport_loop_wrapped,
        snapshot
            .last_execution_context
            .as_ref()
            .map(json_graph_execution_context)
            .unwrap_or_else(|| "null".into()),
    )
}

fn json_runtime_planned_graph_nodes(nodes: &[RuntimePlannedGraphNode]) -> String {
    format!(
        "[{}]",
        nodes
            .iter()
            .map(|node| {
                format!(
                    concat!(
                        "{{",
                        "\"node_id\":{},",
                        "\"plugin_sandbox_id\":{},",
                        "\"execution_class\":{},",
                        "\"group\":{},",
                        "\"latency_samples\":{},",
                        "\"topology_role\":{},",
                        "\"lane_id\":{},",
                        "\"bus_group_id\":{},",
                        "\"input_bus_id\":{},",
                        "\"output_bus_id\":{}",
                        "}}"
                    ),
                    json_option_string(Some(node.node_id.as_str())),
                    json_option_string(node.plugin_sandbox_id.as_deref()),
                    json_option_string(Some(match node.execution_class {
                        GraphNodeExecutionClass::PureTransform => "PureTransform",
                        GraphNodeExecutionClass::Stateful => "Stateful",
                        GraphNodeExecutionClass::LatencyBearing => "LatencyBearing",
                        GraphNodeExecutionClass::PluginBacked => "PluginBacked",
                    })),
                    json_option_string(Some(match node.group {
                        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
                    })),
                    node.latency_samples,
                    json_option_string(Some(match node.topology_role {
                        GraphNodeTopologyRole::Utility => "Utility",
                        GraphNodeTopologyRole::TrackLane => "TrackLane",
                        GraphNodeTopologyRole::Bus => "Bus",
                        GraphNodeTopologyRole::Send => "Send",
                        GraphNodeTopologyRole::Return => "Return",
                        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
                    })),
                    json_option_string(node.lane_id.as_deref()),
                    json_option_string(node.bus_group_id.as_deref()),
                    json_option_string(Some(node.input_bus_id.as_str())),
                    json_option_string(Some(node.output_bus_id.as_str())),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_execution_topology_lanes(lanes: &[RuntimeExecutionLaneSummary]) -> String {
    format!(
        "[{}]",
        lanes
            .iter()
            .map(|lane| {
                format!(
                    concat!(
                        "{{",
                        "\"lane\":{},",
                        "\"groups\":{},",
                        "\"node_ids\":{},",
                        "\"topology_roles\":{},",
                        "\"track_lane_ids\":{},",
                        "\"bus_group_ids\":{}",
                        "}}"
                    ),
                    json_option_string(Some(match lane.lane {
                        GraphExecutionLane::Realtime => "Realtime",
                        GraphExecutionLane::Anticipative => "Anticipative",
                    })),
                    json_runtime_planning_group_order(&lane.groups),
                    json_string_vec(&lane.node_ids),
                    json_runtime_topology_role_vec(&lane.topology_roles),
                    json_string_vec(&lane.track_lane_ids),
                    json_string_vec(&lane.bus_group_ids),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_execution_topology_nodes(nodes: &[RuntimeExecutionNodeSummary]) -> String {
    format!(
        "[{}]",
        nodes
            .iter()
            .map(|node| {
                format!(
                    concat!(
                        "{{",
                        "\"node_id\":{},",
                        "\"lane\":{},",
                        "\"group\":{},",
                        "\"execution_class\":{},",
                        "\"topology_role\":{},",
                        "\"lane_id\":{},",
                        "\"bus_group_id\":{},",
                        "\"input_bus_id\":{},",
                        "\"output_bus_id\":{},",
                        "\"plugin_sandbox_id\":{}",
                        "}}"
                    ),
                    json_option_string(Some(node.node_id.as_str())),
                    json_option_string(Some(match node.lane {
                        GraphExecutionLane::Realtime => "Realtime",
                        GraphExecutionLane::Anticipative => "Anticipative",
                    })),
                    json_option_string(Some(match node.group {
                        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
                    })),
                    json_option_string(Some(match node.execution_class {
                        GraphNodeExecutionClass::PureTransform => "PureTransform",
                        GraphNodeExecutionClass::Stateful => "Stateful",
                        GraphNodeExecutionClass::LatencyBearing => "LatencyBearing",
                        GraphNodeExecutionClass::PluginBacked => "PluginBacked",
                    })),
                    json_option_string(Some(match node.topology_role {
                        GraphNodeTopologyRole::Utility => "Utility",
                        GraphNodeTopologyRole::TrackLane => "TrackLane",
                        GraphNodeTopologyRole::Bus => "Bus",
                        GraphNodeTopologyRole::Send => "Send",
                        GraphNodeTopologyRole::Return => "Return",
                        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
                    })),
                    json_option_string(node.lane_id.as_deref()),
                    json_option_string(node.bus_group_id.as_deref()),
                    json_option_string(Some(node.input_bus_id.as_str())),
                    json_option_string(Some(node.output_bus_id.as_str())),
                    json_option_string(node.plugin_sandbox_id.as_deref()),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_planning_group_order(groups: &[GraphNodePlanningGroup]) -> String {
    format!(
        "[{}]",
        groups
            .iter()
            .map(|group| {
                json_option_string(Some(match group {
                    GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
                    GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
                    GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
                }))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_topology_role_vec(roles: &[GraphNodeTopologyRole]) -> String {
    format!(
        "[{}]",
        roles
            .iter()
            .map(|role| {
                json_option_string(Some(match role {
                    GraphNodeTopologyRole::Utility => "Utility",
                    GraphNodeTopologyRole::TrackLane => "TrackLane",
                    GraphNodeTopologyRole::Bus => "Bus",
                    GraphNodeTopologyRole::Send => "Send",
                    GraphNodeTopologyRole::Return => "Return",
                    GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
                }))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_string_vec(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_option_string(Some(value.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_execution_lane_order(lanes: &[GraphExecutionLane]) -> String {
    format!(
        "[{}]",
        lanes
            .iter()
            .map(|lane| {
                json_option_string(Some(match lane {
                    GraphExecutionLane::Realtime => "Realtime",
                    GraphExecutionLane::Anticipative => "Anticipative",
                }))
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_graph_execution_context(context: &GraphExecutionContext) -> String {
    format!(
        concat!(
            "{{",
            "\"processing_epoch\":{},",
            "\"block_sequence\":{},",
            "\"projection_epoch\":{},",
            "\"parameter_epoch\":{},",
            "\"configured_block_size\":{},",
            "\"anticipative_enabled\":{},",
            "\"transport_playing\":{},",
            "\"transport_tempo_bpm\":{},",
            "\"timeline_position_samples\":{}",
            "}}"
        ),
        context.processing_epoch,
        context.block_sequence,
        context.projection_epoch,
        context.parameter_epoch,
        context.configured_block_size,
        context.anticipative_enabled,
        context.transport_playing,
        json_option_f64(Some(context.transport_tempo_bpm)),
        json_option_i64(Some(context.timeline_position_samples)),
    )
}

fn json_runtime_control_snapshot(snapshot: &RuntimeControlSnapshot) -> String {
    let last_stop_reason = snapshot
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    let last_reconfigure = snapshot.last_reconfigure.map(|request| {
        format!(
            "sample_rate={} block_size={} anticipative={} realtime_safe={}",
            request.sample_rate.0,
            request.block_size,
            request.anticipative_enabled,
            request.realtime_safe_mode
        )
    });
    format!(
        concat!(
            "{{",
            "\"handshaken\":{},",
            "\"configured\":{},",
            "\"running\":{},",
            "\"handshake_count\":{},",
            "\"configure_count\":{},",
            "\"start_count\":{},",
            "\"stop_count\":{},",
            "\"restart_count\":{},",
            "\"last_client_version\":{},",
            "\"last_stop_reason\":{},",
            "\"last_reconfigure\":{}",
            "}}"
        ),
        snapshot.handshaken,
        snapshot.configured,
        snapshot.running,
        snapshot.handshake_count,
        snapshot.configure_count,
        snapshot.start_count,
        snapshot.stop_count,
        snapshot.restart_count,
        json_option_string(snapshot.last_client_version.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_string(last_reconfigure.as_deref()),
    )
}

fn json_transport_fault_summary(summary: &TransportFaultSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"boundary_mode\":{},",
            "\"total_events\":{},",
            "\"host_broker_events\":{},",
            "\"sandbox_operation_events\":{},",
            "\"runtime_dispatch_events\":{},",
            "\"prepare_events\":{},",
            "\"dispatch_events\":{},",
            "\"teardown_events\":{},",
            "\"control_events\":{},",
            "\"first_processing_epoch\":{},",
            "\"last_processing_epoch\":{},",
            "\"first_block_sequence\":{},",
            "\"last_block_sequence\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", summary.boundary_mode)),
        summary.total_events,
        summary.host_broker_events,
        summary.sandbox_operation_events,
        summary.runtime_dispatch_events,
        summary.prepare_events,
        summary.dispatch_events,
        summary.teardown_events,
        summary.control_events,
        json_option_u64(summary.first_processing_epoch),
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.first_block_sequence),
        json_option_u64(summary.last_block_sequence),
    )
}

fn json_runtime_transport_concurrency_snapshot(
    snapshot: &RuntimeTransportConcurrencySnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"steady_session_limit\":{},",
            "\"recovery_session_limit\":{},",
            "\"current_attached_sessions\":{},",
            "\"peak_attached_sessions\":{},",
            "\"current_recovery_overlap_sessions\":{},",
            "\"peak_recovery_overlap_sessions\":{},",
            "\"current_lingering_sessions\":{},",
            "\"peak_lingering_sessions\":{},",
            "\"current_detach_requested_sessions\":{},",
            "\"current_detach_faulted_sessions\":{},",
            "\"pending_cleanup_work_items\":{},",
            "\"pending_deferred_retry_work_items\":{},",
            "\"next_cleanup_epoch\":{},",
            "\"oldest_pending_cleanup_ready_epoch\":{},",
            "\"pending_cleanup_waves\":{},",
            "\"active_sessions\":{},",
            "\"last_admitted_sandbox_id\":{},",
            "\"last_rejected_sandbox_id\":{},",
            "\"last_rejection_reason\":{}",
            "}}"
        ),
        snapshot.steady_session_limit,
        snapshot.recovery_session_limit,
        snapshot.current_attached_sessions,
        snapshot.peak_attached_sessions,
        snapshot.current_recovery_overlap_sessions,
        snapshot.peak_recovery_overlap_sessions,
        snapshot.current_lingering_sessions,
        snapshot.peak_lingering_sessions,
        snapshot.current_detach_requested_sessions,
        snapshot.current_detach_faulted_sessions,
        snapshot.pending_cleanup_work_items,
        snapshot.pending_deferred_retry_work_items,
        snapshot.next_cleanup_epoch,
        json_option_u64(snapshot.oldest_pending_cleanup_ready_epoch),
        json_pending_lingering_cleanup_wave_summary_vec(&snapshot.pending_cleanup_waves),
        json_active_transport_concurrency_session_vec(&snapshot.active_sessions),
        json_option_string(snapshot.last_admitted_sandbox_id.as_deref()),
        json_option_string(snapshot.last_rejected_sandbox_id.as_deref()),
        json_option_string(snapshot.last_rejection_reason.as_deref()),
    )
}

fn json_transport_session_summary(summary: &TransportSessionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"boundary_mode\":{},",
            "\"current_state\":{},",
            "\"currently_attached\":{},",
            "\"heartbeat_freshness\":{},",
            "\"dispatch_state\":{},",
            "\"current_attached_session_count\":{},",
            "\"max_concurrent_attached_sessions\":{},",
            "\"attach_events\":{},",
            "\"detach_requested_events\":{},",
            "\"detached_events\":{},",
            "\"detach_fault_events\":{},",
            "\"heartbeat_requested_events\":{},",
            "\"heartbeat_responded_events\":{},",
            "\"heartbeat_missed_events\":{},",
            "\"dispatch_requested_events\":{},",
            "\"dispatch_completed_events\":{},",
            "\"dispatch_timed_out_events\":{},",
            "\"first_processing_epoch\":{},",
            "\"last_processing_epoch\":{},",
            "\"first_block_sequence\":{},",
            "\"last_block_sequence\":{},",
            "\"active_sandbox_id\":{},",
            "\"active_lease_id\":{},",
            "\"active_region_id\":{},",
            "\"active_block_sequence\":{},",
            "\"active_sessions\":{},",
            "\"last_sandbox_id\":{},",
            "\"last_lease_id\":{},",
            "\"last_region_id\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", summary.boundary_mode)),
        json_escape_string(&format!("{:?}", summary.current_state)),
        summary.currently_attached,
        json_escape_string(&format!("{:?}", summary.heartbeat_freshness)),
        json_escape_string(&format!("{:?}", summary.dispatch_state)),
        summary.current_attached_session_count,
        summary.max_concurrent_attached_sessions,
        summary.attach_events,
        summary.detach_requested_events,
        summary.detached_events,
        summary.detach_fault_events,
        summary.heartbeat_requested_events,
        summary.heartbeat_responded_events,
        summary.heartbeat_missed_events,
        summary.dispatch_requested_events,
        summary.dispatch_completed_events,
        summary.dispatch_timed_out_events,
        json_option_u64(summary.first_processing_epoch),
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.first_block_sequence),
        json_option_u64(summary.last_block_sequence),
        json_option_string(summary.active_sandbox_id.as_deref()),
        json_option_string(summary.active_lease_id.as_deref()),
        json_option_string(summary.active_region_id.as_deref()),
        json_option_u64(summary.active_block_sequence),
        json_active_transport_session_record_vec(&summary.active_sessions),
        json_option_string(summary.last_sandbox_id.as_deref()),
        json_option_string(summary.last_lease_id.as_deref()),
        json_option_string(summary.last_region_id.as_deref()),
    )
}

fn json_active_transport_concurrency_session(
    session: &ActiveTransportConcurrencySession,
) -> String {
    let last_cleanup_mode = session.last_cleanup_mode.map(|mode| format!("{mode:?}"));
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"intent\":{},",
            "\"provenance\":{},",
            "\"attach_sequence\":{},",
            "\"attach_processing_epoch\":{},",
            "\"state\":{},",
            "\"backing_path\":{},",
            "\"total_bytes\":{},",
            "\"cleanup_attempt_count\":{},",
            "\"last_cleanup_mode\":{},",
            "\"last_cleanup_wave\":{},",
            "\"cleanup_in_progress\":{},",
            "\"last_cleanup_epoch\":{},",
            "\"last_cleanup_error\":{}",
            "}}"
        ),
        json_escape_string(&session.sandbox_id),
        json_escape_string(&session.lease_id),
        json_escape_string(&session.region_id),
        json_escape_string(&format!("{:?}", session.intent)),
        json_escape_string(&format!("{:?}", session.provenance)),
        session.attach_sequence,
        json_option_u64(session.attach_processing_epoch),
        json_escape_string(&format!("{:?}", session.state)),
        json_option_string(session.backing_path.as_deref()),
        json_option_u64(session.total_bytes.map(u64::from)),
        session.cleanup_attempt_count,
        json_option_string(last_cleanup_mode.as_deref()),
        json_option_u64(session.last_cleanup_wave),
        session.cleanup_in_progress,
        json_option_u64(session.last_cleanup_epoch),
        json_option_string(session.last_cleanup_error.as_deref()),
    )
}

fn json_active_transport_concurrency_session_vec(
    sessions: &[ActiveTransportConcurrencySession],
) -> String {
    let joined = sessions
        .iter()
        .map(json_active_transport_concurrency_session)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_pending_lingering_cleanup_wave_summary(
    wave: &PendingLingeringCleanupWaveSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"cleanup_wave\":{},",
            "\"mode\":{},",
            "\"first_trigger\":{},",
            "\"latest_trigger\":{},",
            "\"pending_work_items\":{},",
            "\"deferred_retry_work_items\":{},",
            "\"first_cleanup_epoch\":{},",
            "\"latest_cleanup_epoch\":{},",
            "\"first_processing_epoch\":{},",
            "\"latest_processing_epoch\":{},",
            "\"oldest_ready_at_processing_epoch\":{},",
            "\"newest_ready_at_processing_epoch\":{}",
            "}}"
        ),
        json_escape_string(&wave.sandbox_id),
        wave.cleanup_wave,
        json_escape_string(&format!("{:?}", wave.mode)),
        json_escape_string(&format!("{:?}", wave.first_trigger)),
        json_escape_string(&format!("{:?}", wave.latest_trigger)),
        wave.pending_work_items,
        wave.deferred_retry_work_items,
        wave.first_cleanup_epoch,
        wave.latest_cleanup_epoch,
        wave.first_processing_epoch,
        wave.latest_processing_epoch,
        wave.oldest_ready_at_processing_epoch,
        wave.newest_ready_at_processing_epoch,
    )
}

fn json_pending_lingering_cleanup_wave_summary_vec(
    waves: &[PendingLingeringCleanupWaveSummary],
) -> String {
    let joined = waves
        .iter()
        .map(json_pending_lingering_cleanup_wave_summary)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_active_transport_session_record(record: &ActiveTransportSessionRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"state\":{},",
            "\"currently_attached\":{},",
            "\"heartbeat_freshness\":{},",
            "\"dispatch_state\":{},",
            "\"processing_epoch\":{},",
            "\"active_block_sequence\":{},",
            "\"transport_fault_count\":{},",
            "\"last_transport_fault_source\":{},",
            "\"last_transport_fault_stage\":{},",
            "\"last_transport_fault_phase\":{},",
            "\"last_transport_fault_processing_epoch\":{},",
            "\"last_transport_fault_block_sequence\":{}",
            "}}"
        ),
        json_escape_string(&record.sandbox_id),
        json_escape_string(&record.lease_id),
        json_escape_string(&record.region_id),
        json_escape_string(&format!("{:?}", record.state)),
        record.currently_attached,
        json_escape_string(&format!("{:?}", record.heartbeat_freshness)),
        json_escape_string(&format!("{:?}", record.dispatch_state)),
        json_option_u64(record.processing_epoch),
        json_option_u64(record.active_block_sequence),
        record.transport_fault_count,
        json_option_string(
            record
                .last_transport_fault_source
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            record
                .last_transport_fault_stage
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            record
                .last_transport_fault_phase
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_u64(record.last_transport_fault_processing_epoch),
        json_option_u64(record.last_transport_fault_block_sequence),
    )
}

fn json_active_transport_session_record_vec(records: &[ActiveTransportSessionRecord]) -> String {
    let joined = records
        .iter()
        .map(json_active_transport_session_record)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_recovery_record(record: Option<&RecoveryRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"intent\":{},",
                "\"stop_reason\":{},",
                "\"processing_epoch\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.intent)),
            json_escape_string(&format!("{:?}", record.stop_reason)),
            json_option_u64(record.processing_epoch),
        ),
        None => "null".into(),
    }
}

fn json_plugin_instance_fault_record(record: Option<&PluginSandboxInstanceFaultRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"kind\":{},",
                "\"severity\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_escape_string(record.kind.as_str()),
            json_escape_string(record.severity.as_str()),
            json_escape_string(record.message.as_str()),
        ),
        None => "null".into(),
    }
}

fn json_plugin_instance_state_record(record: Option<&PluginSandboxInstanceStateRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"plugin_type_id\":{},",
                "\"instance_id\":{},",
                "\"lifecycle_state\":{},",
                "\"readiness_state\":{},",
                "\"degraded_reasons\":{},",
                "\"active\":{},",
                "\"processing_epoch\":{},",
                "\"processing_sample_rate_hz\":{},",
                "\"processing_max_block_frames\":{},",
                "\"audio_inputs\":{},",
                "\"audio_outputs\":{},",
                "\"midi_inputs\":{},",
                "\"midi_outputs\":{},",
                "\"last_fault\":{}",
                "}}"
            ),
            json_escape_string(record.sandbox_id.as_str()),
            json_escape_string(record.plugin_type_id.as_str()),
            json_escape_string(record.instance_id.as_str()),
            json_escape_string(record.lifecycle_state.as_str()),
            json_escape_string(record.readiness_state.as_str()),
            format!(
                "[{}]",
                record
                    .degraded_reasons
                    .iter()
                    .map(|reason| json_escape_string(reason.as_str()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            record.active,
            json_option_u64(record.processing_epoch),
            json_option_u64(record.processing_sample_rate_hz.map(u64::from)),
            json_option_u64(record.processing_max_block_frames.map(u64::from)),
            json_option_u64(record.audio_inputs.map(u64::from)),
            json_option_u64(record.audio_outputs.map(u64::from)),
            json_option_u64(record.midi_inputs.map(u64::from)),
            json_option_u64(record.midi_outputs.map(u64::from)),
            json_plugin_instance_fault_record(record.last_fault.as_ref()),
        ),
        None => "null".into(),
    }
}

fn json_plugin_instance_state_record_vec(records: &[PluginSandboxInstanceStateRecord]) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(|record| json_plugin_instance_state_record(Some(record)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_recovery_record_vec(records: &[RecoveryRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_recovery_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_lifecycle_record(record: Option<&PluginSandboxLifecycleRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
        ),
        None => "null".into(),
    }
}

fn json_lifecycle_record_vec(records: &[PluginSandboxLifecycleRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_lifecycle_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_transport_record(record: Option<&PluginSandboxTransportRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"region_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            json_escape_string(&record.region_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
            json_option_string(record.detail.as_deref()),
        ),
        None => "null".into(),
    }
}

fn json_transport_record_vec(records: &[PluginSandboxTransportRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_transport_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_heartbeat_record(record: Option<&HeartbeatCycleRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
        ),
        None => "null".into(),
    }
}

fn json_heartbeat_record_vec(records: &[HeartbeatCycleRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_heartbeat_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_block_dispatch_record(record: Option<&BlockDispatchRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"frame_count\":{},",
                "\"stage\":{},",
                "\"completion_state\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.block_sequence,
            record.frame_count,
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_string(
                record
                    .completion_state
                    .map(|state| format!("{state:?}"))
                    .as_deref()
            ),
        ),
        None => "null".into(),
    }
}

fn json_block_dispatch_record_vec(records: &[BlockDispatchRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_block_dispatch_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_lease_rollover_record(record: Option<&LeaseRolloverRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"previous_lease_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"first_block_sequence\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.previous_lease_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.first_block_sequence,
        ),
        None => "null".into(),
    }
}

fn json_lease_rollover_record_vec(records: &[LeaseRolloverRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_lease_rollover_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_broker_invalidation_record(record: Option<&BrokerInvalidationRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{},",
                "\"reason\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.reason),
        ),
        None => "null".into(),
    }
}

fn json_broker_invalidation_record_vec(records: &[BrokerInvalidationRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_broker_invalidation_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_completion_slot_record(record: Option<&CompletionSlotRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.block_sequence,
            json_escape_string(&format!("{:?}", record.stage)),
        ),
        None => "null".into(),
    }
}

fn json_completion_slot_record_vec(records: &[CompletionSlotRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_completion_slot_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_transport_fault_record(record: Option<&TransportFaultRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"source\":{},",
                "\"stage\":{},",
                "\"phase\":{},",
                "\"resource\":{},",
                "\"operation\":{},",
                "\"error_kind\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.source)),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&format!("{:?}", record.phase)),
            json_escape_string(&format!("{:?}", record.resource)),
            json_escape_string(&record.operation),
            json_option_string(record.error_kind.as_deref()),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

fn json_transport_fault_record_vec(records: &[TransportFaultRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_transport_fault_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_broker_failure_record(record: Option<&BrokerFailureRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

fn json_broker_failure_record_vec(records: &[BrokerFailureRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_broker_failure_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_sandbox_operation_failure_record(record: Option<&SandboxOperationFailureRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"operation\":{},",
                "\"error_kind\":{},",
                "\"stage\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_escape_string(&record.operation),
            json_escape_string(&record.error_kind),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

fn json_sandbox_operation_failure_record_vec(records: &[SandboxOperationFailureRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_sandbox_operation_failure_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

#[derive(Clone, Default)]
pub struct RuntimeEventRecorder {
    events: Arc<Mutex<Vec<RuntimeEvent>>>,
}

impl RuntimeEventRecorder {
    pub fn count(&self) -> usize {
        self.events
            .lock()
            .map(|events| events.len())
            .unwrap_or_default()
    }

    pub fn snapshot(&self) -> Vec<RuntimeEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    pub fn supervision_updates(&self) -> Vec<RuntimeSupervisionSnapshot> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::SupervisionChanged(snapshot) => Some(snapshot),
                _ => None,
            })
            .collect()
    }

    pub fn plugin_faults(&self) -> Vec<PluginFaultRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxFault {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                } => Some(PluginFaultRecord {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn plugin_instance_states(&self) -> Vec<PluginSandboxInstanceStateRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxInstanceState { state } => Some(state),
                _ => None,
            })
            .collect()
    }

    pub fn recovery_events(&self) -> Vec<RecoveryRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::RecoveryCycle {
                    sandbox_id,
                    intent,
                    stop_reason,
                    processing_epoch,
                } => Some(RecoveryRecord {
                    sandbox_id,
                    intent,
                    stop_reason,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn lifecycle_events(&self) -> Vec<PluginSandboxLifecycleRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxLifecycle {
                    sandbox_id,
                    stage,
                    processing_epoch,
                } => Some(PluginSandboxLifecycleRecord {
                    sandbox_id,
                    stage,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn transport_events(&self) -> Vec<PluginSandboxTransportRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxTransport {
                    sandbox_id,
                    lease_id,
                    region_id,
                    stage,
                    processing_epoch,
                    detail,
                } => Some(PluginSandboxTransportRecord {
                    sandbox_id,
                    lease_id,
                    region_id,
                    stage,
                    processing_epoch,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn heartbeat_events(&self) -> Vec<HeartbeatCycleRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::HeartbeatCycle {
                    sandbox_id,
                    stage,
                    processing_epoch,
                    block_sequence,
                } => Some(HeartbeatCycleRecord {
                    sandbox_id,
                    stage,
                    processing_epoch,
                    block_sequence,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn block_dispatch_events(&self) -> Vec<BlockDispatchRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BlockDispatch {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    frame_count,
                    stage,
                    completion_state,
                } => Some(BlockDispatchRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    frame_count,
                    stage,
                    completion_state,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn lease_rollover_events(&self) -> Vec<LeaseRolloverRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::LeaseRollover {
                    sandbox_id,
                    previous_lease_id,
                    lease_id,
                    processing_epoch,
                    first_block_sequence,
                } => Some(LeaseRolloverRecord {
                    sandbox_id,
                    previous_lease_id,
                    lease_id,
                    processing_epoch,
                    first_block_sequence,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn invalidation_events(&self) -> Vec<BrokerInvalidationRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BrokerInvalidation {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    reason,
                } => Some(BrokerInvalidationRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    reason,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn completion_slot_events(&self) -> Vec<CompletionSlotRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::CompletionSlotTransition {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                } => Some(CompletionSlotRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn broker_failure_events(&self) -> Vec<BrokerFailureRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BrokerFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    detail,
                } => Some(BrokerFailureRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn transport_fault_events(&self) -> Vec<TransportFaultRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::BrokerFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    detail,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    source: TransportFaultSource::HostBroker,
                    stage: map_broker_failure_stage(stage),
                    phase: map_broker_failure_phase(stage),
                    resource: map_broker_failure_resource(stage),
                    operation: broker_failure_operation(stage).into(),
                    error_kind: None,
                    detail,
                }),
                RuntimeEvent::SandboxOperationFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    operation,
                    error_kind,
                    stage,
                    detail,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence: None,
                    source: TransportFaultSource::SandboxOperation,
                    stage: map_sandbox_operation_failure_stage(stage),
                    phase: map_sandbox_operation_failure_phase(stage),
                    resource: map_sandbox_operation_failure_resource(stage),
                    operation,
                    error_kind: Some(error_kind),
                    detail,
                }),
                RuntimeEvent::PluginSandboxTransport {
                    sandbox_id,
                    lease_id,
                    stage: PluginSandboxTransportStage::DetachRequested,
                    processing_epoch,
                    detail,
                    ..
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch,
                    block_sequence: None,
                    source: TransportFaultSource::HostBroker,
                    stage: TransportFaultStage::TransportDetachRequested,
                    phase: TransportFaultPhase::Teardown,
                    resource: TransportFaultResource::SharedMemoryLease,
                    operation: "transport.detach_request".into(),
                    error_kind: None,
                    detail: detail.unwrap_or_else(|| "transport detach requested".into()),
                }),
                RuntimeEvent::PluginSandboxTransport {
                    sandbox_id,
                    lease_id,
                    stage: PluginSandboxTransportStage::Detached,
                    processing_epoch,
                    detail,
                    ..
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch,
                    block_sequence: None,
                    source: TransportFaultSource::HostBroker,
                    stage: TransportFaultStage::TransportDetached,
                    phase: TransportFaultPhase::Teardown,
                    resource: TransportFaultResource::SharedMemoryLease,
                    operation: "transport.detached".into(),
                    error_kind: None,
                    detail: detail.unwrap_or_else(|| "transport detached".into()),
                }),
                RuntimeEvent::PluginSandboxTransport {
                    sandbox_id,
                    lease_id,
                    stage: PluginSandboxTransportStage::DetachFault,
                    processing_epoch,
                    detail,
                    ..
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch,
                    block_sequence: None,
                    source: TransportFaultSource::HostBroker,
                    stage: TransportFaultStage::TransportDetachFault,
                    phase: TransportFaultPhase::Teardown,
                    resource: TransportFaultResource::SharedMemoryLease,
                    operation: "transport.detach_fault".into(),
                    error_kind: None,
                    detail: detail.unwrap_or_else(|| "transport detach fault".into()),
                }),
                RuntimeEvent::BrokerInvalidation {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage,
                    reason,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch: Some(processing_epoch),
                    block_sequence,
                    source: TransportFaultSource::RuntimeDispatch,
                    stage: map_broker_invalidation_stage(stage),
                    phase: TransportFaultPhase::Teardown,
                    resource: map_broker_invalidation_resource(stage),
                    operation: broker_invalidation_operation(stage).into(),
                    error_kind: None,
                    detail: reason,
                }),
                RuntimeEvent::CompletionSlotTransition {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage: CompletionSlotStage::TimedOut,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch: Some(processing_epoch),
                    block_sequence: Some(block_sequence),
                    source: TransportFaultSource::RuntimeDispatch,
                    stage: TransportFaultStage::CompletionSlotTimedOut,
                    phase: TransportFaultPhase::Dispatch,
                    resource: TransportFaultResource::CompletionSlot,
                    operation: "completion_slot.timeout".into(),
                    error_kind: None,
                    detail: "completion slot timed out".into(),
                }),
                RuntimeEvent::CompletionSlotTransition {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage: CompletionSlotStage::Invalidated,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch: Some(processing_epoch),
                    block_sequence: Some(block_sequence),
                    source: TransportFaultSource::RuntimeDispatch,
                    stage: TransportFaultStage::CompletionSlotInvalidated,
                    phase: TransportFaultPhase::Dispatch,
                    resource: TransportFaultResource::CompletionSlot,
                    operation: "completion_slot.invalidate".into(),
                    error_kind: None,
                    detail: "completion slot invalidated".into(),
                }),
                RuntimeEvent::CompletionSlotTransition {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    block_sequence,
                    stage: CompletionSlotStage::FallbackApplied,
                } => Some(TransportFaultRecord {
                    sandbox_id,
                    lease_id: Some(lease_id),
                    processing_epoch: Some(processing_epoch),
                    block_sequence: Some(block_sequence),
                    source: TransportFaultSource::RuntimeDispatch,
                    stage: TransportFaultStage::FallbackApplied,
                    phase: TransportFaultPhase::Dispatch,
                    resource: TransportFaultResource::CompletionSlot,
                    operation: "completion_slot.fallback_apply".into(),
                    error_kind: None,
                    detail: "fallback applied after invalidation".into(),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn sandbox_operation_failure_events(&self) -> Vec<SandboxOperationFailureRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::SandboxOperationFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    operation,
                    error_kind,
                    stage,
                    detail,
                } => Some(SandboxOperationFailureRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    operation,
                    error_kind,
                    stage,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn diagnostics(&self) -> RuntimeObservationDiagnostics {
        RuntimeObservationDiagnostics {
            total_events: self.count(),
            supervision_updates: self.supervision_updates(),
            plugin_faults: self.plugin_faults(),
            plugin_instance_states: self.plugin_instance_states(),
            recovery_events: self.recovery_events(),
            lifecycle_events: self.lifecycle_events(),
            transport_events: self.transport_events(),
            heartbeat_events: self.heartbeat_events(),
            block_dispatch_events: self.block_dispatch_events(),
            lease_rollover_events: self.lease_rollover_events(),
            invalidation_events: self.invalidation_events(),
            completion_slot_events: self.completion_slot_events(),
            transport_fault_events: self.transport_fault_events(),
            broker_failure_events: self.broker_failure_events(),
            sandbox_operation_failure_events: self.sandbox_operation_failure_events(),
        }
    }
}

fn map_broker_failure_stage(stage: BrokerFailureStage) -> TransportFaultStage {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultStage::PreparePlanCreate,
        BrokerFailureStage::PayloadWrite => TransportFaultStage::PayloadWrite,
        BrokerFailureStage::PayloadRead => TransportFaultStage::PayloadRead,
        BrokerFailureStage::TransportDestroy => TransportFaultStage::TransportDestroy,
        BrokerFailureStage::TransportTeardown => TransportFaultStage::TransportTeardown,
    }
}

fn map_broker_failure_phase(stage: BrokerFailureStage) -> TransportFaultPhase {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultPhase::Prepare,
        BrokerFailureStage::PayloadWrite | BrokerFailureStage::PayloadRead => {
            TransportFaultPhase::Dispatch
        }
        BrokerFailureStage::TransportDestroy | BrokerFailureStage::TransportTeardown => {
            TransportFaultPhase::Teardown
        }
    }
}

fn map_broker_failure_resource(stage: BrokerFailureStage) -> TransportFaultResource {
    match stage {
        BrokerFailureStage::PreparePlanCreate => TransportFaultResource::PreparePlan,
        BrokerFailureStage::PayloadWrite | BrokerFailureStage::PayloadRead => {
            TransportFaultResource::SharedMemoryPayload
        }
        BrokerFailureStage::TransportDestroy | BrokerFailureStage::TransportTeardown => {
            TransportFaultResource::SharedMemoryLease
        }
    }
}

fn broker_failure_operation(stage: BrokerFailureStage) -> &'static str {
    match stage {
        BrokerFailureStage::PreparePlanCreate => "prepare_plan.create",
        BrokerFailureStage::PayloadWrite => "block_payload.write",
        BrokerFailureStage::PayloadRead => "block_payload.read",
        BrokerFailureStage::TransportDestroy => "lease.destroy_region",
        BrokerFailureStage::TransportTeardown => "lease.teardown_transport",
    }
}

fn map_broker_invalidation_stage(stage: BrokerInvalidationStage) -> TransportFaultStage {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => {
            TransportFaultStage::CompletionRegionInvalidated
        }
        BrokerInvalidationStage::LeaseEpochInvalidated => {
            TransportFaultStage::LeaseEpochInvalidated
        }
    }
}

fn map_broker_invalidation_resource(stage: BrokerInvalidationStage) -> TransportFaultResource {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => {
            TransportFaultResource::SharedMemoryPayload
        }
        BrokerInvalidationStage::LeaseEpochInvalidated => TransportFaultResource::SharedMemoryLease,
    }
}

fn broker_invalidation_operation(stage: BrokerInvalidationStage) -> &'static str {
    match stage {
        BrokerInvalidationStage::CompletionRegionInvalidated => "completion_region.invalidate",
        BrokerInvalidationStage::LeaseEpochInvalidated => "lease_epoch.invalidate",
    }
}

fn map_sandbox_operation_failure_stage(stage: SandboxOperationFailureStage) -> TransportFaultStage {
    match stage {
        SandboxOperationFailureStage::PrepareAttach => TransportFaultStage::PrepareAttach,
        SandboxOperationFailureStage::ProcessAttach => TransportFaultStage::ProcessAttach,
        SandboxOperationFailureStage::ProcessFlush => TransportFaultStage::ProcessFlush,
        SandboxOperationFailureStage::ProcessProtocolViolation => {
            TransportFaultStage::ProcessProtocolViolation
        }
        SandboxOperationFailureStage::ControlProtocolViolation => {
            TransportFaultStage::ControlProtocolViolation
        }
    }
}

fn map_sandbox_operation_failure_phase(stage: SandboxOperationFailureStage) -> TransportFaultPhase {
    match stage {
        SandboxOperationFailureStage::PrepareAttach => TransportFaultPhase::Prepare,
        SandboxOperationFailureStage::ProcessAttach
        | SandboxOperationFailureStage::ProcessFlush
        | SandboxOperationFailureStage::ProcessProtocolViolation => TransportFaultPhase::Dispatch,
        SandboxOperationFailureStage::ControlProtocolViolation => TransportFaultPhase::Control,
    }
}

fn map_sandbox_operation_failure_resource(
    stage: SandboxOperationFailureStage,
) -> TransportFaultResource {
    match stage {
        SandboxOperationFailureStage::PrepareAttach
        | SandboxOperationFailureStage::ProcessAttach => TransportFaultResource::SharedMemoryLease,
        SandboxOperationFailureStage::ProcessFlush => TransportFaultResource::SharedMemoryPayload,
        SandboxOperationFailureStage::ProcessProtocolViolation => {
            TransportFaultResource::ProcessProtocol
        }
        SandboxOperationFailureStage::ControlProtocolViolation => {
            TransportFaultResource::ControlProtocol
        }
    }
}

impl RuntimeEventSink for RuntimeEventRecorder {
    fn push(&mut self, event: RuntimeEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginScanRequest {
    pub roots: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxSpec {
    pub sandbox_id: String,
    pub plugin_format: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxHandle(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyOverride {
    pub tier: BackendPolicyTier,
}

pub trait RuntimeLifecycleApi {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError>;
    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError>;
    fn start(&mut self) -> Result<(), RuntimeError>;
    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError>;
    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError>;
    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError>;
}

pub trait RuntimeProjectionApi {
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError>;
    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError>;
    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError>;
    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError>;
    fn apply_hardware_config(&mut self, request: HardwareConfigRequest)
        -> Result<(), RuntimeError>;
}

pub trait RuntimeObservationApi {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle;
    fn get_readiness(&self) -> RuntimeReadiness;
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    fn get_scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot;
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot;
    fn get_transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot;
    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot;
    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot;
    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot;
    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot;
    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot;
    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot;
}

pub trait RuntimeSupervisorApi {
    fn start_plugin_scan(&mut self, request: PluginScanRequest)
        -> Result<ScanHandle, RuntimeError>;
    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<SandboxHandle, RuntimeError>;
    fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError>;
    fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError>;
    fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError>;
    fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError>;
    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError>;
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}
