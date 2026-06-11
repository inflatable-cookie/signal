use super::*;

/// Transport and parameter state from the last dispatched block; used to
/// decide if the next prework frame is still valid.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePluginDispatchState {
    /// Transport state from the last dispatched block.
    pub transport: Option<TransportProjection>,
    /// Parameter batch from the last dispatched block.
    pub parameter_batch: Option<ParameterBatch>,
}

/// State of the anticipative prework cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkCacheState {
    #[default]
    /// Prework caching is disabled.
    Disabled,
    /// Cache is enabled but contains no entries.
    Empty,
    /// A prework entry has been admitted and is ready to consume.
    Admitted,
    /// The admitted entry has been consumed by the realtime thread.
    Consumed,
    /// The cache was invalidated before the entry could be consumed.
    Invalidated,
}

/// Reason the prework cache was invalidated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkInvalidationReason {
    /// The runtime was reconfigured.
    RuntimeReconfigured,
    /// The runtime was stopped.
    RuntimeStopped,
    /// Planning was disabled.
    PlanningDisabled,
    /// The forecast plan changed.
    ForecastPlanChanged,
    /// The graph projection changed.
    GraphProjectionChanged,
    /// Transport playback started.
    TransportStarted,
    /// Transport playback stopped.
    TransportStopped,
    /// Transport position was seeked.
    TransportSeeked,
    /// Transport tempo changed.
    TransportTempoChanged,
    /// Transport loop state changed.
    TransportLoopStateChanged,
    /// Transport looped back to the start.
    TransportLoopWrapped,
    /// A parameter batch was applied.
    ParameterBatchApplied,
    /// The input signature changed.
    InputSignatureChanged,
    /// The processing epoch expired.
    ProcessingEpochExpired,
    /// The block sequence expired.
    BlockSequenceExpired,
    /// A newer admission superseded this entry.
    SupersededByAdmission,
    /// The planning window was revised.
    PlanningWindowRevised,
    /// The cache queue capacity was exceeded.
    QueueCapacityExceeded,
}

/// Reason a prework cache entry was retired without being consumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkRetirementReason {
    /// The runtime was reconfigured.
    RuntimeReconfigured,
    /// The runtime was stopped.
    RuntimeStopped,
    /// The forecast plan changed.
    ForecastPlanChanged,
    /// The graph projection changed.
    GraphProjectionChanged,
    /// Transport playback started.
    TransportStarted,
    /// Transport playback stopped.
    TransportStopped,
    /// Transport position was seeked.
    TransportSeeked,
    /// Transport tempo changed.
    TransportTempoChanged,
    /// Transport loop state changed.
    TransportLoopStateChanged,
    /// Transport looped back to the start.
    TransportLoopWrapped,
    /// A parameter batch was applied.
    ParameterBatchApplied,
    /// The input signature changed.
    InputSignatureChanged,
    /// The processing epoch expired.
    ProcessingEpochExpired,
    /// The block sequence expired.
    BlockSequenceExpired,
    /// Planning was disabled.
    PlanningDisabled,
    /// A newer admission superseded this entry.
    SupersededByAdmission,
    /// The planning window was revised.
    PlanningWindowRevised,
    /// The cache queue capacity was exceeded.
    QueueCapacityExceeded,
}

/// Freshness of the prework cache relative to the current transport position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkFreshnessState {
    #[default]
    /// Prework caching is disabled.
    Disabled,
    /// Cache is enabled but no entries are present.
    Empty,
    /// Cached entries cover the upcoming transport window.
    Fresh,
    /// Cached entries are close to the current position; freshness is expiring.
    Expiring,
    /// Cached entries have been exhausted by the current position.
    Exhausted,
    /// Cache was invalidated.
    Invalidated,
}

/// Operating state of the anticipative prework service.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServiceState {
    #[default]
    /// Prework service is disabled.
    Disabled,
    /// Service is enabled but not currently processing.
    Idle,
    /// Service has work queued but has not yet started.
    Pending,
    /// Service is actively processing prework.
    Servicing,
    /// Service is yielding CPU back to the realtime thread.
    Yielding,
    /// Service is temporarily paused.
    Paused,
    /// Service cannot make progress due to insufficient input.
    Starved,
}

/// Realtime pressure level signalled to the prework service.
///
/// Set via `set_prework_service_pressure()`.  The service uses this to decide
/// whether to yield CPU back to the realtime thread.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServicePressure {
    #[default]
    /// Realtime thread is operating within normal load bounds.
    Normal,
    /// Realtime load is elevated; service should yield more aggressively.
    Elevated,
    /// Realtime load is critical; service should yield immediately.
    Critical,
}

/// How the prework service should prioritise latency vs. plugin constraints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkServiceSemanticPolicy {
    #[default]
    /// Balance latency and plugin constraints equally.
    Balanced,
    /// Prioritise minimising latency over plugin constraints.
    LatencyFocused,
    /// Respect plugin execution constraints over latency targets.
    PluginConstrained,
}

/// A topology compatibility issue found by the scheduler planner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeSchedulerTopologyIssue {
    /// Some track lane nodes are missing their track lane ID assignment.
    MissingTrackLaneIds {
        /// Number of affected nodes.
        node_count: usize,
    },
    /// Some bus nodes are missing their bus group ID assignment.
    MissingBusGroupIds {
        /// Number of affected nodes.
        node_count: usize,
    },
    /// Some send/return nodes are missing their send/return ID assignment.
    MissingSendReturnIds {
        /// Number of affected nodes.
        node_count: usize,
    },
    /// Some console nodes are missing their console group ID assignment.
    MissingConsoleGroupIds {
        /// Number of affected nodes.
        node_count: usize,
    },
    /// The topology has no realtime dispatch lane.
    MissingRealtimeLaneForTopology,
    /// An anticipative lane is positioned after the realtime lane.
    AnticipativeLaneMustPrecedeRealtime,
    /// The realtime dispatch does not terminate the topology.
    RealtimeDispatchMustTerminateTopology,
    /// No schedule projection is available for the declared track lanes.
    MissingScheduleProjectionForTrackLanes {
        /// Number of streams required to cover all track lanes.
        required_streams: usize,
    },
    /// The schedule projection has fewer streams than the topology requires.
    InsufficientScheduleStreams {
        /// Number of streams required.
        required_streams: usize,
        /// Number of streams actually available.
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
    /// Number of graph nodes mapped to track lanes.
    pub track_lane_node_count: usize,
    /// Number of distinct track lane groups.
    pub track_lane_group_count: usize,
    /// Number of graph nodes mapped to buses.
    pub bus_node_count: usize,
    /// Number of distinct bus groups.
    pub bus_group_count: usize,
    /// Number of graph nodes mapped to send/return paths.
    pub send_return_node_count: usize,
    /// Number of distinct send/return groups.
    pub send_return_group_count: usize,
    /// Number of graph nodes mapped to the console.
    pub console_node_count: usize,
    /// Number of distinct console groups.
    pub console_group_count: usize,
    /// Number of schedule streams in the current schedule projection.
    pub schedule_stream_count: Option<usize>,
    /// Whether the topology is compatible with the current schedule.
    pub compatible: bool,
    /// Whether the host must reinterpret the plan boundary.
    pub requires_host_reinterpretation: bool,
    /// List of topology compatibility issues.
    pub issues: Vec<RuntimeSchedulerTopologyIssue>,
}

/// Type of transport state transition observed at a block boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTransportTransitionKind {
    /// First transport state observed at runtime start.
    Initial,
    /// Transport started playing.
    Started,
    /// Transport stopped.
    Stopped,
    /// Transport position was seeked to a new location.
    Seeked,
    /// Transport tempo changed.
    TempoChanged,
    /// Transport loop state toggled.
    LoopStateChanged,
    /// Transport position wrapped at the loop end.
    LoopWrapped,
}

/// Urgency class of a pending prework target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePreworkBacklogClass {
    #[default]
    /// Target must be processed in the next scheduling cycle.
    Immediate,
    /// Target should be processed within a few cycles.
    NearTerm,
    /// Target can be deferred to a later cycle.
    Deferred,
}

/// Block-level deadline pressure reported after each realtime block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBlockDeadlinePressure {
    #[default]
    /// Block completed within normal deadline bounds.
    Normal,
    /// Block deadline was close; pressure is elevated.
    Elevated,
    /// Block deadline was critically close.
    Critical,
    /// Block deadline was missed (xrun).
    Overrun,
}

/// Pre-computed audio for one future block that the prework cache admitted.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkWindowTarget {
    /// Block sequence number this prework target is prepared for.
    pub target_block_sequence: u64,
    /// Block sequence number at which this target was admitted into the cache.
    pub admitted_from_block_sequence: u64,
    /// Pre-rendered audio buffer for the target block.
    pub buffer: AudioBuffer,
    /// Parameter epoch override to apply when consuming this target, if any.
    pub parameter_epoch_override: Option<u64>,
    /// Transport state override to apply when consuming this target, if any.
    pub transport_override: Option<TransportProjection>,
}

/// Detailed policy parameters for the prework forecast engine (used in
/// `RawPolicyOverride` mode).
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkForecastPolicy {
    /// Number of blocks ahead the forecast engine should target.
    pub target_window_blocks: usize,
    /// Maximum number of prework blocks to prepare per scheduling cycle.
    pub prepare_budget_per_cycle: usize,
    /// Seed offset applied to the buffer sequence for reproducibility.
    pub buffer_seed_offset: u64,
    /// Whether the transport is considered to be playing for forecast purposes.
    pub transport_playing: bool,
    /// Transport tempo in BPM used by the forecast engine.
    pub transport_tempo_bpm: f64,
    /// Loop length in blocks used by the forecast engine.
    pub transport_loop_length_blocks: usize,
    /// Parameter automation target path the forecast engine tracks.
    pub parameter_target: String,
    /// Automation cycle length in blocks used by the forecast engine.
    pub parameter_cycle_length: u64,
}

/// Prework forecast operating mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkForecastMode {
    #[default]
    /// Prework forecasting is disabled.
    Disabled,
    /// Use the profile implied by `RuntimeProfile`.
    RuntimeRoleDefault,
    /// Use a caller-selected [`RuntimePreworkForecastProfile`].
    ExplicitProfile,
    /// Override with a raw `RuntimePreworkForecastPolicy`.
    RawPolicyOverride,
}

/// Pre-built forecast profile suitable for a deployment type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfile {
    /// Profile tuned for local (low-latency) deployments.
    Local,
    /// Profile tuned for server (high-throughput) deployments.
    Server,
}

/// How the active forecast profile was selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfileSource {
    /// Profile was derived from the runtime role default.
    RuntimeRoleDefault,
    /// Profile was explicitly selected by the caller.
    ExplicitSelection,
    /// Profile is overridden by a raw policy.
    RawPolicyOverride,
}

/// Caller-supplied forecast profile selection with an optional window override.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePreworkForecastProfileSelection {
    /// The forecast profile to activate.
    pub profile: RuntimePreworkForecastProfile,
    /// Optional override for the target window size in blocks.
    pub target_window_blocks_override: Option<usize>,
}
