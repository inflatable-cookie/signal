use crate::interfaces::{
    LingeringCleanupMode, LingeringCleanupPlan, LingeringCleanupQueueReceipt,
    LingeringCleanupTrigger, RuntimeError, RuntimeTransportConcurrencySnapshot,
    RuntimeTransportSessionAttachRequest, TransportAttachIntent,
};

use super::runtime_transport_concurrency::{
    RuntimeLingeringCleanupEnqueueRequest, RuntimeTransportSessionKey,
};
use super::runtime_utils::transport_session_provenance;
use super::SignalRuntime;

impl SignalRuntime {
    pub fn begin_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot =
            self.transport_concurrency
                .begin_session(RuntimeTransportSessionAttachRequest {
                    sandbox_id: sandbox_id.to_string(),
                    lease_id: lease_id.to_string(),
                    region_id: region_id.to_string(),
                    intent,
                    provenance: transport_session_provenance(intent),
                    attach_processing_epoch: None,
                    backing_path: None,
                    total_bytes: None,
                })?;
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
        self.begin_transport_session_with_metadata_for_epoch(RuntimeTransportSessionAttachRequest {
            sandbox_id: sandbox_id.to_string(),
            lease_id: lease_id.to_string(),
            region_id: region_id.to_string(),
            intent,
            provenance: transport_session_provenance(intent),
            attach_processing_epoch: None,
            backing_path,
            total_bytes,
        })
    }

    pub fn begin_transport_session_with_metadata_for_epoch(
        &mut self,
        request: RuntimeTransportSessionAttachRequest,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let attach_processing_epoch = request.attach_processing_epoch;
        let snapshot = self.transport_concurrency.begin_session(request)?;
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
        self.transport_concurrency
            .enqueue_cleanup_work(RuntimeLingeringCleanupEnqueueRequest {
                sandbox_id: sandbox_id.to_string(),
                mode,
                trigger,
                retry_count: 0,
                processing_epoch,
                cleanup_wave: None,
                exclude_session: match (exclude_lease_id, exclude_region_id) {
                    (Some(lease_id), Some(region_id)) => Some(
                        RuntimeTransportSessionKey::from_parts(sandbox_id, lease_id, region_id),
                    ),
                    _ => None,
                },
            })
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
            let session_key =
                RuntimeTransportSessionKey::from_parts(sandbox_id, lease_id, region_id);
            let retry_count = self
                .transport_concurrency
                .cleanup_attempt_count(sandbox_id, lease_id, region_id);
            let cleanup_wave = self
                .transport_concurrency
                .cleanup_wave_for_session(sandbox_id, lease_id, region_id);
            let _ = self.transport_concurrency.enqueue_cleanup_work(
                RuntimeLingeringCleanupEnqueueRequest {
                    sandbox_id: sandbox_id.to_string(),
                    mode,
                    trigger: LingeringCleanupTrigger::DeferredRetry,
                    retry_count,
                    processing_epoch,
                    cleanup_wave,
                    exclude_session: Some(session_key),
                },
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
