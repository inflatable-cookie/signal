use std::path::PathBuf;

use super::{AudioBuffer, SignalRuntime};

#[path = "runtime_fixtures/offline_render.rs"]
mod offline_render;
#[path = "runtime_fixtures/realtime.rs"]
mod realtime;

pub(super) fn prepare_offline_render_engine_runtime() -> (SignalRuntime, PathBuf) {
    offline_render::prepare_offline_render_engine_runtime()
}

pub(super) fn prepare_offline_render_engine_runtime_without_cached_plugin_render(
) -> (SignalRuntime, PathBuf) {
    offline_render::prepare_offline_render_engine_runtime_without_cached_plugin_render()
}

pub(super) fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    realtime::filled_stereo_buffer(sample_rate_hz, frames, value)
}

pub(super) fn prepare_sidechain_runtime() -> SignalRuntime {
    realtime::prepare_sidechain_runtime()
}

pub(super) fn prepare_spatial_runtime() -> SignalRuntime {
    realtime::prepare_spatial_runtime()
}
