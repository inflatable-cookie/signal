use super::*;

impl SignalRuntime {
    pub(in super::super) fn offline_plugin_delegated_execution_request(
        boundary: &RuntimeOfflinePluginExecutionBoundary,
    ) -> RuntimeOfflinePluginDelegatedExecutionRequest {
        boundary.delegated_execution_request()
    }

    pub(in super::super) fn offline_render_execution_interruption_class(
        state: RuntimeOfflineRenderExecutionState,
    ) -> RuntimeInterruptionClass {
        match state {
            RuntimeOfflineRenderExecutionState::Running
            | RuntimeOfflineRenderExecutionState::Completed => RuntimeInterruptionClass::Steady,
            RuntimeOfflineRenderExecutionState::Paused
            | RuntimeOfflineRenderExecutionState::Recoverable => {
                RuntimeInterruptionClass::Resumable
            }
            RuntimeOfflineRenderExecutionState::Cancelled
            | RuntimeOfflineRenderExecutionState::Failed => RuntimeInterruptionClass::Terminal,
        }
    }

    pub(in super::super) fn offline_render_execution_observed_interruption_class(
        session: &RuntimeOfflineRenderExecutionSession,
    ) -> RuntimeInterruptionClass {
        session
            .interruption_class_override
            .unwrap_or_else(|| Self::offline_render_execution_interruption_class(session.state))
    }

    pub(in super::super) fn offline_render_execution_interruption_rebindable(
        session: &RuntimeOfflineRenderExecutionSession,
    ) -> bool {
        session.delegated_execution_request.stage_count > 0
            && matches!(
                session.state,
                RuntimeOfflineRenderExecutionState::Paused
                    | RuntimeOfflineRenderExecutionState::Recoverable
            )
    }

    fn ensure_offline_buffer_compatible(
        label: &str,
        expected: &AudioBuffer,
        actual: &AudioBuffer,
    ) -> Result<(), RuntimeError> {
        if expected.sample_rate() != actual.sample_rate() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline merge `{label}` sample rate does not match runtime result (expected={} actual={})",
                    expected.sample_rate().0,
                    actual.sample_rate().0,
                ),
            ));
        }
        if expected.channel_count() != actual.channel_count() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline merge `{label}` channel count does not match runtime result (expected={} actual={})",
                    expected.channel_count().0,
                    actual.channel_count().0,
                ),
            ));
        }
        if expected.frames() != actual.frames() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline merge `{label}` frame count does not match runtime result (expected={} actual={})",
                    expected.frames().0,
                    actual.frames().0,
                ),
            ));
        }
        Ok(())
    }

    pub(in super::super) fn apply_delegated_execution_merge(
        result: &mut RuntimeOfflineRenderResult,
        merge: &RuntimeOfflinePluginDelegatedExecutionMerge,
    ) -> Result<(), RuntimeError> {
        if merge.request_id != result.request_id {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline merge request `{}` does not match runtime result request `{}`",
                    merge.request_id, result.request_id
                ),
            ));
        }
        if let Some(main_mix) = merge.main_mix.as_ref() {
            let expected = result.main_mix.as_ref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "delegated offline merge provided a main mix override for a render result without main mix output",
                )
            })?;
            Self::ensure_offline_buffer_compatible("main_mix", expected, main_mix)?;
            result.main_mix = Some(main_mix.clone());
        }

        let mut seen_stems = BTreeSet::new();
        for stem_merge in &merge.stems {
            if !seen_stems.insert(stem_merge.stem_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "delegated offline merge stem `{}` is duplicated",
                        stem_merge.stem_id
                    ),
                ));
            }
            let stem = result
                .stems
                .iter_mut()
                .find(|stem| stem.stem_id == stem_merge.stem_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "delegated offline merge references unknown stem `{}`",
                            stem_merge.stem_id
                        ),
                    )
                })?;
            Self::ensure_offline_buffer_compatible(
                &format!("stem:{}", stem_merge.stem_id),
                &stem.output,
                &stem_merge.output,
            )?;
            stem.output = stem_merge.output.clone();
        }

        let mut seen_artifacts = BTreeSet::new();
        for artifact_merge in &merge.freeze_artifacts {
            if !seen_artifacts.insert(artifact_merge.artifact_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "delegated offline merge freeze artifact `{}` is duplicated",
                        artifact_merge.artifact_id
                    ),
                ));
            }
            let artifact = result
                .freeze_artifacts
                .iter_mut()
                .find(|artifact| artifact.artifact_id == artifact_merge.artifact_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "delegated offline merge references unknown freeze artifact `{}`",
                            artifact_merge.artifact_id
                        ),
                    )
                })?;
            Self::ensure_offline_buffer_compatible(
                &format!("freeze:{}", artifact_merge.artifact_id),
                &artifact.output,
                &artifact_merge.output,
            )?;
            artifact.output = artifact_merge.output.clone();
        }

        refresh_offline_render_result(result);
        Ok(())
    }

    /// Prepares a delegated plugin execution request from the current runtime state for the given render request.
    pub fn prepare_offline_plugin_delegated_execution_request(
        &self,
        request: &RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflinePluginDelegatedExecutionRequest, RuntimeError> {
        let boundary = self.prepare_offline_plugin_execution_boundary(request)?;
        Ok(Self::offline_plugin_delegated_execution_request(&boundary))
    }

    /// Applies a delegated plugin execution receipt to an existing render result and re-materializes the manifest.
    pub fn apply_offline_plugin_delegated_execution_receipt(
        &self,
        result: &RuntimeOfflineRenderResult,
        receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        let expected_request =
            Self::offline_plugin_delegated_execution_request(&result.plugin_execution_boundary);
        let mut updated = result.clone();
        updated.manifest.delegated_execution_request = expected_request;
        updated
            .manifest
            .apply_delegated_execution_receipt(receipt)?;
        updated.manifest = materialize_offline_render_delivery(&updated)?;
        Ok(updated)
    }

    /// Applies a delegated plugin execution outcome (receipt + audio merge) to an existing render result.
    pub fn apply_offline_plugin_delegated_execution_outcome(
        &self,
        result: &RuntimeOfflineRenderResult,
        outcome: RuntimeOfflinePluginDelegatedExecutionOutcome,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        let expected_request =
            Self::offline_plugin_delegated_execution_request(&result.plugin_execution_boundary);
        let mut updated = result.clone();
        updated.manifest.delegated_execution_request = expected_request;
        updated
            .manifest
            .apply_delegated_execution_receipt(outcome.receipt)?;
        Self::apply_delegated_execution_merge(&mut updated, &outcome.merge)?;
        updated.manifest = materialize_offline_render_delivery(&updated)?;
        Ok(updated)
    }
}
