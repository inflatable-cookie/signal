use super::*;
use crate::runtime::runtime_utils::offline_render_plugin_override_status;

impl SignalRuntime {
    pub(crate) fn offline_plugin_execution_boundary_from_preview(
        &self,
        request: &RuntimeOfflineRenderRequest,
        _preview: &RuntimeOfflineRenderContractPreview,
    ) -> RuntimeOfflinePluginExecutionBoundary {
        let boundary_counts = runtime_plugin_boundary_counts(&self.engine.snapshot.planned_nodes);
        let sandboxes = self
            .plugin_lifecycle
            .snapshot(
                &self.plugin_placement_policy,
                &boundary_counts.sandbox_stage_counts,
                &self.plugin_discovery.discovered_types,
                &self.plugin_discovery.platform_coverage,
            )
            .sandboxes
            .into_iter()
            .map(|sandbox| (sandbox.sandbox_id.clone(), sandbox))
            .collect::<BTreeMap<_, _>>();
        let node_by_id = self
            .applied_graph
            .as_ref()
            .map(|graph| {
                graph
                    .nodes
                    .iter()
                    .map(|node| (node.node_id.as_str(), node))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let handoff = self.plugin_recall_handoff_snapshot();
        let last_processing_epoch = self.engine.snapshot.last_processing_epoch;
        let last_block_sequence = self.engine.snapshot.last_block_sequence;
        let block_size = self.config.graph.block_size.max(1);
        let block_count = (request.duration_samples as usize)
            .saturating_add(block_size.saturating_sub(1))
            / block_size;

        let stages = handoff
            .stages
            .iter()
            .map(|stage| {
                let execution_owner = match node_by_id.get(stage.node_id.as_str()) {
                    Some(node) if node.stages.is_empty() => {
                        RuntimeOfflinePluginExecutionOwner::HostDelegated
                    }
                    _ => RuntimeOfflinePluginExecutionOwner::SignalStageModel,
                };
                let (override_state, _) = offline_render_plugin_override_status(
                    self.engine.latest_plugin_node_renders.get(&stage.node_id),
                    stage.recall_payload.sandbox_id.as_ref(),
                    &sandboxes,
                    last_processing_epoch,
                    last_block_sequence,
                );
                let latest_override = self.engine.latest_plugin_node_renders.get(&stage.node_id);
                let host_delegate_required =
                    execution_owner == RuntimeOfflinePluginExecutionOwner::HostDelegated;
                let mut boundary = RuntimeOfflinePluginExecutionStageBoundary {
                    stage_id: stage.stage_id.clone(),
                    node_id: stage.node_id.clone(),
                    chain_id: stage.chain_id.clone(),
                    stage_index: stage.stage_index,
                    sandbox_id: stage.recall_payload.sandbox_id.clone(),
                    plugin_type_id: stage.recall_payload.plugin_type_id.clone(),
                    plugin_format: stage.recall_payload.plugin_format,
                    track_lane_id: stage.track_lane_id.clone(),
                    bus_group_id: stage.bus_group_id.clone(),
                    console_group_id: stage.console_group_id.clone(),
                    send_return_id: stage.send_return_id.clone(),
                    recall_state: stage.recall_state,
                    recall_payload: stage.recall_payload.clone(),
                    execution_owner,
                    host_delegate_required,
                    override_state,
                    latest_override_processing_epoch: latest_override
                        .map(|latest| latest.processing_epoch),
                    latest_override_block_sequence: latest_override.map(|latest| latest.block_sequence),
                    summary: String::new(),
                };
                boundary.summary = format!(
                    "stage={}:{} owner={:?} host_delegate={} override={:?} sandbox={:?} recall={:?}",
                    boundary.chain_id,
                    boundary.stage_index,
                    boundary.execution_owner,
                    boundary.host_delegate_required,
                    boundary.override_state,
                    boundary.sandbox_id.as_deref(),
                    boundary.recall_state,
                );
                boundary
            })
            .collect::<Vec<_>>();

        let mut boundary = RuntimeOfflinePluginExecutionBoundary {
            request_id: request.request_id.clone(),
            timeline_start_samples: request.timeline_start_samples,
            duration_samples: request.duration_samples,
            runtime_sample_rate_hz: self.config.sample_rate.0,
            export_sample_rate_hz: request.export_sample_rate_hz,
            block_size,
            block_count,
            stage_count: stages.len(),
            signal_stage_model_stage_count: stages
                .iter()
                .filter(|stage| {
                    stage.execution_owner == RuntimeOfflinePluginExecutionOwner::SignalStageModel
                })
                .count(),
            host_delegate_stage_count: stages
                .iter()
                .filter(|stage| stage.host_delegate_required)
                .count(),
            fresh_override_stage_count: stages
                .iter()
                .filter(|stage| {
                    stage.override_state == RuntimeOfflinePluginOverrideState::FreshLatestBlock
                })
                .count(),
            stale_override_stage_count: stages
                .iter()
                .filter(|stage| {
                    stage.override_state == RuntimeOfflinePluginOverrideState::StaleLatestBlock
                })
                .count(),
            stages,
            summary: String::new(),
        };
        boundary.summary = format!(
            "request={} stages={} signal_stage_model={} host_delegate={} fresh_overrides={} stale_overrides={} blocks={} runtime_rate={} export_rate={}",
            boundary.request_id,
            boundary.stage_count,
            boundary.signal_stage_model_stage_count,
            boundary.host_delegate_stage_count,
            boundary.fresh_override_stage_count,
            boundary.stale_override_stage_count,
            boundary.block_count,
            boundary.runtime_sample_rate_hz,
            boundary.export_sample_rate_hz,
        );
        boundary
    }

    pub(in super::super) fn offline_render_plugin_node_overrides(
        &self,
    ) -> Result<Vec<GraphNodeRenderOverride>, RuntimeError> {
        let Some(graph) = self.applied_graph.as_ref() else {
            return Ok(Vec::new());
        };
        let last_processing_epoch = self.engine.snapshot.last_processing_epoch;
        let last_block_sequence = self.engine.snapshot.last_block_sequence;
        let boundary_counts = runtime_plugin_boundary_counts(&self.engine.snapshot.planned_nodes);
        let sandboxes = self
            .plugin_lifecycle
            .snapshot(
                &self.plugin_placement_policy,
                &boundary_counts.sandbox_stage_counts,
                &self.plugin_discovery.discovered_types,
                &self.plugin_discovery.platform_coverage,
            )
            .sandboxes
            .into_iter()
            .map(|sandbox| (sandbox.sandbox_id.clone(), sandbox))
            .collect::<BTreeMap<_, _>>();

        graph
            .nodes
            .iter()
            .filter(|node| node.execution_class == GraphNodeExecutionClass::PluginBacked)
            .filter_map(|node| {
                let bound_sandbox_id = self.engine.plugin_node_bindings.get(&node.node_id);
                let (_, fresh_override) = offline_render_plugin_override_status(
                    self.engine.latest_plugin_node_renders.get(&node.node_id),
                    bound_sandbox_id,
                    &sandboxes,
                    last_processing_epoch,
                    last_block_sequence,
                );
                fresh_override.map(|render| {
                    Ok(GraphNodeRenderOverride {
                        node_id: node.node_id.clone(),
                        buffer: render.output.clone(),
                        latency_samples: render.latency_samples,
                        tail_samples: render.tail_samples,
                        bypassed: render.bypassed,
                    })
                })
            })
            .collect()
    }
}
