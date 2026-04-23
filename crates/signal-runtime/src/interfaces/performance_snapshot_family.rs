use super::*;

mod performance_snapshot_render;
pub(crate) use performance_snapshot_render::runtime_execution_lane_name;
mod performance_snapshot_capture;

/// Comprehensive per-tick performance snapshot covering CPU load, block timing,
/// scheduler topology, prework service, background service, and hot-path
/// latency analysis.  One of these is produced each observation cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceSnapshot {
    /// Audio sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Audio block size in frames.
    pub block_size: usize,
    /// Total number of audio blocks processed since runtime start.
    pub processed_block_count: u64,
    /// Sequence number of the last processed block, if any.
    pub last_block_sequence: Option<u64>,
    /// CPU load of the audio thread as a percentage.
    pub cpu_load_percent: f32,
    /// Graph processing latency in milliseconds.
    pub graph_latency_ms: f32,
    /// Wall-clock execution time of the last block in nanoseconds, if known.
    pub last_block_execution_time_ns: Option<u64>,
    /// Deadline budget available for the last block in nanoseconds, if known.
    pub last_block_deadline_budget_ns: Option<u64>,
    /// Budget utilization of the last block as a percentage, if known.
    pub last_block_budget_utilization_percent: Option<f32>,
    /// Amount by which the last block overran its deadline budget in nanoseconds, if any.
    pub last_block_budget_overrun_ns: Option<u64>,
    /// Deadline pressure classification for the last block.
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    /// Cumulative number of deadline budget overruns since runtime start.
    pub budget_overrun_count: u64,
    /// Peak block execution time in nanoseconds since runtime start.
    pub peak_block_execution_time_ns: u64,
    /// Peak block budget utilization percentage since runtime start.
    pub peak_block_budget_utilization_percent: f32,
    /// Peak budget overrun amount in nanoseconds since runtime start.
    pub peak_block_budget_overrun_ns: u64,
    /// Cumulative number of xruns since runtime start.
    pub xrun_count: u64,
    /// Number of phases in the current scheduler plan.
    pub scheduler_phase_count: usize,
    /// Number of execution lanes in the current scheduler plan.
    pub scheduler_lane_count: usize,
    /// Total number of node dispatches in the current block.
    pub scheduler_dispatch_count: usize,
    /// Number of dispatches that were pre-prepared (anticipative).
    pub scheduler_prepared_dispatch_count: usize,
    /// Number of dispatches executed in real-time (non-anticipative).
    pub scheduler_realtime_dispatch_count: usize,
    /// Number of dispatch handoffs between real-time and anticipative lanes.
    pub scheduler_dispatch_handoff_count: usize,
    /// Whether the scheduler topology is compatible with the current graph.
    pub scheduler_topology_compatible: bool,
    /// Whether the host must reinterpret the scheduler topology.
    pub scheduler_topology_requires_host_reinterpretation: bool,
    /// Number of topology compatibility issues detected.
    pub scheduler_topology_issue_count: usize,
    /// Current state of the prework (anticipative) service.
    pub prework_service_state: RuntimePreworkServiceState,
    /// Current pressure level of the prework service.
    pub prework_service_pressure: RuntimePreworkServicePressure,
    /// Active semantic policy of the prework service.
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    /// Number of anticipative targets currently pending.
    pub pending_prework_target_count: usize,
    /// Number of deferred anticipative targets pending.
    pub pending_prework_deferred_target_count: usize,
    /// Current depth of the prework cache queue.
    pub prework_queue_depth: usize,
    /// Peak depth of the prework cache queue since runtime start.
    pub prework_peak_queue_depth: usize,
    /// Cumulative number of prework service cycles.
    pub prework_service_cycle_count: u64,
    /// Cumulative number of times the prework service was starved.
    pub prework_service_starvation_count: u64,
    /// Cumulative number of times the prework service was throttled.
    pub prework_service_throttle_count: u64,
    /// Cumulative number of times the prework service yielded.
    pub prework_service_yield_count: u64,
    /// Number of cycles the prework service effectively ran in the last block.
    pub last_prework_service_effective_cycles: usize,
    /// Budget allocated per prework service cycle in the last block, if known.
    pub last_prework_service_budget_per_cycle: Option<usize>,
    /// Effective budget used per prework service cycle in the last block, if known.
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    /// Backlog class serviced in the last prework cycle, if any.
    pub last_prework_serviced_backlog_class: Option<String>,
    /// Whether the transport gate is currently blocking anticipative dispatch.
    pub transport_gate_active: bool,
    /// Whether the plugin gate is currently blocking anticipative dispatch.
    pub plugin_gate_active: bool,
    /// ID of the node with the highest individual latency in the last block, if any.
    pub hot_latency_node_id: Option<String>,
    /// Planning group of the hot-latency node, if any.
    pub hot_latency_node_group: Option<String>,
    /// Topology role of the hot-latency node, if any.
    pub hot_latency_node_topology_role: Option<String>,
    /// Plugin sandbox ID of the hot-latency node, if any.
    pub hot_latency_node_plugin_sandbox_id: Option<String>,
    /// Latency of the hot-latency node in samples.
    pub hot_latency_node_samples: u32,
    /// Planning group with the highest total latency in the last block, if any.
    pub hot_latency_group: Option<String>,
    /// Number of nodes in the hot-latency group.
    pub hot_latency_group_node_count: usize,
    /// Total latency of the hot-latency group in samples.
    pub hot_latency_group_total_samples: u32,
    /// Execution lane with the highest total latency in the last block, if any.
    pub critical_path_lane: Option<String>,
    /// Number of nodes in the critical path lane.
    pub critical_path_lane_node_count: usize,
    /// Number of plugin-backed nodes in the critical path lane.
    pub critical_path_lane_plugin_backed_node_count: usize,
    /// Number of distinct planning groups in the critical path lane.
    pub critical_path_lane_planning_group_count: usize,
    /// Total latency of the critical path lane in samples.
    pub critical_path_lane_total_latency_samples: u32,
    /// Per-lane instrumentation summaries.
    pub worker_lane_summaries: Vec<RuntimeWorkerLaneInstrumentationSummary>,
    /// Class of the last background (deferred) service work, if any.
    pub background_service_class: Option<RuntimeDeferredServiceClass>,
    /// Decision made by the background service scheduler in the last block.
    pub background_service_decision: Option<RuntimeDeferredServiceDecision>,
    /// Reason for the background service decision in the last block.
    pub background_service_reason: Option<RuntimeDeferredServiceReason>,
    /// Priority band of the work item serviced in the last block, if any.
    pub background_service_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    /// Priority band that is blocking lower-priority work, if any.
    pub background_service_blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    /// Source of back-pressure affecting the background service, if any.
    pub background_service_backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    /// Whether the background service is at risk of starvation.
    pub background_service_starvation_risk: bool,
    /// Number of work items currently starved by the background service.
    pub background_service_starved_work_item_count: usize,
    /// Cause of the most recent background service cancellation, if any.
    pub background_service_cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    /// Number of work items cancelled in the last background service cycle.
    pub background_service_cancelled_work_item_count: usize,
    /// Number of work items queued in the background service.
    pub background_queued_work_item_count: usize,
    /// Number of work items deferred in the background service.
    pub background_deferred_work_item_count: usize,
    /// Number of work items pending cleanup in the background service.
    pub background_pending_cleanup_work_item_count: usize,
    /// Number of work items pending retry in the background service.
    pub background_pending_retry_work_item_count: usize,
    /// Human-readable summary of this performance snapshot.
    pub summary: String,
}

/// Instrumentation summary for a single scheduler worker lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeWorkerLaneInstrumentationSummary {
    /// Execution lane this summary covers.
    pub lane: GraphExecutionLane,
    /// Number of graph nodes dispatched in this lane.
    pub node_count: usize,
    /// Number of plugin-backed nodes in this lane.
    pub plugin_backed_node_count: usize,
    /// Number of distinct planning groups in this lane.
    pub planning_group_count: usize,
    /// Total latency across all nodes in this lane in samples.
    pub total_latency_samples: u32,
    /// Maximum individual node latency in this lane in samples.
    pub max_node_latency_samples: u32,
}
