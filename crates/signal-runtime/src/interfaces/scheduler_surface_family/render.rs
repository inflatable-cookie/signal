use super::*;

pub(crate) fn format_runtime_scheduler_summary_compact(
    summary: &RuntimeSchedulerExportSummary,
) -> String {
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

pub(crate) fn format_runtime_scheduler_summary_multiline(
    summary: &RuntimeSchedulerExportSummary,
) -> String {
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

pub(crate) fn format_runtime_block_summary_compact(
    summary: &RuntimeBlockExecutionSummary,
) -> String {
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

pub(crate) fn format_runtime_block_summary_multiline(
    summary: &RuntimeBlockExecutionSummary,
) -> String {
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
