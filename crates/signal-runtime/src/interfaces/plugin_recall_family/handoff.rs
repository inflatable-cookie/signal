use super::*;

/// Stable identity for a plugin chain stage used in recall handoff selection.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimePluginRecallHandoffStageId {
    /// Chain identifier for this stage.
    pub chain_id: String,
    /// Index of this stage within the chain.
    pub stage_index: usize,
    /// Graph node identifier for this stage.
    pub node_id: String,
}

/// A named subset of recall handoff stages, e.g. all stages for a freeze artifact.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSelection {
    /// Expected number of stages in this selection.
    pub stage_count: usize,
    /// Stage identities included in this selection.
    pub stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
}

/// Single stage in a recall handoff: identity, chain location, and recall payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffStage {
    /// Composite stage identity.
    pub stage_id: RuntimePluginRecallHandoffStageId,
    /// Graph node identifier for this stage.
    pub node_id: String,
    /// Index of this stage within the chain.
    pub stage_index: usize,
    /// Chain identifier for this stage.
    pub chain_id: String,
    /// Track lane ID, if this stage is on a track lane chain.
    pub track_lane_id: Option<String>,
    /// Bus group ID, if this stage is on a bus chain.
    pub bus_group_id: Option<String>,
    /// Console group ID, if this stage is on a console chain.
    pub console_group_id: Option<String>,
    /// Send/return ID, if this stage is on a send/return chain.
    pub send_return_id: Option<String>,
    /// Recall readiness state for this stage.
    pub recall_state: RuntimePluginRecallState,
    /// Full recall payload for this stage.
    pub recall_payload: RuntimePluginRecallPayload,
}

/// Aggregate recall handoff snapshot: counts by state and the full stage list.
/// Built via `from_plugin_chain_snapshot()`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallHandoffSnapshot {
    /// Total number of stages in this handoff snapshot.
    pub stage_count: usize,
    /// Number of stages with no sandbox binding.
    pub unbound_stage_count: usize,
    /// Number of stages with a cold (untransferred) state.
    pub cold_stage_count: usize,
    /// Number of stages with a warm (ready) state.
    pub warm_stage_count: usize,
    /// Number of stages with a recovered state.
    pub recovered_stage_count: usize,
    /// Number of stages in an unavailable state.
    pub unavailable_stage_count: usize,
    /// Full list of recall handoff stage records.
    pub stages: Vec<RuntimePluginRecallHandoffStage>,
    /// Human-readable one-line summary.
    pub summary: String,
}

impl RuntimePluginRecallHandoffSnapshot {
    /// Builds a recall handoff snapshot from the full plugin chain snapshot.
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

    /// Returns the handoff stage matching the given stage identity, or `None` if not found.
    pub fn resolve_stage(
        &self,
        stage_id: &RuntimePluginRecallHandoffStageId,
    ) -> Option<&RuntimePluginRecallHandoffStage> {
        self.stages.iter().find(|stage| &stage.stage_id == stage_id)
    }

    /// Resolves all stages in a selection, returning `None` if any stage is missing or the count is inconsistent.
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
