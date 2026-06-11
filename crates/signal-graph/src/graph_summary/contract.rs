use super::*;
use signal_primitives::ChannelLayout;
use std::collections::{BTreeMap, BTreeSet};

impl ExecutableGraph {
    /// Validate all node contracts in the plan and return an aggregate summary.
    ///
    /// Checks bus IDs, channel adaptation compatibility, topology annotations,
    /// and ordering constraints. Issues are non-fatal; they are recorded in the
    /// returned [`GraphContractSummary`] and surfaced in [`GraphBlockReport`].
    pub fn contract_summary(&self) -> GraphContractSummary {
        let mut summary = GraphContractSummary::default();
        let all_output_buses = self
            .plan
            .nodes
            .iter()
            .map(|node| node.buffer_contract.output.bus_id.clone())
            .collect::<BTreeSet<_>>();
        let mut seen_output_buses = BTreeSet::new();
        let mut output_bus_layouts = BTreeMap::new();

        for node in &self.plan.nodes {
            let role = node.topology.role.unwrap_or(GraphNodeTopologyRole::Utility);
            let adaptation_result = classify_channel_adaptation(
                node.buffer_contract.input.channels,
                node.buffer_contract.output.channels,
                node.buffer_contract.channel_adaptation,
            );

            if node.buffer_contract.input.bus_id.trim().is_empty() {
                summary.issues.push(GraphContractIssue::EmptyInputBusId {
                    node_id: node.node_id.clone(),
                });
            }
            if node.buffer_contract.output.bus_id.trim().is_empty() {
                summary.issues.push(GraphContractIssue::EmptyOutputBusId {
                    node_id: node.node_id.clone(),
                });
            }
            if adaptation_result == GraphChannelAdaptationResult::Unsupported {
                summary
                    .issues
                    .push(GraphContractIssue::UnsupportedChannelAdaptation {
                        node_id: node.node_id.clone(),
                        input: node.buffer_contract.input.channels,
                        output: node.buffer_contract.output.channels,
                        mode: node.buffer_contract.channel_adaptation,
                    });
            }
            if node.buffer_contract.input.bus_id != "main:in" {
                if seen_output_buses.contains(&node.buffer_contract.input.bus_id) {
                } else if all_output_buses.contains(&node.buffer_contract.input.bus_id) {
                    summary
                        .issues
                        .push(GraphContractIssue::UnsupportedForwardReference {
                            node_id: node.node_id.clone(),
                            bus_id: node.buffer_contract.input.bus_id.clone(),
                        });
                } else {
                    summary
                        .issues
                        .push(GraphContractIssue::MissingInputBusProducer {
                            node_id: node.node_id.clone(),
                            bus_id: node.buffer_contract.input.bus_id.clone(),
                        });
                }
            }
            if role == GraphNodeTopologyRole::Send
                && node.buffer_contract.input.bus_id == node.buffer_contract.output.bus_id
            {
                summary
                    .issues
                    .push(GraphContractIssue::SendRequiresDistinctBuses {
                        node_id: node.node_id.clone(),
                    });
            }
            if role == GraphNodeTopologyRole::Return
                && node.buffer_contract.input.bus_id == node.buffer_contract.output.bus_id
            {
                summary
                    .issues
                    .push(GraphContractIssue::ReturnRequiresDistinctBuses {
                        node_id: node.node_id.clone(),
                    });
            }
            match role {
                GraphNodeTopologyRole::TrackLane if node.topology.track_lane_id.is_none() => {
                    summary.issues.push(GraphContractIssue::MissingTrackLaneId {
                        node_id: node.node_id.clone(),
                    });
                }
                GraphNodeTopologyRole::Bus if node.topology.bus_group_id.is_none() => {
                    summary.issues.push(GraphContractIssue::MissingBusGroupId {
                        node_id: node.node_id.clone(),
                    });
                }
                GraphNodeTopologyRole::ConsoleNode if node.topology.console_group_id.is_none() => {
                    summary
                        .issues
                        .push(GraphContractIssue::MissingConsoleGroupId {
                            node_id: node.node_id.clone(),
                        });
                }
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return
                    if node.topology.send_return_id.is_none() =>
                {
                    summary
                        .issues
                        .push(GraphContractIssue::MissingSendReturnId {
                            node_id: node.node_id.clone(),
                        });
                }
                GraphNodeTopologyRole::Utility => {}
                _ => {}
            }
            match output_bus_layouts.get(&node.buffer_contract.output.bus_id) {
                Some(expected) if *expected != node.buffer_contract.output.channels => {
                    summary
                        .issues
                        .push(GraphContractIssue::InconsistentOutputBusChannels {
                            node_id: node.node_id.clone(),
                            bus_id: node.buffer_contract.output.bus_id.clone(),
                            expected: *expected,
                            actual: node.buffer_contract.output.channels,
                        });
                }
                Some(_) => {}
                None => {
                    output_bus_layouts.insert(
                        node.buffer_contract.output.bus_id.clone(),
                        node.buffer_contract.output.channels,
                    );
                }
            }

            if node.buffer_contract.silence_policy == GraphNodeSilencePolicy::ClearOutput {
                summary.silence_clear_node_count += 1;
            }
            if adaptation_result != GraphChannelAdaptationResult::Exact {
                summary.adaptive_channel_node_count += 1;
            }
            if node.buffer_contract.reset_policy != GraphNodeResetPolicy::RetainAcrossBlocks {
                summary.resettable_node_count += 1;
            }
            summary.scratch_buffer_count += node.buffer_contract.scratch_buffers;

            match role {
                GraphNodeTopologyRole::Utility => {}
                GraphNodeTopologyRole::TrackLane => summary.track_lane_node_count += 1,
                GraphNodeTopologyRole::Bus => summary.bus_node_count += 1,
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                    summary.send_return_node_count += 1
                }
                GraphNodeTopologyRole::ConsoleNode => summary.console_node_count += 1,
            }

            summary.node_contracts.push(GraphNodeContractSummary {
                node_id: node.node_id.clone(),
                input_bus_id: node.buffer_contract.input.bus_id.clone(),
                output_bus_id: node.buffer_contract.output.bus_id.clone(),
                input_channels: node.buffer_contract.input.channels,
                output_channels: node.buffer_contract.output.channels,
                silence_policy: node.buffer_contract.silence_policy,
                channel_adaptation: node.buffer_contract.channel_adaptation,
                adaptation_result,
                scratch_buffers: node.buffer_contract.scratch_buffers,
                reset_policy: node.buffer_contract.reset_policy,
                topology_role: role,
                track_lane_id: node.topology.track_lane_id.clone(),
                bus_group_id: node.topology.bus_group_id.clone(),
                console_group_id: node.topology.console_group_id.clone(),
                send_return_id: node.topology.send_return_id.clone(),
            });

            seen_output_buses.insert(node.buffer_contract.output.bus_id.clone());
        }

        summary.issue_count = summary.issues.len();
        summary
    }
}

pub(crate) fn classify_channel_adaptation(
    input: ChannelLayout,
    output: ChannelLayout,
    mode: GraphChannelAdaptationMode,
) -> GraphChannelAdaptationResult {
    if input == output {
        return GraphChannelAdaptationResult::Exact;
    }

    if mode != GraphChannelAdaptationMode::AdaptiveMonoStereo {
        return GraphChannelAdaptationResult::Unsupported;
    }

    match (input, output) {
        (ChannelLayout::Mono, ChannelLayout::Stereo) => GraphChannelAdaptationResult::MonoToStereo,
        (ChannelLayout::Stereo, ChannelLayout::Mono) => GraphChannelAdaptationResult::StereoToMono,
        _ => GraphChannelAdaptationResult::Unsupported,
    }
}
