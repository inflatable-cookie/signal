use super::*;

fn json_runtime_scheduler_topology_issue(issue: &RuntimeSchedulerTopologyIssue) -> String {
    match issue {
        RuntimeSchedulerTopologyIssue::MissingTrackLaneIds { node_count } => format!(
            "{{\"kind\":\"MissingTrackLaneIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingBusGroupIds { node_count } => format!(
            "{{\"kind\":\"MissingBusGroupIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingSendReturnIds { node_count } => format!(
            "{{\"kind\":\"MissingSendReturnIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingConsoleGroupIds { node_count } => format!(
            "{{\"kind\":\"MissingConsoleGroupIds\",\"node_count\":{}}}",
            node_count
        ),
        RuntimeSchedulerTopologyIssue::MissingRealtimeLaneForTopology => {
            "{\"kind\":\"MissingRealtimeLaneForTopology\"}".into()
        }
        RuntimeSchedulerTopologyIssue::AnticipativeLaneMustPrecedeRealtime => {
            "{\"kind\":\"AnticipativeLaneMustPrecedeRealtime\"}".into()
        }
        RuntimeSchedulerTopologyIssue::RealtimeDispatchMustTerminateTopology => {
            "{\"kind\":\"RealtimeDispatchMustTerminateTopology\"}".into()
        }
        RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
            required_streams,
        } => format!(
            "{{\"kind\":\"MissingScheduleProjectionForTrackLanes\",\"required_streams\":{}}}",
            required_streams
        ),
        RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
            required_streams,
            actual_streams,
        } => format!(
            "{{\"kind\":\"InsufficientScheduleStreams\",\"required_streams\":{},\"actual_streams\":{}}}",
            required_streams, actual_streams
        ),
    }
}

pub(crate) fn json_runtime_scheduler_topology_summary(
    summary: &RuntimeSchedulerTopologySummary,
) -> String {
    let issues = summary
        .issues
        .iter()
        .map(json_runtime_scheduler_topology_issue)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"track_lane_node_count\":{},",
            "\"track_lane_group_count\":{},",
            "\"bus_node_count\":{},",
            "\"bus_group_count\":{},",
            "\"send_return_node_count\":{},",
            "\"send_return_group_count\":{},",
            "\"console_node_count\":{},",
            "\"console_group_count\":{},",
            "\"schedule_stream_count\":{},",
            "\"compatible\":{},",
            "\"requires_host_reinterpretation\":{},",
            "\"issues\":[{}]",
            "}}"
        ),
        summary.track_lane_node_count,
        summary.track_lane_group_count,
        summary.bus_node_count,
        summary.bus_group_count,
        summary.send_return_node_count,
        summary.send_return_group_count,
        summary.console_node_count,
        summary.console_group_count,
        json_option_usize(summary.schedule_stream_count),
        summary.compatible,
        summary.requires_host_reinterpretation,
        issues,
    )
}
