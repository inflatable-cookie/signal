use super::*;
#[path = "control/advance.rs"]
mod advance;

impl SignalRuntime {
    pub fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        let (snapshot, receipt) = {
            let session = self
                .offline_render_executions
                .get_mut(request_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!("offline render execution `{request_id}` is not active"),
                    )
                })?;
            session.state = RuntimeOfflineRenderExecutionState::Paused;
            session.interruption_class_override = None;
            session.last_state_summary = format!(
                "request={} state=paused checkpoints={}/{} rendered_frames={} rendered_blocks={}",
                request_id,
                session.emitted_checkpoint_count,
                session.checkpoint_count,
                session.rendered_frames,
                session.block_count
            );
            (
                Self::offline_render_session_state_snapshot(session),
                Self::offline_render_execution_status_receipt(session),
            )
        };
        self.record_last_offline_render_session_snapshot(snapshot);
        Ok(receipt)
    }

    pub fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        let (snapshot, receipt) = {
            let session = self
                .offline_render_executions
                .get_mut(request_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!("offline render execution `{request_id}` is not active"),
                    )
                })?;
            session.state = RuntimeOfflineRenderExecutionState::Running;
            session.interruption_class_override = None;
            session.last_state_summary = format!(
                "request={} state=running checkpoints={}/{} interruption_count={}",
                request_id,
                session.emitted_checkpoint_count,
                session.checkpoint_count,
                session.interruption_count
            );
            (
                Self::offline_render_session_state_snapshot(session),
                Self::offline_render_execution_status_receipt(session),
            )
        };
        self.record_last_offline_render_session_snapshot(snapshot);
        Ok(receipt)
    }

    pub fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        let reason = reason.trim().to_string();
        if reason.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render execution interruption reason cannot be empty",
            ));
        }
        let (snapshot, receipt) = {
            let session = self
                .offline_render_executions
                .get_mut(request_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!("offline render execution `{request_id}` is not active"),
                    )
                })?;
            session.state = RuntimeOfflineRenderExecutionState::Recoverable;
            session.interruption_count = session.interruption_count.saturating_add(1);
            session.interruption_class_override = None;
            session.last_state_summary = format!(
                "request={} state=recoverable checkpoints={}/{} interruptions={} reason={}",
                request_id,
                session.emitted_checkpoint_count,
                session.checkpoint_count,
                session.interruption_count,
                reason
            );
            (
                Self::offline_render_session_state_snapshot(session),
                Self::offline_render_execution_status_receipt(session),
            )
        };
        self.record_last_offline_render_session_snapshot(snapshot);
        Ok(receipt)
    }

    pub fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError> {
        let session = self
            .offline_render_executions
            .remove(request_id)
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("offline render execution `{request_id}` is not active"),
                )
            })?;
        let receipt = RuntimeOfflineRenderExecutionCancellationReceipt {
            request_id: request_id.to_string(),
            cancelled_after_checkpoint_count: session.emitted_checkpoint_count,
            checkpoint_count: session.checkpoint_count,
            rendered_frame_count: session.rendered_frames,
            rendered_block_count: session.block_count,
            summary: format!(
                "request={} state=cancelled checkpoints={}/{} rendered_frames={} rendered_blocks={}",
                request_id,
                session.emitted_checkpoint_count,
                session.checkpoint_count,
                session.rendered_frames,
                session.block_count
            ),
        };
        self.last_offline_render_cancellation_receipt
            .replace(Some(receipt.clone()));
        let mut cancelled_session = session;
        cancelled_session.state = RuntimeOfflineRenderExecutionState::Cancelled;
        cancelled_session.interruption_class_override = None;
        cancelled_session.last_state_summary = receipt.summary.clone();
        self.record_last_offline_render_session_snapshot(
            Self::offline_render_session_state_snapshot(&cancelled_session),
        );
        Ok(receipt)
    }
}
