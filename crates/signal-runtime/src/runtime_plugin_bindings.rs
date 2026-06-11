use super::*;

impl SignalRuntime {
    pub(crate) fn summarize_plugin_backed_bindings(&self) -> RuntimePluginBackedBindingSummary {
        let bound_sandbox_ids = self
            .engine
            .snapshot
            .planned_nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .filter_map(|node| node.plugin_sandbox_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let mut summary = RuntimePluginBackedBindingSummary {
            bound_sandbox_ids,
            ..RuntimePluginBackedBindingSummary::default()
        };
        for sandbox_id in &summary.bound_sandbox_ids {
            let matching_states = self
                .transport_concurrency
                .active_states_for_sandbox(sandbox_id);
            if matching_states
                .iter()
                .any(|state| matches!(state, TransportSessionState::AttachActive))
            {
                summary.active_bound_sandboxes += 1;
            } else if matching_states.iter().any(|state| {
                matches!(
                    state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            }) {
                summary.degraded_bound_sandboxes += 1;
            } else {
                summary.missing_bound_sandboxes += 1;
            }
        }
        summary
    }
}
