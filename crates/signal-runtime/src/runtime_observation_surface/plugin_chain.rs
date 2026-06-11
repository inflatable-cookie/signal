use super::*;

impl SignalRuntime {
    pub(crate) fn plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot {
        let lifecycle = self.plugin_lifecycle_snapshot();
        let lifecycle_by_sandbox = lifecycle
            .sandboxes
            .iter()
            .map(|sandbox| (sandbox.sandbox_id.as_str(), sandbox))
            .collect::<HashMap<_, _>>();
        let mut chain_indexes = HashMap::new();
        let mut chains = Vec::<RuntimePluginExecutionChainSummary>::new();

        for node in
            self.plan.planned_nodes.iter().filter(|node| {
                matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            })
        {
            let chain_id = runtime_plugin_chain_id(
                node.track_lane_id.as_deref(),
                node.bus_group_id.as_deref(),
                node.console_group_id.as_deref(),
                node.send_return_id.as_deref(),
            );
            let chain_index = if let Some(index) = chain_indexes.get(chain_id.as_str()) {
                *index
            } else {
                let index = chains.len();
                chains.push(RuntimePluginExecutionChainSummary {
                    chain_id: chain_id.clone(),
                    track_lane_id: node.track_lane_id.clone(),
                    bus_group_id: node.bus_group_id.clone(),
                    console_group_id: node.console_group_id.clone(),
                    send_return_id: node.send_return_id.clone(),
                    ..RuntimePluginExecutionChainSummary::default()
                });
                chain_indexes.insert(chain_id.clone(), index);
                index
            };

            let sandbox_id = node.plugin_sandbox_id.clone();
            let sandbox = sandbox_id
                .as_deref()
                .and_then(|sandbox_id| lifecycle_by_sandbox.get(sandbox_id).copied());
            let (
                placement_outcome,
                sandbox_group_key,
                placement_rule_id,
                shared_boundary_member_count,
                continuity_class,
                rebindable,
            ) = runtime_plugin_stage_assignment(sandbox_id.as_deref(), sandbox);
            let stage_index = chains[chain_index].stages.len();
            let stage = self.build_plugin_chain_stage(
                node,
                sandbox_id,
                sandbox,
                placement_outcome,
                sandbox_group_key,
                placement_rule_id,
                shared_boundary_member_count,
                continuity_class,
                rebindable,
                stage_index,
            );

            let chain = &mut chains[chain_index];
            chain.stage_count = chain.stage_count.saturating_add(1);
            match stage.placement_outcome {
                RuntimePluginIsolationOutcome::SharedSandbox => {
                    chain.shared_sandbox_stage_count =
                        chain.shared_sandbox_stage_count.saturating_add(1);
                }
                RuntimePluginIsolationOutcome::IsolatedSandbox => {
                    chain.isolated_sandbox_stage_count =
                        chain.isolated_sandbox_stage_count.saturating_add(1);
                }
                RuntimePluginIsolationOutcome::InProcess => {
                    chain.in_process_stage_count = chain.in_process_stage_count.saturating_add(1);
                }
            }
            chain.total_planned_latency_samples = chain
                .total_planned_latency_samples
                .saturating_add(stage.planned_latency_samples);
            chain.total_realized_latency_samples = chain
                .total_realized_latency_samples
                .saturating_add(stage.realized_latency_samples.unwrap_or(0));
            chain.total_tail_samples = chain
                .total_tail_samples
                .saturating_add(stage.tail_samples.unwrap_or(0));
            if matches!(
                stage.compensation_state,
                RuntimePluginCompensationState::PendingRender
            ) {
                chain.pending_render_stage_count =
                    chain.pending_render_stage_count.saturating_add(1);
            }
            if matches!(
                stage.compensation_state,
                RuntimePluginCompensationState::Settling
            ) {
                chain.settling_stage_count = chain.settling_stage_count.saturating_add(1);
            }
            if matches!(
                stage.compensation_state,
                RuntimePluginCompensationState::Compensated
            ) {
                chain.compensated_stage_count = chain.compensated_stage_count.saturating_add(1);
            }
            if matches!(
                stage.compensation_state,
                RuntimePluginCompensationState::Degraded
            ) {
                chain.degraded_stage_count = chain.degraded_stage_count.saturating_add(1);
            }
            if stage.bypassed {
                chain.bypassed_stage_count = chain.bypassed_stage_count.saturating_add(1);
            }
            if matches!(
                stage.compensation_state,
                RuntimePluginCompensationState::MissingBinding
            ) {
                chain.missing_binding_stage_count =
                    chain.missing_binding_stage_count.saturating_add(1);
            }
            if stage.rebindable {
                chain.rebindable_stage_count = chain.rebindable_stage_count.saturating_add(1);
            }
            if stage.continuity_class == RuntimeInterruptionClass::Terminal {
                chain.terminal_stage_count = chain.terminal_stage_count.saturating_add(1);
            }
            chain.stages.push(stage);
        }

        for _chain in &mut chains {}

        let snapshot = RuntimePluginChainSnapshot {
            chain_count: chains.len(),
            stage_count: chains.iter().map(|chain| chain.stage_count).sum(),
            shared_sandbox_stage_count: chains
                .iter()
                .map(|chain| chain.shared_sandbox_stage_count)
                .sum(),
            isolated_sandbox_stage_count: chains
                .iter()
                .map(|chain| chain.isolated_sandbox_stage_count)
                .sum(),
            in_process_stage_count: chains
                .iter()
                .map(|chain| chain.in_process_stage_count)
                .sum(),
            pending_render_stage_count: chains
                .iter()
                .map(|chain| chain.pending_render_stage_count)
                .sum(),
            settling_stage_count: chains.iter().map(|chain| chain.settling_stage_count).sum(),
            compensated_stage_count: chains
                .iter()
                .map(|chain| chain.compensated_stage_count)
                .sum(),
            degraded_stage_count: chains.iter().map(|chain| chain.degraded_stage_count).sum(),
            bypassed_stage_count: chains.iter().map(|chain| chain.bypassed_stage_count).sum(),
            missing_binding_stage_count: chains
                .iter()
                .map(|chain| chain.missing_binding_stage_count)
                .sum(),
            rebindable_stage_count: chains
                .iter()
                .map(|chain| chain.rebindable_stage_count)
                .sum(),
            terminal_stage_count: chains.iter().map(|chain| chain.terminal_stage_count).sum(),
            total_planned_latency_samples: chains
                .iter()
                .map(|chain| chain.total_planned_latency_samples)
                .sum(),
            total_realized_latency_samples: chains
                .iter()
                .map(|chain| chain.total_realized_latency_samples)
                .sum(),
            total_tail_samples: chains.iter().map(|chain| chain.total_tail_samples).sum(),
            chains,
        };
        snapshot
    }
}
