use super::*;
use crate::runtime::runtime_plugin_recording::runtime_plugin_parity_coverage;
#[path = "state_model/sandbox_state.rs"]
mod sandbox_state;
#[path = "state_model/snapshot.rs"]
mod snapshot;
#[path = "state_model/transitions.rs"]
mod transitions;

pub(super) use super::placement::{
    runtime_plugin_placement_decision, runtime_plugin_sandbox_snapshot,
};
use sandbox_state::RuntimePluginLifecyclePolicy;
pub(super) use sandbox_state::RuntimePluginSandboxStateModel;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginLifecycleStateModel {
    policy: RuntimePluginLifecyclePolicy,
    sandboxes: BTreeMap<String, RuntimePluginSandboxStateModel>,
    active_sandbox_count: u32,
}

impl RuntimePluginLifecycleStateModel {
    fn sandbox_mut(&mut self, sandbox_id: &str) -> &mut RuntimePluginSandboxStateModel {
        self.sandboxes
            .entry(sandbox_id.to_string())
            .or_insert_with(|| RuntimePluginSandboxStateModel::new(sandbox_id.to_string()))
    }

    pub(crate) fn record_spec(&mut self, spec: &PluginSandboxSpec) {
        let sandbox = self.sandbox_mut(spec.sandbox_id.as_str());
        sandbox.plugin_format = Some(spec.plugin_format);
        sandbox.plugin_type_id = spec.plugin_type_id.clone();
    }

    pub(crate) fn shared_boundary_member_ids(
        &self,
        policy: &RuntimePluginPlacementPolicy,
        sandbox_id: &str,
    ) -> Vec<String> {
        let Some(target) = self.sandboxes.get(sandbox_id) else {
            return vec![sandbox_id.to_string()];
        };
        let placement = runtime_plugin_placement_decision(target, policy);
        if placement.outcome != RuntimePluginIsolationOutcome::SharedSandbox {
            return vec![sandbox_id.to_string()];
        }
        self.sandboxes
            .values()
            .filter(|sandbox| {
                let candidate = runtime_plugin_placement_decision(sandbox, policy);
                candidate.outcome == RuntimePluginIsolationOutcome::SharedSandbox
                    && candidate.sandbox_group_key == placement.sandbox_group_key
            })
            .map(|sandbox| sandbox.sandbox_id.clone())
            .collect()
    }

    pub(crate) fn set_active_sandbox_count(&mut self, count: u32) {
        self.active_sandbox_count = count;
    }

    pub(crate) fn record_lv2_prepared_negotiation(
        &mut self,
        sandbox_id: &str,
        negotiation: RuntimeLv2PreparedNegotiationRecord,
    ) {
        self.sandbox_mut(sandbox_id).lv2_prepared_negotiation = Some(negotiation);
    }
}
