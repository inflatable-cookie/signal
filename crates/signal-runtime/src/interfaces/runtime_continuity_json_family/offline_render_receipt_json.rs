use super::*;

pub(crate) fn json_runtime_offline_render_checkpoint_receipt(
    checkpoint: &RuntimeOfflineRenderCheckpointReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"stage\":{},",
            "\"checkpoint_index\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"total_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"total_block_count\":{},",
            "\"progress_percent\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(checkpoint.request_id.as_str())),
        json_option_string(Some(match checkpoint.stage {
            RuntimeOfflineRenderCheckpointStage::PreparingInput => "PreparingInput",
            RuntimeOfflineRenderCheckpointStage::RenderingGraph => "RenderingGraph",
            RuntimeOfflineRenderCheckpointStage::MaterializingOutputs => "MaterializingOutputs",
            RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts => "FinalizingArtifacts",
        })),
        checkpoint.checkpoint_index,
        checkpoint.checkpoint_count,
        checkpoint.rendered_frame_count,
        checkpoint.total_frame_count,
        checkpoint.rendered_block_count,
        checkpoint.total_block_count,
        checkpoint.progress_percent,
        json_option_string(Some(checkpoint.summary.as_str())),
    )
}

pub(crate) fn json_runtime_offline_render_execution_cancellation_receipt(
    receipt: &RuntimeOfflineRenderExecutionCancellationReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"cancelled_after_checkpoint_count\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(receipt.request_id.as_str())),
        receipt.cancelled_after_checkpoint_count,
        receipt.checkpoint_count,
        receipt.rendered_frame_count,
        receipt.rendered_block_count,
        json_option_string(Some(receipt.summary.as_str())),
    )
}

pub(crate) fn json_runtime_offline_render_purge_receipt(
    receipt: &RuntimeOfflineRenderPurgeReceipt,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"orchestration\":{},",
            "\"artifact_root_path\":{},",
            "\"report_path\":{},",
            "\"purged_artifact_root\":{},",
            "\"purged_artifact_file_count\":{},",
            "\"purged_artifact_byte_count\":{},",
            "\"purged_report\":{},",
            "\"purged_report_byte_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(receipt.request_id.as_str())),
        receipt.orchestration.render_json(),
        json_option_string(receipt.artifact_root_path.as_deref()),
        json_option_string(receipt.report_path.as_deref()),
        receipt.purged_artifact_root,
        receipt.purged_artifact_file_count,
        receipt.purged_artifact_byte_count,
        receipt.purged_report,
        receipt.purged_report_byte_count,
        json_option_string(Some(receipt.summary.as_str())),
    )
}

pub(crate) fn json_runtime_offline_render_session_state_snapshot(
    snapshot: &RuntimeOfflineRenderSessionStateSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"request_id\":{},",
            "\"state\":{},",
            "\"interruption_class\":{},",
            "\"interruption_rebindable\":{},",
            "\"interruption_count\":{},",
            "\"emitted_checkpoint_count\":{},",
            "\"checkpoint_count\":{},",
            "\"rendered_frame_count\":{},",
            "\"total_frame_count\":{},",
            "\"rendered_block_count\":{},",
            "\"total_block_count\":{},",
            "\"artifact_root_path\":{},",
            "\"report_path\":{},",
            "\"materialized\":{},",
            "\"artifact_count\":{},",
            "\"report_materialized\":{},",
            "\"active_checkpoint\":{},",
            "\"last_checkpoint\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.request_id.as_str())),
        json_option_string(Some(match snapshot.state {
            RuntimeOfflineRenderExecutionState::Running => "Running",
            RuntimeOfflineRenderExecutionState::Paused => "Paused",
            RuntimeOfflineRenderExecutionState::Recoverable => "Recoverable",
            RuntimeOfflineRenderExecutionState::Completed => "Completed",
            RuntimeOfflineRenderExecutionState::Cancelled => "Cancelled",
            RuntimeOfflineRenderExecutionState::Failed => "Failed",
        })),
        json_option_string(Some(match snapshot.interruption_class {
            RuntimeInterruptionClass::Steady => "Steady",
            RuntimeInterruptionClass::Resumable => "Resumable",
            RuntimeInterruptionClass::Restartable => "Restartable",
            RuntimeInterruptionClass::Recoverable => "Recoverable",
            RuntimeInterruptionClass::Terminal => "Terminal",
        })),
        snapshot.interruption_rebindable,
        snapshot.interruption_count,
        snapshot.emitted_checkpoint_count,
        snapshot.checkpoint_count,
        snapshot.rendered_frame_count,
        snapshot.total_frame_count,
        snapshot.rendered_block_count,
        snapshot.total_block_count,
        json_option_string(snapshot.artifact_root_path.as_deref()),
        json_option_string(snapshot.report_path.as_deref()),
        snapshot.materialized,
        snapshot.artifact_count,
        snapshot.report_materialized,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(json_runtime_offline_render_checkpoint_receipt)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(json_runtime_offline_render_checkpoint_receipt)
            .unwrap_or_else(|| "null".into()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
