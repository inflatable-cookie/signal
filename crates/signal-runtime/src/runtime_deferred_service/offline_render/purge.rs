use super::*;

impl SignalRuntime {
    /// Purges the artifact directory and report file for a completed offline render request.
    pub fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError> {
        let request_id = request.request_id.trim().to_string();
        if request_id.is_empty() {
            self.record_deferred_service_receipt(deferred_service_receipt(
                RuntimeDeferredServiceReceiptInput {
                    work_class: RuntimeDeferredServiceClass::OfflineRenderPurge,
                    decision: RuntimeDeferredServiceDecision::Abort,
                    reason: RuntimeDeferredServiceReason::InvalidRequest,
                    queued_work_item_count: 1,
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
                "offline render artifact purge requires a request id",
            ));
        }
        let mut orchestration = self.offline_render_purge_receipt();
        if orchestration.decision != RuntimeDeferredServiceDecision::Run {
            self.record_deferred_service_receipt(orchestration.clone());
            let decision = orchestration.decision;
            let reason = orchestration.reason;
            let _summary = format!(
                "request={} deferred decision={:?} reason={:?}",
                request_id, decision, reason
            );
            let receipt = RuntimeOfflineRenderPurgeReceipt {
                request_id,
                orchestration,
                artifact_root_path: request.artifact_root_path,
                report_path: request.report_path,
                purged_artifact_root: false,
                purged_artifact_file_count: 0,
                purged_artifact_byte_count: 0,
                purged_report: false,
                purged_report_byte_count: 0,
            };
            self.last_offline_render_purge_receipt
                .replace(Some(receipt.clone()));
            return Ok(receipt);
        }
        let purged_report = request
            .report_path
            .as_deref()
            .map(Path::new)
            .map(purge_offline_render_file)
            .transpose()?
            .unwrap_or_default();
        let purged_artifact_root = request
            .artifact_root_path
            .as_deref()
            .map(Path::new)
            .map(purge_offline_render_directory)
            .transpose()?
            .unwrap_or_default();
        orchestration.completed_work_item_count = 1;
        orchestration.deferred_work_item_count = 0;
        self.record_deferred_service_receipt(orchestration.clone());
        let receipt = RuntimeOfflineRenderPurgeReceipt {
            request_id: request_id.clone(),
            orchestration,
            artifact_root_path: request.artifact_root_path,
            report_path: request.report_path,
            purged_artifact_root: purged_artifact_root.removed,
            purged_artifact_file_count: purged_artifact_root.file_count,
            purged_artifact_byte_count: purged_artifact_root.byte_count,
            purged_report: purged_report.removed,
            purged_report_byte_count: purged_report.byte_count,
        };
        self.last_offline_render_purge_receipt
            .replace(Some(receipt.clone()));
        Ok(receipt)
    }
}
