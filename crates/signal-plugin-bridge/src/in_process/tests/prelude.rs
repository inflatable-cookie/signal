//! Shared imports for in-process backend unit tests.

pub(crate) use super::super::common::{
    convert_block_events, AU_EVENT_SUPPORT, CLAP_EVENT_SUPPORT, EVENT_SCRATCH_CAPACITY,
};
pub use super::super::*;
pub use signal_plugin::{MidiEvent, NoteExpressionEvent, NoteExpressionKind, PluginEvent};
pub use signal_plugin_vst3::VST3_RESTART_IO_CHANGED;
pub use signal_render_plane::RenderPluginEventSupport;
pub use std::sync::atomic::Ordering;
pub use std::sync::Arc;

pub use signal_plugin_clap::fixture::{
    compile_clap_fixture, compile_clap_instrument_fixture, rustc_available, CLAP_FIXTURE_GAIN,
};
pub use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
};
pub use signal_render_plane::{
    render_plan_to_pcm, render_plane, ChannelFormat, OfflineRenderOptions, RenderBlockPluginEvent,
    RenderEdgeSpec, RenderNoteExpressionKind, RenderPlanSpec, RenderPluginEvent,
    RenderPluginEventBuffer, RenderPluginEventKind, RenderPluginProcessor, RenderStageKind,
    RenderStageSpec,
};
