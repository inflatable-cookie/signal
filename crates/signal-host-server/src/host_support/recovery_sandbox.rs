use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BrokerInvalidationStage, CompletionSlotStage, RecoveryRestartIntent, RuntimeError, StopReason,
};

use super::super::{RecoveryFailureInjection, ServerRuntimeHost};
use super::LifecycleRunSummary;

impl ServerRuntimeHost {
    pub(crate) fn recover_sandbox(
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
                super::LingeringSessionRecovery {
                    sandbox_id,
                    lifecycle,
                    run,
                    prior_history,
                    next_epoch,
                    failure,
                },
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
        let replacement_run = match self.prepare_replacement_recovery_session(
            protocol,
            sandbox_id,
            run,
            next_epoch,
            failure,
            &mut replacement_lifecycle,
        ) {
            Ok(run) => run,
            Err(contention_error)
                if matches!(
                    failure,
                    Some(RecoveryFailureInjection::CompetingOverlapAttach)
                ) =>
            {
                self.abort_origin_recovery_session(protocol, sandbox_id, lifecycle, run);
                self.runtime.set_active_plugin_sandboxes(0);
                return Err(contention_error);
            }
            Err(error) => return Err(error),
        };
        self.finish_recovery_overlap_transition(
            protocol,
            super::RecoveryOverlapTransition {
                sandbox_id,
                lifecycle,
                run,
                failure,
                replacement_lifecycle: &mut replacement_lifecycle,
                replacement_run: &replacement_run,
            },
        )?;
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &replacement_run);
        *lifecycle = replacement_lifecycle;
        Ok(replacement_run)
    }
}
