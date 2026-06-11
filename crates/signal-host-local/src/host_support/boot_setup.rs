use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeConfigRequest, RuntimeError, RuntimeLifecycleApi,
    RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::LocalRuntimeHost;
use super::{local_demo_runtime_assembly, LocalRuntimeHostSummary, STEADY_STATE_BLOCKS};

impl LocalRuntimeHost {
    /// Boots the local host: handshake, configure, graph projection,
    /// hardware negotiation, plugin scan over explicitly configured roots
    /// (empty by default — no system plugin directories are touched),
    /// optional sandbox sessions for explicitly configured fixture plugins,
    /// and a bounded run of engine blocks through the output pump.
    pub(crate) fn boot_local(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        let runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;
        let assembly = local_demo_runtime_assembly();
        self.runtime
            .apply_graph_projection(assembly.graph.clone())?;
        self.runtime
            .apply_graph_contract_projection(assembly.graph_contracts.clone())?;

        let hardware_stream = self.prepare_default_output_hardware()?;

        self.start_plugin_scan(PluginScanRequest {
            roots: assembly.scan_roots.clone(),
            formats: assembly.scan_formats.clone(),
        })?;

        for sandbox in &assembly.plugin_sandboxes {
            self.ensure_plugin_sandbox(sandbox.spec())?;
        }
        if !assembly.plugin_sandboxes.is_empty() {
            self.runtime
                .apply_plugin_backed_node_bindings(assembly.plugin_bindings())?;
        }
        self.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());

        self.runtime.start()?;

        let processing_epoch = 1;
        let mut last_engine_result = None;
        for _ in 0..STEADY_STATE_BLOCKS {
            let block_sequence = self.runtime.allocate_block_sequence();
            let result =
                self.process_engine_block_through_output_pump(processing_epoch, block_sequence)?;
            last_engine_result = Some((block_sequence, result));
        }

        Ok(self.summarize_boot_outcome(&hardware_stream, last_engine_result))
    }
}
