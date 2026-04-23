use super::*;

/// Profiling receipt for a completed offline render: frame/block counts, chain health,
/// delegation counts, and artifact materialization status.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderProfilingReceipt {
    /// ID of the render request this receipt belongs to.
    pub request_id: String,
    /// Total number of frames processed at the runtime sample rate.
    pub runtime_frame_count: usize,
    /// Total number of frames rendered at the export sample rate.
    pub rendered_frame_count: usize,
    /// Total number of audio blocks processed.
    pub block_count: usize,
    /// Sample rate of the exported audio in Hz.
    pub export_sample_rate_hz: u32,
    /// Number of stems rendered.
    pub stem_count: usize,
    /// Number of freeze artifacts rendered.
    pub freeze_artifact_count: usize,
    /// Peak level of the main mix output in linear scale, if rendered.
    pub main_mix_peak_level: Option<f32>,
    /// RMS level of the main mix output in linear scale, if rendered.
    pub main_mix_rms_level: Option<f32>,
    /// Total number of plugin chain stages involved in the render.
    pub chain_stage_count: usize,
    /// Number of plugin chain stages in a degraded state.
    pub chain_degraded_stage_count: usize,
    /// Number of plugin chain stages missing their sandbox binding.
    pub chain_missing_binding_stage_count: usize,
    /// Total planned latency across all plugin chain stages in samples.
    pub chain_total_planned_latency_samples: u32,
    /// Total realized latency across all plugin chain stages in samples.
    pub chain_total_realized_latency_samples: u32,
    /// Total plugin chain tail length across all stages in samples.
    pub chain_total_tail_samples: u32,
    /// Number of stages delegated to host execution.
    pub delegated_stage_count: usize,
    /// Number of stages using a fresh (current-block) plugin override.
    pub fresh_override_stage_count: usize,
    /// Number of stages using a stale plugin override.
    pub stale_override_stage_count: usize,
    /// Number of artifacts materialized to disk.
    pub artifact_count: usize,
    /// Whether the render report file was materialized.
    pub report_materialized: bool,
    /// Human-readable summary of this profiling receipt.
    pub summary: String,
}

impl RuntimeOfflineRenderProfilingReceipt {
    /// Renders a multi-line diagnostic string with one field per line.
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "request_id={}",
                "\nruntime_frame_count={}",
                "\nrendered_frame_count={}",
                "\nblock_count={}",
                "\nexport_sample_rate_hz={}",
                "\nstem_count={}",
                "\nfreeze_artifact_count={}",
                "\nmain_mix_peak_level={:?}",
                "\nmain_mix_rms_level={:?}",
                "\nchain_stage_count={}",
                "\nchain_degraded_stage_count={}",
                "\nchain_missing_binding_stage_count={}",
                "\nchain_total_planned_latency_samples={}",
                "\nchain_total_realized_latency_samples={}",
                "\nchain_total_tail_samples={}",
                "\ndelegated_stage_count={}",
                "\nfresh_override_stage_count={}",
                "\nstale_override_stage_count={}",
                "\nartifact_count={}",
                "\nreport_materialized={}",
                "\nsummary={}",
            ),
            self.request_id,
            self.runtime_frame_count,
            self.rendered_frame_count,
            self.block_count,
            self.export_sample_rate_hz,
            self.stem_count,
            self.freeze_artifact_count,
            self.main_mix_peak_level,
            self.main_mix_rms_level,
            self.chain_stage_count,
            self.chain_degraded_stage_count,
            self.chain_missing_binding_stage_count,
            self.chain_total_planned_latency_samples,
            self.chain_total_realized_latency_samples,
            self.chain_total_tail_samples,
            self.delegated_stage_count,
            self.fresh_override_stage_count,
            self.stale_override_stage_count,
            self.artifact_count,
            self.report_materialized,
            self.summary,
        )
    }

    /// Renders a JSON object of all profiling receipt fields.
    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"request_id\":{},",
                "\"runtime_frame_count\":{},",
                "\"rendered_frame_count\":{},",
                "\"block_count\":{},",
                "\"export_sample_rate_hz\":{},",
                "\"stem_count\":{},",
                "\"freeze_artifact_count\":{},",
                "\"main_mix_peak_level\":{},",
                "\"main_mix_rms_level\":{},",
                "\"chain_stage_count\":{},",
                "\"chain_degraded_stage_count\":{},",
                "\"chain_missing_binding_stage_count\":{},",
                "\"chain_total_planned_latency_samples\":{},",
                "\"chain_total_realized_latency_samples\":{},",
                "\"chain_total_tail_samples\":{},",
                "\"delegated_stage_count\":{},",
                "\"fresh_override_stage_count\":{},",
                "\"stale_override_stage_count\":{},",
                "\"artifact_count\":{},",
                "\"report_materialized\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&self.request_id),
            self.runtime_frame_count,
            self.rendered_frame_count,
            self.block_count,
            self.export_sample_rate_hz,
            self.stem_count,
            self.freeze_artifact_count,
            json_option_f32(self.main_mix_peak_level),
            json_option_f32(self.main_mix_rms_level),
            self.chain_stage_count,
            self.chain_degraded_stage_count,
            self.chain_missing_binding_stage_count,
            self.chain_total_planned_latency_samples,
            self.chain_total_realized_latency_samples,
            self.chain_total_tail_samples,
            self.delegated_stage_count,
            self.fresh_override_stage_count,
            self.stale_override_stage_count,
            self.artifact_count,
            self.report_materialized,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl RuntimeOfflineRenderResult {
    /// Constructs a profiling receipt summarising this render result.
    pub fn profiling_receipt(&self) -> RuntimeOfflineRenderProfilingReceipt {
        RuntimeOfflineRenderProfilingReceipt {
            request_id: self.request_id.clone(),
            runtime_frame_count: self.runtime_frame_count,
            rendered_frame_count: self.rendered_frame_count,
            block_count: self.block_count,
            export_sample_rate_hz: self.export_sample_rate_hz,
            stem_count: self.stems.len(),
            freeze_artifact_count: self.freeze_artifacts.len(),
            main_mix_peak_level: self.main_mix_peak_level,
            main_mix_rms_level: self.main_mix_rms_level,
            chain_stage_count: self.contract_preview.chain_contract.stage_count,
            chain_degraded_stage_count: self.contract_preview.chain_contract.degraded_stage_count,
            chain_missing_binding_stage_count: self
                .contract_preview
                .chain_contract
                .missing_binding_stage_count,
            chain_total_planned_latency_samples: self
                .contract_preview
                .chain_contract
                .total_planned_latency_samples,
            chain_total_realized_latency_samples: self
                .contract_preview
                .chain_contract
                .total_realized_latency_samples,
            chain_total_tail_samples: self.contract_preview.chain_contract.total_tail_samples,
            delegated_stage_count: self.plugin_execution_boundary.host_delegate_stage_count,
            fresh_override_stage_count: self.plugin_execution_boundary.fresh_override_stage_count,
            stale_override_stage_count: self.plugin_execution_boundary.stale_override_stage_count,
            artifact_count: self.manifest.artifact_count,
            report_materialized: self.manifest.report.is_some(),
            summary: format!(
                "request={} runtime_frames={} rendered_frames={} blocks={} export_rate={} stems={} freeze_artifacts={} chain={}/degraded={}/missing={} delegated={} overrides={}/{} artifacts={} report={}",
                self.request_id,
                self.runtime_frame_count,
                self.rendered_frame_count,
                self.block_count,
                self.export_sample_rate_hz,
                self.stems.len(),
                self.freeze_artifacts.len(),
                self.contract_preview.chain_contract.stage_count,
                self.contract_preview.chain_contract.degraded_stage_count,
                self.contract_preview.chain_contract.missing_binding_stage_count,
                self.plugin_execution_boundary.host_delegate_stage_count,
                self.plugin_execution_boundary.fresh_override_stage_count,
                self.plugin_execution_boundary.stale_override_stage_count,
                self.manifest.artifact_count,
                self.manifest.report.is_some(),
            ),
        }
    }
}
