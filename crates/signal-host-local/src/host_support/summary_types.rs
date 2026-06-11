use signal_hardware::{AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract};
use signal_runtime::{
    RuntimeExecutionTopologySummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
};

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

/// Summary of the engine blocks driven through the output pump during boot.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalEngineSummary {
    /// Number of engine blocks processed (audio pump callbacks).
    pub processed_blocks: u64,
    /// Block sequence of the last engine block, if any ran.
    pub last_block_sequence: Option<u64>,
    /// Graph ID reported by the last engine block, if any.
    pub last_graph_id: Option<String>,
    /// Peak output level from the last engine block, if measured.
    pub last_output_peak: Option<f32>,
    /// RMS output level from the last engine block, if measured.
    pub last_output_rms: Option<f32>,
}

/// Observability snapshot from a completed local host boot.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalRuntimeHostSummary {
    /// Short name of the hardware backend (e.g. `"coreaudio"`).
    pub backend_name: &'static str,
    /// Hardware configuration and diagnostic state.
    pub hardware: LocalHardwareSummary,
    /// Audio pump statistics.
    pub audio_pump: LocalAudioPumpSummary,
    /// Plugin scan roots used during the run (explicit configuration only).
    pub scan_roots: Vec<String>,
    /// Engine block statistics from the boot run.
    pub engine: LocalEngineSummary,
    /// Runtime execution topology summary (graph node count, etc.).
    pub topology: RuntimeExecutionTopologySummary,
}
