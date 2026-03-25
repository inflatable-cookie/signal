use crate::interfaces::{
    LingeringCleanupMode, LingeringCleanupPlan, LingeringCleanupQueueReceipt,
    LingeringCleanupTrigger, RuntimeError, RuntimeTransportConcurrencySnapshot,
    TransportAttachIntent, TransportSessionProvenance,
};

use super::{transport_session_provenance, SignalRuntime};

impl SignalRuntime {
    pub fn begin_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            transport_session_provenance(intent),
            None,
            None,
            None,
        )?;
        self.refresh_prework_service_policy_and_state(None);
        Ok(snapshot)
    }

    pub fn set_transport_session_limits(
        &mut self,
        steady_session_limit: usize,
        recovery_session_limit: usize,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self
            .transport_concurrency
            .set_policy(steady_session_limit, recovery_session_limit)?;
        self.refresh_prework_service_policy_and_state(None);
        Ok(snapshot)
    }

    pub fn begin_transport_session_with_metadata(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        self.begin_transport_session_with_metadata_for_epoch(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            None,
            transport_session_provenance(intent),
            backing_path,
            total_bytes,
        )
    }

    pub fn begin_transport_session_with_metadata_for_epoch(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        attach_processing_epoch: Option<u64>,
        provenance: TransportSessionProvenance,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            provenance,
            attach_processing_epoch,
            backing_path,
            total_bytes,
        )?;
        self.refresh_prework_service_policy_and_state(attach_processing_epoch);
        Ok(snapshot)
    }

    pub fn enqueue_lingering_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        self.transport_concurrency.enqueue_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            0,
            processing_epoch,
            None,
            exclude_lease_id,
            exclude_region_id,
        )
    }

    pub fn dequeue_lingering_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<LingeringCleanupPlan> {
        self.transport_concurrency
            .dequeue_cleanup_work_for_sandbox(sandbox_id, current_processing_epoch)
    }

    pub fn record_lingering_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency.record_cleanup_failure(
            sandbox_id,
            lease_id,
            region_id,
            mode,
            processing_epoch,
            error,
        );
        if matches!(mode, LingeringCleanupMode::BestEffortPostStart) {
            let retry_count = self
                .transport_concurrency
                .cleanup_attempt_count(sandbox_id, lease_id, region_id);
            let cleanup_wave = self
                .transport_concurrency
                .cleanup_wave_for_session(sandbox_id, lease_id, region_id);
            let _ = self.transport_concurrency.enqueue_cleanup_work(
                sandbox_id,
                mode,
                LingeringCleanupTrigger::DeferredRetry,
                retry_count,
                processing_epoch,
                cleanup_wave,
                Some(lease_id),
                Some(region_id),
            );
        }
        self.transport_concurrency.snapshot()
    }

    pub fn clear_lingering_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id)
    }

    pub fn complete_lingering_cleanup_success(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id);
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }

    pub fn end_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }

    pub fn promote_transport_session_to_steady_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        let snapshot = self
            .transport_concurrency
            .promote_session_to_steady_state(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }
}
