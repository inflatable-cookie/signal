#[path = "offline_render/cached_render.rs"]
mod cached_render;
#[path = "offline_render/stage_model.rs"]
mod stage_model;

use super::super::*;

pub(super) fn prepare_offline_render_engine_runtime() -> (SignalRuntime, PathBuf) {
    cached_render::prepare_offline_render_engine_runtime()
}

pub(super) fn prepare_offline_render_engine_runtime_without_cached_plugin_render(
) -> (SignalRuntime, PathBuf) {
    stage_model::prepare_offline_render_engine_runtime_without_cached_plugin_render()
}
