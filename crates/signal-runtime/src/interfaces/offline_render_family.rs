use super::*;

mod render_metrics;
mod request_preview;
mod session_receipts;
mod soak_metrics;

pub use render_metrics::RuntimeOfflineRenderProfilingReceipt;
pub use request_preview::*;
pub use session_receipts::*;
pub use soak_metrics::RuntimeOfflineRenderSoakReceipt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderArtifactKind {
    MainMix,
    Stem,
    FreezeArtifact,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderArtifactReceipt {
    pub artifact_id: String,
    pub artifact_kind: RuntimeOfflineRenderArtifactKind,
    pub output_path: String,
    pub sample_rate_hz: u32,
    pub channel_count: usize,
    pub frame_count: usize,
    pub byte_size: u64,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderReportReceipt {
    pub request_id: String,
    pub report_path: String,
    pub artifact_count: usize,
    pub byte_size: u64,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderManifest {
    pub request_id: String,
    pub artifact_root_path: Option<String>,
    pub materialized: bool,
    pub artifact_count: usize,
    pub artifacts: Vec<RuntimeOfflineRenderArtifactReceipt>,
    pub report: Option<RuntimeOfflineRenderReportReceipt>,
    pub delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    pub delegated_execution_receipt: Option<RuntimeOfflinePluginDelegatedExecutionReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageRequest {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
    pub override_state: RuntimeOfflinePluginOverrideState,
    pub latest_override_processing_epoch: Option<u64>,
    pub latest_override_block_sequence: Option<u64>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionRequest {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub runtime_sample_rate_hz: u32,
    pub export_sample_rate_hz: u32,
    pub block_size: usize,
    pub block_count: usize,
    pub stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageRequest>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginDelegatedExecutionStatus {
    #[default]
    Completed,
    Rejected,
    Unavailable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageReceipt {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub status: RuntimeOfflinePluginDelegatedExecutionStatus,
    pub delegate_label: Option<String>,
    pub detail: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionReceipt {
    pub request_id: String,
    pub stage_count: usize,
    pub completed_stage_count: usize,
    pub rejected_stage_count: usize,
    pub unavailable_stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedStemOutput {
    pub stem_id: String,
    pub output: AudioBuffer,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
    pub artifact_id: String,
    pub output: AudioBuffer,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionMerge {
    pub request_id: String,
    pub main_mix: Option<AudioBuffer>,
    pub stems: Vec<RuntimeOfflinePluginDelegatedStemOutput>,
    pub freeze_artifacts: Vec<RuntimeOfflinePluginDelegatedFreezeArtifactOutput>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionOutcome {
    pub receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    pub merge: RuntimeOfflinePluginDelegatedExecutionMerge,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginExecutionOwner {
    #[default]
    SignalStageModel,
    HostDelegated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginOverrideState {
    #[default]
    NotAvailable,
    FreshLatestBlock,
    StaleLatestBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionStageBoundary {
    pub stage_id: RuntimePluginRecallHandoffStageId,
    pub node_id: String,
    pub chain_id: String,
    pub stage_index: usize,
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub recall_state: RuntimePluginRecallState,
    pub recall_payload: RuntimePluginRecallPayload,
    pub execution_owner: RuntimeOfflinePluginExecutionOwner,
    pub host_delegate_required: bool,
    pub override_state: RuntimeOfflinePluginOverrideState,
    pub latest_override_processing_epoch: Option<u64>,
    pub latest_override_block_sequence: Option<u64>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionBoundary {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub runtime_sample_rate_hz: u32,
    pub export_sample_rate_hz: u32,
    pub block_size: usize,
    pub block_count: usize,
    pub stage_count: usize,
    pub signal_stage_model_stage_count: usize,
    pub host_delegate_stage_count: usize,
    pub fresh_override_stage_count: usize,
    pub stale_override_stage_count: usize,
    pub stages: Vec<RuntimeOfflinePluginExecutionStageBoundary>,
    pub summary: String,
}
