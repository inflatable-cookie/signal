use super::*;

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

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
    let mut manifest = RuntimeOfflineRenderManifest {
        request_id: request_id.to_string(),
        artifact_root_path: artifact_root_path.map(str::to_string),
        materialized,
        artifact_count,
        artifacts,
        report,
        delegated_execution_request,
        delegated_execution_receipt,
        summary: String::new(),
    };
    manifest.summary = format!(
        "request={} root={:?} materialized={} artifacts={} report={} delegated_request_stages={} delegated_receipt={}",
        manifest.request_id,
        manifest.artifact_root_path.as_deref(),
        manifest.materialized,
        manifest.artifact_count,
        manifest.report.is_some(),
        manifest.delegated_execution_request.stage_count,
        manifest.delegated_execution_receipt.is_some(),
    );
    manifest
}

fn offline_render_report_json(
    result: &RuntimeOfflineRenderResult,
    artifact_receipts: &[RuntimeOfflineRenderArtifactReceipt],
) -> String {
    let artifacts = artifact_receipts
        .iter()
        .map(|artifact| {
            format!(
                "{{\"artifact_id\":\"{}\",\"artifact_kind\":\"{:?}\",\"output_path\":\"{}\",\"sample_rate_hz\":{},\"channel_count\":{},\"frame_count\":{},\"byte_size\":{},\"peak_level\":{:.6},\"rms_level\":{:.6}}}",
                json_escape(&artifact.artifact_id),
                artifact.artifact_kind,
                json_escape(&artifact.output_path),
                artifact.sample_rate_hz,
                artifact.channel_count,
                artifact.frame_count,
                artifact.byte_size,
                artifact.peak_level,
                artifact.rms_level,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let delegated_request_stages = result
        .manifest
        .delegated_execution_request
        .stages
        .iter()
        .map(|stage| {
            format!(
                "{{\"chain_id\":\"{}\",\"stage_index\":{},\"node_id\":\"{}\",\"sandbox_id\":{},\"plugin_type_id\":{},\"plugin_format\":{},\"recall_state\":\"{:?}\",\"override_state\":\"{:?}\"}}",
                json_escape(&stage.chain_id),
                stage.stage_index,
                json_escape(&stage.node_id),
                stage.sandbox_id.as_deref().map(|id| format!("\"{}\"", json_escape(id))).unwrap_or_else(|| "null".into()),
                stage.plugin_type_id.as_deref().map(|id| format!("\"{}\"", json_escape(id))).unwrap_or_else(|| "null".into()),
                stage.plugin_format.map(|format| format!("\"{format:?}\"")).unwrap_or_else(|| "null".into()),
                stage.recall_state,
                stage.override_state,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let delegated_receipt = result
        .manifest
        .delegated_execution_receipt
        .as_ref()
        .map(|receipt| {
            let stages = receipt
                .stages
                .iter()
                .map(|stage| {
                    format!(
                        "{{\"chain_id\":\"{}\",\"stage_index\":{},\"node_id\":\"{}\",\"status\":\"{:?}\",\"delegate_label\":{},\"detail\":{}}}",
                        json_escape(&stage.chain_id),
                        stage.stage_index,
                        json_escape(&stage.node_id),
                        stage.status,
                        stage
                            .delegate_label
                            .as_deref()
                            .map(|label| format!("\"{}\"", json_escape(label)))
                            .unwrap_or_else(|| "null".into()),
                        stage
                            .detail
                            .as_deref()
                            .map(|detail| format!("\"{}\"", json_escape(detail)))
                            .unwrap_or_else(|| "null".into()),
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"stage_count\":{},\"completed_stage_count\":{},\"rejected_stage_count\":{},\"unavailable_stage_count\":{},\"stages\":[{}]}}",
                receipt.stage_count,
                receipt.completed_stage_count,
                receipt.rejected_stage_count,
                receipt.unavailable_stage_count,
                stages,
            )
        })
        .unwrap_or_else(|| "null".into());
    format!(
        "{{\"request_id\":\"{}\",\"runtime_frame_count\":{},\"rendered_frame_count\":{},\"block_count\":{},\"export_sample_rate_hz\":{},\"artifact_count\":{},\"delegated_stage_count\":{},\"delegated_receipt_stage_count\":{},\"delegated_execution_request\":{{\"stage_count\":{},\"stages\":[{}]}},\"delegated_execution_receipt\":{},\"summary\":\"{}\",\"artifacts\":[{}]}}",
        json_escape(&result.request_id),
        result.runtime_frame_count,
        result.rendered_frame_count,
        result.block_count,
        result.export_sample_rate_hz,
        artifact_receipts.len(),
        result.manifest.delegated_execution_request.stage_count,
        result
            .manifest
            .delegated_execution_receipt
            .as_ref()
            .map_or(0, |receipt| receipt.stage_count),
        result.manifest.delegated_execution_request.stage_count,
        delegated_request_stages,
        delegated_receipt,
        json_escape(&result.summary),
        artifacts,
    )
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
        summary: format!(
            "request={} report_path={} artifacts={} bytes={}",
            request_id,
            report_path.display(),
            artifact_count,
            report_size,
        ),
    })
}
