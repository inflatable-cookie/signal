use super::*;

fn json_runtime_routed_meter_aggregate(aggregate: &RuntimeRoutedMeterAggregate) -> String {
    format!(
        concat!(
            "{{",
            "\"meter_count\":{},",
            "\"metered_bus_ids\":{},",
            "\"producer_node_ids\":{},",
            "\"peak_level\":{},",
            "\"rms_level\":{},",
            "\"latency_samples\":{},",
            "\"tail_samples\":{},",
            "\"summary\":{}",
            "}}"
        ),
        aggregate.meter_count,
        json_string_vec(&aggregate.metered_bus_ids),
        json_string_vec(&aggregate.producer_node_ids),
        json_option_f32(aggregate.peak_level),
        json_option_f32(aggregate.rms_level),
        aggregate.latency_samples,
        aggregate.tail_samples,
        json_option_string(Some(aggregate.summary.as_str())),
    )
}

fn json_runtime_track_lane_meter_summary_vec(summaries: &[RuntimeTrackLaneMeterSummary]) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"track_lane_id\":{},",
                    "\"bus_group_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.track_lane_id.as_str())),
                json_string_vec(&summary.bus_group_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_bus_group_meter_summary_vec(summaries: &[RuntimeBusGroupMeterSummary]) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"bus_group_id\":{},",
                    "\"topology_roles\":{},",
                    "\"node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.bus_group_id.as_str())),
                json_runtime_topology_role_vec(&summary.topology_roles),
                json_string_vec(&summary.node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_console_group_meter_summary_vec(
    summaries: &[RuntimeConsoleGroupMeterSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"console_group_id\":{},",
                    "\"node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.console_group_id.as_str())),
                json_string_vec(&summary.node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_send_return_meter_summary_vec(
    summaries: &[RuntimeSendReturnMeterSummary],
) -> String {
    let joined = summaries
        .iter()
        .map(|summary| {
            format!(
                concat!(
                    "{{",
                    "\"send_return_id\":{},",
                    "\"send_node_ids\":{},",
                    "\"return_node_ids\":{},",
                    "\"input_bus_ids\":{},",
                    "\"output_bus_ids\":{},",
                    "\"aggregate\":{}",
                    "}}"
                ),
                json_option_string(Some(summary.send_return_id.as_str())),
                json_string_vec(&summary.send_node_ids),
                json_string_vec(&summary.return_node_ids),
                json_string_vec(&summary.input_bus_ids),
                json_string_vec(&summary.output_bus_ids),
                json_runtime_routed_meter_aggregate(&summary.aggregate),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_metering_snapshot(snapshot: &RuntimeMeteringSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"meter_count\":{},",
            "\"main_output_peak_level\":{},",
            "\"main_output_rms_level\":{},",
            "\"momentary_loudness_lufs\":{},",
            "\"short_term_loudness_lufs\":{},",
            "\"integrated_loudness_lufs\":{},",
            "\"clipped_sample_count\":{},",
            "\"meters\":{},",
            "\"track_lanes\":{},",
            "\"bus_groups\":{},",
            "\"console_groups\":{},",
            "\"send_returns\":{},",
            "\"bus_connection_count\":{},",
            "\"auxiliary_path_count\":{},",
            "\"bus_connections\":{},",
            "\"auxiliary_paths\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.meter_count,
        json_option_f32(snapshot.main_output_peak_level),
        json_option_f32(snapshot.main_output_rms_level),
        json_option_f32(snapshot.momentary_loudness_lufs),
        json_option_f32(snapshot.short_term_loudness_lufs),
        json_option_f32(snapshot.integrated_loudness_lufs),
        snapshot.clipped_sample_count,
        json_runtime_meter_source_snapshot_vec(&snapshot.meters),
        json_runtime_track_lane_meter_summary_vec(&snapshot.track_lanes),
        json_runtime_bus_group_meter_summary_vec(&snapshot.bus_groups),
        json_runtime_console_group_meter_summary_vec(&snapshot.console_groups),
        json_runtime_send_return_meter_summary_vec(&snapshot.send_returns),
        snapshot.bus_connection_count,
        snapshot.auxiliary_path_count,
        json_runtime_bus_connection_summary_vec(&snapshot.bus_connections),
        json_runtime_auxiliary_path_summary_vec(&snapshot.auxiliary_paths),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
