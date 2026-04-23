use signal_plugin::{CompletionState, WatchdogTriggerReason};
use signal_runtime::{PluginSandboxInstanceStateRecord, RecoveryRestartIntent, StopReason};

/// Per-block statistics for the plugin dispatch path on the server host.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerPluginDispatchSummary {
    /// Processing epoch of the most recent block.
    pub processing_epoch: u64,
    /// Monotonically increasing block sequence counter.
    pub block_sequence: u64,
    /// Automation value applied to the demo parameter in the last block, if any.
    pub automation_value: Option<f32>,
}

/// Counts and per-block stats for the last audio payload delivered to the plugin.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerPayloadSummary {
    /// Total number of events in the last payload.
    pub event_count: usize,
    /// Number of parameter-value events.
    pub parameter_event_count: usize,
    /// Number of parameter-gesture events (touch/release).
    pub parameter_gesture_event_count: usize,
    /// Number of parameter-modulation events.
    pub parameter_modulation_event_count: usize,
    /// Number of note-on/note-off events.
    pub note_event_count: usize,
    /// Number of note-expression events.
    pub note_expression_event_count: usize,
    /// Number of raw MIDI events.
    pub midi_event_count: usize,
    /// Total bytes written into the event output buffer.
    pub generated_event_bytes: u32,
    /// First sample value of the plugin output buffer, if any was produced.
    pub first_output_sample: Option<f32>,
}

/// Aggregate execution statistics for a completed server host run.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerExecutionSummary {
    /// Number of control messages sent to the sandbox.
    pub control_requests: usize,
    /// Number of control responses received from the sandbox.
    pub control_responses: usize,
    /// Number of heartbeat responses received.
    pub heartbeat_responses: usize,
    /// Number of audio blocks processed by the host.
    pub processed_blocks: usize,
    /// Number of audio blocks processed by the engine.
    pub engine_processed_blocks: usize,
    /// Name of the last control message sent.
    pub last_control_message: String,
    /// Completion state of the last block.
    pub last_completion_state: CompletionState,
    /// Block sequence number of the last processed block.
    pub last_block_sequence: u64,
    /// Graph ID from the last engine output, if any.
    pub last_engine_graph_id: Option<String>,
    /// Peak output level from the last engine block, if measured.
    pub last_engine_output_peak: Option<f32>,
    /// RMS output level from the last engine block, if measured.
    pub last_engine_output_rms: Option<f32>,
    /// Processing epoch of the current session.
    pub processing_epoch: u64,
    /// Number of recovery restarts that occurred during the run.
    pub restart_count: u64,
    /// Number of sandbox teardowns that occurred during the run.
    pub teardown_count: u64,
    /// Intent of the last recovery restart, if one occurred.
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    /// Reason the host stopped, if it has stopped.
    pub last_stop_reason: Option<StopReason>,
    /// Final plugin instance state record, if available.
    pub last_plugin_state: Option<PluginSandboxInstanceStateRecord>,
}

/// Identifies the shared-memory region used for the current sandbox session.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerTransportSummary {
    /// Sandbox ID for the active session.
    pub sandbox_id: String,
    /// Lease ID used when the shared-memory region was created.
    pub shared_memory_lease_id: String,
    /// Unique region ID assigned by the broker.
    pub shared_memory_region_id: String,
    /// Filesystem path to the backing file.
    pub shared_memory_path: String,
    /// Size of the shared-memory region in bytes.
    pub shared_memory_bytes: u32,
}

/// Fault counters and watchdog state from the most recent server host run.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerFaultSummary {
    /// Number of audio callback deadline misses.
    pub deadline_misses: u32,
    /// Number of heartbeat timeouts.
    pub heartbeat_misses: u32,
    /// Whether the watchdog was triggered during the run.
    pub watchdog_triggered: bool,
    /// Reason the watchdog fired, if it did.
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

/// Full observability snapshot from a completed server host run.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerRuntimeHostSummary {
    /// Plugin scan roots used during the run.
    pub scan_roots: Vec<String>,
    /// Execution counters and last-block state.
    pub execution: ServerExecutionSummary,
    /// Shared-memory transport identity.
    pub transport: ServerTransportSummary,
    /// Plugin dispatch statistics, if a plugin was active.
    pub plugin_dispatch: Option<ServerPluginDispatchSummary>,
    /// Last audio payload statistics.
    pub last_payload: ServerPayloadSummary,
    /// Fault counters and watchdog state.
    pub faults: ServerFaultSummary,
}
