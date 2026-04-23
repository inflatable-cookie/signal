use super::super::*;
use super::RuntimeDeferredServiceReceipt;

impl RuntimeDeferredServiceReceipt {
    /// Renders this receipt as a multi-line key=value string for logging or diagnostics.
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "work_class={:?}",
                "\ndecision={:?}",
                "\nreason={:?}",
                "\npriority_band={:?}",
                "\nblocking_priority_band={:?}",
                "\nbackpressure_source={:?}",
                "\nstarvation_risk={}",
                "\nstarved_work_item_count={}",
                "\ncancellation_cause={:?}",
                "\ncancelled_work_item_count={}",
                "\ninterruption_class={:?}",
                "\ninterruption_rebindable={}",
                "\nqueued_work_item_count={}",
                "\nadmitted_work_item_count={}",
                "\ncompleted_work_item_count={}",
                "\ndeferred_work_item_count={}",
                "\nruntime_running={}",
                "\nsafe_mode_enabled={}",
                "\nreadiness_degraded={}",
                "\npending_cleanup_work_items={}",
                "\npending_deferred_retry_work_items={}",
                "\nrecovery_overlap_session_count={}",
                "\nsummary={}",
            ),
            self.work_class,
            self.decision,
            self.reason,
            self.priority_band,
            self.blocking_priority_band,
            self.backpressure_source,
            self.starvation_risk,
            self.starved_work_item_count,
            self.cancellation_cause,
            self.cancelled_work_item_count,
            self.interruption_class,
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            self.summary,
        )
    }

    /// Renders this receipt as a JSON object string.
    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"work_class\":{},",
                "\"decision\":{},",
                "\"reason\":{},",
                "\"priority_band\":{},",
                "\"blocking_priority_band\":{},",
                "\"backpressure_source\":{},",
                "\"starvation_risk\":{},",
                "\"starved_work_item_count\":{},",
                "\"cancellation_cause\":{},",
                "\"cancelled_work_item_count\":{},",
                "\"interruption_class\":{},",
                "\"interruption_rebindable\":{},",
                "\"queued_work_item_count\":{},",
                "\"admitted_work_item_count\":{},",
                "\"completed_work_item_count\":{},",
                "\"deferred_work_item_count\":{},",
                "\"runtime_running\":{},",
                "\"safe_mode_enabled\":{},",
                "\"readiness_degraded\":{},",
                "\"pending_cleanup_work_items\":{},",
                "\"pending_deferred_retry_work_items\":{},",
                "\"recovery_overlap_session_count\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&format!("{:?}", self.work_class)),
            json_string(&format!("{:?}", self.decision)),
            json_string(&format!("{:?}", self.reason)),
            json_string(&format!("{:?}", self.priority_band)),
            json_option_string(
                self.blocking_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.backpressure_source
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.starvation_risk,
            self.starved_work_item_count,
            json_option_string(
                self.cancellation_cause
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.cancelled_work_item_count,
            json_string(&format!("{:?}", self.interruption_class)),
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl Default for RuntimeDeferredServiceReceipt {
    fn default() -> Self {
        Self {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision: RuntimeDeferredServiceDecision::Abort,
            reason: RuntimeDeferredServiceReason::InvalidRequest,
            priority_band: RuntimeDeferredServicePriorityBand::UserVisible,
            blocking_priority_band: None,
            backpressure_source: None,
            starvation_risk: false,
            starved_work_item_count: 0,
            cancellation_cause: Some(RuntimeDeferredServiceCancellationCause::InvalidRequest),
            cancelled_work_item_count: 0,
            interruption_class: RuntimeInterruptionClass::Terminal,
            interruption_rebindable: false,
            queued_work_item_count: 0,
            admitted_work_item_count: 0,
            completed_work_item_count: 0,
            deferred_work_item_count: 0,
            runtime_running: false,
            safe_mode_enabled: false,
            readiness_degraded: false,
            pending_cleanup_work_items: 0,
            pending_deferred_retry_work_items: 0,
            recovery_overlap_session_count: 0,
            summary: "class=OfflineRenderQueue decision=Abort reason=InvalidRequest queued=0 admitted=0 completed=0 deferred=0".to_string(),
        }
    }
}
