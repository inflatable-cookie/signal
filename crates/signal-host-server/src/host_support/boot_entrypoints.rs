use signal_hardware::HardwareConfigRequest;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BackendPolicyOverride, HandshakeRequest, PluginScanRequest, RuntimeConfigRequest, RuntimeError,
    RuntimeLifecycleApi, RuntimeProjectionApi, RuntimeSupervisorApi,
};

use super::super::{FaultInjection, ServerRuntimeHost, SOAK_RESTART_EPISODES, STEADY_STATE_BLOCKS};
use super::{server_demo_runtime_assembly, ServerRuntimeHostSummary};

impl ServerRuntimeHost {
    pub fn boot_default(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(None)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_timeout_recovery(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Timeout))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_crash_recovery(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Crash))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_heartbeat_miss_recovery(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::HeartbeatMiss))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_teardown_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_then_cleanup(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownThenCleanup))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_cleanup_retry(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownCleanupRetry))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_restart_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_overlap_contention(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryOverlapContention))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_interleaved_failures(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryInterleavedFailures))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_escalating_heartbeat_failures(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: 2,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_watchdog_soak(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_mixed_watchdog_soak(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    pub(crate) fn boot_with_fault_recovery(
        &mut self,
        fault: Option<FaultInjection>,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        let mut runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        runtime_config.anticipative_enabled = false;
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;
        let assembly = server_demo_runtime_assembly();
        self.runtime
            .apply_graph_projection(assembly.graph.clone())?;

        let hardware_request = HardwareConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
            signal_hardware::BackendPolicyTier::Tier0InHost,
        );
        self.runtime.apply_hardware_config(hardware_request)?;
        self.runtime
            .set_active_output_device("server:virtual-output");
        self.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })?;
        self.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);

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
        self.runtime.set_cpu_load_percent(1.2);
        self.runtime.set_graph_latency_ms(1.1);
        self.runtime.start()?;

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:server",
            "instance:server:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
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
        if let Some(fault) = fault {
            self.apply_boot_fault_recovery(&protocol, sandbox, &mut run, &mut lifecycle, fault)?;
        } else {
            self.execute_block_sequence(
                &protocol,
                &mut run,
                STEADY_STATE_BLOCKS,
                &mut lifecycle,
                false,
            )?;
        }
        Ok(self.summarize_boot_outcome(sandbox.request.sandbox_id.as_str(), &protocol, run))
    }
}
