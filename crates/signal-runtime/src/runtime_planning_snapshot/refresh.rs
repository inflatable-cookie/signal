use super::super::*;

impl RuntimeEngineState {
    pub(crate) fn refresh_planning(&mut self, anticipative_enabled: bool) {
        if !anticipative_enabled {
            self.invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if let Some(graph) = self.graph.as_ref() {
            let planning = graph.planning_summary(anticipative_enabled);
            let contract = graph.contract_summary();
            let stages_by_node = graph
                .plan()
                .nodes
                .iter()
                .map(|node| (node.node_id.as_str(), node.stages.clone()))
                .collect::<BTreeMap<_, _>>();
            let contract_by_node = contract
                .node_contracts
                .iter()
                .map(|node| (node.node_id.as_str(), node))
                .collect::<BTreeMap<_, _>>();
            self.snapshot.graph_id = Some(graph.graph_id().to_string());
            self.snapshot.node_count = graph.node_count();
            self.snapshot.stateful_node_count = graph.stateful_node_count();
            self.snapshot.latency_node_count = graph.latency_node_count();
            self.snapshot.plugin_backed_node_count = graph.plugin_backed_node_count();
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = planning.inline_realtime_node_count;
            self.snapshot.stateful_realtime_node_count = planning.stateful_realtime_node_count;
            self.snapshot.anticipative_eligible_node_count =
                planning.anticipative_eligible_node_count;
            self.snapshot.phase_count = planning.phase_count;
            self.snapshot.anticipative_phase_count = planning.anticipative_phase_count;
            self.snapshot.phase_order = planning.phase_order.clone();
            self.snapshot.lane_count = planning.lane_count;
            self.snapshot.anticipative_lane_count = planning.anticipative_lane_count;
            self.snapshot.lane_order = planning.lane_order.clone();
            self.snapshot.dispatch_count = planning.dispatch_count;
            self.snapshot.dispatch_boundary_count = planning.dispatch_boundary_count;
            self.snapshot.dispatch_order = planning
                .dispatches
                .iter()
                .map(|dispatch| dispatch.lane)
                .collect();
            self.snapshot.prepared_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
                .count();
            self.snapshot.realtime_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Realtime)
                .count();
            self.snapshot.dispatch_handoff_count = usize::from(
                self.snapshot.prepared_dispatch_count > 0
                    && self.snapshot.realtime_dispatch_count > 0,
            );
            self.snapshot.prework_cache_enabled = self.snapshot.prepared_dispatch_count > 0;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
            self.snapshot.prework_cache_state = if !self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Disabled
            } else if !self.prework_queue.is_empty() {
                match self.snapshot.prework_cache_state {
                    RuntimePreworkCacheState::Consumed => RuntimePreworkCacheState::Consumed,
                    RuntimePreworkCacheState::Admitted => RuntimePreworkCacheState::Admitted,
                    _ => RuntimePreworkCacheState::Admitted,
                }
            } else if matches!(
                self.snapshot.prework_cache_state,
                RuntimePreworkCacheState::Invalidated
            ) {
                RuntimePreworkCacheState::Invalidated
            } else {
                RuntimePreworkCacheState::Empty
            };
            self.snapshot.last_prework_cache_hit = false;
            let latest = self.prework_queue.back();
            self.snapshot.prework_cache_freshness_state =
                self.prework_freshness_state(latest, None);
            self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
                cache
                    .valid_until_block_sequence
                    .saturating_sub(cache.source_block_sequence)
            });
            self.snapshot.prework_cache_valid_until_processing_epoch =
                latest.map(|cache| cache.valid_until_processing_epoch);
            self.snapshot.prework_cache_valid_until_block_sequence =
                latest.map(|cache| cache.valid_until_block_sequence);
            self.snapshot.last_prework_source_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                latest.map(|cache| cache.admitted_from_block_sequence);
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
            self.snapshot.prework_cache_peak_queue_depth = self
                .snapshot
                .prework_cache_peak_queue_depth
                .max(self.prework_queue.len());
            self.snapshot.planned_nodes = planning
                .planned_nodes
                .into_iter()
                .map(|node| {
                    let contract = contract_by_node.get(node.node_id.as_str());
                    let topology_role = contract
                        .map(|contract| contract.topology_role)
                        .unwrap_or(GraphNodeTopologyRole::Utility);
                    let input_channels = contract
                        .map(|contract| contract.input_channels)
                        .unwrap_or(signal_primitives::ChannelLayout::Stereo);
                    let output_channels = contract
                        .map(|contract| contract.output_channels)
                        .unwrap_or(signal_primitives::ChannelLayout::Stereo);
                    let (input_bus_intent, output_bus_intent) =
                        crate::interfaces::runtime_bus_intents_for_topology_role(topology_role);
                    let secondary_input =
                        self.secondary_input_contracts
                            .get(&node.node_id)
                            .map(|contract| {
                                RuntimeSecondaryInputRouteSummary::from_contract_for_target(
                                    contract,
                                    RuntimeSecondaryInputTargetKind::NodeInput,
                                    node.node_id.as_str(),
                                )
                            });
                    let input_layout = crate::RuntimeMultichannelLayoutSummary::from_channel_layout(
                        input_channels,
                    );
                    let output_layout =
                        crate::RuntimeMultichannelLayoutSummary::from_channel_layout(
                            output_channels,
                        );
                    let spatial_execution =
                        crate::interfaces::runtime_spatial_execution_summary_for_stages(
                            node.node_id.as_str(),
                            stages_by_node
                                .get(node.node_id.as_str())
                                .map(|stages| stages.as_slice())
                                .unwrap_or(&[]),
                            &input_layout,
                            &output_layout,
                        );
                    crate::interfaces::RuntimePlannedGraphNode {
                        topology_role,
                        track_lane_id: contract.and_then(|contract| contract.track_lane_id.clone()),
                        bus_group_id: contract.and_then(|contract| contract.bus_group_id.clone()),
                        console_group_id: contract
                            .and_then(|contract| contract.console_group_id.clone()),
                        send_return_id: contract
                            .and_then(|contract| contract.send_return_id.clone()),
                        input_bus_id: contract
                            .map(|contract| contract.input_bus_id.clone())
                            .unwrap_or_else(|| "main:in".into()),
                        output_bus_id: contract
                            .map(|contract| contract.output_bus_id.clone())
                            .unwrap_or_else(|| "main:out".into()),
                        input_channels,
                        output_channels,
                        input_layout,
                        output_layout,
                        input_bus_intent,
                        output_bus_intent,
                        secondary_input,
                        spatial_execution,
                        plugin_sandbox_id: self.plugin_node_bindings.get(&node.node_id).cloned(),
                        node_id: node.node_id,
                        execution_class: node.execution_class,
                        group: node.group,
                        latency_samples: node.latency_samples,
                    }
                })
                .collect();
            self.snapshot.stage_count = graph.stage_count();
            self.snapshot.total_latency_samples = graph.total_latency_samples();
            self.snapshot.max_node_latency_samples = graph.max_node_latency_samples();
            self.snapshot.total_tail_samples = graph.total_tail_samples();
            self.snapshot.max_node_tail_samples = graph.max_node_tail_samples();
        } else {
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = 0;
            self.snapshot.stateful_realtime_node_count = 0;
            self.snapshot.anticipative_eligible_node_count = 0;
            self.snapshot.plugin_backed_node_count = 0;
            self.snapshot.phase_count = 0;
            self.snapshot.anticipative_phase_count = 0;
            self.snapshot.phase_order.clear();
            self.snapshot.lane_count = 0;
            self.snapshot.anticipative_lane_count = 0;
            self.snapshot.lane_order.clear();
            self.snapshot.dispatch_count = 0;
            self.snapshot.dispatch_boundary_count = 0;
            self.snapshot.dispatch_order.clear();
            self.snapshot.prepared_dispatch_count = 0;
            self.snapshot.realtime_dispatch_count = 0;
            self.snapshot.dispatch_handoff_count = 0;
            self.snapshot.stage_count = 0;
            self.snapshot.total_latency_samples = 0;
            self.snapshot.max_node_latency_samples = 0;
            self.snapshot.total_tail_samples = 0;
            self.snapshot.max_node_tail_samples = 0;
            self.snapshot.output_tail_samples = 0;
            self.snapshot.max_bus_tail_samples = 0;
            self.snapshot.prework_cache_enabled = false;
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
            self.snapshot.last_prework_cache_hit = false;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Disabled;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_cache_queue_depth = 0;
            self.snapshot.prework_pending_target_count = 0;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            self.snapshot.last_prework_consumption_processing_epoch = None;
            self.snapshot.last_prework_consumption_block_sequence = None;
            self.snapshot.last_prework_consumed_from_block_sequence = None;
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.planned_nodes.clear();
            self.plugin_node_bindings.clear();
            self.secondary_input_contracts.clear();
            self.latest_plugin_node_renders.clear();
            self.prework_queue.clear();
            self.pending_prework_targets.clear();
        }
    }
}
