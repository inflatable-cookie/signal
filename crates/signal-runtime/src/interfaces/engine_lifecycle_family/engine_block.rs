use super::*;

/// Comprehensive execution snapshot captured at the end of every processed
/// block.
///
/// Holds the full planning shape (phases, lanes, dispatch counts), prework
/// cache and service metrics, transport position and transition, per-block
/// timing and deadline pressure, I/O peak levels, and the serialised planning
/// node list.  Used as the primary instrument for scheduler diagnostics and
/// by the host supervisor to decide whether to adjust pressure or topology.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeEngineBlockSnapshot {
    /// Graph ID of the topology active for this block.
    pub graph_id: Option<String>,
    /// Total number of nodes in the planning graph.
    pub node_count: usize,
    /// Number of stateful nodes (those that retain state across blocks).
    pub stateful_node_count: usize,
    /// Number of nodes that introduce latency compensation.
    pub latency_node_count: usize,
    /// Number of nodes backed by a plugin sandbox.
    pub plugin_backed_node_count: usize,
    /// Whether anticipative (prework) planning is enabled.
    pub anticipative_planning_enabled: bool,
    /// Number of nodes planned into the inline primary lane.
    pub inline_realtime_node_count: usize,
    /// Number of stateful realtime nodes.
    pub stateful_realtime_node_count: usize,
    /// Number of nodes eligible for anticipative prework.
    pub anticipative_eligible_node_count: usize,
    /// Total number of planning phases in the current topology.
    pub phase_count: usize,
    /// Number of phases that involve anticipative (prework) execution.
    pub anticipative_phase_count: usize,
    /// Ordered list of planning groups making up the phase sequence.
    pub phase_order: Vec<GraphNodePlanningGroup>,
    /// Total number of execution lanes.
    pub lane_count: usize,
    /// Number of lanes used for anticipative execution.
    pub anticipative_lane_count: usize,
    /// Ordered list of execution lanes.
    pub lane_order: Vec<GraphExecutionLane>,
    /// Scheduler topology summary for this block.
    pub scheduler_topology: RuntimeSchedulerTopologySummary,
    /// Total number of dispatches issued for this block.
    pub dispatch_count: usize,
    /// Number of dispatch boundary events.
    pub dispatch_boundary_count: usize,
    /// Ordered sequence of lanes in the dispatch plan.
    pub dispatch_order: Vec<GraphExecutionLane>,
    /// Number of dispatches prepared (prework) before the block started.
    pub prepared_dispatch_count: usize,
    /// Number of dispatches issued on the primary (non-prework) lane.
    pub realtime_dispatch_count: usize,
    /// Number of dispatch hand-offs from the prework service to realtime.
    pub dispatch_handoff_count: usize,
    /// Whether the prework cache is enabled.
    pub prework_cache_enabled: bool,
    /// Current state of the prework cache.
    pub prework_cache_state: RuntimePreworkCacheState,
    /// Maximum capacity of the prework cache queue.
    pub prework_cache_queue_capacity: usize,
    /// Current depth of the prework cache queue.
    pub prework_cache_queue_depth: usize,
    /// Peak depth the prework cache queue has reached.
    pub prework_cache_peak_queue_depth: usize,
    /// Total number of pending prework targets.
    pub prework_pending_target_count: usize,
    /// Number of pending targets scheduled for immediate processing.
    pub prework_pending_immediate_target_count: usize,
    /// Number of pending targets in the near-term window.
    pub prework_pending_near_term_target_count: usize,
    /// Number of pending targets deferred beyond the near-term window.
    pub prework_pending_deferred_target_count: usize,
    /// Block sequence of the next pending prework target, if any.
    pub prework_next_pending_target_block_sequence: Option<u64>,
    /// Current state of the prework service.
    pub prework_service_state: RuntimePreworkServiceState,
    /// Current pressure level on the prework service.
    pub prework_service_pressure: RuntimePreworkServicePressure,
    /// Semantic policy applied to prework service scheduling.
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    /// Number of plugin sandboxes currently active in the prework service.
    pub prework_service_active_plugin_sandboxes: u32,
    /// Number of plugin sandboxes bound to the prework service.
    pub prework_service_bound_plugin_sandboxes: usize,
    /// Number of bound sandboxes that are currently active.
    pub prework_service_active_bound_plugin_sandboxes: usize,
    /// Number of bound sandboxes in a degraded state.
    pub prework_service_degraded_bound_plugin_sandboxes: usize,
    /// Number of bound sandboxes that are missing (not yet available).
    pub prework_service_missing_bound_plugin_sandboxes: usize,
    /// Whether the plugin gate is currently active (blocking prework dispatch).
    pub prework_service_plugin_gate_active: bool,
    /// Number of recovery overlap sessions currently running in the prework service.
    pub prework_service_recovery_overlap_sessions: usize,
    /// Number of lingering (unreleased) prework service sessions.
    pub prework_service_lingering_sessions: usize,
    /// Number of sessions that faulted during detach in the prework service.
    pub prework_service_detach_faulted_sessions: usize,
    /// Whether the transport gate is currently active in the prework service.
    pub prework_service_transport_gate_active: bool,
    /// Total prework service cycles completed since startup.
    pub prework_service_cycle_count: u64,
    /// Total prework targets prepared by the service.
    pub prework_service_prepared_targets: u64,
    /// Total times the prework service was paused.
    pub prework_service_pause_count: u64,
    /// Total times the prework service was resumed.
    pub prework_service_resume_count: u64,
    /// Total times the prework service was starved of work.
    pub prework_service_starvation_count: u64,
    /// Total times the prework service was throttled.
    pub prework_service_throttle_count: u64,
    /// Total times the prework service yielded execution.
    pub prework_service_yield_count: u64,
    /// Processing epoch of the last prework service cycle.
    pub last_prework_service_processing_epoch: Option<u64>,
    /// Number of cycles requested in the last prework service run.
    pub last_prework_service_requested_cycles: usize,
    /// Number of cycles actually executed in the last prework service run.
    pub last_prework_service_effective_cycles: usize,
    /// Cycle count recorded for the last prework service run.
    pub last_prework_service_cycle_count: usize,
    /// Per-cycle budget (in frames) used in the last prework service run.
    pub last_prework_service_budget_per_cycle: Option<usize>,
    /// Effective per-cycle budget after any throttling in the last run.
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    /// Number of prework targets prepared in the last service run.
    pub last_prework_service_prepared_targets: usize,
    /// Block sequence of the last target serviced by the prework service.
    pub last_prework_serviced_target_block_sequence: Option<u64>,
    /// Backlog class observed for the last serviced prework target.
    pub last_prework_serviced_backlog_class: Option<RuntimePreworkBacklogClass>,
    /// Forecast mode requested by the host.
    pub prework_forecast_requested_mode: RuntimePreworkForecastMode,
    /// Forecast mode currently in effect.
    pub prework_forecast_mode: RuntimePreworkForecastMode,
    /// Whether a forecast policy has been configured.
    pub prework_forecast_policy_configured: bool,
    /// Active forecast profile, if any.
    pub prework_forecast_profile: Option<RuntimePreworkForecastProfile>,
    /// Source that provided the active forecast profile.
    pub prework_forecast_profile_source: Option<RuntimePreworkForecastProfileSource>,
    /// Override for the forecast profile's target window, if set.
    pub prework_forecast_profile_target_window_override: Option<usize>,
    /// Policy-derived target window for prework in blocks.
    pub prework_forecast_policy_target_window_blocks: Option<usize>,
    /// Number of block-sequence targets in the prework cache window.
    pub prework_cache_window_target_count: usize,
    /// Block sequences present in the prework cache window.
    pub prework_cache_window_target_block_sequences: Vec<u64>,
    /// Total prework cache admissions since startup.
    pub prework_cache_admissions: u64,
    /// Total prework cache consumptions since startup.
    pub prework_cache_consumptions: u64,
    /// Total queued admissions pending flush to the cache.
    pub prework_cache_queued_admissions: u64,
    /// Total queued consumptions pending flush from the cache.
    pub prework_cache_queued_consumptions: u64,
    /// Total prework cache hits since startup.
    pub prework_cache_hits: u64,
    /// Total prework cache misses since startup.
    pub prework_cache_misses: u64,
    /// Total times a cache entry was invalidated.
    pub prework_cache_invalidation_count: u64,
    /// Total times a cache entry was retired.
    pub prework_cache_retirement_count: u64,
    /// Number of retired entries that were never consumed.
    pub prework_cache_unconsumed_retirement_count: u64,
    /// Number of retired entries that were successfully consumed.
    pub prework_cache_consumed_retirement_count: u64,
    /// Whether the most recent block was a prework cache hit.
    pub last_prework_cache_hit: bool,
    /// Reason the last prework entry was invalidated, if applicable.
    pub last_prework_invalidation_reason: Option<RuntimePreworkInvalidationReason>,
    /// Reason the last prework entry was retired, if applicable.
    pub last_prework_retirement_reason: Option<RuntimePreworkRetirementReason>,
    /// Whether the last retired prework entry was unconsumed.
    pub last_prework_retired_unconsumed: Option<bool>,
    /// Freshness state of the prework cache.
    pub prework_cache_freshness_state: RuntimePreworkFreshnessState,
    /// Number of blocks for which the current cache entry remains valid.
    pub prework_cache_block_freshness_window: u64,
    /// Remaining valid blocks before the cache entry expires.
    pub prework_cache_remaining_valid_blocks: Option<u64>,
    /// Processing epoch at which the cache entry's validity expires.
    pub prework_cache_valid_until_processing_epoch: Option<u64>,
    /// Block sequence at which the cache entry's validity expires.
    pub prework_cache_valid_until_block_sequence: Option<u64>,
    /// Processing epoch when the prework source was last evaluated.
    pub last_prework_source_processing_epoch: Option<u64>,
    /// Block sequence of the last prework source evaluation.
    pub last_prework_source_block_sequence: Option<u64>,
    /// Processing epoch of the last prework cache admission.
    pub last_prework_admission_processing_epoch: Option<u64>,
    /// Block sequence of the last prework cache admission.
    pub last_prework_admission_block_sequence: Option<u64>,
    /// Block sequence the last admitted entry was drawn from.
    pub last_prework_admitted_from_block_sequence: Option<u64>,
    /// Processing epoch of the last prework cache consumption.
    pub last_prework_consumption_processing_epoch: Option<u64>,
    /// Block sequence of the last prework cache consumption.
    pub last_prework_consumption_block_sequence: Option<u64>,
    /// Block sequence the last consumed entry was drawn from.
    pub last_prework_consumed_from_block_sequence: Option<u64>,
    /// Processing epoch of the last prework cache retirement.
    pub last_prework_retirement_processing_epoch: Option<u64>,
    /// Block sequence of the last prework cache retirement.
    pub last_prework_retirement_block_sequence: Option<u64>,
    /// Ordered list of planned graph nodes for this block.
    pub planned_nodes: Vec<RuntimePlannedGraphNode>,
    /// Total number of stages in the execution plan.
    pub stage_count: usize,
    /// Accumulated latency in samples across all latency-compensated nodes.
    pub total_latency_samples: u32,
    /// Highest single-node latency in samples.
    pub max_node_latency_samples: u32,
    /// Accumulated tail length in samples across all nodes.
    pub total_tail_samples: u32,
    /// Highest single-node tail length in samples.
    pub max_node_tail_samples: u32,
    /// Tail length at the final output bus in samples.
    pub output_tail_samples: u32,
    /// Highest tail length across all buses in samples.
    pub max_bus_tail_samples: u32,
    /// Parameter epoch at the start of this block.
    pub parameter_epoch: Option<u64>,
    /// Number of parameter events applied this block.
    pub parameter_event_count: usize,
    /// Number of nodes targeted by parameter events this block.
    pub parameter_targeted_node_count: usize,
    /// Number of parameter events ignored (e.g. targeting unknown nodes).
    pub parameter_ignored_event_count: usize,
    /// Number of sub-blocks generated by parameter ramp splitting.
    pub parameter_sub_block_count: usize,
    /// Number of parameter events coalesced (duplicates removed).
    pub parameter_coalesced_event_count: usize,
    /// Total number of blocks processed since startup.
    pub processed_blocks: u64,
    /// Processing epoch for the most recently completed block.
    pub last_processing_epoch: Option<u64>,
    /// Block sequence number of the most recently completed block.
    pub last_block_sequence: Option<u64>,
    /// Frame count of the most recently completed block.
    pub last_frame_count: usize,
    /// Channel count of the most recently completed block.
    pub last_channel_count: usize,
    /// Wall-clock execution time of the most recently completed block in nanoseconds.
    pub last_block_execution_time_ns: Option<u64>,
    /// Deadline budget available for the most recently completed block in nanoseconds.
    pub last_block_deadline_budget_ns: Option<u64>,
    /// Fraction of the deadline budget consumed by the most recently completed block (0–100+).
    pub last_block_budget_utilization_percent: Option<f32>,
    /// Amount by which the most recently completed block overran its deadline in nanoseconds.
    pub last_block_budget_overrun_ns: Option<u64>,
    /// Deadline pressure classification for the most recently completed block.
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    /// Total number of blocks that have overrun their deadline since startup.
    pub budget_overrun_count: u64,
    /// Peak block execution time in nanoseconds since startup.
    pub peak_block_execution_time_ns: u64,
    /// Peak budget utilisation percentage seen across all blocks.
    pub peak_block_budget_utilization_percent: f32,
    /// Peak deadline overrun in nanoseconds seen across all blocks.
    pub peak_block_budget_overrun_ns: u64,
    /// Peak input level for the most recently completed block.
    pub last_input_peak: Option<f32>,
    /// Peak prework output level for the most recently completed block.
    pub last_prework_output_peak: Option<f32>,
    /// Peak realtime input level for the most recently completed block.
    pub last_realtime_input_peak: Option<f32>,
    /// Peak output level for the most recently completed block.
    pub last_output_peak: Option<f32>,
    /// RMS output level for the most recently completed block.
    pub last_output_rms: Option<f32>,
    /// First output sample value of the most recently completed block (for smoke-test assertions).
    pub last_first_output_sample: Option<f32>,
    /// Transport epoch at the time of this block.
    pub transport_epoch: u64,
    /// Transport transition kind that occurred during this block, if any.
    pub transport_transition: Option<RuntimeTransportTransitionKind>,
    /// Timeline position in samples at the start of this block.
    pub transport_block_start_samples: Option<i64>,
    /// Timeline position in samples at the end of this block.
    pub transport_block_end_samples: Option<i64>,
    /// Whether the transport loop boundary was crossed during this block.
    pub transport_loop_wrapped: bool,
    /// Execution context in which the most recent block was processed.
    pub last_execution_context: Option<GraphExecutionContext>,
}
