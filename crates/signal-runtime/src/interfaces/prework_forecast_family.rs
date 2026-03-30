use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePluginDispatchState {
    pub transport: Option<TransportProjection>,
    pub parameter_batch: Option<ParameterBatch>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkCacheState {
    #[default]
    Disabled,
    Empty,
    Admitted,
    Consumed,
    Invalidated,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkFreshnessState {
    #[default]
    Disabled,
    Empty,
    Fresh,
    Expiring,
    Exhausted,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServiceState {
    #[default]
    Disabled,
    Idle,
    Pending,
    Servicing,
    Yielding,
    Paused,
    Starved,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServicePressure {
    #[default]
    Normal,
    Elevated,
    Critical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServiceSemanticPolicy {
    #[default]
    Balanced,
    LatencyFocused,
    PluginConstrained,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePreworkBacklogClass {
    #[default]
    Immediate,
    NearTerm,
    Deferred,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBlockDeadlinePressure {
    #[default]
    Normal,
    Elevated,
    Critical,
    Overrun,
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
