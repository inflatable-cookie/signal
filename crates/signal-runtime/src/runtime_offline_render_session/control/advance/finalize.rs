use super::*;

impl SignalRuntime {
    pub(super) fn advance_offline_render_execution_finalize(
        &mut self,
        request_id: &str,
        mut session: RuntimeOfflineRenderExecutionSession,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        if session.materialized_result.is_none() {
            let result = match self.materialize_offline_render_session_outputs(&mut session) {
                Ok(result) => result,
                Err(error) => {
                    self.record_terminal_offline_render_session_failure(
                        &mut session,
                        RuntimeOfflineRenderCheckpointStage::MaterializingOutputs,
                        &error,
                    );
                    return Err(error);
                }
            };
            session.materialized_result = Some(result);
            let rendered_frames = session.rendered_frames;
            let block_count = session.block_count;
            let checkpoint = Self::emit_offline_render_session_checkpoint(
                &mut session,
                RuntimeOfflineRenderCheckpointStage::MaterializingOutputs,
                rendered_frames,
                block_count,
                95,
            );
            let emitted_checkpoint_count = session.emitted_checkpoint_count;
            let checkpoint_count = session.checkpoint_count;
            self.record_last_offline_render_session_snapshot(
                Self::offline_render_session_state_snapshot(&session),
            );
            self.offline_render_executions
                .insert(request_id.to_string(), session);
            return Ok(RuntimeOfflineRenderExecutionProgressReceipt {
                request_id: request_id.to_string(),
                state: RuntimeOfflineRenderExecutionState::Running,
                interruption_class: Self::offline_render_execution_interruption_class(
                    RuntimeOfflineRenderExecutionState::Running,
                ),
                interruption_rebindable: false,
                emitted_checkpoint_count,
                checkpoint_count,
                checkpoint: Some(checkpoint),
                result: None,
            });
        }

        if !session.finalizing_checkpoint_emitted {
            session.finalizing_checkpoint_emitted = true;
            let rendered_frames = session.rendered_frames;
            let block_count = session.block_count;
            let checkpoint = Self::emit_offline_render_session_checkpoint(
                &mut session,
                RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts,
                rendered_frames,
                block_count,
                99,
            );
            let emitted_checkpoint_count = session.emitted_checkpoint_count;
            let checkpoint_count = session.checkpoint_count;
            self.record_last_offline_render_session_snapshot(
                Self::offline_render_session_state_snapshot(&session),
            );
            self.offline_render_executions
                .insert(request_id.to_string(), session);
            return Ok(RuntimeOfflineRenderExecutionProgressReceipt {
                request_id: request_id.to_string(),
                state: RuntimeOfflineRenderExecutionState::Running,
                interruption_class: Self::offline_render_execution_interruption_class(
                    RuntimeOfflineRenderExecutionState::Running,
                ),
                interruption_rebindable: false,
                emitted_checkpoint_count,
                checkpoint_count,
                checkpoint: Some(checkpoint),
                result: None,
            });
        }

        let mut result = session.materialized_result.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                format!(
                    "offline render execution `{request_id}` has no materialized result to finalize"
                ),
            )
        })?;
        result.manifest = match materialize_offline_render_delivery(&result) {
            Ok(manifest) => manifest,
            Err(error) => {
                self.record_terminal_offline_render_session_failure(
                    &mut session,
                    RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts,
                    &error,
                );
                return Err(error);
            }
        };
        session.state = RuntimeOfflineRenderExecutionState::Completed;
        session.interruption_class_override = None;
        session.materialized_result = Some(result.clone());
        self.record_last_offline_render_session_snapshot(
            Self::offline_render_session_state_snapshot(&session),
        );
        Ok(RuntimeOfflineRenderExecutionProgressReceipt {
            request_id: request_id.to_string(),
            state: RuntimeOfflineRenderExecutionState::Completed,
            interruption_class: Self::offline_render_execution_interruption_class(
                RuntimeOfflineRenderExecutionState::Completed,
            ),
            interruption_rebindable: false,
            emitted_checkpoint_count: session.emitted_checkpoint_count,
            checkpoint_count: session.checkpoint_count,
            checkpoint: None,
            result: Some(result),
        })
    }
}
