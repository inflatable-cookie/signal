use super::super::super::*;

impl RuntimeEngineState {
    pub(crate) fn apply_graph_projection(
        &mut self,
        projection: &GraphProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        if projection.node_count != projection.nodes.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph node_count must match node projection count",
            ));
        }
        if projection
            .nodes
            .iter()
            .any(|node| node.node_id.is_empty() || node.stages.is_empty())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph nodes must have non-empty ids and at least one stage",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(node.execution_class, GraphNodeExecutionClass::PureTransform)
                && node.latency_samples != 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "pure-transform graph nodes must report zero latency",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(
                node.execution_class,
                GraphNodeExecutionClass::LatencyBearing
            ) && node.latency_samples == 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "latency-bearing graph nodes must report non-zero latency",
            ));
        }

        self.graph = Some(ExecutableGraph::new(
            projection.graph_id.clone(),
            projection
                .nodes
                .iter()
                .map(|node| GraphNodeSpec {
                    node_id: node.node_id.clone(),
                    execution_class: node.execution_class,
                    latency_samples: node.latency_samples,
                    tail_samples: 0,
                    buffer_contract: GraphNodeBufferContract::default(),
                    topology: GraphNodeTopologyMetadata::default(),
                    stages: node.stages.clone(),
                })
                .collect(),
        ));
        self.plugin_node_bindings.clear();
        self.secondary_input_contracts.clear();
        self.pending_plugin_node_renders.clear();
        self.latest_plugin_node_renders.clear();
        self.invalidate_prework_cache(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    pub(crate) fn apply_graph_contract_projection(
        &mut self,
        projection: &GraphContractProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot apply graph node contracts before a graph is applied",
            ));
        };
        if projection.contract_count != projection.nodes.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph contract_count must match node contract projection count",
            ));
        }
        if projection.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph contract projection graph_id must match the active graph",
            ));
        }
        if projection
            .nodes
            .iter()
            .any(|node| node.node_id.trim().is_empty())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph node contracts must reference non-empty node ids",
            ));
        }

        let plan = graph.plan().clone();
        let known_node_ids = plan
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut seen_contract_nodes = BTreeSet::new();
        for node in &projection.nodes {
            if !known_node_ids.contains(node.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "graph node contract references unknown node '{}'",
                        node.node_id
                    ),
                ));
            }
            if !seen_contract_nodes.insert(node.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("graph node contract repeats node '{}'", node.node_id),
                ));
            }
        }

        let contract_by_node = projection
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<HashMap<_, _>>();

        self.graph = Some(ExecutableGraph::new(
            plan.graph_id,
            plan.nodes
                .into_iter()
                .map(|mut node| {
                    if let Some(contract) = contract_by_node.get(node.node_id.as_str()) {
                        node.buffer_contract = GraphNodeBufferContract {
                            input: signal_graph::GraphNodeBusEndpoint::new(
                                contract.buffer_contract.input.bus_id.clone(),
                                contract.buffer_contract.input.channels,
                            ),
                            output: signal_graph::GraphNodeBusEndpoint::new(
                                contract.buffer_contract.output.bus_id.clone(),
                                contract.buffer_contract.output.channels,
                            ),
                            scratch_buffers: contract.buffer_contract.scratch_buffers,
                            silence_policy: contract.buffer_contract.silence_policy,
                            channel_adaptation: contract.buffer_contract.channel_adaptation,
                            reset_policy: contract.buffer_contract.reset_policy,
                        };
                        node.topology = GraphNodeTopologyMetadata {
                            role: contract.topology.role,
                            track_lane_id: contract.topology.track_lane_id.clone(),
                            bus_group_id: contract.topology.bus_group_id.clone(),
                            console_group_id: contract.topology.console_group_id.clone(),
                            send_return_id: contract.topology.send_return_id.clone(),
                        };
                    }
                    node
                })
                .collect(),
        ));
        self.secondary_input_contracts = projection
            .nodes
            .iter()
            .filter_map(|node| {
                node.buffer_contract
                    .secondary_input
                    .as_ref()
                    .map(|secondary_input| (node.node_id.clone(), secondary_input.clone()))
            })
            .collect();
        self.pending_plugin_node_renders.clear();
        self.latest_plugin_node_renders.clear();
        self.invalidate_prework_cache(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }
}
