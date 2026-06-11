use super::*;

/// Human-readable reason string included in a `Degraded` readiness state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedReason(pub &'static str);

/// Category of plugin sandbox fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    /// Sandbox exceeded its block processing deadline.
    Timeout,
    /// Sandbox process crashed or was killed.
    Crash,
    /// Sandbox violated the communication protocol.
    ProtocolViolation,
}

/// What caused the watchdog to fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWatchdogTrigger {
    /// Consecutive block-deadline misses exceeded the threshold.
    DeadlineMisses,
    /// Consecutive heartbeat misses exceeded the threshold.
    HeartbeatMisses,
}

/// Immutable record of a watchdog-triggered sandbox restart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogRestartRecord {
    /// Sandbox that was restarted.
    pub sandbox_id: String,
    /// What caused the watchdog to fire.
    pub trigger: RuntimeWatchdogTrigger,
    /// Processing epoch at which the restart occurred.
    pub processing_epoch: u64,
}

/// Aggregated supervision counters: watchdog restarts, safe mode, xrun overload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupervisionSnapshot {
    /// Total watchdog-triggered restarts since startup.
    pub watchdog_restart_count: u32,
    /// Whether safe mode is currently engaged.
    pub safe_mode_enabled: bool,
    /// Whether xrun overload is currently active.
    pub xrun_overload_active: bool,
    /// Most recent watchdog trigger kind, if any restart has occurred.
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    /// Sandbox ID involved in the most recent watchdog restart.
    pub last_sandbox_id: Option<String>,
    /// Processing epoch of the most recent watchdog restart.
    pub last_processing_epoch: Option<u64>,
}

/// Timeline and transport observation: block sequence continuity, transport
/// epochs, and loop tracking.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTimelineSnapshot {
    /// Next expected block sequence number.
    pub next_block_sequence: u64,
    /// Continuity report for the block sequence.
    pub block_sequence_continuity: BlockSequenceContinuityReport,
    /// Current transport epoch.
    pub transport_epoch: u64,
    /// Most recent transport transition kind, if any.
    pub last_transport_transition: Option<RuntimeTransportTransitionKind>,
    /// Processing epoch of the most recent transport transition.
    pub last_transport_transition_processing_epoch: Option<u64>,
    /// Block sequence of the most recent transport transition.
    pub last_transport_transition_block_sequence: Option<u64>,
    /// Whether the transport was playing at the most recent block.
    pub last_transport_playing: Option<bool>,
    /// Tempo in BPM at the most recent block.
    pub last_transport_tempo_bpm: Option<f64>,
    /// Timeline position in samples at the most recent block.
    pub last_transport_timeline_position_samples: Option<i64>,
    /// Loop start position in samples, if looping.
    pub last_transport_loop_start_samples: Option<i64>,
    /// Loop end position in samples, if looping.
    pub last_transport_loop_end_samples: Option<i64>,
    /// Engine block start position in samples at the most recent block.
    pub last_engine_block_start_samples: Option<i64>,
    /// Engine block end position in samples at the most recent block.
    pub last_engine_block_end_samples: Option<i64>,
    /// Total number of loop wraps since playback started.
    pub loop_wrap_count: u64,
}

/// Side-by-side view of projected vs. observed transport state.
///
/// Useful for diagnosing lag between what the host projected and what the
/// engine actually processed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTransportObservationSnapshot {
    /// Transport epoch for this observation.
    pub transport_epoch: u64,
    /// Transport playing state projected by the host.
    pub projected_playing: Option<bool>,
    /// Tempo in BPM projected by the host.
    pub projected_tempo_bpm: Option<f64>,
    /// Timeline position in samples projected by the host.
    pub projected_timeline_position_samples: Option<i64>,
    /// Loop start in samples projected by the host.
    pub projected_loop_start_samples: Option<i64>,
    /// Loop end in samples projected by the host.
    pub projected_loop_end_samples: Option<i64>,
    /// Transport playing state observed by the engine.
    pub observed_playing: Option<bool>,
    /// Tempo in BPM observed by the engine.
    pub observed_tempo_bpm: Option<f64>,
    /// Timeline position in samples observed by the engine.
    pub observed_timeline_position_samples: Option<i64>,
    /// Loop start in samples observed by the engine.
    pub observed_loop_start_samples: Option<i64>,
    /// Loop end in samples observed by the engine.
    pub observed_loop_end_samples: Option<i64>,
    /// Most recent transport transition observed.
    pub last_transition: Option<RuntimeTransportTransitionKind>,
    /// Processing epoch of the most recent transport transition.
    pub last_transition_processing_epoch: Option<u64>,
    /// Block sequence of the most recent transport transition.
    pub last_transition_block_sequence: Option<u64>,
    /// Engine block start in samples at the most recent block.
    pub last_engine_block_start_samples: Option<i64>,
    /// Engine block end in samples at the most recent block.
    pub last_engine_block_end_samples: Option<i64>,
    /// Total loop wraps observed since playback started.
    pub loop_wrap_count: u64,
}

/// State of the active recording capture session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureState {
    /// No capture is in progress.
    Idle,
    /// A capture is actively recording.
    Capturing,
    /// The capture session has failed.
    Failed,
}

/// Media type being recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureKind {
    /// Audio capture to a file.
    Audio,
    /// MIDI capture to a file.
    Midi,
}

/// Stage of a recording capture checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureCheckpointClass {
    /// Capture is armed and waiting to start.
    Armed,
    /// Audio/MIDI data is streaming to disk.
    Streaming,
    /// Data has been buffered but not yet committed.
    Buffered,
    /// Capture was successfully committed.
    Committed,
    /// Capture failed at this checkpoint.
    Failed,
}

/// Snapshot of a single recording capture checkpoint (armed/streaming/buffered/
/// committed).
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCheckpointSnapshot {
    /// Media type being captured.
    pub capture_kind: RuntimeRecordingCaptureKind,
    /// Stage of this checkpoint.
    pub checkpoint_class: RuntimeRecordingCaptureCheckpointClass,
    /// Interruption class active when this checkpoint was recorded.
    pub interruption_class: RuntimeInterruptionClass,
    /// Take identifier for this capture.
    pub take_id: String,
    /// Track identifier for this capture.
    pub track_id: String,
    /// Timeline position in samples where capture started.
    pub capture_start_samples: i64,
    /// File path being written to.
    pub capture_path: String,
    /// Number of blocks buffered in this checkpoint.
    pub buffered_block_count: u64,
    /// Number of audio frames buffered.
    pub buffered_frame_count: u64,
    /// Number of MIDI events buffered.
    pub buffered_event_count: u64,
    /// Number of channels captured.
    pub captured_channel_count: usize,
    /// Peak signal level observed, if available.
    pub peak_level: Option<f32>,
    /// Number of backpressure events encountered during capture.
    pub pressure_event_count: u64,
    /// Most recent error string, if any.
    pub last_error: Option<String>,
}

/// Request to begin recording to a file path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordingCaptureStartRequest {
    /// Media type to capture.
    pub capture_kind: RuntimeRecordingCaptureKind,
    /// Take identifier.
    pub take_id: String,
    /// Track identifier.
    pub track_id: String,
    /// Timeline position in samples where recording should start.
    pub start_samples: i64,
    /// File path to write the captured media to.
    pub capture_path: String,
}

/// Receipt returned after a successful recording capture commit.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCommitReceipt {
    /// Media type that was captured.
    pub capture_kind: RuntimeRecordingCaptureKind,
    /// Take identifier.
    pub take_id: String,
    /// Track identifier.
    pub track_id: String,
    /// Timeline position in samples where capture started.
    pub start_samples: i64,
    /// Duration of the captured media in samples.
    pub duration_samples: u32,
    /// Number of channels captured.
    pub channel_count: usize,
    /// Peak signal level across the capture.
    pub peak_level: f32,
    /// File path the capture was written to.
    pub capture_path: String,
    /// Snapshot of the checkpoint that was committed.
    pub committed_checkpoint: RuntimeRecordingCaptureCheckpointSnapshot,
}

/// Live snapshot of the recording capture subsystem.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRecordingCaptureSnapshot {
    /// Whether the capture subsystem is ready to accept a start request.
    pub capture_ready: bool,
    /// Current capture state, if a session exists.
    pub state: Option<RuntimeRecordingCaptureState>,
    /// Media type of the active capture, if any.
    pub capture_kind: Option<RuntimeRecordingCaptureKind>,
    /// Take ID of the active capture, if any.
    pub active_take_id: Option<String>,
    /// Track ID of the active capture, if any.
    pub active_track_id: Option<String>,
    /// Timeline start position in samples of the active capture.
    pub capture_start_samples: Option<i64>,
    /// File path being written to by the active capture.
    pub active_capture_path: Option<String>,
    /// Number of blocks buffered so far.
    pub buffered_block_count: u64,
    /// Number of audio frames buffered so far.
    pub buffered_frame_count: u64,
    /// Number of MIDI events buffered so far.
    pub buffered_event_count: u64,
    /// Number of channels in the active capture.
    pub captured_channel_count: usize,
    /// Running peak signal level, if available.
    pub peak_level: Option<f32>,
    /// Number of backpressure events encountered.
    pub pressure_event_count: u64,
    /// Snapshot of the active (in-progress) checkpoint, if any.
    pub active_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    /// Snapshot of the most recently completed checkpoint.
    pub last_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    /// Take ID of the most recently committed capture.
    pub last_committed_take_id: Option<String>,
    /// File path of the most recently committed capture.
    pub last_committed_path: Option<String>,
    /// Duration in samples of the most recently committed capture.
    pub last_committed_duration_samples: Option<u32>,
    /// Most recent error string, if any.
    pub last_error: Option<String>,
}

/// Type of deferred background work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceClass {
    /// Offline render queue processing.
    OfflineRenderQueue,
    /// Offline render artifact purge.
    OfflineRenderPurge,
}

/// Decision the deferred service made for a work item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceDecision {
    /// Work is running now.
    Run,
    /// Work deferred until runtime pressure drops.
    Defer,
    /// Work throttled (running at reduced rate).
    Throttle,
    /// Work aborted (runtime shutting down or invalid request).
    Abort,
}

/// Priority band assigned to a deferred work item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServicePriorityBand {
    /// Must run before any audio processing starts.
    RealtimeCritical,
    /// Must run before recovery can complete.
    RecoveryCritical,
    /// User-visible background work.
    UserVisible,
    /// Low-priority housekeeping.
    Maintenance,
}

/// Why the deferred service made its decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceReason {
    /// Work was admitted and is running.
    Ready,
    /// Deferred because the realtime audio thread is active.
    RealtimeActive,
    /// Deferred due to a pending cleanup backlog.
    PendingCleanup,
    /// Deferred because recovery is degraded.
    RecoveryDegraded,
    /// Deferred because safe mode is active.
    SafeMode,
    /// Work item was invalid and could not be processed.
    InvalidRequest,
}

/// Source of the backpressure that caused a defer or throttle decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceBackpressureSource {
    /// Backpressure from the realtime audio callback.
    RealtimeAudio,
    /// Backpressure from concurrent recovery sessions.
    RecoveryOverlap,
    /// Backpressure from a cleanup work backlog.
    CleanupBacklog,
    /// Backpressure from safe mode being active.
    SafeMode,
}

/// Reason a work item was cancelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceCancellationCause {
    /// The work item request was invalid.
    InvalidRequest,
}

/// Receipt describing the deferred service outcome for the most recent work
/// admission attempt.  Inspectable via
/// `RuntimeObservationApi::get_last_deferred_service_receipt()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeferredServiceReceipt {
    /// Class of work that was submitted.
    pub work_class: RuntimeDeferredServiceClass,
    /// Decision made by the deferred service.
    pub decision: RuntimeDeferredServiceDecision,
    /// Reason for the decision.
    pub reason: RuntimeDeferredServiceReason,
    /// Priority band of the submitted work.
    pub priority_band: RuntimeDeferredServicePriorityBand,
    /// Priority band of the work currently blocking this item, if any.
    pub blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    /// Source of backpressure that caused a defer or throttle.
    pub backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    /// Whether starvation of the submitted work is at risk.
    pub starvation_risk: bool,
    /// Number of work items that are currently starved.
    pub starved_work_item_count: usize,
    /// Reason a work item was cancelled, if applicable.
    pub cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    /// Number of work items cancelled in this receipt.
    pub cancelled_work_item_count: usize,
    /// Interruption class at the time of this receipt.
    pub interruption_class: RuntimeInterruptionClass,
    /// Whether rebinding is possible at the time of this receipt.
    pub interruption_rebindable: bool,
    /// Number of work items currently queued.
    pub queued_work_item_count: usize,
    /// Number of work items admitted in this receipt.
    pub admitted_work_item_count: usize,
    /// Number of work items completed in this receipt.
    pub completed_work_item_count: usize,
    /// Number of work items deferred in this receipt.
    pub deferred_work_item_count: usize,
    /// Whether the runtime is currently running.
    pub runtime_running: bool,
    /// Whether safe mode is engaged.
    pub safe_mode_enabled: bool,
    /// Whether readiness is degraded.
    pub readiness_degraded: bool,
    /// Number of pending cleanup work items.
    pub pending_cleanup_work_items: usize,
    /// Number of pending deferred-retry work items.
    pub pending_deferred_retry_work_items: usize,
    /// Number of concurrent recovery overlap sessions.
    pub recovery_overlap_session_count: usize,
}
impl Default for RuntimeDeferredServiceReceipt {
    fn default() -> Self {
        Self {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision: RuntimeDeferredServiceDecision::Abort,
            reason: RuntimeDeferredServiceReason::InvalidRequest,
            priority_band: RuntimeDeferredServicePriorityBand::UserVisible,
            blocking_priority_band: None,
            backpressure_source: None,
            starvation_risk: false,
            starved_work_item_count: 0,
            cancellation_cause: Some(RuntimeDeferredServiceCancellationCause::InvalidRequest),
            cancelled_work_item_count: 0,
            interruption_class: RuntimeInterruptionClass::Terminal,
            interruption_rebindable: false,
            queued_work_item_count: 0,
            admitted_work_item_count: 0,
            completed_work_item_count: 0,
            deferred_work_item_count: 0,
            runtime_running: false,
            safe_mode_enabled: false,
            readiness_degraded: false,
            pending_cleanup_work_items: 0,
            pending_deferred_retry_work_items: 0,
            recovery_overlap_session_count: 0,
        }
    }
}
