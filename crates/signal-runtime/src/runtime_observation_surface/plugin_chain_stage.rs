use super::*;

impl SignalRuntime {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_plugin_chain_stage(
        &self,
        node: &crate::interfaces::RuntimePlannedGraphNode,
        sandbox_id: Option<String>,
        sandbox: Option<&RuntimePluginSandboxSnapshot>,
        placement_outcome: RuntimePluginIsolationOutcome,
        sandbox_group_key: Option<String>,
        placement_rule_id: Option<String>,
        shared_boundary_member_count: usize,
        continuity_class: RuntimeInterruptionClass,
        rebindable: bool,
        current_block_sequence: Option<u64>,
        current_frame_count: usize,
        stage_index: usize,
    ) -> RuntimePluginChainStageSnapshot {
        let realized = self.engine.latest_plugin_node_renders.get(&node.node_id);
        let lifecycle_state = sandbox.map(|sandbox| sandbox.state);
        let lifecycle_stage = sandbox.and_then(|sandbox| sandbox.lifecycle_stage);
        let transport_stage = sandbox.and_then(|sandbox| sandbox.transport_stage);
        let recall = runtime_plugin_recall_snapshot(
            sandbox_id.as_deref(),
            sandbox,
            &self.plugin_discovery.discovered_types,
        );
        let recall_state = recall.state;
        let compensation = runtime_plugin_compensation_observation(
            sandbox_id.as_deref(),
            sandbox,
            realized,
            current_block_sequence,
            current_frame_count,
        );
        let compensation_state = compensation.state;
        let bypassed = matches!(compensation_state, RuntimePluginCompensationState::Bypassed);
        let active_transport = sandbox.is_some_and(|sandbox| sandbox.active_transport);
        let degraded_reasons = sandbox
            .map(|sandbox| sandbox.degraded_reasons.clone())
            .unwrap_or_default();
        let summary = format!(
            "node={} sandbox={:?} group={:?} placement={:?} rule={:?} members={} continuity={:?} rebindable={} lifecycle={:?}/{:?} transport={:?} recall={:?} compensation={:?} planned_latency={} realized_latency={:?} tail={:?} bypassed={} active_transport={}",
            node.node_id,
            sandbox_id,
            sandbox_group_key,
            placement_outcome,
            placement_rule_id,
            shared_boundary_member_count,
            continuity_class,
            rebindable,
            lifecycle_state,
            lifecycle_stage,
            transport_stage,
            recall_state,
            compensation_state,
            node.latency_samples,
            compensation.realized_latency_samples,
            compensation.tail_samples,
            bypassed,
            active_transport,
        );
        RuntimePluginChainStageSnapshot {
            node_id: node.node_id.clone(),
            stage_index,
            sandbox_id,
            sandbox_group_key,
            track_lane_id: node.track_lane_id.clone(),
            bus_group_id: node.bus_group_id.clone(),
            console_group_id: node.console_group_id.clone(),
            send_return_id: node.send_return_id.clone(),
            placement_outcome,
            placement_rule_id,
            shared_boundary_member_count,
            continuity_class,
            rebindable,
            io_layout: recall
                .payload
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| {
                    self.plugin_discovery
                        .discovered_types
                        .iter()
                        .find(|record| record.plugin_type_id == plugin_type_id)
                        .map(|record| record.default_multichannel_io.clone())
                })
                .unwrap_or_default(),
            complex_io_summary: recall
                .payload
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| {
                    self.plugin_discovery
                        .discovered_types
                        .iter()
                        .find(|record| record.plugin_type_id == plugin_type_id)
                        .map(|record| record.complex_io_summary.clone())
                })
                .unwrap_or_default(),
            secondary_input: node.secondary_input.as_ref().map(|route| {
                RuntimeSecondaryInputRouteSummary {
                    source_kind: route.source_kind,
                    source_id: route.source_id.clone(),
                    source_bus_id: route.source_bus_id.clone(),
                    target_kind: RuntimeSecondaryInputTargetKind::PluginInput,
                    target_id: node.node_id.clone(),
                    target_bus_id: route.target_bus_id.clone(),
                    attachment_policy: route.attachment_policy,
                    fallback_outcome: route.fallback_outcome,
                    summary: format!(
                        "source={:?}:{}/{} target={:?}:{}/{} policy={:?} fallback={:?}",
                        route.source_kind,
                        route.source_id,
                        route.source_bus_id.as_deref().unwrap_or("none"),
                        RuntimeSecondaryInputTargetKind::PluginInput,
                        node.node_id,
                        route.target_bus_id,
                        route.attachment_policy,
                        route.fallback_outcome,
                    ),
                }
            }),
            spatial_execution: node.spatial_execution.clone(),
            lifecycle_state,
            lifecycle_stage,
            transport_stage,
            recall_state,
            recall,
            compensation_state,
            planned_latency_samples: node.latency_samples,
            realized_latency_samples: compensation.realized_latency_samples,
            tail_samples: compensation.tail_samples,
            bypassed,
            active_transport,
            degraded_reasons,
            summary,
        }
    }
}
