use super::*;
#[path = "advance/finalize.rs"]
mod finalize;
#[path = "advance/render_progress.rs"]
mod render_progress;

impl SignalRuntime {
    pub fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        let mut session = self
            .offline_render_executions
            .remove(request_id)
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("offline render execution `{request_id}` is not active"),
                )
            })?;

        if session.state != RuntimeOfflineRenderExecutionState::Running {
            let receipt = Self::offline_render_execution_status_receipt(&session);
            self.record_last_offline_render_session_snapshot(
                Self::offline_render_session_state_snapshot(&session),
            );
            self.offline_render_executions
                .insert(request_id.to_string(), session);
            return Ok(receipt);
        }

        if let Some(receipt) =
            self.advance_offline_render_execution_render_progress(request_id, &mut session)?
        {
            return Ok(receipt);
        }

        self.advance_offline_render_execution_finalize(request_id, session)
    }
}
