use super::*;

/// Progress receipt for a render job within the offline render queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderQueueProgressReceipt {
    /// ID of the render request this progress belongs to.
    pub request_id: String,
    /// Zero-based index of this job in the current queue.
    pub queue_index: usize,
    /// Total number of jobs in the current queue.
    pub queue_count: usize,
    /// Number of jobs completed so far in this queue run.
    pub completed_job_count: usize,
    /// Overall queue progress as a percentage (0–100).
    pub progress_percent: u8,
}

/// Stage reached by a render checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderCheckpointStage {
    /// Input clips and plugin state are being prepared.
    PreparingInput,
    /// The audio graph is being processed block by block.
    RenderingGraph,
    /// Rendered audio is being written to output buffers or files.
    MaterializingOutputs,
    /// Output artifacts are being finalized and reports written.
    FinalizingArtifacts,
}

/// Point-in-time progress snapshot for one offline render checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderCheckpointReceipt {
    /// ID of the render request this checkpoint belongs to.
    pub request_id: String,
    /// Stage of the render pipeline at which this checkpoint was taken.
    pub stage: RuntimeOfflineRenderCheckpointStage,
    /// Zero-based index of this checkpoint.
    pub checkpoint_index: usize,
    /// Total number of checkpoints expected for this render.
    pub checkpoint_count: usize,
    /// Number of frames rendered so far.
    pub rendered_frame_count: usize,
    /// Total number of frames to render.
    pub total_frame_count: usize,
    /// Number of audio blocks processed so far.
    pub rendered_block_count: usize,
    /// Total number of audio blocks to process.
    pub total_block_count: usize,
    /// Render progress as a percentage (0–100).
    pub progress_percent: u8,
}

/// Final execution receipt for a completed offline render: all checkpoints and result.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionReceipt {
    /// ID of the render request this receipt belongs to.
    pub request_id: String,
    /// Number of checkpoints emitted during this render.
    pub checkpoint_count: usize,
    /// All checkpoint receipts in order.
    pub checkpoints: Vec<RuntimeOfflineRenderCheckpointReceipt>,
    /// Final render result.
    pub result: RuntimeOfflineRenderResult,
}

/// Current execution state of an offline render session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderExecutionState {
    /// Render is actively processing.
    Running,
    /// Render is paused and waiting to resume.
    Paused,
    /// Render encountered an interruption but can be recovered.
    Recoverable,
    /// Render completed successfully.
    Completed,
    /// Render was cancelled by the caller.
    Cancelled,
    /// Render failed with an unrecoverable error.
    Failed,
}

/// Live progress receipt for an active offline render session.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderExecutionProgressReceipt {
    /// ID of the render request this progress belongs to.
    pub request_id: String,
    /// Current execution state of the render session.
    pub state: RuntimeOfflineRenderExecutionState,
    /// Interruption class if the session has been interrupted.
    pub interruption_class: RuntimeInterruptionClass,
    /// Whether the interruption can be resolved by rebinding plugin state.
    pub interruption_rebindable: bool,
    /// Number of checkpoints emitted so far.
    pub emitted_checkpoint_count: usize,
    /// Total number of checkpoints expected.
    pub checkpoint_count: usize,
    /// Most recent checkpoint receipt, if any.
    pub checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    /// Final render result, available only once the session completes.
    pub result: Option<RuntimeOfflineRenderResult>,
}

/// Receipt confirming an offline render session was cancelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderExecutionCancellationReceipt {
    /// ID of the cancelled render request.
    pub request_id: String,
    /// Number of checkpoints that had been emitted before cancellation.
    pub cancelled_after_checkpoint_count: usize,
    /// Total number of checkpoints that were expected.
    pub checkpoint_count: usize,
    /// Number of frames that had been rendered before cancellation.
    pub rendered_frame_count: usize,
    /// Number of audio blocks that had been processed before cancellation.
    pub rendered_block_count: usize,
}

/// Full state snapshot for one active or completed offline render session.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderSessionStateSnapshot {
    /// ID of the render request this snapshot belongs to.
    pub request_id: String,
    /// Current execution state.
    pub state: RuntimeOfflineRenderExecutionState,
    /// Interruption class if the session was interrupted.
    pub interruption_class: RuntimeInterruptionClass,
    /// Whether the interruption is rebindable.
    pub interruption_rebindable: bool,
    /// Number of times this session has been interrupted.
    pub interruption_count: usize,
    /// Number of checkpoints emitted so far.
    pub emitted_checkpoint_count: usize,
    /// Total number of checkpoints expected.
    pub checkpoint_count: usize,
    /// Number of frames rendered so far.
    pub rendered_frame_count: usize,
    /// Total number of frames to render.
    pub total_frame_count: usize,
    /// Number of audio blocks processed so far.
    pub rendered_block_count: usize,
    /// Total number of audio blocks to process.
    pub total_block_count: usize,
    /// Root path for rendered artifacts, if any.
    pub artifact_root_path: Option<String>,
    /// Path of the render report file, if any.
    pub report_path: Option<String>,
    /// Whether the render has been materialized (artifacts written to disk).
    pub materialized: bool,
    /// Number of artifacts materialized.
    pub artifact_count: usize,
    /// Whether the report file was materialized.
    pub report_materialized: bool,
    /// The currently active checkpoint, if any.
    pub active_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    /// The most recently completed checkpoint, if any.
    pub last_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
}

/// Aggregate snapshot of the offline render session queue: active, paused,
/// and recoverable sessions, plus last cancellation and purge records.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeOfflineRenderSessionSnapshot {
    /// Number of actively running render sessions.
    pub active_session_count: usize,
    /// Number of paused render sessions.
    pub paused_session_count: usize,
    /// Number of recoverable (interrupted but rebindable) sessions.
    pub recoverable_session_count: usize,
    /// State snapshots for all active and paused sessions.
    pub active_sessions: Vec<RuntimeOfflineRenderSessionStateSnapshot>,
    /// Snapshot of the most recently completed or cancelled session, if any.
    pub last_session: Option<RuntimeOfflineRenderSessionStateSnapshot>,
    /// Most recent cancellation receipt, if any.
    pub last_cancellation: Option<RuntimeOfflineRenderExecutionCancellationReceipt>,
    /// Most recent purge receipt, if any.
    pub last_purge: Option<RuntimeOfflineRenderPurgeReceipt>,
}

/// Result of processing the offline render queue: completed jobs, progress
/// receipts, per-job results, and deferred requests.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderQueueResult {
    /// Number of jobs in the queue at the start of this processing cycle.
    pub queue_count: usize,
    /// Number of jobs completed during this cycle.
    pub completed_job_count: usize,
    /// Deferred service orchestration receipt for this queue run.
    pub orchestration: RuntimeDeferredServiceReceipt,
    /// Per-job queue progress receipts.
    pub progress: Vec<RuntimeOfflineRenderQueueProgressReceipt>,
    /// Per-job render results for completed jobs.
    pub results: Vec<RuntimeOfflineRenderResult>,
    /// Requests that were deferred to the next processing cycle.
    pub deferred_requests: Vec<RuntimeOfflineRenderRequest>,
}

/// Request to delete render artifacts and/or the report file for a prior render.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeRequest {
    /// ID of the render request whose artifacts should be purged.
    pub request_id: String,
    /// Root path of the artifact directory to delete, if applicable.
    pub artifact_root_path: Option<String>,
    /// Path of the report file to delete, if applicable.
    pub report_path: Option<String>,
}

/// Receipt confirming which files were deleted by a purge request.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderPurgeReceipt {
    /// ID of the render request that was purged.
    pub request_id: String,
    /// Deferred service orchestration receipt for this purge.
    pub orchestration: RuntimeDeferredServiceReceipt,
    /// Root path of the artifact directory that was targeted, if any.
    pub artifact_root_path: Option<String>,
    /// Path of the report file that was targeted, if any.
    pub report_path: Option<String>,
    /// Whether the artifact root directory was successfully deleted.
    pub purged_artifact_root: bool,
    /// Number of artifact files deleted.
    pub purged_artifact_file_count: usize,
    /// Total bytes reclaimed from artifact files.
    pub purged_artifact_byte_count: u64,
    /// Whether the report file was successfully deleted.
    pub purged_report: bool,
    /// Bytes reclaimed from the report file.
    pub purged_report_byte_count: u64,
}
