#[path = "realtime/buffers.rs"]
mod buffers;
#[path = "realtime/sidechain.rs"]
mod sidechain;
#[path = "realtime/spatial.rs"]
mod spatial;

use super::super::*;

pub(super) fn prepare_sidechain_runtime() -> SignalRuntime {
    sidechain::prepare_sidechain_runtime()
}

pub(super) fn prepare_spatial_runtime() -> SignalRuntime {
    spatial::prepare_spatial_runtime()
}

pub(super) fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    buffers::filled_stereo_buffer(sample_rate_hz, frames, value)
}
