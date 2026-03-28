use super::*;
#[path = "offline_render/purge.rs"]
mod purge;
#[path = "offline_render/queue.rs"]
mod queue;

impl SignalRuntime {
    pub(crate) fn record_deferred_service_receipt(&self, receipt: RuntimeDeferredServiceReceipt) {
        self.last_deferred_service_receipt.replace(Some(receipt));
    }

    pub(crate) fn offline_render_queue_receipt(
        &self,
        queue_count: usize,
    ) -> RuntimeDeferredServiceReceipt {
        let transport_concurrency = self.transport_concurrency.snapshot();
        let readiness_degraded = matches!(self.readiness, RuntimeReadiness::Degraded { .. });
        let (decision, reason, admitted_work_item_count) = if self.safe_mode_enabled {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::SafeMode,
                0,
            )
        } else if readiness_degraded || transport_concurrency.current_recovery_overlap_sessions > 0
        {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::RecoveryDegraded,
                0,
            )
        } else if transport_concurrency.pending_cleanup_work_items > 0
            || transport_concurrency.pending_deferred_retry_work_items > 0
        {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::PendingCleanup,
                0,
            )
        } else if self.control.running {
            (
                RuntimeDeferredServiceDecision::Throttle,
                RuntimeDeferredServiceReason::RealtimeActive,
                queue_count.min(1),
            )
        } else {
            (
                RuntimeDeferredServiceDecision::Run,
                RuntimeDeferredServiceReason::Ready,
                queue_count,
            )
        };
        deferred_service_receipt(RuntimeDeferredServiceReceiptInput {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision,
            reason,
            queued_work_item_count: queue_count,
            admitted_work_item_count,
            runtime_running: self.control.running,
            safe_mode_enabled: self.safe_mode_enabled,
            readiness_degraded,
            pending_cleanup_work_items: transport_concurrency.pending_cleanup_work_items,
            pending_deferred_retry_work_items: transport_concurrency
                .pending_deferred_retry_work_items,
            recovery_overlap_session_count: transport_concurrency.current_recovery_overlap_sessions,
        })
    }

    pub(crate) fn offline_render_purge_receipt(&self) -> RuntimeDeferredServiceReceipt {
        let transport_concurrency = self.transport_concurrency.snapshot();
        let readiness_degraded = matches!(self.readiness, RuntimeReadiness::Degraded { .. });
        let (decision, reason, admitted_work_item_count) = if self.safe_mode_enabled {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::SafeMode,
                0,
            )
        } else if readiness_degraded || transport_concurrency.current_recovery_overlap_sessions > 0
        {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::RecoveryDegraded,
                0,
            )
        } else if transport_concurrency.pending_cleanup_work_items > 0
            || transport_concurrency.pending_deferred_retry_work_items > 0
        {
            (
                RuntimeDeferredServiceDecision::Defer,
                RuntimeDeferredServiceReason::PendingCleanup,
                0,
            )
        } else {
            (
                RuntimeDeferredServiceDecision::Run,
                RuntimeDeferredServiceReason::Ready,
                1,
            )
        };
        deferred_service_receipt(RuntimeDeferredServiceReceiptInput {
            work_class: RuntimeDeferredServiceClass::OfflineRenderPurge,
            decision,
            reason,
            queued_work_item_count: 1,
            admitted_work_item_count,
            runtime_running: self.control.running,
            safe_mode_enabled: self.safe_mode_enabled,
            readiness_degraded,
            pending_cleanup_work_items: transport_concurrency.pending_cleanup_work_items,
            pending_deferred_retry_work_items: transport_concurrency
                .pending_deferred_retry_work_items,
            recovery_overlap_session_count: transport_concurrency.current_recovery_overlap_sessions,
        })
    }
}
