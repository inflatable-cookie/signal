use super::*;

pub(super) fn offline_render_manifest(
    request_id: &str,
    artifact_root_path: Option<&str>,
    artifacts: Vec<RuntimeOfflineRenderArtifactReceipt>,
    report: Option<RuntimeOfflineRenderReportReceipt>,
    delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    delegated_execution_receipt: Option<RuntimeOfflinePluginDelegatedExecutionReceipt>,
) -> RuntimeOfflineRenderManifest {
    let materialized = !artifacts.is_empty() || report.is_some();
    let artifact_count = artifacts.len();
    let manifest = RuntimeOfflineRenderManifest {
        request_id: request_id.to_string(),
        artifact_root_path: artifact_root_path.map(str::to_string),
        materialized,
        artifact_count,
        artifacts,
        report,
        delegated_execution_request,
        delegated_execution_receipt,
    };
    manifest
}

fn offline_render_report_json(
    result: &RuntimeOfflineRenderResult,
    artifact_receipts: &[RuntimeOfflineRenderArtifactReceipt],
) -> String {
    let request = &result.manifest.delegated_execution_request;
    let receipt = result.manifest.delegated_execution_receipt.as_ref();
    let request_stages = request
        .stages
        .iter()
        .map(|stage| {
            serde_json::json!({
                "chain_id": stage.chain_id,
                "stage_index": stage.stage_index,
                "node_id": stage.node_id,
                "sandbox_id": stage.sandbox_id,
                "plugin_type_id": stage.plugin_type_id,
                "plugin_format": stage.plugin_format.map(|format| format!("{format:?}")),
                "recall_state": stage.recall_state,
                "override_state": stage.override_state,
            })
        })
        .collect::<Vec<_>>();
    let receipt_value = receipt
        .map(|receipt| {
            serde_json::json!({
                "stage_count": receipt.stage_count,
                "completed_stage_count": receipt.completed_stage_count,
                "rejected_stage_count": receipt.rejected_stage_count,
                "unavailable_stage_count": receipt.unavailable_stage_count,
                "stages": receipt
                    .stages
                    .iter()
                    .map(|stage| {
                        serde_json::json!({
                            "chain_id": stage.chain_id,
                            "stage_index": stage.stage_index,
                            "node_id": stage.node_id,
                            "status": stage.status,
                            "delegate_label": stage.delegate_label,
                            "detail": stage.detail,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
        })
        .unwrap_or(serde_json::Value::Null);
    serde_json::json!({
        "request_id": result.request_id,
        "runtime_frame_count": result.runtime_frame_count,
        "rendered_frame_count": result.rendered_frame_count,
        "block_count": result.block_count,
        "export_sample_rate_hz": result.export_sample_rate_hz,
        "artifact_count": artifact_receipts.len(),
        "delegated_stage_count": request.stage_count,
        "delegated_receipt_stage_count": receipt.map_or(0, |receipt| receipt.stage_count),
        "delegated_execution_request": {
            "stage_count": request.stage_count,
            "stages": request_stages,
        },
        "delegated_execution_receipt": receipt_value,
        "artifacts": artifact_receipts,
    })
    .to_string()
}

pub(super) fn write_offline_render_report(
    path: &Path,
    result: &RuntimeOfflineRenderResult,
) -> Result<u64, RuntimeError> {
    let report_body = offline_render_report_json(result, &result.manifest.artifacts);
    fs::write(path, report_body.as_bytes()).map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to write offline render report {}: {error}",
                path.display()
            ),
        )
    })?;
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "failed to inspect offline render report {}: {error}",
                    path.display()
                ),
            )
        })
}

pub(super) fn offline_render_report_receipt(
    request_id: &str,
    report_path: &Path,
    artifact_count: usize,
) -> Result<RuntimeOfflineRenderReportReceipt, RuntimeError> {
    let report_size = fs::metadata(report_path)
        .map(|metadata| metadata.len())
        .map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "failed to inspect offline render report {}: {error}",
                    report_path.display()
                ),
            )
        })?;
    Ok(RuntimeOfflineRenderReportReceipt {
        request_id: request_id.to_string(),
        report_path: report_path.display().to_string(),
        artifact_count,
        byte_size: report_size,
    })
}
