//! Compiled plan node and source types (render-side data, preallocated at compile time).

use std::sync::Arc;

use signal_dsp::PolyphaseInterpolationTable;

use crate::stream::STREAM_HELD_CHUNK_SLOTS;
use crate::{
    RenderBlockPluginEvent, RenderLiveInputHandle, RenderNoteBuffer, RenderPluginEvent,
    RenderPluginProcessor, RenderSampleBuffer, RenderStreamHandle, StreamChunk,
};

/// Declick fade applied inside each clip window edge (shortened for tiny
/// windows so short clips stay audible).
pub(crate) const CLIP_EDGE_FADE_FRAMES: u64 = 32;

pub(crate) enum CompiledSource {
    Silence,
    Tone {
        phase: f32,
        step: f32,
    },
    Samples {
        buffer: RenderSampleBuffer,
        /// Source frames advanced per stream frame (rate ratio).
        step: f64,
        /// Repeat from the source start once exhausted.
        loop_source: bool,
        /// Polyphase windowed-sinc table for rate-converted playback; `None`
        /// at 1:1, where samples are read directly.
        table: Option<PolyphaseInterpolationTable>,
        /// Source→stage channel up/down-mix, row-major `source_channels ×
        /// dest_channels`. `None` when the counts match (direct per-channel
        /// read, byte-identical to the historical stereo path).
        channel_adapter: Option<Vec<f32>>,
    },
    Stream {
        handle: RenderStreamHandle,
        /// Chunks currently held for rendering; drained from the handle's
        /// mailbox between blocks, returned via the retired mailbox once
        /// behind the playhead or outside the seek window. Moves across
        /// plan swaps through the clip inheritance map so an identity
        /// recompile never drops the read-ahead.
        held: [Option<StreamChunk>; STREAM_HELD_CHUNK_SLOTS],
        /// Source frames advanced per stream frame (rate ratio).
        step: f64,
        /// Polyphase windowed-sinc table for rate-converted playback; `None`
        /// at 1:1.
        table: Option<PolyphaseInterpolationTable>,
    },
    /// Live input monitor: drains the handle's ring each block. All state
    /// (ring, underrun counter) lives in the Arc-shared handle, so plan
    /// swaps carry the feed inherently — nothing to migrate or reset.
    LiveInput {
        handle: RenderLiveInputHandle,
    },
    /// Built-in instrument: stateless additive sine voices. Everything a
    /// voice needs is a pure function of the stream position, so there is
    /// nothing to inherit across plan swaps or seeks.
    Notes {
        buffer: RenderNoteBuffer,
        /// Per-note phase step in radians per stream frame, precomputed at
        /// compile (parallel to `buffer.notes`) — no per-sample
        /// transcendentals beyond `sin()`.
        steps: Arc<[f64]>,
        /// Attack ramp length at the plan rate.
        attack_frames: u64,
        /// Release tail length at the plan rate.
        release_frames: u64,
        /// Longest note duration in the buffer: bounds the sorted-scan
        /// lookback window (a sounding note starts at most this many frames
        /// plus the release tail before the block).
        max_duration_frames: u64,
    },
}

pub(crate) struct CompiledClip {
    pub(crate) start_frames: u64,
    pub(crate) end_frames: u64,
    /// Declick fade length at each window edge, shortened for tiny windows.
    pub(crate) edge_fade_frames: u64,
    /// Equal-power fade-in span from `start_frames` (0 = declick only on
    /// that side). Clamped to the window length at compile.
    pub(crate) fade_in_frames: u64,
    /// Equal-power fade-out span ending at `end_frames` (0 = declick only).
    pub(crate) fade_out_frames: u64,
    /// Stable identity, read control-side when building inheritance maps.
    pub(crate) clip_id: u64,
    pub(crate) source: CompiledSource,
}

/// One compiled input edge. `source_index` is a position in the plan's
/// topologically-ordered stage list and is always strictly less than the
/// consuming stage's position, so the executor can split-borrow.
pub(crate) struct CompiledInput {
    pub(crate) source_index: usize,
    pub(crate) source_channels: usize,
    /// Row-major `source_channels × dest_channels`; edge gain folded in.
    pub(crate) matrix: Vec<f32>,
}

pub(crate) struct CompiledNode {
    /// Matches stage state (smoothed gain, tone phase) across plan swaps.
    pub(crate) stage_id: u64,
    pub(crate) channels: usize,
    /// Gain the stage is moving toward (spec value).
    pub(crate) gain_target: f32,
    /// Smoothed gain as currently applied; inherited across plan swaps so
    /// gain edits never step.
    pub(crate) gain_current: f32,
    /// Per-block smoothed-gain interpolation, written when the stage renders
    /// and read wherever its output is consumed (edges, boundary).
    pub(crate) block_gain_begin: f32,
    pub(crate) block_gain_slope: f32,
    /// Sorted automation breakpoints `(frame, gain)`; empty = no automation
    /// (static `gain_target` smoothing applies).
    pub(crate) gain_envelope: Vec<(u64, f32)>,
    /// Clip content (Source stages; empty for Sum/Output).
    pub(crate) clips: Vec<CompiledClip>,
    pub(crate) inputs: Vec<CompiledInput>,
    /// Plugin processor applied to the summed scratch (Sum stages only).
    pub(crate) processor: Option<RenderPluginProcessor>,
    /// Compiled plugin event stream (absolute frames, sorted); empty when
    /// the stage carries none. Shared with the spec's Arc — no copy.
    pub(crate) events: Arc<[RenderPluginEvent]>,
    /// Preallocated per-block event slice handed to the processor
    /// ([`PLUGIN_EVENTS_PER_BLOCK_CAPACITY`]); the audio thread only ever
    /// clears and pushes within capacity.
    pub(crate) event_scratch: Vec<RenderBlockPluginEvent>,
    /// Whether this stage accepts host-pushed live events (g13.018).
    pub(crate) accepts_live_events: bool,
    /// Live-event ring: preallocated at compile
    /// ([`LIVE_EVENT_RING_CAPACITY`]) for accepting stages, empty otherwise.
    /// `PushLiveEvents` appends within capacity (overflow drops and counts);
    /// every rendered block drains and clears it. The audio thread only ever
    /// pushes within capacity and clears — never allocates.
    pub(crate) live_events: Vec<RenderPluginEvent>,
    /// Interleaved fixed-size delay ring plus its next sample position.
    /// Empty for non-delay stages and zero-frame delays.
    pub(crate) delay_ring: Vec<f32>,
    pub(crate) delay_cursor: usize,
    /// Interleaved scratch at the stage's format: `MAX_BLOCK_FRAMES × channels`.
    pub(crate) scratch: Vec<f32>,
}
