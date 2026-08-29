//! Placeholder stream handle used after capture stop.

use crate::input_stream::InputStreamHandle;

/// Placeholder handle installed by `CaptureSession::stop` after the real
/// stream is dropped (the box must hold something while the report is
/// assembled).
pub(crate) struct StoppedStream {
    pub(crate) sample_rate_hz: u32,
    pub(crate) channels: u16,
}

impl InputStreamHandle for StoppedStream {
    fn state(&self) -> crate::input_stream::InputStreamState {
        crate::input_stream::InputStreamState::Stopped
    }
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }
    fn channels(&self) -> u16 {
        self.channels
    }
}
