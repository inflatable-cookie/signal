use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    PluginSandboxLifecycleStage, PluginSandboxTransportStage, RecoveryRestartIntent, RuntimeError,
    StopReason,
};

use super::super::{LocalRuntimeHost, RecoveryFailureInjection};
use super::{lifecycle_stage_for_request, record_runtime_fault, runtime_error_from_failure, runtime_error_from_io, LifecycleRunSummary};

impl LocalRuntimeHost {
    fn recover_sandbox(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        intent: RecoveryRestartIntent,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        let current_transport = run.transport.clone().ok_or_else(|| {
            RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "lifecycle completed without brokered shared-memory transport",
            )
        })?;
        let prior_history = run.recovery_history();
        let next_epoch = run.processing_epoch.saturating_add(1);
        self.stop_runtime_for_recovery()?;
        self.supervisor.last_recovery_intent = Some(intent);
        self.supervisor.last_stop_reason = Some(StopReason::DegradedModeRecovery);
        self.runtime.record_recovery_cycle(
            sandbox_id,
            intent,
            StopReason::DegradedModeRecovery,
            Some(run.processing_epoch),
        );
        let (completion_invalidated, lease_invalidated) =
            lifecycle.invalidate_active_epoch(run.processing_epoch);
        let recovery_reason = match intent {
            RecoveryRestartIntent::CrashRecovery => "crash recovery teardown",
            RecoveryRestartIntent::WatchdogRecovery => "watchdog recovery teardown",
        };
        if completion_invalidated {
            self.runtime.record_completion_slot_transition(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                run.last_block_sequence,
                CompletionSlotStage::Invalidated,
            );
            self.runtime.record_broker_invalidation(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                Some(run.last_block_sequence),
                BrokerInvalidationStage::CompletionRegionInvalidated,
                recovery_reason,
            );
        }
        if lease_invalidated {
            self.runtime.record_broker_invalidation(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                Some(run.last_block_sequence),
                BrokerInvalidationStage::LeaseEpochInvalidated,
                recovery_reason,
            );
        }
        if failure != Some(RecoveryFailureInjection::CompetingOverlapAttach)
            && self.session_is_lingering(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            )
        {
            return self.recover_from_lingering_session(
                protocol,
                sandbox_id,
                lifecycle,
                run,
                prior_history,
                next_epoch,
                failure,
            );
        }
        self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            run.processing_epoch,
            Some(run.shared_memory_lease_id.as_str()),
            Some(current_transport.region_id.as_str()),
            signal_runtime::LingeringCleanupMode::StrictPreAttach,
        )?;
        let mut replacement_lifecycle = ClapSandboxLifecycleHarness::default();
        let mut replacement_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, &mut replacement_lifecycle)?;
        replacement_run.apply_recovery_history(prior_history);
        self.runtime.set_active_plugin_sandboxes(2);
        if matches!(
            failure,
            Some(RecoveryFailureInjection::CompetingOverlapAttach)
        ) {
            let mut competing_lifecycle = ClapSandboxLifecycleHarness::default();
            let contention_error = match self.run_lifecycle(
                protocol,
                sandbox_id,
                next_epoch.saturating_add(1),
                &mut competing_lifecycle,
            ) {
                Ok(competing_run) => {
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        &mut competing_lifecycle,
                        &competing_run,
                    );
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                        "expected overlapping replacement attach contention",
                    )
                }
                Err(error) => error,
            };
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.abort_origin_recovery_session(protocol, sandbox_id, lifecycle, run);
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(contention_error);
        }
        for request in protocol.teardown_sequence(sandbox_id, run.processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(run.processing_epoch),
                        );
                    }
                }
                Err(failure) => {
                    record_runtime_fault(&mut self.runtime, &failure);
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        &mut replacement_lifecycle,
                        &replacement_run,
                    );
                    self.runtime.set_active_plugin_sandboxes(0);
                    return Err(runtime_error_from_failure(&failure));
                }
            }
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if let Err(error) = self.teardown_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            None,
        );
        if matches!(
            failure,
            Some(RecoveryFailureInjection::DeferredOldTransportTeardown)
        ) {
            let error =
                std::io::Error::other("deferred old transport teardown during recovery retry");
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(1);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = self.broker.destroy_region(&current_transport) {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if matches!(
            failure,
            Some(RecoveryFailureInjection::OldTransportTeardown)
        ) {
            let error = std::io::Error::other(
                "injected old transport teardown failure during overlap recovery",
            );
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = lifecycle.teardown_active_transport() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(run.processing_epoch),
            None,
        );
        self.runtime.end_transport_session(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(run.processing_epoch),
        );
        if let Err(error) = self.restart_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if matches!(failure, Some(RecoveryFailureInjection::ReplacementStart)) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "injected replacement start failure during overlap recovery",
            ));
        }
        if let Err(error) = self.runtime.start() {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        if let Some(transport) = replacement_run.transport.as_ref() {
            self.runtime.promote_transport_session_to_steady_state(
                sandbox_id,
                replacement_run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
        }
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &replacement_run);
        *lifecycle = replacement_lifecycle;
        Ok(replacement_run)
    }
}
