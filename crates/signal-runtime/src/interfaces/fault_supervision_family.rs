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
