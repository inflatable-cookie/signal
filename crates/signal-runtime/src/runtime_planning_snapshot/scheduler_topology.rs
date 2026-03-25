use super::super::*;

impl SignalRuntime {
    pub(crate) fn refresh_scheduler_topology_summary(&mut self) {
        let Some(graph) = self.engine.graph.as_ref() else {
            self.engine.snapshot.scheduler_topology = RuntimeSchedulerTopologySummary::default();
            return;
        };

        let contract = graph.contract_summary();
        let mut track_lane_groups = BTreeSet::new();
        let mut bus_groups = BTreeSet::new();
        let mut send_return_groups = BTreeSet::new();
        let mut console_groups = BTreeSet::new();
        let mut missing_track_lane_ids = 0usize;
        let mut missing_bus_group_ids = 0usize;
        let mut missing_send_return_ids = 0usize;
        let mut missing_console_group_ids = 0usize;

        for node in &contract.node_contracts {
            match node.topology_role {
                GraphNodeTopologyRole::Utility => {}
                GraphNodeTopologyRole::TrackLane => {
                    if let Some(track_lane_id) = &node.track_lane_id {
                        track_lane_groups.insert(track_lane_id.clone());
                    } else {
                        missing_track_lane_ids = missing_track_lane_ids.saturating_add(1);
                    }
                    if let Some(bus_group_id) = &node.bus_group_id {
                        bus_groups.insert(bus_group_id.clone());
                    }
                }
                GraphNodeTopologyRole::Bus => {
                    if let Some(bus_group_id) = &node.bus_group_id {
                        bus_groups.insert(bus_group_id.clone());
                    } else {
                        missing_bus_group_ids = missing_bus_group_ids.saturating_add(1);
                    }
                }
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                    if let Some(send_return_id) = &node.send_return_id {
                        send_return_groups.insert(send_return_id.clone());
                    } else {
                        missing_send_return_ids = missing_send_return_ids.saturating_add(1);
                    }
                }
                GraphNodeTopologyRole::ConsoleNode => {
                    if let Some(console_group_id) = &node.console_group_id {
                        console_groups.insert(console_group_id.clone());
                    } else {
                        missing_console_group_ids = missing_console_group_ids.saturating_add(1);
                    }
                }
            }
        }

        let schedule_stream_count = self
            .applied_schedule
            .as_ref()
            .map(|schedule| schedule.stream_count);
        let has_topology_groups = contract.track_lane_node_count > 0
            || contract.bus_node_count > 0
            || contract.send_return_node_count > 0
            || contract.console_node_count > 0;
        let realtime_lane_index = self
            .engine
            .snapshot
            .lane_order
            .iter()
            .position(|lane| *lane == signal_graph::GraphExecutionLane::Realtime);
        let anticipative_lane_index = self
            .engine
            .snapshot
            .lane_order
            .iter()
            .position(|lane| *lane == signal_graph::GraphExecutionLane::Anticipative);

        let mut issues = Vec::new();
        if missing_track_lane_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingTrackLaneIds {
                node_count: missing_track_lane_ids,
            });
        }
        if missing_bus_group_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingBusGroupIds {
                node_count: missing_bus_group_ids,
            });
        }
        if missing_send_return_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingSendReturnIds {
                node_count: missing_send_return_ids,
            });
        }
        if missing_console_group_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingConsoleGroupIds {
                node_count: missing_console_group_ids,
            });
        }
        if has_topology_groups && realtime_lane_index.is_none() {
            issues.push(RuntimeSchedulerTopologyIssue::MissingRealtimeLaneForTopology);
        }
        if let (Some(anticipative_index), Some(realtime_index)) =
            (anticipative_lane_index, realtime_lane_index)
        {
            if anticipative_index > realtime_index {
                issues.push(RuntimeSchedulerTopologyIssue::AnticipativeLaneMustPrecedeRealtime);
            }
        }
        if contract.console_node_count > 0
            && self.engine.snapshot.dispatch_order.last().copied()
                != Some(signal_graph::GraphExecutionLane::Realtime)
        {
            issues.push(RuntimeSchedulerTopologyIssue::RealtimeDispatchMustTerminateTopology);
        }
        if !track_lane_groups.is_empty() {
            match schedule_stream_count {
                Some(actual_streams) if actual_streams < track_lane_groups.len() => {
                    issues.push(RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
                        required_streams: track_lane_groups.len(),
                        actual_streams,
                    });
                }
                None => issues.push(
                    RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
                        required_streams: track_lane_groups.len(),
                    },
                ),
                _ => {}
            }
        }

        self.engine.snapshot.scheduler_topology = RuntimeSchedulerTopologySummary {
            track_lane_node_count: contract.track_lane_node_count,
            track_lane_group_count: track_lane_groups.len(),
            bus_node_count: contract.bus_node_count,
            bus_group_count: bus_groups.len(),
            send_return_node_count: contract.send_return_node_count,
            send_return_group_count: send_return_groups.len(),
            console_node_count: contract.console_node_count,
            console_group_count: console_groups.len(),
            schedule_stream_count,
            compatible: issues.is_empty(),
            requires_host_reinterpretation: !issues.is_empty(),
            issues,
        };
    }
}
