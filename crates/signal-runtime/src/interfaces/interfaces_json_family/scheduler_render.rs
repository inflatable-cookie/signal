use super::*;

pub(crate) fn format_scheduler_topology_compact(
    summary: &RuntimeSchedulerTopologySummary,
) -> String {
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

pub(crate) fn format_scheduler_topology_multiline(
    summary: &RuntimeSchedulerTopologySummary,
) -> String {
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

pub(crate) fn format_runtime_scheduler_snapshot_compact(
    snapshot: &RuntimeSchedulerSnapshot,
) -> String {
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

pub(crate) fn format_runtime_scheduler_snapshot_multiline(
    snapshot: &RuntimeSchedulerSnapshot,
) -> String {
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
