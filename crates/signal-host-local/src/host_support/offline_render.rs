use signal_runtime::{
    RuntimeError, RuntimeOfflinePluginDelegatedExecutionMerge,
    RuntimeOfflinePluginDelegatedExecutionOutcome, RuntimeOfflinePluginDelegatedExecutionReceipt,
    RuntimeOfflinePluginDelegatedExecutionStageReceipt,
    RuntimeOfflinePluginDelegatedExecutionStatus,
    RuntimeOfflinePluginDelegatedFreezeArtifactOutput, RuntimeOfflinePluginDelegatedStemOutput,
    RuntimeOfflineRenderRequest, RuntimeOfflineRenderResult,
};

use super::super::LocalRuntimeHost;
use super::transfer::scale_audio_buffer;

impl LocalRuntimeHost {
    fn local_delegated_execution_outcome(
        &self,
        result: &RuntimeOfflineRenderResult,
    ) -> Result<Option<RuntimeOfflinePluginDelegatedExecutionOutcome>, RuntimeError> {
        let delegated_request = if result.manifest.delegated_execution_request.stage_count > 0 {
            result.manifest.delegated_execution_request.clone()
        } else {
            result
                .plugin_execution_boundary
                .delegated_execution_request()
        };
        if delegated_request.stage_count == 0 {
            return Ok(None);
        }

        let attenuation = 1.0_f32 / (delegated_request.stage_count as f32 + 1.0);
        let receipt = RuntimeOfflinePluginDelegatedExecutionReceipt {
            request_id: result.request_id.clone(),
            stage_count: delegated_request.stage_count,
            completed_stage_count: delegated_request.stage_count,
            rejected_stage_count: 0,
            unavailable_stage_count: 0,
            stages: delegated_request
                .stages
                .iter()
                .map(|stage| RuntimeOfflinePluginDelegatedExecutionStageReceipt {
                    stage_id: stage.stage_id.clone(),
                    node_id: stage.node_id.clone(),
                    chain_id: stage.chain_id.clone(),
                    stage_index: stage.stage_index,
                    status: RuntimeOfflinePluginDelegatedExecutionStatus::Completed,
                    delegate_label: Some("local-host-delegated-executor".into()),
                    detail: Some(format!(
                        "local delegated executor rendered stage {}:{}",
                        stage.chain_id, stage.stage_index
                    )),
                })
                .collect(),
        };
        let merge = RuntimeOfflinePluginDelegatedExecutionMerge {
            request_id: result.request_id.clone(),
            main_mix: result
                .main_mix
                .as_ref()
                .map(|buffer| scale_audio_buffer(buffer, attenuation)),
            stems: result
                .stems
                .iter()
                .map(|stem| RuntimeOfflinePluginDelegatedStemOutput {
                    stem_id: stem.stem_id.clone(),
                    output: scale_audio_buffer(&stem.output, attenuation),
                })
                .collect(),
            freeze_artifacts: result
                .freeze_artifacts
                .iter()
                .map(
                    |artifact| RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
                        artifact_id: artifact.artifact_id.clone(),
                        output: scale_audio_buffer(&artifact.output, attenuation),
                    },
                )
                .collect(),
        };
        Ok(Some(RuntimeOfflinePluginDelegatedExecutionOutcome {
            receipt,
            merge,
        }))
    }

    /// Applies a locally-simulated delegated execution outcome to a completed offline render result.
    pub fn finalize_offline_render_with_local_delegated_executor(
        &self,
        result: RuntimeOfflineRenderResult,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        let Some(outcome) = self.local_delegated_execution_outcome(&result)? else {
            return Ok(result);
        };
        self.runtime
            .apply_offline_plugin_delegated_execution_outcome(&result, outcome)
    }

    /// Renders offline audio and applies the local delegated execution outcome in one call.
    pub fn render_offline_with_local_delegated_executor(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        let result = self.runtime.render_offline(request)?;
        self.finalize_offline_render_with_local_delegated_executor(result)
    }
}
