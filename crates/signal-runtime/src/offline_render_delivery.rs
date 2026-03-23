use super::*;

#[path = "offline_render_delivery/artifact_materialization.rs"]
mod artifact_materialization;
#[path = "offline_render_delivery/manifest_report.rs"]
mod manifest_report;

pub(super) fn offline_render_manifest(
    request_id: &str,
    artifact_root_path: Option<&str>,
    artifacts: Vec<RuntimeOfflineRenderArtifactReceipt>,
    report: Option<RuntimeOfflineRenderReportReceipt>,
    delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    delegated_execution_receipt: Option<RuntimeOfflinePluginDelegatedExecutionReceipt>,
) -> RuntimeOfflineRenderManifest {
    manifest_report::offline_render_manifest(
        request_id,
        artifact_root_path,
        artifacts,
        report,
        delegated_execution_request,
        delegated_execution_receipt,
    )
}

pub(super) fn materialize_offline_render_delivery(
    result: &RuntimeOfflineRenderResult,
) -> Result<RuntimeOfflineRenderManifest, RuntimeError> {
    artifact_materialization::materialize_offline_render_delivery(result)
}
