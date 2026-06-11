use super::performance_trace_builder::build_runtime_performance_trace_receipt;
use super::*;

/// Aggregated performance trace across a span of observation reports.
///
/// Built via `RuntimeObservationReport::build_performance_trace_receipt()` or
/// `RuntimeSupervisorReport::build_performance_trace_receipt()`.  Contains
/// peak and cumulative counters for CPU load, block timing, background service,
/// deadline pressure, xruns, and hot-path latency across all observations.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceTraceReceipt {
    /// Number of observations included in this trace.
    pub observation_count: usize,
    /// Block sequence number of the first observation, if any.
    pub first_block_sequence: Option<u64>,
    /// Block sequence number of the last observation, if any.
    pub last_block_sequence: Option<u64>,
    /// Number of blocks spanned by this trace.
    pub processed_block_span: u64,
    /// Peak CPU load as a percentage across all observations.
    pub peak_cpu_load_percent: f32,
    /// Peak graph latency in milliseconds across all observations.
    pub peak_graph_latency_ms: f32,
    /// Peak block execution time in nanoseconds across all observations.
    pub peak_block_execution_time_ns: u64,
    /// Peak block budget utilization as a percentage across all observations.
    pub peak_block_budget_utilization_percent: f32,
    /// Peak block budget overrun in nanoseconds across all observations.
    pub peak_block_budget_overrun_ns: u64,
    /// Peak number of pending prework targets across all observations.
    pub peak_pending_prework_target_count: usize,
    /// Peak prework queue depth across all observations.
    pub peak_prework_queue_depth: usize,
    /// Peak number of queued background work items across all observations.
    pub peak_background_queued_work_item_count: usize,
    /// Peak number of deferred background work items across all observations.
    pub peak_background_deferred_work_item_count: usize,
    /// Number of observations where playback was active.
    pub playback_active_observation_count: usize,
    /// Number of observations where recording was active.
    pub recording_active_observation_count: usize,
    /// Total number of background service runs across all observations.
    pub background_service_run_count: usize,
    /// Total number of background service deferrals across all observations.
    pub background_service_defer_count: usize,
    /// Total number of background service throttle events across all observations.
    pub background_service_throttle_count: usize,
    /// Total number of background service aborts across all observations.
    pub background_service_abort_count: usize,
    /// Number of background service runs that occurred while playing.
    pub background_service_while_playing_count: usize,
    /// Number of background service runs that occurred while recording.
    pub background_service_while_recording_count: usize,
    /// Number of observations where background starvation was detected.
    pub background_starvation_observation_count: usize,
    /// Peak number of starved background work items across all observations.
    pub peak_background_starved_work_item_count: usize,
    /// Number of observations where background cancellation occurred.
    pub background_cancellation_observation_count: usize,
    /// Peak number of cancelled background work items across all observations.
    pub peak_background_cancelled_work_item_count: usize,
    /// Number of observations with realtime back-pressure on the background service.
    pub background_realtime_backpressure_observation_count: usize,
    /// Number of observations with recovery back-pressure on the background service.
    pub background_recovery_backpressure_observation_count: usize,
    /// Number of observations where the topology was incompatible.
    pub topology_incompatible_observation_count: usize,
    /// Number of observations with elevated deadline pressure.
    pub elevated_deadline_pressure_observation_count: usize,
    /// Number of observations with critical deadline pressure.
    pub critical_deadline_pressure_observation_count: usize,
    /// Number of observations with an overrun (missed) deadline.
    pub overrun_deadline_pressure_observation_count: usize,
    /// Cumulative increase in budget overrun count over this trace.
    pub budget_overrun_count_delta: u64,
    /// Cumulative increase in xrun count over this trace.
    pub xrun_count_delta: u64,
    /// Cumulative increase in prework service starvation count over this trace.
    pub prework_service_starvation_count_delta: u64,
    /// Cumulative increase in prework service throttle count over this trace.
    pub prework_service_throttle_count_delta: u64,
    /// Cumulative increase in prework service yield count over this trace.
    pub prework_service_yield_count_delta: u64,
    /// Node ID of the peak hot-path latency node, if any.
    pub peak_hot_latency_node_id: Option<String>,
    /// Group of the peak hot-path latency node, if any.
    pub peak_hot_latency_node_group: Option<String>,
    /// Latency contribution of the peak hot-path node in samples.
    pub peak_hot_latency_node_samples: u32,
    /// Group name of the peak latency group, if any.
    pub peak_hot_latency_group: Option<String>,
    /// Number of nodes in the peak latency group.
    pub peak_hot_latency_group_node_count: usize,
    /// Total latency of the peak latency group in samples.
    pub peak_hot_latency_group_total_samples: u32,
    /// Lane identifier of the peak critical path, if any.
    pub peak_critical_path_lane: Option<String>,
    /// Number of nodes in the peak critical path lane.
    pub peak_critical_path_lane_node_count: usize,
    /// Number of plugin-backed nodes in the peak critical path lane.
    pub peak_critical_path_lane_plugin_backed_node_count: usize,
    /// Total latency of the peak critical path lane in samples.
    pub peak_critical_path_lane_total_latency_samples: u32,
}

impl RuntimeObservationReport {
    /// Aggregates a performance trace receipt from a slice of observation reports.
    pub fn build_performance_trace_receipt(
        observations: &[Self],
    ) -> RuntimePerformanceTraceReceipt {
        build_runtime_performance_trace_receipt(observations)
    }
}

impl RuntimeSupervisorReport {
    /// Aggregates a performance trace receipt from a slice of supervisor reports.
    pub fn build_performance_trace_receipt(reports: &[Self]) -> RuntimePerformanceTraceReceipt {
        let observations = reports
            .iter()
            .map(|report| report.observation.clone())
            .collect::<Vec<_>>();
        RuntimeObservationReport::build_performance_trace_receipt(&observations)
    }
}
