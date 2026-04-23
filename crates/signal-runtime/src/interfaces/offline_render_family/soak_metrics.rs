use super::*;

/// Soak receipt for a completed offline render: clip readiness, recall stage coverage,
/// delegation outcomes, and artifact materialization status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderSoakReceipt {
    /// ID of the render request this receipt belongs to.
    pub request_id: String,
    /// Total number of clips involved in the render.
    pub clip_count: usize,
    /// Number of clips in the ready state at render time.
    pub ready_clip_count: usize,
    /// Number of freeze artifacts rendered.
    pub freeze_artifact_count: usize,
    /// Number of plugin recall stages involved in the render.
    pub recall_stage_count: usize,
    /// Number of recall stages that were successfully recovered.
    pub recovered_recall_stage_count: usize,
    /// Number of recall stages that were unavailable.
    pub unavailable_recall_stage_count: usize,
    /// Number of stages delegated to host execution.
    pub delegated_stage_count: usize,
    /// Number of delegated stages that completed successfully.
    pub delegated_completed_stage_count: usize,
    /// Number of delegated stages that were rejected by the host.
    pub delegated_rejected_stage_count: usize,
    /// Number of delegated stages that were unavailable.
    pub delegated_unavailable_stage_count: usize,
    /// Number of artifacts materialized to disk.
    pub materialized_artifact_count: usize,
    /// Whether the render report file was materialized.
    pub report_materialized: bool,
    /// Human-readable summary of this soak receipt.
    pub summary: String,
}

impl RuntimeOfflineRenderSoakReceipt {
    /// Renders a multi-line diagnostic string with one field per line.
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "request_id={}",
                "\nclip_count={}",
                "\nready_clip_count={}",
                "\nfreeze_artifact_count={}",
                "\nrecall_stage_count={}",
                "\nrecovered_recall_stage_count={}",
                "\nunavailable_recall_stage_count={}",
                "\ndelegated_stage_count={}",
                "\ndelegated_completed_stage_count={}",
                "\ndelegated_rejected_stage_count={}",
                "\ndelegated_unavailable_stage_count={}",
                "\nmaterialized_artifact_count={}",
                "\nreport_materialized={}",
                "\nsummary={}",
            ),
            self.request_id,
            self.clip_count,
            self.ready_clip_count,
            self.freeze_artifact_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            self.delegated_stage_count,
            self.delegated_completed_stage_count,
            self.delegated_rejected_stage_count,
            self.delegated_unavailable_stage_count,
            self.materialized_artifact_count,
            self.report_materialized,
            self.summary,
        )
    }

    /// Renders a JSON object of all soak receipt fields.
    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"clip_count\":{},",
                "\"ready_clip_count\":{},",
                "\"freeze_artifact_count\":{},",
                "\"recall_stage_count\":{},",
                "\"recovered_recall_stage_count\":{},",
                "\"unavailable_recall_stage_count\":{},",
                "\"delegated_stage_count\":{},",
                "\"delegated_completed_stage_count\":{},",
                "\"delegated_rejected_stage_count\":{},",
                "\"delegated_unavailable_stage_count\":{},",
                "\"materialized_artifact_count\":{},",
                "\"report_materialized\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&self.request_id),
            self.clip_count,
            self.ready_clip_count,
            self.freeze_artifact_count,
            self.recall_stage_count,
            self.recovered_recall_stage_count,
            self.unavailable_recall_stage_count,
            self.delegated_stage_count,
            self.delegated_completed_stage_count,
            self.delegated_rejected_stage_count,
            self.delegated_unavailable_stage_count,
            self.materialized_artifact_count,
            self.report_materialized,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl RuntimeOfflineRenderResult {
    /// Constructs a soak receipt summarising this render result.
    pub fn soak_receipt(&self) -> RuntimeOfflineRenderSoakReceipt {
        let delegated_receipt = self.manifest.delegated_execution_receipt.as_ref();
        RuntimeOfflineRenderSoakReceipt {
            request_id: self.request_id.clone(),
            clip_count: self.contract_preview.clip_count,
            ready_clip_count: self.contract_preview.ready_clip_count,
            freeze_artifact_count: self.freeze_artifacts.len(),
            recall_stage_count: self.contract_preview.chain_contract.recall_stage_count,
            recovered_recall_stage_count: self
                .contract_preview
                .chain_contract
                .recovered_recall_stage_count,
            unavailable_recall_stage_count: self
                .contract_preview
                .chain_contract
                .unavailable_recall_stage_count,
            delegated_stage_count: self.plugin_execution_boundary.host_delegate_stage_count,
            delegated_completed_stage_count: delegated_receipt
                .map(|receipt| receipt.completed_stage_count)
                .unwrap_or(0),
            delegated_rejected_stage_count: delegated_receipt
                .map(|receipt| receipt.rejected_stage_count)
                .unwrap_or(0),
            delegated_unavailable_stage_count: delegated_receipt
                .map(|receipt| receipt.unavailable_stage_count)
                .unwrap_or(0),
            materialized_artifact_count: self.manifest.artifact_count,
            report_materialized: self.manifest.report.is_some(),
            summary: format!(
                "request={} clips={}/{} freeze_artifacts={} recall={}/recovered={}/unavailable={} delegated={}/{}/{}/{} artifacts={} report={}",
                self.request_id,
                self.contract_preview.ready_clip_count,
                self.contract_preview.clip_count,
                self.freeze_artifacts.len(),
                self.contract_preview.chain_contract.recall_stage_count,
                self.contract_preview.chain_contract.recovered_recall_stage_count,
                self.contract_preview.chain_contract.unavailable_recall_stage_count,
                self.plugin_execution_boundary.host_delegate_stage_count,
                delegated_receipt
                    .map(|receipt| receipt.completed_stage_count)
                    .unwrap_or(0),
                delegated_receipt
                    .map(|receipt| receipt.rejected_stage_count)
                    .unwrap_or(0),
                delegated_receipt
                    .map(|receipt| receipt.unavailable_stage_count)
                    .unwrap_or(0),
                self.manifest.artifact_count,
                self.manifest.report.is_some(),
            ),
        }
    }
}
