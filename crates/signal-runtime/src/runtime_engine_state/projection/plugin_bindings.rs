use super::super::super::*;

impl RuntimePreworkTransportCondition {
    pub(crate) fn gate_active(self, pressure: RuntimePreworkServicePressure) -> bool {
        pressure != RuntimePreworkServicePressure::Normal
            && (self.lingering_sessions > 0 || self.detach_faulted_sessions > 0)
    }

    pub(crate) fn reduce_service_scope(
        self,
        effective_cycles: usize,
        effective_budget_per_cycle: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> (usize, usize, RuntimePreworkBacklogClass) {
        if self.detach_faulted_sessions > 0 || self.lingering_sessions > 0 {
            (
                effective_cycles.min(1),
                effective_budget_per_cycle.min(1),
                RuntimePreworkBacklogClass::Immediate,
            )
        } else if self.recovery_overlap_sessions > 0 {
            (
                effective_cycles.min(1),
                effective_budget_per_cycle.min(1),
                max_backlog_class.min(RuntimePreworkBacklogClass::NearTerm),
            )
        } else {
            (
                effective_cycles,
                effective_budget_per_cycle,
                max_backlog_class,
            )
        }
    }
}

impl RuntimeEngineState {
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
        self.pending_plugin_node_renders.clear();
        self.latest_plugin_node_renders.clear();
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    pub(crate) fn apply_plugin_node_render_batch(
        &mut self,
        batch: PluginNodeRenderBatch,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot apply plugin node renders before a graph is applied",
            ));
        };
        if batch.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "plugin node render batch must target the currently applied graph",
            ));
        }

        let planning = graph.planning_summary(true);
        let mut seen_node_ids = BTreeSet::new();
        for render in &batch.renders {
            if !planning.planned_nodes.iter().any(|node| {
                node.node_id == render.node_id
                    && matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            }) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin node render '{}' does not resolve to a plugin-backed node",
                        render.node_id
                    ),
                ));
            }
            if !seen_node_ids.insert(render.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("plugin node render batch repeats node '{}'", render.node_id),
                ));
            }
            if let Some(bound_sandbox_id) = self.plugin_node_bindings.get(&render.node_id) {
                if bound_sandbox_id != &render.sandbox_id {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "plugin node render '{}' is bound to sandbox '{}' not '{}'",
                            render.node_id, bound_sandbox_id, render.sandbox_id
                        ),
                    ));
                }
            } else {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin node render '{}' has no active plugin-backed binding",
                        render.node_id
                    ),
                ));
            }
        }

        self.pending_plugin_node_renders
            .insert((batch.processing_epoch, batch.block_sequence), batch);
        Ok(())
    }
}
