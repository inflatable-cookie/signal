use signal_hardware::{AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract};
use signal_runtime::{RuntimeExecutionTopologySummary, RuntimeHostAudioStreamState};

/// Current state of the audio output stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalAudioStreamState {
    /// The stream is not running.
    Stopped,
    /// An output stream has been negotiated and the host is live.
    Running,
    /// The stream encountered a fault and is no longer available.
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

/// Output stream state reported after a host boot.
///
/// Production playback lives in `signal-render-plane`; this summary reports
/// the negotiated control-surface stream state only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalAudioPumpSummary {
    /// Current stream state.
    pub stream_state: LocalAudioStreamState,
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

/// Observability snapshot from a completed local host boot.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalRuntimeHostSummary {
    /// Short name of the hardware backend (e.g. `"coreaudio"`).
    pub backend_name: &'static str,
    /// Hardware configuration and diagnostic state.
    pub hardware: LocalHardwareSummary,
    /// Output stream state.
    pub audio_pump: LocalAudioPumpSummary,
    /// Plugin scan roots used during the run (explicit configuration only).
    pub scan_roots: Vec<String>,
    /// Runtime execution topology summary (graph node count, etc.).
    pub topology: RuntimeExecutionTopologySummary,
}
