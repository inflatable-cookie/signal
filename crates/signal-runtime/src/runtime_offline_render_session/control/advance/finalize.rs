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
            let materializing_summary = format!(
                "request={} stage=materializing-outputs main_mix={} stems={} freeze_artifacts={}",
                session.request.request_id,
                session.request.include_main_mix,
                session.preview.stem_count,
                session.preview.freeze_artifact_count,
            );
            let checkpoint = Self::emit_offline_render_session_checkpoint(
                &mut session,
                RuntimeOfflineRenderCheckpointStage::MaterializingOutputs,
                rendered_frames,
                block_count,
                95,
                materializing_summary,
            );
            let emitted_checkpoint_count = session.emitted_checkpoint_count;
            let checkpoint_count = session.checkpoint_count;
            let summary = format!(
                "request={} state=running checkpoints={}/{} stage=materializing-outputs",
                request_id, emitted_checkpoint_count, checkpoint_count
            );
            session.last_state_summary = summary.clone();
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
                summary,
            });
        }

        if !session.finalizing_checkpoint_emitted {
            session.finalizing_checkpoint_emitted = true;
            let rendered_frames = session.rendered_frames;
            let block_count = session.block_count;
            let finalizing_summary = format!(
                "request={} stage=finalizing-artifacts artifact_root={} pending_delivery=true",
                session.request.request_id,
                session
                    .request
                    .artifact_root_path
                    .as_deref()
                    .unwrap_or("none"),
            );
            let checkpoint = Self::emit_offline_render_session_checkpoint(
                &mut session,
                RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts,
                rendered_frames,
                block_count,
                99,
                finalizing_summary,
            );
            let emitted_checkpoint_count = session.emitted_checkpoint_count;
            let checkpoint_count = session.checkpoint_count;
            let summary = format!(
                "request={} state=running checkpoints={}/{} stage=finalizing-artifacts",
                request_id, emitted_checkpoint_count, checkpoint_count
            );
            session.last_state_summary = summary.clone();
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
                summary,
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
        session.last_state_summary = format!(
            "request={} state=completed checkpoints={}/{} artifacts={} report={}",
            request_id,
            session.emitted_checkpoint_count,
            session.checkpoint_count,
            result.manifest.artifact_count,
            result.manifest.report.is_some(),
        );
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
            summary: format!(
                "request={} state=completed checkpoints={}/{}",
                request_id, session.emitted_checkpoint_count, session.checkpoint_count
            ),
        })
    }
}
