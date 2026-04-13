use signal_runtime::{LingeringCleanupMode, RuntimeError, RuntimeObservationApi};

use super::super::LocalRuntimeHost;
use super::LifecycleRunSummary;

impl LocalRuntimeHost {
    pub(crate) fn session_is_lingering(
        &self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> bool {
        self.runtime
            .get_transport_concurrency_snapshot()
            .active_sessions
            .iter()
            .find(|session| {
                session.sandbox_id == sandbox_id
                    && session.lease_id == lease_id
                    && session.region_id == region_id
            })
            .is_some_and(|session| {
                matches!(
                    session.state,
                    signal_runtime::TransportSessionState::DetachRequested
                        | signal_runtime::TransportSessionState::DetachFaulted
                )
            })
    }

    pub(crate) fn cleanup_orphan_lingering_sessions_for_sandbox(
        &mut self,
        sandbox_id: &str,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
    ) -> Result<(), RuntimeError> {
        let trigger = match mode {
            LingeringCleanupMode::StrictPreAttach => {
                signal_runtime::LingeringCleanupTrigger::RecoveryPreAttach
            }
            LingeringCleanupMode::BestEffortPostStart => {
                signal_runtime::LingeringCleanupTrigger::PostStartReconciliation
            }
        };
        let _ = self.runtime.enqueue_lingering_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            processing_epoch,
            exclude_lease_id,
            exclude_region_id,
        );
        while let Some(plan) = self
            .runtime
            .dequeue_lingering_cleanup_work_for_sandbox(sandbox_id, processing_epoch)
        {
            for session in plan.candidates {
                if let Err(error) =
                    self.cleanup_orphan_lingering_transport(&session, plan.processing_epoch)
                {
                    self.runtime.record_lingering_cleanup_failure(
                        session.sandbox_id.as_str(),
                        session.lease_id.as_str(),
                        session.region_id.as_str(),
                        plan.mode,
                        plan.processing_epoch,
                        error.message.as_str(),
                    );
                    if plan.mode == LingeringCleanupMode::StrictPreAttach {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn cleanup_exact_lingering_session_if_present(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        processing_epoch: u64,
    ) {
        let lingering_session = self
            .runtime
            .get_transport_concurrency_snapshot()
            .active_sessions
            .into_iter()
            .find(|session| {
                session.sandbox_id == sandbox_id
                    && session.lease_id == lease_id
                    && session.region_id == region_id
                    && matches!(
                        session.state,
                        signal_runtime::TransportSessionState::DetachRequested
                            | signal_runtime::TransportSessionState::DetachFaulted
                    )
            });

        if let Some(session) = lingering_session {
            let _ = self.cleanup_orphan_lingering_transport(&session, processing_epoch);
        }
    }

    pub(crate) fn reconcile_late_lingering_sessions_after_start(
        &mut self,
        sandbox_id: &str,
        active_run: &LifecycleRunSummary,
    ) {
        let _ = self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            active_run.processing_epoch,
            Some(active_run.shared_memory_lease_id.as_str()),
            active_run
                .transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
            LingeringCleanupMode::BestEffortPostStart,
        );
    }
}
