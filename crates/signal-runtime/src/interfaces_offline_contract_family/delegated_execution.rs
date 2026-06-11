use super::*;
use std::collections::BTreeSet;

impl RuntimeOfflinePluginExecutionBoundary {
    /// Builds a delegated execution request containing only the stages that require host delegation.
    pub fn delegated_execution_request(&self) -> RuntimeOfflinePluginDelegatedExecutionRequest {
        let stages = self
            .stages
            .iter()
            .filter(|stage| stage.host_delegate_required)
            .map(|stage| RuntimeOfflinePluginDelegatedExecutionStageRequest {
                stage_id: stage.stage_id.clone(),
                node_id: stage.node_id.clone(),
                chain_id: stage.chain_id.clone(),
                stage_index: stage.stage_index,
                sandbox_id: stage.sandbox_id.clone(),
                plugin_type_id: stage.plugin_type_id.clone(),
                plugin_format: stage.plugin_format,
                recall_state: stage.recall_state,
                recall_payload: stage.recall_payload.clone(),
                override_state: stage.override_state,
                latest_override_processing_epoch: stage.latest_override_processing_epoch,
                latest_override_block_sequence: stage.latest_override_block_sequence,
            })
            .collect::<Vec<_>>();
        let stage_count = stages.len();

        RuntimeOfflinePluginDelegatedExecutionRequest {
            request_id: self.request_id.clone(),
            timeline_start_samples: self.timeline_start_samples,
            duration_samples: self.duration_samples,
            runtime_sample_rate_hz: self.runtime_sample_rate_hz,
            export_sample_rate_hz: self.export_sample_rate_hz,
            block_size: self.block_size,
            block_count: self.block_count,
            stage_count,
            stages,
        }
    }
}

impl RuntimeOfflineRenderManifest {
    /// Validates and applies a delegated plugin execution receipt to this manifest.
    pub fn apply_delegated_execution_receipt(
        &mut self,
        receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    ) -> Result<(), RuntimeError> {
        if receipt.request_id != self.request_id {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt request `{}` does not match manifest request `{}`",
                    receipt.request_id, self.request_id
                ),
            ));
        }
        if receipt.stage_count != receipt.stages.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage_count must match stage receipt count",
            ));
        }
        if self.delegated_execution_request.stage_count != receipt.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt stages do not match request (request={} receipt={})",
                    self.delegated_execution_request.stage_count, receipt.stage_count
                ),
            ));
        }

        let expected_stage_ids = self
            .delegated_execution_request
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        let receipt_stage_ids = receipt
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        if expected_stage_ids != receipt_stage_ids {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage ids must match the delegated request",
            ));
        }
        let completed_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Completed)
            .count();
        let rejected_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Rejected)
            .count();
        let unavailable_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| {
                stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Unavailable
            })
            .count();
        if receipt.completed_stage_count != completed_stage_count
            || receipt.rejected_stage_count != rejected_stage_count
            || receipt.unavailable_stage_count != unavailable_stage_count
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt status counters must match stage receipt statuses",
            ));
        }

        self.delegated_execution_receipt = Some(receipt);
        Ok(())
    }
}
