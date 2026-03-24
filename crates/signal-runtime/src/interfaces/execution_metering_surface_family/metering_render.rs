use super::*;

pub(crate) fn format_runtime_metering_snapshot_compact(
    snapshot: &RuntimeMeteringSnapshot,
) -> String {
    let track_lane_shapes = snapshot
        .track_lanes
        .iter()
        .map(|track_lane| {
            format!(
                "{}:{}",
                track_lane.track_lane_id, track_lane.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let bus_group_shapes = snapshot
        .bus_groups
        .iter()
        .map(|bus_group| {
            format!(
                "{}:{}",
                bus_group.bus_group_id, bus_group.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let send_return_shapes = snapshot
        .send_returns
        .iter()
        .map(|send_return| {
            format!(
                "{}:{}",
                send_return.send_return_id, send_return.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let console_group_shapes = snapshot
        .console_groups
        .iter()
        .map(|console_group| {
            format!(
                "{}:{}",
                console_group.console_group_id, console_group.aggregate.meter_count
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    format!(
        " metering_snapshot_meters={} metering_snapshot_main={:?}/{:?} metering_snapshot_loudness={:?}/{:?}/{:?} metering_snapshot_clipped={} metering_snapshot_routes={}/{}/{}/{} metering_snapshot_track_lane_shapes={} metering_snapshot_bus_group_shapes={} metering_snapshot_send_return_shapes={} metering_snapshot_console_group_shapes={}",
        snapshot.meter_count,
        snapshot.main_output_peak_level,
        snapshot.main_output_rms_level,
        snapshot.momentary_loudness_lufs,
        snapshot.short_term_loudness_lufs,
        snapshot.integrated_loudness_lufs,
        snapshot.clipped_sample_count,
        snapshot.track_lanes.len(),
        snapshot.bus_groups.len(),
        snapshot.send_returns.len(),
        snapshot.console_groups.len(),
        track_lane_shapes,
        bus_group_shapes,
        send_return_shapes,
        console_group_shapes,
    )
}

pub(crate) fn format_runtime_metering_snapshot_multiline(
    snapshot: &RuntimeMeteringSnapshot,
) -> String {
    let meter_lines = snapshot
        .meters
        .iter()
        .enumerate()
        .map(|(index, meter)| {
            format!(
                "\nmetering_snapshot_meter_{}={}/{:?}/peak={:.3}/rms={:.3}/track_lane_id={:?}/bus_group_id={:?}/console_group_id={:?}/send_return_id={:?}/latency={}/tail={}/producers={:?}",
                index,
                meter.bus_id,
                meter.topology_role,
                meter.peak_level,
                meter.rms_level,
                meter.track_lane_id,
                meter.bus_group_id,
                meter.console_group_id,
                meter.send_return_id,
                meter.latency_samples,
                meter.tail_samples,
                meter.producer_node_ids,
            )
        })
        .collect::<String>();
    let track_lane_lines = snapshot
        .track_lanes
        .iter()
        .enumerate()
        .map(|(index, track_lane)| {
            format!(
                "\nmetering_snapshot_track_lane_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/producers={:?}/input={:?}/output={:?}",
                index,
                track_lane.track_lane_id,
                track_lane.aggregate.meter_count,
                track_lane.aggregate.peak_level,
                track_lane.aggregate.rms_level,
                track_lane.aggregate.metered_bus_ids,
                track_lane.aggregate.producer_node_ids,
                track_lane.input_bus_ids,
                track_lane.output_bus_ids,
            )
        })
        .collect::<String>();
    let bus_group_lines = snapshot
        .bus_groups
        .iter()
        .enumerate()
        .map(|(index, bus_group)| {
            format!(
                "\nmetering_snapshot_bus_group_{}={}/roles={:?}/meters={}/peak={:?}/rms={:?}/buses={:?}/nodes={:?}/input={:?}/output={:?}",
                index,
                bus_group.bus_group_id,
                bus_group.topology_roles,
                bus_group.aggregate.meter_count,
                bus_group.aggregate.peak_level,
                bus_group.aggregate.rms_level,
                bus_group.aggregate.metered_bus_ids,
                bus_group.node_ids,
                bus_group.input_bus_ids,
                bus_group.output_bus_ids,
            )
        })
        .collect::<String>();
    let console_group_lines = snapshot
        .console_groups
        .iter()
        .enumerate()
        .map(|(index, console_group)| {
            format!(
                "\nmetering_snapshot_console_group_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/nodes={:?}/input={:?}/output={:?}",
                index,
                console_group.console_group_id,
                console_group.aggregate.meter_count,
                console_group.aggregate.peak_level,
                console_group.aggregate.rms_level,
                console_group.aggregate.metered_bus_ids,
                console_group.node_ids,
                console_group.input_bus_ids,
                console_group.output_bus_ids,
            )
        })
        .collect::<String>();
    let send_return_lines = snapshot
        .send_returns
        .iter()
        .enumerate()
        .map(|(index, send_return)| {
            format!(
                "\nmetering_snapshot_send_return_{}={}/meters={}/peak={:?}/rms={:?}/buses={:?}/sends={:?}/returns={:?}/input={:?}/output={:?}",
                index,
                send_return.send_return_id,
                send_return.aggregate.meter_count,
                send_return.aggregate.peak_level,
                send_return.aggregate.rms_level,
                send_return.aggregate.metered_bus_ids,
                send_return.send_node_ids,
                send_return.return_node_ids,
                send_return.input_bus_ids,
                send_return.output_bus_ids,
            )
        })
        .collect::<String>();
    format!(
        "\nmetering_snapshot_meter_count={}\nmetering_snapshot_main_output_peak_level={:?}\nmetering_snapshot_main_output_rms_level={:?}\nmetering_snapshot_momentary_loudness_lufs={:?}\nmetering_snapshot_short_term_loudness_lufs={:?}\nmetering_snapshot_integrated_loudness_lufs={:?}\nmetering_snapshot_clipped_sample_count={}\nmetering_snapshot_track_lane_count={}\nmetering_snapshot_bus_group_count={}\nmetering_snapshot_send_return_count={}\nmetering_snapshot_console_group_count={}{}{}{}{}{}",
        snapshot.meter_count,
        snapshot.main_output_peak_level,
        snapshot.main_output_rms_level,
        snapshot.momentary_loudness_lufs,
        snapshot.short_term_loudness_lufs,
        snapshot.integrated_loudness_lufs,
        snapshot.clipped_sample_count,
        snapshot.track_lanes.len(),
        snapshot.bus_groups.len(),
        snapshot.send_returns.len(),
        snapshot.console_groups.len(),
        meter_lines,
        track_lane_lines,
        bus_group_lines,
        console_group_lines,
        send_return_lines,
    )
}
