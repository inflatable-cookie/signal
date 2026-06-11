//! Output-stream contract: the boundary where Signal hands rendered audio to
//! the operating system.
//!
//! [`HardwareBackend`](crate::HardwareBackend) ends at stream *negotiation*;
//! this contract covers actually opening and running a stream. Backends that
//! can produce sound implement [`OutputStreamBackend`].
//!
//! # Real-time contract
//!
//! The render callback runs on the OS audio thread. Implementations MUST
//! invoke it without holding locks, and callers MUST ensure the callback
//! itself never allocates, locks, blocks, or performs I/O. Cross-thread
//! hand-off into the callback belongs in lock-free structures (SPSC rings,
//! atomics), never mutexes.

/// Lifecycle state of an open output stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStreamState {
    /// Stream is running and invoking the render callback.
    Running,
    /// Stream has been stopped (explicitly or by drop).
    Stopped,
    /// The backend reported an unrecoverable stream error.
    Faulted,
}

/// Error opening or operating an output stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputStreamError {
    /// Human-readable description of the failure.
    pub message: String,
}

impl OutputStreamError {
    /// Build an error from any displayable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for OutputStreamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "output stream error: {}", self.message)
    }
}

impl std::error::Error for OutputStreamError {}

/// Requested shape for an output stream. The backend negotiates against the
/// device's supported configurations and the handle reports the values the
/// stream actually runs at; callers MUST read the negotiated rate/channels
/// back from the handle rather than assuming the request was honoured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputStreamSpec {
    /// Requested sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Requested interleaved channel count.
    pub channels: u16,
    /// Preferred callback buffer size in frames; `None` accepts the device
    /// default.
    pub buffer_frames: Option<u32>,
}

/// Render callback: fill `frames` (interleaved f32, `channels` wide) for the
/// current callback quantum. Runs on the audio thread — see the module-level
/// real-time contract.
pub type OutputRenderFn = Box<dyn FnMut(&mut [f32]) + Send + 'static>;

/// Handle to a running output stream. Dropping the handle stops the stream.
pub trait OutputStreamHandle: Send {
    /// Current lifecycle state of the stream.
    fn state(&self) -> OutputStreamState;
    /// The sample rate the stream actually runs at.
    fn sample_rate_hz(&self) -> u32;
    /// The channel count the stream actually runs with.
    fn channels(&self) -> u16;
    /// Human-readable detail of the most recent backend-reported stream
    /// error, when the backend captures one (typically alongside a
    /// [`OutputStreamState::Faulted`] transition). Default: `None` for
    /// backends without error capture.
    fn last_error(&self) -> Option<String> {
        None
    }
}

/// A backend capable of opening real output streams.
pub trait OutputStreamBackend {
    /// Open and start an output stream that pulls audio from `render`.
    fn open_output_stream(
        &self,
        spec: OutputStreamSpec,
        render: OutputRenderFn,
    ) -> Result<Box<dyn OutputStreamHandle>, OutputStreamError>;
}
