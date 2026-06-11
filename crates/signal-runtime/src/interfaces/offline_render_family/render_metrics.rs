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
}

impl RuntimeOfflineRenderProfilingReceipt {

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
        }
    }
}
