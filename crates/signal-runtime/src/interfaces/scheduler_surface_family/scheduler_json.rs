use super::*;

pub(crate) fn json_runtime_scheduler_export_summary(
    summary: &RuntimeSchedulerExportSummary,
) -> String {
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

pub(crate) fn json_runtime_scheduler_snapshot(snapshot: &RuntimeSchedulerSnapshot) -> String {
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
