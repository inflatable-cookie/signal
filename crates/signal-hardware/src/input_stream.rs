//! Input-stream contract: the boundary where the operating system hands
//! captured audio to Signal.
//!
//! Mirror of the output contract in [`crate::output_stream`]: backends that
//! can capture sound implement [`InputStreamBackend`], negotiation semantics
//! are identical (the handle reports the values the stream actually runs at,
//! never the requested ones).
//!
//! # Real-time contract
//!
//! The capture callback runs on the OS audio thread. Implementations MUST
//! invoke it without holding locks, and callers MUST ensure the callback
//! itself never allocates, locks, blocks, or performs I/O. Cross-thread
//! hand-off out of the callback belongs in lock-free structures (SPSC rings,
//! atomics), never mutexes.

/// Lifecycle state of an open input stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputStreamState {
    /// Stream is running and invoking the capture callback.
    Running,
    /// Stream has been stopped (explicitly or by drop).
    Stopped,
    /// The backend reported an unrecoverable stream error.
    Faulted,
}

/// Error opening or operating an input stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputStreamError {
    /// Human-readable description of the failure.
    pub message: String,
}

impl InputStreamError {
    /// Build an error from any displayable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for InputStreamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "input stream error: {}", self.message)
    }
}

impl std::error::Error for InputStreamError {}

/// Requested shape for an input stream. The backend negotiates against the
/// device's supported configurations and the handle reports the values the
/// stream actually runs at; callers MUST read the negotiated rate/channels
/// back from the handle rather than assuming the request was honoured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputStreamSpec {
    /// Requested sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Requested interleaved channel count.
    pub channels: u16,
    /// Preferred callback buffer size in frames; `None` accepts the device
    /// default.
    pub buffer_frames: Option<u32>,
}

/// Capture callback: consume `frames` (interleaved f32, `channels` wide) for
/// the current callback quantum. Runs on the audio thread — see the
/// module-level real-time contract.
pub type InputCaptureFn = Box<dyn FnMut(&[f32]) + Send + 'static>;

/// Handle to a running input stream. Dropping the handle stops the stream.
pub trait InputStreamHandle: Send {
    /// Current lifecycle state of the stream.
    fn state(&self) -> InputStreamState;
    /// The sample rate the stream actually runs at.
    fn sample_rate_hz(&self) -> u32;
    /// The channel count the stream actually runs with.
    fn channels(&self) -> u16;
    /// Human-readable detail of the most recent backend-reported stream
    /// error, when the backend captures one (typically alongside an
    /// [`InputStreamState::Faulted`] transition). Default: `None` for
    /// backends without error capture.
    fn last_error(&self) -> Option<String> {
        None
    }
    /// Most recently observed input latency in microseconds: the gap between
    /// the first frame of a buffer being captured at the ADC and the capture
    /// callback delivering it, as reported by the backend's stream
    /// timestamps. `None` until the backend has observed a usable timestamp
    /// pair (the first callbacks may not carry one) or for backends without
    /// timestamp support. Default: `None`.
    fn input_latency_micros(&self) -> Option<u64> {
        None
    }
    /// OS-reported name of the device this stream actually opened on, when
    /// the backend records it. Hosts compare this against the current
    /// default device to detect that the OS default moved. Default: `None`
    /// for backends without device identity.
    fn device_name(&self) -> Option<String> {
        None
    }
}

/// A backend capable of opening real input streams.
pub trait InputStreamBackend {
    /// Open and start an input stream that pushes captured audio into
    /// `capture`.
    fn open_input_stream(
        &self,
        spec: InputStreamSpec,
        capture: InputCaptureFn,
    ) -> Result<Box<dyn InputStreamHandle>, InputStreamError>;
}
