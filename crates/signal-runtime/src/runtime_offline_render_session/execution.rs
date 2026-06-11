use super::*;
#[path = "execution/internal_render.rs"]
mod internal_render;

impl SignalRuntime {
    /// Performs a synchronous offline render and returns the result.
    pub fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        self.render_offline_internal(request, false)
            .map(|execution| execution.result)
    }

    /// Performs a synchronous offline render and returns the result together with all emitted checkpoints.
    pub fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError> {
        self.render_offline_internal(request, true)
    }

    /// Registers a new offline render execution session and emits the first checkpoint.
    pub fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        if self
            .offline_render_executions
            .contains_key(request.request_id.as_str())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "offline render execution `{}` is already active",
                    request.request_id
                ),
            ));
        }
        let mut session = self.build_offline_render_execution_session(request)?;
        let checkpoint = Self::emit_offline_render_session_checkpoint(
            &mut session,
            RuntimeOfflineRenderCheckpointStage::PreparingInput,
            0,
            0,
            5,
        );
        let request_id = session.request.request_id.clone();
        let checkpoint_count = session.checkpoint_count;
        let emitted_checkpoint_count = session.emitted_checkpoint_count;
        self.offline_render_executions
            .insert(request_id.clone(), session);
        let (interruption_rebindable, session_snapshot) = self
            .offline_render_executions
            .get(request_id.as_str())
            .map(|session| {
                (
                    Self::offline_render_execution_interruption_rebindable(session),
                    Self::offline_render_session_state_snapshot(session),
                )
            })
            .unwrap_or((
                false,
                crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot {
                    request_id: request_id.clone(),
                    state: RuntimeOfflineRenderExecutionState::Running,
                    interruption_class: RuntimeInterruptionClass::Steady,
                    interruption_rebindable: false,
                    interruption_count: 0,
                    emitted_checkpoint_count,
                    checkpoint_count,
                    rendered_frame_count: 0,
                    total_frame_count: 0,
                    rendered_block_count: 0,
                    total_block_count: 0,
                    artifact_root_path: None,
                    report_path: None,
                    materialized: false,
                    artifact_count: 0,
                    report_materialized: false,
                    active_checkpoint: None,
                    last_checkpoint: None,
                },
            ));
        self.record_last_offline_render_session_snapshot(session_snapshot);
        Ok(RuntimeOfflineRenderExecutionProgressReceipt {
            request_id: request_id.clone(),
            state: RuntimeOfflineRenderExecutionState::Running,
            interruption_class: Self::offline_render_execution_interruption_class(
                RuntimeOfflineRenderExecutionState::Running,
            ),
            interruption_rebindable,
            emitted_checkpoint_count,
            checkpoint_count,
            checkpoint: Some(checkpoint),
            result: None,
        })
    }
}
