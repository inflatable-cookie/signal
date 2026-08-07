//! Real-time render plane: the control/render split for Signal playback.
//!
//! The runtime's `process_engine_block` is a proof/observation API: every
//! block it allocates (graph-id strings, summary recomputation, snapshot
//! clones, by-value buffers), which disqualifies it from the audio thread.
//! This crate is the execution substrate that *is* allowed there:
//!
//! - the **control side** compiles a [`RenderPlanSpec`] into an immutable,
//!   fully preallocated plan and hands it across a bounded lock-free mailbox;
//! - the **render side** ([`RenderPlaneExecutor`]) swaps plans atomically
//!   between blocks and executes [`RenderPlaneExecutor::render_block`] with a
//!   hard no-alloc / no-lock / no-I/O contract;
//! - retired plans travel **back** to the control side for deallocation —
//!   nothing is ever freed on the audio thread.
//!
//! Both mailboxes are `std::sync::mpsc::sync_channel`s: bounded array-backed
//! channels whose send/receive operations neither allocate nor free.
//!
//! Plans are **graphs**, not lane lists: a spec is a set of format-typed
//! stages ([`RenderStageSpec`] — Source, Sum, exactly one Output) connected
//! by edges that each carry a gain and an N×M channel mix matrix. Compile
//! topologically sorts the graph into a flat execution schedule, preallocates
//! a per-stage scratch buffer ([`MAX_BLOCK_FRAMES`] × stage channels — the
//! buffer pool *is* the plan), and resolves every edge's matrix (explicit
//! coefficients, or a default adapter from `signal_dsp::default_adapter_matrix`
//! when formats differ). Per chorus a14 the graph is channel-format-typed:
//! nothing in it assumes stereo. The only forced collapse is the hardware
//! boundary, where the master stage's format is adapted to the negotiated
//! stream format (downmix matrix when the device is narrower, silence-filled
//! extra channels when wider).

#![warn(missing_docs)]

mod binaural_bank;
pub use binaural_bank::{BankHrir, BankSound, BinauralVoiceBank};
mod convolution_reverb;
pub use convolution_reverb::ConvolutionReverbProcessor;
mod offline;

pub use offline::{
    apply_soft_limiter_to_pcm, build_offline_stretch_artifact_cache_handoff,
    build_offline_stretch_artifact_render_source, materialize_offline_stretch_artifact_pcm,
    plan_offline_stretch_artifact, render_plan_to_pcm, write_wav, OfflineRenderOptions,
    OfflineRenderOutput, OfflineStretchArtifactBuildRequest, OfflineStretchArtifactCacheDecision,
    OfflineStretchArtifactCacheDecisionKind, OfflineStretchArtifactCacheHandoff,
    OfflineStretchArtifactMaterializationReceipt, OfflineStretchArtifactMaterializeError,
    OfflineStretchArtifactPcm, OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError,
    OfflineStretchArtifactReadiness, OfflineStretchArtifactRenderCacheBridge,
    OfflineStretchArtifactRenderSource, OfflineStretchArtifactScope, WavBitDepth,
};

mod live_input;
mod notes;
mod plan;
mod plan_render;
mod plan_spec;
mod plane;
mod plugin_events;
mod plugin_processor;
mod sample_buffer;
mod stream;

pub use live_input::{
    render_live_input, LiveInputFeeder, RenderLiveInputHandle, LIVE_INPUT_DEFAULT_CAPACITY_FRAMES,
};
pub use notes::{RenderNote, RenderNoteBuffer, RenderPitchIntent, NOTE_POLYPHONY_LIMIT};
pub use plan::RenderPlan;
pub use plan_spec::{
    ChannelFormat, ChannelLayout, RenderClipSpec, RenderEdgeSpec, RenderLimiterSpec,
    RenderParamEnvelope, RenderPlanCompileError, RenderPlanSpec, RenderSource, RenderStageKind,
    RenderStageSpec,
};
pub use plane::{render_plane, RenderPlaneController, RenderPlaneError, RenderPlaneExecutor};
pub use plugin_events::{
    RenderBlockPluginEvent, RenderNoteExpressionKind, RenderPluginEvent, RenderPluginEventBuffer,
    RenderPluginEventKind, RenderPluginEventSupport, RenderVoiceParam, LIVE_EVENT_PUSH_CAPACITY,
    LIVE_EVENT_RING_CAPACITY, PLUGIN_EVENTS_PER_BLOCK_CAPACITY,
};
pub use plugin_processor::{PluginBlockProcessor, RenderPluginProcessor};
pub use sample_buffer::{RenderSampleBuffer, MAX_BLOCK_FRAMES, METER_SLOT_CAPACITY};
pub use stream::{
    render_stream, RenderStreamHandle, StreamChunk, StreamFeeder, STREAM_RETIRE_LOOKAHEAD_FRAMES,
};

#[cfg(test)]
mod tests;
