use super::*;

impl SignalRuntime {
    pub(in super::super) fn offline_render_session_state_snapshot(
        session: &RuntimeOfflineRenderExecutionSession,
    ) -> crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot {
        let (report_path, artifact_count, report_materialized) = session
            .materialized_result
            .as_ref()
            .map(|result| {
                (
                    result
                        .manifest
                        .report
                        .as_ref()
                        .map(|receipt| receipt.report_path.clone()),
                    result.manifest.artifact_count,
                    result.manifest.report.is_some(),
                )
            })
            .unwrap_or((None, 0, false));
        crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot {
            request_id: session.request.request_id.clone(),
            state: session.state,
            interruption_class: Self::offline_render_execution_observed_interruption_class(session),
            interruption_rebindable: Self::offline_render_execution_interruption_rebindable(
                session,
            ),
            interruption_count: session.interruption_count,
            emitted_checkpoint_count: session.emitted_checkpoint_count,
            checkpoint_count: session.checkpoint_count,
            rendered_frame_count: session.rendered_frames,
            total_frame_count: session.total_frames,
            rendered_block_count: session.block_count,
            total_block_count: session.total_block_count,
            artifact_root_path: session.request.artifact_root_path.clone(),
            report_path,
            materialized: session.materialized_result.is_some(),
            artifact_count,
            report_materialized,
            active_checkpoint: (session.state != RuntimeOfflineRenderExecutionState::Completed
                && session.state != RuntimeOfflineRenderExecutionState::Cancelled
                && session.state != RuntimeOfflineRenderExecutionState::Failed)
                .then(|| session.last_checkpoint.clone())
                .flatten(),
            last_checkpoint: session.last_checkpoint.clone(),
            summary: session.last_state_summary.clone(),
        }
    }

    pub(in super::super) fn offline_render_session_snapshot(
        &self,
    ) -> crate::interfaces::RuntimeOfflineRenderSessionSnapshot {
        let active_sessions = self
            .offline_render_executions
            .values()
            .map(Self::offline_render_session_state_snapshot)
            .collect::<Vec<_>>();
        let active_session_count = active_sessions.len();
        let paused_session_count = active_sessions
            .iter()
            .filter(|session| session.state == RuntimeOfflineRenderExecutionState::Paused)
            .count();
        let recoverable_session_count = active_sessions
            .iter()
            .filter(|session| session.state == RuntimeOfflineRenderExecutionState::Recoverable)
            .count();
        let last_session = self.last_offline_render_session_snapshot.borrow().clone();
        let last_cancellation = self
            .last_offline_render_cancellation_receipt
            .borrow()
            .clone();
        let last_purge = self.last_offline_render_purge_receipt.borrow().clone();
        let last_session_present = last_session.is_some();
        let last_cancellation_present = last_cancellation.is_some();
        let last_purge_present = last_purge.is_some();
        crate::interfaces::RuntimeOfflineRenderSessionSnapshot {
            active_session_count,
            paused_session_count,
            recoverable_session_count,
            active_sessions,
            last_session,
            last_cancellation,
            last_purge,
            summary: format!(
                "active_sessions={} paused_sessions={} recoverable_sessions={} last_session={} last_cancellation={} last_purge={}",
                active_session_count,
                paused_session_count,
                recoverable_session_count,
                last_session_present,
                last_cancellation_present,
                last_purge_present,
            ),
        }
    }

    pub(in super::super) fn record_last_offline_render_session_snapshot(
        &self,
        snapshot: crate::interfaces::RuntimeOfflineRenderSessionStateSnapshot,
    ) {
        self.last_offline_render_session_snapshot
            .replace(Some(snapshot));
    }

    pub(in super::super) fn mark_offline_render_sessions_restartable(&mut self, reason: &str) {
        let summary_snapshot = self
            .offline_render_executions
            .values_mut()
            .map(|session| {
                session.state = RuntimeOfflineRenderExecutionState::Recoverable;
                session.interruption_count = session.interruption_count.saturating_add(1);
                session.interruption_class_override = Some(RuntimeInterruptionClass::Restartable);
                session.last_state_summary = format!(
                    "request={} state=recoverable interruption=restartable checkpoints={}/{} interruptions={} reason={}",
                    session.request.request_id,
                    session.emitted_checkpoint_count,
                    session.checkpoint_count,
                    session.interruption_count,
                    reason,
                );
                Self::offline_render_session_state_snapshot(session)
            })
            .last();
        if let Some(snapshot) = summary_snapshot {
            self.record_last_offline_render_session_snapshot(snapshot);
        }
    }

    pub(in super::super) fn record_terminal_offline_render_session_failure(
        &self,
        session: &mut RuntimeOfflineRenderExecutionSession,
        stage: RuntimeOfflineRenderCheckpointStage,
        error: &RuntimeError,
    ) {
        session.state = RuntimeOfflineRenderExecutionState::Failed;
        session.interruption_count = session.interruption_count.saturating_add(1);
        session.interruption_class_override = Some(RuntimeInterruptionClass::Terminal);
        session.last_state_summary = format!(
            "request={} state=failed stage={:?} checkpoints={}/{} interruptions={} error={:?}",
            session.request.request_id,
            stage,
            session.emitted_checkpoint_count,
            session.checkpoint_count,
            session.interruption_count,
            error,
        );
        self.record_last_offline_render_session_snapshot(
            Self::offline_render_session_state_snapshot(session),
        );
    }

    pub(in super::super) fn offline_render_execution_status_receipt(
        session: &RuntimeOfflineRenderExecutionSession,
    ) -> RuntimeOfflineRenderExecutionProgressReceipt {
        RuntimeOfflineRenderExecutionProgressReceipt {
            request_id: session.request.request_id.clone(),
            state: session.state,
            interruption_class: Self::offline_render_execution_observed_interruption_class(session),
            interruption_rebindable: Self::offline_render_execution_interruption_rebindable(
                session,
            ),
            emitted_checkpoint_count: session.emitted_checkpoint_count,
            checkpoint_count: session.checkpoint_count,
            checkpoint: None,
            result: None,
            summary: session.last_state_summary.clone(),
        }
    }
}
