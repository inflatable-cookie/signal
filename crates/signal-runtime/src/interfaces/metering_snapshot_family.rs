use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMeterSourceRole {
    Utility,
    TrackLane,
    Bus,
    Send,
    Return,
    ConsoleNode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeterSourceSnapshot {
    pub bus_id: String,
    pub topology_role: RuntimeMeterSourceRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: f32,
    pub rms_level: f32,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeteringSnapshot {
    pub meter_count: usize,
    pub main_output_peak_level: Option<f32>,
    pub main_output_rms_level: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
    pub clipped_sample_count: u64,
    pub meters: Vec<RuntimeMeterSourceSnapshot>,
    pub track_lanes: Vec<RuntimeTrackLaneMeterSummary>,
    pub bus_groups: Vec<RuntimeBusGroupMeterSummary>,
    pub console_groups: Vec<RuntimeConsoleGroupMeterSummary>,
    pub send_returns: Vec<RuntimeSendReturnMeterSummary>,
    pub bus_connection_count: usize,
    pub auxiliary_path_count: usize,
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRoutedMeterAggregate {
    pub meter_count: usize,
    pub metered_bus_ids: Vec<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: Option<f32>,
    pub rms_level: Option<f32>,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTrackLaneMeterSummary {
    pub track_lane_id: String,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeBusGroupMeterSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeConsoleGroupMeterSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeSendReturnMeterSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

impl RuntimeMeteringSnapshot {
    pub fn with_execution_topology(mut self, topology: &RuntimeExecutionTopologySummary) -> Self {
        self.track_lanes = topology
            .track_lanes
            .iter()
            .map(|track_lane| RuntimeTrackLaneMeterSummary {
                track_lane_id: track_lane.track_lane_id.clone(),
                bus_group_ids: track_lane.bus_group_ids.clone(),
                input_bus_ids: track_lane.input_bus_ids.clone(),
                output_bus_ids: track_lane.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.track_lane_id.as_deref() == Some(track_lane.track_lane_id.as_str())
                    }),
                    format!("track_lane={}", track_lane.track_lane_id),
                ),
            })
            .collect();
        self.bus_groups = topology
            .bus_groups
            .iter()
            .map(|bus_group| RuntimeBusGroupMeterSummary {
                bus_group_id: bus_group.bus_group_id.clone(),
                topology_roles: bus_group.topology_roles.clone(),
                node_ids: bus_group.node_ids.clone(),
                input_bus_ids: bus_group.input_bus_ids.clone(),
                output_bus_ids: bus_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.bus_group_id.as_deref() == Some(bus_group.bus_group_id.as_str())
                    }),
                    format!("bus_group={}", bus_group.bus_group_id),
                ),
            })
            .collect();
        self.console_groups = topology
            .console_groups
            .iter()
            .map(|console_group| RuntimeConsoleGroupMeterSummary {
                console_group_id: console_group.console_group_id.clone(),
                node_ids: console_group.node_ids.clone(),
                input_bus_ids: console_group.input_bus_ids.clone(),
                output_bus_ids: console_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.console_group_id.as_deref()
                            == Some(console_group.console_group_id.as_str())
                    }),
                    format!("console_group={}", console_group.console_group_id),
                ),
            })
            .collect();
        self.send_returns = topology
            .send_returns
            .iter()
            .map(|send_return| RuntimeSendReturnMeterSummary {
                send_return_id: send_return.send_return_id.clone(),
                send_node_ids: send_return.send_node_ids.clone(),
                return_node_ids: send_return.return_node_ids.clone(),
                input_bus_ids: send_return.input_bus_ids.clone(),
                output_bus_ids: send_return.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.send_return_id.as_deref() == Some(send_return.send_return_id.as_str())
                    }),
                    format!("send_return={}", send_return.send_return_id),
                ),
            })
            .collect();
        self.bus_connection_count = topology.bus_connection_count;
        self.auxiliary_path_count = topology.auxiliary_path_count;
        self.bus_connections = topology.bus_connections.clone();
        self.auxiliary_paths = topology.auxiliary_paths.clone();
        self.summary = format!(
            "meters={} main_peak={:?} main_rms={:?} momentary_lufs={:?} short_term_lufs={:?} integrated_lufs={:?} clipped={} routes={}/{}/{}/{} bus_connections={} auxiliary_paths={}",
            self.meter_count,
            self.main_output_peak_level,
            self.main_output_rms_level,
            self.momentary_loudness_lufs,
            self.short_term_loudness_lufs,
            self.integrated_loudness_lufs,
            self.clipped_sample_count,
            self.track_lanes.len(),
            self.bus_groups.len(),
            self.send_returns.len(),
            self.console_groups.len(),
            self.bus_connection_count,
            self.auxiliary_path_count,
        );
        self
    }
}

fn aggregate_runtime_meter_sources<'a>(
    meters: impl Iterator<Item = &'a RuntimeMeterSourceSnapshot>,
    scope: String,
) -> RuntimeRoutedMeterAggregate {
    let mut aggregate = RuntimeRoutedMeterAggregate::default();

    for meter in meters {
        aggregate.meter_count += 1;
        if !aggregate.metered_bus_ids.contains(&meter.bus_id) {
            aggregate.metered_bus_ids.push(meter.bus_id.clone());
        }
        for producer_node_id in &meter.producer_node_ids {
            if !aggregate.producer_node_ids.contains(producer_node_id) {
                aggregate.producer_node_ids.push(producer_node_id.clone());
            }
        }
        aggregate.peak_level = Some(match aggregate.peak_level {
            Some(peak_level) => peak_level.max(meter.peak_level),
            None => meter.peak_level,
        });
        aggregate.rms_level = Some(match aggregate.rms_level {
            Some(rms_level) => rms_level.max(meter.rms_level),
            None => meter.rms_level,
        });
        aggregate.latency_samples = aggregate.latency_samples.max(meter.latency_samples);
        aggregate.tail_samples = aggregate.tail_samples.max(meter.tail_samples);
    }

    aggregate.summary = format!(
        "{scope} meters={} peak={:?} rms={:?} buses={:?} producers={}",
        aggregate.meter_count,
        aggregate.peak_level,
        aggregate.rms_level,
        aggregate.metered_bus_ids,
        aggregate.producer_node_ids.len(),
    );
    aggregate
}
