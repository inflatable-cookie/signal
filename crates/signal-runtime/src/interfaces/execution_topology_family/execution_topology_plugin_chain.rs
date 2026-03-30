use super::super::*;

impl RuntimeExecutionTopologySummary {
    pub fn with_plugin_chain_snapshot(mut self, snapshot: &RuntimePluginChainSnapshot) -> Self {
        let mut stage_by_node =
            std::collections::BTreeMap::<&str, &RuntimePluginChainStageSnapshot>::new();
        for chain in &snapshot.chains {
            self.plugin_chain.include_chain(chain);
            if let Some(track_lane_id) = chain.track_lane_id.as_deref() {
                if let Some(summary) = self
                    .track_lanes
                    .iter_mut()
                    .find(|summary| summary.track_lane_id == track_lane_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(bus_group_id) = chain.bus_group_id.as_deref() {
                if let Some(summary) = self
                    .bus_groups
                    .iter_mut()
                    .find(|summary| summary.bus_group_id == bus_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(console_group_id) = chain.console_group_id.as_deref() {
                if let Some(summary) = self
                    .console_groups
                    .iter_mut()
                    .find(|summary| summary.console_group_id == console_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(send_return_id) = chain.send_return_id.as_deref() {
                if let Some(summary) = self
                    .send_returns
                    .iter_mut()
                    .find(|summary| summary.send_return_id == send_return_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            for stage in &chain.stages {
                stage_by_node.insert(stage.node_id.as_str(), stage);
            }
        }

        for node in &mut self.nodes {
            if let Some(stage) = stage_by_node.get(node.node_id.as_str()) {
                node.spatial_execution = stage.spatial_execution.clone();
                node.plugin_recall_state = Some(stage.recall_state);
                node.plugin_recall = Some(stage.recall.clone());
                node.plugin_compensation_state = Some(stage.compensation_state);
                node.plugin_realized_latency_samples = stage.realized_latency_samples;
                node.plugin_tail_samples = stage.tail_samples;
            }
        }

        self
    }
}
