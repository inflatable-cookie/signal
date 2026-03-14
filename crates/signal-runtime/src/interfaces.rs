//! Typed runtime-host interfaces for embedded Signal assemblies.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use signal_graph::{
    GraphChannelAdaptationMode, GraphDynamicStageStateModel, GraphExecutionContext,
    GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, BackendPolicyTier, HardwareClockSource,
    HardwareClockTopology, HardwareConfigRequest, HardwareLifecycleOwnership,
    HardwareRestartPolicy,
};
use signal_plugin::{
    BlockSequenceContinuityReport, CompletionState, PluginFeature, PluginFormat, PluginIoLayout,
    PluginLifecycleContract, PluginProcessingContract, PluginStateContract,
};
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
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginIsolationOutcome {
    InProcess,
    SharedSandbox,
    #[default]
    IsolatedSandbox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginPlacementRuleMatcher {
    Any,
    PluginFormat(PluginFormat),
    PluginTypeId(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementRule {
    pub rule_id: String,
    pub matcher: RuntimePluginPlacementRuleMatcher,
    pub outcome: RuntimePluginIsolationOutcome,
    pub sandbox_group_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementPolicy {
    pub default_outcome: RuntimePluginIsolationOutcome,
    pub rules: Vec<RuntimePluginPlacementRule>,
}

impl Default for RuntimePluginPlacementPolicy {
    fn default() -> Self {
        Self {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: Vec::new(),
        }
    }
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
    MissingSendReturnIds {
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBlockDeadlinePressure {
    #[default]
    Normal,
    Elevated,
    Critical,
    Overrun,
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
    pub parameter_epoch: Option<u64>,
    pub parameter_event_count: usize,
    pub parameter_targeted_node_count: usize,
    pub parameter_ignored_event_count: usize,
    pub parameter_sub_block_count: usize,
    pub parameter_coalesced_event_count: usize,
    pub processed_blocks: u64,
    pub last_processing_epoch: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub last_frame_count: usize,
    pub last_channel_count: usize,
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
    pub peak_block_execution_time_ns: u64,
    pub peak_block_budget_utilization_percent: f32,
    pub peak_block_budget_overrun_ns: u64,
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
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub plugin_sandbox_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEngineBlockResult {
    pub snapshot: RuntimeEngineBlockSnapshot,
    pub output: AudioBuffer,
    pub meter_sources: Vec<RuntimeMeterSourceSnapshot>,
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
    pub sample_offset: usize,
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
pub enum RuntimeAutomationInterpolation {
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationResolution {
    pub ramp_step_samples: usize,
    pub max_sub_blocks: usize,
}

impl Default for RuntimeAutomationResolution {
    fn default() -> Self {
        Self {
            ramp_step_samples: 32,
            max_sub_blocks: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationTargetProjection {
    pub node_id: String,
    pub parameter_id: String,
}

impl RuntimeAutomationTargetProjection {
    pub fn parameter_path(&self) -> String {
        format!("{}.{}", self.node_id, self.parameter_id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationPointProjection {
    pub time_samples: i64,
    pub normalized_value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationLaneProjection {
    pub automation_lane_id: String,
    pub target: RuntimeAutomationTargetProjection,
    pub base_normalized_value: f32,
    pub interpolation: RuntimeAutomationInterpolation,
    pub resolution: RuntimeAutomationResolution,
    pub point_count: usize,
    pub points: Vec<RuntimeAutomationPointProjection>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationProjection {
    pub lane_count: usize,
    pub point_count: usize,
    pub lanes: Vec<RuntimeAutomationLaneProjection>,
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
    pub xrun_overload_active: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureKind {
    Audio,
    Midi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureCheckpointClass {
    Armed,
    Streaming,
    Buffered,
    Committed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCheckpointSnapshot {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub checkpoint_class: RuntimeRecordingCaptureCheckpointClass,
    pub interruption_class: RuntimeInterruptionClass,
    pub take_id: String,
    pub track_id: String,
    pub capture_start_samples: i64,
    pub capture_path: String,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordingCaptureStartRequest {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub capture_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCommitReceipt {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub channel_count: usize,
    pub peak_level: f32,
    pub capture_path: String,
    pub committed_checkpoint: RuntimeRecordingCaptureCheckpointSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRecordingCaptureSnapshot {
    pub capture_ready: bool,
    pub state: Option<RuntimeRecordingCaptureState>,
    pub capture_kind: Option<RuntimeRecordingCaptureKind>,
    pub active_take_id: Option<String>,
    pub active_track_id: Option<String>,
    pub capture_start_samples: Option<i64>,
    pub active_capture_path: Option<String>,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub active_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    pub last_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaIndexingState {
    #[default]
    Empty,
    Syncing,
    Ready,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaPreviewState {
    #[default]
    Unavailable,
    Ready,
    Previewing,
    Invalidated,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaServiceSnapshot {
    pub indexed_asset_count: usize,
    pub analysis_ready_asset_count: usize,
    pub waveform_ready_asset_count: usize,
    pub waveform_pending_asset_count: usize,
    pub previewable_asset_count: usize,
    pub invalidated_asset_count: usize,
    pub invalidation_active: bool,
    pub indexing_state: RuntimeMediaIndexingState,
    pub preview_state: RuntimeMediaPreviewState,
    pub previewing_asset_id: Option<String>,
    pub last_invalidated_asset_id: Option<String>,
    pub last_invalidation_error: Option<String>,
    pub last_preview_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpMode {
    Off,
    Repitch,
    ElastiqueDraft,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoMapInterpolation {
    #[default]
    Hold,
    Linear,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentProjection {
    pub segment_id: String,
    pub start_samples: i64,
    pub end_samples: Option<i64>,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: Option<f64>,
    pub interpolation: RuntimeTempoMapInterpolation,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapProjection {
    pub segment_count: usize,
    pub segments: Vec<RuntimeTempoMapSegmentProjection>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoSource {
    #[default]
    DefaultFallback,
    TransportProjection,
    TempoMapSegment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentSnapshot {
    pub segment_id: String,
    pub start_samples: i64,
    pub end_samples: Option<i64>,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: Option<f64>,
    pub interpolation: RuntimeTempoMapInterpolation,
    pub covers_timeline_position: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapSnapshot {
    pub segment_count: usize,
    pub active_segment_id: Option<String>,
    pub active_segment_index: Option<usize>,
    pub next_segment_start_samples: Option<i64>,
    pub resolved_tempo_bpm: f64,
    pub tempo_source: RuntimeTempoSource,
    pub timeline_position_samples: Option<i64>,
    pub segments: Vec<RuntimeTempoMapSegmentSnapshot>,
    pub summary: String,
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
    pub project_tempo_source: RuntimeTempoSource,
    pub project_tempo_segment_id: Option<String>,
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
    pub resolved_project_tempo_bpm: f64,
    pub resolved_project_tempo_source: RuntimeTempoSource,
    pub resolved_project_tempo_segment_id: Option<String>,
    pub clips: Vec<RuntimeWarpClipSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingReadiness {
    Ready,
    PendingMedia,
    PendingWarp,
    Invalid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipFadeShape {
    #[default]
    Linear,
    EqualPower,
    SmoothStep,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipGainShape {
    #[default]
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingStage {
    Warp,
    FadeIn,
    GainShape,
    FadeOut,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipFadeEnvelope {
    pub duration_samples: u32,
    pub shape: RuntimeClipFadeShape,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipGainEnvelope {
    pub start_linear: f32,
    pub end_linear: f32,
    pub shape: RuntimeClipGainShape,
}

impl Default for RuntimeClipGainEnvelope {
    fn default() -> Self {
        Self {
            start_linear: 1.0,
            end_linear: 1.0,
            shape: RuntimeClipGainShape::Hold,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingRegistration {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub warp_mode: RuntimeWarpMode,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub fade_in: RuntimeClipFadeEnvelope,
    pub fade_out: RuntimeClipFadeEnvelope,
    pub clip_gain: RuntimeClipGainEnvelope,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub warp_mode: RuntimeWarpMode,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub fade_in: RuntimeClipFadeEnvelope,
    pub fade_out: RuntimeClipFadeEnvelope,
    pub fade_in_end_samples: i64,
    pub fade_out_start_samples: i64,
    pub clip_gain: RuntimeClipGainEnvelope,
    pub treatment_stages: Vec<RuntimeClipProcessingStage>,
    pub realized_warp_ratio: Option<f64>,
    pub project_tempo_source: Option<RuntimeTempoSource>,
    pub project_tempo_segment_id: Option<String>,
    pub readiness: RuntimeClipProcessingReadiness,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipProcessingPipelineSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_media_clip_count: usize,
    pub pending_warp_clip_count: usize,
    pub invalid_clip_count: usize,
    pub faded_clip_count: usize,
    pub gain_shaped_clip_count: usize,
    pub warped_clip_count: usize,
    pub treatment_stage_count: usize,
    pub clips: Vec<RuntimeClipProcessingSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipRenderInputStage {
    RawClip,
    #[default]
    PostWarp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderRequest {
    pub clip_id: String,
    pub timeline_start_samples: i64,
    pub input_stage: RuntimeClipRenderInputStage,
    pub buffer: AudioBuffer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderResult {
    pub clip_id: String,
    pub timeline_start_samples: i64,
    pub timeline_end_samples: i64,
    pub input_stage: RuntimeClipRenderInputStage,
    pub clip_processing_snapshot: RuntimeClipProcessingSnapshot,
    pub first_frame_gain: Option<f32>,
    pub last_frame_gain: Option<f32>,
    pub peak_applied_gain: Option<f32>,
    pub output: AudioBuffer,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderTargetKind {
    MainMix,
    TrackLane,
    BusGroup,
    ConsoleGroup,
    SendReturn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemTarget {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactRequest {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_selection: RuntimePluginRecallHandoffSelection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderRequest {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub export_sample_rate_hz: u32,
    pub include_main_mix: bool,
    pub artifact_root_path: Option<String>,
    pub stem_targets: Vec<RuntimeOfflineRenderStemTarget>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemPreview {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
    pub resolved_node_ids: Vec<String>,
    pub resolved_output_bus_ids: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactPreview {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_stage_count: usize,
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    pub recall_states: Vec<RuntimePluginRecallState>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderChainDependencyPreview {
    pub chain_count: usize,
    pub stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub recall_stage_count: usize,
    pub unbound_recall_stage_count: usize,
    pub cold_recall_stage_count: usize,
    pub warm_recall_stage_count: usize,
    pub recovered_recall_stage_count: usize,
    pub unavailable_recall_stage_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderContractPreview {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub timeline_end_samples: i64,
    pub duration_samples: u32,
    pub export_sample_rate_hz: u32,
    pub include_main_mix: bool,
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub stem_count: usize,
    pub freeze_artifact_count: usize,
    pub resolved_tempo_bpm: f64,
    pub resolved_tempo_source: RuntimeTempoSource,
    pub chain_contract: RuntimeOfflineRenderChainDependencyPreview,
    pub stem_targets: Vec<RuntimeOfflineRenderStemPreview>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactPreview>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderStemResult {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
    pub output: AudioBuffer,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineFreezeArtifactResult {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_stage_count: usize,
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    pub recall_states: Vec<RuntimePluginRecallState>,
    pub output: AudioBuffer,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderResult {
    pub request_id: String,
    pub runtime_frame_count: usize,
    pub rendered_frame_count: usize,
    pub block_count: usize,
    pub export_sample_rate_hz: u32,
    pub main_mix: Option<AudioBuffer>,
    pub main_mix_peak_level: Option<f32>,
    pub main_mix_rms_level: Option<f32>,
    pub stems: Vec<RuntimeOfflineRenderStemResult>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactResult>,
    pub manifest: RuntimeOfflineRenderManifest,
    pub plugin_execution_boundary: RuntimeOfflinePluginExecutionBoundary,
    pub contract_preview: RuntimeOfflineRenderContractPreview,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceClass {
    OfflineRenderQueue,
    OfflineRenderPurge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceDecision {
    Run,
    Defer,
    Throttle,
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceReason {
    Ready,
    RealtimeActive,
    PendingCleanup,
    RecoveryDegraded,
    SafeMode,
    InvalidRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeferredServiceReceipt {
    pub work_class: RuntimeDeferredServiceClass,
    pub decision: RuntimeDeferredServiceDecision,
    pub reason: RuntimeDeferredServiceReason,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub queued_work_item_count: usize,
    pub admitted_work_item_count: usize,
    pub completed_work_item_count: usize,
    pub deferred_work_item_count: usize,
    pub runtime_running: bool,
    pub safe_mode_enabled: bool,
    pub readiness_degraded: bool,
    pub pending_cleanup_work_items: usize,
    pub pending_deferred_retry_work_items: usize,
    pub recovery_overlap_session_count: usize,
    pub summary: String,
}

impl RuntimeDeferredServiceReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "work_class={:?}",
                "\ndecision={:?}",
                "\nreason={:?}",
                "\ninterruption_class={:?}",
                "\ninterruption_rebindable={}",
                "\nqueued_work_item_count={}",
                "\nadmitted_work_item_count={}",
                "\ncompleted_work_item_count={}",
                "\ndeferred_work_item_count={}",
                "\nruntime_running={}",
                "\nsafe_mode_enabled={}",
                "\nreadiness_degraded={}",
                "\npending_cleanup_work_items={}",
                "\npending_deferred_retry_work_items={}",
                "\nrecovery_overlap_session_count={}",
                "\nsummary={}",
            ),
            self.work_class,
            self.decision,
            self.reason,
            self.interruption_class,
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"work_class\":{},",
                "\"decision\":{},",
                "\"reason\":{},",
                "\"interruption_class\":{},",
                "\"interruption_rebindable\":{},",
                "\"queued_work_item_count\":{},",
                "\"admitted_work_item_count\":{},",
                "\"completed_work_item_count\":{},",
                "\"deferred_work_item_count\":{},",
                "\"runtime_running\":{},",
                "\"safe_mode_enabled\":{},",
                "\"readiness_degraded\":{},",
                "\"pending_cleanup_work_items\":{},",
                "\"pending_deferred_retry_work_items\":{},",
                "\"recovery_overlap_session_count\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&format!("{:?}", self.work_class)),
            json_string(&format!("{:?}", self.decision)),
            json_string(&format!("{:?}", self.reason)),
            json_string(&format!("{:?}", self.interruption_class)),
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl Default for RuntimeDeferredServiceReceipt {
    fn default() -> Self {
        Self {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision: RuntimeDeferredServiceDecision::Abort,
            reason: RuntimeDeferredServiceReason::InvalidRequest,
            interruption_class: RuntimeInterruptionClass::Terminal,
            interruption_rebindable: false,
            queued_work_item_count: 0,
            admitted_work_item_count: 0,
            completed_work_item_count: 0,
            deferred_work_item_count: 0,
            runtime_running: false,
            safe_mode_enabled: false,
            readiness_degraded: false,
            pending_cleanup_work_items: 0,
            pending_deferred_retry_work_items: 0,
            recovery_overlap_session_count: 0,
            summary: "class=OfflineRenderQueue decision=Abort reason=InvalidRequest queued=0 admitted=0 completed=0 deferred=0".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderQueueProgressReceipt {
    pub request_id: String,
    pub queue_index: usize,
    pub queue_count: usize,
    pub completed_job_count: usize,
    pub progress_percent: u8,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderCheckpointStage {
    PreparingInput,
    RenderingGraph,
    MaterializingOutputs,
    FinalizingArtifacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderCheckpointReceipt {
    pub request_id: String,
    pub stage: RuntimeOfflineRenderCheckpointStage,
    pub checkpoint_index: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub total_frame_count: usize,
    pub rendered_block_count: usize,
    pub total_block_count: usize,
    pub progress_percent: u8,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionReceipt {
    pub request_id: String,
    pub checkpoint_count: usize,
    pub checkpoints: Vec<RuntimeOfflineRenderCheckpointReceipt>,
    pub result: RuntimeOfflineRenderResult,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderExecutionState {
    Running,
    Paused,
    Recoverable,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionProgressReceipt {
    pub request_id: String,
    pub state: RuntimeOfflineRenderExecutionState,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub emitted_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub result: Option<RuntimeOfflineRenderResult>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderExecutionCancellationReceipt {
    pub request_id: String,
    pub cancelled_after_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub rendered_block_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderSessionStateSnapshot {
    pub request_id: String,
    pub state: RuntimeOfflineRenderExecutionState,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub interruption_count: usize,
    pub emitted_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub total_frame_count: usize,
    pub rendered_block_count: usize,
    pub total_block_count: usize,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
    pub materialized: bool,
    pub artifact_count: usize,
    pub report_materialized: bool,
    pub active_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub last_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderSessionSnapshot {
    pub active_session_count: usize,
    pub paused_session_count: usize,
    pub recoverable_session_count: usize,
    pub active_sessions: Vec<RuntimeOfflineRenderSessionStateSnapshot>,
    pub last_session: Option<RuntimeOfflineRenderSessionStateSnapshot>,
    pub last_cancellation: Option<RuntimeOfflineRenderExecutionCancellationReceipt>,
    pub last_purge: Option<RuntimeOfflineRenderPurgeReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderQueueResult {
    pub queue_count: usize,
    pub completed_job_count: usize,
    pub orchestration: RuntimeDeferredServiceReceipt,
    pub progress: Vec<RuntimeOfflineRenderQueueProgressReceipt>,
    pub results: Vec<RuntimeOfflineRenderResult>,
    pub deferred_requests: Vec<RuntimeOfflineRenderRequest>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeRequest {
    pub request_id: String,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeReceipt {
    pub request_id: String,
    pub orchestration: RuntimeDeferredServiceReceipt,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
    pub purged_artifact_root: bool,
    pub purged_artifact_file_count: usize,
    pub purged_artifact_byte_count: u64,
    pub purged_report: bool,
    pub purged_report_byte_count: u64,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderProfilingReceipt {
    pub request_id: String,
    pub runtime_frame_count: usize,
    pub rendered_frame_count: usize,
    pub block_count: usize,
    pub export_sample_rate_hz: u32,
    pub stem_count: usize,
    pub freeze_artifact_count: usize,
    pub main_mix_peak_level: Option<f32>,
    pub main_mix_rms_level: Option<f32>,
    pub chain_stage_count: usize,
    pub chain_degraded_stage_count: usize,
    pub chain_missing_binding_stage_count: usize,
    pub chain_total_planned_latency_samples: u32,
    pub chain_total_realized_latency_samples: u32,
    pub chain_total_tail_samples: u32,
    pub delegated_stage_count: usize,
    pub fresh_override_stage_count: usize,
    pub stale_override_stage_count: usize,
    pub artifact_count: usize,
    pub report_materialized: bool,
    pub summary: String,
}

impl RuntimeOfflineRenderProfilingReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "request_id={}",
                "\nruntime_frame_count={}",
                "\nrendered_frame_count={}",
                "\nblock_count={}",
                "\nexport_sample_rate_hz={}",
                "\nstem_count={}",
                "\nfreeze_artifact_count={}",
                "\nmain_mix_peak_level={:?}",
                "\nmain_mix_rms_level={:?}",
                "\nchain_stage_count={}",
                "\nchain_degraded_stage_count={}",
                "\nchain_missing_binding_stage_count={}",
                "\nchain_total_planned_latency_samples={}",
                "\nchain_total_realized_latency_samples={}",
                "\nchain_total_tail_samples={}",
                "\ndelegated_stage_count={}",
                "\nfresh_override_stage_count={}",
                "\nstale_override_stage_count={}",
                "\nartifact_count={}",
                "\nreport_materialized={}",
                "\nsummary={}",
            ),
            self.request_id,
            self.runtime_frame_count,
            self.rendered_frame_count,
            self.block_count,
            self.export_sample_rate_hz,
            self.stem_count,
            self.freeze_artifact_count,
            self.main_mix_peak_level,
            self.main_mix_rms_level,
            self.chain_stage_count,
            self.chain_degraded_stage_count,
            self.chain_missing_binding_stage_count,
            self.chain_total_planned_latency_samples,
            self.chain_total_realized_latency_samples,
            self.chain_total_tail_samples,
            self.delegated_stage_count,
            self.fresh_override_stage_count,
            self.stale_override_stage_count,
            self.artifact_count,
            self.report_materialized,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"runtime_frame_count\":{},",
                "\"rendered_frame_count\":{},",
                "\"block_count\":{},",
                "\"export_sample_rate_hz\":{},",
                "\"stem_count\":{},",
                "\"freeze_artifact_count\":{},",
                "\"main_mix_peak_level\":{},",
                "\"main_mix_rms_level\":{},",
                "\"chain_stage_count\":{},",
                "\"chain_degraded_stage_count\":{},",
                "\"chain_missing_binding_stage_count\":{},",
                "\"chain_total_planned_latency_samples\":{},",
                "\"chain_total_realized_latency_samples\":{},",
                "\"chain_total_tail_samples\":{},",
                "\"delegated_stage_count\":{},",
                "\"fresh_override_stage_count\":{},",
                "\"stale_override_stage_count\":{},",
                "\"artifact_count\":{},",
                "\"report_materialized\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&self.request_id),
            self.runtime_frame_count,
            self.rendered_frame_count,
            self.block_count,
            self.export_sample_rate_hz,
            self.stem_count,
            self.freeze_artifact_count,
            json_option_f32(self.main_mix_peak_level),
            json_option_f32(self.main_mix_rms_level),
            self.chain_stage_count,
            self.chain_degraded_stage_count,
            self.chain_missing_binding_stage_count,
            self.chain_total_planned_latency_samples,
            self.chain_total_realized_latency_samples,
            self.chain_total_tail_samples,
            self.delegated_stage_count,
            self.fresh_override_stage_count,
            self.stale_override_stage_count,
            self.artifact_count,
            self.report_materialized,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderSoakReceipt {
    pub request_id: String,
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub freeze_artifact_count: usize,
    pub recall_stage_count: usize,
    pub recovered_recall_stage_count: usize,
    pub unavailable_recall_stage_count: usize,
    pub delegated_stage_count: usize,
    pub delegated_completed_stage_count: usize,
    pub delegated_rejected_stage_count: usize,
    pub delegated_unavailable_stage_count: usize,
    pub materialized_artifact_count: usize,
    pub report_materialized: bool,
    pub summary: String,
}

impl RuntimeOfflineRenderSoakReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "request_id={}",
                "\nclip_count={}",
                "\nready_clip_count={}",
                "\nfreeze_artifact_count={}",
                "\nrecall_stage_count={}",
                "\nrecovered_recall_stage_count={}",
                "\nunavailable_recall_stage_count={}",
                "\ndelegated_stage_count={}",
                "\ndelegated_completed_stage_count={}",
                "\ndelegated_rejected_stage_count={}",
                "\ndelegated_unavailable_stage_count={}",
                "\nmaterialized_artifact_count={}",
                "\nreport_materialized={}",
                "\nsummary={}",
            ),
            self.request_id,
            self.clip_count,
            self.ready_clip_count,
            self.freeze_artifact_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            self.delegated_stage_count,
            self.delegated_completed_stage_count,
            self.delegated_rejected_stage_count,
            self.delegated_unavailable_stage_count,
            self.materialized_artifact_count,
            self.report_materialized,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"clip_count\":{},",
                "\"ready_clip_count\":{},",
                "\"freeze_artifact_count\":{},",
                "\"recall_stage_count\":{},",
                "\"recovered_recall_stage_count\":{},",
                "\"unavailable_recall_stage_count\":{},",
                "\"delegated_stage_count\":{},",
                "\"delegated_completed_stage_count\":{},",
                "\"delegated_rejected_stage_count\":{},",
                "\"delegated_unavailable_stage_count\":{},",
                "\"materialized_artifact_count\":{},",
                "\"report_materialized\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&self.request_id),
            self.clip_count,
            self.ready_clip_count,
            self.freeze_artifact_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            self.delegated_stage_count,
            self.delegated_completed_stage_count,
            self.delegated_rejected_stage_count,
            self.delegated_unavailable_stage_count,
            self.materialized_artifact_count,
            self.report_materialized,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl RuntimeOfflineRenderResult {
    pub fn profiling_receipt(&self) -> RuntimeOfflineRenderProfilingReceipt {
        RuntimeOfflineRenderProfilingReceipt {
            request_id: self.request_id.clone(),
            runtime_frame_count: self.runtime_frame_count,
            rendered_frame_count: self.rendered_frame_count,
            block_count: self.block_count,
            export_sample_rate_hz: self.export_sample_rate_hz,
            stem_count: self.stems.len(),
            freeze_artifact_count: self.freeze_artifacts.len(),
            main_mix_peak_level: self.main_mix_peak_level,
            main_mix_rms_level: self.main_mix_rms_level,
            chain_stage_count: self.contract_preview.chain_contract.stage_count,
            chain_degraded_stage_count: self.contract_preview.chain_contract.degraded_stage_count,
            chain_missing_binding_stage_count: self
                .contract_preview
                .chain_contract
                .missing_binding_stage_count,
            chain_total_planned_latency_samples: self
                .contract_preview
                .chain_contract
                .total_planned_latency_samples,
            chain_total_realized_latency_samples: self
                .contract_preview
                .chain_contract
                .total_realized_latency_samples,
            chain_total_tail_samples: self.contract_preview.chain_contract.total_tail_samples,
            delegated_stage_count: self.plugin_execution_boundary.host_delegate_stage_count,
            fresh_override_stage_count: self.plugin_execution_boundary.fresh_override_stage_count,
            stale_override_stage_count: self.plugin_execution_boundary.stale_override_stage_count,
            artifact_count: self.manifest.artifact_count,
            report_materialized: self.manifest.report.is_some(),
            summary: format!(
                "request={} runtime_frames={} rendered_frames={} blocks={} export_rate={} stems={} freeze_artifacts={} chain={}/degraded={}/missing={} delegated={} overrides={}/{} artifacts={} report={}",
                self.request_id,
                self.runtime_frame_count,
                self.rendered_frame_count,
                self.block_count,
                self.export_sample_rate_hz,
                self.stems.len(),
                self.freeze_artifacts.len(),
                self.contract_preview.chain_contract.stage_count,
                self.contract_preview.chain_contract.degraded_stage_count,
                self.contract_preview.chain_contract.missing_binding_stage_count,
                self.plugin_execution_boundary.host_delegate_stage_count,
                self.plugin_execution_boundary.fresh_override_stage_count,
                self.plugin_execution_boundary.stale_override_stage_count,
                self.manifest.artifact_count,
                self.manifest.report.is_some(),
            ),
        }
    }

    pub fn soak_receipt(&self) -> RuntimeOfflineRenderSoakReceipt {
        let delegated_receipt = self.manifest.delegated_execution_receipt.as_ref();
        RuntimeOfflineRenderSoakReceipt {
            request_id: self.request_id.clone(),
            clip_count: self.contract_preview.clip_count,
            ready_clip_count: self.contract_preview.ready_clip_count,
            freeze_artifact_count: self.freeze_artifacts.len(),
            recall_stage_count: self.contract_preview.chain_contract.recall_stage_count,
            recovered_recall_stage_count: self
                .contract_preview
                .chain_contract
                .recovered_recall_stage_count,
            unavailable_recall_stage_count: self
                .contract_preview
                .chain_contract
                .unavailable_recall_stage_count,
            delegated_stage_count: self.plugin_execution_boundary.host_delegate_stage_count,
            delegated_completed_stage_count: delegated_receipt
                .map(|receipt| receipt.completed_stage_count)
                .unwrap_or(0),
            delegated_rejected_stage_count: delegated_receipt
                .map(|receipt| receipt.rejected_stage_count)
                .unwrap_or(0),
            delegated_unavailable_stage_count: delegated_receipt
                .map(|receipt| receipt.unavailable_stage_count)
                .unwrap_or(0),
            materialized_artifact_count: self.manifest.artifact_count,
            report_materialized: self.manifest.report.is_some(),
            summary: format!(
                "request={} clips={}/{} freeze_artifacts={} recall={}/recovered={}/unavailable={} delegated={}/{}/{}/{} artifacts={} report={}",
                self.request_id,
                self.contract_preview.ready_clip_count,
                self.contract_preview.clip_count,
                self.freeze_artifacts.len(),
                self.contract_preview.chain_contract.recall_stage_count,
                self.contract_preview.chain_contract.recovered_recall_stage_count,
                self.contract_preview.chain_contract.unavailable_recall_stage_count,
                self.plugin_execution_boundary.host_delegate_stage_count,
                delegated_receipt
                    .map(|receipt| receipt.completed_stage_count)
                    .unwrap_or(0),
                delegated_receipt
                    .map(|receipt| receipt.rejected_stage_count)
                    .unwrap_or(0),
                delegated_receipt
                    .map(|receipt| receipt.unavailable_stage_count)
                    .unwrap_or(0),
                self.manifest.artifact_count,
                self.manifest.report.is_some(),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderArtifactKind {
    MainMix,
    Stem,
    FreezeArtifact,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderArtifactReceipt {
    pub artifact_id: String,
    pub artifact_kind: RuntimeOfflineRenderArtifactKind,
    pub output_path: String,
    pub sample_rate_hz: u32,
    pub channel_count: usize,
    pub frame_count: usize,
    pub byte_size: u64,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderReportReceipt {
    pub request_id: String,
    pub report_path: String,
    pub artifact_count: usize,
    pub byte_size: u64,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderManifest {
    pub request_id: String,
    pub artifact_root_path: Option<String>,
    pub materialized: bool,
    pub artifact_count: usize,
    pub artifacts: Vec<RuntimeOfflineRenderArtifactReceipt>,
    pub report: Option<RuntimeOfflineRenderReportReceipt>,
    pub delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    pub delegated_execution_receipt: Option<RuntimeOfflinePluginDelegatedExecutionReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageRequest {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
    pub override_state: RuntimeOfflinePluginOverrideState,
    pub latest_override_processing_epoch: Option<u64>,
    pub latest_override_block_sequence: Option<u64>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionRequest {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub runtime_sample_rate_hz: u32,
    pub export_sample_rate_hz: u32,
    pub block_size: usize,
    pub block_count: usize,
    pub stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageRequest>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginDelegatedExecutionStatus {
    #[default]
    Completed,
    Rejected,
    Unavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageReceipt {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub status: RuntimeOfflinePluginDelegatedExecutionStatus,
    pub delegate_label: Option<String>,
    pub detail: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionReceipt {
    pub request_id: String,
    pub stage_count: usize,
    pub completed_stage_count: usize,
    pub rejected_stage_count: usize,
    pub unavailable_stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedStemOutput {
    pub stem_id: String,
    pub output: AudioBuffer,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
    pub artifact_id: String,
    pub output: AudioBuffer,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionMerge {
    pub request_id: String,
    pub main_mix: Option<AudioBuffer>,
    pub stems: Vec<RuntimeOfflinePluginDelegatedStemOutput>,
    pub freeze_artifacts: Vec<RuntimeOfflinePluginDelegatedFreezeArtifactOutput>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionOutcome {
    pub receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    pub merge: RuntimeOfflinePluginDelegatedExecutionMerge,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginExecutionOwner {
    #[default]
    SignalStageModel,
    HostDelegated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginOverrideState {
    #[default]
    NotAvailable,
    FreshLatestBlock,
    StaleLatestBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionStageBoundary {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
    pub execution_owner: RuntimeOfflinePluginExecutionOwner,
    pub host_delegate_required: bool,
    pub override_state: RuntimeOfflinePluginOverrideState,
    pub latest_override_processing_epoch: Option<u64>,
    pub latest_override_block_sequence: Option<u64>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionBoundary {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub runtime_sample_rate_hz: u32,
    pub export_sample_rate_hz: u32,
    pub block_size: usize,
    pub block_count: usize,
    pub stage_count: usize,
    pub signal_stage_model_stage_count: usize,
    pub host_delegate_stage_count: usize,
    pub fresh_override_stage_count: usize,
    pub stale_override_stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginExecutionStageBoundary>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    pub lane_count: usize,
    pub point_count: usize,
    pub projected_segment_count: usize,
    pub mapped_lane_count: usize,
    pub unmapped_lane_count: usize,
    pub hold_lane_count: usize,
    pub linear_lane_count: usize,
    pub last_batch_epoch: Option<u64>,
    pub last_batch_event_count: usize,
    pub last_batch_ignored_event_count: usize,
    pub last_batch_sub_block_count: usize,
    pub last_batch_coalesced_event_count: usize,
    pub last_batch_strategy_max_sub_blocks: usize,
    pub last_batch_min_ramp_step_samples: Option<usize>,
    pub last_batch_max_sample_offset: Option<usize>,
    pub last_block_sequence: Option<u64>,
    pub last_timeline_position_samples: Option<i64>,
    pub transport_playing: Option<bool>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginLifecycleState {
    Booting,
    Ready,
    Degraded,
    Faulted,
    Restarting,
    Quarantined,
    #[default]
    Stopped,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginSandboxSnapshot {
    pub sandbox_id: String,
    pub sandbox_group_key: String,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub instance_id: Option<String>,
    pub placement_outcome: RuntimePluginIsolationOutcome,
    pub placement_rule_id: Option<String>,
    pub shared_boundary_member_count: usize,
    pub continuity_class: RuntimeInterruptionClass,
    pub rebindable: bool,
    pub state: RuntimePluginLifecycleState,
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub transport_stage: Option<PluginSandboxTransportStage>,
    pub active: bool,
    pub active_transport: bool,
    pub restart_count: u32,
    pub recovery_count: u32,
    pub fault_count: u32,
    pub last_fault_kind: Option<PluginFaultKind>,
    pub last_fault_detail: Option<String>,
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_processing_epoch: Option<u64>,
    pub readiness_state: Option<String>,
    pub degraded_reasons: Vec<String>,
    pub active_lease_id: Option<String>,
    pub active_region_id: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginLifecycleSnapshot {
    pub sandbox_count: usize,
    pub active_sandbox_count: u32,
    pub shared_sandbox_count: usize,
    pub isolated_sandbox_count: usize,
    pub ready_sandbox_count: usize,
    pub booting_sandbox_count: usize,
    pub degraded_sandbox_count: usize,
    pub faulted_sandbox_count: usize,
    pub restarting_sandbox_count: usize,
    pub quarantined_sandbox_count: usize,
    pub stopped_sandbox_count: usize,
    pub rebindable_sandbox_count: usize,
    pub terminal_sandbox_count: usize,
    pub sandboxes: Vec<RuntimePluginSandboxSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginRecallState {
    #[default]
    Unbound,
    Cold,
    Warm,
    Recovered,
    Unavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallPayload {
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub transport_stage: Option<PluginSandboxTransportStage>,
    pub readiness_state: Option<String>,
    pub recovery_count: u32,
    pub restart_count: u32,
    pub fault_count: u32,
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_fault_kind: Option<PluginFaultKind>,
    pub last_fault_detail: Option<String>,
    pub degraded_reasons: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallSnapshot {
    pub state: RuntimePluginRecallState,
    pub payload: RuntimePluginRecallPayload,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginCompensationState {
    #[default]
    MissingBinding,
    PendingRender,
    Settling,
    Compensated,
    Bypassed,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginChainStageSnapshot {
    pub node_id: String,
    pub stage_index: usize,
    pub sandbox_id: Option<String>,
    pub sandbox_group_key: Option<String>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub placement_outcome: RuntimePluginIsolationOutcome,
    pub placement_rule_id: Option<String>,
    pub shared_boundary_member_count: usize,
    pub continuity_class: RuntimeInterruptionClass,
    pub rebindable: bool,
    pub lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub transport_stage: Option<PluginSandboxTransportStage>,
    pub recall_state: RuntimePluginRecallState,
    pub recall: RuntimePluginRecallSnapshot,
    pub compensation_state: RuntimePluginCompensationState,
    pub planned_latency_samples: u32,
    pub realized_latency_samples: Option<u32>,
    pub tail_samples: Option<u32>,
    pub bypassed: bool,
    pub active_transport: bool,
    pub degraded_reasons: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginExecutionChainSummary {
    pub chain_id: String,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub stage_count: usize,
    pub shared_sandbox_stage_count: usize,
    pub isolated_sandbox_stage_count: usize,
    pub in_process_stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub rebindable_stage_count: usize,
    pub terminal_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub stages: Vec<RuntimePluginChainStageSnapshot>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginChainSnapshot {
    pub chain_count: usize,
    pub stage_count: usize,
    pub shared_sandbox_stage_count: usize,
    pub isolated_sandbox_stage_count: usize,
    pub in_process_stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub rebindable_stage_count: usize,
    pub terminal_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub chains: Vec<RuntimePluginExecutionChainSummary>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePluginRecallHandoffStageId {
    pub chain_id: String,
    pub stage_index: usize,
    pub node_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSelection {
    pub stage_count: usize,
    pub stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffStage {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub stage_index: usize,
    pub chain_id: String,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSnapshot {
    pub stage_count: usize,
    pub unbound_stage_count: usize,
    pub cold_stage_count: usize,
    pub warm_stage_count: usize,
    pub recovered_stage_count: usize,
    pub unavailable_stage_count: usize,
    pub stages: Vec<RuntimePluginRecallHandoffStage>,
    pub summary: String,
}

impl RuntimePluginRecallHandoffSnapshot {
    pub fn from_plugin_chain_snapshot(snapshot: &RuntimePluginChainSnapshot) -> Self {
        let stages = snapshot
            .chains
            .iter()
            .flat_map(|chain| {
                chain
                    .stages
                    .iter()
                    .map(|stage| RuntimePluginRecallHandoffStage {
                        stage_id: RuntimePluginRecallHandoffStageId {
                            chain_id: chain.chain_id.clone(),
                            stage_index: stage.stage_index,
                            node_id: stage.node_id.clone(),
                        },
                        node_id: stage.node_id.clone(),
                        stage_index: stage.stage_index,
                        chain_id: chain.chain_id.clone(),
                        track_lane_id: stage.track_lane_id.clone(),
                        bus_group_id: stage.bus_group_id.clone(),
                        console_group_id: stage.console_group_id.clone(),
                        send_return_id: stage.send_return_id.clone(),
                        recall_state: stage.recall_state,
                        recall_payload: stage.recall.payload.clone(),
                    })
            })
            .collect::<Vec<_>>();
        let mut handoff = Self {
            stage_count: stages.len(),
            unbound_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Unbound)
                .count(),
            cold_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Cold)
                .count(),
            warm_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Warm)
                .count(),
            recovered_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Recovered)
                .count(),
            unavailable_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Unavailable)
                .count(),
            stages,
            summary: String::new(),
        };
        handoff.summary = format!(
            "stages={} unbound={} cold={} warm={} recovered={} unavailable={}",
            handoff.stage_count,
            handoff.unbound_stage_count,
            handoff.cold_stage_count,
            handoff.warm_stage_count,
            handoff.recovered_stage_count,
            handoff.unavailable_stage_count,
        );
        handoff
    }

    pub fn resolve_stage(
        &self,
        stage_id: &RuntimePluginRecallHandoffStageId,
    ) -> Option<&RuntimePluginRecallHandoffStage> {
        self.stages.iter().find(|stage| &stage.stage_id == stage_id)
    }

    pub fn resolve_selection<'a>(
        &'a self,
        selection: &RuntimePluginRecallHandoffSelection,
    ) -> Option<Vec<&'a RuntimePluginRecallHandoffStage>> {
        if selection.stage_count != selection.stage_ids.len() {
            return None;
        }
        selection
            .stage_ids
            .iter()
            .map(|stage_id| self.resolve_stage(stage_id))
            .collect()
    }
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
    pub topology_compatible: bool,
    pub topology_issue_count: usize,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultCause {
    XrunOverload,
    PluginFault,
    WatchdogRestart,
    DeviceLoss,
    TransportFault,
    MissingPluginBinding,
    RuntimeError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticFamily {
    XrunPressure,
    CallbackPressure,
    PluginBoundaryFault,
    DevicePathFault,
    DeferredWorkPressure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticAuthority {
    RuntimeCanonical,
    HostAdvisory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultContributionReceipt {
    pub family: RuntimeFaultDiagnosticFamily,
    pub authority: RuntimeFaultDiagnosticAuthority,
    pub active: bool,
    pub event_count: u64,
    pub detail: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultDiagnosticReceipt {
    pub primary_family: Option<RuntimeFaultDiagnosticFamily>,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub interruption_class: RuntimeInterruptionClass,
    pub recovery_state: RuntimeRecoveryState,
    pub safe_mode_enabled: bool,
    pub rebindable: bool,
    pub contributions: Vec<RuntimeFaultContributionReceipt>,
    pub summary: String,
}

impl RuntimeFaultDiagnosticReceipt {
    pub fn capture(
        fault_status: &RuntimeFaultStatusSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
        degradation_summary: &RuntimeDegradationSummary,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
        host_io: Option<&RuntimeHostIoSummary>,
    ) -> Self {
        let plugin_boundary_event_count = degradation_summary
            .plugin_fault_count
            .saturating_add(degradation_summary.transport_fault_event_count)
            .saturating_add(degradation_summary.broker_failure_event_count)
            .saturating_add(degradation_summary.sandbox_operation_failure_event_count)
            .saturating_add(usize::from(
                fault_status.missing_plugin_binding_active
                    || degradation_summary.missing_bound_plugin_sandboxes > 0,
            )) as u64;
        let plugin_boundary_active = fault_status.plugin_fault_active
            || fault_status.transport_fault_active
            || fault_status.missing_plugin_binding_active;
        let xrun_event_count = degradation_summary.xrun_count;
        let deferred_event_count = engine_block_snapshot
            .prework_service_starvation_count
            .saturating_add(engine_block_snapshot.prework_service_throttle_count)
            .saturating_add(engine_block_snapshot.prework_service_yield_count)
            .saturating_add(
                last_deferred_service_receipt
                    .map(|receipt| receipt.deferred_work_item_count as u64)
                    .unwrap_or(0),
            );
        let deferred_active = matches!(
            last_deferred_service_receipt.map(|receipt| receipt.decision),
            Some(RuntimeDeferredServiceDecision::Defer | RuntimeDeferredServiceDecision::Throttle)
        ) || matches!(
            engine_block_snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
                | RuntimePreworkServiceState::Paused
                | RuntimePreworkServiceState::Starved
        ) || matches!(
            engine_block_snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated | RuntimePreworkServicePressure::Critical
        );
        let callback_event_count = host_io
            .map(|host_io| {
                host_io
                    .hardware
                    .callback_overrun_count
                    .saturating_add(host_io.hardware.xrun_count)
            })
            .unwrap_or(0);
        let callback_active = host_io
            .map(|host_io| {
                host_io.hardware.callback_overrun_count > 0
                    || host_io.hardware.xrun_count > 0
                    || host_io.hardware.restart_failure_count > 0
            })
            .unwrap_or(false);

        let mut contributions = vec![
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::XrunPressure,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: fault_status.xrun_overload_active,
                event_count: xrun_event_count,
                detail: Some(format!(
                    "xrun_overload_active={} safe_mode={} runtime_xruns={}",
                    fault_status.xrun_overload_active,
                    fault_status.safe_mode_enabled,
                    xrun_event_count
                )),
                summary: String::new(),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::PluginBoundaryFault,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: plugin_boundary_active,
                event_count: plugin_boundary_event_count,
                detail: Some(format!(
                    "plugin_faults={} transport_fault_events={} broker_failures={} sandbox_operation_failures={} missing_bindings={}",
                    degradation_summary.plugin_fault_count,
                    degradation_summary.transport_fault_event_count,
                    degradation_summary.broker_failure_event_count,
                    degradation_summary.sandbox_operation_failure_event_count,
                    usize::from(
                        fault_status.missing_plugin_binding_active
                            || degradation_summary.missing_bound_plugin_sandboxes > 0
                    )
                )),
                summary: String::new(),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::DevicePathFault,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: fault_status.device_loss_active,
                event_count: fault_status.device_loss_count,
                detail: Some(format!(
                    "device_loss_active={} device_losses={} watchdog_restarts={}",
                    fault_status.device_loss_active,
                    fault_status.device_loss_count,
                    fault_status.watchdog_restart_count
                )),
                summary: String::new(),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::DeferredWorkPressure,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: deferred_active,
                event_count: deferred_event_count,
                detail: Some(format!(
                    "decision={:?} reason={:?} prework_state={:?} prework_pressure={:?} starvations={} throttles={} yields={} deferred_items={}",
                    last_deferred_service_receipt.map(|receipt| receipt.decision),
                    last_deferred_service_receipt.map(|receipt| receipt.reason),
                    engine_block_snapshot.prework_service_state,
                    engine_block_snapshot.prework_service_pressure,
                    engine_block_snapshot.prework_service_starvation_count,
                    engine_block_snapshot.prework_service_throttle_count,
                    engine_block_snapshot.prework_service_yield_count,
                    last_deferred_service_receipt
                        .map(|receipt| receipt.deferred_work_item_count)
                        .unwrap_or(0)
                )),
                summary: String::new(),
            },
        ];
        if let Some(host_io) = host_io {
            contributions.push(RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::CallbackPressure,
                authority: RuntimeFaultDiagnosticAuthority::HostAdvisory,
                active: callback_active,
                event_count: callback_event_count,
                detail: Some(format!(
                    "callback_count={} callback_interval_ms={:.3} callback_overruns={} backend_xruns={} restart_failures={}",
                    host_io.audio_pump.callback_count,
                    host_io.clocking.callback_interval_ms,
                    host_io.hardware.callback_overrun_count,
                    host_io.hardware.xrun_count,
                    host_io.hardware.restart_failure_count
                )),
                summary: String::new(),
            });
        }

        for contribution in &mut contributions {
            contribution.summary = format!(
                "family={:?} authority={:?} active={} events={} detail={}",
                contribution.family,
                contribution.authority,
                contribution.active,
                contribution.event_count,
                contribution.detail.as_deref().unwrap_or("none")
            );
        }

        let primary_family = match fault_status.primary_fault_cause {
            Some(RuntimeFaultCause::XrunOverload) => {
                Some(RuntimeFaultDiagnosticFamily::XrunPressure)
            }
            Some(
                RuntimeFaultCause::PluginFault
                | RuntimeFaultCause::TransportFault
                | RuntimeFaultCause::MissingPluginBinding,
            ) => Some(RuntimeFaultDiagnosticFamily::PluginBoundaryFault),
            Some(RuntimeFaultCause::DeviceLoss) => {
                Some(RuntimeFaultDiagnosticFamily::DevicePathFault)
            }
            Some(RuntimeFaultCause::WatchdogRestart) => {
                if plugin_boundary_active {
                    Some(RuntimeFaultDiagnosticFamily::PluginBoundaryFault)
                } else if fault_status.device_loss_active {
                    Some(RuntimeFaultDiagnosticFamily::DevicePathFault)
                } else if fault_status.xrun_overload_active {
                    Some(RuntimeFaultDiagnosticFamily::XrunPressure)
                } else if deferred_active {
                    Some(RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
                } else {
                    None
                }
            }
            Some(RuntimeFaultCause::RuntimeError) => None,
            None if deferred_active => Some(RuntimeFaultDiagnosticFamily::DeferredWorkPressure),
            None => None,
        };

        let mut receipt = Self {
            primary_family,
            primary_fault_cause: fault_status.primary_fault_cause,
            interruption_class: interruption_summary.class,
            recovery_state: fault_status.recovery_state,
            safe_mode_enabled: fault_status.safe_mode_enabled,
            rebindable: interruption_summary.rebindable,
            contributions,
            summary: String::new(),
        };
        receipt.summary = format!(
            "primary_family={:?} primary_cause={:?} interruption={:?} recovery={:?} rebindable={} contributions={}",
            receipt.primary_family,
            receipt.primary_fault_cause,
            receipt.interruption_class,
            receipt.recovery_state,
            receipt.rebindable,
            receipt.contributions.len()
        );
        receipt
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecoveryState {
    Steady,
    Recovering,
    Faulted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeInterruptionClass {
    #[default]
    Steady,
    Resumable,
    Restartable,
    Recoverable,
    Terminal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultStatusSnapshot {
    pub recovery_state: RuntimeRecoveryState,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub active_fault_count: usize,
    pub xrun_overload_active: bool,
    pub plugin_fault_active: bool,
    pub watchdog_active: bool,
    pub device_loss_active: bool,
    pub transport_fault_active: bool,
    pub missing_plugin_binding_active: bool,
    pub safe_mode_enabled: bool,
    pub restart_count: u64,
    pub watchdog_restart_count: u32,
    pub plugin_fault_count: usize,
    pub transport_faulted_session_count: usize,
    pub device_loss_count: u64,
    pub summary: String,
}

impl RuntimeFaultStatusSnapshot {
    pub fn capture(
        readiness: RuntimeReadiness,
        control_snapshot: &RuntimeControlSnapshot,
        diagnostics_snapshot: &RuntimeDiagnosticsSnapshot,
        supervision_snapshot: &RuntimeSupervisionSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        transport_concurrency_snapshot: &RuntimeTransportConcurrencySnapshot,
        plugin_lifecycle_snapshot: &RuntimePluginLifecycleSnapshot,
        device_loss_active: bool,
        device_loss_count: u64,
    ) -> Self {
        let xrun_overload_active = supervision_snapshot.xrun_overload_active;
        let plugin_fault_count = plugin_lifecycle_snapshot
            .faulted_sandbox_count
            .saturating_add(plugin_lifecycle_snapshot.quarantined_sandbox_count);
        let plugin_fault_active = plugin_fault_count > 0;
        let watchdog_active = supervision_snapshot.safe_mode_enabled
            && supervision_snapshot.watchdog_restart_count > 0;
        let transport_faulted_session_count =
            transport_concurrency_snapshot.current_detach_faulted_sessions;
        let transport_fault_active = transport_faulted_session_count > 0;
        let missing_plugin_binding_active =
            engine_block_snapshot.prework_service_missing_bound_plugin_sandboxes > 0;
        let runtime_error_active = matches!(readiness, RuntimeReadiness::Failed { .. });
        let primary_fault_cause = if device_loss_active {
            Some(RuntimeFaultCause::DeviceLoss)
        } else if watchdog_active {
            Some(RuntimeFaultCause::WatchdogRestart)
        } else if plugin_fault_active {
            Some(RuntimeFaultCause::PluginFault)
        } else if transport_fault_active {
            Some(RuntimeFaultCause::TransportFault)
        } else if xrun_overload_active {
            Some(RuntimeFaultCause::XrunOverload)
        } else if missing_plugin_binding_active {
            Some(RuntimeFaultCause::MissingPluginBinding)
        } else if runtime_error_active {
            Some(RuntimeFaultCause::RuntimeError)
        } else {
            None
        };
        let mut active_fault_count = usize::from(xrun_overload_active)
            + usize::from(plugin_fault_active)
            + usize::from(watchdog_active)
            + usize::from(device_loss_active)
            + usize::from(transport_fault_active)
            + usize::from(missing_plugin_binding_active);
        if runtime_error_active && primary_fault_cause == Some(RuntimeFaultCause::RuntimeError) {
            active_fault_count = active_fault_count.saturating_add(1);
        }
        let recovery_state = if runtime_error_active {
            RuntimeRecoveryState::Faulted
        } else if supervision_snapshot.safe_mode_enabled
            || xrun_overload_active
            || device_loss_active
            || watchdog_active
            || transport_fault_active
            || plugin_lifecycle_snapshot.restarting_sandbox_count > 0
            || control_snapshot.restart_count > 0
        {
            RuntimeRecoveryState::Recovering
        } else {
            RuntimeRecoveryState::Steady
        };
        let mut snapshot = Self {
            recovery_state,
            primary_fault_cause,
            active_fault_count,
            xrun_overload_active,
            plugin_fault_active,
            watchdog_active,
            device_loss_active,
            transport_fault_active,
            missing_plugin_binding_active,
            safe_mode_enabled: supervision_snapshot.safe_mode_enabled,
            restart_count: control_snapshot.restart_count,
            watchdog_restart_count: supervision_snapshot.watchdog_restart_count,
            plugin_fault_count,
            transport_faulted_session_count,
            device_loss_count,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "recovery={:?} primary={:?} faults={} xruns={} plugin_faults={} watchdog_restarts={} device_losses={} transport_faulted_sessions={} safe_mode={} restarts={}",
            snapshot.recovery_state,
            snapshot.primary_fault_cause,
            snapshot.active_fault_count,
            diagnostics_snapshot.xruns,
            snapshot.plugin_fault_count,
            snapshot.watchdog_restart_count,
            snapshot.device_loss_count,
            snapshot.transport_faulted_session_count,
            snapshot.safe_mode_enabled,
            snapshot.restart_count,
        );
        snapshot
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInterruptionSummary {
    pub active: bool,
    pub class: RuntimeInterruptionClass,
    pub rebindable: bool,
    pub recovery_state: RuntimeRecoveryState,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub safe_mode_enabled: bool,
    pub deferred_service_class: Option<RuntimeDeferredServiceClass>,
    pub deferred_service_decision: Option<RuntimeDeferredServiceDecision>,
    pub summary: String,
}

impl RuntimeInterruptionSummary {
    pub fn capture(
        fault_status: &RuntimeFaultStatusSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
    ) -> Self {
        let class = match fault_status.recovery_state {
            RuntimeRecoveryState::Faulted => RuntimeInterruptionClass::Terminal,
            RuntimeRecoveryState::Recovering
                if matches!(
                    fault_status.primary_fault_cause,
                    Some(
                        RuntimeFaultCause::DeviceLoss
                            | RuntimeFaultCause::WatchdogRestart
                            | RuntimeFaultCause::PluginFault
                            | RuntimeFaultCause::TransportFault
                            | RuntimeFaultCause::MissingPluginBinding
                    )
                ) =>
            {
                RuntimeInterruptionClass::Restartable
            }
            RuntimeRecoveryState::Recovering => RuntimeInterruptionClass::Recoverable,
            RuntimeRecoveryState::Steady
                if matches!(
                    last_deferred_service_receipt.map(|receipt| receipt.decision),
                    Some(
                        RuntimeDeferredServiceDecision::Defer
                            | RuntimeDeferredServiceDecision::Throttle
                    )
                ) =>
            {
                RuntimeInterruptionClass::Resumable
            }
            RuntimeRecoveryState::Steady => RuntimeInterruptionClass::Steady,
        };
        let rebindable = matches!(
            fault_status.primary_fault_cause,
            Some(
                RuntimeFaultCause::DeviceLoss
                    | RuntimeFaultCause::PluginFault
                    | RuntimeFaultCause::TransportFault
                    | RuntimeFaultCause::MissingPluginBinding
            )
        );
        let mut summary = Self {
            active: class != RuntimeInterruptionClass::Steady,
            class,
            rebindable,
            recovery_state: fault_status.recovery_state,
            primary_fault_cause: fault_status.primary_fault_cause,
            safe_mode_enabled: fault_status.safe_mode_enabled,
            deferred_service_class: last_deferred_service_receipt.map(|receipt| receipt.work_class),
            deferred_service_decision: last_deferred_service_receipt
                .map(|receipt| receipt.decision),
            summary: String::new(),
        };
        summary.summary = format!(
            "class={:?} active={} rebindable={} recovery={:?} primary={:?} deferred={:?}/{:?} safe_mode={}",
            summary.class,
            summary.active,
            summary.rebindable,
            summary.recovery_state,
            summary.primary_fault_cause,
            summary.deferred_service_class,
            summary.deferred_service_decision,
            summary.safe_mode_enabled,
        );
        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMeterSourceRole {
    Utility,
    TrackLane,
    Bus,
    Send,
    Return,
    ConsoleNode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeterSourceSnapshot {
    pub bus_id: String,
    pub topology_role: RuntimeMeterSourceRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: f32,
    pub rms_level: f32,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeteringSnapshot {
    pub meter_count: usize,
    pub main_output_peak_level: Option<f32>,
    pub main_output_rms_level: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
    pub clipped_sample_count: u64,
    pub meters: Vec<RuntimeMeterSourceSnapshot>,
    pub track_lanes: Vec<RuntimeTrackLaneMeterSummary>,
    pub bus_groups: Vec<RuntimeBusGroupMeterSummary>,
    pub console_groups: Vec<RuntimeConsoleGroupMeterSummary>,
    pub send_returns: Vec<RuntimeSendReturnMeterSummary>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRoutedMeterAggregate {
    pub meter_count: usize,
    pub metered_bus_ids: Vec<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: Option<f32>,
    pub rms_level: Option<f32>,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTrackLaneMeterSummary {
    pub track_lane_id: String,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeBusGroupMeterSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeConsoleGroupMeterSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeSendReturnMeterSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

impl RuntimeMeteringSnapshot {
    pub fn with_execution_topology(mut self, topology: &RuntimeExecutionTopologySummary) -> Self {
        self.track_lanes = topology
            .track_lanes
            .iter()
            .map(|track_lane| RuntimeTrackLaneMeterSummary {
                track_lane_id: track_lane.track_lane_id.clone(),
                bus_group_ids: track_lane.bus_group_ids.clone(),
                input_bus_ids: track_lane.input_bus_ids.clone(),
                output_bus_ids: track_lane.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.track_lane_id.as_deref() == Some(track_lane.track_lane_id.as_str())
                    }),
                    format!("track_lane={}", track_lane.track_lane_id),
                ),
            })
            .collect();
        self.bus_groups = topology
            .bus_groups
            .iter()
            .map(|bus_group| RuntimeBusGroupMeterSummary {
                bus_group_id: bus_group.bus_group_id.clone(),
                topology_roles: bus_group.topology_roles.clone(),
                node_ids: bus_group.node_ids.clone(),
                input_bus_ids: bus_group.input_bus_ids.clone(),
                output_bus_ids: bus_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.bus_group_id.as_deref() == Some(bus_group.bus_group_id.as_str())
                    }),
                    format!("bus_group={}", bus_group.bus_group_id),
                ),
            })
            .collect();
        self.console_groups = topology
            .console_groups
            .iter()
            .map(|console_group| RuntimeConsoleGroupMeterSummary {
                console_group_id: console_group.console_group_id.clone(),
                node_ids: console_group.node_ids.clone(),
                input_bus_ids: console_group.input_bus_ids.clone(),
                output_bus_ids: console_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.console_group_id.as_deref()
                            == Some(console_group.console_group_id.as_str())
                    }),
                    format!("console_group={}", console_group.console_group_id),
                ),
            })
            .collect();
        self.send_returns = topology
            .send_returns
            .iter()
            .map(|send_return| RuntimeSendReturnMeterSummary {
                send_return_id: send_return.send_return_id.clone(),
                send_node_ids: send_return.send_node_ids.clone(),
                return_node_ids: send_return.return_node_ids.clone(),
                input_bus_ids: send_return.input_bus_ids.clone(),
                output_bus_ids: send_return.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.send_return_id.as_deref() == Some(send_return.send_return_id.as_str())
                    }),
                    format!("send_return={}", send_return.send_return_id),
                ),
            })
            .collect();
        self.summary = format!(
            "meters={} main_peak={:?} main_rms={:?} momentary_lufs={:?} short_term_lufs={:?} integrated_lufs={:?} clipped={} routes={}/{}/{}/{}",
            self.meter_count,
            self.main_output_peak_level,
            self.main_output_rms_level,
            self.momentary_loudness_lufs,
            self.short_term_loudness_lufs,
            self.integrated_loudness_lufs,
            self.clipped_sample_count,
            self.track_lanes.len(),
            self.bus_groups.len(),
            self.send_returns.len(),
            self.console_groups.len(),
        );
        self
    }
}

fn aggregate_runtime_meter_sources<'a>(
    meters: impl Iterator<Item = &'a RuntimeMeterSourceSnapshot>,
    scope: String,
) -> RuntimeRoutedMeterAggregate {
    let mut aggregate = RuntimeRoutedMeterAggregate::default();

    for meter in meters {
        aggregate.meter_count += 1;
        if !aggregate.metered_bus_ids.contains(&meter.bus_id) {
            aggregate.metered_bus_ids.push(meter.bus_id.clone());
        }
        for producer_node_id in &meter.producer_node_ids {
            if !aggregate.producer_node_ids.contains(producer_node_id) {
                aggregate.producer_node_ids.push(producer_node_id.clone());
            }
        }
        aggregate.peak_level = Some(match aggregate.peak_level {
            Some(peak_level) => peak_level.max(meter.peak_level),
            None => meter.peak_level,
        });
        aggregate.rms_level = Some(match aggregate.rms_level {
            Some(rms_level) => rms_level.max(meter.rms_level),
            None => meter.rms_level,
        });
        aggregate.latency_samples = aggregate.latency_samples.max(meter.latency_samples);
        aggregate.tail_samples = aggregate.tail_samples.max(meter.tail_samples);
    }

    aggregate.summary = format!(
        "{scope} meters={} peak={:?} rms={:?} buses={:?} producers={}",
        aggregate.meter_count,
        aggregate.peak_level,
        aggregate.rms_level,
        aggregate.metered_bus_ids,
        aggregate.producer_node_ids.len(),
    );
    aggregate
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
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
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
            last_block_execution_time_ns: snapshot.last_block_execution_time_ns,
            last_block_deadline_budget_ns: snapshot.last_block_deadline_budget_ns,
            last_block_budget_utilization_percent: snapshot.last_block_budget_utilization_percent,
            last_block_budget_overrun_ns: snapshot.last_block_budget_overrun_ns,
            last_block_deadline_pressure: snapshot.last_block_deadline_pressure,
            budget_overrun_count: snapshot.budget_overrun_count,
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

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceSnapshot {
    pub sample_rate_hz: u32,
    pub block_size: usize,
    pub processed_block_count: u64,
    pub last_block_sequence: Option<u64>,
    pub cpu_load_percent: f32,
    pub graph_latency_ms: f32,
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
    pub peak_block_execution_time_ns: u64,
    pub peak_block_budget_utilization_percent: f32,
    pub peak_block_budget_overrun_ns: u64,
    pub xrun_count: u64,
    pub scheduler_phase_count: usize,
    pub scheduler_lane_count: usize,
    pub scheduler_dispatch_count: usize,
    pub scheduler_prepared_dispatch_count: usize,
    pub scheduler_realtime_dispatch_count: usize,
    pub scheduler_dispatch_handoff_count: usize,
    pub scheduler_topology_compatible: bool,
    pub scheduler_topology_requires_host_reinterpretation: bool,
    pub scheduler_topology_issue_count: usize,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub pending_prework_target_count: usize,
    pub pending_prework_deferred_target_count: usize,
    pub prework_queue_depth: usize,
    pub prework_peak_queue_depth: usize,
    pub prework_service_cycle_count: u64,
    pub prework_service_starvation_count: u64,
    pub prework_service_throttle_count: u64,
    pub prework_service_yield_count: u64,
    pub last_prework_service_effective_cycles: usize,
    pub last_prework_service_budget_per_cycle: Option<usize>,
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    pub last_prework_serviced_backlog_class: Option<String>,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub hot_latency_node_id: Option<String>,
    pub hot_latency_node_group: Option<String>,
    pub hot_latency_node_topology_role: Option<String>,
    pub hot_latency_node_plugin_sandbox_id: Option<String>,
    pub hot_latency_node_samples: u32,
    pub hot_latency_group: Option<String>,
    pub hot_latency_group_node_count: usize,
    pub hot_latency_group_total_samples: u32,
    pub critical_path_lane: Option<String>,
    pub critical_path_lane_node_count: usize,
    pub critical_path_lane_plugin_backed_node_count: usize,
    pub critical_path_lane_planning_group_count: usize,
    pub critical_path_lane_total_latency_samples: u32,
    pub worker_lane_summaries: Vec<RuntimeWorkerLaneInstrumentationSummary>,
    pub background_service_class: Option<RuntimeDeferredServiceClass>,
    pub background_service_decision: Option<RuntimeDeferredServiceDecision>,
    pub background_service_reason: Option<RuntimeDeferredServiceReason>,
    pub background_queued_work_item_count: usize,
    pub background_deferred_work_item_count: usize,
    pub background_pending_cleanup_work_item_count: usize,
    pub background_pending_retry_work_item_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeWorkerLaneInstrumentationSummary {
    pub lane: GraphExecutionLane,
    pub node_count: usize,
    pub plugin_backed_node_count: usize,
    pub planning_group_count: usize,
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
}

impl RuntimePerformanceSnapshot {
    pub fn capture(
        effective_config: &EffectiveRuntimeConfig,
        diagnostics_snapshot: &RuntimeDiagnosticsSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
    ) -> Self {
        let worker_lane_summaries =
            runtime_worker_lane_instrumentation_summaries(engine_block_snapshot);
        let hot_latency_node = engine_block_snapshot
            .planned_nodes
            .iter()
            .max_by_key(|node| node.latency_samples)
            .filter(|node| node.latency_samples > 0);
        let mut inline_realtime_group_total_samples = 0u32;
        let mut inline_realtime_group_node_count = 0usize;
        let mut stateful_realtime_group_total_samples = 0u32;
        let mut stateful_realtime_group_node_count = 0usize;
        let mut anticipative_group_total_samples = 0u32;
        let mut anticipative_group_node_count = 0usize;
        for node in &engine_block_snapshot.planned_nodes {
            match node.group {
                GraphNodePlanningGroup::InlineRealtime => {
                    inline_realtime_group_total_samples =
                        inline_realtime_group_total_samples.saturating_add(node.latency_samples);
                    inline_realtime_group_node_count =
                        inline_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::StatefulRealtime => {
                    stateful_realtime_group_total_samples =
                        stateful_realtime_group_total_samples.saturating_add(node.latency_samples);
                    stateful_realtime_group_node_count =
                        stateful_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::AnticipativeEligible => {
                    anticipative_group_total_samples =
                        anticipative_group_total_samples.saturating_add(node.latency_samples);
                    anticipative_group_node_count = anticipative_group_node_count.saturating_add(1);
                }
            }
        }
        let hot_latency_group = [
            (
                GraphNodePlanningGroup::InlineRealtime,
                inline_realtime_group_total_samples,
                inline_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::StatefulRealtime,
                stateful_realtime_group_total_samples,
                stateful_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::AnticipativeEligible,
                anticipative_group_total_samples,
                anticipative_group_node_count,
            ),
        ]
        .into_iter()
        .max_by_key(|(_, total_samples, _)| *total_samples)
        .filter(|(_, total_samples, _)| *total_samples > 0);
        let critical_path_lane = worker_lane_summaries
            .iter()
            .max_by_key(|summary| summary.total_latency_samples)
            .filter(|summary| summary.total_latency_samples > 0);
        let mut snapshot = Self {
            sample_rate_hz: effective_config.sample_rate.0,
            block_size: effective_config.block_size,
            processed_block_count: engine_block_snapshot.processed_blocks,
            last_block_sequence: engine_block_snapshot.last_block_sequence,
            cpu_load_percent: diagnostics_snapshot.cpu_load_percent,
            graph_latency_ms: diagnostics_snapshot.graph_latency_ms,
            last_block_execution_time_ns: engine_block_snapshot.last_block_execution_time_ns,
            last_block_deadline_budget_ns: engine_block_snapshot.last_block_deadline_budget_ns,
            last_block_budget_utilization_percent: engine_block_snapshot
                .last_block_budget_utilization_percent,
            last_block_budget_overrun_ns: engine_block_snapshot.last_block_budget_overrun_ns,
            last_block_deadline_pressure: engine_block_snapshot.last_block_deadline_pressure,
            budget_overrun_count: engine_block_snapshot.budget_overrun_count,
            peak_block_execution_time_ns: engine_block_snapshot.peak_block_execution_time_ns,
            peak_block_budget_utilization_percent: engine_block_snapshot
                .peak_block_budget_utilization_percent,
            peak_block_budget_overrun_ns: engine_block_snapshot.peak_block_budget_overrun_ns,
            xrun_count: diagnostics_snapshot.xruns,
            scheduler_phase_count: engine_block_snapshot.phase_count,
            scheduler_lane_count: engine_block_snapshot.lane_count,
            scheduler_dispatch_count: engine_block_snapshot.dispatch_count,
            scheduler_prepared_dispatch_count: engine_block_snapshot.prepared_dispatch_count,
            scheduler_realtime_dispatch_count: engine_block_snapshot.realtime_dispatch_count,
            scheduler_dispatch_handoff_count: engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology_compatible: engine_block_snapshot.scheduler_topology.compatible,
            scheduler_topology_requires_host_reinterpretation: engine_block_snapshot
                .scheduler_topology
                .requires_host_reinterpretation,
            scheduler_topology_issue_count: engine_block_snapshot.scheduler_topology.issues.len(),
            prework_service_state: engine_block_snapshot.prework_service_state,
            prework_service_pressure: engine_block_snapshot.prework_service_pressure,
            prework_service_semantic_policy: engine_block_snapshot.prework_service_semantic_policy,
            pending_prework_target_count: engine_block_snapshot.prework_pending_target_count,
            pending_prework_deferred_target_count: engine_block_snapshot
                .prework_pending_deferred_target_count,
            prework_queue_depth: engine_block_snapshot.prework_cache_queue_depth,
            prework_peak_queue_depth: engine_block_snapshot.prework_cache_peak_queue_depth,
            prework_service_cycle_count: engine_block_snapshot.prework_service_cycle_count,
            prework_service_starvation_count: engine_block_snapshot
                .prework_service_starvation_count,
            prework_service_throttle_count: engine_block_snapshot.prework_service_throttle_count,
            prework_service_yield_count: engine_block_snapshot.prework_service_yield_count,
            last_prework_service_effective_cycles: engine_block_snapshot
                .last_prework_service_effective_cycles,
            last_prework_service_budget_per_cycle: engine_block_snapshot
                .last_prework_service_budget_per_cycle,
            last_prework_service_effective_budget_per_cycle: engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            last_prework_serviced_backlog_class: engine_block_snapshot
                .last_prework_serviced_backlog_class
                .map(|value| runtime_prework_backlog_class_name(value).to_string()),
            transport_gate_active: engine_block_snapshot.prework_service_transport_gate_active,
            plugin_gate_active: engine_block_snapshot.prework_service_plugin_gate_active,
            hot_latency_node_id: hot_latency_node.map(|node| node.node_id.clone()),
            hot_latency_node_group: hot_latency_node
                .map(|node| runtime_graph_node_planning_group_name(node.group).to_string()),
            hot_latency_node_topology_role: hot_latency_node
                .map(|node| runtime_graph_node_topology_role_name(node.topology_role).to_string()),
            hot_latency_node_plugin_sandbox_id: hot_latency_node
                .and_then(|node| node.plugin_sandbox_id.clone()),
            hot_latency_node_samples: hot_latency_node.map_or(0, |node| node.latency_samples),
            hot_latency_group: hot_latency_group
                .map(|(group, _, _)| runtime_graph_node_planning_group_name(group).to_string()),
            hot_latency_group_node_count: hot_latency_group
                .map_or(0, |(_, _, node_count)| node_count),
            hot_latency_group_total_samples: hot_latency_group
                .map_or(0, |(_, total_samples, _)| total_samples),
            critical_path_lane: critical_path_lane
                .map(|summary| runtime_execution_lane_name(summary.lane).to_string()),
            critical_path_lane_node_count: critical_path_lane
                .map_or(0, |summary| summary.node_count),
            critical_path_lane_plugin_backed_node_count: critical_path_lane
                .map_or(0, |summary| summary.plugin_backed_node_count),
            critical_path_lane_planning_group_count: critical_path_lane
                .map_or(0, |summary| summary.planning_group_count),
            critical_path_lane_total_latency_samples: critical_path_lane
                .map_or(0, |summary| summary.total_latency_samples),
            worker_lane_summaries,
            background_service_class: last_deferred_service_receipt
                .map(|receipt| receipt.work_class),
            background_service_decision: last_deferred_service_receipt
                .map(|receipt| receipt.decision),
            background_service_reason: last_deferred_service_receipt.map(|receipt| receipt.reason),
            background_queued_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.queued_work_item_count)
                .unwrap_or(0),
            background_deferred_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.deferred_work_item_count)
                .unwrap_or(0),
            background_pending_cleanup_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_cleanup_work_items)
                .unwrap_or(0),
            background_pending_retry_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_deferred_retry_work_items)
                .unwrap_or(0),
            summary: String::new(),
        };
        let dispatch_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_dispatch_count,
            snapshot.scheduler_prepared_dispatch_count,
            snapshot.scheduler_realtime_dispatch_count
        );
        let topology_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_topology_compatible,
            snapshot.scheduler_topology_requires_host_reinterpretation,
            snapshot.scheduler_topology_issue_count
        );
        let prework_summary = format!(
            "{:?}/{:?}/{:?}",
            snapshot.prework_service_state,
            snapshot.prework_service_pressure,
            snapshot.prework_service_semantic_policy,
        );
        let service_summary = format!(
            "{}/{}/{}/{}",
            snapshot.prework_service_starvation_count,
            snapshot.prework_service_throttle_count,
            snapshot.prework_service_yield_count,
            snapshot.last_prework_service_effective_cycles,
        );
        let hot_node_summary = format!(
            "{:?}/{:?}/{:?}/{}",
            snapshot.hot_latency_node_id,
            snapshot.hot_latency_node_group,
            snapshot.hot_latency_node_topology_role,
            snapshot.hot_latency_node_samples,
        );
        let hot_group_summary = format!(
            "{:?}/{}/{}",
            snapshot.hot_latency_group,
            snapshot.hot_latency_group_node_count,
            snapshot.hot_latency_group_total_samples
        );
        let critical_lane_summary = format!(
            "{:?}/{}/{}/{}/{}",
            snapshot.critical_path_lane,
            snapshot.critical_path_lane_node_count,
            snapshot.critical_path_lane_plugin_backed_node_count,
            snapshot.critical_path_lane_planning_group_count,
            snapshot.critical_path_lane_total_latency_samples
        );
        let worker_lane_summary = snapshot
            .worker_lane_summaries
            .iter()
            .map(|summary| {
                format!(
                    "{}:{}/{}/{}/{}/{}",
                    runtime_execution_lane_name(summary.lane),
                    summary.node_count,
                    summary.plugin_backed_node_count,
                    summary.planning_group_count,
                    summary.total_latency_samples,
                    summary.max_node_latency_samples,
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let background_summary = format!(
            "{:?}/{:?}/{:?}",
            snapshot.background_service_class,
            snapshot.background_service_decision,
            snapshot.background_service_reason,
        );
        snapshot.summary = format!(
            "sample_rate={} block_size={} blocks={} cpu_load={:.3} graph_latency_ms={:.3} timing={:?}/{:?}/{:?}/{:?}/{:?}/{} xruns={} phases={} lanes={} dispatches={} handoff={} topology={} prework={} pending_targets={}/{} queue={}/{} service={} cycles={} budget={:?}/{:?} backlog={:?} gates={}/{} hot_node={} hot_group={} critical_lane={} worker_lanes={} background={} items={}/{}/{}/{}",
            snapshot.sample_rate_hz,
            snapshot.block_size,
            snapshot.processed_block_count,
            snapshot.cpu_load_percent,
            snapshot.graph_latency_ms,
            snapshot.last_block_execution_time_ns,
            snapshot.last_block_deadline_budget_ns,
            snapshot.last_block_budget_utilization_percent,
            snapshot.last_block_budget_overrun_ns,
            snapshot.last_block_deadline_pressure,
            snapshot.budget_overrun_count,
            snapshot.xrun_count,
            snapshot.scheduler_phase_count,
            snapshot.scheduler_lane_count,
            dispatch_summary,
            snapshot.scheduler_dispatch_handoff_count,
            topology_summary,
            prework_summary,
            snapshot.pending_prework_target_count,
            snapshot.pending_prework_deferred_target_count,
            snapshot.prework_queue_depth,
            snapshot.prework_peak_queue_depth,
            service_summary,
            snapshot.prework_service_cycle_count,
            snapshot.last_prework_service_budget_per_cycle,
            snapshot.last_prework_service_effective_budget_per_cycle,
            snapshot.last_prework_serviced_backlog_class,
            snapshot.transport_gate_active,
            snapshot.plugin_gate_active,
            hot_node_summary,
            hot_group_summary,
            critical_lane_summary,
            worker_lane_summary,
            background_summary,
            snapshot.background_queued_work_item_count,
            snapshot.background_deferred_work_item_count,
            snapshot.background_pending_cleanup_work_item_count,
            snapshot.background_pending_retry_work_item_count,
        );
        snapshot
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"sample_rate_hz\":{},",
                "\"block_size\":{},",
                "\"processed_block_count\":{},",
                "\"last_block_sequence\":{},",
                "\"cpu_load_percent\":{},",
                "\"graph_latency_ms\":{},",
                "\"last_block_execution_time_ns\":{},",
                "\"last_block_deadline_budget_ns\":{},",
                "\"last_block_budget_utilization_percent\":{},",
                "\"last_block_budget_overrun_ns\":{},",
                "\"last_block_deadline_pressure\":\"{:?}\",",
                "\"budget_overrun_count\":{},",
                "\"peak_block_execution_time_ns\":{},",
                "\"peak_block_budget_utilization_percent\":{},",
                "\"peak_block_budget_overrun_ns\":{},",
                "\"xrun_count\":{},",
                "\"scheduler_phase_count\":{},",
                "\"scheduler_lane_count\":{},",
                "\"scheduler_dispatch_count\":{},",
                "\"scheduler_prepared_dispatch_count\":{},",
                "\"scheduler_realtime_dispatch_count\":{},",
                "\"scheduler_dispatch_handoff_count\":{},",
                "\"scheduler_topology_compatible\":{},",
                "\"scheduler_topology_requires_host_reinterpretation\":{},",
                "\"scheduler_topology_issue_count\":{},",
                "\"prework_service_state\":\"{:?}\",",
                "\"prework_service_pressure\":\"{:?}\",",
                "\"prework_service_semantic_policy\":\"{:?}\",",
                "\"pending_prework_target_count\":{},",
                "\"pending_prework_deferred_target_count\":{},",
                "\"prework_queue_depth\":{},",
                "\"prework_peak_queue_depth\":{},",
                "\"prework_service_cycle_count\":{},",
                "\"prework_service_starvation_count\":{},",
                "\"prework_service_throttle_count\":{},",
                "\"prework_service_yield_count\":{},",
                "\"last_prework_service_effective_cycles\":{},",
                "\"last_prework_service_budget_per_cycle\":{},",
                "\"last_prework_service_effective_budget_per_cycle\":{},",
                "\"last_prework_serviced_backlog_class\":{},",
                "\"transport_gate_active\":{},",
                "\"plugin_gate_active\":{},",
                "\"hot_latency_node_id\":{},",
                "\"hot_latency_node_group\":{},",
                "\"hot_latency_node_topology_role\":{},",
                "\"hot_latency_node_plugin_sandbox_id\":{},",
                "\"hot_latency_node_samples\":{},",
                "\"hot_latency_group\":{},",
                "\"hot_latency_group_node_count\":{},",
                "\"hot_latency_group_total_samples\":{},",
                "\"critical_path_lane\":{},",
                "\"critical_path_lane_node_count\":{},",
                "\"critical_path_lane_plugin_backed_node_count\":{},",
                "\"critical_path_lane_planning_group_count\":{},",
                "\"critical_path_lane_total_latency_samples\":{},",
                "\"worker_lane_summaries\":{},",
                "\"background_service_class\":{},",
                "\"background_service_decision\":{},",
                "\"background_service_reason\":{},",
                "\"background_queued_work_item_count\":{},",
                "\"background_deferred_work_item_count\":{},",
                "\"background_pending_cleanup_work_item_count\":{},",
                "\"background_pending_retry_work_item_count\":{},",
                "\"summary\":{}}}",
            ),
            self.sample_rate_hz,
            self.block_size,
            self.processed_block_count,
            json_option_u64(self.last_block_sequence),
            self.cpu_load_percent,
            self.graph_latency_ms,
            json_option_u64(self.last_block_execution_time_ns),
            json_option_u64(self.last_block_deadline_budget_ns),
            json_option_f32(self.last_block_budget_utilization_percent),
            json_option_u64(self.last_block_budget_overrun_ns),
            self.last_block_deadline_pressure,
            self.budget_overrun_count,
            self.peak_block_execution_time_ns,
            self.peak_block_budget_utilization_percent,
            self.peak_block_budget_overrun_ns,
            self.xrun_count,
            self.scheduler_phase_count,
            self.scheduler_lane_count,
            self.scheduler_dispatch_count,
            self.scheduler_prepared_dispatch_count,
            self.scheduler_realtime_dispatch_count,
            self.scheduler_dispatch_handoff_count,
            self.scheduler_topology_compatible,
            self.scheduler_topology_requires_host_reinterpretation,
            self.scheduler_topology_issue_count,
            self.prework_service_state,
            self.prework_service_pressure,
            self.prework_service_semantic_policy,
            self.pending_prework_target_count,
            self.pending_prework_deferred_target_count,
            self.prework_queue_depth,
            self.prework_peak_queue_depth,
            self.prework_service_cycle_count,
            self.prework_service_starvation_count,
            self.prework_service_throttle_count,
            self.prework_service_yield_count,
            self.last_prework_service_effective_cycles,
            json_option_u64(
                self.last_prework_service_budget_per_cycle
                    .map(|value| value as u64)
            ),
            json_option_u64(
                self.last_prework_service_effective_budget_per_cycle
                    .map(|value| value as u64),
            ),
            json_option_string(self.last_prework_serviced_backlog_class.as_deref()),
            self.transport_gate_active,
            self.plugin_gate_active,
            json_option_string(self.hot_latency_node_id.as_deref()),
            json_option_string(self.hot_latency_node_group.as_deref()),
            json_option_string(self.hot_latency_node_topology_role.as_deref()),
            json_option_string(self.hot_latency_node_plugin_sandbox_id.as_deref()),
            self.hot_latency_node_samples,
            json_option_string(self.hot_latency_group.as_deref()),
            self.hot_latency_group_node_count,
            self.hot_latency_group_total_samples,
            json_option_string(self.critical_path_lane.as_deref()),
            self.critical_path_lane_node_count,
            self.critical_path_lane_plugin_backed_node_count,
            self.critical_path_lane_planning_group_count,
            self.critical_path_lane_total_latency_samples,
            json_runtime_worker_lane_instrumentation_summaries(&self.worker_lane_summaries),
            json_option_string(
                self.background_service_class
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceClass::OfflineRenderQueue => "OfflineRenderQueue",
                        RuntimeDeferredServiceClass::OfflineRenderPurge => "OfflineRenderPurge",
                    }),
            ),
            json_option_string(self.background_service_decision.as_ref().map(
                |value| match value {
                    RuntimeDeferredServiceDecision::Run => "Run",
                    RuntimeDeferredServiceDecision::Defer => "Defer",
                    RuntimeDeferredServiceDecision::Throttle => "Throttle",
                    RuntimeDeferredServiceDecision::Abort => "Abort",
                }
            ),),
            json_option_string(
                self.background_service_reason
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceReason::Ready => "Ready",
                        RuntimeDeferredServiceReason::RealtimeActive => "RealtimeActive",
                        RuntimeDeferredServiceReason::PendingCleanup => "PendingCleanup",
                        RuntimeDeferredServiceReason::RecoveryDegraded => "RecoveryDegraded",
                        RuntimeDeferredServiceReason::SafeMode => "SafeMode",
                        RuntimeDeferredServiceReason::InvalidRequest => "InvalidRequest",
                    }),
            ),
            self.background_queued_work_item_count,
            self.background_deferred_work_item_count,
            self.background_pending_cleanup_work_item_count,
            self.background_pending_retry_work_item_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

fn runtime_worker_lane_instrumentation_summaries(
    engine_block_snapshot: &RuntimeEngineBlockSnapshot,
) -> Vec<RuntimeWorkerLaneInstrumentationSummary> {
    let mut lane_order = engine_block_snapshot.lane_order.clone();
    for node in &engine_block_snapshot.planned_nodes {
        let lane = runtime_lane_for_group(node.group);
        if !lane_order.contains(&lane) {
            lane_order.push(lane);
        }
    }

    let mut summaries = Vec::new();
    for lane in lane_order {
        let mut node_count = 0usize;
        let mut plugin_backed_node_count = 0usize;
        let mut planning_groups = Vec::new();
        let mut total_latency_samples = 0u32;
        let mut max_node_latency_samples = 0u32;

        for node in engine_block_snapshot
            .planned_nodes
            .iter()
            .filter(|node| runtime_lane_for_group(node.group) == lane)
        {
            node_count = node_count.saturating_add(1);
            if matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked) {
                plugin_backed_node_count = plugin_backed_node_count.saturating_add(1);
            }
            if !planning_groups.contains(&node.group) {
                planning_groups.push(node.group);
            }
            total_latency_samples = total_latency_samples.saturating_add(node.latency_samples);
            max_node_latency_samples = max_node_latency_samples.max(node.latency_samples);
        }

        if node_count > 0 {
            summaries.push(RuntimeWorkerLaneInstrumentationSummary {
                lane,
                node_count,
                plugin_backed_node_count,
                planning_group_count: planning_groups.len(),
                total_latency_samples,
                max_node_latency_samples,
            });
        }
    }

    summaries
}

fn runtime_graph_node_planning_group_name(group: GraphNodePlanningGroup) -> &'static str {
    match group {
        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
    }
}

fn runtime_graph_node_topology_role_name(role: GraphNodeTopologyRole) -> &'static str {
    match role {
        GraphNodeTopologyRole::Utility => "Utility",
        GraphNodeTopologyRole::TrackLane => "TrackLane",
        GraphNodeTopologyRole::Bus => "Bus",
        GraphNodeTopologyRole::Send => "Send",
        GraphNodeTopologyRole::Return => "Return",
        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
    }
}

fn runtime_execution_lane_name(lane: GraphExecutionLane) -> &'static str {
    match lane {
        GraphExecutionLane::Realtime => "Realtime",
        GraphExecutionLane::Anticipative => "Anticipative",
    }
}

fn runtime_prework_backlog_class_name(value: RuntimePreworkBacklogClass) -> &'static str {
    match value {
        RuntimePreworkBacklogClass::Immediate => "Immediate",
        RuntimePreworkBacklogClass::NearTerm => "NearTerm",
        RuntimePreworkBacklogClass::Deferred => "Deferred",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDomain {
    SameClock,
    CrossClock,
    Aggregate,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockFallbackState {
    Direct,
    RuntimeResampled,
    RecoveryConstrained,
    Unconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockTransitionState {
    InitialObservation,
    Stable,
    EnteredAggregateClock,
    EnteredCrossClockFallback,
    EnteredRecoveryFallback,
    ReturnedToDirect,
    LostConfiguration,
    Reconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostClockingSummary {
    pub clock_source: RuntimeHostClockSource,
    pub ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub processing_sample_rate_hz: u32,
    pub hardware_sample_rate_hz: u32,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub crossing_required: bool,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoHealthState {
    Ready,
    FallbackActive,
    Recovering,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoDeviceChangeState {
    Stable,
    PendingRestart,
    Recovering,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeExternalIoSnapshot {
    pub health_state: RuntimeExternalIoHealthState,
    pub device_change_state: RuntimeExternalIoDeviceChangeState,
    pub backend_name: String,
    pub active_output_device_id: String,
    pub active_output_device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub clock_source: RuntimeHostClockSource,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub fallback_active: bool,
    pub runtime_graph_id_matches_pump: bool,
    pub output_latency_samples: u32,
    pub estimated_output_latency_samples: u32,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

impl RuntimeHostIoSummary {
    pub fn build_external_io_snapshot(&self) -> RuntimeExternalIoSnapshot {
        let fallback_active = self.clocking.fallback_state != RuntimeHostClockFallbackState::Direct;
        let health_state = if self.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted {
            RuntimeExternalIoHealthState::Faulted
        } else if fallback_active {
            RuntimeExternalIoHealthState::FallbackActive
        } else if self.hardware.device_loss_count > 0
            || self.hardware.restart_attempt_count > 0
            || self.clocking.transition_state != RuntimeHostClockTransitionState::Stable
            || matches!(
                self.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
        {
            RuntimeExternalIoHealthState::Recovering
        } else {
            RuntimeExternalIoHealthState::Ready
        };
        let device_change_state = if self.audio_pump.stream_state
            == RuntimeHostAudioStreamState::Faulted
            && self.restart_failure_count() > 0
        {
            RuntimeExternalIoDeviceChangeState::Failed
        } else if self.hardware.restart_attempt_count > 0
            || self.clocking.transition_state
                == RuntimeHostClockTransitionState::EnteredRecoveryFallback
        {
            RuntimeExternalIoDeviceChangeState::Recovering
        } else if self.hardware.device_loss_count > 0
            || self.clocking.transition_state != RuntimeHostClockTransitionState::Stable
        {
            RuntimeExternalIoDeviceChangeState::PendingRestart
        } else {
            RuntimeExternalIoDeviceChangeState::Stable
        };

        RuntimeExternalIoSnapshot {
            health_state,
            device_change_state,
            backend_name: self.hardware.backend_name.clone(),
            active_output_device_id: self.hardware.device_id.clone(),
            active_output_device_name: self.hardware.device_name.clone(),
            stream_state: self.audio_pump.stream_state,
            clock_source: self.clocking.clock_source,
            clock_domain: self.clocking.clock_domain,
            fallback_state: self.clocking.fallback_state,
            transition_state: self.clocking.transition_state,
            fallback_active,
            runtime_graph_id_matches_pump: self.runtime_graph_id_matches_pump,
            output_latency_samples: self.latency.output_latency_samples,
            estimated_output_latency_samples: self.latency.estimated_output_latency_samples,
            xrun_count: self.hardware.xrun_count,
            callback_overrun_count: self.hardware.callback_overrun_count,
            device_loss_count: self.hardware.device_loss_count,
            restart_attempt_count: self.hardware.restart_attempt_count,
            restart_failure_count: self.hardware.restart_failure_count,
            summary: format!(
                "health={health_state:?} device_change={device_change_state:?} backend={} device={} stream={:?} clock={:?}/{:?}/{:?} fallback={} graph_matches={} output_latency={} estimated_output_latency={} xruns={} overruns={} device_losses={} restart_attempts={} restart_failures={}",
                self.hardware.backend_name,
                self.hardware.device_id,
                self.audio_pump.stream_state,
                self.clocking.clock_source,
                self.clocking.clock_domain,
                self.clocking.transition_state,
                fallback_active,
                self.runtime_graph_id_matches_pump,
                self.latency.output_latency_samples,
                self.latency.estimated_output_latency_samples,
                self.hardware.xrun_count,
                self.hardware.callback_overrun_count,
                self.hardware.device_loss_count,
                self.hardware.restart_attempt_count,
                self.hardware.restart_failure_count,
            ),
        }
    }

    fn restart_failure_count(&self) -> u64 {
        self.hardware.restart_failure_count
    }
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
            "{} host_backend={} host_device={} host_stream_state={:?} host_clock_source={:?} host_clock_domain={:?} host_clock_fallback_state={:?} host_clock_transition_state={:?} host_clock_crossing_required={} host_clock_processing_sample_rate={} host_clock_hardware_sample_rate={} host_clock_ownership={:?} host_clock_restart_policy={:?} host_callback_interval_ms={:.3} host_output_latency_samples={} host_graph_latency_samples={} host_estimated_output_latency_samples={} host_backend_health={:?} host_backend_xruns={} host_backend_device_losses={} host_backend_restart_attempts={} host_backend_restart_failures={} host_audio_callbacks={} host_audio_frames={} host_audio_copied_samples={} host_audio_zero_filled_samples={} host_audio_dropped_samples={} host_audio_peak={:?} host_audio_graph={:?} host_audio_graph_matches_runtime={}",
            self.observation.render_compact(),
            self.host_io.hardware.backend_name,
            self.host_io.hardware.device_id,
            self.host_io.audio_pump.stream_state,
            self.host_io.clocking.clock_source,
            self.host_io.clocking.clock_domain,
            self.host_io.clocking.fallback_state,
            self.host_io.clocking.transition_state,
            self.host_io.clocking.crossing_required,
            self.host_io.clocking.processing_sample_rate_hz,
            self.host_io.clocking.hardware_sample_rate_hz,
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
                "\nhost_clock_domain={:?}",
                "\nhost_clock_fallback_state={:?}",
                "\nhost_clock_transition_state={:?}",
                "\nhost_clock_crossing_required={}",
                "\nhost_clock_processing_sample_rate_hz={}",
                "\nhost_clock_hardware_sample_rate_hz={}",
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
            self.host_io.clocking.clock_domain,
            self.host_io.clocking.fallback_state,
            self.host_io.clocking.transition_state,
            self.host_io.clocking.crossing_required,
            self.host_io.clocking.processing_sample_rate_hz,
            self.host_io.clocking.hardware_sample_rate_hz,
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
                "\"fault_status\":{},",
                "\"fault_diagnostic_receipt\":{},",
                "\"interruption_summary\":{},",
                "\"degradation_summary\":{},",
                "\"metering_snapshot\":{},",
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
                "\"clock_domain\":{},",
                "\"fallback_state\":{},",
                "\"transition_state\":{},",
                "\"crossing_required\":{},",
                "\"processing_sample_rate_hz\":{},",
                "\"hardware_sample_rate_hz\":{},",
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
            json_runtime_fault_status(&self.observation.fault_status),
            json_runtime_fault_diagnostic_receipt(&self.observation.fault_diagnostic_receipt),
            json_runtime_interruption_summary(&self.observation.interruption_summary),
            json_runtime_degradation_summary(&self.observation.degradation_summary),
            json_runtime_metering_snapshot(&self.observation.metering_snapshot),
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
            json_option_string(Some(match self.host_io.clocking.clock_domain {
                RuntimeHostClockDomain::SameClock => "SameClock",
                RuntimeHostClockDomain::CrossClock => "CrossClock",
                RuntimeHostClockDomain::Aggregate => "Aggregate",
                RuntimeHostClockDomain::Degraded => "Degraded",
            })),
            json_option_string(Some(match self.host_io.clocking.fallback_state {
                RuntimeHostClockFallbackState::Direct => "Direct",
                RuntimeHostClockFallbackState::RuntimeResampled => "RuntimeResampled",
                RuntimeHostClockFallbackState::RecoveryConstrained => "RecoveryConstrained",
                RuntimeHostClockFallbackState::Unconfigured => "Unconfigured",
            })),
            json_option_string(Some(match self.host_io.clocking.transition_state {
                RuntimeHostClockTransitionState::InitialObservation => "InitialObservation",
                RuntimeHostClockTransitionState::Stable => "Stable",
                RuntimeHostClockTransitionState::EnteredAggregateClock => {
                    "EnteredAggregateClock"
                }
                RuntimeHostClockTransitionState::EnteredCrossClockFallback => {
                    "EnteredCrossClockFallback"
                }
                RuntimeHostClockTransitionState::EnteredRecoveryFallback => {
                    "EnteredRecoveryFallback"
                }
                RuntimeHostClockTransitionState::ReturnedToDirect => "ReturnedToDirect",
                RuntimeHostClockTransitionState::LostConfiguration => "LostConfiguration",
                RuntimeHostClockTransitionState::Reconfigured => "Reconfigured",
            })),
            self.host_io.clocking.crossing_required,
            self.host_io.clocking.processing_sample_rate_hz,
            self.host_io.clocking.hardware_sample_rate_hz,
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

impl From<HardwareClockTopology> for RuntimeHostClockDomain {
    fn from(value: HardwareClockTopology) -> Self {
        match value {
            HardwareClockTopology::SingleEndpoint => Self::SameClock,
            HardwareClockTopology::Aggregate => Self::Aggregate,
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

    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(
            &self.observation.observation,
            Some(&self.observation.host_io),
        )
    }

    pub fn soak_receipt(&self) -> RuntimeSoakReceipt {
        build_runtime_soak_receipt(&self.observation.observation, self.events.len())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeProfilingReceipt {
    pub sample_rate_hz: u32,
    pub block_size: usize,
    pub engine_processed_blocks: u64,
    pub engine_last_block_sequence: Option<u64>,
    pub engine_node_count: usize,
    pub engine_stage_count: usize,
    pub engine_total_latency_samples: u32,
    pub engine_total_tail_samples: u32,
    pub runtime_cpu_load_percent: f32,
    pub runtime_graph_latency_ms: f32,
    pub runtime_xrun_count: u64,
    pub active_plugin_sandboxes: u32,
    pub readiness_degraded: bool,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub recovery_overlap_sessions: usize,
    pub lingering_sessions: usize,
    pub detach_faulted_sessions: usize,
    pub plugin_chain_stage_count: usize,
    pub plugin_chain_degraded_stage_count: usize,
    pub plugin_chain_missing_binding_stage_count: usize,
    pub plugin_chain_total_planned_latency_samples: u32,
    pub plugin_chain_total_realized_latency_samples: u32,
    pub plugin_chain_total_tail_samples: u32,
    pub output_peak: Option<f32>,
    pub output_rms: Option<f32>,
    pub host_callback_count: Option<u64>,
    pub host_callback_interval_ms: Option<f32>,
    pub host_output_latency_ms: Option<f32>,
    pub host_graph_latency_ms: Option<f32>,
    pub host_estimated_output_latency_ms: Option<f32>,
    pub host_backend_xrun_count: Option<u64>,
    pub host_callback_overrun_count: Option<u64>,
    pub host_device_loss_count: Option<u64>,
    pub host_restart_attempt_count: Option<u64>,
    pub host_restart_failure_count: Option<u64>,
    pub host_copied_output_samples: Option<u64>,
    pub host_zero_filled_output_samples: Option<u64>,
    pub host_dropped_output_samples: Option<u64>,
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    pub summary: String,
}

impl RuntimeProfilingReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "sample_rate_hz={}",
                "\nblock_size={}",
                "\nengine_processed_blocks={}",
                "\nengine_last_block_sequence={:?}",
                "\nengine_node_count={}",
                "\nengine_stage_count={}",
                "\nengine_total_latency_samples={}",
                "\nengine_total_tail_samples={}",
                "\nruntime_cpu_load_percent={:.3}",
                "\nruntime_graph_latency_ms={:.3}",
                "\nruntime_xrun_count={}",
                "\nactive_plugin_sandboxes={}",
                "\nreadiness_degraded={}",
                "\ntransport_gate_active={}",
                "\nplugin_gate_active={}",
                "\ndegraded_bound_plugin_sandboxes={}",
                "\nmissing_bound_plugin_sandboxes={}",
                "\nrecovery_overlap_sessions={}",
                "\nlingering_sessions={}",
                "\ndetach_faulted_sessions={}",
                "\nplugin_chain_stage_count={}",
                "\nplugin_chain_degraded_stage_count={}",
                "\nplugin_chain_missing_binding_stage_count={}",
                "\nplugin_chain_total_planned_latency_samples={}",
                "\nplugin_chain_total_realized_latency_samples={}",
                "\nplugin_chain_total_tail_samples={}",
                "\noutput_peak={:?}",
                "\noutput_rms={:?}",
                "\nhost_callback_count={:?}",
                "\nhost_callback_interval_ms={:?}",
                "\nhost_output_latency_ms={:?}",
                "\nhost_graph_latency_ms={:?}",
                "\nhost_estimated_output_latency_ms={:?}",
                "\nhost_backend_xrun_count={:?}",
                "\nhost_callback_overrun_count={:?}",
                "\nhost_device_loss_count={:?}",
                "\nhost_restart_attempt_count={:?}",
                "\nhost_restart_failure_count={:?}",
                "\nhost_copied_output_samples={:?}",
                "\nhost_zero_filled_output_samples={:?}",
                "\nhost_dropped_output_samples={:?}",
                "\nfault_diagnostic_primary_family={:?}",
                "\nfault_diagnostic_primary_fault_cause={:?}",
                "\nfault_diagnostic_interruption_class={:?}",
                "\nfault_diagnostic_contribution_count={}",
                "\nsummary={}",
            ),
            self.sample_rate_hz,
            self.block_size,
            self.engine_processed_blocks,
            self.engine_last_block_sequence,
            self.engine_node_count,
            self.engine_stage_count,
            self.engine_total_latency_samples,
            self.engine_total_tail_samples,
            self.runtime_cpu_load_percent,
            self.runtime_graph_latency_ms,
            self.runtime_xrun_count,
            self.active_plugin_sandboxes,
            self.readiness_degraded,
            self.transport_gate_active,
            self.plugin_gate_active,
            self.degraded_bound_plugin_sandboxes,
            self.missing_bound_plugin_sandboxes,
            self.recovery_overlap_sessions,
            self.lingering_sessions,
            self.detach_faulted_sessions,
            self.plugin_chain_stage_count,
            self.plugin_chain_degraded_stage_count,
            self.plugin_chain_missing_binding_stage_count,
            self.plugin_chain_total_planned_latency_samples,
            self.plugin_chain_total_realized_latency_samples,
            self.plugin_chain_total_tail_samples,
            self.output_peak,
            self.output_rms,
            self.host_callback_count,
            self.host_callback_interval_ms,
            self.host_output_latency_ms,
            self.host_graph_latency_ms,
            self.host_estimated_output_latency_ms,
            self.host_backend_xrun_count,
            self.host_callback_overrun_count,
            self.host_device_loss_count,
            self.host_restart_attempt_count,
            self.host_restart_failure_count,
            self.host_copied_output_samples,
            self.host_zero_filled_output_samples,
            self.host_dropped_output_samples,
            self.fault_diagnostic_receipt.primary_family,
            self.fault_diagnostic_receipt.primary_fault_cause,
            self.fault_diagnostic_receipt.interruption_class,
            self.fault_diagnostic_receipt.contributions.len(),
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"sample_rate_hz\":{},",
                "\"block_size\":{},",
                "\"engine_processed_blocks\":{},",
                "\"engine_last_block_sequence\":{},",
                "\"engine_node_count\":{},",
                "\"engine_stage_count\":{},",
                "\"engine_total_latency_samples\":{},",
                "\"engine_total_tail_samples\":{},",
                "\"runtime_cpu_load_percent\":{},",
                "\"runtime_graph_latency_ms\":{},",
                "\"runtime_xrun_count\":{},",
                "\"active_plugin_sandboxes\":{},",
                "\"readiness_degraded\":{},",
                "\"transport_gate_active\":{},",
                "\"plugin_gate_active\":{},",
                "\"degraded_bound_plugin_sandboxes\":{},",
                "\"missing_bound_plugin_sandboxes\":{},",
                "\"recovery_overlap_sessions\":{},",
                "\"lingering_sessions\":{},",
                "\"detach_faulted_sessions\":{},",
                "\"plugin_chain_stage_count\":{},",
                "\"plugin_chain_degraded_stage_count\":{},",
                "\"plugin_chain_missing_binding_stage_count\":{},",
                "\"plugin_chain_total_planned_latency_samples\":{},",
                "\"plugin_chain_total_realized_latency_samples\":{},",
                "\"plugin_chain_total_tail_samples\":{},",
                "\"output_peak\":{},",
                "\"output_rms\":{},",
                "\"host_callback_count\":{},",
                "\"host_callback_interval_ms\":{},",
                "\"host_output_latency_ms\":{},",
                "\"host_graph_latency_ms\":{},",
                "\"host_estimated_output_latency_ms\":{},",
                "\"host_backend_xrun_count\":{},",
                "\"host_callback_overrun_count\":{},",
                "\"host_device_loss_count\":{},",
                "\"host_restart_attempt_count\":{},",
                "\"host_restart_failure_count\":{},",
                "\"host_copied_output_samples\":{},",
                "\"host_zero_filled_output_samples\":{},",
                "\"host_dropped_output_samples\":{},",
                "\"fault_diagnostic_receipt\":{},",
                "\"summary\":{}",
                "}}"
            ),
            self.sample_rate_hz,
            self.block_size,
            self.engine_processed_blocks,
            json_option_u64(self.engine_last_block_sequence),
            self.engine_node_count,
            self.engine_stage_count,
            self.engine_total_latency_samples,
            self.engine_total_tail_samples,
            self.runtime_cpu_load_percent,
            self.runtime_graph_latency_ms,
            self.runtime_xrun_count,
            self.active_plugin_sandboxes,
            self.readiness_degraded,
            self.transport_gate_active,
            self.plugin_gate_active,
            self.degraded_bound_plugin_sandboxes,
            self.missing_bound_plugin_sandboxes,
            self.recovery_overlap_sessions,
            self.lingering_sessions,
            self.detach_faulted_sessions,
            self.plugin_chain_stage_count,
            self.plugin_chain_degraded_stage_count,
            self.plugin_chain_missing_binding_stage_count,
            self.plugin_chain_total_planned_latency_samples,
            self.plugin_chain_total_realized_latency_samples,
            self.plugin_chain_total_tail_samples,
            json_option_f32(self.output_peak),
            json_option_f32(self.output_rms),
            json_option_u64(self.host_callback_count),
            json_option_f32(self.host_callback_interval_ms),
            json_option_f32(self.host_output_latency_ms),
            json_option_f32(self.host_graph_latency_ms),
            json_option_f32(self.host_estimated_output_latency_ms),
            json_option_u64(self.host_backend_xrun_count),
            json_option_u64(self.host_callback_overrun_count),
            json_option_u64(self.host_device_loss_count),
            json_option_u64(self.host_restart_attempt_count),
            json_option_u64(self.host_restart_failure_count),
            json_option_u64(self.host_copied_output_samples),
            json_option_u64(self.host_zero_filled_output_samples),
            json_option_u64(self.host_dropped_output_samples),
            json_runtime_fault_diagnostic_receipt(&self.fault_diagnostic_receipt),
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceTraceReceipt {
    pub observation_count: usize,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub processed_block_span: u64,
    pub peak_cpu_load_percent: f32,
    pub peak_graph_latency_ms: f32,
    pub peak_block_execution_time_ns: u64,
    pub peak_block_budget_utilization_percent: f32,
    pub peak_block_budget_overrun_ns: u64,
    pub peak_pending_prework_target_count: usize,
    pub peak_prework_queue_depth: usize,
    pub peak_background_queued_work_item_count: usize,
    pub peak_background_deferred_work_item_count: usize,
    pub playback_active_observation_count: usize,
    pub recording_active_observation_count: usize,
    pub background_service_run_count: usize,
    pub background_service_defer_count: usize,
    pub background_service_throttle_count: usize,
    pub background_service_abort_count: usize,
    pub background_service_while_playing_count: usize,
    pub background_service_while_recording_count: usize,
    pub topology_incompatible_observation_count: usize,
    pub elevated_deadline_pressure_observation_count: usize,
    pub critical_deadline_pressure_observation_count: usize,
    pub overrun_deadline_pressure_observation_count: usize,
    pub budget_overrun_count_delta: u64,
    pub xrun_count_delta: u64,
    pub prework_service_starvation_count_delta: u64,
    pub prework_service_throttle_count_delta: u64,
    pub prework_service_yield_count_delta: u64,
    pub peak_hot_latency_node_id: Option<String>,
    pub peak_hot_latency_node_group: Option<String>,
    pub peak_hot_latency_node_samples: u32,
    pub peak_hot_latency_group: Option<String>,
    pub peak_hot_latency_group_node_count: usize,
    pub peak_hot_latency_group_total_samples: u32,
    pub peak_critical_path_lane: Option<String>,
    pub peak_critical_path_lane_node_count: usize,
    pub peak_critical_path_lane_plugin_backed_node_count: usize,
    pub peak_critical_path_lane_total_latency_samples: u32,
    pub summary: String,
}

impl RuntimePerformanceTraceReceipt {
    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"observation_count\":{},",
                "\"first_block_sequence\":{},",
                "\"last_block_sequence\":{},",
                "\"processed_block_span\":{},",
                "\"peak_cpu_load_percent\":{},",
                "\"peak_graph_latency_ms\":{},",
                "\"peak_block_execution_time_ns\":{},",
                "\"peak_block_budget_utilization_percent\":{},",
                "\"peak_block_budget_overrun_ns\":{},",
                "\"peak_pending_prework_target_count\":{},",
                "\"peak_prework_queue_depth\":{},",
                "\"peak_background_queued_work_item_count\":{},",
                "\"peak_background_deferred_work_item_count\":{},",
                "\"playback_active_observation_count\":{},",
                "\"recording_active_observation_count\":{},",
                "\"background_service_run_count\":{},",
                "\"background_service_defer_count\":{},",
                "\"background_service_throttle_count\":{},",
                "\"background_service_abort_count\":{},",
                "\"background_service_while_playing_count\":{},",
                "\"background_service_while_recording_count\":{},",
                "\"topology_incompatible_observation_count\":{},",
                "\"elevated_deadline_pressure_observation_count\":{},",
                "\"critical_deadline_pressure_observation_count\":{},",
                "\"overrun_deadline_pressure_observation_count\":{},",
                "\"budget_overrun_count_delta\":{},",
                "\"xrun_count_delta\":{},",
                "\"prework_service_starvation_count_delta\":{},",
                "\"prework_service_throttle_count_delta\":{},",
                "\"prework_service_yield_count_delta\":{},",
                "\"peak_hot_latency_node_id\":{},",
                "\"peak_hot_latency_node_group\":{},",
                "\"peak_hot_latency_node_samples\":{},",
                "\"peak_hot_latency_group\":{},",
                "\"peak_hot_latency_group_node_count\":{},",
                "\"peak_hot_latency_group_total_samples\":{},",
                "\"peak_critical_path_lane\":{},",
                "\"peak_critical_path_lane_node_count\":{},",
                "\"peak_critical_path_lane_plugin_backed_node_count\":{},",
                "\"peak_critical_path_lane_total_latency_samples\":{},",
                "\"summary\":{}",
                "}}"
            ),
            self.observation_count,
            json_option_u64(self.first_block_sequence),
            json_option_u64(self.last_block_sequence),
            self.processed_block_span,
            self.peak_cpu_load_percent,
            self.peak_graph_latency_ms,
            self.peak_block_execution_time_ns,
            self.peak_block_budget_utilization_percent,
            self.peak_block_budget_overrun_ns,
            self.peak_pending_prework_target_count,
            self.peak_prework_queue_depth,
            self.peak_background_queued_work_item_count,
            self.peak_background_deferred_work_item_count,
            self.playback_active_observation_count,
            self.recording_active_observation_count,
            self.background_service_run_count,
            self.background_service_defer_count,
            self.background_service_throttle_count,
            self.background_service_abort_count,
            self.background_service_while_playing_count,
            self.background_service_while_recording_count,
            self.topology_incompatible_observation_count,
            self.elevated_deadline_pressure_observation_count,
            self.critical_deadline_pressure_observation_count,
            self.overrun_deadline_pressure_observation_count,
            self.budget_overrun_count_delta,
            self.xrun_count_delta,
            self.prework_service_starvation_count_delta,
            self.prework_service_throttle_count_delta,
            self.prework_service_yield_count_delta,
            json_option_string(self.peak_hot_latency_node_id.as_deref()),
            json_option_string(self.peak_hot_latency_node_group.as_deref()),
            self.peak_hot_latency_node_samples,
            json_option_string(self.peak_hot_latency_group.as_deref()),
            self.peak_hot_latency_group_node_count,
            self.peak_hot_latency_group_total_samples,
            json_option_string(self.peak_critical_path_lane.as_deref()),
            self.peak_critical_path_lane_node_count,
            self.peak_critical_path_lane_plugin_backed_node_count,
            self.peak_critical_path_lane_total_latency_samples,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSoakReceipt {
    pub event_stream_count: usize,
    pub restart_count: u64,
    pub stop_count: u64,
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub readiness_degraded: bool,
    pub plugin_fault_count: usize,
    pub recovery_event_count: usize,
    pub lifecycle_event_count: usize,
    pub transport_event_count: usize,
    pub heartbeat_event_count: usize,
    pub block_dispatch_event_count: usize,
    pub lease_rollover_event_count: usize,
    pub invalidation_event_count: usize,
    pub completion_slot_event_count: usize,
    pub transport_fault_event_count: usize,
    pub broker_failure_event_count: usize,
    pub sandbox_operation_failure_event_count: usize,
    pub peak_attached_sessions: usize,
    pub peak_recovery_overlap_sessions: usize,
    pub peak_lingering_sessions: usize,
    pub pending_cleanup_waves: usize,
    pub plugin_ready_sandbox_count: usize,
    pub plugin_degraded_sandbox_count: usize,
    pub plugin_faulted_sandbox_count: usize,
    pub plugin_restarting_sandbox_count: usize,
    pub plugin_quarantined_sandbox_count: usize,
    pub recall_stage_count: usize,
    pub recovered_recall_stage_count: usize,
    pub unavailable_recall_stage_count: usize,
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub summary: String,
}

impl RuntimeSoakReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "event_stream_count={}",
                "\nrestart_count={}",
                "\nstop_count={}",
                "\nwatchdog_restart_count={}",
                "\nsafe_mode_enabled={}",
                "\nreadiness_degraded={}",
                "\nplugin_fault_count={}",
                "\nrecovery_event_count={}",
                "\nlifecycle_event_count={}",
                "\ntransport_event_count={}",
                "\nheartbeat_event_count={}",
                "\nblock_dispatch_event_count={}",
                "\nlease_rollover_event_count={}",
                "\ninvalidation_event_count={}",
                "\ncompletion_slot_event_count={}",
                "\ntransport_fault_event_count={}",
                "\nbroker_failure_event_count={}",
                "\nsandbox_operation_failure_event_count={}",
                "\npeak_attached_sessions={}",
                "\npeak_recovery_overlap_sessions={}",
                "\npeak_lingering_sessions={}",
                "\npending_cleanup_waves={}",
                "\nplugin_ready_sandbox_count={}",
                "\nplugin_degraded_sandbox_count={}",
                "\nplugin_faulted_sandbox_count={}",
                "\nplugin_restarting_sandbox_count={}",
                "\nplugin_quarantined_sandbox_count={}",
                "\nrecall_stage_count={}",
                "\nrecovered_recall_stage_count={}",
                "\nunavailable_recall_stage_count={}",
                "\nlast_recovery_intent={:?}",
                "\nlast_stop_reason={:?}",
                "\nsummary={}",
            ),
            self.event_stream_count,
            self.restart_count,
            self.stop_count,
            self.watchdog_restart_count,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.plugin_fault_count,
            self.recovery_event_count,
            self.lifecycle_event_count,
            self.transport_event_count,
            self.heartbeat_event_count,
            self.block_dispatch_event_count,
            self.lease_rollover_event_count,
            self.invalidation_event_count,
            self.completion_slot_event_count,
            self.transport_fault_event_count,
            self.broker_failure_event_count,
            self.sandbox_operation_failure_event_count,
            self.peak_attached_sessions,
            self.peak_recovery_overlap_sessions,
            self.peak_lingering_sessions,
            self.pending_cleanup_waves,
            self.plugin_ready_sandbox_count,
            self.plugin_degraded_sandbox_count,
            self.plugin_faulted_sandbox_count,
            self.plugin_restarting_sandbox_count,
            self.plugin_quarantined_sandbox_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            self.last_recovery_intent,
            self.last_stop_reason,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"event_stream_count\":{},",
                "\"restart_count\":{},",
                "\"stop_count\":{},",
                "\"watchdog_restart_count\":{},",
                "\"safe_mode_enabled\":{},",
                "\"readiness_degraded\":{},",
                "\"plugin_fault_count\":{},",
                "\"recovery_event_count\":{},",
                "\"lifecycle_event_count\":{},",
                "\"transport_event_count\":{},",
                "\"heartbeat_event_count\":{},",
                "\"block_dispatch_event_count\":{},",
                "\"lease_rollover_event_count\":{},",
                "\"invalidation_event_count\":{},",
                "\"completion_slot_event_count\":{},",
                "\"transport_fault_event_count\":{},",
                "\"broker_failure_event_count\":{},",
                "\"sandbox_operation_failure_event_count\":{},",
                "\"peak_attached_sessions\":{},",
                "\"peak_recovery_overlap_sessions\":{},",
                "\"peak_lingering_sessions\":{},",
                "\"pending_cleanup_waves\":{},",
                "\"plugin_ready_sandbox_count\":{},",
                "\"plugin_degraded_sandbox_count\":{},",
                "\"plugin_faulted_sandbox_count\":{},",
                "\"plugin_restarting_sandbox_count\":{},",
                "\"plugin_quarantined_sandbox_count\":{},",
                "\"recall_stage_count\":{},",
                "\"recovered_recall_stage_count\":{},",
                "\"unavailable_recall_stage_count\":{},",
                "\"last_recovery_intent\":{},",
                "\"last_stop_reason\":{},",
                "\"summary\":{}",
                "}}"
            ),
            self.event_stream_count,
            self.restart_count,
            self.stop_count,
            self.watchdog_restart_count,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.plugin_fault_count,
            self.recovery_event_count,
            self.lifecycle_event_count,
            self.transport_event_count,
            self.heartbeat_event_count,
            self.block_dispatch_event_count,
            self.lease_rollover_event_count,
            self.invalidation_event_count,
            self.completion_slot_event_count,
            self.transport_fault_event_count,
            self.broker_failure_event_count,
            self.sandbox_operation_failure_event_count,
            self.peak_attached_sessions,
            self.peak_recovery_overlap_sessions,
            self.peak_lingering_sessions,
            self.pending_cleanup_waves,
            self.plugin_ready_sandbox_count,
            self.plugin_degraded_sandbox_count,
            self.plugin_faulted_sandbox_count,
            self.plugin_restarting_sandbox_count,
            self.plugin_quarantined_sandbox_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            json_option_string(
                self.last_recovery_intent
                    .as_ref()
                    .map(|intent| match intent {
                        RecoveryRestartIntent::WatchdogRecovery => "WatchdogRecovery",
                        RecoveryRestartIntent::CrashRecovery => "CrashRecovery",
                    }),
            ),
            json_option_string(self.last_stop_reason.as_ref().map(|reason| match reason {
                StopReason::UserRequested => "UserRequested",
                StopReason::DegradedModeRecovery => "DegradedModeRecovery",
                StopReason::DeviceReconfigure => "DeviceReconfigure",
            })),
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAcceptanceReceipt {
    pub runtime_lane_count: usize,
    pub runtime_ready_lane_count: usize,
    pub playback_ready: bool,
    pub recording_ready: bool,
    pub media_ready: bool,
    pub clip_processing_ready: bool,
    pub plugin_ready: bool,
    pub recovery_ready: bool,
    pub minimum_trace_observation_count: usize,
    pub minimum_soak_event_count: usize,
    pub summary: String,
}

impl RuntimeAcceptanceReceipt {
    pub fn capture(runtime: &impl RuntimeObservationApi) -> Self {
        build_runtime_acceptance_receipt(
            runtime.get_readiness(),
            runtime.get_effective_config(),
            runtime.get_control_snapshot(),
            runtime.get_scheduler_topology_summary(),
            runtime.get_recording_capture_snapshot(),
            runtime.get_media_service_snapshot(),
            runtime.get_clip_processing_pipeline_snapshot(),
            runtime.get_plugin_lifecycle_snapshot(),
        )
    }
}

impl RuntimeObservationReport {
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(self, None)
    }

    pub fn build_performance_trace_receipt(
        observations: &[Self],
    ) -> RuntimePerformanceTraceReceipt {
        build_runtime_performance_trace_receipt(observations)
    }

    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        RuntimePerformanceSnapshot::capture(
            &self.effective_config,
            &self.diagnostics_snapshot,
            &self.engine_block_snapshot,
            self.last_deferred_service_receipt.as_ref(),
        )
    }
}

impl RuntimeSupervisorReport {
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        self.observation.profiling_receipt()
    }

    pub fn build_performance_trace_receipt(reports: &[Self]) -> RuntimePerformanceTraceReceipt {
        let observations = reports
            .iter()
            .map(|report| report.observation.clone())
            .collect::<Vec<_>>();
        RuntimeObservationReport::build_performance_trace_receipt(&observations)
    }

    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        self.observation.performance_snapshot()
    }

    pub fn soak_receipt(&self) -> RuntimeSoakReceipt {
        build_runtime_soak_receipt(&self.observation, self.events.len())
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
    pub console_group_ids: Vec<String>,
    pub send_return_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRoutedPluginChainSummary {
    pub chain_count: usize,
    pub stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub chain_ids: Vec<String>,
    pub node_ids: Vec<String>,
    pub sandbox_ids: Vec<String>,
}

impl Default for RuntimeRoutedPluginChainSummary {
    fn default() -> Self {
        Self {
            chain_count: 0,
            stage_count: 0,
            pending_render_stage_count: 0,
            settling_stage_count: 0,
            compensated_stage_count: 0,
            degraded_stage_count: 0,
            bypassed_stage_count: 0,
            missing_binding_stage_count: 0,
            total_planned_latency_samples: 0,
            total_realized_latency_samples: 0,
            total_tail_samples: 0,
            chain_ids: Vec::new(),
            node_ids: Vec::new(),
            sandbox_ids: Vec::new(),
        }
    }
}

impl RuntimeRoutedPluginChainSummary {
    fn include_chain(&mut self, chain: &RuntimePluginExecutionChainSummary) {
        if !self.chain_ids.contains(&chain.chain_id) {
            self.chain_count = self.chain_count.saturating_add(1);
            self.chain_ids.push(chain.chain_id.clone());
        }
        self.stage_count = self.stage_count.saturating_add(chain.stage_count);
        self.pending_render_stage_count = self
            .pending_render_stage_count
            .saturating_add(chain.pending_render_stage_count);
        self.settling_stage_count = self
            .settling_stage_count
            .saturating_add(chain.settling_stage_count);
        self.compensated_stage_count = self
            .compensated_stage_count
            .saturating_add(chain.compensated_stage_count);
        self.degraded_stage_count = self
            .degraded_stage_count
            .saturating_add(chain.degraded_stage_count);
        self.bypassed_stage_count = self
            .bypassed_stage_count
            .saturating_add(chain.bypassed_stage_count);
        self.missing_binding_stage_count = self
            .missing_binding_stage_count
            .saturating_add(chain.missing_binding_stage_count);
        self.total_planned_latency_samples = self
            .total_planned_latency_samples
            .saturating_add(chain.total_planned_latency_samples);
        self.total_realized_latency_samples = self
            .total_realized_latency_samples
            .saturating_add(chain.total_realized_latency_samples);
        self.total_tail_samples = self
            .total_tail_samples
            .saturating_add(chain.total_tail_samples);
        for stage in &chain.stages {
            if !self.node_ids.contains(&stage.node_id) {
                self.node_ids.push(stage.node_id.clone());
            }
            if let Some(sandbox_id) = &stage.sandbox_id {
                if !self.sandbox_ids.contains(sandbox_id) {
                    self.sandbox_ids.push(sandbox_id.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerTrackLaneSummary {
    pub track_lane_id: String,
    pub node_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerBusGroupSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerConsoleGroupSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerSendReturnSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    pub node_id: String,
    pub lane: GraphExecutionLane,
    pub group: GraphNodePlanningGroup,
    pub execution_class: GraphNodeExecutionClass,
    pub topology_role: GraphNodeTopologyRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub plugin_sandbox_id: Option<String>,
    pub plugin_recall_state: Option<RuntimePluginRecallState>,
    pub plugin_recall: Option<RuntimePluginRecallSnapshot>,
    pub plugin_compensation_state: Option<RuntimePluginCompensationState>,
    pub plugin_realized_latency_samples: Option<u32>,
    pub plugin_tail_samples: Option<u32>,
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
    pub send_return_group_count: usize,
    pub console_group_count: usize,
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    pub track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    pub bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    pub console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    pub send_returns: Vec<RuntimeMixerSendReturnSummary>,
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

impl RuntimeExecutionTopologySummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        let mut track_lane_ids = std::collections::BTreeSet::new();
        let mut bus_group_ids = std::collections::BTreeSet::new();
        let mut send_return_ids = std::collections::BTreeSet::new();
        let mut console_group_ids = std::collections::BTreeSet::new();
        let mut lanes = Vec::new();

        for lane in &snapshot.lane_order {
            let mut groups = Vec::new();
            let mut node_ids = Vec::new();
            let mut topology_roles = Vec::new();
            let mut lane_ids = Vec::new();
            let mut bus_groups = Vec::new();
            let mut console_groups = Vec::new();
            let mut send_returns = Vec::new();

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
                if let Some(track_lane_id) = &node.track_lane_id {
                    if !lane_ids.contains(track_lane_id) {
                        lane_ids.push(track_lane_id.clone());
                    }
                    track_lane_ids.insert(track_lane_id.clone());
                }
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !bus_groups.contains(bus_group_id) {
                        bus_groups.push(bus_group_id.clone());
                    }
                    bus_group_ids.insert(bus_group_id.clone());
                }
                if let Some(console_group_id) = &node.console_group_id {
                    if !console_groups.contains(console_group_id) {
                        console_groups.push(console_group_id.clone());
                    }
                    console_group_ids.insert(console_group_id.clone());
                }
                if let Some(send_return_id) = &node.send_return_id {
                    if !send_returns.contains(send_return_id) {
                        send_returns.push(send_return_id.clone());
                    }
                    send_return_ids.insert(send_return_id.clone());
                }
            }

            lanes.push(RuntimeExecutionLaneSummary {
                lane: *lane,
                groups,
                node_ids,
                topology_roles,
                track_lane_ids: lane_ids,
                bus_group_ids: bus_groups,
                console_group_ids: console_groups,
                send_return_ids: send_returns,
            });
        }

        let mut track_lanes_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerTrackLaneSummary>::new();
        let mut bus_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerBusGroupSummary>::new();
        let mut console_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerConsoleGroupSummary>::new();
        let mut send_returns_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerSendReturnSummary>::new();

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
            if let Some(track_lane_id) = &node.track_lane_id {
                let summary = track_lanes_by_id
                    .entry(track_lane_id.clone())
                    .or_insert_with(|| RuntimeMixerTrackLaneSummary {
                        track_lane_id: track_lane_id.clone(),
                        node_ids: Vec::new(),
                        bus_group_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                summary.node_ids.push(node.node_id.clone());
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !summary.bus_group_ids.contains(bus_group_id) {
                        summary.bus_group_ids.push(bus_group_id.clone());
                    }
                }
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(bus_group_id) = &node.bus_group_id {
                let summary = bus_groups_by_id
                    .entry(bus_group_id.clone())
                    .or_insert_with(|| RuntimeMixerBusGroupSummary {
                        bus_group_id: bus_group_id.clone(),
                        topology_roles: Vec::new(),
                        node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                if !summary.topology_roles.contains(&node.topology_role) {
                    summary.topology_roles.push(node.topology_role);
                }
                summary.node_ids.push(node.node_id.clone());
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(console_group_id) = &node.console_group_id {
                let summary = console_groups_by_id
                    .entry(console_group_id.clone())
                    .or_insert_with(|| RuntimeMixerConsoleGroupSummary {
                        console_group_id: console_group_id.clone(),
                        node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                summary.node_ids.push(node.node_id.clone());
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(send_return_id) = &node.send_return_id {
                let summary = send_returns_by_id
                    .entry(send_return_id.clone())
                    .or_insert_with(|| RuntimeMixerSendReturnSummary {
                        send_return_id: send_return_id.clone(),
                        send_node_ids: Vec::new(),
                        return_node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                match node.topology_role {
                    GraphNodeTopologyRole::Send => summary.send_node_ids.push(node.node_id.clone()),
                    GraphNodeTopologyRole::Return => {
                        summary.return_node_ids.push(node.node_id.clone());
                    }
                    _ => {}
                }
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            nodes.push(RuntimeExecutionNodeSummary {
                node_id: node.node_id.clone(),
                lane: runtime_lane_for_group(node.group),
                group: node.group,
                execution_class: node.execution_class,
                topology_role: node.topology_role,
                track_lane_id: node.track_lane_id.clone(),
                bus_group_id: node.bus_group_id.clone(),
                console_group_id: node.console_group_id.clone(),
                send_return_id: node.send_return_id.clone(),
                input_bus_id: node.input_bus_id.clone(),
                output_bus_id: node.output_bus_id.clone(),
                plugin_sandbox_id: node.plugin_sandbox_id.clone(),
                plugin_recall_state: None,
                plugin_recall: None,
                plugin_compensation_state: None,
                plugin_realized_latency_samples: None,
                plugin_tail_samples: None,
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
            send_return_group_count: send_return_ids.len(),
            console_group_count: console_group_ids.len(),
            lanes,
            track_lanes: track_lanes_by_id.into_values().collect(),
            bus_groups: bus_groups_by_id.into_values().collect(),
            console_groups: console_groups_by_id.into_values().collect(),
            send_returns: send_returns_by_id.into_values().collect(),
            nodes,
            plugin_chain: RuntimeRoutedPluginChainSummary::default(),
        }
    }

    pub fn with_plugin_chain_snapshot(mut self, snapshot: &RuntimePluginChainSnapshot) -> Self {
        let mut stage_by_node =
            std::collections::BTreeMap::<&str, &RuntimePluginChainStageSnapshot>::new();
        for chain in &snapshot.chains {
            self.plugin_chain.include_chain(chain);
            if let Some(track_lane_id) = chain.track_lane_id.as_deref() {
                if let Some(summary) = self
                    .track_lanes
                    .iter_mut()
                    .find(|summary| summary.track_lane_id == track_lane_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(bus_group_id) = chain.bus_group_id.as_deref() {
                if let Some(summary) = self
                    .bus_groups
                    .iter_mut()
                    .find(|summary| summary.bus_group_id == bus_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(console_group_id) = chain.console_group_id.as_deref() {
                if let Some(summary) = self
                    .console_groups
                    .iter_mut()
                    .find(|summary| summary.console_group_id == console_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(send_return_id) = chain.send_return_id.as_deref() {
                if let Some(summary) = self
                    .send_returns
                    .iter_mut()
                    .find(|summary| summary.send_return_id == send_return_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            for stage in &chain.stages {
                stage_by_node.insert(stage.node_id.as_str(), stage);
            }
        }

        for node in &mut self.nodes {
            if let Some(stage) = stage_by_node.get(node.node_id.as_str()) {
                node.plugin_recall_state = Some(stage.recall_state);
                node.plugin_recall = Some(stage.recall.clone());
                node.plugin_compensation_state = Some(stage.compensation_state);
                node.plugin_realized_latency_samples = stage.realized_latency_samples;
                node.plugin_tail_samples = stage.tail_samples;
            }
        }

        self
    }
}

impl RuntimeOfflineRenderContractPreview {
    pub fn chain_contract_from_runtime_state(
        topology: &RuntimeExecutionTopologySummary,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<RuntimeOfflineRenderChainDependencyPreview, RuntimeError> {
        let plugin_chain = &topology.plugin_chain;
        if plugin_chain.stage_count != recall_handoff.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                format!(
                    "offline render contract requires aligned plugin chain and recall handoff stages (chain={} recall={})",
                    plugin_chain.stage_count, recall_handoff.stage_count
                ),
            ));
        }

        Ok(RuntimeOfflineRenderChainDependencyPreview {
            chain_count: plugin_chain.chain_count,
            stage_count: plugin_chain.stage_count,
            pending_render_stage_count: plugin_chain.pending_render_stage_count,
            settling_stage_count: plugin_chain.settling_stage_count,
            compensated_stage_count: plugin_chain.compensated_stage_count,
            degraded_stage_count: plugin_chain.degraded_stage_count,
            bypassed_stage_count: plugin_chain.bypassed_stage_count,
            missing_binding_stage_count: plugin_chain.missing_binding_stage_count,
            total_planned_latency_samples: plugin_chain.total_planned_latency_samples,
            total_realized_latency_samples: plugin_chain.total_realized_latency_samples,
            total_tail_samples: plugin_chain.total_tail_samples,
            recall_stage_count: recall_handoff.stage_count,
            unbound_recall_stage_count: recall_handoff.unbound_stage_count,
            cold_recall_stage_count: recall_handoff.cold_stage_count,
            warm_recall_stage_count: recall_handoff.warm_stage_count,
            recovered_recall_stage_count: recall_handoff.recovered_stage_count,
            unavailable_recall_stage_count: recall_handoff.unavailable_stage_count,
            summary: format!(
                "chains={} stages={} pending={} settling={} compensated={} degraded={} bypassed={} missing={} latency={}/{} tail={} recall={}/unbound={} cold={} warm={} recovered={} unavailable={}",
                plugin_chain.chain_count,
                plugin_chain.stage_count,
                plugin_chain.pending_render_stage_count,
                plugin_chain.settling_stage_count,
                plugin_chain.compensated_stage_count,
                plugin_chain.degraded_stage_count,
                plugin_chain.bypassed_stage_count,
                plugin_chain.missing_binding_stage_count,
                plugin_chain.total_planned_latency_samples,
                plugin_chain.total_realized_latency_samples,
                plugin_chain.total_tail_samples,
                recall_handoff.stage_count,
                recall_handoff.unbound_stage_count,
                recall_handoff.cold_stage_count,
                recall_handoff.warm_stage_count,
                recall_handoff.recovered_stage_count,
                recall_handoff.unavailable_stage_count,
            ),
        })
    }

    pub fn from_runtime_state(
        request: &RuntimeOfflineRenderRequest,
        topology: &RuntimeExecutionTopologySummary,
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        tempo_map: &RuntimeTempoMapSnapshot,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<Self, RuntimeError> {
        if request.request_id.trim().is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-empty request id",
            ));
        }
        if request.duration_samples == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-zero duration",
            ));
        }
        if request.export_sample_rate_hz == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a positive export sample rate",
            ));
        }

        let mut seen_stem_ids = std::collections::BTreeSet::new();
        let mut stem_targets = Vec::with_capacity(request.stem_targets.len());
        for stem in &request.stem_targets {
            if stem.stem_id.trim().is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "offline render stem targets require a non-empty stem id",
                ));
            }
            if !seen_stem_ids.insert(stem.stem_id.clone()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("offline render stem id `{}` is duplicated", stem.stem_id),
                ));
            }
            stem_targets.push(resolve_offline_render_stem_target(stem, topology)?);
        }

        let mut freeze_artifacts = Vec::with_capacity(request.freeze_artifacts.len());
        for artifact in &request.freeze_artifacts {
            if artifact.artifact_id.trim().is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "offline freeze artifacts require a non-empty artifact id",
                ));
            }
            if !request.include_main_mix
                && !stem_targets
                    .iter()
                    .any(|stem| stem.stem_id == artifact.source_stem_id)
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline freeze artifact `{}` references unknown stem `{}`",
                        artifact.artifact_id, artifact.source_stem_id
                    ),
                ));
            }
            let resolved_selection = recall_handoff
                .resolve_selection(&artifact.recall_selection)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline freeze artifact `{}` references an unknown recall handoff stage",
                            artifact.artifact_id
                        ),
                    )
                })?;
            freeze_artifacts.push(RuntimeOfflineFreezeArtifactPreview {
                artifact_id: artifact.artifact_id.clone(),
                source_stem_id: artifact.source_stem_id.clone(),
                recall_stage_count: resolved_selection.len(),
                recall_stage_ids: resolved_selection
                    .iter()
                    .map(|stage| stage.stage_id.clone())
                    .collect(),
                recall_states: resolved_selection
                    .iter()
                    .map(|stage| stage.recall_state)
                    .collect(),
                summary: format!(
                    "artifact={} source_stem={} recall_stages={} recall_states={:?}",
                    artifact.artifact_id,
                    artifact.source_stem_id,
                    resolved_selection.len(),
                    resolved_selection
                        .iter()
                        .map(|stage| stage.recall_state)
                        .collect::<Vec<_>>(),
                ),
            });
        }

        let timeline_end_samples = request
            .timeline_start_samples
            .saturating_add(request.duration_samples as i64);
        let chain_contract = Self::chain_contract_from_runtime_state(topology, recall_handoff)?;
        let mut preview = Self {
            request_id: request.request_id.clone(),
            timeline_start_samples: request.timeline_start_samples,
            timeline_end_samples,
            duration_samples: request.duration_samples,
            export_sample_rate_hz: request.export_sample_rate_hz,
            include_main_mix: request.include_main_mix,
            clip_count: clip_processing.clip_count,
            ready_clip_count: clip_processing.ready_clip_count,
            stem_count: stem_targets.len(),
            freeze_artifact_count: freeze_artifacts.len(),
            resolved_tempo_bpm: tempo_map.resolved_tempo_bpm,
            resolved_tempo_source: tempo_map.tempo_source,
            chain_contract,
            stem_targets,
            freeze_artifacts,
            summary: String::new(),
        };
        preview.summary = format!(
            "request={} timeline={}..{} duration={} export_sample_rate={} clips={}/{} stems={} freeze_artifacts={} tempo={:.3}/{:?} chain_contract={}",
            preview.request_id,
            preview.timeline_start_samples,
            preview.timeline_end_samples,
            preview.duration_samples,
            preview.export_sample_rate_hz,
            preview.ready_clip_count,
            preview.clip_count,
            preview.stem_count,
            preview.freeze_artifact_count,
            preview.resolved_tempo_bpm,
            preview.resolved_tempo_source,
            preview.chain_contract.summary,
        );
        Ok(preview)
    }
}

fn resolve_offline_render_stem_target(
    stem: &RuntimeOfflineRenderStemTarget,
    topology: &RuntimeExecutionTopologySummary,
) -> Result<RuntimeOfflineRenderStemPreview, RuntimeError> {
    let (target_id, resolved_node_ids, resolved_output_bus_ids) = match stem.target_kind {
        RuntimeOfflineRenderTargetKind::MainMix => (
            None,
            topology
                .nodes
                .iter()
                .map(|node| node.node_id.clone())
                .collect::<Vec<_>>(),
            topology
                .nodes
                .iter()
                .map(|node| node.output_bus_id.clone())
                .collect::<Vec<_>>(),
        ),
        RuntimeOfflineRenderTargetKind::TrackLane => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a track lane id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .track_lanes
                .iter()
                .find(|summary| summary.track_lane_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown track lane `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::BusGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a bus group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .bus_groups
                .iter()
                .find(|summary| summary.bus_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown bus group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::ConsoleGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a console group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .console_groups
                .iter()
                .find(|summary| summary.console_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown console group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::SendReturn => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a send/return id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .send_returns
                .iter()
                .find(|summary| summary.send_return_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown send/return `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            let mut node_ids = summary.send_node_ids.clone();
            node_ids.extend(summary.return_node_ids.clone());
            (
                Some(target_id.to_string()),
                node_ids,
                summary.output_bus_ids.clone(),
            )
        }
    };

    let resolved_node_count = resolved_node_ids.len();
    let resolved_output_bus_count = resolved_output_bus_ids.len();
    Ok(RuntimeOfflineRenderStemPreview {
        stem_id: stem.stem_id.clone(),
        target_kind: stem.target_kind,
        target_id,
        resolved_node_ids,
        resolved_output_bus_ids,
        summary: format!(
            "stem={} target={:?}/{:?} nodes={} output_buses={}",
            stem.stem_id,
            stem.target_kind,
            stem.target_id,
            resolved_node_count,
            resolved_output_bus_count,
        ),
    })
}

impl RuntimeOfflinePluginExecutionBoundary {
    pub fn delegated_execution_request(&self) -> RuntimeOfflinePluginDelegatedExecutionRequest {
        let stages = self
            .stages
            .iter()
            .filter(|stage| stage.host_delegate_required)
            .map(|stage| {
                let mut request = RuntimeOfflinePluginDelegatedExecutionStageRequest {
                    stage_id: stage.stage_id.clone(),
                    node_id: stage.node_id.clone(),
                    chain_id: stage.chain_id.clone(),
                    stage_index: stage.stage_index,
                    sandbox_id: stage.sandbox_id.clone(),
                    plugin_type_id: stage.plugin_type_id.clone(),
                    plugin_format: stage.plugin_format,
                    recall_state: stage.recall_state,
                    recall_payload: stage.recall_payload.clone(),
                    override_state: stage.override_state,
                    latest_override_processing_epoch: stage.latest_override_processing_epoch,
                    latest_override_block_sequence: stage.latest_override_block_sequence,
                    summary: String::new(),
                };
                request.summary = format!(
                    "stage={}:{} sandbox={:?} recall={:?} override={:?}",
                    request.chain_id,
                    request.stage_index,
                    request.sandbox_id.as_deref(),
                    request.recall_state,
                    request.override_state,
                );
                request
            })
            .collect::<Vec<_>>();
        let stage_count = stages.len();
        let mut request = RuntimeOfflinePluginDelegatedExecutionRequest {
            request_id: self.request_id.clone(),
            timeline_start_samples: self.timeline_start_samples,
            duration_samples: self.duration_samples,
            runtime_sample_rate_hz: self.runtime_sample_rate_hz,
            export_sample_rate_hz: self.export_sample_rate_hz,
            block_size: self.block_size,
            block_count: self.block_count,
            stage_count,
            stages,
            summary: String::new(),
        };
        request.summary = format!(
            "request={} delegated_stages={} blocks={} sample_rate={}->{}",
            request.request_id,
            request.stage_count,
            request.block_count,
            request.runtime_sample_rate_hz,
            request.export_sample_rate_hz,
        );
        request
    }
}

impl RuntimeOfflineRenderManifest {
    pub fn apply_delegated_execution_receipt(
        &mut self,
        receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    ) -> Result<(), RuntimeError> {
        if receipt.request_id != self.request_id {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt request `{}` does not match manifest request `{}`",
                    receipt.request_id, self.request_id
                ),
            ));
        }
        if receipt.stage_count != receipt.stages.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage_count must match stage receipt count",
            ));
        }
        if self.delegated_execution_request.stage_count != receipt.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt stages do not match request (request={} receipt={})",
                    self.delegated_execution_request.stage_count, receipt.stage_count
                ),
            ));
        }

        let expected_stage_ids = self
            .delegated_execution_request
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        let receipt_stage_ids = receipt
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        if expected_stage_ids != receipt_stage_ids {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage ids must match the delegated request",
            ));
        }
        let completed_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Completed)
            .count();
        let rejected_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Rejected)
            .count();
        let unavailable_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| {
                stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Unavailable
            })
            .count();
        if receipt.completed_stage_count != completed_stage_count
            || receipt.rejected_stage_count != rejected_stage_count
            || receipt.unavailable_stage_count != unavailable_stage_count
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt status counters must match stage receipt statuses",
            ));
        }

        self.delegated_execution_receipt = Some(receipt);
        self.summary = format!(
            "request={} root={:?} materialized={} artifacts={} report={} delegated_request_stages={} delegated_receipt={}",
            self.request_id,
            self.artifact_root_path.as_deref(),
            self.materialized,
            self.artifact_count,
            self.report.is_some(),
            self.delegated_execution_request.stage_count,
            self.delegated_execution_receipt.is_some(),
        );
        Ok(())
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
    pub metering_snapshot: RuntimeMeteringSnapshot,
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    pub fault_status: RuntimeFaultStatusSnapshot,
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    pub interruption_summary: RuntimeInterruptionSummary,
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    pub tempo_map_snapshot: RuntimeTempoMapSnapshot,
    pub warp_pipeline_snapshot: RuntimeWarpPipelineSnapshot,
    pub clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    pub recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    pub offline_render_session_snapshot: RuntimeOfflineRenderSessionSnapshot,
    pub automation_snapshot: RuntimeAutomationSnapshot,
    pub engine_block_snapshot: RuntimeEngineBlockSnapshot,
    pub transport_concurrency_snapshot: RuntimeTransportConcurrencySnapshot,
    pub plugin_discovery_snapshot: RuntimePluginDiscoverySnapshot,
    pub plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
    pub plugin_chain_snapshot: RuntimePluginChainSnapshot,
    pub scheduler_summary: RuntimeSchedulerExportSummary,
    pub block_summary: RuntimeBlockExecutionSummary,
    pub degradation_summary: RuntimeDegradationSummary,
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    pub transport_fault_summary: TransportFaultSummary,
    pub transport_session_summary: TransportSessionSummary,
    pub last_deferred_service_receipt: Option<RuntimeDeferredServiceReceipt>,
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
        let metering_snapshot = runtime.get_metering_snapshot();
        let supervision_snapshot = runtime.get_supervision_snapshot();
        let timeline_snapshot = runtime.get_timeline_snapshot();
        let tempo_map_snapshot = runtime.get_tempo_map_snapshot();
        let warp_pipeline_snapshot = runtime.get_warp_pipeline_snapshot();
        let clip_processing_pipeline_snapshot = runtime.get_clip_processing_pipeline_snapshot();
        let recording_capture_snapshot = runtime.get_recording_capture_snapshot();
        let offline_render_session_snapshot = runtime.get_offline_render_session_snapshot();
        let automation_snapshot = runtime.get_automation_snapshot();
        let engine_block_snapshot = runtime.get_engine_block_snapshot();
        let execution_topology_summary = runtime.get_execution_topology_summary();
        let transport_concurrency_snapshot = runtime.get_transport_concurrency_snapshot();
        let plugin_discovery_snapshot = runtime.get_plugin_discovery_snapshot();
        let plugin_lifecycle_snapshot = runtime.get_plugin_lifecycle_snapshot();
        let plugin_chain_snapshot = runtime.get_plugin_chain_snapshot();
        let last_deferred_service_receipt = runtime.get_last_deferred_service_receipt();
        let scheduler_summary =
            RuntimeSchedulerExportSummary::from_snapshot(&engine_block_snapshot);
        let block_summary = RuntimeBlockExecutionSummary::from_snapshot(&engine_block_snapshot);
        let fault_status = RuntimeFaultStatusSnapshot::capture(
            readiness.clone(),
            &control_snapshot,
            &diagnostics_snapshot,
            &supervision_snapshot,
            &engine_block_snapshot,
            &transport_concurrency_snapshot,
            &plugin_lifecycle_snapshot,
            false,
            0,
        );
        let degradation_summary = RuntimeDegradationSummary::capture(
            &readiness,
            diagnostics_snapshot,
            &supervision_snapshot,
            &engine_block_snapshot,
            &transport_concurrency_snapshot,
            &observation,
        );
        let interruption_summary = RuntimeInterruptionSummary::capture(
            &fault_status,
            last_deferred_service_receipt.as_ref(),
        );
        let fault_diagnostic_receipt = RuntimeFaultDiagnosticReceipt::capture(
            &fault_status,
            &interruption_summary,
            &degradation_summary,
            &engine_block_snapshot,
            last_deferred_service_receipt.as_ref(),
            None,
        );
        Self {
            readiness: readiness.clone(),
            effective_config,
            control_snapshot,
            scheduler_snapshot,
            diagnostics_snapshot,
            metering_snapshot,
            supervision_snapshot: supervision_snapshot.clone(),
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
            timeline_snapshot,
            tempo_map_snapshot,
            warp_pipeline_snapshot,
            clip_processing_pipeline_snapshot,
            recording_capture_snapshot,
            offline_render_session_snapshot,
            automation_snapshot,
            engine_block_snapshot,
            transport_concurrency_snapshot,
            plugin_discovery_snapshot,
            plugin_lifecycle_snapshot,
            plugin_chain_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            execution_topology_summary,
            transport_fault_summary: TransportFaultSummary::from_records(
                &observation.transport_fault_events,
            ),
            transport_session_summary: TransportSessionSummary::from_diagnostics(&observation),
            last_deferred_service_receipt,
            observation,
        }
    }

    pub fn render_compact(&self) -> String {
        let tempo_map = (self.tempo_map_snapshot.segment_count > 0)
            .then(|| format_runtime_tempo_map_snapshot_compact(&self.tempo_map_snapshot))
            .unwrap_or_default();
        let warp = (self.warp_pipeline_snapshot.clip_count > 0)
            .then(|| format_runtime_warp_pipeline_snapshot_compact(&self.warp_pipeline_snapshot))
            .unwrap_or_default();
        let clip_processing = (self.clip_processing_pipeline_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_clip_processing_pipeline_snapshot_compact(
                    &self.clip_processing_pipeline_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_discovery = (self.plugin_discovery_snapshot.scan_count > 0)
            .then(|| {
                format_runtime_plugin_discovery_snapshot_compact(&self.plugin_discovery_snapshot)
            })
            .unwrap_or_default();
        let plugin_lifecycle = (self.plugin_lifecycle_snapshot.sandbox_count > 0)
            .then(|| {
                format_runtime_plugin_lifecycle_snapshot_compact(&self.plugin_lifecycle_snapshot)
            })
            .unwrap_or_default();
        let plugin_chain = (self.plugin_chain_snapshot.chain_count > 0)
            .then(|| format_runtime_plugin_chain_snapshot_compact(&self.plugin_chain_snapshot))
            .unwrap_or_default();
        let automation = (self.automation_snapshot.parameter_id != 0
            || self.automation_snapshot.lane_count > 0
            || self.automation_snapshot.last_batch_epoch.is_some())
            .then(|| {
                let snapshot = &self.automation_snapshot;
                format!(
                    " automation_param={} automation_projection={}/{}/{} automation_shapes={}/{} automation_batch_policy={}/{:?} automation_segments={} automation_first_epoch={:?} automation_last_epoch={:?} automation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.lane_count,
                    snapshot.point_count,
                    snapshot.projected_segment_count,
                    snapshot.hold_lane_count,
                    snapshot.linear_lane_count,
                    snapshot.last_batch_strategy_max_sub_blocks,
                    snapshot.last_batch_min_ramp_step_samples,
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
        let fault_status = format_runtime_fault_status_compact(&self.fault_status);
        let fault_diagnostic_receipt =
            format_runtime_fault_diagnostic_receipt_compact(&self.fault_diagnostic_receipt);
        let interruption_summary =
            format_runtime_interruption_summary_compact(&self.interruption_summary);
        let execution_topology_summary =
            format_runtime_execution_topology_summary_compact(&self.execution_topology_summary);
        let metering_summary = format_runtime_metering_snapshot_compact(&self.metering_snapshot);
        let recording_capture =
            format_runtime_recording_capture_snapshot_compact(&self.recording_capture_snapshot);
        let offline_render_session = (self.offline_render_session_snapshot.active_session_count
            > 0
            || self.offline_render_session_snapshot.last_session.is_some()
            || self
                .offline_render_session_snapshot
                .last_cancellation
                .is_some()
            || self.offline_render_session_snapshot.last_purge.is_some())
        .then(|| {
            format_runtime_offline_render_session_snapshot_compact(
                &self.offline_render_session_snapshot,
            )
        })
        .unwrap_or_default();
        let deferred_service = self
            .last_deferred_service_receipt
            .as_ref()
            .map(|receipt| {
                format!(
                    " deferred_service_class={:?} deferred_service_decision={:?} deferred_service_reason={:?} deferred_service_deferred_items={}",
                    receipt.work_class,
                    receipt.decision,
                    receipt.reason,
                    receipt.deferred_work_item_count,
                )
            })
            .unwrap_or_default();
        let compact = format!(
            "readiness={:?} sample_rate={} block_size={} handshaken={} configured={} running={} handshakes={} configures={} starts={} stops={} restarts={} xruns={} active_sandboxes={} safe_mode={} next_block_sequence={} sequence_segments={} sequence_first_block={:?} sequence_last_block={:?}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{} engine_graph_id={:?} engine_node_count={} engine_stateful_nodes={} engine_latency_nodes={} engine_plugin_backed_nodes={} engine_planning_anticipative={} engine_inline_realtime_nodes={} engine_stateful_realtime_nodes={} engine_anticipative_eligible_nodes={} engine_phase_count={} engine_anticipative_phases={} engine_phase_order={:?} engine_lane_count={} engine_anticipative_lanes={} engine_lane_order={:?} engine_dispatch_count={} engine_dispatch_boundaries={} engine_dispatch_order={:?} engine_prepared_dispatches={} engine_realtime_dispatches={} engine_dispatch_handoffs={}{} engine_prework_cache_enabled={} engine_prework_cache_state={:?} engine_prework_service_state={:?} engine_prework_service_pressure={:?} engine_prework_service_semantic_policy={:?} engine_prework_service_active_plugin_sandboxes={} engine_prework_service_bound_plugin_sandboxes={} engine_prework_service_active_bound_plugin_sandboxes={} engine_prework_service_degraded_bound_plugin_sandboxes={} engine_prework_service_missing_bound_plugin_sandboxes={} engine_prework_service_plugin_gate_active={} engine_prework_pending_targets={} engine_prework_pending_immediate_targets={} engine_prework_pending_near_term_targets={} engine_prework_pending_deferred_targets={} engine_prework_next_pending_target_block={:?} engine_prework_service_cycles={} engine_prework_service_prepared_targets={} engine_prework_service_pauses={} engine_prework_service_resumes={} engine_prework_service_starvations={} engine_prework_service_throttles={} engine_prework_service_yields={} engine_last_prework_service_epoch={:?} engine_last_prework_serviced_target_block={:?} engine_last_prework_serviced_backlog_class={:?} engine_prework_requested_mode={:?} engine_prework_mode={:?} engine_prework_policy_configured={} engine_prework_profile={:?} engine_prework_profile_source={:?} engine_prework_profile_window_override={:?} engine_prework_policy_window_blocks={:?} engine_prework_queue_capacity={} engine_prework_queue_depth={} engine_prework_peak_queue_depth={} engine_prework_window_targets={} engine_prework_window_blocks={:?} engine_prework_freshness_state={:?} engine_prework_block_window={} engine_prework_remaining_valid_blocks={:?} engine_prework_cache_admissions={} engine_prework_cache_consumptions={} engine_prework_queued_admissions={} engine_prework_queued_consumptions={} engine_prework_cache_hits={} engine_prework_cache_misses={} engine_prework_cache_invalidations={} engine_prework_cache_retirements={} engine_prework_unconsumed_retirements={} engine_prework_consumed_retirements={} engine_last_prework_cache_hit={} engine_last_prework_invalidation={:?} engine_last_prework_retirement={:?} engine_last_prework_retired_unconsumed={:?} engine_prework_cache_valid_until={:?} engine_prework_cache_valid_until_block={:?} engine_last_prework_source_epoch={:?} engine_last_prework_source_block={:?} engine_last_prework_admission_epoch={:?} engine_last_prework_admission_block={:?} engine_last_prework_admitted_from_block={:?} engine_last_prework_consumption_epoch={:?} engine_last_prework_consumption_block={:?} engine_last_prework_consumed_from_block={:?} engine_last_prework_retirement_epoch={:?} engine_last_prework_retirement_block={:?} engine_stage_count={} engine_dynamic_kernel_stages={} engine_dynamic_stage_state_model={:?} engine_total_latency_samples={} engine_max_node_latency_samples={} engine_total_tail_samples={} engine_max_node_tail_samples={} engine_output_tail_samples={} engine_max_bus_tail_samples={} engine_processed_blocks={} engine_last_block={:?} engine_prework_output_peak={:?} engine_realtime_input_peak={:?} engine_output_peak={:?} engine_output_rms={:?} engine_projection_epoch={:?} engine_parameter_epoch={:?} engine_context_anticipative={:?} engine_transport_playing={:?} engine_transport_tempo={:?} engine_timeline_position={:?}{} transport_concurrency_limits={}/{} transport_concurrency_current={} transport_concurrency_peak={} transport_concurrency_recovery_current={} transport_concurrency_recovery_peak={} transport_concurrency_cleanup_pending={} transport_concurrency_deferred_retries={} transport_concurrency_next_cleanup_epoch={} transport_concurrency_oldest_ready_epoch={:?} transport_fault_boundary={:?} transport_fault_sources={}/{}/{} transport_fault_phases={}/{}/{}/{} transport_session_boundary={:?} transport_session_state={:?} transport_session_attached={} transport_session_heartbeat_state={:?} transport_session_dispatch_state={:?} transport_session_attached_sessions={} transport_session_max_attached_sessions={} transport_session_attach={} transport_session_detach={}/{}/{} transport_session_heartbeat={}/{}/{} transport_session_dispatch={}/{}/{} {}",
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
            tempo_map,
            warp,
            clip_processing,
            plugin_discovery,
            plugin_lifecycle,
            plugin_chain,
            automation,
            transport_timeline,
            scheduler_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
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
        format!(
            "{compact}{recording_capture}{offline_render_session}{execution_topology_summary}{metering_summary}{deferred_service}"
        )
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
        let tempo_map = (self.observation.tempo_map_snapshot.segment_count > 0)
            .then(|| {
                format_runtime_tempo_map_snapshot_multiline(&self.observation.tempo_map_snapshot)
            })
            .unwrap_or_default();
        let warp = (self.observation.warp_pipeline_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_warp_pipeline_snapshot_multiline(
                    &self.observation.warp_pipeline_snapshot,
                )
            })
            .unwrap_or_default();
        let clip_processing = (self
            .observation
            .clip_processing_pipeline_snapshot
            .clip_count
            > 0)
        .then(|| {
            format_runtime_clip_processing_pipeline_snapshot_multiline(
                &self.observation.clip_processing_pipeline_snapshot,
            )
        })
        .unwrap_or_default();
        let plugin_discovery = (self.observation.plugin_discovery_snapshot.scan_count > 0)
            .then(|| {
                format_runtime_plugin_discovery_snapshot_multiline(
                    &self.observation.plugin_discovery_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_lifecycle = (self.observation.plugin_lifecycle_snapshot.sandbox_count > 0)
            .then(|| {
                format_runtime_plugin_lifecycle_snapshot_multiline(
                    &self.observation.plugin_lifecycle_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_chain = (self.observation.plugin_chain_snapshot.chain_count > 0)
            .then(|| {
                format_runtime_plugin_chain_snapshot_multiline(
                    &self.observation.plugin_chain_snapshot,
                )
            })
            .unwrap_or_default();
        let _automation = (self.observation.automation_snapshot.parameter_id != 0
            || self.observation.automation_snapshot.lane_count > 0
            || self.observation.automation_snapshot.last_batch_epoch.is_some())
            .then(|| {
                let snapshot = &self.observation.automation_snapshot;
                format!(
                    "\nautomation_param={}\nautomation_lane_count={}\nautomation_point_count={}\nautomation_projected_segment_count={}\nautomation_mapped_lanes={}\nautomation_unmapped_lanes={}\nautomation_hold_lanes={}\nautomation_linear_lanes={}\nautomation_last_batch_epoch={:?}\nautomation_last_batch_event_count={}\nautomation_last_batch_ignored_event_count={}\nautomation_last_batch_sub_block_count={}\nautomation_last_batch_coalesced_event_count={}\nautomation_last_batch_strategy_max_sub_blocks={}\nautomation_last_batch_min_ramp_step_samples={:?}\nautomation_last_batch_max_sample_offset={:?}\nautomation_last_block_sequence={:?}\nautomation_last_timeline_position_samples={:?}\nautomation_transport_playing={:?}\nautomation_value_events={}\nautomation_modulation_events={}\nautomation_gesture_begin_events={}\nautomation_gesture_end_events={}\nautomation_first_value={:?}\nautomation_last_value={:?}\nautomation_last_modulation={:?}\nautomation_first_epoch={:?}\nautomation_last_epoch={:?}\nautomation_segments={}\nautomation_segment_epochs={:?}\nautomation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.lane_count,
                    snapshot.point_count,
                    snapshot.projected_segment_count,
                    snapshot.mapped_lane_count,
                    snapshot.unmapped_lane_count,
                    snapshot.hold_lane_count,
                    snapshot.linear_lane_count,
                    snapshot.last_batch_epoch,
                    snapshot.last_batch_event_count,
                    snapshot.last_batch_ignored_event_count,
                    snapshot.last_batch_sub_block_count,
                    snapshot.last_batch_coalesced_event_count,
                    snapshot.last_batch_strategy_max_sub_blocks,
                    snapshot.last_batch_min_ramp_step_samples,
                    snapshot.last_batch_max_sample_offset,
                    snapshot.last_block_sequence,
                    snapshot.last_timeline_position_samples,
                    snapshot.transport_playing,
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
        let _transport_timeline = format!(
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
        let _engine_transport = format!(
            "\nengine_transport_epoch={}\nengine_transport_transition={:?}\nengine_transport_block_start_samples={:?}\nengine_transport_block_end_samples={:?}\nengine_transport_loop_wrapped={}",
            self.observation.engine_block_snapshot.transport_epoch,
            self.observation.engine_block_snapshot.transport_transition,
            self.observation
                .engine_block_snapshot
                .transport_block_start_samples,
            self.observation.engine_block_snapshot.transport_block_end_samples,
            self.observation.engine_block_snapshot.transport_loop_wrapped,
        );
        let _scheduler_topology = format_scheduler_topology_multiline(
            &self.observation.engine_block_snapshot.scheduler_topology,
        );
        let _scheduler_snapshot =
            format_runtime_scheduler_snapshot_multiline(&self.observation.scheduler_snapshot);
        let _scheduler_summary =
            format_runtime_scheduler_summary_multiline(&self.observation.scheduler_summary);
        let _block_summary =
            format_runtime_block_summary_multiline(&self.observation.block_summary);
        let _degradation_summary =
            format_runtime_degradation_summary_multiline(&self.observation.degradation_summary);
        let _fault_status = format_runtime_fault_status_multiline(&self.observation.fault_status);
        let _fault_diagnostic_receipt = format_runtime_fault_diagnostic_receipt_multiline(
            &self.observation.fault_diagnostic_receipt,
        );
        let _interruption_summary =
            format_runtime_interruption_summary_multiline(&self.observation.interruption_summary);
        let execution_topology_summary = format_runtime_execution_topology_summary_multiline(
            &self.observation.execution_topology_summary,
        );
        let metering_summary =
            format_runtime_metering_snapshot_multiline(&self.observation.metering_snapshot);
        let recording_capture = format_runtime_recording_capture_snapshot_multiline(
            &self.observation.recording_capture_snapshot,
        );
        let offline_render_session = format_runtime_offline_render_session_snapshot_multiline(
            &self.observation.offline_render_session_snapshot,
        );
        let deferred_service = self
            .observation
            .last_deferred_service_receipt
            .as_ref()
            .map(|receipt| format!("\nlast_deferred_service=\n{}", receipt.render_multiline()))
            .unwrap_or_default();
        /* let multiline = format!(
            "readiness={:?}\nsample_rate={}\nblock_size={}\nhandshaken={}\nconfigured={}\nrunning={}\nhandshake_count={}\nconfigure_count={}\nstart_count={}\nstop_count={}\nrestart_count={:?}\nlast_client_version={:?}\nlast_stop_reason={:?}\nlast_reconfigure={:?}\nxruns={}\nactive_sandboxes={}\nsafe_mode={}\nnext_block_sequence={}\nsequence_segments={}\nsequence_segment_epochs={:?}\nsequence_first_block={:?}\nsequence_last_block={:?}\nsequence_gaps={}\nsequence_lease_rollovers={}{}{}{}{}{}{}{}{}{}\nengine_graph_id={:?}\nengine_node_count={}\nengine_stateful_nodes={}\nengine_latency_nodes={}\nengine_plugin_backed_nodes={}\nengine_planning_anticipative={}\nengine_inline_realtime_nodes={}\nengine_stateful_realtime_nodes={}\nengine_anticipative_eligible_nodes={}\nengine_phase_count={}\nengine_anticipative_phases={}\nengine_phase_order={:?}\nengine_lane_count={}\nengine_anticipative_lanes={}\nengine_lane_order={:?}\nengine_dispatch_count={}\nengine_dispatch_boundaries={}\nengine_dispatch_order={:?}\nengine_prepared_dispatches={}\nengine_realtime_dispatches={}\nengine_dispatch_handoffs={}{}\nengine_prework_cache_enabled={}\nengine_prework_cache_state={:?}\nengine_prework_service_state={:?}\nengine_prework_service_pressure={:?}\nengine_prework_service_semantic_policy={:?}\nengine_prework_service_active_plugin_sandboxes={}\nengine_prework_service_bound_plugin_sandboxes={}\nengine_prework_service_active_bound_plugin_sandboxes={}\nengine_prework_service_degraded_bound_plugin_sandboxes={}\nengine_prework_service_missing_bound_plugin_sandboxes={}\nengine_prework_service_plugin_gate_active={}\nengine_prework_pending_targets={}\nengine_prework_pending_immediate_targets={}\nengine_prework_pending_near_term_targets={}\nengine_prework_pending_deferred_targets={}\nengine_prework_next_pending_target_block={:?}\nengine_prework_service_cycles={}\nengine_prework_service_prepared_targets={}\nengine_prework_service_pauses={}\nengine_prework_service_resumes={}\nengine_prework_service_starvations={}\nengine_prework_service_throttles={}\nengine_prework_service_yields={}\nengine_last_prework_service_epoch={:?}\nengine_last_prework_service_requested_cycles={}\nengine_last_prework_service_effective_cycles={}\nengine_last_prework_service_cycle_count={}\nengine_last_prework_service_budget={:?}\nengine_last_prework_service_effective_budget={:?}\nengine_last_prework_service_prepared_targets={}\nengine_last_prework_serviced_target_block={:?}\nengine_last_prework_serviced_backlog_class={:?}\nengine_prework_requested_mode={:?}\nengine_prework_mode={:?}\nengine_prework_policy_configured={}\nengine_prework_profile={:?}\nengine_prework_profile_source={:?}\nengine_prework_profile_window_override={:?}\nengine_prework_policy_window_blocks={:?}\nengine_prework_queue_capacity={}\nengine_prework_queue_depth={}\nengine_prework_peak_queue_depth={}\nengine_prework_window_targets={}\nengine_prework_window_blocks={:?}\nengine_prework_freshness_state={:?}\nengine_prework_block_window={}\nengine_prework_remaining_valid_blocks={:?}\nengine_prework_cache_admissions={}\nengine_prework_cache_consumptions={}\nengine_prework_queued_admissions={}\nengine_prework_queued_consumptions={}\nengine_prework_cache_hits={}\nengine_prework_cache_misses={}\nengine_prework_cache_invalidations={}\nengine_last_prework_cache_hit={}\nengine_last_prework_invalidation={:?}\nengine_prework_cache_valid_until={:?}\nengine_prework_cache_valid_until_block={:?}\nengine_last_prework_source_epoch={:?}\nengine_last_prework_source_block={:?}\nengine_last_prework_admission_epoch={:?}\nengine_last_prework_admission_block={:?}\nengine_last_prework_admitted_from_block={:?}\nengine_last_prework_consumption_epoch={:?}\nengine_last_prework_consumption_block={:?}\nengine_last_prework_consumed_from_block={:?}\nengine_planned_nodes={:?}\nengine_stage_count={}\nengine_dynamic_kernel_stages={}\nengine_dynamic_stage_state_model={:?}\nengine_total_latency_samples={}\nengine_max_node_latency_samples={}\nengine_total_tail_samples={}\nengine_max_node_tail_samples={}\nengine_output_tail_samples={}\nengine_max_bus_tail_samples={}\nengine_processed_blocks={}\nengine_last_processing_epoch={:?}\nengine_last_block_sequence={:?}\nengine_last_frame_count={}\nengine_last_channel_count={}\nengine_last_input_peak={:?}\nengine_last_prework_output_peak={:?}\nengine_last_realtime_input_peak={:?}\nengine_last_output_peak={:?}\nengine_last_output_rms={:?}\nengine_last_first_output_sample={:?}\nengine_projection_epoch={:?}\nengine_parameter_epoch={:?}\nengine_context_anticipative={:?}\nengine_transport_playing={:?}\nengine_transport_tempo_bpm={:?}\nengine_timeline_position_samples={:?}{}{}\ntransport_concurrency_steady_limit={}\ntransport_concurrency_recovery_limit={}\ntransport_concurrency_current_attached={}\ntransport_concurrency_peak_attached={}\ntransport_concurrency_current_recovery_overlap={}\ntransport_concurrency_peak_recovery_overlap={}\ntransport_concurrency_current_lingering={}\ntransport_concurrency_peak_lingering={}\ntransport_concurrency_current_detach_requested={}\ntransport_concurrency_current_detach_faulted={}\ntransport_concurrency_active_sessions={:?}\ntransport_concurrency_pending_cleanup_waves={:?}\ntransport_concurrency_last_admitted_sandbox_id={:?}\ntransport_concurrency_last_rejected_sandbox_id={:?}\ntransport_concurrency_last_rejection_reason={:?}\ntransport_fault_boundary={:?}\ntransport_fault_host_broker_events={}\ntransport_fault_sandbox_operation_events={}\ntransport_fault_runtime_dispatch_events={}\ntransport_fault_prepare_events={}\ntransport_fault_dispatch_events={}\ntransport_fault_teardown_events={}\ntransport_fault_control_events={}\ntransport_fault_first_epoch={:?}\ntransport_fault_last_epoch={:?}\ntransport_fault_first_block={:?}\ntransport_fault_last_block={:?}\ntransport_session_boundary={:?}\ntransport_session_state={:?}\ntransport_session_currently_attached={}\ntransport_session_heartbeat_state={:?}\ntransport_session_dispatch_state={:?}\ntransport_session_current_attached_sessions={}\ntransport_session_max_attached_sessions={}\ntransport_session_attach_events={}\ntransport_session_detach_requested_events={}\ntransport_session_detached_events={}\ntransport_session_detach_fault_events={}\ntransport_session_heartbeat_requested_events={}\ntransport_session_heartbeat_responded_events={}\ntransport_session_heartbeat_missed_events={}\ntransport_session_dispatch_requested_events={}\ntransport_session_dispatch_completed_events={}\ntransport_session_dispatch_timed_out_events={}\ntransport_session_first_epoch={:?}\ntransport_session_last_epoch={:?}\ntransport_session_first_block={:?}\ntransport_session_last_block={:?}\ntransport_session_active_sandbox_id={:?}\ntransport_session_active_lease_id={:?}\ntransport_session_active_region_id={:?}\ntransport_session_active_block_sequence={:?}\ntransport_session_active_sessions={:?}\ntransport_session_last_sandbox_id={:?}\ntransport_session_last_lease_id={:?}\ntransport_session_last_region_id={:?}\nevent_stream={}\nsupervision_updates={}\nplugin_faults={}\nrecovery_events={}\nlifecycle_events={}\ntransport_events={}\nheartbeat_events={}\nblock_dispatch_events={}\nlease_rollover_events={}\ninvalidation_events={}\ncompletion_slot_events={}\ntransport_fault_events={}\nbroker_failure_events={}\nsandbox_operation_failure_events={}\nlast_watchdog={}\nlast_fault={}\nlast_recovery={:?}\nlast_lifecycle={:?}\nlast_transport={:?}\nlast_heartbeat={:?}\nlast_dispatch={:?}\nlast_rollover={:?}\nlast_invalidation={:?}\nlast_completion_slot={:?}\nlast_transport_fault={:?}\nlast_broker_failure={:?}\nlast_sandbox_operation_failure={:?}\nrecovery_sequence={:?}\nlifecycle_sequence={:?}\ntransport_sequence={:?}\nheartbeat_sequence={:?}\nblock_dispatch_sequence={:?}\nlease_rollover_sequence={:?}\ninvalidation_sequence={:?}\ncompletion_slot_sequence={:?}\ntransport_fault_sequence={:?}\nbroker_failure_sequence={:?}\nsandbox_operation_failure_sequence={:?}",
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
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
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
        ); */
        let multiline = self.observation.render_compact().replace(' ', "\n");
        format!(
            "{multiline}{tempo_map}{warp}{clip_processing}{recording_capture}{offline_render_session}{plugin_discovery}{plugin_lifecycle}{plugin_chain}{execution_topology_summary}{metering_summary}{deferred_service}"
        )
    }

    pub fn render_json(&self) -> String {
        let timeline = &self.observation.timeline_snapshot.block_sequence_continuity;
        let last_fault = self.observation.observation.plugin_faults.last();
        let last_plugin_instance_state = self.observation.observation.last_plugin_instance_state();
        let automation = &self.observation.automation_snapshot;
        let automation = if automation.parameter_id == 0
            && automation.lane_count == 0
            && automation.last_batch_epoch.is_none()
        {
            "null".into()
        } else {
            json_runtime_automation_snapshot(automation)
        };
        let deferred_service = self
            .observation
            .last_deferred_service_receipt
            .as_ref()
            .map(RuntimeDeferredServiceReceipt::render_json)
            .unwrap_or_else(|| "null".into());
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
                "\"fault_status\":{},",
                "\"fault_diagnostic_receipt\":{},",
                "\"interruption_summary\":{},",
                "\"degradation_summary\":{},",
                "\"tempo_map_snapshot\":{},",
                "\"warp_pipeline_snapshot\":{},",
                "\"clip_processing_pipeline_snapshot\":{},",
                "\"recording_capture_snapshot\":{},",
                "\"offline_render_session_snapshot\":{},",
                "\"plugin_discovery_snapshot\":{},",
                "\"plugin_lifecycle_snapshot\":{},",
                "\"plugin_chain_snapshot\":{},",
                "\"metering_snapshot\":{},",
                "\"execution_topology_summary\":{},",
                "\"transport_concurrency_snapshot\":{},",
                "\"transport_fault_summary\":{},",
                "\"transport_session_summary\":{},",
                "\"last_deferred_service\":{},",
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
            json_runtime_fault_status(&self.observation.fault_status),
            json_runtime_fault_diagnostic_receipt(&self.observation.fault_diagnostic_receipt),
            json_runtime_interruption_summary(&self.observation.interruption_summary),
            json_runtime_degradation_summary(&self.observation.degradation_summary),
            json_runtime_tempo_map_snapshot(&self.observation.tempo_map_snapshot),
            json_runtime_warp_pipeline_snapshot(&self.observation.warp_pipeline_snapshot),
            json_runtime_clip_processing_pipeline_snapshot(
                &self.observation.clip_processing_pipeline_snapshot,
            ),
            json_runtime_recording_capture_snapshot(&self.observation.recording_capture_snapshot,),
            json_runtime_offline_render_session_snapshot(
                &self.observation.offline_render_session_snapshot,
            ),
            json_runtime_plugin_discovery_snapshot(&self.observation.plugin_discovery_snapshot),
            json_runtime_plugin_lifecycle_snapshot(&self.observation.plugin_lifecycle_snapshot),
            json_runtime_plugin_chain_snapshot(&self.observation.plugin_chain_snapshot),
            json_runtime_metering_snapshot(&self.observation.metering_snapshot),
            json_runtime_execution_topology_summary(&self.observation.execution_topology_summary,),
            json_runtime_transport_concurrency_snapshot(
                &self.observation.transport_concurrency_snapshot,
            ),
            json_transport_fault_summary(&self.observation.transport_fault_summary),
            json_transport_session_summary(&self.observation.transport_session_summary),
            deferred_service,
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

impl RuntimeObservationReport {
    pub fn render_json(&self) -> String {
        RuntimeSupervisorReport {
            observation: self.clone(),
            events: Vec::new(),
        }
        .render_json()
    }
}

fn build_runtime_profiling_receipt(
    observation: &RuntimeObservationReport,
    host_io: Option<&RuntimeHostIoSummary>,
) -> RuntimeProfilingReceipt {
    let plugin_chain = &observation.execution_topology_summary.plugin_chain;
    let fault_diagnostic_receipt = RuntimeFaultDiagnosticReceipt::capture(
        &observation.fault_status,
        &observation.interruption_summary,
        &observation.degradation_summary,
        &observation.engine_block_snapshot,
        observation.last_deferred_service_receipt.as_ref(),
        host_io,
    );
    let fault_diagnostic_primary_family = fault_diagnostic_receipt.primary_family;
    RuntimeProfilingReceipt {
        sample_rate_hz: observation.effective_config.sample_rate.0,
        block_size: observation.effective_config.block_size,
        engine_processed_blocks: observation.engine_block_snapshot.processed_blocks,
        engine_last_block_sequence: observation.engine_block_snapshot.last_block_sequence,
        engine_node_count: observation.engine_block_snapshot.node_count,
        engine_stage_count: observation.engine_block_snapshot.stage_count,
        engine_total_latency_samples: observation.engine_block_snapshot.total_latency_samples,
        engine_total_tail_samples: observation.engine_block_snapshot.total_tail_samples,
        runtime_cpu_load_percent: observation.diagnostics_snapshot.cpu_load_percent,
        runtime_graph_latency_ms: observation.diagnostics_snapshot.graph_latency_ms,
        runtime_xrun_count: observation.diagnostics_snapshot.xruns,
        active_plugin_sandboxes: observation.diagnostics_snapshot.active_plugin_sandboxes,
        readiness_degraded: observation.degradation_summary.readiness_degraded,
        transport_gate_active: observation.degradation_summary.transport_gate_active,
        plugin_gate_active: observation.degradation_summary.plugin_gate_active,
        degraded_bound_plugin_sandboxes: observation
            .degradation_summary
            .degraded_bound_plugin_sandboxes,
        missing_bound_plugin_sandboxes: observation
            .degradation_summary
            .missing_bound_plugin_sandboxes,
        recovery_overlap_sessions: observation.degradation_summary.recovery_overlap_sessions,
        lingering_sessions: observation.degradation_summary.lingering_sessions,
        detach_faulted_sessions: observation.degradation_summary.detach_faulted_sessions,
        plugin_chain_stage_count: plugin_chain.stage_count,
        plugin_chain_degraded_stage_count: plugin_chain.degraded_stage_count,
        plugin_chain_missing_binding_stage_count: plugin_chain.missing_binding_stage_count,
        plugin_chain_total_planned_latency_samples: plugin_chain.total_planned_latency_samples,
        plugin_chain_total_realized_latency_samples: plugin_chain.total_realized_latency_samples,
        plugin_chain_total_tail_samples: plugin_chain.total_tail_samples,
        output_peak: observation.diagnostics_snapshot.last_output_peak,
        output_rms: observation.diagnostics_snapshot.last_output_rms,
        host_callback_count: host_io.map(|host_io| host_io.audio_pump.callback_count),
        host_callback_interval_ms: host_io.map(|host_io| host_io.clocking.callback_interval_ms),
        host_output_latency_ms: host_io.map(|host_io| host_io.latency.output_latency_ms),
        host_graph_latency_ms: host_io.map(|host_io| host_io.latency.graph_latency_ms),
        host_estimated_output_latency_ms: host_io
            .map(|host_io| host_io.latency.estimated_output_latency_ms),
        host_backend_xrun_count: host_io.map(|host_io| host_io.hardware.xrun_count),
        host_callback_overrun_count: host_io.map(|host_io| host_io.hardware.callback_overrun_count),
        host_device_loss_count: host_io.map(|host_io| host_io.hardware.device_loss_count),
        host_restart_attempt_count: host_io.map(|host_io| host_io.hardware.restart_attempt_count),
        host_restart_failure_count: host_io.map(|host_io| host_io.hardware.restart_failure_count),
        host_copied_output_samples: host_io.map(|host_io| host_io.audio_pump.copied_output_samples),
        host_zero_filled_output_samples: host_io
            .map(|host_io| host_io.audio_pump.zero_filled_output_samples),
        host_dropped_output_samples: host_io
            .map(|host_io| host_io.audio_pump.dropped_output_samples),
        fault_diagnostic_receipt,
        summary: format!(
            "sample_rate={} block_size={} engine_blocks={} cpu_load={:.3} xruns={} host_callbacks={:?} degraded={} gates={}/{} plugin_chain={}/degraded={}/missing={} sessions={}/{}/{} primary_family={:?}",
            observation.effective_config.sample_rate.0,
            observation.effective_config.block_size,
            observation.engine_block_snapshot.processed_blocks,
            observation.diagnostics_snapshot.cpu_load_percent,
            observation.diagnostics_snapshot.xruns,
            host_io.map(|host_io| host_io.audio_pump.callback_count),
            observation.degradation_summary.readiness_degraded,
            observation.degradation_summary.transport_gate_active,
            observation.degradation_summary.plugin_gate_active,
            plugin_chain.stage_count,
            plugin_chain.degraded_stage_count,
            plugin_chain.missing_binding_stage_count,
            observation.degradation_summary.recovery_overlap_sessions,
            observation.degradation_summary.lingering_sessions,
            observation.degradation_summary.detach_faulted_sessions,
            fault_diagnostic_primary_family,
        ),
    }
}

fn build_runtime_performance_trace_receipt(
    observations: &[RuntimeObservationReport],
) -> RuntimePerformanceTraceReceipt {
    if observations.is_empty() {
        return RuntimePerformanceTraceReceipt {
            observation_count: 0,
            first_block_sequence: None,
            last_block_sequence: None,
            processed_block_span: 0,
            peak_cpu_load_percent: 0.0,
            peak_graph_latency_ms: 0.0,
            peak_block_execution_time_ns: 0,
            peak_block_budget_utilization_percent: 0.0,
            peak_block_budget_overrun_ns: 0,
            peak_pending_prework_target_count: 0,
            peak_prework_queue_depth: 0,
            peak_background_queued_work_item_count: 0,
            peak_background_deferred_work_item_count: 0,
            playback_active_observation_count: 0,
            recording_active_observation_count: 0,
            background_service_run_count: 0,
            background_service_defer_count: 0,
            background_service_throttle_count: 0,
            background_service_abort_count: 0,
            background_service_while_playing_count: 0,
            background_service_while_recording_count: 0,
            topology_incompatible_observation_count: 0,
            elevated_deadline_pressure_observation_count: 0,
            critical_deadline_pressure_observation_count: 0,
            overrun_deadline_pressure_observation_count: 0,
            budget_overrun_count_delta: 0,
            xrun_count_delta: 0,
            prework_service_starvation_count_delta: 0,
            prework_service_throttle_count_delta: 0,
            prework_service_yield_count_delta: 0,
            peak_hot_latency_node_id: None,
            peak_hot_latency_node_group: None,
            peak_hot_latency_node_samples: 0,
            peak_hot_latency_group: None,
            peak_hot_latency_group_node_count: 0,
            peak_hot_latency_group_total_samples: 0,
            peak_critical_path_lane: None,
            peak_critical_path_lane_node_count: 0,
            peak_critical_path_lane_plugin_backed_node_count: 0,
            peak_critical_path_lane_total_latency_samples: 0,
            summary: "observations=0".to_string(),
        };
    }

    let first_snapshot = observations[0].performance_snapshot();
    let mut receipt = RuntimePerformanceTraceReceipt {
        observation_count: observations.len(),
        first_block_sequence: first_snapshot.last_block_sequence,
        last_block_sequence: first_snapshot.last_block_sequence,
        processed_block_span: 0,
        peak_cpu_load_percent: first_snapshot.cpu_load_percent,
        peak_graph_latency_ms: first_snapshot.graph_latency_ms,
        peak_block_execution_time_ns: first_snapshot.peak_block_execution_time_ns,
        peak_block_budget_utilization_percent: first_snapshot.peak_block_budget_utilization_percent,
        peak_block_budget_overrun_ns: first_snapshot.peak_block_budget_overrun_ns,
        peak_pending_prework_target_count: first_snapshot.pending_prework_target_count,
        peak_prework_queue_depth: first_snapshot.prework_queue_depth,
        peak_background_queued_work_item_count: first_snapshot.background_queued_work_item_count,
        peak_background_deferred_work_item_count: first_snapshot
            .background_deferred_work_item_count,
        playback_active_observation_count: 0,
        recording_active_observation_count: 0,
        background_service_run_count: 0,
        background_service_defer_count: 0,
        background_service_throttle_count: 0,
        background_service_abort_count: 0,
        background_service_while_playing_count: 0,
        background_service_while_recording_count: 0,
        topology_incompatible_observation_count: 0,
        elevated_deadline_pressure_observation_count: 0,
        critical_deadline_pressure_observation_count: 0,
        overrun_deadline_pressure_observation_count: 0,
        budget_overrun_count_delta: 0,
        xrun_count_delta: 0,
        prework_service_starvation_count_delta: 0,
        prework_service_throttle_count_delta: 0,
        prework_service_yield_count_delta: 0,
        peak_hot_latency_node_id: first_snapshot.hot_latency_node_id.clone(),
        peak_hot_latency_node_group: first_snapshot.hot_latency_node_group.clone(),
        peak_hot_latency_node_samples: first_snapshot.hot_latency_node_samples,
        peak_hot_latency_group: first_snapshot.hot_latency_group.clone(),
        peak_hot_latency_group_node_count: first_snapshot.hot_latency_group_node_count,
        peak_hot_latency_group_total_samples: first_snapshot.hot_latency_group_total_samples,
        peak_critical_path_lane: first_snapshot.critical_path_lane.clone(),
        peak_critical_path_lane_node_count: first_snapshot.critical_path_lane_node_count,
        peak_critical_path_lane_plugin_backed_node_count: first_snapshot
            .critical_path_lane_plugin_backed_node_count,
        peak_critical_path_lane_total_latency_samples: first_snapshot
            .critical_path_lane_total_latency_samples,
        summary: String::new(),
    };

    let mut last_snapshot = first_snapshot.clone();
    for observation in observations {
        let snapshot = observation.performance_snapshot();
        let playback_active = observation
            .timeline_snapshot
            .last_transport_playing
            .unwrap_or(false);
        let recording_active = matches!(
            observation.recording_capture_snapshot.state,
            Some(RuntimeRecordingCaptureState::Capturing)
        );
        if playback_active {
            receipt.playback_active_observation_count =
                receipt.playback_active_observation_count.saturating_add(1);
        }
        if recording_active {
            receipt.recording_active_observation_count =
                receipt.recording_active_observation_count.saturating_add(1);
        }
        match snapshot.background_service_decision {
            Some(RuntimeDeferredServiceDecision::Run) => {
                receipt.background_service_run_count =
                    receipt.background_service_run_count.saturating_add(1);
            }
            Some(RuntimeDeferredServiceDecision::Defer) => {
                receipt.background_service_defer_count =
                    receipt.background_service_defer_count.saturating_add(1);
            }
            Some(RuntimeDeferredServiceDecision::Throttle) => {
                receipt.background_service_throttle_count =
                    receipt.background_service_throttle_count.saturating_add(1);
            }
            Some(RuntimeDeferredServiceDecision::Abort) => {
                receipt.background_service_abort_count =
                    receipt.background_service_abort_count.saturating_add(1);
            }
            None => {}
        }
        if snapshot.background_service_decision.is_some() && playback_active {
            receipt.background_service_while_playing_count = receipt
                .background_service_while_playing_count
                .saturating_add(1);
        }
        if snapshot.background_service_decision.is_some() && recording_active {
            receipt.background_service_while_recording_count = receipt
                .background_service_while_recording_count
                .saturating_add(1);
        }
        if !snapshot.scheduler_topology_compatible {
            receipt.topology_incompatible_observation_count = receipt
                .topology_incompatible_observation_count
                .saturating_add(1);
        }
        match snapshot.last_block_deadline_pressure {
            RuntimeBlockDeadlinePressure::Normal => {}
            RuntimeBlockDeadlinePressure::Elevated => {
                receipt.elevated_deadline_pressure_observation_count = receipt
                    .elevated_deadline_pressure_observation_count
                    .saturating_add(1);
            }
            RuntimeBlockDeadlinePressure::Critical => {
                receipt.critical_deadline_pressure_observation_count = receipt
                    .critical_deadline_pressure_observation_count
                    .saturating_add(1);
            }
            RuntimeBlockDeadlinePressure::Overrun => {
                receipt.overrun_deadline_pressure_observation_count = receipt
                    .overrun_deadline_pressure_observation_count
                    .saturating_add(1);
            }
        }
        receipt.last_block_sequence = snapshot.last_block_sequence;
        receipt.peak_cpu_load_percent =
            receipt.peak_cpu_load_percent.max(snapshot.cpu_load_percent);
        receipt.peak_graph_latency_ms =
            receipt.peak_graph_latency_ms.max(snapshot.graph_latency_ms);
        receipt.peak_block_execution_time_ns = receipt
            .peak_block_execution_time_ns
            .max(snapshot.peak_block_execution_time_ns);
        receipt.peak_block_budget_utilization_percent = receipt
            .peak_block_budget_utilization_percent
            .max(snapshot.peak_block_budget_utilization_percent);
        receipt.peak_block_budget_overrun_ns = receipt
            .peak_block_budget_overrun_ns
            .max(snapshot.peak_block_budget_overrun_ns);
        receipt.peak_pending_prework_target_count = receipt
            .peak_pending_prework_target_count
            .max(snapshot.pending_prework_target_count);
        receipt.peak_prework_queue_depth = receipt
            .peak_prework_queue_depth
            .max(snapshot.prework_queue_depth);
        receipt.peak_background_queued_work_item_count = receipt
            .peak_background_queued_work_item_count
            .max(snapshot.background_queued_work_item_count);
        receipt.peak_background_deferred_work_item_count = receipt
            .peak_background_deferred_work_item_count
            .max(snapshot.background_deferred_work_item_count);
        if snapshot.hot_latency_node_samples > receipt.peak_hot_latency_node_samples {
            receipt.peak_hot_latency_node_id = snapshot.hot_latency_node_id.clone();
            receipt.peak_hot_latency_node_group = snapshot.hot_latency_node_group.clone();
            receipt.peak_hot_latency_node_samples = snapshot.hot_latency_node_samples;
            receipt.peak_hot_latency_group = snapshot.hot_latency_group.clone();
            receipt.peak_hot_latency_group_node_count = snapshot.hot_latency_group_node_count;
            receipt.peak_hot_latency_group_total_samples = snapshot.hot_latency_group_total_samples;
        }
        if snapshot.critical_path_lane_total_latency_samples
            > receipt.peak_critical_path_lane_total_latency_samples
        {
            receipt.peak_critical_path_lane = snapshot.critical_path_lane.clone();
            receipt.peak_critical_path_lane_node_count = snapshot.critical_path_lane_node_count;
            receipt.peak_critical_path_lane_plugin_backed_node_count =
                snapshot.critical_path_lane_plugin_backed_node_count;
            receipt.peak_critical_path_lane_total_latency_samples =
                snapshot.critical_path_lane_total_latency_samples;
        }
        last_snapshot = snapshot;
    }

    receipt.processed_block_span = last_snapshot
        .processed_block_count
        .saturating_sub(first_snapshot.processed_block_count);
    receipt.xrun_count_delta = last_snapshot
        .xrun_count
        .saturating_sub(first_snapshot.xrun_count);
    receipt.budget_overrun_count_delta = last_snapshot
        .budget_overrun_count
        .saturating_sub(first_snapshot.budget_overrun_count);
    receipt.prework_service_starvation_count_delta = last_snapshot
        .prework_service_starvation_count
        .saturating_sub(first_snapshot.prework_service_starvation_count);
    receipt.prework_service_throttle_count_delta = last_snapshot
        .prework_service_throttle_count
        .saturating_sub(first_snapshot.prework_service_throttle_count);
    receipt.prework_service_yield_count_delta = last_snapshot
        .prework_service_yield_count
        .saturating_sub(first_snapshot.prework_service_yield_count);
    receipt.summary = format!(
        "observations={} blocks={} playback_active={} recording_active={} background={}/{}/{}/{} overlap={}/{} queue_peak={}/{} prework_delta={}/{}/{} deadline={}/{}/{} budget_overruns={} hot_node={:?}/{} hot_group={:?}/{}/{} critical_lane={:?}/{}/{}/{} topology_incompatible={}",
        receipt.observation_count,
        receipt.processed_block_span,
        receipt.playback_active_observation_count,
        receipt.recording_active_observation_count,
        receipt.background_service_run_count,
        receipt.background_service_defer_count,
        receipt.background_service_throttle_count,
        receipt.background_service_abort_count,
        receipt.background_service_while_playing_count,
        receipt.background_service_while_recording_count,
        receipt.peak_pending_prework_target_count,
        receipt.peak_prework_queue_depth,
        receipt.prework_service_starvation_count_delta,
        receipt.prework_service_throttle_count_delta,
        receipt.prework_service_yield_count_delta,
        receipt.elevated_deadline_pressure_observation_count,
        receipt.critical_deadline_pressure_observation_count,
        receipt.overrun_deadline_pressure_observation_count,
        receipt.budget_overrun_count_delta,
        receipt.peak_hot_latency_node_id,
        receipt.peak_hot_latency_node_samples,
        receipt.peak_hot_latency_group,
        receipt.peak_hot_latency_group_node_count,
        receipt.peak_hot_latency_group_total_samples,
        receipt.peak_critical_path_lane,
        receipt.peak_critical_path_lane_node_count,
        receipt.peak_critical_path_lane_plugin_backed_node_count,
        receipt.peak_critical_path_lane_total_latency_samples,
        receipt.topology_incompatible_observation_count,
    );
    receipt
}

fn build_runtime_soak_receipt(
    observation: &RuntimeObservationReport,
    event_stream_count: usize,
) -> RuntimeSoakReceipt {
    let recall_handoff = RuntimePluginRecallHandoffSnapshot::from_plugin_chain_snapshot(
        &observation.plugin_chain_snapshot,
    );
    RuntimeSoakReceipt {
        event_stream_count,
        restart_count: observation.control_snapshot.restart_count,
        stop_count: observation.control_snapshot.stop_count,
        watchdog_restart_count: observation.supervision_snapshot.watchdog_restart_count,
        safe_mode_enabled: observation.supervision_snapshot.safe_mode_enabled,
        readiness_degraded: observation.degradation_summary.readiness_degraded,
        plugin_fault_count: observation.observation.plugin_fault_count(),
        recovery_event_count: observation.observation.recovery_event_count(),
        lifecycle_event_count: observation.observation.lifecycle_event_count(),
        transport_event_count: observation.observation.transport_event_count(),
        heartbeat_event_count: observation.observation.heartbeat_event_count(),
        block_dispatch_event_count: observation.observation.block_dispatch_event_count(),
        lease_rollover_event_count: observation.observation.lease_rollover_event_count(),
        invalidation_event_count: observation.observation.invalidation_event_count(),
        completion_slot_event_count: observation.observation.completion_slot_event_count(),
        transport_fault_event_count: observation.observation.transport_fault_event_count(),
        broker_failure_event_count: observation.observation.broker_failure_event_count(),
        sandbox_operation_failure_event_count: observation
            .observation
            .sandbox_operation_failure_event_count(),
        peak_attached_sessions: observation.transport_concurrency_snapshot.peak_attached_sessions,
        peak_recovery_overlap_sessions: observation
            .transport_concurrency_snapshot
            .peak_recovery_overlap_sessions,
        peak_lingering_sessions: observation.transport_concurrency_snapshot.peak_lingering_sessions,
        pending_cleanup_waves: observation.transport_concurrency_snapshot.pending_cleanup_waves.len(),
        plugin_ready_sandbox_count: observation.plugin_lifecycle_snapshot.ready_sandbox_count,
        plugin_degraded_sandbox_count: observation.plugin_lifecycle_snapshot.degraded_sandbox_count,
        plugin_faulted_sandbox_count: observation.plugin_lifecycle_snapshot.faulted_sandbox_count,
        plugin_restarting_sandbox_count: observation
            .plugin_lifecycle_snapshot
            .restarting_sandbox_count,
        plugin_quarantined_sandbox_count: observation
            .plugin_lifecycle_snapshot
            .quarantined_sandbox_count,
        recall_stage_count: recall_handoff.stage_count,
        recovered_recall_stage_count: recall_handoff.recovered_stage_count,
        unavailable_recall_stage_count: recall_handoff.unavailable_stage_count,
        last_recovery_intent: observation
            .observation
            .last_recovery_event()
            .map(|record| record.intent),
        last_stop_reason: observation.control_snapshot.last_stop_reason,
        summary: format!(
            "events={} restarts={} watchdog_restarts={} safe_mode={} degraded={} recoveries={} transport_faults={} sandboxes={}/{}/{}/{} recall={}/{}/{}",
            event_stream_count,
            observation.control_snapshot.restart_count,
            observation.supervision_snapshot.watchdog_restart_count,
            observation.supervision_snapshot.safe_mode_enabled,
            observation.degradation_summary.readiness_degraded,
            observation.observation.recovery_event_count(),
            observation.observation.transport_fault_event_count(),
            observation.plugin_lifecycle_snapshot.ready_sandbox_count,
            observation.plugin_lifecycle_snapshot.degraded_sandbox_count,
            observation.plugin_lifecycle_snapshot.faulted_sandbox_count,
            observation.plugin_lifecycle_snapshot.quarantined_sandbox_count,
            recall_handoff.stage_count,
            recall_handoff.recovered_stage_count,
            recall_handoff.unavailable_stage_count,
        ),
    }
}

const RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS: usize = 128;
const RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS: usize = 64;

fn build_runtime_acceptance_receipt(
    readiness: RuntimeReadiness,
    effective_config: EffectiveRuntimeConfig,
    control_snapshot: RuntimeControlSnapshot,
    scheduler_topology_summary: RuntimeSchedulerTopologySummary,
    recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    media_service_snapshot: RuntimeMediaServiceSnapshot,
    clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
) -> RuntimeAcceptanceReceipt {
    let playback_ready = effective_config.block_size > 0
        && effective_config.sample_rate.0 > 0
        && scheduler_topology_summary.compatible;
    let recording_ready = recording_capture_snapshot.capture_ready
        || recording_capture_snapshot.last_checkpoint.is_some();
    let media_ready = media_service_snapshot.indexed_asset_count > 0
        && !media_service_snapshot.invalidation_active
        && matches!(
            media_service_snapshot.indexing_state,
            RuntimeMediaIndexingState::Ready
        )
        && matches!(
            media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Ready | RuntimeMediaPreviewState::Previewing
        );
    let clip_processing_ready = clip_processing_pipeline_snapshot.clip_count > 0
        && clip_processing_pipeline_snapshot.ready_clip_count
            == clip_processing_pipeline_snapshot.clip_count
        && clip_processing_pipeline_snapshot.pending_media_clip_count == 0
        && clip_processing_pipeline_snapshot.pending_warp_clip_count == 0
        && clip_processing_pipeline_snapshot.invalid_clip_count == 0;
    let plugin_ready = plugin_lifecycle_snapshot.sandbox_count > 0
        && plugin_lifecycle_snapshot.ready_sandbox_count == plugin_lifecycle_snapshot.sandbox_count
        && plugin_lifecycle_snapshot.faulted_sandbox_count == 0
        && plugin_lifecycle_snapshot.quarantined_sandbox_count == 0;
    let recovery_ready =
        !matches!(readiness, RuntimeReadiness::Failed { .. }) || control_snapshot.restart_count > 0;
    let runtime_ready_lane_count = [
        playback_ready,
        recording_ready,
        media_ready,
        clip_processing_ready,
        plugin_ready,
        recovery_ready,
    ]
    .into_iter()
    .filter(|ready| *ready)
    .count();

    RuntimeAcceptanceReceipt {
        runtime_lane_count: 6,
        runtime_ready_lane_count,
        playback_ready,
        recording_ready,
        media_ready,
        clip_processing_ready,
        plugin_ready,
        recovery_ready,
        minimum_trace_observation_count: RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS,
        minimum_soak_event_count: RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS,
        summary: format!(
            "runtime_lanes={}/{} playback={} recording={} media={} clip_processing={} plugin={} recovery={} trace_target={} soak_target={}",
            runtime_ready_lane_count,
            6,
            playback_ready,
            recording_ready,
            media_ready,
            clip_processing_ready,
            plugin_ready,
            recovery_ready,
            RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS,
            RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS,
        ),
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
        " block_summary_processed={} block_summary_last={:?}/{:?}/{}ch@{} block_summary_timing={:?}/{:?}/{:?}/{:?}/{:?}/{} block_summary_prework={:?}/{:?}/{:?} block_summary_latency_tail={}/{}/{} block_summary_levels={:?}/{:?}/{:?} block_summary_transport={}/{:?}/{} block_summary_context={:?}/{:?}/{:?}/{:?}",
        summary.processed_blocks,
        summary.last_processing_epoch,
        summary.last_block_sequence,
        summary.last_channel_count,
        summary.last_frame_count,
        summary.last_block_execution_time_ns,
        summary.last_block_deadline_budget_ns,
        summary.last_block_budget_utilization_percent,
        summary.last_block_budget_overrun_ns,
        summary.last_block_deadline_pressure,
        summary.budget_overrun_count,
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
        "\nblock_summary_processed_blocks={}\nblock_summary_last_processing_epoch={:?}\nblock_summary_last_block_sequence={:?}\nblock_summary_last_frame_count={}\nblock_summary_last_channel_count={}\nblock_summary_last_block_execution_time_ns={:?}\nblock_summary_last_block_deadline_budget_ns={:?}\nblock_summary_last_block_budget_utilization_percent={:?}\nblock_summary_last_block_budget_overrun_ns={:?}\nblock_summary_last_block_deadline_pressure={:?}\nblock_summary_budget_overrun_count={}\nblock_summary_prework_cache_state={:?}\nblock_summary_prework_cache_freshness_state={:?}\nblock_summary_last_prework_invalidation_reason={:?}\nblock_summary_total_latency_samples={}\nblock_summary_total_tail_samples={}\nblock_summary_output_tail_samples={}\nblock_summary_max_bus_tail_samples={}\nblock_summary_last_input_peak={:?}\nblock_summary_last_output_peak={:?}\nblock_summary_last_output_rms={:?}\nblock_summary_transport_epoch={}\nblock_summary_transport_transition={:?}\nblock_summary_transport_loop_wrapped={}\nblock_summary_context_anticipative={:?}\nblock_summary_transport_playing={:?}\nblock_summary_transport_tempo_bpm={:?}\nblock_summary_timeline_position_samples={:?}",
        summary.processed_blocks,
        summary.last_processing_epoch,
        summary.last_block_sequence,
        summary.last_frame_count,
        summary.last_channel_count,
        summary.last_block_execution_time_ns,
        summary.last_block_deadline_budget_ns,
        summary.last_block_budget_utilization_percent,
        summary.last_block_budget_overrun_ns,
        summary.last_block_deadline_pressure,
        summary.budget_overrun_count,
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

fn format_runtime_fault_status_compact(snapshot: &RuntimeFaultStatusSnapshot) -> String {
    format!(
        " fault_status={:?}/{:?}/faults={} xruns={} plugin_fault_active={} watchdog={} device_loss={} transport_fault={} missing_binding={} safe_mode={} restarts={}",
        snapshot.recovery_state,
        snapshot.primary_fault_cause,
        snapshot.active_fault_count,
        snapshot.xrun_overload_active,
        snapshot.plugin_fault_active,
        snapshot.watchdog_restart_count,
        snapshot.device_loss_count,
        snapshot.transport_faulted_session_count,
        snapshot.missing_plugin_binding_active,
        snapshot.safe_mode_enabled,
        snapshot.restart_count,
    )
}

fn format_runtime_fault_diagnostic_receipt_compact(
    receipt: &RuntimeFaultDiagnosticReceipt,
) -> String {
    format!(
        " fault_diagnostic={:?}/{:?}/{:?}/rebindable={} contributions={}",
        receipt.primary_family,
        receipt.primary_fault_cause,
        receipt.interruption_class,
        receipt.rebindable,
        receipt.contributions.len(),
    )
}

fn format_runtime_recording_capture_snapshot_compact(
    snapshot: &RuntimeRecordingCaptureSnapshot,
) -> String {
    let checkpoint_class = snapshot
        .active_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.checkpoint_class)
        .or_else(|| {
            snapshot
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class)
        });
    let interruption_class = snapshot
        .active_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.interruption_class)
        .or_else(|| {
            snapshot
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class)
        });
    format!(
        " recording_capture={:?}/{:?}/{:?} ready={} take={:?} track={:?} frames={} events={} blocks={} pressure={} last_take={:?} last_path={:?} last_duration={:?}",
        snapshot.state,
        snapshot.capture_kind,
        checkpoint_class,
        snapshot.capture_ready,
        snapshot.active_take_id,
        snapshot.active_track_id,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.buffered_block_count,
        snapshot.pressure_event_count,
        snapshot.last_committed_take_id,
        snapshot.last_committed_path,
        snapshot.last_committed_duration_samples,
    ) + &format!(" recording_capture_interruption={interruption_class:?}")
}

fn format_runtime_recording_capture_snapshot_multiline(
    snapshot: &RuntimeRecordingCaptureSnapshot,
) -> String {
    format!(
        concat!(
            "\nrecording_capture_ready={}",
            "\nrecording_capture_state={:?}",
            "\nrecording_capture_kind={:?}",
            "\nrecording_capture_active_take_id={:?}",
            "\nrecording_capture_active_track_id={:?}",
            "\nrecording_capture_start_samples={:?}",
            "\nrecording_capture_active_path={:?}",
            "\nrecording_capture_buffered_blocks={}",
            "\nrecording_capture_buffered_frames={}",
            "\nrecording_capture_buffered_events={}",
            "\nrecording_capture_channel_count={}",
            "\nrecording_capture_peak_level={:?}",
            "\nrecording_capture_pressure_events={}",
            "\nrecording_capture_active_checkpoint={}",
            "\nrecording_capture_last_checkpoint={}",
            "\nrecording_capture_last_committed_take_id={:?}",
            "\nrecording_capture_last_committed_path={:?}",
            "\nrecording_capture_last_committed_duration_samples={:?}",
            "\nrecording_capture_last_error={:?}",
            "\nrecording_capture_summary={}",
        ),
        snapshot.capture_ready,
        snapshot.state,
        snapshot.capture_kind,
        snapshot.active_take_id,
        snapshot.active_track_id,
        snapshot.capture_start_samples,
        snapshot.active_capture_path,
        snapshot.buffered_block_count,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.captured_channel_count,
        snapshot.peak_level,
        snapshot.pressure_event_count,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.summary.as_str())
            .unwrap_or("none"),
        snapshot.last_committed_take_id,
        snapshot.last_committed_path,
        snapshot.last_committed_duration_samples,
        snapshot.last_error,
        snapshot.summary,
    )
}

fn format_runtime_fault_status_multiline(snapshot: &RuntimeFaultStatusSnapshot) -> String {
    format!(
        concat!(
            "\nfault_status_recovery_state={:?}",
            "\nfault_status_primary_fault_cause={:?}",
            "\nfault_status_active_fault_count={}",
            "\nfault_status_xrun_overload_active={}",
            "\nfault_status_plugin_fault_active={}",
            "\nfault_status_watchdog_active={}",
            "\nfault_status_device_loss_active={}",
            "\nfault_status_transport_fault_active={}",
            "\nfault_status_missing_plugin_binding_active={}",
            "\nfault_status_safe_mode_enabled={}",
            "\nfault_status_restart_count={}",
            "\nfault_status_watchdog_restart_count={}",
            "\nfault_status_plugin_fault_count={}",
            "\nfault_status_transport_faulted_session_count={}",
            "\nfault_status_device_loss_count={}",
            "\nfault_status_summary={}",
        ),
        snapshot.recovery_state,
        snapshot.primary_fault_cause,
        snapshot.active_fault_count,
        snapshot.xrun_overload_active,
        snapshot.plugin_fault_active,
        snapshot.watchdog_active,
        snapshot.device_loss_active,
        snapshot.transport_fault_active,
        snapshot.missing_plugin_binding_active,
        snapshot.safe_mode_enabled,
        snapshot.restart_count,
        snapshot.watchdog_restart_count,
        snapshot.plugin_fault_count,
        snapshot.transport_faulted_session_count,
        snapshot.device_loss_count,
        snapshot.summary,
    )
}

fn format_runtime_fault_diagnostic_receipt_multiline(
    receipt: &RuntimeFaultDiagnosticReceipt,
) -> String {
    let contributions = receipt
        .contributions
        .iter()
        .map(|contribution| contribution.summary.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        concat!(
            "\nfault_diagnostic_primary_family={:?}",
            "\nfault_diagnostic_primary_fault_cause={:?}",
            "\nfault_diagnostic_interruption_class={:?}",
            "\nfault_diagnostic_recovery_state={:?}",
            "\nfault_diagnostic_safe_mode_enabled={}",
            "\nfault_diagnostic_rebindable={}",
            "\nfault_diagnostic_contribution_count={}",
            "\nfault_diagnostic_contributions={}",
            "\nfault_diagnostic_summary={}",
        ),
        receipt.primary_family,
        receipt.primary_fault_cause,
        receipt.interruption_class,
        receipt.recovery_state,
        receipt.safe_mode_enabled,
        receipt.rebindable,
        receipt.contributions.len(),
        if contributions.is_empty() {
            "none"
        } else {
            contributions.as_str()
        },
        receipt.summary,
    )
}

fn format_runtime_interruption_summary_compact(summary: &RuntimeInterruptionSummary) -> String {
    format!(
        " interruption={:?}/active={} rebindable={} recovery={:?} primary={:?} deferred={:?}/{:?}",
        summary.class,
        summary.active,
        summary.rebindable,
        summary.recovery_state,
        summary.primary_fault_cause,
        summary.deferred_service_class,
        summary.deferred_service_decision,
    )
}

fn format_runtime_interruption_summary_multiline(summary: &RuntimeInterruptionSummary) -> String {
    format!(
        concat!(
            "\ninterruption_active={}",
            "\ninterruption_class={:?}",
            "\ninterruption_rebindable={}",
            "\ninterruption_recovery_state={:?}",
            "\ninterruption_primary_fault_cause={:?}",
            "\ninterruption_safe_mode_enabled={}",
            "\ninterruption_deferred_service_class={:?}",
            "\ninterruption_deferred_service_decision={:?}",
            "\ninterruption_summary={}",
        ),
        summary.active,
        summary.class,
        summary.rebindable,
        summary.recovery_state,
        summary.primary_fault_cause,
        summary.safe_mode_enabled,
        summary.deferred_service_class,
        summary.deferred_service_decision,
        summary.summary,
    )
}

fn format_runtime_tempo_map_snapshot_compact(snapshot: &RuntimeTempoMapSnapshot) -> String {
    format!(
        " tempo_map_segments={} tempo_map_active={:?}/{:?} tempo_map_source={:?} tempo_map_tempo={:.3} tempo_map_next_segment={:?}",
        snapshot.segment_count,
        snapshot.active_segment_index,
        snapshot.active_segment_id,
        snapshot.tempo_source,
        snapshot.resolved_tempo_bpm,
        snapshot.next_segment_start_samples,
    )
}

fn format_runtime_tempo_map_snapshot_multiline(snapshot: &RuntimeTempoMapSnapshot) -> String {
    let segment_lines = snapshot
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            format!(
                "\ntempo_map_segment_{}={}/interp={:?}/start={}/end={:?}/tempo={:.3}->{:?}/active={}",
                index,
                segment.segment_id,
                segment.interpolation,
                segment.start_samples,
                segment.end_samples,
                segment.start_tempo_bpm,
                segment.end_tempo_bpm,
                segment.covers_timeline_position,
            )
        })
        .collect::<String>();
    format!(
        "\ntempo_map_segment_count={}\ntempo_map_active_segment_id={:?}\ntempo_map_active_segment_index={:?}\ntempo_map_next_segment_start_samples={:?}\ntempo_map_resolved_tempo_bpm={:.3}\ntempo_map_source={:?}\ntempo_map_timeline_position_samples={:?}{}",
        snapshot.segment_count,
        snapshot.active_segment_id,
        snapshot.active_segment_index,
        snapshot.next_segment_start_samples,
        snapshot.resolved_tempo_bpm,
        snapshot.tempo_source,
        snapshot.timeline_position_samples,
        segment_lines,
    )
}

fn format_runtime_warp_pipeline_snapshot_compact(snapshot: &RuntimeWarpPipelineSnapshot) -> String {
    format!(
        " warp_clips={}/{}/{}/{} warp_tempo={:.3}/{:?}/{:?}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        snapshot.resolved_project_tempo_segment_id,
    )
}

fn format_runtime_warp_pipeline_snapshot_multiline(
    snapshot: &RuntimeWarpPipelineSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nwarp_clip_{}={}/mode={:?}/readiness={:?}/ratio={:.3}/project_tempo={:.3}/{:?}/{:?}/source_tempo={:?}/error={:?}",
                index,
                clip.clip_id,
                clip.mode,
                clip.readiness,
                clip.realized_ratio,
                clip.project_tempo_bpm,
                clip.project_tempo_source,
                clip.project_tempo_segment_id,
                clip.source_tempo_bpm,
                clip.last_error,
            )
        })
        .collect::<String>();
    format!(
        "\nwarp_clip_count={}\nwarp_ready_clip_count={}\nwarp_degraded_clip_count={}\nwarp_bypassed_clip_count={}\nwarp_active_clip_count={}\nwarp_resolved_project_tempo_bpm={:.3}\nwarp_resolved_project_tempo_source={:?}\nwarp_resolved_project_tempo_segment_id={:?}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.active_warp_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        snapshot.resolved_project_tempo_segment_id,
        clip_lines,
    )
}

fn format_runtime_clip_processing_pipeline_snapshot_compact(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    format!(
        " clip_processing_clips={}/{}/{}/{} clip_processing_shapes={}/{}/{} clip_processing_treatment_stages={}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count + snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
    )
}

fn format_runtime_clip_processing_pipeline_snapshot_multiline(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nclip_processing_clip_{}={}/readiness={:?}/warp={:?}/{:?}/{:?}/fade_in={}/{:?}/fade_out={}/{:?}/gain={:.3}->{:.3}/{:?}/stages={:?}/error={:?}",
                index,
                clip.clip_id,
                clip.readiness,
                clip.warp_mode,
                clip.realized_warp_ratio,
                clip.project_tempo_source,
                clip.fade_in.duration_samples,
                clip.fade_in.shape,
                clip.fade_out.duration_samples,
                clip.fade_out.shape,
                clip.clip_gain.start_linear,
                clip.clip_gain.end_linear,
                clip.clip_gain.shape,
                clip.treatment_stages,
                clip.last_error,
            )
        })
        .collect::<String>();
    format!(
        "\nclip_processing_clip_count={}\nclip_processing_ready_clip_count={}\nclip_processing_pending_media_clip_count={}\nclip_processing_pending_warp_clip_count={}\nclip_processing_invalid_clip_count={}\nclip_processing_faded_clip_count={}\nclip_processing_gain_shaped_clip_count={}\nclip_processing_warped_clip_count={}\nclip_processing_treatment_stage_count={}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
        clip_lines,
    )
}

fn format_runtime_plugin_chain_snapshot_compact(snapshot: &RuntimePluginChainSnapshot) -> String {
    format!(
        " plugin_chains={}/{} plugin_chain_placement={}/{}/{} plugin_chain_pending={} plugin_chain_settling={} plugin_chain_compensated={} plugin_chain_degraded={} plugin_chain_bypassed={} plugin_chain_missing={} plugin_chain_rebindable={} plugin_chain_terminal={} plugin_chain_latency={}/{} plugin_chain_tail={}",
        snapshot.chain_count,
        snapshot.stage_count,
        snapshot.shared_sandbox_stage_count,
        snapshot.isolated_sandbox_stage_count,
        snapshot.in_process_stage_count,
        snapshot.pending_render_stage_count,
        snapshot.settling_stage_count,
        snapshot.compensated_stage_count,
        snapshot.degraded_stage_count,
        snapshot.bypassed_stage_count,
        snapshot.missing_binding_stage_count,
        snapshot.rebindable_stage_count,
        snapshot.terminal_stage_count,
        snapshot.total_realized_latency_samples,
        snapshot.total_planned_latency_samples,
        snapshot.total_tail_samples,
    )
}

fn format_runtime_plugin_discovery_snapshot_compact(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    format!(
        " plugin_scans={} plugin_filtered_scans={} plugin_discovered_types={} plugin_discovered_formats={} plugin_capability_coverage={} plugin_last_scan={}",
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        snapshot.capability_coverage.summary,
        snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.summary.as_str())
            .unwrap_or("none"),
    )
}

fn format_runtime_plugin_lifecycle_snapshot_compact(
    snapshot: &RuntimePluginLifecycleSnapshot,
) -> String {
    format!(
        " plugin_sandboxes={}/{} plugin_sandbox_placement={}/{} plugin_sandbox_rebindable={} plugin_sandbox_terminal={}",
        snapshot.sandbox_count,
        snapshot.active_sandbox_count,
        snapshot.shared_sandbox_count,
        snapshot.isolated_sandbox_count,
        snapshot.rebindable_sandbox_count,
        snapshot.terminal_sandbox_count,
    )
}

fn format_runtime_plugin_recall_snapshot_compact(snapshot: &RuntimePluginRecallSnapshot) -> String {
    format!(
        "{:?}/sandbox={:?}/plugin={:?}/{:?}/lifecycle={:?}/{:?}/{:?}/readiness={:?}/recoveries={}/restarts={}/faults={}/fault_kind={:?}/stop_reason={:?}/degraded={:?}",
        snapshot.state,
        snapshot.payload.sandbox_id.as_deref(),
        snapshot.payload.plugin_type_id.as_deref(),
        snapshot.payload.plugin_format,
        snapshot.payload.lifecycle_state,
        snapshot.payload.lifecycle_stage,
        snapshot.payload.transport_stage,
        snapshot.payload.readiness_state.as_deref(),
        snapshot.payload.recovery_count,
        snapshot.payload.restart_count,
        snapshot.payload.fault_count,
        snapshot.payload.last_fault_kind,
        snapshot.payload.last_stop_reason,
        &snapshot.payload.degraded_reasons,
    )
}

fn format_runtime_plugin_discovery_snapshot_multiline(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    let last_scan = snapshot
        .last_scan
        .as_ref()
        .map(|scan| {
            format!(
                "\nplugin_last_scan_handle={}\nplugin_last_scan_roots={:?}\nplugin_last_scan_formats={:?}\nplugin_last_scan_targeted_format_count={}\nplugin_last_scan_discovered_type_count={}\nplugin_last_scan_summary={}",
                scan.scan_handle.0,
                scan.roots,
                scan.formats,
                scan.targeted_format_count,
                scan.discovered_type_count,
                scan.summary,
            )
        })
        .unwrap_or_default();
    let format_coverage_lines = snapshot
        .format_coverage
        .iter()
        .enumerate()
        .map(|(index, coverage)| {
            format!(
                "\nplugin_format_coverage_{}={:?}/types={}/features={}/{}/{}/{}/{} snapshot={} prepare={} activate={} midi_in={} midi_out={} max_audio_buses={} max_parameters={}",
                index,
                coverage.format,
                coverage.discovered_type_count,
                coverage.audio_effect_count,
                coverage.instrument_count,
                coverage.analyzer_count,
                coverage.utility_count,
                coverage.note_effect_count,
                coverage.supports_snapshot_count,
                coverage.supports_prepare_count,
                coverage.supports_activate_count,
                coverage.accepts_midi_count,
                coverage.produces_midi_count,
                coverage.max_audio_bus_count,
                coverage.max_parameter_count,
            )
        })
        .collect::<String>();
    let discovered_type_lines = snapshot
        .discovered_types
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "\nplugin_discovered_type_{}={}/plugin_id={}/vendor={}/name={}/format={:?}/version={:?}/features={:?}/io={:?}/audio_buses={}/parameters={}",
                index,
                record.plugin_type_id,
                record.plugin_id,
                record.vendor,
                record.name,
                record.format,
                record.version,
                record.features,
                record.default_io_layout,
                record.audio_bus_count,
                record.parameter_count,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_scan_count={}\nplugin_format_filtered_scan_count={}\nplugin_discovered_type_count={}\nplugin_discovered_format_count={}\nplugin_capability_coverage_summary={}\nplugin_capability_coverage_multi_format_catalog={}\nplugin_capability_coverage_max_audio_bus_count={}\nplugin_capability_coverage_max_parameter_count={}{}{}{}",
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        snapshot.capability_coverage.summary,
        snapshot.capability_coverage.multi_format_catalog,
        snapshot.capability_coverage.max_audio_bus_count,
        snapshot.capability_coverage.max_parameter_count,
        last_scan,
        format_coverage_lines,
        discovered_type_lines,
    )
}

fn format_runtime_plugin_lifecycle_snapshot_multiline(
    snapshot: &RuntimePluginLifecycleSnapshot,
) -> String {
    let sandbox_lines = snapshot
        .sandboxes
        .iter()
        .enumerate()
        .map(|(index, sandbox)| {
            format!(
                "\nplugin_sandbox_{}={}/group={}/placement={:?}/rule={:?}/members={}/continuity={:?}/rebindable={}/state={:?}/lifecycle={:?}/transport={:?}/ready={:?}/restarts={}/recoveries={}/faults={}/active={}/transport_active={}/degraded={:?}",
                index,
                sandbox.sandbox_id,
                sandbox.sandbox_group_key,
                sandbox.placement_outcome,
                sandbox.placement_rule_id,
                sandbox.shared_boundary_member_count,
                sandbox.continuity_class,
                sandbox.rebindable,
                sandbox.state,
                sandbox.lifecycle_stage,
                sandbox.transport_stage,
                sandbox.readiness_state,
                sandbox.restart_count,
                sandbox.recovery_count,
                sandbox.fault_count,
                sandbox.active,
                sandbox.active_transport,
                sandbox.degraded_reasons,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_sandbox_count={}\nplugin_active_sandbox_count={}\nplugin_shared_sandbox_count={}\nplugin_isolated_sandbox_count={}\nplugin_ready_sandbox_count={}\nplugin_booting_sandbox_count={}\nplugin_degraded_sandbox_count={}\nplugin_faulted_sandbox_count={}\nplugin_restarting_sandbox_count={}\nplugin_quarantined_sandbox_count={}\nplugin_stopped_sandbox_count={}\nplugin_rebindable_sandbox_count={}\nplugin_terminal_sandbox_count={}{}",
        snapshot.sandbox_count,
        snapshot.active_sandbox_count,
        snapshot.shared_sandbox_count,
        snapshot.isolated_sandbox_count,
        snapshot.ready_sandbox_count,
        snapshot.booting_sandbox_count,
        snapshot.degraded_sandbox_count,
        snapshot.faulted_sandbox_count,
        snapshot.restarting_sandbox_count,
        snapshot.quarantined_sandbox_count,
        snapshot.stopped_sandbox_count,
        snapshot.rebindable_sandbox_count,
        snapshot.terminal_sandbox_count,
        sandbox_lines,
    )
}

fn format_runtime_plugin_chain_snapshot_multiline(snapshot: &RuntimePluginChainSnapshot) -> String {
    let chain_lines = snapshot
        .chains
        .iter()
        .enumerate()
        .map(|(chain_index, chain)| {
            let stage_lines = chain
                .stages
                .iter()
                .enumerate()
                .map(|(stage_index, stage)| {
                    format!(
                        "\nplugin_chain_{}_stage_{}={}/sandbox={:?}/group={:?}/placement={:?}/rule={:?}/members={}/continuity={:?}/rebindable={}/lifecycle={:?}/{:?}/transport={:?}/recall={}/compensation={:?}/latency={}/{:?}/{:?}/bypassed={}/active_transport={}/degraded_reasons={:?}",
                        chain_index,
                        stage_index,
                        stage.node_id,
                        stage.sandbox_id,
                        stage.sandbox_group_key,
                        stage.placement_outcome,
                        stage.placement_rule_id,
                        stage.shared_boundary_member_count,
                        stage.continuity_class,
                        stage.rebindable,
                        stage.lifecycle_state,
                        stage.lifecycle_stage,
                        stage.transport_stage,
                        format_runtime_plugin_recall_snapshot_compact(&stage.recall),
                        stage.compensation_state,
                        stage.planned_latency_samples,
                        stage.realized_latency_samples,
                        stage.tail_samples,
                        stage.bypassed,
                        stage.active_transport,
                        stage.degraded_reasons,
                    )
                })
                .collect::<String>();
            format!(
                "\nplugin_chain_{}={}/track={:?}/bus={:?}/console={:?}/send_return={:?}/stages={}/shared={}/isolated={}/in_process={}/pending={}/settling={}/compensated={}/degraded={}/bypassed={}/missing={}/rebindable={}/terminal={}/latency={}/{}/{}{}",
                chain_index,
                chain.chain_id,
                chain.track_lane_id,
                chain.bus_group_id,
                chain.console_group_id,
                chain.send_return_id,
                chain.stage_count,
                chain.shared_sandbox_stage_count,
                chain.isolated_sandbox_stage_count,
                chain.in_process_stage_count,
                chain.pending_render_stage_count,
                chain.settling_stage_count,
                chain.compensated_stage_count,
                chain.degraded_stage_count,
                chain.bypassed_stage_count,
                chain.missing_binding_stage_count,
                chain.rebindable_stage_count,
                chain.terminal_stage_count,
                chain.total_planned_latency_samples,
                chain.total_realized_latency_samples,
                chain.total_tail_samples,
                stage_lines,
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_chain_count={}\nplugin_chain_stage_count={}\nplugin_chain_shared_sandbox_stage_count={}\nplugin_chain_isolated_sandbox_stage_count={}\nplugin_chain_in_process_stage_count={}\nplugin_chain_pending_render_stage_count={}\nplugin_chain_settling_stage_count={}\nplugin_chain_compensated_stage_count={}\nplugin_chain_degraded_stage_count={}\nplugin_chain_bypassed_stage_count={}\nplugin_chain_missing_binding_stage_count={}\nplugin_chain_rebindable_stage_count={}\nplugin_chain_terminal_stage_count={}\nplugin_chain_total_planned_latency_samples={}\nplugin_chain_total_realized_latency_samples={}\nplugin_chain_total_tail_samples={}{}",
        snapshot.chain_count,
        snapshot.stage_count,
        snapshot.shared_sandbox_stage_count,
        snapshot.isolated_sandbox_stage_count,
        snapshot.in_process_stage_count,
        snapshot.pending_render_stage_count,
        snapshot.settling_stage_count,
        snapshot.compensated_stage_count,
        snapshot.degraded_stage_count,
        snapshot.bypassed_stage_count,
        snapshot.missing_binding_stage_count,
        snapshot.rebindable_stage_count,
        snapshot.terminal_stage_count,
        snapshot.total_planned_latency_samples,
        snapshot.total_realized_latency_samples,
        snapshot.total_tail_samples,
        chain_lines,
    )
}

fn format_runtime_routed_plugin_chain_summary_compact(
    summary: &RuntimeRoutedPluginChainSummary,
) -> String {
    format!(
        "{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
        summary.chain_count,
        summary.stage_count,
        summary.pending_render_stage_count,
        summary.settling_stage_count,
        summary.compensated_stage_count,
        summary.degraded_stage_count,
        summary.bypassed_stage_count,
        summary.missing_binding_stage_count,
        summary.total_planned_latency_samples,
        summary.total_realized_latency_samples,
        summary.total_tail_samples,
    )
}

fn format_runtime_metering_snapshot_compact(snapshot: &RuntimeMeteringSnapshot) -> String {
    let track_lane_shapes = snapshot
        .track_lanes
        .iter()
        .map(|track_lane| {
            format!(
                "{}:{}",
                track_lane.track_lane_id, track_lane.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let bus_group_shapes = snapshot
        .bus_groups
        .iter()
        .map(|bus_group| {
            format!(
                "{}:{}",
                bus_group.bus_group_id, bus_group.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let send_return_shapes = snapshot
        .send_returns
        .iter()
        .map(|send_return| {
            format!(
                "{}:{}",
                send_return.send_return_id, send_return.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let console_group_shapes = snapshot
        .console_groups
        .iter()
        .map(|console_group| {
            format!(
                "{}:{}",
                console_group.console_group_id, console_group.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    format!(
        " metering_snapshot_meters={} metering_snapshot_main={:?}/{:?} metering_snapshot_loudness={:?}/{:?}/{:?} metering_snapshot_clipped={} metering_snapshot_routes={}/{}/{}/{} metering_snapshot_track_lane_shapes={} metering_snapshot_bus_group_shapes={} metering_snapshot_send_return_shapes={} metering_snapshot_console_group_shapes={}",
        snapshot.meter_count,
        snapshot.main_output_peak_level,
        snapshot.main_output_rms_level,
        snapshot.momentary_loudness_lufs,
        snapshot.short_term_loudness_lufs,
        snapshot.integrated_loudness_lufs,
        snapshot.clipped_sample_count,
        snapshot.track_lanes.len(),
        snapshot.bus_groups.len(),
        snapshot.send_returns.len(),
        snapshot.console_groups.len(),
        track_lane_shapes,
        bus_group_shapes,
        send_return_shapes,
        console_group_shapes,
    )
}

fn format_runtime_metering_snapshot_multiline(snapshot: &RuntimeMeteringSnapshot) -> String {
    let meter_lines = snapshot
        .meters
        .iter()
        .enumerate()
        .map(|(index, meter)| {
            format!(
                "\nmetering_snapshot_meter_{}={}/{:?}/peak={:.3}/rms={:.3}/track_lane_id={:?}/bus_group_id={:?}/console_group_id={:?}/send_return_id={:?}/latency={}/tail={}/producers={:?}",
                index,
                meter.bus_id,
                meter.topology_role,
                meter.peak_level,
                meter.rms_level,
                meter.track_lane_id,
                meter.bus_group_id,
                meter.console_group_id,
                meter.send_return_id,
                meter.latency_samples,
                meter.tail_samples,
                meter.producer_node_ids,
            )
        })
        .collect::<String>();
    let track_lane_lines = snapshot
        .track_lanes
        .iter()
        .enumerate()
        .map(|(index, track_lane)| {
            format!(
                "\nmetering_snapshot_track_lane_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/producers={:?}/input={:?}/output={:?}",
                index,
                track_lane.track_lane_id,
                track_lane.aggregate.meter_count,
                track_lane.aggregate.peak_level,
                track_lane.aggregate.rms_level,
                track_lane.aggregate.metered_bus_ids,
                track_lane.aggregate.producer_node_ids,
                track_lane.input_bus_ids,
                track_lane.output_bus_ids,
            )
        })
        .collect::<String>();
    let bus_group_lines = snapshot
        .bus_groups
        .iter()
        .enumerate()
        .map(|(index, bus_group)| {
            format!(
                "\nmetering_snapshot_bus_group_{}={}/roles={:?}/meters={}/peak={:?}/rms={:?}/buses={:?}/nodes={:?}/input={:?}/output={:?}",
                index,
                bus_group.bus_group_id,
                bus_group.topology_roles,
                bus_group.aggregate.meter_count,
                bus_group.aggregate.peak_level,
                bus_group.aggregate.rms_level,
                bus_group.aggregate.metered_bus_ids,
                bus_group.node_ids,
                bus_group.input_bus_ids,
                bus_group.output_bus_ids,
            )
        })
        .collect::<String>();
    let console_group_lines = snapshot
        .console_groups
        .iter()
        .enumerate()
        .map(|(index, console_group)| {
            format!(
                "\nmetering_snapshot_console_group_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/nodes={:?}/input={:?}/output={:?}",
                index,
                console_group.console_group_id,
                console_group.aggregate.meter_count,
                console_group.aggregate.peak_level,
                console_group.aggregate.rms_level,
                console_group.aggregate.metered_bus_ids,
                console_group.node_ids,
                console_group.input_bus_ids,
                console_group.output_bus_ids,
            )
        })
        .collect::<String>();
    let send_return_lines = snapshot
        .send_returns
        .iter()
        .enumerate()
        .map(|(index, send_return)| {
            format!(
                "\nmetering_snapshot_send_return_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/sends={:?}/returns={:?}/input={:?}/output={:?}",
                index,
                send_return.send_return_id,
                send_return.aggregate.meter_count,
                send_return.aggregate.peak_level,
                send_return.aggregate.rms_level,
                send_return.aggregate.metered_bus_ids,
                send_return.send_node_ids,
                send_return.return_node_ids,
                send_return.input_bus_ids,
                send_return.output_bus_ids,
            )
        })
        .collect::<String>();
    format!(
        "\nmetering_snapshot_meter_count={}\nmetering_snapshot_main_output_peak_level={:?}\nmetering_snapshot_main_output_rms_level={:?}\nmetering_snapshot_momentary_loudness_lufs={:?}\nmetering_snapshot_short_term_loudness_lufs={:?}\nmetering_snapshot_integrated_loudness_lufs={:?}\nmetering_snapshot_clipped_sample_count={}\nmetering_snapshot_track_lane_count={}\nmetering_snapshot_bus_group_count={}\nmetering_snapshot_send_return_count={}\nmetering_snapshot_console_group_count={}{}{}{}{}{}",
        snapshot.meter_count,
        snapshot.main_output_peak_level,
        snapshot.main_output_rms_level,
        snapshot.momentary_loudness_lufs,
        snapshot.short_term_loudness_lufs,
        snapshot.integrated_loudness_lufs,
        snapshot.clipped_sample_count,
        snapshot.track_lanes.len(),
        snapshot.bus_groups.len(),
        snapshot.send_returns.len(),
        snapshot.console_groups.len(),
        meter_lines,
        track_lane_lines,
        bus_group_lines,
        console_group_lines,
        send_return_lines,
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
        " execution_topology_summary_nodes={} execution_topology_summary_roles={}/{}/{}/{}/{} execution_topology_summary_groups={}/{}/{} execution_topology_summary_plugin_chain={} execution_topology_summary_lanes={} execution_topology_summary_lane_shapes={}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.console_group_count,
        format_runtime_routed_plugin_chain_summary_compact(&summary.plugin_chain),
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
                "\nexecution_topology_summary_lane_{}={:?}/groups={:?}/nodes={:?}/roles={:?}/track_lanes={:?}/bus_groups={:?}/console_groups={:?}/send_returns={:?}",
                index,
                lane.lane,
                lane.groups,
                lane.node_ids,
                lane.topology_roles,
                lane.track_lane_ids,
                lane.bus_group_ids,
                lane.console_group_ids,
                lane.send_return_ids,
            )
        })
        .collect::<String>();
    let track_lane_lines = summary
        .track_lanes
        .iter()
        .enumerate()
        .map(|(index, track_lane)| {
            format!(
                "\nexecution_topology_summary_track_lane_{}={}/nodes={:?}/bus_groups={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                track_lane.track_lane_id,
                track_lane.node_ids,
                track_lane.bus_group_ids,
                track_lane.input_bus_ids,
                track_lane.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&track_lane.plugin_chain),
            )
        })
        .collect::<String>();
    let bus_group_lines = summary
        .bus_groups
        .iter()
        .enumerate()
        .map(|(index, bus_group)| {
            format!(
                "\nexecution_topology_summary_bus_group_{}={}/roles={:?}/nodes={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                bus_group.bus_group_id,
                bus_group.topology_roles,
                bus_group.node_ids,
                bus_group.input_bus_ids,
                bus_group.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&bus_group.plugin_chain),
            )
        })
        .collect::<String>();
    let console_group_lines = summary
        .console_groups
        .iter()
        .enumerate()
        .map(|(index, console_group)| {
            format!(
                "\nexecution_topology_summary_console_group_{}={}/nodes={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                console_group.console_group_id,
                console_group.node_ids,
                console_group.input_bus_ids,
                console_group.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&console_group.plugin_chain),
            )
        })
        .collect::<String>();
    let send_return_lines = summary
        .send_returns
        .iter()
        .enumerate()
        .map(|(index, send_return)| {
            format!(
                "\nexecution_topology_summary_send_return_{}={}/sends={:?}/returns={:?}/input={:?}/output={:?}/plugin_chain={}",
                index,
                send_return.send_return_id,
                send_return.send_node_ids,
                send_return.return_node_ids,
                send_return.input_bus_ids,
                send_return.output_bus_ids,
                format_runtime_routed_plugin_chain_summary_compact(&send_return.plugin_chain),
            )
        })
        .collect::<String>();
    let node_lines = summary
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            format!(
                "\nexecution_topology_summary_node_{}={}/{:?}/{:?}/{:?}/track_lane_id={:?}/bus_group_id={:?}/console_group_id={:?}/send_return_id={:?}/input={}/output={}/plugin={:?}/plugin_recall={:?}/plugin_recall_payload={:?}/plugin_compensation={:?}/plugin_realized_latency={:?}/plugin_tail={:?}",
                index,
                node.node_id,
                node.lane,
                node.group,
                node.topology_role,
                node.track_lane_id,
                node.bus_group_id,
                node.console_group_id,
                node.send_return_id,
                node.input_bus_id,
                node.output_bus_id,
                node.plugin_sandbox_id,
                node.plugin_recall_state,
                node.plugin_recall
                    .as_ref()
                    .map(format_runtime_plugin_recall_snapshot_compact),
                node.plugin_compensation_state,
                node.plugin_realized_latency_samples,
                node.plugin_tail_samples,
            )
        })
        .collect::<String>();
    format!(
        "\nexecution_topology_summary_node_count={}\nexecution_topology_summary_utility_nodes={}\nexecution_topology_summary_track_lane_nodes={}\nexecution_topology_summary_bus_nodes={}\nexecution_topology_summary_send_return_nodes={}\nexecution_topology_summary_console_nodes={}\nexecution_topology_summary_lane_count={}\nexecution_topology_summary_track_lane_groups={}\nexecution_topology_summary_bus_groups={}\nexecution_topology_summary_send_return_groups={}\nexecution_topology_summary_console_groups={}\nexecution_topology_summary_plugin_chain={}{}{}{}{}{}{}",
        summary.node_count,
        summary.utility_node_count,
        summary.track_lane_node_count,
        summary.bus_node_count,
        summary.send_return_node_count,
        summary.console_node_count,
        summary.lane_count,
        summary.track_lane_group_count,
        summary.bus_group_count,
        summary.send_return_group_count,
        summary.console_group_count,
        format_runtime_routed_plugin_chain_summary_compact(&summary.plugin_chain),
        lane_lines,
        track_lane_lines,
        bus_group_lines,
        console_group_lines,
        send_return_lines,
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
            "\"last_block_execution_time_ns\":{},",
            "\"last_block_deadline_budget_ns\":{},",
            "\"last_block_budget_utilization_percent\":{},",
            "\"last_block_budget_overrun_ns\":{},",
            "\"last_block_deadline_pressure\":\"{:?}\",",
            "\"budget_overrun_count\":{},",
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
        json_option_u64(summary.last_block_execution_time_ns),
        json_option_u64(summary.last_block_deadline_budget_ns),
        json_option_f32(summary.last_block_budget_utilization_percent),
        json_option_u64(summary.last_block_budget_overrun_ns),
        summary.last_block_deadline_pressure,
        summary.budget_overrun_count,
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

fn json_runtime_fault_status(snapshot: &RuntimeFaultStatusSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"recovery_state\":{},",
            "\"primary_fault_cause\":{},",
            "\"active_fault_count\":{},",
            "\"xrun_overload_active\":{},",
            "\"plugin_fault_active\":{},",
            "\"watchdog_active\":{},",
            "\"device_loss_active\":{},",
            "\"transport_fault_active\":{},",
            "\"missing_plugin_binding_active\":{},",
            "\"safe_mode_enabled\":{},",
            "\"restart_count\":{},",
            "\"watchdog_restart_count\":{},",
            "\"plugin_fault_count\":{},",
            "\"transport_faulted_session_count\":{},",
            "\"device_loss_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", snapshot.recovery_state)),
        json_option_string(
            snapshot
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        snapshot.active_fault_count,
        snapshot.xrun_overload_active,
        snapshot.plugin_fault_active,
        snapshot.watchdog_active,
        snapshot.device_loss_active,
        snapshot.transport_fault_active,
        snapshot.missing_plugin_binding_active,
        snapshot.safe_mode_enabled,
        snapshot.restart_count,
        snapshot.watchdog_restart_count,
        snapshot.plugin_fault_count,
        snapshot.transport_faulted_session_count,
        snapshot.device_loss_count,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_fault_contribution_receipt(receipt: &RuntimeFaultContributionReceipt) -> String {
    format!(
        concat!(
            "{{",
            "\"family\":{},",
            "\"authority\":{},",
            "\"active\":{},",
            "\"event_count\":{},",
            "\"detail\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", receipt.family)),
        json_string(&format!("{:?}", receipt.authority)),
        receipt.active,
        receipt.event_count,
        json_option_string(receipt.detail.as_deref()),
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_fault_diagnostic_receipt(receipt: &RuntimeFaultDiagnosticReceipt) -> String {
    let contributions = receipt
        .contributions
        .iter()
        .map(json_runtime_fault_contribution_receipt)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"primary_family\":{},",
            "\"primary_fault_cause\":{},",
            "\"interruption_class\":{},",
            "\"recovery_state\":{},",
            "\"safe_mode_enabled\":{},",
            "\"rebindable\":{},",
            "\"contributions\":[{}],",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(
            receipt
                .primary_family
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
        json_option_string(
            receipt
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref()
        ),
        json_string(&format!("{:?}", receipt.interruption_class)),
        json_string(&format!("{:?}", receipt.recovery_state)),
        receipt.safe_mode_enabled,
        receipt.rebindable,
        contributions,
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn format_runtime_offline_render_session_snapshot_compact(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    let active = snapshot
        .active_sessions
        .first()
        .map(|session| format!("{}:{:?}", session.request_id, session.state))
        .unwrap_or_else(|| "none".into());
    let last = snapshot
        .last_session
        .as_ref()
        .map(|session| format!("{}:{:?}", session.request_id, session.state))
        .unwrap_or_else(|| "none".into());
    let last_checkpoint = snapshot
        .last_session
        .as_ref()
        .and_then(|session| session.last_checkpoint.as_ref())
        .map(|checkpoint| format!("{:?}", checkpoint.stage))
        .unwrap_or_else(|| "none".into());
    format!(
        " offline_render_sessions={}/{}/{} active={} last={} last_checkpoint={} last_cancellation={} last_purge={}",
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        active,
        last,
        last_checkpoint,
        snapshot.last_cancellation.is_some(),
        snapshot.last_purge.is_some(),
    )
}

fn format_runtime_offline_render_session_snapshot_multiline(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    let active = snapshot
        .active_sessions
        .iter()
        .map(|session| session.summary.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        concat!(
            "\noffline_render_session_active_count={}",
            "\noffline_render_session_paused_count={}",
            "\noffline_render_session_recoverable_count={}",
            "\noffline_render_session_active_summaries={}",
            "\noffline_render_session_last_summary={}",
            "\noffline_render_session_last_cancellation={}",
            "\noffline_render_session_last_purge={}",
            "\noffline_render_session_summary={}",
        ),
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        if active.is_empty() {
            "none"
        } else {
            active.as_str()
        },
        snapshot
            .last_session
            .as_ref()
            .map(|session| session.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_cancellation
            .as_ref()
            .map(|receipt| receipt.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_purge
            .as_ref()
            .map(|receipt| receipt.summary.as_str())
            .unwrap_or("none"),
        snapshot.summary,
    )
}

fn json_runtime_recording_capture_checkpoint(
    checkpoint: &RuntimeRecordingCaptureCheckpointSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"capture_kind\":{},",
            "\"checkpoint_class\":{},",
            "\"interruption_class\":{},",
            "\"take_id\":{},",
            "\"track_id\":{},",
            "\"capture_start_samples\":{},",
            "\"capture_path\":{},",
            "\"buffered_block_count\":{},",
            "\"buffered_frame_count\":{},",
            "\"buffered_event_count\":{},",
            "\"captured_channel_count\":{},",
            "\"peak_level\":{},",
            "\"pressure_event_count\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(match checkpoint.capture_kind {
            RuntimeRecordingCaptureKind::Audio => "Audio",
            RuntimeRecordingCaptureKind::Midi => "Midi",
        })),
        json_option_string(Some(match checkpoint.checkpoint_class {
            RuntimeRecordingCaptureCheckpointClass::Armed => "Armed",
            RuntimeRecordingCaptureCheckpointClass::Streaming => "Streaming",
            RuntimeRecordingCaptureCheckpointClass::Buffered => "Buffered",
            RuntimeRecordingCaptureCheckpointClass::Committed => "Committed",
            RuntimeRecordingCaptureCheckpointClass::Failed => "Failed",
        })),
        json_option_string(Some(match checkpoint.interruption_class {
            RuntimeInterruptionClass::Steady => "Steady",
            RuntimeInterruptionClass::Resumable => "Resumable",
            RuntimeInterruptionClass::Restartable => "Restartable",
            RuntimeInterruptionClass::Recoverable => "Recoverable",
            RuntimeInterruptionClass::Terminal => "Terminal",
        })),
        json_option_string(Some(checkpoint.take_id.as_str())),
        json_option_string(Some(checkpoint.track_id.as_str())),
        checkpoint.capture_start_samples,
        json_option_string(Some(checkpoint.capture_path.as_str())),
        checkpoint.buffered_block_count,
        checkpoint.buffered_frame_count,
        checkpoint.buffered_event_count,
        checkpoint.captured_channel_count,
        json_option_f32(checkpoint.peak_level),
        checkpoint.pressure_event_count,
        json_option_string(checkpoint.last_error.as_deref()),
        json_option_string(Some(checkpoint.summary.as_str())),
    )
}

fn json_runtime_recording_capture_snapshot(snapshot: &RuntimeRecordingCaptureSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"capture_ready\":{},",
            "\"state\":{},",
            "\"capture_kind\":{},",
            "\"active_take_id\":{},",
            "\"active_track_id\":{},",
            "\"capture_start_samples\":{},",
            "\"active_capture_path\":{},",
            "\"buffered_block_count\":{},",
            "\"buffered_frame_count\":{},",
            "\"buffered_event_count\":{},",
            "\"captured_channel_count\":{},",
            "\"peak_level\":{},",
            "\"pressure_event_count\":{},",
            "\"active_checkpoint\":{},",
            "\"last_checkpoint\":{},",
            "\"last_committed_take_id\":{},",
            "\"last_committed_path\":{},",
            "\"last_committed_duration_samples\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.capture_ready,
        json_option_string(snapshot.state.map(|state| match state {
            RuntimeRecordingCaptureState::Idle => "Idle",
            RuntimeRecordingCaptureState::Capturing => "Capturing",
            RuntimeRecordingCaptureState::Failed => "Failed",
        })),
        json_option_string(snapshot.capture_kind.map(|kind| match kind {
            RuntimeRecordingCaptureKind::Audio => "Audio",
            RuntimeRecordingCaptureKind::Midi => "Midi",
        })),
        json_option_string(snapshot.active_take_id.as_deref()),
        json_option_string(snapshot.active_track_id.as_deref()),
        json_option_i64(snapshot.capture_start_samples),
        json_option_string(snapshot.active_capture_path.as_deref()),
        snapshot.buffered_block_count,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.captured_channel_count,
        json_option_f32(snapshot.peak_level),
        snapshot.pressure_event_count,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(json_runtime_recording_capture_checkpoint)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(json_runtime_recording_capture_checkpoint)
            .unwrap_or_else(|| "null".into()),
        json_option_string(snapshot.last_committed_take_id.as_deref()),
        json_option_string(snapshot.last_committed_path.as_deref()),
        json_option_u32(snapshot.last_committed_duration_samples),
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_offline_render_checkpoint_receipt(
    checkpoint: &RuntimeOfflineRenderCheckpointReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"stage\":{},",
            "\"checkpoint_index\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"total_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"total_block_count\":{},",
            "\"progress_percent\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(checkpoint.request_id.as_str())),
        json_option_string(Some(match checkpoint.stage {
            RuntimeOfflineRenderCheckpointStage::PreparingInput => "PreparingInput",
            RuntimeOfflineRenderCheckpointStage::RenderingGraph => "RenderingGraph",
            RuntimeOfflineRenderCheckpointStage::MaterializingOutputs => "MaterializingOutputs",
            RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts => "FinalizingArtifacts",
        })),
        checkpoint.checkpoint_index,
        checkpoint.checkpoint_count,
        checkpoint.rendered_frame_count,
        checkpoint.total_frame_count,
        checkpoint.rendered_block_count,
        checkpoint.total_block_count,
        checkpoint.progress_percent,
        json_option_string(Some(checkpoint.summary.as_str())),
    )
}

fn json_runtime_offline_render_execution_cancellation_receipt(
    receipt: &RuntimeOfflineRenderExecutionCancellationReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"cancelled_after_checkpoint_count\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(receipt.request_id.as_str())),
        receipt.cancelled_after_checkpoint_count,
        receipt.checkpoint_count,
        receipt.rendered_frame_count,
        receipt.rendered_block_count,
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_offline_render_purge_receipt(receipt: &RuntimeOfflineRenderPurgeReceipt) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"orchestration\":{},",
            "\"artifact_root_path\":{},",
            "\"report_path\":{},",
            "\"purged_artifact_root\":{},",
            "\"purged_artifact_file_count\":{},",
            "\"purged_artifact_byte_count\":{},",
            "\"purged_report\":{},",
            "\"purged_report_byte_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(receipt.request_id.as_str())),
        receipt.orchestration.render_json(),
        json_option_string(receipt.artifact_root_path.as_deref()),
        json_option_string(receipt.report_path.as_deref()),
        receipt.purged_artifact_root,
        receipt.purged_artifact_file_count,
        receipt.purged_artifact_byte_count,
        receipt.purged_report,
        receipt.purged_report_byte_count,
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_offline_render_session_state_snapshot(
    snapshot: &RuntimeOfflineRenderSessionStateSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"state\":{},",
            "\"interruption_class\":{},",
            "\"interruption_rebindable\":{},",
            "\"interruption_count\":{},",
            "\"emitted_checkpoint_count\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"total_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"total_block_count\":{},",
            "\"artifact_root_path\":{},",
            "\"report_path\":{},",
            "\"materialized\":{},",
            "\"artifact_count\":{},",
            "\"report_materialized\":{},",
            "\"active_checkpoint\":{},",
            "\"last_checkpoint\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.request_id.as_str())),
        json_option_string(Some(match snapshot.state {
            RuntimeOfflineRenderExecutionState::Running => "Running",
            RuntimeOfflineRenderExecutionState::Paused => "Paused",
            RuntimeOfflineRenderExecutionState::Recoverable => "Recoverable",
            RuntimeOfflineRenderExecutionState::Completed => "Completed",
            RuntimeOfflineRenderExecutionState::Cancelled => "Cancelled",
            RuntimeOfflineRenderExecutionState::Failed => "Failed",
        })),
        json_option_string(Some(match snapshot.interruption_class {
            RuntimeInterruptionClass::Steady => "Steady",
            RuntimeInterruptionClass::Resumable => "Resumable",
            RuntimeInterruptionClass::Restartable => "Restartable",
            RuntimeInterruptionClass::Recoverable => "Recoverable",
            RuntimeInterruptionClass::Terminal => "Terminal",
        })),
        snapshot.interruption_rebindable,
        snapshot.interruption_count,
        snapshot.emitted_checkpoint_count,
        snapshot.checkpoint_count,
        snapshot.rendered_frame_count,
        snapshot.total_frame_count,
        snapshot.rendered_block_count,
        snapshot.total_block_count,
        json_option_string(snapshot.artifact_root_path.as_deref()),
        json_option_string(snapshot.report_path.as_deref()),
        snapshot.materialized,
        snapshot.artifact_count,
        snapshot.report_materialized,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(json_runtime_offline_render_checkpoint_receipt)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(json_runtime_offline_render_checkpoint_receipt)
            .unwrap_or_else(|| "null".into()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_offline_render_session_snapshot(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"active_session_count\":{},",
            "\"paused_session_count\":{},",
            "\"recoverable_session_count\":{},",
            "\"active_sessions\":{},",
            "\"last_session\":{},",
            "\"last_cancellation\":{},",
            "\"last_purge\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        format!(
            "[{}]",
            snapshot
                .active_sessions
                .iter()
                .map(json_runtime_offline_render_session_state_snapshot)
                .collect::<Vec<_>>()
                .join(",")
        ),
        snapshot
            .last_session
            .as_ref()
            .map(json_runtime_offline_render_session_state_snapshot)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_cancellation
            .as_ref()
            .map(json_runtime_offline_render_execution_cancellation_receipt)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_purge
            .as_ref()
            .map(json_runtime_offline_render_purge_receipt)
            .unwrap_or_else(|| "null".into()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_interruption_summary(summary: &RuntimeInterruptionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"active\":{},",
            "\"class\":{},",
            "\"rebindable\":{},",
            "\"recovery_state\":{},",
            "\"primary_fault_cause\":{},",
            "\"safe_mode_enabled\":{},",
            "\"deferred_service_class\":{},",
            "\"deferred_service_decision\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.active,
        json_string(&format!("{:?}", summary.class)),
        summary.rebindable,
        json_string(&format!("{:?}", summary.recovery_state)),
        json_option_string(
            summary
                .primary_fault_cause
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        summary.safe_mode_enabled,
        json_option_string(
            summary
                .deferred_service_class
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            summary
                .deferred_service_decision
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_meter_source_snapshot(snapshot: &RuntimeMeterSourceSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"bus_id\":{},",
            "\"topology_role\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"producer_node_ids\":{},",
            "\"peak_level\":{},",
            "\"rms_level\":{},",
            "\"latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.bus_id.as_str())),
        json_escape_string(&format!("{:?}", snapshot.topology_role)),
        json_option_string(snapshot.track_lane_id.as_deref()),
        json_option_string(snapshot.bus_group_id.as_deref()),
        json_option_string(snapshot.console_group_id.as_deref()),
        json_option_string(snapshot.send_return_id.as_deref()),
        json_string_vec(&snapshot.producer_node_ids),
        snapshot.peak_level,
        snapshot.rms_level,
        snapshot.latency_samples,
        snapshot.tail_samples,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_meter_source_snapshot_vec(snapshots: &[RuntimeMeterSourceSnapshot]) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_meter_source_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_tempo_map_segment_snapshot(snapshot: &RuntimeTempoMapSegmentSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"segment_id\":{},",
            "\"start_samples\":{},",
            "\"end_samples\":{},",
            "\"start_tempo_bpm\":{},",
            "\"end_tempo_bpm\":{},",
            "\"interpolation\":\"{:?}\",",
            "\"covers_timeline_position\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.segment_id.as_str())),
        snapshot.start_samples,
        json_option_i64(snapshot.end_samples),
        snapshot.start_tempo_bpm,
        json_option_f64(snapshot.end_tempo_bpm),
        snapshot.interpolation,
        snapshot.covers_timeline_position,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_tempo_map_segment_snapshot_vec(
    snapshots: &[RuntimeTempoMapSegmentSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_tempo_map_segment_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_tempo_map_snapshot(snapshot: &RuntimeTempoMapSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"segment_count\":{},",
            "\"active_segment_id\":{},",
            "\"active_segment_index\":{},",
            "\"next_segment_start_samples\":{},",
            "\"resolved_tempo_bpm\":{},",
            "\"tempo_source\":\"{:?}\",",
            "\"timeline_position_samples\":{},",
            "\"segments\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.segment_count,
        json_option_string(snapshot.active_segment_id.as_deref()),
        json_option_usize(snapshot.active_segment_index),
        json_option_i64(snapshot.next_segment_start_samples),
        snapshot.resolved_tempo_bpm,
        snapshot.tempo_source,
        json_option_i64(snapshot.timeline_position_samples),
        json_runtime_tempo_map_segment_snapshot_vec(&snapshot.segments),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_warp_clip_snapshot(snapshot: &RuntimeWarpClipSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"mode\":\"{:?}\",",
            "\"source_tempo_bpm\":{},",
            "\"project_tempo_bpm\":{},",
            "\"project_tempo_source\":\"{:?}\",",
            "\"project_tempo_segment_id\":{},",
            "\"realized_ratio\":{},",
            "\"anchor_timeline_samples\":{},",
            "\"start_samples\":{},",
            "\"duration_samples\":{},",
            "\"readiness\":\"{:?}\",",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.mode,
        json_option_f64(snapshot.source_tempo_bpm),
        snapshot.project_tempo_bpm,
        snapshot.project_tempo_source,
        json_option_string(snapshot.project_tempo_segment_id.as_deref()),
        snapshot.realized_ratio,
        snapshot.anchor_timeline_samples,
        snapshot.start_samples,
        snapshot.duration_samples,
        snapshot.readiness,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_warp_clip_snapshot_vec(snapshots: &[RuntimeWarpClipSnapshot]) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_warp_clip_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_warp_pipeline_snapshot(snapshot: &RuntimeWarpPipelineSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"degraded_clip_count\":{},",
            "\"bypassed_clip_count\":{},",
            "\"active_warp_count\":{},",
            "\"resolved_project_tempo_bpm\":{},",
            "\"resolved_project_tempo_source\":\"{:?}\",",
            "\"resolved_project_tempo_segment_id\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.active_warp_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        json_option_string(snapshot.resolved_project_tempo_segment_id.as_deref()),
        json_runtime_warp_clip_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_clip_processing_stage_vec(stages: &[RuntimeClipProcessingStage]) -> String {
    let joined = stages
        .iter()
        .map(|stage| json_escape_string(&format!("{stage:?}")))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_clip_processing_snapshot(snapshot: &RuntimeClipProcessingSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"warp_mode\":\"{:?}\",",
            "\"start_samples\":{},",
            "\"duration_samples\":{},",
            "\"fade_in\":{{\"duration_samples\":{},\"shape\":\"{:?}\"}},",
            "\"fade_out\":{{\"duration_samples\":{},\"shape\":\"{:?}\"}},",
            "\"fade_in_end_samples\":{},",
            "\"fade_out_start_samples\":{},",
            "\"clip_gain\":{{\"start_linear\":{},\"end_linear\":{},\"shape\":\"{:?}\"}},",
            "\"treatment_stages\":{},",
            "\"realized_warp_ratio\":{},",
            "\"project_tempo_source\":{},",
            "\"project_tempo_segment_id\":{},",
            "\"readiness\":\"{:?}\",",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.warp_mode,
        snapshot.start_samples,
        snapshot.duration_samples,
        snapshot.fade_in.duration_samples,
        snapshot.fade_in.shape,
        snapshot.fade_out.duration_samples,
        snapshot.fade_out.shape,
        snapshot.fade_in_end_samples,
        snapshot.fade_out_start_samples,
        snapshot.clip_gain.start_linear,
        snapshot.clip_gain.end_linear,
        snapshot.clip_gain.shape,
        json_runtime_clip_processing_stage_vec(&snapshot.treatment_stages),
        json_option_f64(snapshot.realized_warp_ratio),
        json_option_string(
            snapshot
                .project_tempo_source
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(snapshot.project_tempo_segment_id.as_deref()),
        snapshot.readiness,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_clip_processing_snapshot_vec(
    snapshots: &[RuntimeClipProcessingSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_clip_processing_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_clip_processing_pipeline_snapshot(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"pending_media_clip_count\":{},",
            "\"pending_warp_clip_count\":{},",
            "\"invalid_clip_count\":{},",
            "\"faded_clip_count\":{},",
            "\"gain_shaped_clip_count\":{},",
            "\"warped_clip_count\":{},",
            "\"treatment_stage_count\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
        json_runtime_clip_processing_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_recall_snapshot(snapshot: &RuntimePluginRecallSnapshot) -> String {
    let lifecycle_state = snapshot
        .payload
        .lifecycle_state
        .map(|state| format!("{state:?}"));
    let lifecycle_stage = snapshot
        .payload
        .lifecycle_stage
        .map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot
        .payload
        .transport_stage
        .map(|stage| format!("{stage:?}"));
    let last_restart_intent = snapshot
        .payload
        .last_restart_intent
        .map(|intent| format!("{intent:?}"));
    let last_stop_reason = snapshot
        .payload
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    let last_fault_kind = snapshot
        .payload
        .last_fault_kind
        .map(|kind| format!("{kind:?}"));
    let plugin_format = snapshot
        .payload
        .plugin_format
        .map(|format| format!("{format:?}"));
    format!(
        concat!(
            "{{",
            "\"state\":\"{:?}\",",
            "\"payload\":{{",
            "\"sandbox_id\":{},",
            "\"plugin_type_id\":{},",
            "\"plugin_format\":{},",
            "\"lifecycle_state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"readiness_state\":{},",
            "\"recovery_count\":{},",
            "\"restart_count\":{},",
            "\"fault_count\":{},",
            "\"last_restart_intent\":{},",
            "\"last_stop_reason\":{},",
            "\"last_fault_kind\":{},",
            "\"last_fault_detail\":{},",
            "\"degraded_reasons\":{}",
            "}},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.state,
        json_option_string(snapshot.payload.sandbox_id.as_deref()),
        json_option_string(snapshot.payload.plugin_type_id.as_deref()),
        json_option_string(plugin_format.as_deref()),
        json_option_string(lifecycle_state.as_deref()),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        json_option_string(snapshot.payload.readiness_state.as_deref()),
        snapshot.payload.recovery_count,
        snapshot.payload.restart_count,
        snapshot.payload.fault_count,
        json_option_string(last_restart_intent.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_string(last_fault_kind.as_deref()),
        json_option_string(snapshot.payload.last_fault_detail.as_deref()),
        json_string_vec(&snapshot.payload.degraded_reasons),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_discovery_snapshot(snapshot: &RuntimePluginDiscoverySnapshot) -> String {
    let last_scan = snapshot
        .last_scan
        .as_ref()
        .map(json_runtime_plugin_scan_receipt)
        .unwrap_or_else(|| "null".into());
    format!(
        concat!(
            "{{",
            "\"scan_count\":{},",
            "\"format_filtered_scan_count\":{},",
            "\"discovered_type_count\":{},",
            "\"discovered_format_count\":{},",
            "\"last_scan\":{},",
            "\"format_coverage\":{},",
            "\"capability_coverage\":{},",
            "\"discovered_types\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        last_scan,
        json_runtime_plugin_format_coverage_vec(&snapshot.format_coverage),
        json_runtime_plugin_capability_coverage_summary(&snapshot.capability_coverage),
        json_runtime_plugin_discovered_type_record_vec(&snapshot.discovered_types),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_scan_receipt(receipt: &RuntimePluginScanReceipt) -> String {
    format!(
        concat!(
            "{{",
            "\"scan_handle\":{},",
            "\"roots\":{},",
            "\"formats\":{},",
            "\"targeted_format_count\":{},",
            "\"discovered_type_count\":{},",
            "\"discovered_format_count\":{},",
            "\"format_coverage\":{},",
            "\"capability_coverage\":{},",
            "\"summary\":{}",
            "}}"
        ),
        receipt.scan_handle.0,
        json_string_vec(&receipt.roots),
        json_plugin_format_vec(&receipt.formats),
        receipt.targeted_format_count,
        receipt.discovered_type_count,
        receipt.discovered_format_count,
        json_runtime_plugin_format_coverage_vec(&receipt.format_coverage),
        json_runtime_plugin_capability_coverage_summary(&receipt.capability_coverage),
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_plugin_format_coverage_vec(
    records: &[RuntimePluginFormatCoverageRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(json_runtime_plugin_format_coverage_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_format_coverage_record(
    record: &RuntimePluginFormatCoverageRecord,
) -> String {
    format!(
        concat!(
            "{{",
            "\"format\":{},",
            "\"discovered_type_count\":{},",
            "\"instrument_count\":{},",
            "\"audio_effect_count\":{},",
            "\"analyzer_count\":{},",
            "\"utility_count\":{},",
            "\"note_effect_count\":{},",
            "\"supports_snapshot_count\":{},",
            "\"supports_prepare_count\":{},",
            "\"supports_activate_count\":{},",
            "\"accepts_midi_count\":{},",
            "\"produces_midi_count\":{},",
            "\"max_audio_bus_count\":{},",
            "\"max_parameter_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", record.format)),
        record.discovered_type_count,
        record.instrument_count,
        record.audio_effect_count,
        record.analyzer_count,
        record.utility_count,
        record.note_effect_count,
        record.supports_snapshot_count,
        record.supports_prepare_count,
        record.supports_activate_count,
        record.accepts_midi_count,
        record.produces_midi_count,
        record.max_audio_bus_count,
        record.max_parameter_count,
        json_option_string(Some(record.summary.as_str())),
    )
}

fn json_runtime_plugin_capability_coverage_summary(
    summary: &RuntimePluginCapabilityCoverageSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"discovered_format_count\":{},",
            "\"multi_format_catalog\":{},",
            "\"instrument_count\":{},",
            "\"audio_effect_count\":{},",
            "\"analyzer_count\":{},",
            "\"utility_count\":{},",
            "\"note_effect_count\":{},",
            "\"supports_snapshot_count\":{},",
            "\"supports_reset_count\":{},",
            "\"supports_bypass_count\":{},",
            "\"exposes_latency_count\":{},",
            "\"exposes_tail_count\":{},",
            "\"sample_accurate_automation_count\":{},",
            "\"accepts_midi_count\":{},",
            "\"accepts_note_events_count\":{},",
            "\"produces_midi_count\":{},",
            "\"silence_aware_count\":{},",
            "\"requires_main_thread_for_state_count\":{},",
            "\"supports_prepare_count\":{},",
            "\"supports_activate_count\":{},",
            "\"supports_reset_while_active_count\":{},",
            "\"max_audio_bus_count\":{},",
            "\"max_parameter_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.discovered_format_count,
        summary.multi_format_catalog,
        summary.instrument_count,
        summary.audio_effect_count,
        summary.analyzer_count,
        summary.utility_count,
        summary.note_effect_count,
        summary.supports_snapshot_count,
        summary.supports_reset_count,
        summary.supports_bypass_count,
        summary.exposes_latency_count,
        summary.exposes_tail_count,
        summary.sample_accurate_automation_count,
        summary.accepts_midi_count,
        summary.accepts_note_events_count,
        summary.produces_midi_count,
        summary.silence_aware_count,
        summary.requires_main_thread_for_state_count,
        summary.supports_prepare_count,
        summary.supports_activate_count,
        summary.supports_reset_while_active_count,
        summary.max_audio_bus_count,
        summary.max_parameter_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_plugin_discovered_type_record_vec(
    records: &[RuntimePluginDiscoveredTypeRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(json_runtime_plugin_discovered_type_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_discovered_type_record(
    record: &RuntimePluginDiscoveredTypeRecord,
) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_id\":{},",
            "\"plugin_id\":{},",
            "\"vendor\":{},",
            "\"name\":{},",
            "\"format\":{},",
            "\"version\":{},",
            "\"features\":{},",
            "\"default_io_layout\":{},",
            "\"audio_bus_count\":{},",
            "\"parameter_count\":{},",
            "\"state_contract\":{},",
            "\"processing_contract\":{},",
            "\"lifecycle_contract\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(record.plugin_type_id.as_str())),
        json_option_string(Some(record.plugin_id.as_str())),
        json_option_string(Some(record.vendor.as_str())),
        json_option_string(Some(record.name.as_str())),
        json_option_string(Some(&format!("{:?}", record.format))),
        json_option_string(record.version.as_deref()),
        json_plugin_feature_vec(&record.features),
        json_plugin_io_layout(record.default_io_layout),
        record.audio_bus_count,
        record.parameter_count,
        json_plugin_state_contract(record.state_contract),
        json_plugin_processing_contract(record.processing_contract),
        json_plugin_lifecycle_contract(record.lifecycle_contract),
        json_option_string(Some(record.summary.as_str())),
    )
}

fn json_runtime_plugin_chain_stage_snapshot(snapshot: &RuntimePluginChainStageSnapshot) -> String {
    let lifecycle_state = snapshot.lifecycle_state.map(|state| format!("{state:?}"));
    let lifecycle_stage = snapshot.lifecycle_stage.map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot.transport_stage.map(|stage| format!("{stage:?}"));
    format!(
        concat!(
            "{{",
            "\"node_id\":{},",
            "\"stage_index\":{},",
            "\"sandbox_id\":{},",
            "\"sandbox_group_key\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"placement_outcome\":\"{:?}\",",
            "\"placement_rule_id\":{},",
            "\"shared_boundary_member_count\":{},",
            "\"continuity_class\":\"{:?}\",",
            "\"rebindable\":{},",
            "\"lifecycle_state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"recall_state\":\"{:?}\",",
            "\"recall\":{},",
            "\"compensation_state\":\"{:?}\",",
            "\"planned_latency_samples\":{},",
            "\"realized_latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"bypassed\":{},",
            "\"active_transport\":{},",
            "\"degraded_reasons\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.node_id.as_str())),
        snapshot.stage_index,
        json_option_string(snapshot.sandbox_id.as_deref()),
        json_option_string(snapshot.sandbox_group_key.as_deref()),
        json_option_string(snapshot.track_lane_id.as_deref()),
        json_option_string(snapshot.bus_group_id.as_deref()),
        json_option_string(snapshot.console_group_id.as_deref()),
        json_option_string(snapshot.send_return_id.as_deref()),
        snapshot.placement_outcome,
        json_option_string(snapshot.placement_rule_id.as_deref()),
        snapshot.shared_boundary_member_count,
        snapshot.continuity_class,
        snapshot.rebindable,
        json_option_string(lifecycle_state.as_deref()),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        snapshot.recall_state,
        json_runtime_plugin_recall_snapshot(&snapshot.recall),
        snapshot.compensation_state,
        snapshot.planned_latency_samples,
        json_option_u32(snapshot.realized_latency_samples),
        json_option_u32(snapshot.tail_samples),
        snapshot.bypassed,
        snapshot.active_transport,
        json_string_vec(&snapshot.degraded_reasons),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_chain_stage_snapshot_vec(
    snapshots: &[RuntimePluginChainStageSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_plugin_chain_stage_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_plugin_execution_chain_summary(
    summary: &RuntimePluginExecutionChainSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_id\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"stage_count\":{},",
            "\"shared_sandbox_stage_count\":{},",
            "\"isolated_sandbox_stage_count\":{},",
            "\"in_process_stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"rebindable_stage_count\":{},",
            "\"terminal_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"stages\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(summary.chain_id.as_str())),
        json_option_string(summary.track_lane_id.as_deref()),
        json_option_string(summary.bus_group_id.as_deref()),
        json_option_string(summary.console_group_id.as_deref()),
        json_option_string(summary.send_return_id.as_deref()),
        summary.stage_count,
        summary.shared_sandbox_stage_count,
        summary.isolated_sandbox_stage_count,
        summary.in_process_stage_count,
        summary.pending_render_stage_count,
        summary.settling_stage_count,
        summary.compensated_stage_count,
        summary.degraded_stage_count,
        summary.bypassed_stage_count,
        summary.missing_binding_stage_count,
        summary.rebindable_stage_count,
        summary.terminal_stage_count,
        summary.total_planned_latency_samples,
        summary.total_realized_latency_samples,
        summary.total_tail_samples,
        json_runtime_plugin_chain_stage_snapshot_vec(&summary.stages),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_plugin_execution_chain_summary_vec(
    summaries: &[RuntimePluginExecutionChainSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(json_runtime_plugin_execution_chain_summary)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_plugin_chain_snapshot(snapshot: &RuntimePluginChainSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_count\":{},",
            "\"stage_count\":{},",
            "\"shared_sandbox_stage_count\":{},",
            "\"isolated_sandbox_stage_count\":{},",
            "\"in_process_stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"rebindable_stage_count\":{},",
            "\"terminal_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"chains\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.chain_count,
        snapshot.stage_count,
        snapshot.shared_sandbox_stage_count,
        snapshot.isolated_sandbox_stage_count,
        snapshot.in_process_stage_count,
        snapshot.pending_render_stage_count,
        snapshot.settling_stage_count,
        snapshot.compensated_stage_count,
        snapshot.degraded_stage_count,
        snapshot.bypassed_stage_count,
        snapshot.missing_binding_stage_count,
        snapshot.rebindable_stage_count,
        snapshot.terminal_stage_count,
        snapshot.total_planned_latency_samples,
        snapshot.total_realized_latency_samples,
        snapshot.total_tail_samples,
        json_runtime_plugin_execution_chain_summary_vec(&snapshot.chains),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_sandbox_snapshot(snapshot: &RuntimePluginSandboxSnapshot) -> String {
    let plugin_format = snapshot.plugin_format.map(|format| format!("{format:?}"));
    let state = format!("{:?}", snapshot.state);
    let continuity_class = format!("{:?}", snapshot.continuity_class);
    let lifecycle_stage = snapshot.lifecycle_stage.map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot.transport_stage.map(|stage| format!("{stage:?}"));
    let last_fault_kind = snapshot.last_fault_kind.map(|kind| format!("{kind:?}"));
    let last_restart_intent = snapshot
        .last_restart_intent
        .map(|intent| format!("{intent:?}"));
    let last_stop_reason = snapshot
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"sandbox_group_key\":{},",
            "\"plugin_type_id\":{},",
            "\"plugin_format\":{},",
            "\"instance_id\":{},",
            "\"placement_outcome\":\"{:?}\",",
            "\"placement_rule_id\":{},",
            "\"shared_boundary_member_count\":{},",
            "\"continuity_class\":{},",
            "\"rebindable\":{},",
            "\"state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"active\":{},",
            "\"active_transport\":{},",
            "\"restart_count\":{},",
            "\"recovery_count\":{},",
            "\"fault_count\":{},",
            "\"last_fault_kind\":{},",
            "\"last_fault_detail\":{},",
            "\"last_restart_intent\":{},",
            "\"last_stop_reason\":{},",
            "\"last_processing_epoch\":{},",
            "\"readiness_state\":{},",
            "\"degraded_reasons\":{},",
            "\"active_lease_id\":{},",
            "\"active_region_id\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.sandbox_id.as_str())),
        json_option_string(Some(snapshot.sandbox_group_key.as_str())),
        json_option_string(snapshot.plugin_type_id.as_deref()),
        json_option_string(plugin_format.as_deref()),
        json_option_string(snapshot.instance_id.as_deref()),
        snapshot.placement_outcome,
        json_option_string(snapshot.placement_rule_id.as_deref()),
        snapshot.shared_boundary_member_count,
        json_option_string(Some(continuity_class.as_str())),
        snapshot.rebindable,
        json_option_string(Some(state.as_str())),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        snapshot.active,
        snapshot.active_transport,
        snapshot.restart_count,
        snapshot.recovery_count,
        snapshot.fault_count,
        json_option_string(last_fault_kind.as_deref()),
        json_option_string(snapshot.last_fault_detail.as_deref()),
        json_option_string(last_restart_intent.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_u64(snapshot.last_processing_epoch),
        json_option_string(snapshot.readiness_state.as_deref()),
        json_string_vec(&snapshot.degraded_reasons),
        json_option_string(snapshot.active_lease_id.as_deref()),
        json_option_string(snapshot.active_region_id.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_sandbox_snapshot_vec(sandboxes: &[RuntimePluginSandboxSnapshot]) -> String {
    format!(
        "[{}]",
        sandboxes
            .iter()
            .map(json_runtime_plugin_sandbox_snapshot)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_lifecycle_snapshot(snapshot: &RuntimePluginLifecycleSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_count\":{},",
            "\"active_sandbox_count\":{},",
            "\"shared_sandbox_count\":{},",
            "\"isolated_sandbox_count\":{},",
            "\"ready_sandbox_count\":{},",
            "\"booting_sandbox_count\":{},",
            "\"degraded_sandbox_count\":{},",
            "\"faulted_sandbox_count\":{},",
            "\"restarting_sandbox_count\":{},",
            "\"quarantined_sandbox_count\":{},",
            "\"stopped_sandbox_count\":{},",
            "\"rebindable_sandbox_count\":{},",
            "\"terminal_sandbox_count\":{},",
            "\"sandboxes\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.sandbox_count,
        snapshot.active_sandbox_count,
        snapshot.shared_sandbox_count,
        snapshot.isolated_sandbox_count,
        snapshot.ready_sandbox_count,
        snapshot.booting_sandbox_count,
        snapshot.degraded_sandbox_count,
        snapshot.faulted_sandbox_count,
        snapshot.restarting_sandbox_count,
        snapshot.quarantined_sandbox_count,
        snapshot.stopped_sandbox_count,
        snapshot.rebindable_sandbox_count,
        snapshot.terminal_sandbox_count,
        json_runtime_plugin_sandbox_snapshot_vec(&snapshot.sandboxes),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_routed_plugin_chain_summary(summary: &RuntimeRoutedPluginChainSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"chain_count\":{},",
            "\"stage_count\":{},",
            "\"pending_render_stage_count\":{},",
            "\"settling_stage_count\":{},",
            "\"compensated_stage_count\":{},",
            "\"degraded_stage_count\":{},",
            "\"bypassed_stage_count\":{},",
            "\"missing_binding_stage_count\":{},",
            "\"total_planned_latency_samples\":{},",
            "\"total_realized_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"chain_ids\":{},",
            "\"node_ids\":{},",
            "\"sandbox_ids\":{}",
            "}}"
        ),
        summary.chain_count,
        summary.stage_count,
        summary.pending_render_stage_count,
        summary.settling_stage_count,
        summary.compensated_stage_count,
        summary.degraded_stage_count,
        summary.bypassed_stage_count,
        summary.missing_binding_stage_count,
        summary.total_planned_latency_samples,
        summary.total_realized_latency_samples,
        summary.total_tail_samples,
        json_string_vec(&summary.chain_ids),
        json_string_vec(&summary.node_ids),
        json_string_vec(&summary.sandbox_ids),
    )
}

fn json_runtime_routed_meter_aggregate(aggregate: &RuntimeRoutedMeterAggregate) -> String {
    format!(
        concat!(
            "{{",
            "\"meter_count\":{},",
            "\"metered_bus_ids\":{},",
            "\"producer_node_ids\":{},",
            "\"peak_level\":{},",
            "\"rms_level\":{},",
            "\"latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"summary\":{}",
            "}}"
        ),
        aggregate.meter_count,
        json_string_vec(&aggregate.metered_bus_ids),
        json_string_vec(&aggregate.producer_node_ids),
        json_option_f32(aggregate.peak_level),
        json_option_f32(aggregate.rms_level),
        aggregate.latency_samples,
        aggregate.tail_samples,
        json_option_string(Some(aggregate.summary.as_str())),
    )
}

fn json_runtime_track_lane_meter_summary_vec(summaries: &[RuntimeTrackLaneMeterSummary]) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"track_lane_id\":{},",
                    "\"bus_group_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.track_lane_id.as_str())),
                json_string_vec(&summary.bus_group_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_bus_group_meter_summary_vec(summaries: &[RuntimeBusGroupMeterSummary]) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"bus_group_id\":{},",
                    "\"topology_roles\":{},",
                    "\"node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.bus_group_id.as_str())),
                json_runtime_topology_role_vec(&summary.topology_roles),
                json_string_vec(&summary.node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_console_group_meter_summary_vec(
    summaries: &[RuntimeConsoleGroupMeterSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"console_group_id\":{},",
                    "\"node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.console_group_id.as_str())),
                json_string_vec(&summary.node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_send_return_meter_summary_vec(
    summaries: &[RuntimeSendReturnMeterSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"send_return_id\":{},",
                    "\"send_node_ids\":{},",
                    "\"return_node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.send_return_id.as_str())),
                json_string_vec(&summary.send_node_ids),
                json_string_vec(&summary.return_node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_metering_snapshot(snapshot: &RuntimeMeteringSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"meter_count\":{},",
            "\"main_output_peak_level\":{},",
            "\"main_output_rms_level\":{},",
            "\"momentary_loudness_lufs\":{},",
            "\"short_term_loudness_lufs\":{},",
            "\"integrated_loudness_lufs\":{},",
            "\"clipped_sample_count\":{},",
            "\"meters\":{},",
            "\"track_lanes\":{},",
            "\"bus_groups\":{},",
            "\"console_groups\":{},",
            "\"send_returns\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.meter_count,
        json_option_f32(snapshot.main_output_peak_level),
        json_option_f32(snapshot.main_output_rms_level),
        json_option_f32(snapshot.momentary_loudness_lufs),
        json_option_f32(snapshot.short_term_loudness_lufs),
        json_option_f32(snapshot.integrated_loudness_lufs),
        snapshot.clipped_sample_count,
        json_runtime_meter_source_snapshot_vec(&snapshot.meters),
        json_runtime_track_lane_meter_summary_vec(&snapshot.track_lanes),
        json_runtime_bus_group_meter_summary_vec(&snapshot.bus_groups),
        json_runtime_console_group_meter_summary_vec(&snapshot.console_groups),
        json_runtime_send_return_meter_summary_vec(&snapshot.send_returns),
        json_option_string(Some(snapshot.summary.as_str())),
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
            "\"send_return_group_count\":{},",
            "\"console_group_count\":{},",
            "\"plugin_chain\":{},",
            "\"lanes\":{},",
            "\"track_lanes\":{},",
            "\"bus_groups\":{},",
            "\"console_groups\":{},",
            "\"send_returns\":{},",
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
        summary.send_return_group_count,
        summary.console_group_count,
        json_runtime_routed_plugin_chain_summary(&summary.plugin_chain),
        json_runtime_execution_topology_lanes(&summary.lanes),
        json_runtime_mixer_track_lanes(&summary.track_lanes),
        json_runtime_mixer_bus_groups(&summary.bus_groups),
        json_runtime_mixer_console_groups(&summary.console_groups),
        json_runtime_mixer_send_returns(&summary.send_returns),
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
        RuntimeSchedulerTopologyIssue::MissingSendReturnIds { node_count } => format!(
            "{{\"kind\":\"MissingSendReturnIds\",\"node_count\":{}}}",
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
            "\"lane_count\":{},",
            "\"point_count\":{},",
            "\"projected_segment_count\":{},",
            "\"mapped_lane_count\":{},",
            "\"unmapped_lane_count\":{},",
            "\"hold_lane_count\":{},",
            "\"linear_lane_count\":{},",
            "\"last_batch_epoch\":{},",
            "\"last_batch_event_count\":{},",
            "\"last_batch_ignored_event_count\":{},",
            "\"last_batch_sub_block_count\":{},",
            "\"last_batch_coalesced_event_count\":{},",
            "\"last_batch_strategy_max_sub_blocks\":{},",
            "\"last_batch_min_ramp_step_samples\":{},",
            "\"last_batch_max_sample_offset\":{},",
            "\"last_block_sequence\":{},",
            "\"last_timeline_position_samples\":{},",
            "\"transport_playing\":{},",
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
        snapshot.lane_count,
        snapshot.point_count,
        snapshot.projected_segment_count,
        snapshot.mapped_lane_count,
        snapshot.unmapped_lane_count,
        snapshot.hold_lane_count,
        snapshot.linear_lane_count,
        json_option_u64(snapshot.last_batch_epoch),
        snapshot.last_batch_event_count,
        snapshot.last_batch_ignored_event_count,
        snapshot.last_batch_sub_block_count,
        snapshot.last_batch_coalesced_event_count,
        snapshot.last_batch_strategy_max_sub_blocks,
        json_option_usize(snapshot.last_batch_min_ramp_step_samples),
        json_option_usize(snapshot.last_batch_max_sample_offset),
        json_option_u64(snapshot.last_block_sequence),
        json_option_i64(snapshot.last_timeline_position_samples),
        match snapshot.transport_playing {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
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
            "\"last_block_execution_time_ns\":{},",
            "\"last_block_deadline_budget_ns\":{},",
            "\"last_block_budget_utilization_percent\":{},",
            "\"last_block_budget_overrun_ns\":{},",
            "\"last_block_deadline_pressure\":\"{:?}\",",
            "\"budget_overrun_count\":{},",
            "\"peak_block_execution_time_ns\":{},",
            "\"peak_block_budget_utilization_percent\":{},",
            "\"peak_block_budget_overrun_ns\":{},",
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
        json_option_u64(snapshot.last_block_execution_time_ns),
        json_option_u64(snapshot.last_block_deadline_budget_ns),
        json_option_f32(snapshot.last_block_budget_utilization_percent),
        json_option_u64(snapshot.last_block_budget_overrun_ns),
        snapshot.last_block_deadline_pressure,
        snapshot.budget_overrun_count,
        snapshot.peak_block_execution_time_ns,
        snapshot.peak_block_budget_utilization_percent,
        snapshot.peak_block_budget_overrun_ns,
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
                        "\"track_lane_id\":{},",
                        "\"bus_group_id\":{},",
                        "\"console_group_id\":{},",
                        "\"send_return_id\":{},",
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
                    json_option_string(node.track_lane_id.as_deref()),
                    json_option_string(node.bus_group_id.as_deref()),
                    json_option_string(node.console_group_id.as_deref()),
                    json_option_string(node.send_return_id.as_deref()),
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
                        "\"bus_group_ids\":{},",
                        "\"console_group_ids\":{},",
                        "\"send_return_ids\":{}",
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
                    json_string_vec(&lane.console_group_ids),
                    json_string_vec(&lane.send_return_ids),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_mixer_track_lanes(track_lanes: &[RuntimeMixerTrackLaneSummary]) -> String {
    format!(
        "[{}]",
        track_lanes
            .iter()
            .map(|track_lane| {
                format!(
                    concat!(
                        "{{",
                        "\"track_lane_id\":{},",
                        "\"node_ids\":{},",
                        "\"bus_group_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(track_lane.track_lane_id.as_str())),
                    json_string_vec(&track_lane.node_ids),
                    json_string_vec(&track_lane.bus_group_ids),
                    json_string_vec(&track_lane.input_bus_ids),
                    json_string_vec(&track_lane.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&track_lane.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_mixer_bus_groups(bus_groups: &[RuntimeMixerBusGroupSummary]) -> String {
    format!(
        "[{}]",
        bus_groups
            .iter()
            .map(|bus_group| {
                format!(
                    concat!(
                        "{{",
                        "\"bus_group_id\":{},",
                        "\"topology_roles\":{},",
                        "\"node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(bus_group.bus_group_id.as_str())),
                    json_runtime_topology_role_vec(&bus_group.topology_roles),
                    json_string_vec(&bus_group.node_ids),
                    json_string_vec(&bus_group.input_bus_ids),
                    json_string_vec(&bus_group.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&bus_group.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_mixer_console_groups(console_groups: &[RuntimeMixerConsoleGroupSummary]) -> String {
    format!(
        "[{}]",
        console_groups
            .iter()
            .map(|console_group| {
                format!(
                    concat!(
                        "{{",
                        "\"console_group_id\":{},",
                        "\"node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(console_group.console_group_id.as_str())),
                    json_string_vec(&console_group.node_ids),
                    json_string_vec(&console_group.input_bus_ids),
                    json_string_vec(&console_group.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&console_group.plugin_chain),
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_mixer_send_returns(send_returns: &[RuntimeMixerSendReturnSummary]) -> String {
    format!(
        "[{}]",
        send_returns
            .iter()
            .map(|send_return| {
                format!(
                    concat!(
                        "{{",
                        "\"send_return_id\":{},",
                        "\"send_node_ids\":{},",
                        "\"return_node_ids\":{},",
                        "\"input_bus_ids\":{},",
                        "\"output_bus_ids\":{},",
                        "\"plugin_chain\":{}",
                        "}}"
                    ),
                    json_option_string(Some(send_return.send_return_id.as_str())),
                    json_string_vec(&send_return.send_node_ids),
                    json_string_vec(&send_return.return_node_ids),
                    json_string_vec(&send_return.input_bus_ids),
                    json_string_vec(&send_return.output_bus_ids),
                    json_runtime_routed_plugin_chain_summary(&send_return.plugin_chain),
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
                        "\"track_lane_id\":{},",
                        "\"bus_group_id\":{},",
                        "\"console_group_id\":{},",
                        "\"send_return_id\":{},",
                        "\"input_bus_id\":{},",
                        "\"output_bus_id\":{},",
                        "\"plugin_sandbox_id\":{},",
                        "\"plugin_recall_state\":{},",
                        "\"plugin_recall\":{},",
                        "\"plugin_compensation_state\":{},",
                        "\"plugin_realized_latency_samples\":{},",
                        "\"plugin_tail_samples\":{}",
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
                    json_option_string(node.track_lane_id.as_deref()),
                    json_option_string(node.bus_group_id.as_deref()),
                    json_option_string(node.console_group_id.as_deref()),
                    json_option_string(node.send_return_id.as_deref()),
                    json_option_string(Some(node.input_bus_id.as_str())),
                    json_option_string(Some(node.output_bus_id.as_str())),
                    json_option_string(node.plugin_sandbox_id.as_deref()),
                    json_option_string(
                        node.plugin_recall_state
                            .map(|state| format!("{state:?}"))
                            .as_deref(),
                    ),
                    node.plugin_recall
                        .as_ref()
                        .map_or_else(|| "null".into(), json_runtime_plugin_recall_snapshot,),
                    json_option_string(
                        node.plugin_compensation_state
                            .map(|state| format!("{state:?}"))
                            .as_deref(),
                    ),
                    json_option_u32(node.plugin_realized_latency_samples),
                    json_option_u32(node.plugin_tail_samples),
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

fn json_plugin_format_vec(values: &[PluginFormat]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_plugin_feature_vec(values: &[PluginFeature]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_plugin_io_layout(layout: PluginIoLayout) -> String {
    format!(
        concat!(
            "{{",
            "\"audio_inputs\":{},",
            "\"audio_outputs\":{},",
            "\"midi_inputs\":{},",
            "\"midi_outputs\":{}",
            "}}"
        ),
        layout.audio_inputs, layout.audio_outputs, layout.midi_inputs, layout.midi_outputs,
    )
}

fn json_plugin_state_contract(contract: PluginStateContract) -> String {
    format!(
        concat!(
            "{{",
            "\"supports_snapshot\":{},",
            "\"supports_reset\":{},",
            "\"supports_bypass\":{},",
            "\"exposes_latency\":{},",
            "\"exposes_tail\":{}",
            "}}"
        ),
        contract.supports_snapshot,
        contract.supports_reset,
        contract.supports_bypass,
        contract.exposes_latency,
        contract.exposes_tail,
    )
}

fn json_plugin_processing_contract(contract: PluginProcessingContract) -> String {
    format!(
        concat!(
            "{{",
            "\"max_block_frames\":{},",
            "\"sample_accurate_automation\":{},",
            "\"accepts_midi\":{},",
            "\"accepts_note_events\":{},",
            "\"produces_midi\":{},",
            "\"silence_aware\":{}",
            "}}"
        ),
        contract.max_block_frames,
        contract.sample_accurate_automation,
        contract.accepts_midi,
        contract.accepts_note_events,
        contract.produces_midi,
        contract.silence_aware,
    )
}

fn json_plugin_lifecycle_contract(contract: PluginLifecycleContract) -> String {
    format!(
        concat!(
            "{{",
            "\"requires_main_thread_for_state\":{},",
            "\"supports_prepare\":{},",
            "\"supports_activate\":{},",
            "\"supports_reset_while_active\":{}",
            "}}"
        ),
        contract.requires_main_thread_for_state,
        contract.supports_prepare,
        contract.supports_activate,
        contract.supports_reset_while_active,
    )
}

fn json_string(value: &str) -> String {
    json_option_string(Some(value))
}

fn json_runtime_execution_lane_order(lanes: &[GraphExecutionLane]) -> String {
    format!(
        "[{}]",
        lanes
            .iter()
            .map(|lane| json_option_string(Some(runtime_execution_lane_name(*lane))))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_worker_lane_instrumentation_summaries(
    summaries: &[RuntimeWorkerLaneInstrumentationSummary],
) -> String {
    format!(
        "[{}]",
        summaries
            .iter()
            .map(|summary| {
                format!(
                    concat!(
                        "{{",
                        "\"lane\":{},",
                        "\"node_count\":{},",
                        "\"plugin_backed_node_count\":{},",
                        "\"planning_group_count\":{},",
                        "\"total_latency_samples\":{},",
                        "\"max_node_latency_samples\":{}",
                        "}}"
                    ),
                    json_string(runtime_execution_lane_name(summary.lane)),
                    summary.node_count,
                    summary.plugin_backed_node_count,
                    summary.planning_group_count,
                    summary.total_latency_samples,
                    summary.max_node_latency_samples,
                )
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
    pub formats: Vec<PluginFormat>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScanHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginScanReceipt {
    pub scan_handle: ScanHandle,
    pub roots: Vec<String>,
    pub formats: Vec<PluginFormat>,
    pub targeted_format_count: usize,
    pub discovered_type_count: usize,
    pub discovered_format_count: usize,
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDiscoveredTypeRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: PluginFormat,
    pub version: Option<String>,
    pub features: Vec<PluginFeature>,
    pub default_io_layout: PluginIoLayout,
    pub audio_bus_count: usize,
    pub parameter_count: usize,
    pub state_contract: PluginStateContract,
    pub processing_contract: PluginProcessingContract,
    pub lifecycle_contract: PluginLifecycleContract,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginFormatCoverageRecord {
    pub format: PluginFormat,
    pub discovered_type_count: usize,
    pub instrument_count: usize,
    pub audio_effect_count: usize,
    pub analyzer_count: usize,
    pub utility_count: usize,
    pub note_effect_count: usize,
    pub supports_snapshot_count: usize,
    pub supports_prepare_count: usize,
    pub supports_activate_count: usize,
    pub accepts_midi_count: usize,
    pub produces_midi_count: usize,
    pub max_audio_bus_count: usize,
    pub max_parameter_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCapabilityCoverageSummary {
    pub discovered_format_count: usize,
    pub multi_format_catalog: bool,
    pub instrument_count: usize,
    pub audio_effect_count: usize,
    pub analyzer_count: usize,
    pub utility_count: usize,
    pub note_effect_count: usize,
    pub supports_snapshot_count: usize,
    pub supports_reset_count: usize,
    pub supports_bypass_count: usize,
    pub exposes_latency_count: usize,
    pub exposes_tail_count: usize,
    pub sample_accurate_automation_count: usize,
    pub accepts_midi_count: usize,
    pub accepts_note_events_count: usize,
    pub produces_midi_count: usize,
    pub silence_aware_count: usize,
    pub requires_main_thread_for_state_count: usize,
    pub supports_prepare_count: usize,
    pub supports_activate_count: usize,
    pub supports_reset_while_active_count: usize,
    pub max_audio_bus_count: usize,
    pub max_parameter_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginDiscoverySnapshot {
    pub scan_count: usize,
    pub format_filtered_scan_count: usize,
    pub discovered_type_count: usize,
    pub discovered_format_count: usize,
    pub last_scan: Option<RuntimePluginScanReceipt>,
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    pub discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxSpec {
    pub sandbox_id: String,
    pub plugin_format: PluginFormat,
    pub plugin_type_id: Option<String>,
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
    fn apply_plugin_placement_policy(
        &mut self,
        policy: RuntimePluginPlacementPolicy,
    ) -> Result<(), RuntimeError>;
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
    fn apply_automation_projection(
        &mut self,
        projection: RuntimeAutomationProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_tempo_map_projection(
        &mut self,
        projection: RuntimeTempoMapProjection,
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
    fn get_acceptance_receipt(&self) -> RuntimeAcceptanceReceipt;
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    fn get_scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot;
    fn get_scheduler_topology_summary(&self) -> RuntimeSchedulerTopologySummary;
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    fn get_metering_snapshot(&self) -> RuntimeMeteringSnapshot;
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot;
    fn get_transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot;
    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot;
    fn get_offline_render_session_snapshot(&self) -> RuntimeOfflineRenderSessionSnapshot;
    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot;
    fn get_media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot;
    fn get_tempo_map_snapshot(&self) -> RuntimeTempoMapSnapshot;
    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot;
    fn get_clip_processing_pipeline_snapshot(&self) -> RuntimeClipProcessingPipelineSnapshot;
    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot;
    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot;
    fn get_execution_topology_summary(&self) -> RuntimeExecutionTopologySummary;
    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot;
    fn get_plugin_discovery_snapshot(&self) -> RuntimePluginDiscoverySnapshot;
    fn get_plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot;
    fn get_plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot;
    fn get_plugin_recall_handoff_snapshot(&self) -> RuntimePluginRecallHandoffSnapshot;
    fn get_last_deferred_service_receipt(&self) -> Option<RuntimeDeferredServiceReceipt>;
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
    fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError>;
    fn stop_media_preview(&mut self) -> Result<(), RuntimeError>;
    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError>;
    fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError>;
    fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError>;
    fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError>;
    fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError>;
    fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError>;
    fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError>;
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_hardware::{AudioSampleFormat, BackendHealth};

    fn host_io_summary(
        fallback_state: RuntimeHostClockFallbackState,
        transition_state: RuntimeHostClockTransitionState,
        stream_state: RuntimeHostAudioStreamState,
        backend_health: BackendHealth,
        restart_attempt_count: u64,
        restart_failure_count: u64,
        device_loss_count: u64,
    ) -> RuntimeHostIoSummary {
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_name: "coreaudio".to_string(),
                device_id: "device:main".to_string(),
                device_name: "Main Output".to_string(),
                sample_rate: 48_000,
                buffer_size: 256,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                simulated: false,
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
                callback_count: 32,
                total_callback_frames: 8_192,
                total_runtime_output_frames: 8_192,
                copied_output_samples: 16_384,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.42),
                last_runtime_graph_id: Some("graph:main".to_string()),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: RuntimeHostClockSource::Internal,
                ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
                restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
                processing_sample_rate_hz: 48_000,
                hardware_sample_rate_hz: 48_000,
                clock_domain: RuntimeHostClockDomain::SameClock,
                fallback_state,
                transition_state,
                crossing_required: false,
                callback_interval_ms: 5.333,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples: None,
                output_latency_samples: 256,
                round_trip_latency_samples: None,
                graph_latency_samples: 128,
                estimated_output_latency_samples: 384,
                estimated_round_trip_latency_samples: None,
                output_latency_ms: 5.333,
                graph_latency_ms: 2.667,
                estimated_output_latency_ms: 8.0,
                estimated_round_trip_latency_ms: None,
            },
            runtime_graph_id_matches_pump: true,
        }
    }

    #[test]
    fn runtime_external_io_snapshot_marks_clock_fallback_active() {
        let summary = host_io_summary(
            RuntimeHostClockFallbackState::RuntimeResampled,
            RuntimeHostClockTransitionState::EnteredCrossClockFallback,
            RuntimeHostAudioStreamState::Running,
            BackendHealth::Recovering,
            1,
            0,
            0,
        );

        let snapshot = summary.build_external_io_snapshot();

        assert_eq!(
            snapshot.health_state,
            RuntimeExternalIoHealthState::FallbackActive
        );
        assert_eq!(
            snapshot.device_change_state,
            RuntimeExternalIoDeviceChangeState::Recovering
        );
        assert!(snapshot.fallback_active);
        assert_eq!(
            snapshot.fallback_state,
            RuntimeHostClockFallbackState::RuntimeResampled
        );
        assert!(snapshot.summary.contains("fallback=true"));
    }

    #[test]
    fn runtime_external_io_snapshot_distinguishes_recovering_from_terminal_failure() {
        let recovering = host_io_summary(
            RuntimeHostClockFallbackState::Direct,
            RuntimeHostClockTransitionState::EnteredRecoveryFallback,
            RuntimeHostAudioStreamState::Running,
            BackendHealth::Recovering,
            2,
            1,
            1,
        )
        .build_external_io_snapshot();
        assert_eq!(
            recovering.health_state,
            RuntimeExternalIoHealthState::Recovering
        );
        assert_eq!(
            recovering.device_change_state,
            RuntimeExternalIoDeviceChangeState::Recovering
        );

        let failed = host_io_summary(
            RuntimeHostClockFallbackState::RecoveryConstrained,
            RuntimeHostClockTransitionState::EnteredRecoveryFallback,
            RuntimeHostAudioStreamState::Faulted,
            BackendHealth::Recovering,
            2,
            1,
            1,
        )
        .build_external_io_snapshot();
        assert_eq!(failed.health_state, RuntimeExternalIoHealthState::Faulted);
        assert_eq!(
            failed.device_change_state,
            RuntimeExternalIoDeviceChangeState::Failed
        );
        assert!(failed.fallback_active);
    }
}
