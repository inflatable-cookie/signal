use std::path::PathBuf;

use super::{AudioBuffer, SignalRuntime};

#[path = "runtime_fixtures/realtime.rs"]
mod realtime;

pub(super) fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    realtime::filled_stereo_buffer(sample_rate_hz, frames, value)
}

pub(super) fn prepare_sidechain_runtime() -> SignalRuntime {
    realtime::prepare_sidechain_runtime()
}

pub(super) fn prepare_spatial_runtime() -> SignalRuntime {
    realtime::prepare_spatial_runtime()
}
