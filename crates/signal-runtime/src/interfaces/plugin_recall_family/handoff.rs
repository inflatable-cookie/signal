use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePluginRecallHandoffStageId {
    pub chain_id: String,
    pub stage_index: usize,
    pub node_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSelection {
    pub stage_count: usize,
    pub stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffStage {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub stage_index: usize,
    pub chain_id: String,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSnapshot {
    pub stage_count: usize,
    pub unbound_stage_count: usize,
    pub cold_stage_count: usize,
    pub warm_stage_count: usize,
    pub recovered_stage_count: usize,
    pub unavailable_stage_count: usize,
    pub stages: Vec<RuntimePluginRecallHandoffStage>,
    pub summary: String,
}

impl RuntimePluginRecallHandoffSnapshot {
    pub fn from_plugin_chain_snapshot(snapshot: &RuntimePluginChainSnapshot) -> Self {
        let stages = snapshot
            .chains
            .iter()
            .flat_map(|chain| {
                chain
                    .stages
                    .iter()
                    .map(|stage| RuntimePluginRecallHandoffStage {
                        stage_id: RuntimePluginRecallHandoffStageId {
                            chain_id: chain.chain_id.clone(),
                            stage_index: stage.stage_index,
                            node_id: stage.node_id.clone(),
                        },
                        node_id: stage.node_id.clone(),
                        stage_index: stage.stage_index,
                        chain_id: chain.chain_id.clone(),
                        track_lane_id: stage.track_lane_id.clone(),
                        bus_group_id: stage.bus_group_id.clone(),
                        console_group_id: stage.console_group_id.clone(),
                        send_return_id: stage.send_return_id.clone(),
                        recall_state: stage.recall_state,
                        recall_payload: stage.recall.payload.clone(),
                    })
            })
            .collect::<Vec<_>>();
        let mut handoff = Self {
            stage_count: stages.len(),
            unbound_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Unbound)
                .count(),
            cold_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Cold)
                .count(),
            warm_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Warm)
                .count(),
            recovered_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Recovered)
                .count(),
            unavailable_stage_count: stages
                .iter()
                .filter(|stage| stage.recall_state == RuntimePluginRecallState::Unavailable)
                .count(),
            stages,
            summary: String::new(),
        };
        handoff.summary = format!(
            "stages={} unbound={} cold={} warm={} recovered={} unavailable={}",
            handoff.stage_count,
            handoff.unbound_stage_count,
            handoff.cold_stage_count,
            handoff.warm_stage_count,
            handoff.recovered_stage_count,
            handoff.unavailable_stage_count,
        );
        handoff
    }

    pub fn resolve_stage(
        &self,
        stage_id: &RuntimePluginRecallHandoffStageId,
    ) -> Option<&RuntimePluginRecallHandoffStage> {
        self.stages.iter().find(|stage| &stage.stage_id == stage_id)
    }

    pub fn resolve_selection<'a>(
        &'a self,
        selection: &RuntimePluginRecallHandoffSelection,
    ) -> Option<Vec<&'a RuntimePluginRecallHandoffStage>> {
        if selection.stage_count != selection.stage_ids.len() {
            return None;
        }
        selection
            .stage_ids
            .iter()
            .map(|stage_id| self.resolve_stage(stage_id))
            .collect()
    }
}
