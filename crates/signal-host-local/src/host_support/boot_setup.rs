use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeConfigRequest, RuntimeError, RuntimeLifecycleApi,
    RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::LocalRuntimeHost;
use super::{local_demo_runtime_assembly, LocalRuntimeHostSummary};

impl LocalRuntimeHost {
    /// Boots the local host: handshake, configure, graph projection,
    /// hardware negotiation, plugin scan over explicitly configured roots
    /// (empty by default — no system plugin directories are touched), and
    /// optional sandbox sessions for explicitly configured fixture plugins.
    ///
    /// Production audio playback lives in `signal-render-plane`; this host is
    /// a control/observation surface. The stream state reported after boot
    /// means "an output stream was negotiated with real hardware", not that
    /// this process is pumping audio callbacks.
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
        self.stream_state = super::LocalAudioStreamState::Running;

        Ok(self.summarize_boot_outcome(&hardware_stream))
    }
}
