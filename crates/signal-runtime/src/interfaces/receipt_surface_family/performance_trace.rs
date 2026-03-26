use super::performance_trace_builder::build_runtime_performance_trace_receipt;
use super::*;

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
    pub background_starvation_observation_count: usize,
    pub peak_background_starved_work_item_count: usize,
    pub background_cancellation_observation_count: usize,
    pub peak_background_cancelled_work_item_count: usize,
    pub background_realtime_backpressure_observation_count: usize,
    pub background_recovery_backpressure_observation_count: usize,
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

impl RuntimeObservationReport {
    pub fn build_performance_trace_receipt(
        observations: &[Self],
    ) -> RuntimePerformanceTraceReceipt {
        build_runtime_performance_trace_receipt(observations)
    }
}

impl RuntimeSupervisorReport {
    pub fn build_performance_trace_receipt(reports: &[Self]) -> RuntimePerformanceTraceReceipt {
        let observations = reports
            .iter()
            .map(|report| report.observation.clone())
            .collect::<Vec<_>>();
        RuntimeObservationReport::build_performance_trace_receipt(&observations)
    }
}
