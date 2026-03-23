use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderQueueProgressReceipt {
    pub request_id: String,
    pub queue_index: usize,
    pub queue_count: usize,
    pub completed_job_count: usize,
    pub progress_percent: u8,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderCheckpointStage {
    PreparingInput,
    RenderingGraph,
    MaterializingOutputs,
    FinalizingArtifacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderCheckpointReceipt {
    pub request_id: String,
    pub stage: RuntimeOfflineRenderCheckpointStage,
    pub checkpoint_index: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub total_frame_count: usize,
    pub rendered_block_count: usize,
    pub total_block_count: usize,
    pub progress_percent: u8,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionReceipt {
    pub request_id: String,
    pub checkpoint_count: usize,
    pub checkpoints: Vec<RuntimeOfflineRenderCheckpointReceipt>,
    pub result: RuntimeOfflineRenderResult,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderExecutionState {
    Running,
    Paused,
    Recoverable,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionProgressReceipt {
    pub request_id: String,
    pub state: RuntimeOfflineRenderExecutionState,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub emitted_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub result: Option<RuntimeOfflineRenderResult>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderExecutionCancellationReceipt {
    pub request_id: String,
    pub cancelled_after_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub rendered_block_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderSessionStateSnapshot {
    pub request_id: String,
    pub state: RuntimeOfflineRenderExecutionState,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub interruption_count: usize,
    pub emitted_checkpoint_count: usize,
    pub checkpoint_count: usize,
    pub rendered_frame_count: usize,
    pub total_frame_count: usize,
    pub rendered_block_count: usize,
    pub total_block_count: usize,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
    pub materialized: bool,
    pub artifact_count: usize,
    pub report_materialized: bool,
    pub active_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub last_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderSessionSnapshot {
    pub active_session_count: usize,
    pub paused_session_count: usize,
    pub recoverable_session_count: usize,
    pub active_sessions: Vec<RuntimeOfflineRenderSessionStateSnapshot>,
    pub last_session: Option<RuntimeOfflineRenderSessionStateSnapshot>,
    pub last_cancellation: Option<RuntimeOfflineRenderExecutionCancellationReceipt>,
    pub last_purge: Option<RuntimeOfflineRenderPurgeReceipt>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderQueueResult {
    pub queue_count: usize,
    pub completed_job_count: usize,
    pub orchestration: RuntimeDeferredServiceReceipt,
    pub progress: Vec<RuntimeOfflineRenderQueueProgressReceipt>,
    pub results: Vec<RuntimeOfflineRenderResult>,
    pub deferred_requests: Vec<RuntimeOfflineRenderRequest>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeRequest {
    pub request_id: String,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeReceipt {
    pub request_id: String,
    pub orchestration: RuntimeDeferredServiceReceipt,
    pub artifact_root_path: Option<String>,
    pub report_path: Option<String>,
    pub purged_artifact_root: bool,
    pub purged_artifact_file_count: usize,
    pub purged_artifact_byte_count: u64,
    pub purged_report: bool,
    pub purged_report_byte_count: u64,
    pub summary: String,
}
