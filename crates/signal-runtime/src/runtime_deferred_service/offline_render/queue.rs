use super::*;

impl SignalRuntime {
    /// Renders a batch of offline requests in sequence, deferring any that exceed the admission limit.
    pub fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError> {
        if requests.is_empty() {
            self.record_deferred_service_receipt(deferred_service_receipt(
                RuntimeDeferredServiceReceiptInput {
                    work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
                    decision: RuntimeDeferredServiceDecision::Abort,
                    reason: RuntimeDeferredServiceReason::InvalidRequest,
                    queued_work_item_count: 0,
                    admitted_work_item_count: 0,
                    runtime_running: self.control.running,
                    safe_mode_enabled: self.safe_mode_enabled,
                    readiness_degraded: matches!(self.readiness, RuntimeReadiness::Degraded { .. }),
                    pending_cleanup_work_items: self
                        .transport_concurrency
                        .pending_work_item_count(),
                    pending_deferred_retry_work_items: self
                        .transport_concurrency
                        .pending_deferred_retry_work_count(),
                    recovery_overlap_session_count: self
                        .transport_concurrency
                        .recovery_overlap_session_count(),
                },
            ));
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render queue requires at least one request",
            ));
        }
        let queue_count = requests.len();
        let mut orchestration = self.offline_render_queue_receipt(queue_count);
        let mut progress = Vec::with_capacity(orchestration.admitted_work_item_count);
        let mut results = Vec::with_capacity(orchestration.admitted_work_item_count);
        let mut deferred_requests = Vec::with_capacity(orchestration.deferred_work_item_count);
        for (queue_index, request) in requests.into_iter().enumerate() {
            if queue_index >= orchestration.admitted_work_item_count {
                deferred_requests.push(request);
                continue;
            }
            let result = self.render_offline(request)?;
            let completed_job_count = queue_index.saturating_add(1);
            let progress_percent = ((completed_job_count * 100) / queue_count).clamp(0, 100) as u8;
            progress.push(RuntimeOfflineRenderQueueProgressReceipt {
                request_id: result.request_id.clone(),
                queue_index,
                queue_count,
                completed_job_count,
                progress_percent,
                summary: format!(
                    "request={} queue={}/{} progress={}% blocks={} rendered_frames={}",
                    result.request_id,
                    completed_job_count,
                    queue_count,
                    progress_percent,
                    result.block_count,
                    result.rendered_frame_count,
                ),
            });
            results.push(result);
        }
        orchestration.completed_work_item_count = results.len();
        orchestration.deferred_work_item_count = deferred_requests.len();
        orchestration.summary = summarize_deferred_service_receipt(&orchestration);
        let completed_job_count = progress.len();
        let deferred_job_count = deferred_requests.len();
        let decision = orchestration.decision;
        let summary = format!(
            "queue_count={} completed_job_count={} deferred_job_count={} decision={:?}",
            queue_count, completed_job_count, deferred_job_count, decision
        );
        if let Some(last_result) = results.last() {
            self.record_last_offline_render_session_snapshot(
                crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot {
                    request_id: last_result.request_id.clone(),
                    state: RuntimeOfflineRenderExecutionState::Completed,
                    interruption_class: RuntimeInterruptionClass::Steady,
                    interruption_rebindable: false,
                    interruption_count: 0,
                    emitted_checkpoint_count: 0,
                    checkpoint_count: 0,
                    rendered_frame_count: last_result.runtime_frame_count,
                    total_frame_count: last_result.runtime_frame_count,
                    rendered_block_count: last_result.block_count,
                    total_block_count: last_result.block_count,
                    artifact_root_path: last_result.manifest.artifact_root_path.clone(),
                    report_path: last_result
                        .manifest
                        .report
                        .as_ref()
                        .map(|report| report.report_path.clone()),
                    materialized: true,
                    artifact_count: last_result.manifest.artifact_count,
                    report_materialized: last_result.manifest.report.is_some(),
                    active_checkpoint: None,
                    last_checkpoint: None,
                    summary: format!(
                        "request={} state=completed queue_result=true artifacts={} report={}",
                        last_result.request_id,
                        last_result.manifest.artifact_count,
                        last_result.manifest.report.is_some(),
                    ),
                },
            );
        }
        self.record_deferred_service_receipt(orchestration.clone());
        Ok(RuntimeOfflineRenderQueueResult {
            queue_count,
            completed_job_count,
            orchestration,
            progress,
            results,
            deferred_requests,
            summary,
        })
    }
}
