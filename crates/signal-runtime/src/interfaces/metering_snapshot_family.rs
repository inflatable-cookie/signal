use super::*;

/// Topology role of a metered bus source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMeterSourceRole {
    /// Utility bus not attached to a routing group.
    Utility,
    /// Bus belonging to a track lane.
    TrackLane,
    /// Bus belonging to a bus group.
    Bus,
    /// Send bus in a send-return pair.
    Send,
    /// Return bus in a send-return pair.
    Return,
    /// Bus belonging to a console group node.
    ConsoleNode,
}

/// Peak/RMS snapshot for a single metered bus, including its topology role
/// and latency/tail sample counts.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeterSourceSnapshot {
    /// Unique identifier for the metered bus.
    pub bus_id: String,
    /// Topology role of this bus within the execution graph.
    pub topology_role: RuntimeMeterSourceRole,
    /// ID of the track lane this bus belongs to, if any.
    pub track_lane_id: Option<String>,
    /// ID of the bus group this bus belongs to, if any.
    pub bus_group_id: Option<String>,
    /// ID of the console group this bus belongs to, if any.
    pub console_group_id: Option<String>,
    /// ID of the send-return pair this bus belongs to, if any.
    pub send_return_id: Option<String>,
    /// IDs of graph nodes that produce audio to this bus.
    pub producer_node_ids: Vec<String>,
    /// Peak level of the last metered block in linear scale.
    pub peak_level: f32,
    /// RMS level of the last metered block in linear scale.
    pub rms_level: f32,
    /// Latency introduced by the producing node in samples.
    pub latency_samples: u32,
    /// Tail length of the producing node in samples.
    pub tail_samples: u32,
    /// Human-readable summary of this meter source.
    pub summary: String,
}

/// Full metering snapshot: all bus meters plus per-route aggregates for track
/// lanes, bus groups, console groups, and send-returns.  Enriched via
/// `with_execution_topology()`.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeteringSnapshot {
    /// Total number of metered bus sources.
    pub meter_count: usize,
    /// Peak level of the main output bus in linear scale, if available.
    pub main_output_peak_level: Option<f32>,
    /// RMS level of the main output bus in linear scale, if available.
    pub main_output_rms_level: Option<f32>,
    /// Momentary loudness of the main output in LUFS, if available.
    pub momentary_loudness_lufs: Option<f32>,
    /// Short-term loudness of the main output in LUFS, if available.
    pub short_term_loudness_lufs: Option<f32>,
    /// Integrated loudness of the main output in LUFS, if available.
    pub integrated_loudness_lufs: Option<f32>,
    /// Total number of clipped samples observed at the main output.
    pub clipped_sample_count: u64,
    /// Per-bus meter source snapshots.
    pub meters: Vec<RuntimeMeterSourceSnapshot>,
    /// Per-track-lane meter aggregates, populated by `with_execution_topology()`.
    pub track_lanes: Vec<RuntimeTrackLaneMeterSummary>,
    /// Per-bus-group meter aggregates, populated by `with_execution_topology()`.
    pub bus_groups: Vec<RuntimeBusGroupMeterSummary>,
    /// Per-console-group meter aggregates, populated by `with_execution_topology()`.
    pub console_groups: Vec<RuntimeConsoleGroupMeterSummary>,
    /// Per-send-return meter aggregates, populated by `with_execution_topology()`.
    pub send_returns: Vec<RuntimeSendReturnMeterSummary>,
    /// Number of bus connections in the execution topology.
    pub bus_connection_count: usize,
    /// Number of auxiliary paths in the execution topology.
    pub auxiliary_path_count: usize,
    /// Bus connection summaries from the execution topology.
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    /// Auxiliary path summaries from the execution topology.
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    /// Human-readable summary of the metering snapshot.
    pub summary: String,
}

/// Aggregated peak/RMS across all meters belonging to one routing group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRoutedMeterAggregate {
    /// Number of individual meters contributing to this aggregate.
    pub meter_count: usize,
    /// IDs of all buses contributing to this aggregate.
    pub metered_bus_ids: Vec<String>,
    /// IDs of all graph nodes producing audio to the aggregated buses.
    pub producer_node_ids: Vec<String>,
    /// Maximum peak level across all contributing meters, if any.
    pub peak_level: Option<f32>,
    /// Maximum RMS level across all contributing meters, if any.
    pub rms_level: Option<f32>,
    /// Maximum latency in samples across all contributing meters.
    pub latency_samples: u32,
    /// Maximum tail length in samples across all contributing meters.
    pub tail_samples: u32,
    /// Human-readable summary of this aggregate.
    pub summary: String,
}

/// Metering aggregate for a single track lane.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTrackLaneMeterSummary {
    /// ID of the track lane this summary covers.
    pub track_lane_id: String,
    /// IDs of bus groups belonging to this track lane.
    pub bus_group_ids: Vec<String>,
    /// IDs of input buses for this track lane.
    pub input_bus_ids: Vec<String>,
    /// IDs of output buses for this track lane.
    pub output_bus_ids: Vec<String>,
    /// Aggregated meter data for this track lane.
    pub aggregate: RuntimeRoutedMeterAggregate,
}

/// Metering aggregate for a single bus group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeBusGroupMeterSummary {
    /// ID of the bus group this summary covers.
    pub bus_group_id: String,
    /// Topology roles of nodes in this bus group.
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    /// IDs of graph nodes in this bus group.
    pub node_ids: Vec<String>,
    /// IDs of input buses for this bus group.
    pub input_bus_ids: Vec<String>,
    /// IDs of output buses for this bus group.
    pub output_bus_ids: Vec<String>,
    /// Aggregated meter data for this bus group.
    pub aggregate: RuntimeRoutedMeterAggregate,
}

/// Metering aggregate for a single console group.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeConsoleGroupMeterSummary {
    /// ID of the console group this summary covers.
    pub console_group_id: String,
    /// IDs of graph nodes in this console group.
    pub node_ids: Vec<String>,
    /// IDs of input buses for this console group.
    pub input_bus_ids: Vec<String>,
    /// IDs of output buses for this console group.
    pub output_bus_ids: Vec<String>,
    /// Aggregated meter data for this console group.
    pub aggregate: RuntimeRoutedMeterAggregate,
}

/// Metering aggregate for a single send-return pair.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeSendReturnMeterSummary {
    /// ID of the send-return pair this summary covers.
    pub send_return_id: String,
    /// IDs of graph nodes acting as sends in this pair.
    pub send_node_ids: Vec<String>,
    /// IDs of graph nodes acting as returns in this pair.
    pub return_node_ids: Vec<String>,
    /// IDs of input buses for this send-return pair.
    pub input_bus_ids: Vec<String>,
    /// IDs of output buses for this send-return pair.
    pub output_bus_ids: Vec<String>,
    /// Aggregated meter data for this send-return pair.
    pub aggregate: RuntimeRoutedMeterAggregate,
}

impl RuntimeMeteringSnapshot {
    /// Populates per-route meter aggregates from the given execution topology summary.
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
