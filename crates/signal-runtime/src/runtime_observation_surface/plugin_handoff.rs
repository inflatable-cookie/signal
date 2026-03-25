use super::*;

impl SignalRuntime {
    pub(crate) fn plugin_recall_handoff_snapshot(&self) -> RuntimePluginRecallHandoffSnapshot {
        RuntimePluginRecallHandoffSnapshot::from_plugin_chain_snapshot(
            &self.plugin_chain_snapshot(),
        )
    }
}
