use super::*;

#[path = "runtime_deferred_service/offline_render.rs"]
mod offline_render;
#[path = "runtime_deferred_service/plugin_bindings.rs"]
mod plugin_bindings;

pub(super) fn summarize_deferred_service_receipt(
    receipt: &RuntimeDeferredServiceReceipt,
) -> String {
    format!(
        "class={:?} decision={:?} reason={:?} priority={:?} blocking={:?} backpressure={:?} starvation={}/{} cancellation={:?}/{} interruption={:?}/rebindable={} queued={} admitted={} completed={} deferred={} running={} safe_mode={} degraded={} cleanup_pending={} deferred_retries={} recovery_overlap={}",
        receipt.work_class,
        receipt.decision,
        receipt.reason,
        receipt.priority_band,
        receipt.blocking_priority_band,
        receipt.backpressure_source,
        receipt.starvation_risk,
        receipt.starved_work_item_count,
        receipt.cancellation_cause,
        receipt.cancelled_work_item_count,
        receipt.interruption_class,
        receipt.interruption_rebindable,
        receipt.queued_work_item_count,
        receipt.admitted_work_item_count,
        receipt.completed_work_item_count,
        receipt.deferred_work_item_count,
        receipt.runtime_running,
        receipt.safe_mode_enabled,
        receipt.readiness_degraded,
        receipt.pending_cleanup_work_items,
        receipt.pending_deferred_retry_work_items,
        receipt.recovery_overlap_session_count,
    )
}

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

pub(super) fn deferred_service_receipt(
    work_class: RuntimeDeferredServiceClass,
    decision: RuntimeDeferredServiceDecision,
    reason: RuntimeDeferredServiceReason,
    queued_work_item_count: usize,
    admitted_work_item_count: usize,
    runtime_running: bool,
    safe_mode_enabled: bool,
    readiness_degraded: bool,
    pending_cleanup_work_items: usize,
    pending_deferred_retry_work_items: usize,
    recovery_overlap_session_count: usize,
) -> RuntimeDeferredServiceReceipt {
    let priority_band = deferred_service_priority_band(work_class);
    let deferred_work_item_count = queued_work_item_count.saturating_sub(admitted_work_item_count);
    let starved_work_item_count = match decision {
        RuntimeDeferredServiceDecision::Defer | RuntimeDeferredServiceDecision::Throttle => {
            deferred_work_item_count
        }
        RuntimeDeferredServiceDecision::Run | RuntimeDeferredServiceDecision::Abort => 0,
    };
    let cancelled_work_item_count = match decision {
        RuntimeDeferredServiceDecision::Abort => queued_work_item_count,
        RuntimeDeferredServiceDecision::Run
        | RuntimeDeferredServiceDecision::Defer
        | RuntimeDeferredServiceDecision::Throttle => 0,
    };
    let mut receipt = RuntimeDeferredServiceReceipt {
        work_class,
        decision,
        reason,
        priority_band,
        blocking_priority_band: deferred_service_blocking_priority_band(reason),
        backpressure_source: deferred_service_backpressure_source(reason),
        starvation_risk: starved_work_item_count > 0,
        starved_work_item_count,
        cancellation_cause: deferred_service_cancellation_cause(decision, reason),
        cancelled_work_item_count,
        interruption_class: deferred_service_interruption_class(decision),
        interruption_rebindable: false,
        queued_work_item_count,
        admitted_work_item_count,
        completed_work_item_count: 0,
        deferred_work_item_count,
        runtime_running,
        safe_mode_enabled,
        readiness_degraded,
        pending_cleanup_work_items,
        pending_deferred_retry_work_items,
        recovery_overlap_session_count,
        summary: String::new(),
    };
    receipt.summary = summarize_deferred_service_receipt(&receipt);
    receipt
}
