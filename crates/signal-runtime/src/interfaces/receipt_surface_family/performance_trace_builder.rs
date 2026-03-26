use super::performance_trace_builder_seed::{
    empty_runtime_performance_trace_receipt, seed_runtime_performance_trace_receipt,
};
use super::*;

pub(crate) fn build_runtime_performance_trace_receipt(
    observations: &[RuntimeObservationReport],
) -> RuntimePerformanceTraceReceipt {
    if observations.is_empty() {
        return empty_runtime_performance_trace_receipt();
    }

    let first_snapshot = observations[0].performance_snapshot();
    let mut receipt = seed_runtime_performance_trace_receipt(observations.len(), &first_snapshot);

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
        if snapshot.background_service_starvation_risk {
            receipt.background_starvation_observation_count = receipt
                .background_starvation_observation_count
                .saturating_add(1);
        }
        if snapshot.background_service_cancelled_work_item_count > 0 {
            receipt.background_cancellation_observation_count = receipt
                .background_cancellation_observation_count
                .saturating_add(1);
        }
        match snapshot.background_service_backpressure_source {
            Some(RuntimeDeferredServiceBackpressureSource::RealtimeAudio) => {
                receipt.background_realtime_backpressure_observation_count = receipt
                    .background_realtime_backpressure_observation_count
                    .saturating_add(1);
            }
            Some(
                RuntimeDeferredServiceBackpressureSource::RecoveryOverlap
                | RuntimeDeferredServiceBackpressureSource::CleanupBacklog
                | RuntimeDeferredServiceBackpressureSource::SafeMode,
            ) => {
                receipt.background_recovery_backpressure_observation_count = receipt
                    .background_recovery_backpressure_observation_count
                    .saturating_add(1);
            }
            None => {}
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
        receipt.peak_background_starved_work_item_count = receipt
            .peak_background_starved_work_item_count
            .max(snapshot.background_service_starved_work_item_count);
        receipt.peak_background_cancelled_work_item_count = receipt
            .peak_background_cancelled_work_item_count
            .max(snapshot.background_service_cancelled_work_item_count);
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
        "observations={} blocks={} playback_active={} recording_active={} background={}/{}/{}/{} overlap={}/{} backpressure={}/{} starvation={}/{} cancel={}/{} queue_peak={}/{}/{} prework_delta={}/{}/{} deadline={}/{}/{} budget_overruns={} hot_node={:?}/{} hot_group={:?}/{}/{} critical_lane={:?}/{}/{}/{} topology_incompatible={}",
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
        receipt.background_realtime_backpressure_observation_count,
        receipt.background_recovery_backpressure_observation_count,
        receipt.background_starvation_observation_count,
        receipt.peak_background_starved_work_item_count,
        receipt.background_cancellation_observation_count,
        receipt.peak_background_cancelled_work_item_count,
        receipt.peak_pending_prework_target_count,
        receipt.peak_prework_queue_depth,
        receipt.peak_background_queued_work_item_count,
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
