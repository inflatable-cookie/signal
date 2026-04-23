use super::*;

mod render_metrics;
mod request_preview;
mod session_receipts;
mod soak_metrics;

pub use render_metrics::RuntimeOfflineRenderProfilingReceipt;
pub use request_preview::*;
pub use session_receipts::*;
pub use soak_metrics::RuntimeOfflineRenderSoakReceipt;

/// Kind of audio artifact produced by an offline render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderArtifactKind {
    /// The main mix output artifact.
    MainMix,
    /// A stem (per-route) output artifact.
    Stem,
    /// A freeze artifact capturing plugin state.
    FreezeArtifact,
}

/// Receipt for a single materialized audio artifact from an offline render.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderArtifactReceipt {
    /// Unique identifier for this artifact.
    pub artifact_id: String,
    /// Kind of artifact (main mix, stem, or freeze).
    pub artifact_kind: RuntimeOfflineRenderArtifactKind,
    /// Absolute path to the output file.
    pub output_path: String,
    /// Sample rate of the output file in Hz.
    pub sample_rate_hz: u32,
    /// Number of audio channels in the output file.
    pub channel_count: usize,
    /// Number of audio frames in the output file.
    pub frame_count: usize,
    /// File size in bytes.
    pub byte_size: u64,
    /// Peak level of the artifact in linear scale.
    pub peak_level: f32,
    /// RMS level of the artifact in linear scale.
    pub rms_level: f32,
    /// Human-readable summary of this artifact receipt.
    pub summary: String,
}

/// Receipt for the JSON report file emitted alongside render artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderReportReceipt {
    /// ID of the render request this report belongs to.
    pub request_id: String,
    /// Absolute path to the JSON report file.
    pub report_path: String,
    /// Number of artifacts described in the report.
    pub artifact_count: usize,
    /// File size of the report in bytes.
    pub byte_size: u64,
    /// Human-readable summary of this report receipt.
    pub summary: String,
}

/// Complete manifest for one offline render job: artifacts, report,
/// delegated plugin execution request and receipt.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderManifest {
    /// ID of the render request this manifest belongs to.
    pub request_id: String,
    /// Root directory path for output artifacts, if any.
    pub artifact_root_path: Option<String>,
    /// Whether the manifest has been materialized (artifacts written to disk).
    pub materialized: bool,
    /// Number of artifacts in this manifest.
    pub artifact_count: usize,
    /// Per-artifact receipts.
    pub artifacts: Vec<RuntimeOfflineRenderArtifactReceipt>,
    /// Report file receipt, if the report was materialized.
    pub report: Option<RuntimeOfflineRenderReportReceipt>,
    /// Delegated plugin execution request sent to the host.
    pub delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    /// Delegated plugin execution receipt from the host, if available.
    pub delegated_execution_receipt: Option<RuntimeOfflinePluginDelegatedExecutionReceipt>,
    /// Human-readable summary of this manifest.
    pub summary: String,
}

/// Per-stage request for delegated plugin execution during an offline render.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageRequest {
    /// Recall handoff stage identifier.
    pub stage_id: RuntimePluginRecallHandoffStageId,
    /// ID of the graph node for this stage.
    pub node_id: String,
    /// ID of the plugin chain this stage belongs to.
    pub chain_id: String,
    /// Zero-based index of this stage within its chain.
    pub stage_index: usize,
    /// ID of the plugin sandbox, if known.
    pub sandbox_id: Option<String>,
    /// Plugin type identifier, if known.
    pub plugin_type_id: Option<String>,
    /// Plugin format, if known.
    pub plugin_format: Option<PluginFormat>,
    /// Plugin recall state to restore before execution.
    pub recall_state: RuntimePluginRecallState,
    /// Serialized recall payload for this stage.
    pub recall_payload: RuntimePluginRecallPayload,
    /// Override availability for this stage.
    pub override_state: RuntimeOfflinePluginOverrideState,
    /// Processing epoch of the latest available override block, if any.
    pub latest_override_processing_epoch: Option<u64>,
    /// Block sequence of the latest available override block, if any.
    pub latest_override_block_sequence: Option<u64>,
    /// Human-readable summary of this stage request.
    pub summary: String,
}

/// Full request for host-delegated plugin execution: timeline range, block
/// parameters, and per-stage requests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionRequest {
    /// ID of the render request this belongs to.
    pub request_id: String,
    /// Timeline start position in samples.
    pub timeline_start_samples: i64,
    /// Duration of the execution range in samples.
    pub duration_samples: u32,
    /// Runtime sample rate in Hz.
    pub runtime_sample_rate_hz: u32,
    /// Export sample rate in Hz.
    pub export_sample_rate_hz: u32,
    /// Audio block size in frames.
    pub block_size: usize,
    /// Total number of blocks to process.
    pub block_count: usize,
    /// Number of stages in this request.
    pub stage_count: usize,
    /// Per-stage execution requests.
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageRequest>,
    /// Human-readable summary of this execution request.
    pub summary: String,
}

/// Outcome of one delegated plugin execution stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginDelegatedExecutionStatus {
    #[default]
    /// The stage was executed successfully.
    Completed,
    /// The host rejected this stage's execution request.
    Rejected,
    /// The stage was not available for execution.
    Unavailable,
}

/// Per-stage receipt for a delegated plugin execution attempt.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionStageReceipt {
    /// Recall handoff stage identifier.
    pub stage_id: RuntimePluginRecallHandoffStageId,
    /// ID of the graph node for this stage.
    pub node_id: String,
    /// ID of the plugin chain this stage belongs to.
    pub chain_id: String,
    /// Zero-based index of this stage within its chain.
    pub stage_index: usize,
    /// Execution outcome for this stage.
    pub status: RuntimeOfflinePluginDelegatedExecutionStatus,
    /// Label identifying which host delegate handled this stage, if any.
    pub delegate_label: Option<String>,
    /// Additional detail about the execution outcome, if any.
    pub detail: Option<String>,
    /// Human-readable summary of this stage receipt.
    pub summary: String,
}

/// Aggregate receipt for all delegated plugin execution stages in one render job.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginDelegatedExecutionReceipt {
    /// ID of the render request this receipt belongs to.
    pub request_id: String,
    /// Total number of stages in the execution.
    pub stage_count: usize,
    /// Number of stages that completed successfully.
    pub completed_stage_count: usize,
    /// Number of stages rejected by the host.
    pub rejected_stage_count: usize,
    /// Number of stages that were unavailable.
    pub unavailable_stage_count: usize,
    /// Per-stage receipts.
    pub stages: Vec<RuntimeOfflinePluginDelegatedExecutionStageReceipt>,
    /// Human-readable summary of this execution receipt.
    pub summary: String,
}

/// Rendered audio buffer output for a single stem from a delegated plugin execution.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedStemOutput {
    /// ID of the stem this output corresponds to.
    pub stem_id: String,
    /// Rendered audio output for this stem.
    pub output: AudioBuffer,
    /// Human-readable summary of this stem output.
    pub summary: String,
}

/// Rendered audio buffer output for a single freeze artifact from a delegated plugin execution.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedFreezeArtifactOutput {
    /// ID of the freeze artifact this output corresponds to.
    pub artifact_id: String,
    /// Rendered audio output for this freeze artifact.
    pub output: AudioBuffer,
    /// Human-readable summary of this freeze artifact output.
    pub summary: String,
}

/// Merge of all delegated plugin execution outputs: main mix, stems, and freeze artifacts.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionMerge {
    /// ID of the render request this merge belongs to.
    pub request_id: String,
    /// Main mix audio output, if rendered.
    pub main_mix: Option<AudioBuffer>,
    /// Per-stem audio outputs.
    pub stems: Vec<RuntimeOfflinePluginDelegatedStemOutput>,
    /// Per-freeze-artifact audio outputs.
    pub freeze_artifacts: Vec<RuntimeOfflinePluginDelegatedFreezeArtifactOutput>,
    /// Human-readable summary of this execution merge.
    pub summary: String,
}

/// Combined receipt and merged audio outputs from a delegated plugin execution.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflinePluginDelegatedExecutionOutcome {
    /// Execution receipt summarising stage outcomes.
    pub receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    /// Merged audio outputs from all delegated stages.
    pub merge: RuntimeOfflinePluginDelegatedExecutionMerge,
    /// Human-readable summary of the execution outcome.
    pub summary: String,
}

/// Who executes a plugin stage during offline render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginExecutionOwner {
    #[default]
    /// Signal's own stage model executes this plugin stage.
    SignalStageModel,
    /// The host is delegated to execute this plugin stage.
    HostDelegated,
}

/// Whether a plugin's last processed block is available and fresh enough to
/// serve as an override during offline render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeOfflinePluginOverrideState {
    #[default]
    /// No override block is available for this stage.
    NotAvailable,
    /// A fresh (current-block) override is available.
    FreshLatestBlock,
    /// An override is available but it is from a stale block.
    StaleLatestBlock,
}

/// Boundary contract for a single plugin stage in an offline render: owner,
/// recall state, and override availability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionStageBoundary {
    /// Recall handoff stage identifier.
    pub stage_id: RuntimePluginRecallHandoffStageId,
    /// ID of the graph node for this stage.
    pub node_id: String,
    /// ID of the plugin chain this stage belongs to.
    pub chain_id: String,
    /// Zero-based index of this stage within its chain.
    pub stage_index: usize,
    /// ID of the plugin sandbox, if known.
    pub sandbox_id: Option<String>,
    /// Plugin type identifier, if known.
    pub plugin_type_id: Option<String>,
    /// Plugin format, if known.
    pub plugin_format: Option<PluginFormat>,
    /// ID of the track lane this stage belongs to, if any.
    pub track_lane_id: Option<String>,
    /// ID of the bus group this stage belongs to, if any.
    pub bus_group_id: Option<String>,
    /// ID of the console group this stage belongs to, if any.
    pub console_group_id: Option<String>,
    /// ID of the send-return pair this stage belongs to, if any.
    pub send_return_id: Option<String>,
    /// Plugin recall state for this stage.
    pub recall_state: RuntimePluginRecallState,
    /// Serialized recall payload for this stage.
    pub recall_payload: RuntimePluginRecallPayload,
    /// Who executes this stage during the offline render.
    pub execution_owner: RuntimeOfflinePluginExecutionOwner,
    /// Whether host delegation is required for this stage.
    pub host_delegate_required: bool,
    /// Override availability for this stage.
    pub override_state: RuntimeOfflinePluginOverrideState,
    /// Processing epoch of the latest available override block, if any.
    pub latest_override_processing_epoch: Option<u64>,
    /// Block sequence of the latest available override block, if any.
    pub latest_override_block_sequence: Option<u64>,
    /// Human-readable summary of this stage boundary.
    pub summary: String,
}

/// Full offline render plugin execution boundary: timeline range, block
/// parameters, and per-stage boundary records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflinePluginExecutionBoundary {
    /// ID of the render request this boundary belongs to.
    pub request_id: String,
    /// Timeline start position in samples.
    pub timeline_start_samples: i64,
    /// Duration of the execution range in samples.
    pub duration_samples: u32,
    /// Runtime sample rate in Hz.
    pub runtime_sample_rate_hz: u32,
    /// Export sample rate in Hz.
    pub export_sample_rate_hz: u32,
    /// Audio block size in frames.
    pub block_size: usize,
    /// Total number of audio blocks to process.
    pub block_count: usize,
    /// Total number of plugin stages in this boundary.
    pub stage_count: usize,
    /// Number of stages executed by Signal's stage model.
    pub signal_stage_model_stage_count: usize,
    /// Number of stages delegated to host execution.
    pub host_delegate_stage_count: usize,
    /// Number of stages using a fresh override block.
    pub fresh_override_stage_count: usize,
    /// Number of stages using a stale override block.
    pub stale_override_stage_count: usize,
    /// Per-stage execution boundary records.
    pub stages: Vec<RuntimeOfflinePluginExecutionStageBoundary>,
    /// Human-readable summary of this execution boundary.
    pub summary: String,
}
