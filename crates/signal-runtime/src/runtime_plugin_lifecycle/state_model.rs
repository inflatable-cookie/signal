use super::*;
use crate::runtime::runtime_plugin_recording::runtime_plugin_parity_coverage;
#[path = "state_model/sandbox_state.rs"]
mod sandbox_state;
#[path = "state_model/snapshot.rs"]
mod snapshot;
#[path = "state_model/transitions.rs"]
mod transitions;

use super::placement::runtime_plugin_sandbox_snapshot;
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

    pub(crate) fn record_preset_descriptor(
        &mut self,
        sandbox_id: &str,
        descriptor: RuntimePluginPresetDescriptor,
    ) {
        self.sandbox_mut(sandbox_id).preset_descriptor = Some(descriptor);
    }

    pub(crate) fn record_ara_context(
        &mut self,
        sandbox_id: &str,
        context: RuntimePluginAraContextSnapshot,
    ) {
        self.sandbox_mut(sandbox_id).ara_context = Some(context);
    }

    pub(crate) fn set_active_sandbox_count(&mut self, count: u32) {
        self.active_sandbox_count = count;
    }
}
