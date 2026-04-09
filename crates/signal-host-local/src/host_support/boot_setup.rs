use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    HandshakeRequest, PluginScanRequest, RuntimeConfigRequest, RuntimeError, RuntimeLifecycleApi,
    RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{FaultInjection, LocalRuntimeHost};
use super::{local_demo_runtime_assembly, LocalRuntimeHostSummary, STEADY_STATE_BLOCKS};

impl LocalRuntimeHost {
    pub(crate) fn boot_with_fault_recovery(
        &mut self,
        fault: Option<FaultInjection>,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
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
        self.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())?;
        self.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        let sandbox = assembly.primary_sandbox();

        self.runtime.set_cpu_load_percent(4.5);
        self.runtime.set_graph_latency_ms(2.7);
        self.runtime.start()?;

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:default",
            "instance:local:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let mut run = self.run_lifecycle(
            &protocol,
            sandbox.request.sandbox_id.as_str(),
            1,
            &mut lifecycle,
        )?;
        let executed_steady_state_tail = if let Some(fault) = fault {
            let executed_steady_state_tail = self.apply_boot_fault_recovery(
                &protocol,
                sandbox,
                &mut run,
                &mut lifecycle,
                fault,
            )?;
            if !executed_steady_state_tail {
                self.execute_block_sequence(
                    &protocol,
                    &mut run,
                    STEADY_STATE_BLOCKS,
                    &mut lifecycle,
                    false,
                )?;
            }
            executed_steady_state_tail
        } else {
            self.execute_block_sequence(
                &protocol,
                &mut run,
                STEADY_STATE_BLOCKS,
                &mut lifecycle,
                false,
            )?;
            false
        };
        let _ = executed_steady_state_tail;
        Ok(self.summarize_boot_outcome(
            &hardware_stream,
            &sandbox.request.sandbox_id,
            &protocol,
            run,
        ))
    }
}
