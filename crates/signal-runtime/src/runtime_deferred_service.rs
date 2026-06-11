use super::*;

#[path = "runtime_deferred_service/offline_render.rs"]
mod offline_render;
#[path = "runtime_deferred_service/plugin_bindings.rs"]
mod plugin_bindings;

fn deferred_service_interruption_class(
    decision: RuntimeDeferredServiceDecision,
) -> RuntimeInterruptionClass {
    match decision {
        RuntimeDeferredServiceDecision::Run => RuntimeInterruptionClass::Steady,
        RuntimeDeferredServiceDecision::Defer | RuntimeDeferredServiceDecision::Throttle => {
            RuntimeInterruptionClass::Resumable
        }
        RuntimeDeferredServiceDecision::Abort => RuntimeInterruptionClass::Terminal,
    }
}

fn deferred_service_priority_band(
    work_class: RuntimeDeferredServiceClass,
) -> RuntimeDeferredServicePriorityBand {
    match work_class {
        RuntimeDeferredServiceClass::OfflineRenderQueue => {
            RuntimeDeferredServicePriorityBand::UserVisible
        }
        RuntimeDeferredServiceClass::OfflineRenderPurge => {
            RuntimeDeferredServicePriorityBand::Maintenance
        }
    }
}

fn deferred_service_blocking_priority_band(
    reason: RuntimeDeferredServiceReason,
) -> Option<RuntimeDeferredServicePriorityBand> {
    match reason {
        RuntimeDeferredServiceReason::RealtimeActive => {
            Some(RuntimeDeferredServicePriorityBand::RealtimeCritical)
        }
        RuntimeDeferredServiceReason::PendingCleanup
        | RuntimeDeferredServiceReason::RecoveryDegraded
        | RuntimeDeferredServiceReason::SafeMode => {
            Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
        }
        RuntimeDeferredServiceReason::Ready | RuntimeDeferredServiceReason::InvalidRequest => None,
    }
}

fn deferred_service_backpressure_source(
    reason: RuntimeDeferredServiceReason,
) -> Option<RuntimeDeferredServiceBackpressureSource> {
    match reason {
        RuntimeDeferredServiceReason::RealtimeActive => {
            Some(RuntimeDeferredServiceBackpressureSource::RealtimeAudio)
        }
        RuntimeDeferredServiceReason::PendingCleanup => {
            Some(RuntimeDeferredServiceBackpressureSource::CleanupBacklog)
        }
        RuntimeDeferredServiceReason::RecoveryDegraded => {
            Some(RuntimeDeferredServiceBackpressureSource::RecoveryOverlap)
        }
        RuntimeDeferredServiceReason::SafeMode => {
            Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
        }
        RuntimeDeferredServiceReason::Ready | RuntimeDeferredServiceReason::InvalidRequest => None,
    }
}

fn deferred_service_cancellation_cause(
    decision: RuntimeDeferredServiceDecision,
    reason: RuntimeDeferredServiceReason,
) -> Option<RuntimeDeferredServiceCancellationCause> {
    match (decision, reason) {
        (RuntimeDeferredServiceDecision::Abort, RuntimeDeferredServiceReason::InvalidRequest) => {
            Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
        }
        _ => None,
    }
}

pub(super) struct RuntimeDeferredServiceReceiptInput {
    pub(super) work_class: RuntimeDeferredServiceClass,
    pub(super) decision: RuntimeDeferredServiceDecision,
    pub(super) reason: RuntimeDeferredServiceReason,
    pub(super) queued_work_item_count: usize,
    pub(super) admitted_work_item_count: usize,
    pub(super) runtime_running: bool,
    pub(super) safe_mode_enabled: bool,
    pub(super) readiness_degraded: bool,
    pub(super) pending_cleanup_work_items: usize,
    pub(super) pending_deferred_retry_work_items: usize,
    pub(super) recovery_overlap_session_count: usize,
}

pub(super) fn deferred_service_receipt(
    input: RuntimeDeferredServiceReceiptInput,
) -> RuntimeDeferredServiceReceipt {
    let priority_band = deferred_service_priority_band(input.work_class);
    let deferred_work_item_count = input
        .queued_work_item_count
        .saturating_sub(input.admitted_work_item_count);
    let starved_work_item_count = match input.decision {
        RuntimeDeferredServiceDecision::Defer | RuntimeDeferredServiceDecision::Throttle => {
            deferred_work_item_count
        }
        RuntimeDeferredServiceDecision::Run | RuntimeDeferredServiceDecision::Abort => 0,
    };
    let cancelled_work_item_count = match input.decision {
        RuntimeDeferredServiceDecision::Abort => input.queued_work_item_count,
        RuntimeDeferredServiceDecision::Run
        | RuntimeDeferredServiceDecision::Defer
        | RuntimeDeferredServiceDecision::Throttle => 0,
    };
    let receipt = RuntimeDeferredServiceReceipt {
        work_class: input.work_class,
        decision: input.decision,
        reason: input.reason,
        priority_band,
        blocking_priority_band: deferred_service_blocking_priority_band(input.reason),
        backpressure_source: deferred_service_backpressure_source(input.reason),
        starvation_risk: starved_work_item_count > 0,
        starved_work_item_count,
        cancellation_cause: deferred_service_cancellation_cause(input.decision, input.reason),
        cancelled_work_item_count,
        interruption_class: deferred_service_interruption_class(input.decision),
        interruption_rebindable: false,
        queued_work_item_count: input.queued_work_item_count,
        admitted_work_item_count: input.admitted_work_item_count,
        completed_work_item_count: 0,
        deferred_work_item_count,
        runtime_running: input.runtime_running,
        safe_mode_enabled: input.safe_mode_enabled,
        readiness_degraded: input.readiness_degraded,
        pending_cleanup_work_items: input.pending_cleanup_work_items,
        pending_deferred_retry_work_items: input.pending_deferred_retry_work_items,
        recovery_overlap_session_count: input.recovery_overlap_session_count,
    };
    receipt
}
