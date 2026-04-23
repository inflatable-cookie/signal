use signal_hardware::{AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract};
use signal_plugin::{CompletionState, PluginRenderContext, WatchdogTriggerReason};
use signal_runtime::{
    PluginSandboxInstanceStateRecord, RecoveryRestartIntent, RuntimeExecutionTopologySummary,
    RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy, StopReason,
};

/// Counts and per-block stats for the last audio payload delivered to the plugin.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPayloadSummary {
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

/// Per-block statistics for the plugin dispatch path.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPluginDispatchSummary {
    /// Processing epoch of the most recent block.
    pub processing_epoch: u64,
    /// Monotonically increasing block sequence counter.
    pub block_sequence: u64,
    /// Render context written into shared memory for the last block.
    pub render_context: PluginRenderContext,
    /// Automation value applied to the demo parameter in the last block, if any.
    pub automation_value: Option<f32>,
    /// Number of blocks where plugin render was bypassed.
    pub render_bypass_count: u32,
    /// Whether the last block was rendered in bypass mode.
    pub last_render_bypassed: bool,
    /// Plugin-reported latency in samples after the last block.
    pub last_render_latency_samples: u32,
    /// Plugin-reported tail in samples after the last block.
    pub last_render_tail_samples: u32,
}

/// Aggregate execution statistics for a completed host run.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalExecutionSummary {
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
pub struct LocalTransportSummary {
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

/// Fault counters and watchdog state from the most recent host run.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalFaultSummary {
    /// Number of audio callback deadline misses.
    pub deadline_misses: u32,
    /// Number of heartbeat timeouts.
    pub heartbeat_misses: u32,
    /// Whether the watchdog was triggered during the run.
    pub watchdog_triggered: bool,
    /// Reason the watchdog fired, if it did.
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

/// Current state of the audio output stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalAudioStreamState {
    /// The stream is not running.
    Stopped,
    /// The stream is actively processing audio callbacks.
    Running,
    /// The stream encountered a fault and is no longer producing output.
    Faulted,
}

impl From<LocalAudioStreamState> for RuntimeHostAudioStreamState {
    fn from(value: LocalAudioStreamState) -> Self {
        match value {
            LocalAudioStreamState::Stopped => RuntimeHostAudioStreamState::Stopped,
            LocalAudioStreamState::Running => RuntimeHostAudioStreamState::Running,
            LocalAudioStreamState::Faulted => RuntimeHostAudioStreamState::Faulted,
        }
    }
}

/// Policy parameters governing how the audio pump transfers data each callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalAudioTransferPolicy {
    /// Maximum number of frames the host will process in a single callback.
    pub max_callback_frames: usize,
    /// Maximum number of channels transferred per callback.
    pub max_transfer_channels: u16,
    /// If `true`, output frames not written by the engine are zeroed.
    pub zero_fill_unwritten_output: bool,
}

impl From<LocalAudioTransferPolicy> for RuntimeHostAudioTransferPolicy {
    fn from(value: LocalAudioTransferPolicy) -> Self {
        Self {
            max_callback_frames: value.max_callback_frames,
            max_transfer_channels: value.max_transfer_channels,
            zero_fill_unwritten_output: value.zero_fill_unwritten_output,
        }
    }
}

/// Snapshot of audio pump statistics after a host run.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalAudioPumpSummary {
    /// Current stream state.
    pub stream_state: LocalAudioStreamState,
    /// Transfer policy in effect during the run.
    pub transfer_policy: LocalAudioTransferPolicy,
    /// Total number of audio callbacks fired.
    pub callback_count: u64,
    /// Index of the most recent callback, if any have fired.
    pub last_callback_index: Option<u64>,
    /// Total number of frames requested across all callbacks.
    pub total_callback_frames: u64,
    /// Total number of frames that the runtime produced output for.
    pub total_runtime_output_frames: u64,
    /// Total number of output samples copied from the runtime to the callback buffer.
    pub copied_output_samples: u64,
    /// Total number of output samples zero-filled (engine produced no output).
    pub zero_filled_output_samples: u64,
    /// Total number of output samples dropped (callback buffer overflow).
    pub dropped_output_samples: u64,
    /// Peak output level from the last callback, if measured.
    pub last_callback_output_peak: Option<f32>,
    /// Graph ID from the last runtime output, if any.
    pub last_runtime_graph_id: Option<String>,
}

/// Hardware configuration in effect during the host run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalHardwareSummary {
    /// Device ID of the output device.
    pub device_id: String,
    /// Human-readable name of the output device.
    pub device_name: String,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Buffer size in frames.
    pub buffer_size: usize,
    /// Number of input channels.
    pub input_channels: u16,
    /// Number of output channels.
    pub output_channels: u16,
    /// Sample format in use.
    pub sample_format: AudioSampleFormat,
    /// Lifecycle contract agreed with the backend.
    pub lifecycle: HardwareLifecycleContract,
    /// `true` if a simulated backend was used.
    pub simulated: bool,
    /// Diagnostic snapshot from the backend at the end of the run.
    pub backend_diagnostics: HardwareDiagnosticsSnapshot,
}

/// Full observability snapshot from a completed local host run.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalRuntimeHostSummary {
    /// Short name of the hardware backend (e.g. `"coreaudio"`).
    pub backend_name: &'static str,
    /// Hardware configuration and diagnostic state.
    pub hardware: LocalHardwareSummary,
    /// Audio pump statistics.
    pub audio_pump: LocalAudioPumpSummary,
    /// Plugin scan roots used during the run.
    pub scan_roots: Vec<String>,
    /// Execution counters and last-block state.
    pub execution: LocalExecutionSummary,
    /// Shared-memory transport identity.
    pub transport: LocalTransportSummary,
    /// Runtime execution topology summary (graph node count, etc.).
    pub topology: RuntimeExecutionTopologySummary,
    /// Plugin dispatch statistics, if a plugin was active.
    pub plugin_dispatch: Option<LocalPluginDispatchSummary>,
    /// Last audio payload statistics.
    pub last_payload: LocalPayloadSummary,
    /// Fault counters and watchdog state.
    pub faults: LocalFaultSummary,
}
