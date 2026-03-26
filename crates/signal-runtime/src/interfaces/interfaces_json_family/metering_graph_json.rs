use super::*;

pub(crate) fn json_runtime_meter_source_snapshot(snapshot: &RuntimeMeterSourceSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"bus_id\":{},",
            "\"topology_role\":{},",
            "\"track_lane_id\":{},",
            "\"bus_group_id\":{},",
            "\"console_group_id\":{},",
            "\"send_return_id\":{},",
            "\"producer_node_ids\":{},",
            "\"peak_level\":{},",
            "\"rms_level\":{},",
            "\"latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.bus_id.as_str())),
        json_escape_string(&format!("{:?}", snapshot.topology_role)),
        json_option_string(snapshot.track_lane_id.as_deref()),
        json_option_string(snapshot.bus_group_id.as_deref()),
        json_option_string(snapshot.console_group_id.as_deref()),
        json_option_string(snapshot.send_return_id.as_deref()),
        json_string_vec(&snapshot.producer_node_ids),
        snapshot.peak_level,
        snapshot.rms_level,
        snapshot.latency_samples,
        snapshot.tail_samples,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

pub(crate) fn json_runtime_meter_source_snapshot_vec(
    snapshots: &[RuntimeMeterSourceSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_meter_source_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_execution_lane_order(lanes: &[GraphExecutionLane]) -> String {
    format!(
        "[{}]",
        lanes
            .iter()
            .map(|lane| json_option_string(Some(runtime_execution_lane_name(*lane))))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_worker_lane_instrumentation_summaries(
    summaries: &[RuntimeWorkerLaneInstrumentationSummary],
) -> String {
    format!(
        "[{}]",
        summaries
            .iter()
            .map(|summary| {
                format!(
                    concat!(
                        "{{",
                        "\"lane\":{},",
                        "\"node_count\":{},",
                        "\"plugin_backed_node_count\":{},",
                        "\"planning_group_count\":{},",
                        "\"total_latency_samples\":{},",
                        "\"max_node_latency_samples\":{}",
                        "}}"
                    ),
                    json_string(runtime_execution_lane_name(summary.lane)),
                    summary.node_count,
                    summary.plugin_backed_node_count,
                    summary.planning_group_count,
                    summary.total_latency_samples,
                    summary.max_node_latency_samples,
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_graph_execution_context(context: &GraphExecutionContext) -> String {
    format!(
        concat!(
            "{{",
            "\"processing_epoch\":{},",
            "\"block_sequence\":{},",
            "\"projection_epoch\":{},",
            "\"parameter_epoch\":{},",
            "\"configured_block_size\":{},",
            "\"anticipative_enabled\":{},",
            "\"transport_playing\":{},",
            "\"transport_tempo_bpm\":{},",
            "\"timeline_position_samples\":{}",
            "}}"
        ),
        context.processing_epoch,
        context.block_sequence,
        context.projection_epoch,
        context.parameter_epoch,
        context.configured_block_size,
        context.anticipative_enabled,
        context.transport_playing,
        json_option_f64(Some(context.transport_tempo_bpm)),
        json_option_i64(Some(context.timeline_position_samples)),
    )
}
