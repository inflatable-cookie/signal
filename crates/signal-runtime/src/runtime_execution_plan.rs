use super::*;

/// Planning-only execution state derived from the applied graph projection,
/// contracts, and plugin bindings.
///
/// The block-processing engine simulation was removed in g10.020; production
/// audio executes in `signal-render-plane`. What remains here is the plan
/// vocabulary the control surface still reports: planned nodes, lane order,
/// and declared latency.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeExecutionPlanState {
    pub(crate) graph: Option<ExecutableGraph>,
    pub(crate) graph_id: Option<String>,
    pub(crate) lane_order: Vec<signal_graph::GraphExecutionLane>,
    pub(crate) planned_nodes: Vec<crate::interfaces::RuntimePlannedGraphNode>,
    pub(crate) total_latency_samples: u32,
    pub(crate) plugin_node_bindings: HashMap<String, String>,
}

impl RuntimeExecutionPlanState {
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
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    pub(crate) fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: &PluginBackedNodeBindingProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot bind plugin-backed nodes before a graph is applied",
            ));
        };
        if projection.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "plugin-backed node bindings must target the currently applied graph",
            ));
        }

        let planning = graph.planning_summary(anticipative_enabled);
        let mut bindings = HashMap::new();
        for binding in &projection.bindings {
            if !planning.planned_nodes.iter().any(|node| {
                node.node_id == binding.node_id
                    && matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            }) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin-backed binding node '{}' does not resolve to a plugin-backed node",
                        binding.node_id
                    ),
                ));
            }
            if bindings
                .insert(binding.node_id.clone(), binding.sandbox_id.clone())
                .is_some()
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "duplicate plugin-backed binding provided for node '{}'",
                        binding.node_id
                    ),
                ));
            }
        }

        self.plugin_node_bindings = bindings;
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    pub(crate) fn refresh_planning(&mut self, anticipative_enabled: bool) {
        let Some(graph) = self.graph.as_ref() else {
            self.graph_id = None;
            self.lane_order.clear();
            self.planned_nodes.clear();
            self.total_latency_samples = 0;
            self.plugin_node_bindings.clear();
            return;
        };
        let planning = graph.planning_summary(anticipative_enabled);
        let contract = graph.contract_summary();
        let contract_by_node = contract
            .node_contracts
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        self.graph_id = Some(graph.graph_id().to_string());
        self.lane_order = planning.lane_order.clone();
        self.total_latency_samples = graph.total_latency_samples();
        self.planned_nodes = planning
            .planned_nodes
            .into_iter()
            .map(|node| {
                let contract = contract_by_node.get(node.node_id.as_str());
                let topology_role = contract
                    .map(|contract| contract.topology_role)
                    .unwrap_or(GraphNodeTopologyRole::Utility);
                crate::interfaces::RuntimePlannedGraphNode {
                    topology_role,
                    track_lane_id: contract.and_then(|contract| contract.track_lane_id.clone()),
                    bus_group_id: contract.and_then(|contract| contract.bus_group_id.clone()),
                    console_group_id: contract
                        .and_then(|contract| contract.console_group_id.clone()),
                    send_return_id: contract.and_then(|contract| contract.send_return_id.clone()),
                    input_bus_id: contract
                        .map(|contract| contract.input_bus_id.clone())
                        .unwrap_or_else(|| "main:in".into()),
                    output_bus_id: contract
                        .map(|contract| contract.output_bus_id.clone())
                        .unwrap_or_else(|| "main:out".into()),
                    plugin_sandbox_id: self.plugin_node_bindings.get(&node.node_id).cloned(),
                    node_id: node.node_id,
                    execution_class: node.execution_class,
                    group: node.group,
                    latency_samples: node.latency_samples,
                }
            })
            .collect();
    }
}
