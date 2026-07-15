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

mod offline;

pub use offline::{
    build_offline_stretch_artifact_cache_handoff_with_synthetic_policy,
    build_offline_stretch_artifact_pcm_with_synthetic_policy,
    build_offline_stretch_artifact_render_source_with_synthetic_policy,
    materialize_offline_stretch_artifact_pcm, plan_offline_stretch_artifact,
    plan_offline_stretch_artifact_with_synthetic_policy, render_plan_to_pcm, write_wav,
    OfflineRenderOptions, OfflineRenderOutput, OfflineStretchArtifactBuildRequest,
    OfflineStretchArtifactCacheDecision, OfflineStretchArtifactCacheDecisionKind,
    OfflineStretchArtifactCacheHandoff, OfflineStretchArtifactMaterializationReceipt,
    OfflineStretchArtifactMaterializeError, OfflineStretchArtifactPcm, OfflineStretchArtifactPlan,
    OfflineStretchArtifactPlanError, OfflineStretchArtifactPolicyRequest,
    OfflineStretchArtifactReadiness, OfflineStretchArtifactRenderCacheBridge,
    OfflineStretchArtifactRenderSource, OfflineStretchArtifactScope, WavBitDepth,
};

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;
use std::time::Instant;

use signal_dsp::{
    default_adapter_matrix, DenormalGuard, LimiterState, PolyphaseInterpolationTable,
};
use signal_primitives::SpscRing;

/// Interpolation table shape for rate-converted media playback. 16 taps ×
/// 512 phases ≈ 32 KB per distinct cutoff; built once per plan compile.
const RESAMPLE_TAPS: usize = 16;
const RESAMPLE_PHASES: usize = 512;

/// Largest callback quantum the plan's scratch buffers are sized for. Every
/// stage owns `MAX_BLOCK_FRAMES × channels` samples of scratch, preallocated
/// at compile; `render_block` clamps (and debug-asserts) the frame count.
pub const MAX_BLOCK_FRAMES: usize = 4096;

/// Fixed capacity of the shared per-stage meter table. Plans with more
/// stages than this render normally but stages past the capacity are
/// silently unmetered (slot i meters topological stage i; there is no
/// overflow signalling — meters are cosmetic).
pub const METER_SLOT_CAPACITY: usize = 256;

/// Callback-interval factor past which a missed deadline is inferred: an
/// interval since the previous callback longer than `1.5 ×` the block
/// duration at the plan rate counts as an xrun.
const XRUN_INTERVAL_FACTOR: f64 = 1.5;

/// Shared immutable sample data: interleaved f32 at a source rate. `channels`
/// gives the interleaving stride — 1 (mono), 2 (stereo), or more. When a clip's
/// source channel count differs from its stage's format, the render adapts it
/// with the standard up/down-mix coefficients (`signal_dsp::default_adapter_matrix`).
///
/// Equality is pointer-based so plan specs containing large buffers compare
/// cheaply and a cached buffer keeps reinstalls idempotent.
#[derive(Clone, Debug)]
pub struct RenderSampleBuffer {
    /// Source sample rate of the buffer.
    pub sample_rate_hz: u32,
    /// Interleaved channel count (1 = mono, 2 = stereo, …). `frames.len()` must
    /// be a multiple of this.
    pub channels: u16,
    /// Interleaved frames (`frame_count = frames.len() / channels`).
    pub frames: Arc<[f32]>,
}

impl PartialEq for RenderSampleBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate_hz == other.sample_rate_hz
            && self.channels == other.channels
            && Arc::ptr_eq(&self.frames, &other.frames)
    }
}

impl RenderSampleBuffer {
    /// Build a sample buffer of the given interleaved channel count.
    pub fn new(sample_rate_hz: u32, channels: u16, frames: Arc<[f32]>) -> Self {
        Self {
            sample_rate_hz,
            channels: channels.max(1),
            frames,
        }
    }

    /// Build a mono sample buffer.
    pub fn mono(sample_rate_hz: u32, frames: Arc<[f32]>) -> Self {
        Self::new(sample_rate_hz, 1, frames)
    }

    /// Build a stereo (interleaved L/R) sample buffer.
    pub fn stereo(sample_rate_hz: u32, frames: Arc<[f32]>) -> Self {
        Self::new(sample_rate_hz, 2, frames)
    }

    /// Number of frames in the buffer (`frames.len() / channels`).
    pub fn frame_count(&self) -> usize {
        self.frames.len() / self.channels.max(1) as usize
    }
}

// ── Note sources (built-in instrument, stateless) ───────────────────────────

/// Attack ramp length for note voices, in seconds (linear 0 → 1).
const NOTE_ATTACK_SECONDS: f64 = 0.003;
/// Release tail length after a note's end, in seconds (linear 1 → 0).
const NOTE_RELEASE_SECONDS: f64 = 0.040;
/// Most notes rendered simultaneously per block per clip. The overlap scan
/// walks notes in start order, so when more than this many notes sound in
/// one block the EARLIEST-STARTED ones render and the rest are skipped —
/// deterministic, and counted per block (a block-granular approximation of
/// true simultaneity).
pub const NOTE_POLYPHONY_LIMIT: usize = 32;

/// Explicit per-note pitch override (loophole g12.034 rider): what
/// frequency actually sounds, kept separate from the note's degree (row)
/// identity. `None` on the note derives the frequency from the degree via
/// the tuning default (12-EDO today).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderPitchIntent {
    /// Absolute frequency in hertz.
    FrequencyHz(f64),
    /// Offset in cents from the degree's tuning-derived frequency.
    CentsOffset(f64),
}

/// One note event: clip-relative timing on the stream clock, degree (row)
/// identity + optional pitch intent, normalized velocity. Velocity is the
/// voice's sustain amplitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderNote {
    /// First frame of the note, relative to the owning clip's window start.
    pub start_frame: u64,
    /// Note length in frames (the release tail extends past this).
    pub duration_frames: u64,
    /// Degree (row/scale-step) identity. Under the 12-EDO default tuning a
    /// degree IS the MIDI pitch (69 = A4 = 440 Hz).
    pub degree: i32,
    /// `None` = derive the frequency from `degree` via the 12-EDO default;
    /// `Some` = explicit override.
    pub pitch_intent: Option<RenderPitchIntent>,
    /// Normalized velocity in `0..=1`, applied as the voice amplitude.
    pub velocity: f32,
}

impl RenderNote {
    /// The 12-EDO default frequency for a degree:
    /// `440 · 2^((degree − 69) / 12)`.
    fn degree_default_frequency_hz(degree: i32) -> f64 {
        440.0 * f64::powf(2.0, (f64::from(degree) - 69.0) / 12.0)
    }

    /// Oscillator frequency in hertz, derived from `(degree, pitch_intent)`:
    /// no intent means the degree's 12-EDO default — bit-identical to the
    /// pre-widening `440 · 2^((pitch − 69) / 12)` path.
    pub fn frequency_hz(&self) -> f64 {
        match self.pitch_intent {
            None => Self::degree_default_frequency_hz(self.degree),
            Some(RenderPitchIntent::FrequencyHz(hz)) => hz,
            Some(RenderPitchIntent::CentsOffset(cents)) => {
                Self::degree_default_frequency_hz(self.degree) * f64::powf(2.0, cents / 1200.0)
            }
        }
    }
}

/// Shared immutable note list for one clip, sorted by `start_frame`
/// (compile validates and rejects unsorted buffers).
///
/// Equality is pointer-based (like [`RenderSampleBuffer`]): hosts cache one
/// buffer per clip content so recompiled specs stay idempotent.
#[derive(Clone, Debug)]
pub struct RenderNoteBuffer {
    /// Notes sorted by `start_frame`.
    pub notes: Arc<[RenderNote]>,
}

impl PartialEq for RenderNoteBuffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.notes, &other.notes)
    }
}

// ── Plugin event delivery (g12.034 follow-up) ───────────────────────────────
//
// Stages carrying a plugin processor may also carry a compiled event stream
// (notes + MIDI performance data from the consumer's model). The executor
// slices the
// stream per block and hands the slice to the processor with intra-block
// sample offsets; the processing backend converts to each plugin format's
// native event lists AT THAT BOUNDARY (the model value stays 32-bit float
// 0..1 here — no 7-bit MIDI quantization inside the engine).

/// What one compiled plugin event does. Values stay normalized floats;
/// downconversion to MIDI 1.0 bytes (where a format needs it) happens in
/// the plugin backend, never here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderPluginEventKind {
    /// Note starts sounding.
    NoteOn {
        /// MIDI key number (0–127).
        key: u8,
        /// Velocity in `0..=1`.
        velocity: f32,
    },
    /// Note released.
    NoteOff {
        /// MIDI key number (0–127).
        key: u8,
    },
    /// Continuous-controller change.
    ControlChange {
        /// MIDI controller number (0–127).
        controller: u8,
        /// Controller value in `0..=1` (32-bit; quantized only at the
        /// plugin-format boundary).
        value: f32,
    },
    /// Channel pitch bend, before any instrument-specific bend range.
    PitchBend {
        /// Bipolar wheel position in `-1..=1`; zero is centre.
        value: f32,
    },
    /// Channel pressure (channel aftertouch).
    ChannelPressure {
        /// Pressure in `0..=1`.
        value: f32,
    },
    /// Per-note expression targeting the active note on `channel`/`key`.
    NoteExpression {
        /// MIDI key number (0-127).
        key: u8,
        /// Expression dimension.
        expression: RenderNoteExpressionKind,
        /// Pressure/timbre use `0..=1`; tuning uses cents.
        value: f32,
    },
}

/// Format-neutral per-note expression dimensions supported by the render plane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderNoteExpressionKind {
    /// Per-note pressure / polyphonic aftertouch.
    Pressure,
    /// Per-note spectral brightness or sound character.
    Timbre,
    /// Per-note pitch deviation in cents.
    Tuning,
}

/// Event families one live plugin processor backend can deliver to its
/// native format. This describes host transport support, not the plugin's
/// catalog-declared input capability.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderPluginEventSupport {
    /// Note-on and note-off delivery.
    pub notes: bool,
    /// Continuous-controller delivery.
    pub control_change: bool,
    /// Channel pitch-bend delivery.
    pub pitch_bend: bool,
    /// Channel-pressure delivery.
    pub channel_pressure: bool,
    /// Per-note pressure, timbre, and tuning delivery.
    pub note_expression: bool,
}

/// One compiled plugin event on the ABSOLUTE stream clock (the consumer
/// projects ticks to samples at compile; clip windows are already applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPluginEvent {
    /// Absolute stream-clock frame the event fires at.
    pub frame: u64,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// What the event does.
    pub kind: RenderPluginEventKind,
}

/// Shared immutable event stream for one processor stage, sorted by `frame`
/// (compile validates and rejects unsorted buffers).
///
/// Equality is pointer-based (like [`RenderNoteBuffer`]): hosts cache one
/// buffer per content hash so recompiled specs stay idempotent.
#[derive(Clone, Debug)]
pub struct RenderPluginEventBuffer {
    /// Events sorted by `frame`.
    pub events: Arc<[RenderPluginEvent]>,
}

impl PartialEq for RenderPluginEventBuffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.events, &other.events)
    }
}

/// One plugin event scheduled within the CURRENT block: the absolute frame
/// resolved to an offset from the block (or loop-segment) start. This is
/// the slice element handed to [`PluginBlockProcessor::process_with_events`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBlockPluginEvent {
    /// Frame offset within the current block at which the event fires.
    pub offset_frames: u32,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// What the event does.
    pub kind: RenderPluginEventKind,
}

/// Per-stage, per-block plugin event capacity. The executor's event scratch
/// is preallocated at this size (audio thread never allocates); events past
/// the cap in one block are dropped, earliest-first wins.
pub const PLUGIN_EVENTS_PER_BLOCK_CAPACITY: usize = 1024;

/// Reconcile plugin note, controller, and expression state across a non-
/// contiguous transport jump. Fixed bounded state and the caller's
/// preallocated scratch keep this audio-thread safe. Note IDs are not yet
/// distinct, so note expression is tracked by channel/key.
fn append_plugin_state_chase(
    events: &[RenderPluginEvent],
    from_frame: u64,
    to_frame: u64,
    offset_frames: u32,
    scratch: &mut Vec<RenderBlockPluginEvent>,
) {
    let mut from_active = [0u128; 16];
    let mut to_active = [0u128; 16];
    let mut to_velocity = [[0.0f32; 128]; 16];
    let mut cc_seen = [0u128; 16];
    let mut cc_values = [[0.0f32; 128]; 16];
    let mut bend_seen = 0u16;
    let mut bend_values = [0.0f32; 16];
    let mut pressure_seen = 0u16;
    let mut pressure_values = [0.0f32; 16];
    let mut expression_seen = [[0u128; 16]; 3];
    let mut expression_values = [[[0.0f32; 128]; 16]; 3];
    let scan_end = from_frame.max(to_frame);
    for event in events.iter().take_while(|event| event.frame < scan_end) {
        let channel = usize::from(event.channel.min(15));
        match event.kind {
            RenderPluginEventKind::NoteOn { key, velocity } => {
                let bit = 1u128 << key;
                if event.frame < from_frame {
                    from_active[channel] |= bit;
                }
                if event.frame < to_frame {
                    to_active[channel] |= bit;
                    to_velocity[channel][usize::from(key)] = velocity;
                }
            }
            RenderPluginEventKind::NoteOff { key } => {
                let bit = 1u128 << key;
                if event.frame < from_frame {
                    from_active[channel] &= !bit;
                }
                if event.frame < to_frame {
                    to_active[channel] &= !bit;
                }
            }
            RenderPluginEventKind::ControlChange { controller, value }
                if event.frame < to_frame =>
            {
                cc_seen[channel] |= 1u128 << controller;
                cc_values[channel][usize::from(controller)] = value;
            }
            RenderPluginEventKind::PitchBend { value } if event.frame < to_frame => {
                bend_seen |= 1u16 << channel;
                bend_values[channel] = value;
            }
            RenderPluginEventKind::ChannelPressure { value } if event.frame < to_frame => {
                pressure_seen |= 1u16 << channel;
                pressure_values[channel] = value;
            }
            RenderPluginEventKind::NoteExpression {
                key,
                expression,
                value,
            } if event.frame < to_frame => {
                let index = match expression {
                    RenderNoteExpressionKind::Pressure => 0,
                    RenderNoteExpressionKind::Timbre => 1,
                    RenderNoteExpressionKind::Tuning => 2,
                };
                expression_seen[index][channel] |= 1u128 << key;
                expression_values[index][channel][usize::from(key)] = value;
            }
            _ => {}
        }
    }
    for (channel, active) in from_active.into_iter().enumerate() {
        for key in 0..128u8 {
            if active & (1u128 << key) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::NoteOff { key },
            });
        }
    }
    // Channel-wide state must precede destination attacks.
    for channel in 0..16 {
        for controller in 0..128u8 {
            if cc_seen[channel] & (1u128 << controller) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::ControlChange {
                    controller,
                    value: cc_values[channel][usize::from(controller)],
                },
            });
        }
        for (seen, kind) in [
            (
                bend_seen,
                RenderPluginEventKind::PitchBend {
                    value: bend_values[channel],
                },
            ),
            (
                pressure_seen,
                RenderPluginEventKind::ChannelPressure {
                    value: pressure_values[channel],
                },
            ),
        ] {
            if seen & (1u16 << channel) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind,
            });
        }
    }
    for (channel, active) in to_active.into_iter().enumerate() {
        for key in 0..128u8 {
            if active & (1u128 << key) == 0 {
                continue;
            }
            if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                return;
            }
            scratch.push(RenderBlockPluginEvent {
                offset_frames,
                channel: channel as u8,
                kind: RenderPluginEventKind::NoteOn {
                    key,
                    velocity: to_velocity[channel][usize::from(key)],
                },
            });
        }
    }
    // Per-note state follows attacks so formats that require an active note
    // can address the freshly chased voice.
    for (index, expression) in [
        RenderNoteExpressionKind::Pressure,
        RenderNoteExpressionKind::Timbre,
        RenderNoteExpressionKind::Tuning,
    ]
    .into_iter()
    .enumerate()
    {
        for channel in 0..16 {
            let active = to_active[channel] & expression_seen[index][channel];
            for key in 0..128u8 {
                if active & (1u128 << key) == 0 {
                    continue;
                }
                if scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                    return;
                }
                scratch.push(RenderBlockPluginEvent {
                    offset_frames,
                    channel: channel as u8,
                    kind: RenderPluginEventKind::NoteExpression {
                        key,
                        expression,
                        value: expression_values[index][channel][usize::from(key)],
                    },
                });
            }
        }
    }
}

// ── Disk-streaming clip sources (chunk mailbox) ─────────────────────────────
//
// Long media plays through [`RenderSource::Stream`]: a control-side feeder
// decodes windows of the file into [`StreamChunk`]s and posts them through a
// bounded lock-free mailbox; the executor drains them into a small fixed
// array of held chunks and renders from those with the SAME interpolation
// paths as in-memory `Samples`. Chunks the executor no longer needs travel
// BACK through a retired mailbox and are deallocated control-side — exactly
// the plan-swap ownership discipline, applied per chunk. The executor never
// blocks, never allocates, and never frees: a missing source frame renders
// silence and increments an underrun counter.

/// Capacity of the feeder → executor chunk mailbox per stream handle.
const STREAM_CHUNK_MAILBOX_CAPACITY: usize = 8;
/// Chunks the executor holds per streaming clip while rendering.
const STREAM_HELD_CHUNK_SLOTS: usize = 4;
/// Retired-chunk mailbox capacity: sized so every in-flight chunk (mailbox
/// plus held slots) can retire without saturating while the feeder drains.
const STREAM_RETIRED_MAILBOX_CAPACITY: usize =
    STREAM_CHUNK_MAILBOX_CAPACITY + STREAM_HELD_CHUNK_SLOTS + 2;
/// Held or mailbox chunks starting further than this past the frame the
/// executor currently needs are treated as stale (left over from before a
/// backward seek) and retired. Feeders must keep their read-ahead window
/// comfortably inside this bound or their prefetch gets churned.
pub const STREAM_RETIRE_LOOKAHEAD_FRAMES: u64 = 262_144;

/// One window of decoded media: interleaved stereo f32 at the stream
/// handle's source rate, anchored at `start_frame` on the source clock.
#[derive(Clone, Debug)]
pub struct StreamChunk {
    /// First source frame the chunk covers.
    pub start_frame: u64,
    /// Interleaved stereo frames (length is even; frame count = len / 2).
    pub frames: Arc<[f32]>,
}

impl StreamChunk {
    /// Number of stereo frames in the chunk.
    pub fn frame_count(&self) -> u64 {
        self.frames.len() as u64 / 2
    }

    /// One past the last source frame the chunk covers.
    fn end_frame(&self) -> u64 {
        self.start_frame + self.frame_count()
    }
}

/// Bounded lock-free MPMC queue (Vyukov sequence-stamped ring). Used as the
/// chunk and retired mailboxes inside a stream handle: `try_push`/`try_pop`
/// neither allocate nor free nor block, so both ends are audio-thread safe.
/// (mpsc channels cannot serve here — the handle lives inside clonable plan
/// specs, and `Receiver` is neither clonable nor `Sync`.)
struct ChunkQueueSlot<T> {
    sequence: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

struct ChunkQueue<T> {
    slots: Box<[ChunkQueueSlot<T>]>,
    enqueue_position: AtomicUsize,
    dequeue_position: AtomicUsize,
}

// Safety: access to each slot's value is serialized by its sequence stamp
// (acquire/release): a slot is written only after winning the enqueue CAS
// and read only after winning the dequeue CAS.
unsafe impl<T: Send> Send for ChunkQueue<T> {}
unsafe impl<T: Send> Sync for ChunkQueue<T> {}

impl<T> ChunkQueue<T> {
    fn new(capacity: usize) -> Self {
        let slots = (0..capacity)
            .map(|index| ChunkQueueSlot {
                sequence: AtomicUsize::new(index),
                value: UnsafeCell::new(MaybeUninit::uninit()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        ChunkQueue {
            slots,
            enqueue_position: AtomicUsize::new(0),
            dequeue_position: AtomicUsize::new(0),
        }
    }

    /// Push without blocking or allocating; returns the value when full.
    fn try_push(&self, value: T) -> Result<(), T> {
        let capacity = self.slots.len();
        let mut position = self.enqueue_position.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[position % capacity];
            let sequence = slot.sequence.load(Ordering::Acquire);
            if sequence == position {
                match self.enqueue_position.compare_exchange_weak(
                    position,
                    position + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        unsafe { (*slot.value.get()).write(value) };
                        slot.sequence.store(position + 1, Ordering::Release);
                        return Ok(());
                    }
                    Err(current) => position = current,
                }
            } else if sequence < position {
                return Err(value); // Full.
            } else {
                position = self.enqueue_position.load(Ordering::Relaxed);
            }
        }
    }

    /// Pop without blocking or allocating; `None` when empty.
    fn try_pop(&self) -> Option<T> {
        let capacity = self.slots.len();
        let mut position = self.dequeue_position.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[position % capacity];
            let sequence = slot.sequence.load(Ordering::Acquire);
            if sequence == position + 1 {
                match self.dequeue_position.compare_exchange_weak(
                    position,
                    position + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let value = unsafe { (*slot.value.get()).assume_init_read() };
                        slot.sequence.store(position + capacity, Ordering::Release);
                        return Some(value);
                    }
                    Err(current) => position = current,
                }
            } else if sequence <= position {
                return None; // Empty.
            } else {
                position = self.dequeue_position.load(Ordering::Relaxed);
            }
        }
    }
}

impl<T> Drop for ChunkQueue<T> {
    fn drop(&mut self) {
        while self.try_pop().is_some() {}
    }
}

/// State shared between a stream handle (executor side) and its feeder.
struct StreamInner {
    source_sample_rate_hz: u32,
    total_frames: u64,
    /// Next source frame the executor needs (its read hint), published per
    /// rendered block. Seeks read as jumps here.
    wanted_frame: AtomicU64,
    /// Output frames rendered as silence because the needed source frame was
    /// not held (feeder behind, or seek not yet served).
    underrun_frames: AtomicU64,
    /// Feeder → executor chunk mailbox.
    chunks: ChunkQueue<StreamChunk>,
    /// Executor → feeder retired-chunk mailbox: chunks the executor no
    /// longer needs, returned for control-side deallocation.
    retired: ChunkQueue<StreamChunk>,
}

/// Executor-side handle to a disk-streamed media source. Arc-shared and
/// pointer-equal (like [`RenderSampleBuffer`]): create one per streaming
/// asset and reuse it across plan recompiles so specs stay idempotent.
/// Clips reference the handle plus their own window/anchor, exactly as
/// in-memory sample clips reference a shared buffer.
#[derive(Clone)]
pub struct RenderStreamHandle {
    inner: Arc<StreamInner>,
}

impl std::fmt::Debug for RenderStreamHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderStreamHandle")
            .field("source_sample_rate_hz", &self.inner.source_sample_rate_hz)
            .field("total_frames", &self.inner.total_frames)
            .finish_non_exhaustive()
    }
}

impl PartialEq for RenderStreamHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl RenderStreamHandle {
    /// Source sample rate of the streamed media.
    pub fn source_sample_rate_hz(&self) -> u32 {
        self.inner.source_sample_rate_hz
    }

    /// Total source frames in the streamed media.
    pub fn total_frames(&self) -> u64 {
        self.inner.total_frames
    }

    /// Output frames rendered as silence because the needed source frame
    /// was not available (cumulative).
    pub fn underrun_frames(&self) -> u64 {
        self.inner.underrun_frames.load(Ordering::Relaxed)
    }
}

/// Control-side feeder for one stream handle: reads the executor's wanted
/// frame, posts decoded chunks, and reclaims retired chunks for
/// deallocation. Single producer by design — do not share one feeder across
/// threads (the queue tolerates it, but interleaved feeding is meaningless).
pub struct StreamFeeder {
    inner: Arc<StreamInner>,
}

impl StreamFeeder {
    /// Next source frame the executor needs, as last published. Feed chunks
    /// covering `[wanted, wanted + read-ahead)`.
    pub fn wanted_frame(&self) -> u64 {
        self.inner.wanted_frame.load(Ordering::Relaxed)
    }

    /// Cumulative output frames the executor rendered as silence for want
    /// of this stream's data.
    pub fn underrun_frames(&self) -> u64 {
        self.inner.underrun_frames.load(Ordering::Relaxed)
    }

    /// Post a chunk to the executor; returns the chunk when the mailbox is
    /// full (try again after the executor drains).
    pub fn try_send_chunk(&self, chunk: StreamChunk) -> Result<(), StreamChunk> {
        self.inner.chunks.try_push(chunk)
    }

    /// Reclaim chunks the executor has retired; dropping the returned `Vec`
    /// deallocates them here, on the control side.
    pub fn collect_retired(&self) -> Vec<StreamChunk> {
        let mut retired = Vec::new();
        while let Some(chunk) = self.inner.retired.try_pop() {
            retired.push(chunk);
        }
        retired
    }
}

/// Create a connected feeder/handle pair for one streaming asset:
/// interleaved stereo media at `source_sample_rate_hz`, `total_frames`
/// frames long. The handle goes into [`RenderSource::Stream`] specs; the
/// feeder stays control-side and must be pumped (post chunks toward
/// [`StreamFeeder::wanted_frame`], collect retired) for audio to flow.
pub fn render_stream(
    source_sample_rate_hz: u32,
    total_frames: u64,
) -> (StreamFeeder, RenderStreamHandle) {
    let inner = Arc::new(StreamInner {
        source_sample_rate_hz,
        total_frames,
        wanted_frame: AtomicU64::new(0),
        underrun_frames: AtomicU64::new(0),
        chunks: ChunkQueue::new(STREAM_CHUNK_MAILBOX_CAPACITY),
        retired: ChunkQueue::new(STREAM_RETIRED_MAILBOX_CAPACITY),
    });
    (
        StreamFeeder {
            inner: Arc::clone(&inner),
        },
        RenderStreamHandle { inner },
    )
}

// ── Live input monitor sources (SPSC ring from the input callback) ─────────
//
// Monitoring plays "now", not "then": a live-input clip ignores the timeline
// position entirely and drains whatever the input callback has pushed into
// its ring each block. The ring is interleaved STEREO f32 at the STREAM rate
// — the host feeds already-negotiated-rate audio (no resampling in v1; a
// rate-mismatched input monitors off-pitch until the host reopens one side)
// and duplicates mono to stereo on the feeder side (signal-hardware's
// monitor tee does this). Underrun (input behind) renders silence and
// counts; the executor never blocks and never allocates.

/// Default live-input ring capacity in frames (~170 ms at 48 kHz). This is
/// the MAXIMUM backlog, not the operating latency: steady-state fill is
/// about one callback quantum, and the executor trims any deeper backlog to
/// [`LIVE_INPUT_MAX_BACKLOG_FRAMES`] so monitoring latency stays bounded.
pub const LIVE_INPUT_DEFAULT_CAPACITY_FRAMES: usize = 8_192;

/// Deepest ring backlog the executor tolerates before discarding old input
/// (~21 ms at 48 kHz). A feeder that pushed while the transport was stopped
/// (the executor only renders while playing) would otherwise replay stale
/// audio as extra monitoring latency on the next play.
const LIVE_INPUT_MAX_BACKLOG_FRAMES: usize = 1_024;

/// Stack scratch frames for live-input drain/discard loops (alloc-free).
const LIVE_INPUT_CHUNK_FRAMES: usize = 256;

struct LiveInputInner {
    /// Interleaved stereo samples at the stream rate.
    ring: SpscRing,
    /// Output frames rendered as silence because the ring ran dry.
    underrun_frames: AtomicU64,
}

/// Executor-side handle to a live input feed. Arc-shared and pointer-equal
/// (like [`RenderStreamHandle`]): create one per monitored input and reuse
/// it across plan recompiles so specs stay idempotent. The ring lives inside
/// the shared handle, so plan swaps inherently keep the audio flowing —
/// there is no per-plan state to migrate.
#[derive(Clone)]
pub struct RenderLiveInputHandle {
    inner: Arc<LiveInputInner>,
}

impl std::fmt::Debug for RenderLiveInputHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderLiveInputHandle")
            .field("capacity_frames", &(self.inner.ring.capacity() / 2))
            .finish_non_exhaustive()
    }
}

impl PartialEq for RenderLiveInputHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl RenderLiveInputHandle {
    /// Frames currently buffered (monitoring latency = this fill level).
    pub fn buffered_frames(&self) -> usize {
        self.inner.ring.len() / 2
    }

    /// Output frames rendered as silence because the input was behind
    /// (cumulative).
    pub fn underrun_frames(&self) -> u64 {
        self.inner.underrun_frames.load(Ordering::Relaxed)
    }
}

/// Producer side of a live input feed: the input callback pushes interleaved
/// STEREO frames at the stream rate. `push_slice` is alloc-free, lock-free,
/// and never blocks (a full ring drops the excess — the ring's overrun
/// contract), so it is safe on the OS audio thread.
///
/// SPSC discipline: at most one thread may push at a time. Sequential
/// hand-off between producers (monitor-only session → capture tee at record
/// start) is safe; concurrent pushers are not.
pub struct LiveInputFeeder {
    inner: Arc<LiveInputInner>,
}

impl LiveInputFeeder {
    /// Push interleaved stereo samples (`frames × 2` values). Returns the
    /// number of FRAMES written; the rest were dropped against a full ring.
    pub fn push_slice(&self, stereo_samples: &[f32]) -> usize {
        self.inner.ring.push_slice(stereo_samples) / 2
    }

    /// Total samples dropped against a full ring (see [`SpscRing`]).
    pub fn overrun_samples(&self) -> u64 {
        self.inner.ring.overrun_samples()
    }
}

/// Create a connected feeder/handle pair for one live input. The handle goes
/// into [`RenderSource::LiveInput`] specs; the feeder goes to the input
/// callback. `capacity_frames` bounds the ring (use
/// [`LIVE_INPUT_DEFAULT_CAPACITY_FRAMES`]); keep it shallow — fill level is
/// monitoring latency.
pub fn render_live_input(capacity_frames: usize) -> (LiveInputFeeder, RenderLiveInputHandle) {
    let inner = Arc::new(LiveInputInner {
        ring: SpscRing::with_capacity(capacity_frames.max(2) * 2),
        underrun_frames: AtomicU64::new(0),
    });
    (
        LiveInputFeeder {
            inner: Arc::clone(&inner),
        },
        RenderLiveInputHandle { inner },
    )
}

const COMMAND_MAILBOX_CAPACITY: usize = 64;
// Sized so that even a full command mailbox of plan installs can retire
// without saturating; install_plan also reclaims eagerly. The executor's
// single parked slot is belt-and-braces on top of this invariant.
const RETIRED_MAILBOX_CAPACITY: usize = COMMAND_MAILBOX_CAPACITY + 2;

/// Transport edge ramp length: play, stop, and seek gate through this
/// envelope instead of stepping, so transport actions never click.
const EDGE_RAMP_SECONDS: f32 = 0.005;
/// Full-swing time for stage gain changes across plan swaps.
const GAIN_SMOOTHING_SECONDS: f32 = 0.010;
/// Declick fade applied inside each clip window edge (shortened for tiny
/// windows so short clips stay audible).
const CLIP_EDGE_FADE_FRAMES: u64 = 32;
/// Micro-fade applied to the output buffer around a loop-region wrap point:
/// a linear fade out over the frames before the wrap and in over the frames
/// after it. Tones carry phase across the wrap and stay continuous; this
/// guards sample/stream clips that wrap mid-waveform. Only active on blocks
/// that actually wrap, so non-looping output is bit-identical (the golden
/// render hash stays valid).
const LOOP_WRAP_FADE_FRAMES: usize = 64;

// ── Plugin processors (g11.012) ────────────────────────────────────────────

/// Placement-agnostic per-block plugin processing backend.
///
/// The engine sees only this trait: a backend may run the plugin in-process
/// (direct FFI call — no wait, no crash isolation) or across a sandbox
/// process boundary (shared-memory round trip with a bounded wait). The
/// isolation tier is host configuration, never engine architecture.
///
/// # Contract
///
/// `process` transforms `scratch` (interleaved, `frame_count × channels`
/// samples) IN PLACE and returns `true`. When it returns `false` — deadline
/// miss, dead backend, unsupported channel count — `scratch` must be left
/// EXACTLY as it was (bypass semantics: consumers read the dry signal).
/// Implementations must be audio-thread safe: no allocation, no locks that
/// block, no unbounded waits.
pub trait PluginBlockProcessor: Send + Sync {
    /// Process one block in place; `false` = bypass, scratch untouched.
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool;

    /// Native event families this backend can deliver.
    fn event_support(&self) -> RenderPluginEventSupport {
        RenderPluginEventSupport::default()
    }

    /// Cumulative events rejected because this backend cannot represent or
    /// route them. The counter is monotonic and safe to poll off-thread.
    fn unsupported_event_count(&self) -> u64 {
        0
    }

    /// Processing latency reported by the live plugin, in sample frames.
    fn latency_frames(&self) -> u32 {
        0
    }

    /// Monotonic revision incremented when the backend observes that its
    /// reported processing latency may have changed. Hosts poll this off the
    /// audio thread to decide whether Pulse must rebuild graph compensation.
    fn latency_revision(&self) -> u64 {
        0
    }

    /// Process one block in place, delivering `events` (sorted by
    /// `offset_frames`, all offsets `< frame_count`) alongside the audio.
    /// Backends convert to their plugin format's native event lists here —
    /// this is the MIDI 1.0 downconversion boundary. The default drops the
    /// events and processes audio only (backends without event transport —
    /// the shared-memory tier today — stay bypass-correct for audio).
    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        let _ = events;
        self.process(scratch, frame_count, channels)
    }
}

/// Arc handle to a plugin processing backend, carried by
/// [`RenderStageSpec::processor`] on Sum stages.
///
/// Pointer-equal (like [`RenderStreamHandle`] / [`RenderLiveInputHandle`]):
/// hosts create one per live plugin instance and reuse it across plan
/// recompiles so specs stay idempotent — swapping the handle is a structural
/// plan change; keeping it is not.
#[derive(Clone)]
pub struct RenderPluginProcessor {
    inner: Arc<dyn PluginBlockProcessor>,
}

impl RenderPluginProcessor {
    /// Wrap a processing backend.
    pub fn new(backend: Arc<dyn PluginBlockProcessor>) -> Self {
        Self { inner: backend }
    }

    /// Process one block in place; `false` = bypass (scratch untouched).
    #[inline]
    pub fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.inner.process(scratch, frame_count, channels)
    }

    /// Process one block in place with a per-block event slice; `false` =
    /// bypass (scratch untouched). Backends without event transport ignore
    /// the events (trait default).
    #[inline]
    pub fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.inner
            .process_with_events(scratch, frame_count, channels, events)
    }

    /// Native event families supported by this live backend.
    pub fn event_support(&self) -> RenderPluginEventSupport {
        self.inner.event_support()
    }

    /// Cumulative unsupported-event attempts observed by the backend.
    pub fn unsupported_event_count(&self) -> u64 {
        self.inner.unsupported_event_count()
    }

    /// Processing latency reported by the live plugin, in sample frames.
    pub fn latency_frames(&self) -> u32 {
        self.inner.latency_frames()
    }

    /// Monotonic backend latency revision for control-side plan invalidation.
    pub fn latency_revision(&self) -> u64 {
        self.inner.latency_revision()
    }
}

impl std::fmt::Debug for RenderPluginProcessor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderPluginProcessor")
            .finish_non_exhaustive()
    }
}

impl PartialEq for RenderPluginProcessor {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

// ── Spec vocabulary (control-side description of a plan) ───────────────────

/// Channel layout semantics carried alongside a channel count.
///
/// Extensible past stereo from day one; `Generic` means count-only semantics
/// (no named speaker positions). Named multichannel layouts (5.1, 7.1.4, …)
/// are added here as the engine grows layout-aware coefficient generators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    /// Single channel.
    Mono,
    /// Two channels, left/right semantics.
    Stereo,
    /// Count-only semantics: channels have no named positions.
    Generic,
}

/// Channel format of a stage's output: count plus layout semantics. Scratch
/// sizing, matrix validation, and boundary adaptation all key off this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelFormat {
    /// Number of interleaved channels.
    pub channels: u16,
    /// Layout semantics of those channels.
    pub layout: ChannelLayout,
}

impl ChannelFormat {
    /// Single mono channel.
    pub fn mono() -> Self {
        ChannelFormat {
            channels: 1,
            layout: ChannelLayout::Mono,
        }
    }

    /// Two channels, left/right.
    pub fn stereo() -> Self {
        ChannelFormat {
            channels: 2,
            layout: ChannelLayout::Stereo,
        }
    }
}

/// Audio source for one render clip.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderSource {
    /// Clip renders silence.
    Silence,
    /// Clip renders a sine tone — the audible stand-in for media-less clips
    /// and the soak-test source.
    TestTone {
        /// Tone frequency in hertz.
        frequency_hz: f32,
    },
    /// Clip plays shared sample data from its window start, with windowed-sinc
    /// interpolation when the source rate differs from the stream rate.
    Samples(RenderSampleBuffer),
    /// Clip streams media from disk through a chunk mailbox (see
    /// [`render_stream`]); same anchoring and interpolation as `Samples`,
    /// but source data arrives in feeder-posted chunks and missing frames
    /// render silence (counted on the handle). `loop_source` is ignored.
    Stream(RenderStreamHandle),
    /// Clip monitors a live input (see [`render_live_input`]): each block
    /// drains whatever the input callback pushed into the handle's ring,
    /// ignoring the timeline position entirely — live input renders "now".
    /// Window a live clip `[0, u64::MAX)` so it is always audible while the
    /// transport plays. Underrun (input behind) renders silence and counts
    /// on the handle; `loop_source` is ignored.
    LiveInput(RenderLiveInputHandle),
    /// Clip synthesizes its note events through the built-in instrument: a
    /// STATELESS additive sine voice per note (phase, attack/sustain/release
    /// envelope all pure functions of the stream position), so seeks and
    /// plan swaps are inherently sample-exact — there is no voice state to
    /// inherit. Notes are clip-relative and must be sorted by `start_frame`
    /// (compile rejects unsorted buffers). At most
    /// [`NOTE_POLYPHONY_LIMIT`] notes render simultaneously per block
    /// (earliest-started win). `loop_source` is ignored.
    Notes(RenderNoteBuffer),
    /// Rate-warped playback of a media source (g12.027 repitch/varispeed):
    /// the inner source's samples are consumed at `rate` times normal speed,
    /// so pitch shifts with the rate — like vinyl. `rate` multiplies the
    /// source-frames-per-stream-frame step the inner source would compile to
    /// (its own sample-rate conversion included), reusing the polyphase
    /// windowed-sinc interpolation path; non-finite or non-positive rates
    /// compile as 1.0. Valid over [`RenderSource::Samples`] and
    /// [`RenderSource::Stream`] only — anything else (including nesting)
    /// rejects at compile with
    /// [`RenderPlanCompileError::WarpedSourceUnsupported`].
    Warped {
        /// The media source being rate-warped.
        source: Box<RenderSource>,
        /// Playback-rate multiplier: 2.0 plays double speed (octave up),
        /// 0.5 half speed (octave down).
        rate: f64,
    },
}

/// One clip event in a lane stage: a half-open stream-clock window
/// `[start_frames, end_frames)` and the source that plays inside it.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderClipSpec {
    /// Stable clip identity (consumer-supplied). Used control-side to build
    /// the state-inheritance map across plan swaps, so a clip keeps its
    /// playback state (tone phase, later streaming cursors) when neighbours
    /// are inserted or removed.
    pub clip_id: u64,
    /// First audible frame.
    pub start_frames: u64,
    /// First frame past the audible range.
    pub end_frames: u64,
    /// Source playing inside the window, anchored at `start_frames`.
    pub source: RenderSource,
    /// When true, a `Samples` source repeats from its start once exhausted
    /// instead of going silent; ignored for other sources.
    pub loop_source: bool,
}

/// What a stage does with its inputs (and whether it generates content).
#[derive(Debug, Clone, PartialEq)]
pub enum RenderStageKind {
    /// Source stage: renders clip content at its format. May also sum inputs
    /// (clips render first, then edges add in).
    Source {
        /// Clip events rendered by this source stage.
        clips: Vec<RenderClipSpec>,
    },
    /// Sums its inputs through their edge matrices.
    Sum,
    /// Delays its summed inputs by an exact number of stream frames. The
    /// delay line is allocated when the plan compiles; rendering only swaps
    /// samples through the fixed-size ring. Consumers decide where to place
    /// compensation stages and how many frames they require.
    Delay {
        /// Number of stream frames to delay. Zero is an identity stage.
        frames: u32,
    },
    /// The output boundary: exactly one per plan. Its scratch is what the
    /// hardware boundary adapts onto the stream.
    Output,
}

/// One input edge into a stage: where the signal comes from, how loud, and
/// how its channels map onto this stage's channels.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderEdgeSpec {
    /// `stage_id` of the upstream stage feeding this edge.
    pub source_stage_id: u64,
    /// Linear edge gain (static in v1: compiled into the matrix).
    pub gain: f32,
    /// Row-major `source_channels × dest_channels` mix matrix; `None` picks
    /// the default adapter (identity when channel counts match; standard
    /// up/downmix coefficients otherwise — see
    /// `signal_dsp::default_adapter_matrix` for the exact rules).
    pub matrix: Option<Vec<f32>>,
}

/// One stage in a render plan graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderStageSpec {
    /// Stable consumer-supplied identity; matches smoothed-gain and tone
    /// state across plan swaps. Never read in the render loop.
    pub stage_id: u64,
    /// The stage's output format.
    pub format: ChannelFormat,
    /// Linear stage output gain, smoothed (10 ms full swing) and applied
    /// where the stage's output is consumed. When `gain_automation` is
    /// present it replaces this value during playback.
    pub gain: f32,
    /// Optional compiled automation: sorted `(frame, linear gain)`
    /// breakpoints on the stream clock, linearly interpolated and sampled
    /// per block into sample-accurate ramps. Stateless: the value is a pure
    /// function of the stream position, so plan swaps stay continuous (the
    /// block ramp always starts from the inherited smoothed gain).
    pub gain_automation: Option<Vec<(u64, f32)>>,
    /// What the stage does.
    pub kind: RenderStageKind,
    /// Edges feeding this stage.
    pub inputs: Vec<RenderEdgeSpec>,
    /// Optional plugin processor applied to this stage's summed scratch
    /// before consumers read it (g11.012). Valid on [`RenderStageKind::Sum`]
    /// stages only — compile rejects it elsewhere. Bypass (backend returns
    /// `false`) leaves the scratch untouched, so absent/bypassed processors
    /// keep the golden render hash unchanged.
    pub processor: Option<RenderPluginProcessor>,
    /// Optional compiled plugin event stream (notes + CC on the absolute
    /// stream clock, sorted by frame) delivered to `processor` per block
    /// with intra-block sample offsets. Valid only alongside a processor —
    /// compile rejects events without one. `None` keeps the pre-event
    /// behavior bit-identical.
    pub events: Option<RenderPluginEventBuffer>,
}

/// Optional soft limiter guarding the hardware boundary. Mechanism only —
/// thresholds and whether to limit at all are consumer policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderLimiterSpec {
    /// Linear level where limiting centers (clamped below 0 dBFS).
    pub threshold: f32,
    /// Linear soft-knee width centered on the threshold.
    pub knee_width: f32,
    /// One-pole gain-recovery time constant in seconds.
    pub release_seconds: f32,
}

/// Control-side description of a render plan: a format-typed stage graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlanSpec {
    /// Sample rate the plan renders at.
    pub sample_rate_hz: u32,
    /// Linear master gain applied at the master stage (kept for consumer
    /// compatibility; multiplies the master stage's own gain).
    pub master_gain: f32,
    /// Optional soft limiter applied to the stream buffer after the
    /// boundary write and before the transport edge envelope. `None` = no
    /// limiting (bit-transparent master path).
    pub master_limiter: Option<RenderLimiterSpec>,
    /// The stage graph; any order, exactly one [`RenderStageKind::Output`].
    pub stages: Vec<RenderStageSpec>,
}

impl RenderPlanSpec {
    /// When `other` differs from `self` ONLY in stage gains (same stages,
    /// formats, automation, clips, edges), return the changed
    /// `(stage_id, new_gain)` pairs — hosts use this to take the parameter
    /// fast path (`set_stage_gain`) instead of recompiling the plan.
    /// Returns `None` when anything structural differs (install required).
    pub fn differs_only_in_gains(&self, other: &RenderPlanSpec) -> Option<Vec<(u64, f32)>> {
        if self.sample_rate_hz != other.sample_rate_hz
            || self.master_gain != other.master_gain
            || self.master_limiter != other.master_limiter
            || self.stages.len() != other.stages.len()
        {
            return None;
        }
        let mut changes = Vec::new();
        for (old, new) in self.stages.iter().zip(other.stages.iter()) {
            if old.stage_id != new.stage_id
                || old.format != new.format
                || old.gain_automation != new.gain_automation
                || old.kind != new.kind
                || old.inputs != new.inputs
                || old.processor != new.processor
                || old.events != new.events
            {
                return None;
            }
            if old.gain != new.gain {
                changes.push((new.stage_id, new.gain));
            }
        }
        Some(changes)
    }

    /// Channel count of the master stage (the format the plan mixes at), or
    /// 2 when the spec has no master (such a spec fails compile anyway).
    pub fn output_channels(&self) -> u16 {
        self.stages
            .iter()
            .find(|stage| matches!(stage.kind, RenderStageKind::Output))
            .map(|stage| stage.format.channels)
            .unwrap_or(2)
    }
}

/// Typed error rejecting a plan spec at compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderPlanCompileError {
    /// Two stages share a `stage_id`.
    DuplicateNodeId(u64),
    /// An edge references a `source_stage_id` that does not exist.
    UnknownInputNode {
        /// Stage owning the edge.
        stage_id: u64,
        /// Missing upstream id.
        source_stage_id: u64,
    },
    /// The graph must contain exactly one output stage.
    MasterCount(usize),
    /// The graph contains a cycle and cannot be scheduled.
    Cycle,
    /// A stage declared a zero-channel format.
    InvalidChannelCount(u64),
    /// A gain-automation envelope's breakpoints are not sorted by frame.
    UnsortedAutomation(u64),
    /// A clip's note buffer is not sorted by `start_frame`.
    UnsortedNotes {
        /// Stage owning the clip.
        stage_id: u64,
        /// Clip whose notes are unsorted.
        clip_id: u64,
    },
    /// A plugin processor was attached to a non-Sum stage.
    ProcessorOnNonSumStage(u64),
    /// A plugin event stream was attached to a stage without a processor.
    EventsWithoutProcessor(u64),
    /// A plugin event buffer is not sorted by `frame`.
    UnsortedEvents(u64),
    /// A `Warped` source wraps something other than `Samples` or `Stream`.
    WarpedSourceUnsupported {
        /// Stage owning the clip.
        stage_id: u64,
        /// Clip carrying the unsupported warp.
        clip_id: u64,
    },
    /// An explicit edge matrix has the wrong number of coefficients.
    MatrixDimensions {
        /// Stage owning the edge.
        stage_id: u64,
        /// Upstream stage feeding the edge.
        source_stage_id: u64,
        /// `source_channels × dest_channels`.
        expected: usize,
        /// Length supplied.
        actual: usize,
    },
}

impl std::fmt::Display for RenderPlanCompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderPlanCompileError::DuplicateNodeId(stage_id) => {
                write!(formatter, "duplicate stage id {stage_id}")
            }
            RenderPlanCompileError::UnknownInputNode {
                stage_id,
                source_stage_id,
            } => write!(
                formatter,
                "stage {stage_id} references unknown input stage {source_stage_id}",
            ),
            RenderPlanCompileError::MasterCount(count) => write!(
                formatter,
                "plan must contain exactly one output stage (found {count})",
            ),
            RenderPlanCompileError::Cycle => {
                write!(formatter, "plan graph contains a cycle")
            }
            RenderPlanCompileError::InvalidChannelCount(stage_id) => {
                write!(formatter, "stage {stage_id} declares zero channels")
            }
            RenderPlanCompileError::UnsortedAutomation(stage_id) => write!(
                formatter,
                "stage {stage_id} gain automation breakpoints are not sorted by frame",
            ),
            RenderPlanCompileError::ProcessorOnNonSumStage(stage_id) => write!(
                formatter,
                "stage {stage_id} attaches a plugin processor but is not a Sum stage",
            ),
            RenderPlanCompileError::EventsWithoutProcessor(stage_id) => write!(
                formatter,
                "stage {stage_id} attaches plugin events without a plugin processor",
            ),
            RenderPlanCompileError::UnsortedEvents(stage_id) => write!(
                formatter,
                "stage {stage_id} plugin events are not sorted by frame",
            ),
            RenderPlanCompileError::WarpedSourceUnsupported { stage_id, clip_id } => write!(
                formatter,
                "stage {stage_id} clip {clip_id} warps a source that is not Samples or Stream",
            ),
            RenderPlanCompileError::UnsortedNotes { stage_id, clip_id } => write!(
                formatter,
                "stage {stage_id} clip {clip_id} notes are not sorted by start frame",
            ),
            RenderPlanCompileError::MatrixDimensions {
                stage_id,
                source_stage_id,
                expected,
                actual,
            } => write!(
                formatter,
                "edge {source_stage_id} -> {stage_id} matrix has {actual} coefficients, expected {expected}",
            ),
        }
    }
}

impl std::error::Error for RenderPlanCompileError {}

// ── Compiled plan (render-side data, preallocated at compile time) ─────────

enum CompiledSource {
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

struct CompiledClip {
    start_frames: u64,
    end_frames: u64,
    /// Declick fade length at each window edge, shortened for tiny windows.
    edge_fade_frames: u64,
    /// Stable identity, read control-side when building inheritance maps.
    clip_id: u64,
    source: CompiledSource,
}

/// One compiled input edge. `source_index` is a position in the plan's
/// topologically-ordered stage list and is always strictly less than the
/// consuming stage's position, so the executor can split-borrow.
struct CompiledInput {
    source_index: usize,
    source_channels: usize,
    /// Row-major `source_channels × dest_channels`; edge gain folded in.
    matrix: Vec<f32>,
}

struct CompiledNode {
    /// Matches stage state (smoothed gain, tone phase) across plan swaps.
    stage_id: u64,
    channels: usize,
    /// Gain the stage is moving toward (spec value).
    gain_target: f32,
    /// Smoothed gain as currently applied; inherited across plan swaps so
    /// gain edits never step.
    gain_current: f32,
    /// Per-block smoothed-gain interpolation, written when the stage renders
    /// and read wherever its output is consumed (edges, boundary).
    block_gain_begin: f32,
    block_gain_slope: f32,
    /// Sorted automation breakpoints `(frame, gain)`; empty = no automation
    /// (static `gain_target` smoothing applies).
    gain_envelope: Vec<(u64, f32)>,
    /// Clip content (Source stages; empty for Sum/Output).
    clips: Vec<CompiledClip>,
    inputs: Vec<CompiledInput>,
    /// Plugin processor applied to the summed scratch (Sum stages only).
    processor: Option<RenderPluginProcessor>,
    /// Compiled plugin event stream (absolute frames, sorted); empty when
    /// the stage carries none. Shared with the spec's Arc — no copy.
    events: Arc<[RenderPluginEvent]>,
    /// Preallocated per-block event slice handed to the processor
    /// ([`PLUGIN_EVENTS_PER_BLOCK_CAPACITY`]); the audio thread only ever
    /// clears and pushes within capacity.
    event_scratch: Vec<RenderBlockPluginEvent>,
    /// Interleaved fixed-size delay ring plus its next sample position.
    /// Empty for non-delay stages and zero-frame delays.
    delay_ring: Vec<f32>,
    delay_cursor: usize,
    /// Interleaved scratch at the stage's format: `MAX_BLOCK_FRAMES × channels`.
    scratch: Vec<f32>,
}

/// A compiled, immutable-topology render plan. Source state (tone phases,
/// smoothed gains) mutates during rendering; structure never does. Nodes are
/// stored in topological order (inputs strictly before consumers).
pub struct RenderPlan {
    sample_rate_hz: u32,
    stages: Vec<CompiledNode>,
    /// Position of the master stage in `stages`.
    master_index: usize,
    /// Stream channel count the boundary was compiled for (master channels
    /// when the stream is unknown at install time).
    stream_channels: usize,
    /// Hardware-boundary downmix (row-major `output_channels ×
    /// stream_channels`); empty unless the stream is narrower than the
    /// master format. Compiled at install time — the controller knows the
    /// stream's channel count then. Never applies inside the creative graph.
    boundary_matrix: Vec<f32>,
    /// Per-stage inheritance: `inherit_stage_map[i]` is the index of the
    /// matching stage in the PREVIOUS plan (by stage_id), precomputed by the
    /// controller at install time so the executor never compares identities
    /// on the audio thread.
    inherit_stage_map: Vec<Option<usize>>,
    /// Per-stage, per-clip inheritance into the previous plan's clip list
    /// (by clip_id within the matched stage).
    inherit_clip_maps: Vec<Vec<Option<usize>>>,
    /// Monotonic install stamp assigned by the controller. The executor
    /// publishes it with the meter table so readers can map meter slots to
    /// the stage ids of the matching topology.
    generation: u64,
    /// Optional master soft limiter guarding the hardware boundary; its
    /// smoothed gain inherits across plan swaps.
    limiter: Option<LimiterState>,
}

impl RenderPlan {
    fn compile(
        spec: &RenderPlanSpec,
        stream_channels: Option<u16>,
    ) -> Result<Box<RenderPlan>, RenderPlanCompileError> {
        // Identity and shape validation.
        for stage in &spec.stages {
            if stage.format.channels == 0 {
                return Err(RenderPlanCompileError::InvalidChannelCount(stage.stage_id));
            }
            if spec
                .stages
                .iter()
                .filter(|candidate| candidate.stage_id == stage.stage_id)
                .count()
                > 1
            {
                return Err(RenderPlanCompileError::DuplicateNodeId(stage.stage_id));
            }
        }
        let master_count = spec
            .stages
            .iter()
            .filter(|stage| matches!(stage.kind, RenderStageKind::Output))
            .count();
        if master_count != 1 {
            return Err(RenderPlanCompileError::MasterCount(master_count));
        }
        for stage in &spec.stages {
            if stage.processor.is_some() && !matches!(stage.kind, RenderStageKind::Sum) {
                return Err(RenderPlanCompileError::ProcessorOnNonSumStage(
                    stage.stage_id,
                ));
            }
            if let Some(events) = &stage.events {
                if stage.processor.is_none() {
                    return Err(RenderPlanCompileError::EventsWithoutProcessor(
                        stage.stage_id,
                    ));
                }
                if events
                    .events
                    .windows(2)
                    .any(|pair| pair[0].frame > pair[1].frame)
                {
                    return Err(RenderPlanCompileError::UnsortedEvents(stage.stage_id));
                }
            }
        }
        let position_of = |stage_id: u64| -> Option<usize> {
            spec.stages
                .iter()
                .position(|candidate| candidate.stage_id == stage_id)
        };
        for stage in &spec.stages {
            for input in &stage.inputs {
                let Some(source_index) = position_of(input.source_stage_id) else {
                    return Err(RenderPlanCompileError::UnknownInputNode {
                        stage_id: stage.stage_id,
                        source_stage_id: input.source_stage_id,
                    });
                };
                let expected = spec.stages[source_index].format.channels as usize
                    * stage.format.channels as usize;
                if let Some(matrix) = &input.matrix {
                    if matrix.len() != expected {
                        return Err(RenderPlanCompileError::MatrixDimensions {
                            stage_id: stage.stage_id,
                            source_stage_id: input.source_stage_id,
                            expected,
                            actual: matrix.len(),
                        });
                    }
                }
            }
        }

        // Kahn topological sort (spec order preserved among ready stages).
        let node_count = spec.stages.len();
        let mut indegree: Vec<usize> = spec.stages.iter().map(|stage| stage.inputs.len()).collect();
        let mut consumers: Vec<Vec<usize>> = vec![Vec::new(); node_count];
        for (index, stage) in spec.stages.iter().enumerate() {
            for input in &stage.inputs {
                let source_index = position_of(input.source_stage_id).expect("validated above");
                consumers[source_index].push(index);
            }
        }
        let mut order: Vec<usize> = Vec::with_capacity(node_count);
        let mut ready: Vec<usize> = (0..node_count)
            .filter(|index| indegree[*index] == 0)
            .collect();
        let mut next_ready = 0usize;
        while next_ready < ready.len() {
            let index = ready[next_ready];
            next_ready += 1;
            order.push(index);
            for consumer in &consumers[index] {
                indegree[*consumer] -= 1;
                if indegree[*consumer] == 0 {
                    ready.push(*consumer);
                }
            }
        }
        if order.len() != node_count {
            return Err(RenderPlanCompileError::Cycle);
        }
        // Spec index → topological position, for edge re-indexing.
        let mut topo_position = vec![0usize; node_count];
        for (position, spec_index) in order.iter().enumerate() {
            topo_position[*spec_index] = position;
        }

        let tau = std::f32::consts::TAU;
        let stream_rate = spec.sample_rate_hz.max(1);
        // One table per distinct cutoff in the plan (typically zero or one).
        let mut tables: Vec<(u64, PolyphaseInterpolationTable)> = Vec::new();
        let mut table_for_step = |step: f64| -> Option<PolyphaseInterpolationTable> {
            if step == 1.0 {
                return None;
            }
            let cutoff = (1.0 / step).min(1.0);
            let key = cutoff.to_bits();
            if let Some((_, table)) = tables.iter().find(|(bits, _)| *bits == key) {
                return Some(table.clone());
            }
            let table = PolyphaseInterpolationTable::new(RESAMPLE_TAPS, RESAMPLE_PHASES, cutoff);
            tables.push((key, table.clone()));
            Some(table)
        };

        let mut stages: Vec<CompiledNode> = Vec::with_capacity(node_count);
        let mut master_index = 0usize;
        for (position, spec_index) in order.iter().enumerate() {
            let stage = &spec.stages[*spec_index];
            let dest_channels = stage.format.channels as usize;
            if matches!(stage.kind, RenderStageKind::Output) {
                master_index = position;
            }
            let clips = match &stage.kind {
                RenderStageKind::Source { clips } => clips
                    .iter()
                    .map(|clip| {
                        // Unwrap a rate warp (g12.027): the warp multiplies
                        // the source-frames-per-stream-frame step its inner
                        // media source compiles to. Warping anything but
                        // media (or nesting warps) is a spec bug — reject.
                        let (spec_source, warp_rate) = match &clip.source {
                            RenderSource::Warped { source, rate } => {
                                if !matches!(
                                    source.as_ref(),
                                    RenderSource::Samples(_) | RenderSource::Stream(_)
                                ) {
                                    return Err(RenderPlanCompileError::WarpedSourceUnsupported {
                                        stage_id: stage.stage_id,
                                        clip_id: clip.clip_id,
                                    });
                                }
                                let rate = if rate.is_finite() && *rate > 0.0 {
                                    *rate
                                } else {
                                    1.0
                                };
                                (source.as_ref(), rate)
                            }
                            source => (source, 1.0),
                        };
                        let source = match spec_source {
                            RenderSource::Warped { .. } => {
                                unreachable!("nested warps rejected above")
                            }
                            RenderSource::Silence => CompiledSource::Silence,
                            RenderSource::TestTone { frequency_hz } => CompiledSource::Tone {
                                phase: 0.0,
                                step: frequency_hz * tau / stream_rate as f32,
                            },
                            RenderSource::Samples(buffer) => {
                                let step = buffer.sample_rate_hz.max(1) as f64 / stream_rate as f64
                                    * warp_rate;
                                // Adapt the source's channels to the stage format
                                // only when they differ; matched counts keep the
                                // direct read (golden-hash stable).
                                let source_channels = buffer.channels.max(1) as usize;
                                let channel_adapter = if source_channels == dest_channels {
                                    None
                                } else {
                                    Some(signal_dsp::default_adapter_matrix(
                                        buffer.channels.max(1),
                                        dest_channels as u16,
                                    ))
                                };
                                CompiledSource::Samples {
                                    table: table_for_step(step),
                                    step,
                                    buffer: buffer.clone(),
                                    loop_source: clip.loop_source,
                                    channel_adapter,
                                }
                            }
                            RenderSource::Stream(handle) => {
                                let step = handle.source_sample_rate_hz().max(1) as f64
                                    / stream_rate as f64
                                    * warp_rate;
                                CompiledSource::Stream {
                                    table: table_for_step(step),
                                    step,
                                    handle: handle.clone(),
                                    held: std::array::from_fn(|_| None),
                                }
                            }
                            RenderSource::LiveInput(handle) => CompiledSource::LiveInput {
                                handle: handle.clone(),
                            },
                            RenderSource::Notes(buffer) => {
                                if buffer
                                    .notes
                                    .windows(2)
                                    .any(|pair| pair[0].start_frame > pair[1].start_frame)
                                {
                                    return Err(RenderPlanCompileError::UnsortedNotes {
                                        stage_id: stage.stage_id,
                                        clip_id: clip.clip_id,
                                    });
                                }
                                // Per-note phase steps precomputed here, on
                                // the control side — the render loop does no
                                // transcendentals beyond sin().
                                let steps: Arc<[f64]> = buffer
                                    .notes
                                    .iter()
                                    .map(|note| {
                                        note.frequency_hz() * std::f64::consts::TAU
                                            / stream_rate as f64
                                    })
                                    .collect();
                                CompiledSource::Notes {
                                    steps,
                                    attack_frames: ((NOTE_ATTACK_SECONDS * stream_rate as f64)
                                        as u64)
                                        .max(1),
                                    release_frames: ((NOTE_RELEASE_SECONDS * stream_rate as f64)
                                        as u64)
                                        .max(1),
                                    max_duration_frames: buffer
                                        .notes
                                        .iter()
                                        .map(|note| note.duration_frames)
                                        .max()
                                        .unwrap_or(0),
                                    buffer: buffer.clone(),
                                }
                            }
                        };
                        Ok(CompiledClip {
                            clip_id: clip.clip_id,
                            start_frames: clip.start_frames,
                            end_frames: clip.end_frames,
                            edge_fade_frames: CLIP_EDGE_FADE_FRAMES
                                .min(clip.end_frames.saturating_sub(clip.start_frames).max(2) / 2),
                            source,
                        })
                    })
                    .collect::<Result<Vec<_>, RenderPlanCompileError>>()?,
                RenderStageKind::Sum | RenderStageKind::Delay { .. } | RenderStageKind::Output => {
                    Vec::new()
                }
            };
            let inputs = stage
                .inputs
                .iter()
                .map(|input| {
                    let source_spec_index =
                        position_of(input.source_stage_id).expect("validated above");
                    let source_channels = spec.stages[source_spec_index].format.channels as usize;
                    // Explicit matrix or default adapter, with the static
                    // edge gain folded into the coefficients (v1: per-edge
                    // gain is compile-time data, not a smoothed parameter).
                    let mut matrix = match &input.matrix {
                        Some(matrix) => matrix.clone(),
                        None => {
                            default_adapter_matrix(source_channels as u16, dest_channels as u16)
                        }
                    };
                    for coefficient in matrix.iter_mut() {
                        *coefficient *= input.gain;
                    }
                    CompiledInput {
                        source_index: topo_position[source_spec_index],
                        source_channels,
                        matrix,
                    }
                })
                .collect();
            // The plan's master_gain multiplies the master stage's own gain
            // so the legacy single-knob master keeps working.
            let master_factor = if matches!(stage.kind, RenderStageKind::Output) {
                spec.master_gain
            } else {
                1.0
            };
            let gain = stage.gain * master_factor;
            let delay_ring = match stage.kind {
                RenderStageKind::Delay { frames } => {
                    vec![0.0; frames as usize * dest_channels]
                }
                _ => Vec::new(),
            };
            // Automation envelope: sorted, master factor folded in so the
            // render path samples one curve.
            let gain_envelope: Vec<(u64, f32)> = match &stage.gain_automation {
                Some(points) => {
                    if points.windows(2).any(|pair| pair[0].0 > pair[1].0) {
                        return Err(RenderPlanCompileError::UnsortedAutomation(stage.stage_id));
                    }
                    points
                        .iter()
                        .map(|(frame, value)| (*frame, *value * master_factor))
                        .collect()
                }
                None => Vec::new(),
            };
            stages.push(CompiledNode {
                stage_id: stage.stage_id,
                channels: dest_channels,
                gain_target: gain,
                gain_current: gain,
                block_gain_begin: gain,
                block_gain_slope: 0.0,
                gain_envelope,
                clips,
                inputs,
                processor: stage.processor.clone(),
                events: stage
                    .events
                    .as_ref()
                    .map(|buffer| Arc::clone(&buffer.events))
                    .unwrap_or_else(|| Arc::from([])),
                event_scratch: if stage.events.is_some() {
                    Vec::with_capacity(PLUGIN_EVENTS_PER_BLOCK_CAPACITY)
                } else {
                    Vec::new()
                },
                delay_ring,
                delay_cursor: 0,
                scratch: vec![0.0f32; MAX_BLOCK_FRAMES * dest_channels],
            });
        }

        // Hardware boundary: compiled here, at install time, because the
        // controller knows the stream's channel count now. A narrower stream
        // gets a standard downmix matrix; a wider stream gets the master's
        // channels copied and the extras silence-filled (no upmix policy is
        // invented at the hardware stage); equal formats copy through.
        let output_channels = stages[master_index].channels;
        let stream_channels = stream_channels
            .map(|channels| channels.max(1) as usize)
            .unwrap_or(output_channels);
        let boundary_matrix = if stream_channels < output_channels {
            default_adapter_matrix(output_channels as u16, stream_channels as u16)
        } else {
            Vec::new()
        };

        Ok(Box::new(RenderPlan {
            sample_rate_hz: stream_rate,
            stages,
            master_index,
            stream_channels,
            boundary_matrix,
            inherit_stage_map: Vec::new(),
            inherit_clip_maps: Vec::new(),
            generation: 0,
            limiter: spec.master_limiter.as_ref().map(|limiter| {
                LimiterState::new(
                    signal_primitives::SampleRate(stream_rate),
                    limiter.threshold,
                    limiter.knee_width,
                    signal_primitives::Seconds(limiter.release_seconds),
                )
            }),
        }))
    }

    /// Carry smoothed gains and tone phases over from the plan being
    /// replaced, so a recompile (gain tweak, clip edit) never steps audio.
    /// Matching is precomputed by the controller into `inherit_stage_map` /
    /// `inherit_clip_maps` at install time (by stage_id and clip_id), so this
    /// is O(stages + clips) index copies — no identity comparisons run on
    /// the audio thread, and inserting a clip mid-lane no longer cross-wires
    /// its neighbours' state.
    fn inherit_state(&mut self, previous: &mut RenderPlan) {
        // Limiter recovery gain carries over so a recompile mid-limiting
        // does not snap the gain back to unity.
        if let (Some(limiter), Some(previous_limiter)) =
            (self.limiter.as_mut(), previous.limiter.as_ref())
        {
            limiter.set_gain(previous_limiter.gain());
        }
        if self.inherit_stage_map.len() != self.stages.len() {
            // No map (first install or controller skipped): nothing carries.
            return;
        }
        for (index, stage) in self.stages.iter_mut().enumerate() {
            let Some(previous_index) = self.inherit_stage_map[index] else {
                continue;
            };
            let Some(previous_node) = previous.stages.get_mut(previous_index) else {
                continue;
            };
            stage.gain_current = previous_node.gain_current;
            if stage.delay_ring.len() == previous_node.delay_ring.len()
                && !stage.delay_ring.is_empty()
            {
                stage.delay_ring.copy_from_slice(&previous_node.delay_ring);
                stage.delay_cursor = previous_node.delay_cursor;
            }
            let clip_map = self
                .inherit_clip_maps
                .get(index)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for (clip_index, clip) in stage.clips.iter_mut().enumerate() {
                let Some(previous_clip_index) = clip_map.get(clip_index).copied().flatten() else {
                    continue;
                };
                let Some(previous_clip) = previous_node.clips.get_mut(previous_clip_index) else {
                    continue;
                };
                match (&mut clip.source, &mut previous_clip.source) {
                    (
                        CompiledSource::Tone { phase, step },
                        CompiledSource::Tone {
                            phase: previous_phase,
                            step: previous_step,
                        },
                    ) if *step == *previous_step => {
                        *phase = *previous_phase;
                    }
                    // Streaming clips MOVE their held read-ahead chunks into
                    // the new plan (same handle, same rate ratio), so an
                    // identity recompile mid-stream never underruns. Chunks
                    // that do not transfer ride the retired plan back to the
                    // control side and drop there — never on this thread.
                    (
                        CompiledSource::Stream {
                            handle, held, step, ..
                        },
                        CompiledSource::Stream {
                            handle: previous_handle,
                            held: previous_held,
                            step: previous_step,
                            ..
                        },
                    ) if handle == previous_handle && *step == *previous_step => {
                        for (slot, previous_slot) in held.iter_mut().zip(previous_held.iter_mut()) {
                            *slot = previous_slot.take();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Sample a sorted `(frame, value)` automation envelope at `frame`: linear
/// interpolation between breakpoints, clamped to the first/last values
/// outside the span. Binary search + arithmetic only — audio-thread safe.
#[inline]
fn sample_envelope(points: &[(u64, f32)], frame: u64) -> f32 {
    match points.binary_search_by(|(point_frame, _)| point_frame.cmp(&frame)) {
        Ok(index) => points[index].1,
        Err(0) => points[0].1,
        Err(index) if index == points.len() => points[points.len() - 1].1,
        Err(index) => {
            let (start_frame, start_value) = points[index - 1];
            let (end_frame, end_value) = points[index];
            let span = (end_frame - start_frame).max(1) as f64;
            let progress = (frame - start_frame) as f64 / span;
            start_value + (end_value - start_value) * progress as f32
        }
    }
}

/// Declick gain for a frame inside a clip window: linear fade over
/// `fade_frames` at each edge, unity in between.
#[inline]
fn clip_edge_gain(frame: u64, start_frames: u64, end_frames: u64, fade_frames: u64) -> f32 {
    if fade_frames == 0 {
        return 1.0;
    }
    let fade = fade_frames as f32;
    let from_start = (frame - start_frames + 1) as f32;
    let to_end = (end_frames - frame) as f32;
    (from_start / fade).min(to_end / fade).min(1.0)
}

/// Per-frame interpolated source read shared by the `Samples` and `Stream`
/// paths: polyphase windowed-sinc when rate-converted, clamped lerp at 1:1.
/// `fetch(source_frame, channel)` returns the source sample or `None` when
/// that frame is unavailable (off the buffer for samples, not currently
/// held for streams). Returns `None` — render silence, and for streams
/// count an underrun — when the center frame itself is unavailable;
/// unavailable outer sinc taps just contribute zero, matching the buffer
/// path's edge behavior. Arithmetic and accumulation order are identical to
/// the historical `Samples` implementation (golden-hash stable).
#[inline]
fn interpolate_source_frame(
    fetch: &impl Fn(i64, usize) -> Option<f32>,
    source_index: u64,
    fraction: f64,
    table: Option<&PolyphaseInterpolationTable>,
    channel: usize,
) -> Option<f32> {
    match table {
        // Rate conversion: polyphase windowed-sinc tap dot product (table
        // reads only — no allocation, no transcendentals).
        Some(table) => {
            fetch(source_index as i64, channel)?;
            let row = table.phase_row(fraction);
            let first = table.first_tap_offset();
            let mut acc = 0.0f32;
            for (tap, coefficient) in row.iter().enumerate() {
                let tap_index = source_index as i64 + first as i64 + tap as i64;
                if let Some(value) = fetch(tap_index, channel) {
                    acc += value * coefficient;
                }
            }
            Some(acc)
        }
        // 1:1 playback: direct read with last-frame clamp (`fetch` wraps
        // instead when the source loops).
        None => {
            let a = fetch(source_index as i64, channel)?;
            let b = fetch(source_index as i64 + 1, channel).unwrap_or(a);
            Some(a + (b - a) * fraction as f32)
        }
    }
}

/// Render a lane stage's clips into its scratch at unity gain (stage and edge
/// gains apply downstream, where the scratch is consumed). A clip source whose
/// channel count matches the stage writes directly; a mono or other-count
/// source is up/down-mixed into the stage format through the adapter matrix
/// precompiled on the compiled source.
/// `loop_end_frame` is the active loop region's end while the playhead is
/// inside the region (`None` otherwise): stream clips use it to retire held
/// chunks past the loop end, which would otherwise sit "ahead but within the
/// lookahead" forever on short loops and starve the held slots after a wrap.
fn render_clips_into_scratch(
    clips: &mut [CompiledClip],
    scratch: &mut [f32],
    channels: usize,
    block_start_frame: u64,
    frame_count: usize,
    loop_end_frame: Option<u64>,
) {
    let block_end_frame = block_start_frame + frame_count as u64;
    for clip in clips.iter_mut() {
        // Skip clips entirely outside this block.
        if clip.end_frames <= block_start_frame || clip.start_frames >= block_end_frame {
            continue;
        }
        let clip_start = clip.start_frames;
        let clip_end = clip.end_frames;
        let clip_fade = clip.edge_fade_frames;
        match &mut clip.source {
            CompiledSource::Silence => {}
            CompiledSource::Tone { phase, step } => {
                let mut local_phase = *phase;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    let sample = local_phase.sin();
                    local_phase += *step;
                    if local_phase >= std::f32::consts::TAU {
                        local_phase -= std::f32::consts::TAU;
                    }
                    if frame >= clip_start && frame < clip_end {
                        let sample =
                            sample * clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                        let base = frame_index * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += sample;
                        }
                    }
                }
                *phase = local_phase;
            }
            CompiledSource::Samples {
                buffer,
                step,
                loop_source,
                table,
                channel_adapter,
            } => {
                let source_frames = buffer.frame_count();
                if source_frames == 0 {
                    continue;
                }
                let data = &buffer.frames;
                let source_channels = buffer.channels.max(1) as usize;
                let frame_total = source_frames as i64;
                let loop_source = *loop_source;
                // Loop folding lives in the fetch: wrapped taps and the
                // wrapped lerp neighbour come out identical to the
                // historical inline implementation.
                let fetch = |source_frame: i64, channel: usize| -> Option<f32> {
                    let source_frame = if loop_source {
                        source_frame.rem_euclid(frame_total)
                    } else {
                        source_frame
                    };
                    if source_frame < 0 || source_frame >= frame_total {
                        return None;
                    }
                    Some(data[source_frame as usize * source_channels + channel])
                };
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    if frame < clip_start || frame >= clip_end {
                        continue;
                    }
                    // Source position via the rate ratio, anchored at the
                    // clip's window start.
                    let mut source_position = (frame - clip_start) as f64 * *step;
                    if loop_source {
                        source_position %= source_frames as f64;
                    }
                    let source_index = source_position as usize;
                    if source_index >= source_frames {
                        continue;
                    }
                    let fraction = source_position - source_index as f64;
                    let window_gain = clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                    let base = frame_index * channels;
                    match channel_adapter {
                        // Matched channel counts: direct per-channel read (the
                        // historical stereo path when both are 2).
                        None => {
                            for channel in 0..channels {
                                if let Some(sample) = interpolate_source_frame(
                                    &fetch,
                                    source_index as u64,
                                    fraction,
                                    table.as_ref(),
                                    channel,
                                ) {
                                    scratch[base + channel] += sample * window_gain;
                                }
                            }
                        }
                        // Mismatched: up/down-mix each source channel into the
                        // stage channels via the precompiled adapter matrix.
                        Some(matrix) => {
                            for source_channel in 0..source_channels {
                                let Some(sample) = interpolate_source_frame(
                                    &fetch,
                                    source_index as u64,
                                    fraction,
                                    table.as_ref(),
                                    source_channel,
                                ) else {
                                    continue;
                                };
                                let sample = sample * window_gain;
                                for dest_channel in 0..channels {
                                    let coefficient = matrix[source_channel * channels + dest_channel];
                                    scratch[base + dest_channel] += sample * coefficient;
                                }
                            }
                        }
                    }
                }
            }
            CompiledSource::Stream {
                handle,
                held,
                step,
                table,
            } => {
                let total_frames = handle.inner.total_frames;
                if total_frames == 0 {
                    continue;
                }
                // Publish the read hint: clip-anchored source frame of the
                // first audible frame in this block. Seeks read as jumps.
                let first_audible = block_start_frame.max(clip_start);
                let needed_start = ((first_audible - clip_start) as f64 * *step) as u64;
                handle
                    .inner
                    .wanted_frame
                    .store(needed_start, Ordering::Relaxed);
                // Retire held chunks entirely behind the playhead (minus
                // the sinc tap margin) or stale past the seek window —
                // returned to the feeder, never freed here.
                let behind_cutoff = needed_start.saturating_sub(RESAMPLE_TAPS as u64);
                let mut ahead_cutoff = needed_start.saturating_add(STREAM_RETIRE_LOOKAHEAD_FRAMES);
                // Looping: chunks past the loop end (in this clip's source
                // frame space, plus the sinc tap margin) are useless until
                // the loop clears — retire them so refed wrap-target chunks
                // always find a held slot.
                if let Some(loop_end) = loop_end_frame {
                    let source_cap = ((loop_end.saturating_sub(clip_start)) as f64 * *step) as u64
                        + RESAMPLE_TAPS as u64;
                    ahead_cutoff = ahead_cutoff.min(source_cap.max(needed_start));
                }
                for slot in held.iter_mut() {
                    let stale = slot.as_ref().is_some_and(|chunk| {
                        chunk.end_frame() <= behind_cutoff || chunk.start_frame > ahead_cutoff
                    });
                    if stale {
                        let chunk = slot.take().expect("stale slot is occupied");
                        if let Err(chunk) = handle.inner.retired.try_push(chunk) {
                            // Retired mailbox full: hold the chunk and retry
                            // next block (never drop on the audio thread).
                            *slot = Some(chunk);
                        }
                    }
                }
                // Drain the chunk mailbox into free slots only — a popped
                // chunk must always have a home on this thread. Stale
                // arrivals occupy a slot for one block and retire on the
                // next pass.
                for slot in held.iter_mut() {
                    if slot.is_none() {
                        match handle.inner.chunks.try_pop() {
                            Some(chunk) => *slot = Some(chunk),
                            None => break,
                        }
                    }
                }
                // Backward jumps (loop wraps, seeks) can leave every held
                // slot occupied by valid-but-ahead chunks while the chunk
                // covering the read position waits in the mailbox. When
                // nothing held covers `needed_start` and no slot is free,
                // retire the furthest-ahead chunk so refed wrap-target
                // chunks find a home within a block or two (transient
                // underrun by design, never a permanent stall).
                let covered = held.iter().flatten().any(|chunk| {
                    needed_start >= chunk.start_frame && needed_start < chunk.end_frame()
                });
                if !covered {
                    let furthest = (0..held.len())
                        .filter(|index| held[*index].is_some())
                        .max_by_key(|index| {
                            held[*index]
                                .as_ref()
                                .map(|chunk| chunk.start_frame)
                                .unwrap_or(0)
                        });
                    let all_full = held.iter().all(|slot| slot.is_some());
                    if let (true, Some(index)) = (all_full, furthest) {
                        let chunk = held[index].take().expect("occupied slot");
                        if let Err(chunk) = handle.inner.retired.try_push(chunk) {
                            // Retired mailbox full: keep holding (never drop
                            // on the audio thread) and retry next block.
                            held[index] = Some(chunk);
                        }
                    }
                }
                let held: &[Option<StreamChunk>] = held;
                // Locate a source frame in the held chunks: linear scan over
                // a handful of slots, then an index.
                let fetch = |source_frame: i64, channel: usize| -> Option<f32> {
                    if source_frame < 0 {
                        return None;
                    }
                    let source_frame = source_frame as u64;
                    held.iter().flatten().find_map(|chunk| {
                        (source_frame >= chunk.start_frame && source_frame < chunk.end_frame())
                            .then(|| {
                                chunk.frames
                                    [(source_frame - chunk.start_frame) as usize * 2 + channel]
                            })
                    })
                };
                let mut underrun_frames = 0u64;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    if frame < clip_start || frame >= clip_end {
                        continue;
                    }
                    let source_position = (frame - clip_start) as f64 * *step;
                    let source_index = source_position as u64;
                    if source_index >= total_frames {
                        // Past the asset's end: silence by design, not an
                        // underrun.
                        continue;
                    }
                    let fraction = source_position - source_index as f64;
                    let window_gain = clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                    let base = frame_index * channels;
                    let mut missing = false;
                    for channel in 0..channels.min(2) {
                        match interpolate_source_frame(
                            &fetch,
                            source_index,
                            fraction,
                            table.as_ref(),
                            channel,
                        ) {
                            Some(sample) => scratch[base + channel] += sample * window_gain,
                            None => missing = true,
                        }
                    }
                    if missing {
                        underrun_frames += 1;
                    }
                }
                if underrun_frames > 0 {
                    handle
                        .inner
                        .underrun_frames
                        .fetch_add(underrun_frames, Ordering::Relaxed);
                }
            }
            CompiledSource::Notes {
                buffer,
                steps,
                attack_frames,
                release_frames,
                max_duration_frames,
            } => {
                let notes = &buffer.notes;
                if notes.is_empty() {
                    continue;
                }
                // Audible span of this block inside the clip window, in
                // clip-relative frames (notes are clip-relative).
                let span_start = block_start_frame.max(clip_start);
                let span_end = block_end_frame.min(clip_end);
                if span_start >= span_end {
                    continue;
                }
                let local_start = span_start - clip_start;
                let local_end = span_end - clip_start;
                let attack_frames = *attack_frames;
                let release_frames = *release_frames;
                // Sorted overlap scan: a note sounding in this block starts
                // at most (longest duration + release tail) before it, so
                // binary-search the first candidate and walk until starts
                // pass the block end. Zero-alloc: search + arithmetic only.
                let lookback = max_duration_frames.saturating_add(release_frames);
                let window_lo = local_start.saturating_sub(lookback);
                let first = notes.partition_point(|note| note.start_frame < window_lo);
                let mut sounding = 0usize;
                for (note, step) in notes[first..].iter().zip(steps[first..].iter()) {
                    if note.start_frame >= local_end {
                        break;
                    }
                    let note_end = note.start_frame.saturating_add(note.duration_frames);
                    let tail_end = note_end.saturating_add(release_frames);
                    if tail_end <= local_start {
                        continue;
                    }
                    // Polyphony cap, per block: the scan runs in start
                    // order, so the earliest-started notes render and the
                    // rest skip deterministically.
                    if sounding >= NOTE_POLYPHONY_LIMIT {
                        break;
                    }
                    sounding += 1;
                    let render_lo = local_start.max(note.start_frame);
                    let render_hi = local_end.min(tail_end);
                    let amplitude = note.velocity.clamp(0.0, 1.0);
                    for local_frame in render_lo..render_hi {
                        // STATELESS voice: phase and envelope are pure
                        // functions of the note-relative position, so seeks
                        // and plan swaps reproduce the exact same samples.
                        let rel = local_frame - note.start_frame;
                        let phase = (rel as f64 * step) % std::f64::consts::TAU;
                        let attack_gain = if rel < attack_frames {
                            (rel + 1) as f32 / attack_frames as f32
                        } else {
                            1.0
                        };
                        let release_gain = if rel >= note.duration_frames {
                            1.0 - (rel - note.duration_frames) as f32 / release_frames as f32
                        } else {
                            1.0
                        };
                        let frame = clip_start + local_frame;
                        let window_gain = clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                        let sample = phase.sin() as f32
                            * amplitude
                            * attack_gain
                            * release_gain
                            * window_gain;
                        let base = (frame - block_start_frame) as usize * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += sample;
                        }
                    }
                }
            }
            CompiledSource::LiveInput { handle } => {
                // Position-independent: the clip window only gates WHETHER
                // live input is audible in this block; the content is always
                // the ring's "now". Frames pop in order across segments and
                // blocks, so monitored audio is continuous.
                let span_start = block_start_frame.max(clip_start);
                let span_end = block_end_frame.min(clip_end);
                let span = (span_end - span_start) as usize;
                if span == 0 {
                    continue;
                }
                let ring = &handle.inner.ring;
                let mut chunk = [0.0f32; LIVE_INPUT_CHUNK_FRAMES * 2];
                // Backlog trim: input pushed while the executor was not
                // draining (transport stopped, plan absent) is stale — it
                // would replay as added monitoring latency. Discard down to
                // the bounded backlog before rendering. Alloc-free.
                let buffered = ring.len() / 2;
                let keep = span + LIVE_INPUT_MAX_BACKLOG_FRAMES;
                if buffered > keep {
                    let mut discard = buffered - keep;
                    while discard > 0 {
                        let take = discard.min(LIVE_INPUT_CHUNK_FRAMES);
                        let popped = ring.pop_slice(&mut chunk[..take * 2]) / 2;
                        if popped == 0 {
                            break;
                        }
                        discard -= popped;
                    }
                }
                // Drain up to `span` frames into the window's buffer span at
                // unity (stage/edge gains apply downstream), with the same
                // window edge declick as every other source.
                let mut frame_offset = (span_start - block_start_frame) as usize;
                let mut remaining = span;
                while remaining > 0 {
                    let take = remaining.min(LIVE_INPUT_CHUNK_FRAMES);
                    let popped = ring.pop_slice(&mut chunk[..take * 2]) / 2;
                    for index in 0..popped {
                        let frame = block_start_frame + (frame_offset + index) as u64;
                        let window_gain = clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                        let base = (frame_offset + index) * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += chunk[index * 2 + channel] * window_gain;
                        }
                    }
                    frame_offset += popped;
                    remaining -= popped;
                    if popped < take {
                        // Ring dry: the rest of the span is an underrun —
                        // silence (scratch untouched) plus the counter.
                        handle
                            .inner
                            .underrun_frames
                            .fetch_add(remaining as u64, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }
    }
}

enum RenderCommand {
    InstallPlan(Box<RenderPlan>),
    SetPlaying(bool),
    Seek(u64),
    /// Transport loop region `[start, end)` on the stream clock; `None`
    /// clears it. Validated control-side (`start < end`).
    SetLoopRegion(Option<(u64, u64)>),
    SetStreamChannels(u16),
    /// Parameter fast path: retarget one stage's smoothed gain without a
    /// plan recompile. `stage_index` addresses the ACTIVE plan's topological
    /// stage list; the controller resolves stage ids against the topology of
    /// the most recent install, and the FIFO mailbox guarantees the command
    /// lands after that plan.
    SetStageGain {
        stage_index: usize,
        target: f32,
    },
}

/// One shared meter slot: peak and RMS of the most recently rendered block
/// for one stage, stored as `f32::to_bits` patterns.
///
/// All accesses are `Relaxed` and the peak/RMS pair is not read atomically
/// as a unit: a reader can observe one field from block N and the other
/// from block N+1. That tearing is tolerated by design — meters are
/// cosmetic UI signal, never control flow.
#[derive(Debug)]
struct MeterSlot {
    peak_bits: AtomicU32,
    rms_bits: AtomicU32,
}

impl Default for MeterSlot {
    fn default() -> Self {
        MeterSlot {
            peak_bits: AtomicU32::new(0),
            rms_bits: AtomicU32::new(0),
        }
    }
}

/// Counters shared between the two sides without locks.
#[derive(Debug)]
struct SharedState {
    /// Stream-clock position in frames, written by the executor.
    position_frames: AtomicU64,
    /// Transport gate as last applied by the executor.
    playing: AtomicBool,
    /// Blocks rendered while a retired plan could not be returned because the
    /// retired mailbox was full (plan held in the parking slot instead —
    /// never dropped on the audio thread).
    retired_parked_blocks: AtomicU64,
    /// Fixed-capacity per-stage meter table: slot `i` holds the meter for
    /// the active plan's topological stage `i`. Stages past
    /// [`METER_SLOT_CAPACITY`] are silently unmetered.
    meter_slots: [MeterSlot; METER_SLOT_CAPACITY],
    /// Generation stamp of the plan the meter slots currently describe
    /// (assigned control-side per install, written by the executor alongside
    /// the slots). Readers compare it against their last install to map
    /// slots → stage ids; a mismatch means the executor has not switched to
    /// the latest plan yet and the slots describe the previous topology.
    meter_generation: AtomicU64,
    /// Total render callbacks observed (incremented once per `render_block`).
    callback_count: AtomicU64,
    /// Wall-clock duration of the most recent callback, in microseconds.
    last_callback_duration_micros: AtomicU64,
    /// Maximum observed callback duration, in microseconds.
    max_callback_duration_micros: AtomicU64,
    /// Inferred missed deadlines: callbacks whose interval since the
    /// previous callback exceeded [`XRUN_INTERVAL_FACTOR`] × the block
    /// duration at the plan rate.
    xrun_count: AtomicU64,
}

impl Default for SharedState {
    fn default() -> Self {
        SharedState {
            position_frames: AtomicU64::new(0),
            playing: AtomicBool::new(false),
            retired_parked_blocks: AtomicU64::new(0),
            meter_slots: std::array::from_fn(|_| MeterSlot::default()),
            meter_generation: AtomicU64::new(0),
            callback_count: AtomicU64::new(0),
            last_callback_duration_micros: AtomicU64::new(0),
            max_callback_duration_micros: AtomicU64::new(0),
            xrun_count: AtomicU64::new(0),
        }
    }
}

/// Control-side handle: compiles and installs plans, drives transport, and
/// deallocates retired plans.
pub struct RenderPlaneController {
    commands: SyncSender<RenderCommand>,
    retired: Receiver<Box<RenderPlan>>,
    shared: Arc<SharedState>,
    /// Stream channel count as reported by the host. Plans compile their
    /// hardware-boundary adaptation against this; before it is known, plans
    /// assume the stream matches their master format.
    stream_channels: Option<u16>,
    /// Identity snapshot of the last successfully installed plan, in its
    /// topological stage order: (stage_id, clip ids). Used to precompute the
    /// state-inheritance maps for the next install so the executor does pure
    /// index copies.
    last_topology: Option<Vec<(u64, Vec<u64>)>>,
    /// Generation stamp of the most recent successful install; compared
    /// against the shared meter generation when resolving meter slots.
    plan_generation: u64,
}

/// Error installing a plan or sending a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlaneError {
    /// Human-readable description.
    pub message: String,
}

impl std::fmt::Display for RenderPlaneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "render plane error: {}", self.message)
    }
}

impl std::error::Error for RenderPlaneError {}

impl RenderPlaneController {
    /// Record the channel count of the opened output stream and inform the
    /// executor (output framing keys off the *stream*, not the plan's master
    /// format). Subsequent installs compile their hardware-boundary
    /// adaptation against this count.
    pub fn set_stream_channels(&mut self, channels: u16) -> Result<(), RenderPlaneError> {
        self.stream_channels = Some(channels);
        self.commands
            .try_send(RenderCommand::SetStreamChannels(channels))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected stream channel update: {error}"),
            })
    }

    /// Compile `spec` and install it as the active plan. Eagerly reclaims
    /// any plans the executor has retired. State-inheritance maps (stage and
    /// clip identity vs the previously installed plan) are precomputed here,
    /// control-side, so the executor's hand-off is pure index copies.
    pub fn install_plan(&mut self, spec: &RenderPlanSpec) -> Result<(), RenderPlaneError> {
        self.collect_retired();
        let mut plan =
            RenderPlan::compile(spec, self.stream_channels).map_err(|error| RenderPlaneError {
                message: format!("plan rejected at compile: {error}"),
            })?;

        // Topology of the freshly compiled plan (topo order).
        let topology: Vec<(u64, Vec<u64>)> = plan
            .stages
            .iter()
            .map(|stage| {
                (
                    stage.stage_id,
                    stage.clips.iter().map(|clip| clip.clip_id).collect(),
                )
            })
            .collect();

        if let Some(previous) = self.last_topology.as_ref() {
            plan.inherit_stage_map = topology
                .iter()
                .map(|(stage_id, _)| {
                    previous
                        .iter()
                        .position(|(previous_id, _)| previous_id == stage_id)
                })
                .collect();
            plan.inherit_clip_maps = topology
                .iter()
                .enumerate()
                .map(|(index, (_, clip_ids))| {
                    let Some(previous_index) = plan.inherit_stage_map[index] else {
                        return Vec::new();
                    };
                    let previous_clips = &previous[previous_index].1;
                    clip_ids
                        .iter()
                        .map(|clip_id| {
                            previous_clips
                                .iter()
                                .position(|previous_id| previous_id == clip_id)
                        })
                        .collect()
                })
                .collect();
        }

        plan.generation = self.plan_generation + 1;
        self.commands
            .try_send(RenderCommand::InstallPlan(plan))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected plan install: {error}"),
            })?;
        self.plan_generation += 1;
        self.last_topology = Some(topology);
        Ok(())
    }

    /// Parameter fast path: retarget one stage's smoothed gain without a
    /// plan recompile. Resolves `stage_id` against the topology of the most
    /// recent successful install; the FIFO mailbox guarantees the command
    /// reaches the executor after that plan. Returns an error when the
    /// stage is unknown (callers should fall back to a plan install).
    pub fn set_stage_gain(&self, stage_id: u64, target: f32) -> Result<(), RenderPlaneError> {
        let Some(topology) = self.last_topology.as_ref() else {
            return Err(RenderPlaneError {
                message: "no plan installed; cannot set stage gain".to_string(),
            });
        };
        let Some(stage_index) = topology.iter().position(|(id, _)| *id == stage_id) else {
            return Err(RenderPlaneError {
                message: format!("stage {stage_id} not present in the installed plan"),
            });
        };
        self.commands
            .try_send(RenderCommand::SetStageGain {
                stage_index,
                target,
            })
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected stage gain: {error}"),
            })
    }

    /// Gate rendering on or off (transport play/stop).
    pub fn set_playing(&self, playing: bool) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::SetPlaying(playing))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected transport gate: {error}"),
            })
    }

    /// Set (or clear, with `None`) the transport loop region `[start, end)`
    /// on the stream clock. While playing, a block that crosses `end` wraps
    /// to `start` sample-accurately inside the executor (a control-side seek
    /// would jitter by a mailbox round-trip), with a short micro-fade around
    /// the wrap point. Seeking outside the region is allowed; the loop only
    /// triggers when playback crosses `end`. Rejects `start >= end`.
    pub fn set_loop_region(&self, region: Option<(u64, u64)>) -> Result<(), RenderPlaneError> {
        if let Some((start, end)) = region {
            if start >= end {
                return Err(RenderPlaneError {
                    message: format!("loop region start {start} must be before end {end}"),
                });
            }
        }
        self.commands
            .try_send(RenderCommand::SetLoopRegion(region))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected loop region: {error}"),
            })
    }

    /// Move the stream clock to `position_frames`.
    pub fn seek(&self, position_frames: u64) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::Seek(position_frames))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected seek: {error}"),
            })
    }

    /// Current stream-clock position in frames as last written by the
    /// executor.
    pub fn position_frames(&self) -> u64 {
        self.shared.position_frames.load(Ordering::Relaxed)
    }

    /// Whether the executor is currently rendering (transport gate as
    /// applied).
    pub fn playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    /// Drain and deallocate plans the executor has retired. Call this
    /// periodically from the control side; returns how many were freed.
    pub fn collect_retired(&self) -> usize {
        let mut freed = 0;
        while self.retired.try_recv().is_ok() {
            freed += 1;
        }
        freed
    }

    /// Diagnostic: blocks during which a retired plan sat parked because the
    /// retired mailbox was full.
    pub fn retired_parked_blocks(&self) -> u64 {
        self.shared.retired_parked_blocks.load(Ordering::Relaxed)
    }

    /// Per-stage meters from the most recently rendered block:
    /// `(stage_id, peak, rms)` in the installed plan's topological order.
    ///
    /// Slots are resolved against the topology of the controller's last
    /// successful install (slot `i` = topological stage `i`). When the
    /// executor has not yet switched to that plan (the shared meter table
    /// still carries an older generation), this returns an empty vec rather
    /// than mislabeling slots — the gap lasts at most a block. Stages past
    /// [`METER_SLOT_CAPACITY`] are silently unmetered. Values are read with
    /// relaxed ordering and may tear between peak and RMS of adjacent
    /// blocks; meters are cosmetic.
    pub fn meters(&self) -> Vec<(u64, f32, f32)> {
        let Some(topology) = self.last_topology.as_ref() else {
            return Vec::new();
        };
        if self.shared.meter_generation.load(Ordering::Relaxed) != self.plan_generation {
            return Vec::new();
        }
        topology
            .iter()
            .take(METER_SLOT_CAPACITY)
            .enumerate()
            .map(|(index, (stage_id, _))| {
                let slot = &self.shared.meter_slots[index];
                (
                    *stage_id,
                    f32::from_bits(slot.peak_bits.load(Ordering::Relaxed)),
                    f32::from_bits(slot.rms_bits.load(Ordering::Relaxed)),
                )
            })
            .collect()
    }

    /// Total render callbacks observed by the executor.
    pub fn callback_count(&self) -> u64 {
        self.shared.callback_count.load(Ordering::Relaxed)
    }

    /// Wall-clock duration of the most recent render callback, microseconds.
    pub fn last_callback_duration_micros(&self) -> u64 {
        self.shared
            .last_callback_duration_micros
            .load(Ordering::Relaxed)
    }

    /// Maximum observed render-callback duration, microseconds.
    pub fn max_callback_duration_micros(&self) -> u64 {
        self.shared
            .max_callback_duration_micros
            .load(Ordering::Relaxed)
    }

    /// Inferred missed deadlines: callbacks arriving later than
    /// 1.5 × the block duration after their predecessor.
    pub fn xrun_count(&self) -> u64 {
        self.shared.xrun_count.load(Ordering::Relaxed)
    }
}

/// Render-side executor. Owns the active plan between blocks and renders it
/// inside the audio callback.
///
/// # Real-time contract
///
/// [`RenderPlaneExecutor::render_block`] performs no allocation, takes no
/// locks, and does no I/O. Plan hand-off uses bounded array-backed channels;
/// retired plans are returned to the control side (or parked locally when the
/// return mailbox is full) and are never dropped on the audio thread.
pub struct RenderPlaneExecutor {
    commands: Receiver<RenderCommand>,
    retired: SyncSender<Box<RenderPlan>>,
    shared: Arc<SharedState>,
    plan: Option<Box<RenderPlan>>,
    parked_retired: Option<Box<RenderPlan>>,
    /// Stream channel count as told by the controller; the active plan's
    /// expectation (its master format when no stream was known at install)
    /// applies until the first [`RenderCommand::SetStreamChannels`].
    stream_channels: Option<u16>,
    /// Transport gate target. Audio follows through `edge_gain`.
    playing: bool,
    /// Transport edge envelope: ramps toward 1 when playing, 0 when stopped
    /// or before an in-flight seek, so transport actions never step audio.
    edge_gain: f32,
    /// Seek requested while audible: applied once `edge_gain` reaches zero.
    pending_seek: Option<u64>,
    /// Timeline position abandoned by the most recently applied seek. The
    /// next event-bearing block uses it to release old notes and chase the
    /// destination state, then clears it.
    event_discontinuity_from: Option<u64>,
    /// Transport loop region `[start, end)`; playback wraps to `start` when
    /// a rendered block crosses `end` (see `render_block_inner`).
    loop_region: Option<(u64, u64)>,
    position_frames: u64,
    /// Start instant of the previous callback, for xrun inference.
    last_callback_instant: Option<Instant>,
}

impl RenderPlaneExecutor {
    /// Offline-bounce hook: snap the transport edge envelope to `gain`
    /// without ramping. Realtime transport must NEVER use this — the 5 ms
    /// edge ramp is what keeps play/stop/seek click-free on speakers — but
    /// an offline export has no speaker and must start at full level, so
    /// the offline driver (`offline::render_plan_to_pcm`) snaps the
    /// envelope open after draining transport commands and before the first
    /// rendered block. Crate-private on purpose: hosts cannot reach it.
    pub(crate) fn set_edge_gain_immediate(&mut self, gain: f32) {
        self.edge_gain = gain.clamp(0.0, 1.0);
    }

    fn retire(&mut self, plan: Box<RenderPlan>) {
        // Re-offer any parked plan first to preserve retirement order.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }
        if let Err(TrySendError::Full(plan)) | Err(TrySendError::Disconnected(plan)) =
            self.retired.try_send(plan)
        {
            self.shared
                .retired_parked_blocks
                .fetch_add(1, Ordering::Relaxed);
            debug_assert!(
                self.parked_retired.is_none(),
                "retired mailbox capacity invariant violated",
            );
            // Never deallocate on the audio thread: hold the plan and
            // re-offer it next block. The mailbox capacity invariant makes
            // double saturation unreachable in correct use.
            if self.parked_retired.is_none() {
                self.parked_retired = Some(plan);
            }
        }
    }

    fn apply_seek(&mut self, position_frames: u64) {
        self.event_discontinuity_from = Some(self.position_frames);
        self.position_frames = position_frames;
        self.shared
            .position_frames
            .store(position_frames, Ordering::Relaxed);
    }

    fn drain_commands(&mut self) {
        loop {
            match self.commands.try_recv() {
                Ok(RenderCommand::InstallPlan(mut next_plan)) => {
                    if let Some(mut previous) = self.plan.take() {
                        next_plan.inherit_state(&mut previous);
                        self.retire(previous);
                    }
                    self.plan = Some(next_plan);
                }
                Ok(RenderCommand::SetPlaying(playing)) => {
                    self.playing = playing;
                    self.shared.playing.store(playing, Ordering::Relaxed);
                }
                Ok(RenderCommand::Seek(position_frames)) => {
                    if self.edge_gain <= 0.0 {
                        // Inaudible: jump immediately.
                        self.pending_seek = None;
                        self.apply_seek(position_frames);
                    } else {
                        // Audible: ramp out first, jump at the zero crossing,
                        // ramp back in (handled in render_block).
                        self.pending_seek = Some(position_frames);
                    }
                }
                Ok(RenderCommand::SetStageGain {
                    stage_index,
                    target,
                }) => {
                    if let Some(plan) = self.plan.as_mut() {
                        if let Some(stage) = plan.stages.get_mut(stage_index) {
                            stage.gain_target = target;
                        }
                    }
                }
                Ok(RenderCommand::SetLoopRegion(region)) => {
                    self.loop_region = region;
                }
                Ok(RenderCommand::SetStreamChannels(channels)) => {
                    self.stream_channels = Some(channels.max(1));
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Render one callback quantum into `frames` (interleaved f32 at the
    /// stream's channel count).
    ///
    /// Safe on the audio thread: no allocation, no locks, no I/O.
    /// (`Instant::now()` is the one syscall-shaped exception, used for
    /// callback-health timing: it is `mach_absolute_time` on macOS and
    /// `clock_gettime(CLOCK_MONOTONIC)` elsewhere — no allocation, no lock,
    /// vDSO/commpage fast paths — and is the accepted way to observe
    /// callback cadence.)
    pub fn render_block(&mut self, frames: &mut [f32]) {
        let callback_start = Instant::now();
        // Flush-to-zero for the whole callback: feedback DSP (limiter
        // release, future filters) cannot decay into denormal range and burn
        // CPU. RAII restores the FP control register on exit.
        let _denormals = DenormalGuard::new();
        self.render_block_inner(frames);
        self.publish_callback_health(callback_start, frames.len());
    }

    /// Publish callback-health counters: count, duration (last/max), and
    /// inferred xruns. An xrun is an interval since the previous callback
    /// longer than [`XRUN_INTERVAL_FACTOR`] × the block duration at the
    /// active plan's rate; without a plan no xrun can be inferred (the
    /// expected cadence is unknown) but count and duration still publish.
    fn publish_callback_health(&mut self, callback_start: Instant, samples_len: usize) {
        let shared = &self.shared;
        shared.callback_count.fetch_add(1, Ordering::Relaxed);
        let duration_micros = callback_start.elapsed().as_micros() as u64;
        shared
            .last_callback_duration_micros
            .store(duration_micros, Ordering::Relaxed);
        shared
            .max_callback_duration_micros
            .fetch_max(duration_micros, Ordering::Relaxed);
        if let (Some(previous), Some(plan)) = (self.last_callback_instant, self.plan.as_ref()) {
            let stream_channels = self
                .stream_channels
                .map(|channels| channels as usize)
                .unwrap_or(plan.stream_channels)
                .max(1);
            let frame_count = samples_len / stream_channels;
            let block_seconds = frame_count as f64 / plan.sample_rate_hz.max(1) as f64;
            let interval = callback_start.duration_since(previous).as_secs_f64();
            if frame_count > 0 && interval > block_seconds * XRUN_INTERVAL_FACTOR {
                shared.xrun_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.last_callback_instant = Some(callback_start);
    }

    /// Write per-stage peak/RMS for this block into the shared meter table
    /// and stamp the plan generation. Levels are taken from each stage's
    /// scratch (pre-consumption) and scaled by the stage's end-of-block
    /// smoothed gain so fader moves read on the meters — a per-block
    /// approximation of the post-fader level (the transport edge ramp is
    /// not included). Cheap loops over already-rendered scratch: no
    /// allocation. Stages past [`METER_SLOT_CAPACITY`] are unmetered.
    fn publish_meters(shared: &SharedState, plan: &RenderPlan, frame_count: usize) {
        for (index, stage) in plan.stages.iter().take(METER_SLOT_CAPACITY).enumerate() {
            let samples = &stage.scratch[..frame_count * stage.channels];
            let mut peak = 0.0f32;
            let mut sum_squares = 0.0f32;
            for sample in samples {
                let magnitude = sample.abs();
                if magnitude > peak {
                    peak = magnitude;
                }
                sum_squares += sample * sample;
            }
            let gain = stage.gain_current.abs();
            let rms = (sum_squares / samples.len().max(1) as f32).sqrt() * gain;
            let slot = &shared.meter_slots[index];
            slot.peak_bits
                .store((peak * gain).to_bits(), Ordering::Relaxed);
            slot.rms_bits.store(rms.to_bits(), Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }

    /// Zero the active plan's meter slots (silence: stopped or fully ramped
    /// out) and stamp the generation so readers see live zeros, not stale
    /// levels from the last audible block.
    fn publish_silent_meters(shared: &SharedState, plan: &RenderPlan) {
        for slot in shared
            .meter_slots
            .iter()
            .take(plan.stages.len().min(METER_SLOT_CAPACITY))
        {
            slot.peak_bits.store(0, Ordering::Relaxed);
            slot.rms_bits.store(0, Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }

    fn render_block_inner(&mut self, frames: &mut [f32]) {
        self.drain_commands();

        // Re-offer a parked retired plan without rendering work attached.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }

        frames.fill(0.0);

        let Some(plan) = self.plan.as_mut() else {
            return;
        };
        // Audible while the gate is open, while ramping out after a stop, or
        // while ramping around an in-flight seek.
        if !self.playing && self.edge_gain <= 0.0 {
            // Silent block: meters read zero rather than holding the last
            // audible level.
            Self::publish_silent_meters(&self.shared, plan);
            return;
        }

        let stream_channels = self
            .stream_channels
            .map(|channels| channels as usize)
            .unwrap_or(plan.stream_channels)
            .max(1);
        let frame_count = frames.len() / stream_channels;
        debug_assert!(
            frame_count <= MAX_BLOCK_FRAMES,
            "render_block called with {frame_count} frames; scratch holds {MAX_BLOCK_FRAMES}",
        );
        let frame_count = frame_count.min(MAX_BLOCK_FRAMES);
        let block_start_frame = self.position_frames;
        let playing = self.playing;

        // Loop-region segmentation: while playing with a region set, a block
        // whose span crosses `loop_end` renders as (up to) TWO timeline
        // segments into one output buffer — `[pos, loop_end)` then the
        // remainder from `loop_start`. Each segment is
        // `(timeline_start, buffer_frame_offset, frame_count)`. Segments only
        // change the timeline positions clips see; gain smoothing, automation
        // sampling, meters, and the boundary/limiter/edge paths stay
        // per-BLOCK. Seeks outside the region are allowed — the wrap only
        // triggers when the block actually crosses `loop_end`.
        let mut segments: [(u64, usize, usize); 2] =
            [(block_start_frame, 0, frame_count), (0, 0, 0)];
        let mut segment_count = 1usize;
        // Buffer frame index where the wrap lands (first frame rendered from
        // `loop_start`); drives the wrap micro-fade below.
        let mut wrap_offset: Option<usize> = None;
        let mut end_position = block_start_frame + frame_count as u64;
        if let Some((loop_start, loop_end)) = self.loop_region {
            let block_end_frame = block_start_frame + frame_count as u64;
            if self.playing && block_start_frame < loop_end && block_end_frame >= loop_end {
                let first = (loop_end - block_start_frame) as usize;
                let remainder = frame_count - first;
                segments[0] = (block_start_frame, 0, first);
                segments[1] = (loop_start, first, remainder);
                segment_count = 2;
                wrap_offset = Some(first);
                end_position = loop_start + remainder as u64;
            }
        }
        // Stream chunk-retention cap (see `render_clips_into_scratch`):
        // active only while the playhead is inside the loop region, so
        // seeking past it never churns linear-playback prefetch.
        let loop_end_frame = match self.loop_region {
            Some((_, loop_end)) if self.playing && block_start_frame < loop_end => Some(loop_end),
            _ => None,
        };

        // Smoothed gains: move toward targets at a fixed full-swing rate and
        // interpolate across the block, so edits never step audio.
        let gain_step =
            frame_count as f32 / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0);

        // Walk the schedule: stages are in topological order, so every edge's
        // source sits strictly earlier and split-borrowing is safe.
        for node_index in 0..plan.stages.len() {
            let (earlier, rest) = plan.stages.split_at_mut(node_index);
            let stage = &mut rest[0];

            let gain_begin = stage.gain_current;
            let gain_end = if stage.gain_envelope.is_empty() {
                gain_begin + (stage.gain_target - gain_begin).clamp(-gain_step, gain_step)
            } else {
                // Automation: the target is the envelope's value at the
                // block end. The ramp starts from the inherited smoothed
                // gain, so plan swaps and seeks stay continuous while normal
                // playback tracks the curve sample-accurately block-wise.
                sample_envelope(&stage.gain_envelope, block_start_frame + frame_count as u64)
            };
            stage.block_gain_begin = gain_begin;
            stage.block_gain_slope = (gain_end - gain_begin) / frame_count.max(1) as f32;
            stage.gain_current = gain_end;

            let CompiledNode {
                channels,
                clips,
                inputs,
                processor,
                events,
                event_scratch,
                delay_ring,
                delay_cursor,
                scratch,
                ..
            } = stage;
            let channels = *channels;
            let scratch = &mut scratch[..frame_count * channels];
            scratch.fill(0.0);

            // Lane content first (at unity), then summed inputs. Rendered
            // per loop segment: each segment writes its own buffer span with
            // its own timeline start, so clips see wrapped positions while
            // tone phases carry across the wrap (segments render in order).
            for (segment_start, segment_offset, segment_frames) in
                segments.iter().take(segment_count)
            {
                if *segment_frames == 0 {
                    continue;
                }
                render_clips_into_scratch(
                    clips,
                    &mut scratch
                        [segment_offset * channels..(segment_offset + segment_frames) * channels],
                    channels,
                    *segment_start,
                    *segment_frames,
                    loop_end_frame,
                );
            }

            for input in inputs.iter() {
                let source = &earlier[input.source_index];
                let source_channels = input.source_channels;
                let matrix = &input.matrix;
                // Per-frame: dest[c_out] += src[c_in] * m[c_in][c_out] *
                // source stage's smoothed gain (edge gain is folded into the
                // matrix at compile). Plain loops, alloc-free.
                for frame_index in 0..frame_count {
                    let source_gain =
                        source.block_gain_begin + source.block_gain_slope * frame_index as f32;
                    let source_base = frame_index * source_channels;
                    let dest_base = frame_index * channels;
                    for source_channel in 0..source_channels {
                        let sample = source.scratch[source_base + source_channel] * source_gain;
                        let row = &matrix[source_channel * channels..][..channels];
                        for (dest_channel, coefficient) in row.iter().enumerate() {
                            scratch[dest_base + dest_channel] += sample * coefficient;
                        }
                    }
                }
            }

            // Explicit graph delay: swap the summed block through the
            // preallocated interleaved ring. Cursor state spans callbacks
            // and compatible plan swaps, so the delay is sample-exact even
            // when its length exceeds the current block.
            if !delay_ring.is_empty() {
                for sample in scratch.iter_mut() {
                    std::mem::swap(sample, &mut delay_ring[*delay_cursor]);
                    *delay_cursor += 1;
                    if *delay_cursor == delay_ring.len() {
                        *delay_cursor = 0;
                    }
                }
            }

            // Plugin processor (g11.012): transforms the stage's summed
            // scratch in place before consumers read it. Bypass (`false` —
            // deadline miss, dead sandbox, unsupported layout) leaves the
            // scratch untouched by the backend contract, so the dry signal
            // flows on and plans without processors render bit-identically.
            //
            // Event delivery (g12.034 follow-up): stages carrying a compiled
            // event stream slice it per loop segment — binary search to the
            // segment start, then push events inside `[start, start+frames)`
            // with buffer-relative sample offsets into the preallocated
            // scratch (alloc-free; capacity overflow drops, earliest wins).
            // Delivery is playback-gated: while stopped (edge ramp-out) the
            // position does not advance, so re-firing the same events every
            // block would double-trigger notes.
            if let Some(processor) = processor {
                if events.is_empty() {
                    let _ = processor.process(scratch, frame_count, channels);
                } else {
                    event_scratch.clear();
                    if playing {
                        for (segment_index, (segment_start, segment_offset, segment_frames)) in
                            segments.iter().take(segment_count).enumerate()
                        {
                            if *segment_frames == 0 {
                                continue;
                            }
                            let discontinuity_from = if segment_index == 0 {
                                self.event_discontinuity_from
                            } else {
                                self.loop_region.map(|(_, loop_end)| loop_end)
                            };
                            if let Some(from) = discontinuity_from {
                                append_plugin_state_chase(
                                    events,
                                    from,
                                    *segment_start,
                                    *segment_offset as u32,
                                    event_scratch,
                                );
                            }
                            let segment_end = *segment_start + *segment_frames as u64;
                            let begin =
                                events.partition_point(|event| event.frame < *segment_start);
                            for event in events[begin..]
                                .iter()
                                .take_while(|event| event.frame < segment_end)
                            {
                                if event_scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                                    break;
                                }
                                event_scratch.push(RenderBlockPluginEvent {
                                    offset_frames: (*segment_offset as u64
                                        + (event.frame - segment_start))
                                        as u32,
                                    channel: event.channel,
                                    kind: event.kind,
                                });
                            }
                        }
                    }
                    let _ = processor.process_with_events(
                        scratch,
                        frame_count,
                        channels,
                        event_scratch,
                    );
                }
            }
        }

        // Meters: every stage's scratch for this block is final now (the
        // boundary write below only reads the master's scratch).
        Self::publish_meters(&self.shared, plan, frame_count);

        // Hardware boundary: adapt the master stage's scratch onto the stream
        // with the master's smoothed gain. Equal formats copy; a narrower
        // stream applies the install-time downmix matrix; a wider stream
        // carries the master's channels and leaves the extras silent (the
        // creative mix never upmixes at the hardware stage).
        let master = &plan.stages[plan.master_index];
        let output_channels = master.channels;
        let boundary_matrix = &plan.boundary_matrix;
        let boundary_matrix_valid = boundary_matrix.len() == output_channels * stream_channels;
        for frame_index in 0..frame_count {
            let master_gain =
                master.block_gain_begin + master.block_gain_slope * frame_index as f32;
            let source_base = frame_index * output_channels;
            let dest_base = frame_index * stream_channels;
            if stream_channels < output_channels && boundary_matrix_valid {
                for dest_channel in 0..stream_channels {
                    let mut acc = 0.0f32;
                    for source_channel in 0..output_channels {
                        acc += master.scratch[source_base + source_channel]
                            * boundary_matrix[source_channel * stream_channels + dest_channel];
                    }
                    frames[dest_base + dest_channel] = acc * master_gain;
                }
            } else {
                // Equal formats, a wider stream (extras stay zero-filled),
                // or a stale boundary (stream changed without a reinstall):
                // copy the overlapping channels.
                for channel in 0..output_channels.min(stream_channels) {
                    frames[dest_base + channel] =
                        master.scratch[source_base + channel] * master_gain;
                }
            }
        }

        // Loop-wrap declick: linear micro-fade out over the frames before
        // the wrap point and in over the frames after it, applied to the
        // output buffer only on blocks that wrap (cheap; never touches
        // non-looping renders). When the wrap lands exactly on the block
        // boundary only the fade-out applies — the next block starts from
        // `loop_start` behind the still-open edge envelope.
        if let Some(wrap_offset) = wrap_offset {
            let fade_out = LOOP_WRAP_FADE_FRAMES.min(wrap_offset);
            for step in 0..fade_out {
                let frame_index = wrap_offset - fade_out + step;
                let gain = (fade_out - 1 - step) as f32 / fade_out as f32;
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= gain;
                }
            }
            let fade_in = LOOP_WRAP_FADE_FRAMES.min(frame_count - wrap_offset);
            for step in 0..fade_in {
                let frame_index = wrap_offset + step;
                let gain = (step + 1) as f32 / fade_in as f32;
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= gain;
                }
            }
        }

        // Master soft limiter: guards the stream buffer after the boundary
        // write so the creative graph stays untouched; linked gain across
        // the stream's channels.
        if let Some(limiter) = plan.limiter.as_mut() {
            for frame_index in 0..frame_count {
                let base = frame_index * stream_channels;
                limiter.process_frame(&mut frames[base..base + stream_channels]);
            }
        }

        // Transport edge envelope over the mixed block: ramps toward the
        // gate target (zero while a seek is in flight) and never steps.
        let edge_target = if self.pending_seek.is_some() {
            0.0
        } else if self.playing {
            1.0
        } else {
            0.0
        };
        if self.edge_gain != edge_target || edge_target < 1.0 {
            let edge_step = 1.0 / (EDGE_RAMP_SECONDS * plan.sample_rate_hz as f32).max(1.0);
            for frame_index in 0..frame_count {
                self.edge_gain += (edge_target - self.edge_gain).clamp(-edge_step, edge_step);
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= self.edge_gain;
                }
            }
        }

        // After a wrap block the clock reads `loop_start + remainder`;
        // otherwise it advances linearly by the block.
        self.position_frames = end_position;
        self.shared
            .position_frames
            .store(self.position_frames, Ordering::Relaxed);

        // Seek lands at the envelope's zero crossing; the next block ramps
        // back in from the new position.
        // Any discontinuity consumed by this block is complete. A pending
        // seek applied below installs the source for the NEXT block.
        self.event_discontinuity_from = None;
        if self.edge_gain <= 0.0 {
            if let Some(position) = self.pending_seek.take() {
                self.apply_seek(position);
            }
        }
    }
}

/// Create a connected controller/executor pair.
pub fn render_plane() -> (RenderPlaneController, RenderPlaneExecutor) {
    let (command_tx, command_rx) = sync_channel(COMMAND_MAILBOX_CAPACITY);
    let (retired_tx, retired_rx) = sync_channel(RETIRED_MAILBOX_CAPACITY);
    let shared = Arc::new(SharedState::default());
    (
        RenderPlaneController {
            commands: command_tx,
            retired: retired_rx,
            shared: Arc::clone(&shared),
            stream_channels: None,
            last_topology: None,
            plan_generation: 0,
        },
        RenderPlaneExecutor {
            commands: command_rx,
            retired: retired_tx,
            shared,
            plan: None,
            parked_retired: None,
            stream_channels: None,
            playing: false,
            edge_gain: 0.0,
            pending_seek: None,
            event_discontinuity_from: None,
            loop_region: None,
            position_frames: 0,
            last_callback_instant: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_dsp::equal_power_pan_matrix;

    const MASTER_ID: u64 = 1_000;
    const LANE_ID: u64 = 1;

    fn lane_node(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
        RenderStageSpec {
            processor: None,
            events: None,
            stage_id,
            format: ChannelFormat::stereo(),
            gain,
            gain_automation: None,
            kind: RenderStageKind::Source { clips },
            inputs: Vec::new(),
        }
    }

    fn master_node(inputs: Vec<RenderEdgeSpec>) -> RenderStageSpec {
        RenderStageSpec {
            processor: None,
            events: None,
            stage_id: MASTER_ID,
            format: ChannelFormat::stereo(),
            gain: 1.0,
            gain_automation: None,
            kind: RenderStageKind::Output,
            inputs,
        }
    }

    fn identity_edge(source_stage_id: u64) -> RenderEdgeSpec {
        RenderEdgeSpec {
            source_stage_id,
            gain: 1.0,
            matrix: None,
        }
    }

    /// The old flat shape: one stereo lane summed into a stereo master.
    fn lane_master_spec(lane_gain: f32, clips: Vec<RenderClipSpec>) -> RenderPlanSpec {
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(LANE_ID, lane_gain, clips),
                master_node(vec![identity_edge(LANE_ID)]),
            ],
        }
    }

    fn tone_clip(frequency_hz: f32) -> RenderClipSpec {
        RenderClipSpec {
            clip_id: 1003,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone { frequency_hz },
            loop_source: false,
        }
    }

    fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
        lane_master_spec(0.5, vec![tone_clip(frequency_hz)])
    }

    #[test]
    fn renders_silence_without_plan_and_when_stopped() {
        let (mut controller, mut executor) = render_plane();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        controller.install_plan(&tone_spec(440.0)).unwrap();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(controller.position_frames(), 0);
    }

    #[test]
    fn renders_tone_and_advances_clock_when_playing() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
        assert_eq!(controller.position_frames(), 256);
        assert!(controller.playing());

        // Both channels carry the same mono sum.
        assert_eq!(frames[10], frames[11]);
    }

    #[test]
    fn seek_moves_the_stream_clock() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        controller.seek(96_000).unwrap();

        let mut frames = [0.0f32; 128];
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 96_000 + 64);
    }

    #[test]
    fn windows_gate_lane_audibility_on_the_stream_clock() {
        let (mut controller, mut executor) = render_plane();
        let mut clip = tone_clip(440.0);
        clip.start_frames = 128;
        clip.end_frames = 256;
        let spec = lane_master_spec(0.5, vec![clip]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Block 0 covers frames 0..128: outside the window, silent.
        let mut frames = [0.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        // Block 1 covers frames 128..256: inside the window, audible.
        let mut frames = [0.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    }

    fn samples_spec(
        values: &[f32],
        start_frames: u64,
        end_frames: u64,
        loop_source: bool,
    ) -> RenderPlanSpec {
        // Stereo frames with identical channels at the stream rate.
        let mut data = Vec::new();
        for value in values {
            data.push(*value);
            data.push(*value);
        }
        lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1004,
                start_frames,
                end_frames,
                source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                loop_source,
            }],
        )
    }

    /// Run blocks until the transport edge ramp has fully opened.
    fn warm_up(executor: &mut RenderPlaneExecutor, blocks: usize) {
        let mut frames = [0.0f32; 512];
        for _ in 0..blocks {
            executor.render_block(&mut frames);
        }
    }

    #[test]
    fn sample_clips_play_buffer_content_at_their_window() {
        let (mut controller, mut executor) = render_plane();
        // 1024 source frames: value = index / 1024.
        let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
        // Window starts at frame 512, well past the edge ramp warm-up.
        let spec = samples_spec(&values, 512, 512 + 1024, false);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Two 256-frame blocks open the edge ramp and reach frame 512.
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 512+128 plays source frame 128, past the clip edge fade.
        let index = 128usize;
        let expected = 128.0 / 1024.0;
        assert!((frames[index * 2] - expected).abs() < 1e-5);
        // Same-rate playback: equality on both channels.
        assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    }

    #[test]
    fn mono_source_upmixes_into_stereo_stage() {
        let (mut controller, mut executor) = render_plane();
        // A MONO source (channels = 1): value = index / 1024.
        let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 2001,
                start_frames: 512,
                end_frames: 512 + 1024,
                source: RenderSource::Samples(RenderSampleBuffer::mono(48_000, values.into())),
                loop_source: false,
            }],
        );
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Source frame 128, up-mixed mono→stereo at the equal-power 1/√2 gain.
        let index = 128usize;
        let expected = (128.0 / 1024.0) * std::f32::consts::FRAC_1_SQRT_2;
        assert!((frames[index * 2] - expected).abs() < 1e-5, "L = {}", frames[index * 2]);
        assert!((frames[index * 2 + 1] - expected).abs() < 1e-5, "R = {}", frames[index * 2 + 1]);
        // Mono is duplicated equally to both ears.
        assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    }

    #[test]
    fn sample_clips_play_their_final_frame() {
        let (mut controller, mut executor) = render_plane();
        // 256 source frames of a constant; window longer than the source.
        let values = vec![0.5f32; 256];
        let spec = samples_spec(&values, 0, u64::MAX, false);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 1);

        // Frames 0..256 played in the warm-up block. The final source frame
        // (255) must have rendered; beyond the source, silence.
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        // Replay from the start and inspect the last in-range frame.
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 255 is the final source frame; with the clamp it plays.
        assert!(frames[255 * 2].abs() > 0.1);
    }

    #[test]
    fn looping_sample_clips_wrap_to_their_start() {
        let (mut controller, mut executor) = render_plane();
        // 100 source frames: value = (index + 1) / 100, looped.
        let values: Vec<f32> = (0..100).map(|index| (index + 1) as f32 / 100.0).collect();
        let spec = samples_spec(&values, 0, u64::MAX, true);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2); // 512 frames: ramp open, loop wrapped 5x.

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Block covers frames 512..768; frame 512 plays source 512 % 100 = 12.
        let expected = 13.0 / 100.0;
        assert!((frames[0] - expected).abs() < 1e-5);
        // Frame 600 wraps to source 0.
        let wrapped = (600 - 512) * 2;
        assert!((frames[wrapped] - 1.0 / 100.0).abs() < 1e-5);
    }

    #[test]
    fn transport_stop_ramps_out_instead_of_stepping() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        controller.set_playing(false).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        // Ramp-out block: starts audible, ends silent, no step bigger than
        // the tone's own slope plus the ramp slope.
        assert!(frames[0].abs() > 0.0 || frames[2].abs() > 0.0);
        let tail = &frames[1000..];
        assert!(tail.iter().all(|sample| *sample == 0.0));
        let max_step = frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(max_step < 0.05, "stop produced a step of {max_step}");

        // Fully stopped afterwards: silence and a held clock.
        let position = controller.position_frames();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(controller.position_frames(), position);
    }

    #[test]
    fn seek_while_playing_ramps_out_then_jumps() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);
        let before = controller.position_frames();

        controller.seek(96_000).unwrap();
        let mut frames = [0.0f32; 512];
        // Ramp-out block at the old position; seek lands at its end.
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 96_000);
        let _ = before;
        // Next block plays from the new position, ramping back in.
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
        assert_eq!(controller.position_frames(), 96_000 + 256);
    }

    #[test]
    fn loop_region_rejects_inverted_or_empty_bounds() {
        let (controller, _executor) = render_plane();
        assert!(controller.set_loop_region(Some((100, 100))).is_err());
        assert!(controller.set_loop_region(Some((200, 100))).is_err());
        assert!(controller.set_loop_region(Some((0, 1))).is_ok());
        assert!(controller.set_loop_region(None).is_ok());
    }

    #[test]
    fn loop_region_wraps_sample_accurately_with_a_microfade() {
        // Ramp content (value = frame / 1024) under loop region [480, 992).
        // The block covering 768..1024 crosses 992 at buffer frame 224:
        // frames 0..224 play 768..992, frames 224..256 play 480..512, with
        // the 64-frame micro-fade out before the wrap and in after it.
        let (mut controller, mut executor) = render_plane();
        let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
        let spec = samples_spec(&values, 0, u64::MAX, false);
        controller.install_plan(&spec).unwrap();
        controller.set_loop_region(Some((480, 992))).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 3); // 768 frames: edge ramp open, no wrap yet.
        assert_eq!(controller.position_frames(), 768);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Clock wrapped: 480 + (256 - 224) = 512.
        assert_eq!(controller.position_frames(), 512);

        let wrap = 224usize;
        let fade = LOOP_WRAP_FADE_FRAMES;
        // Before the fade-out window: linear playback of the pre-wrap span.
        for index in [10usize, 100, wrap - fade - 1] {
            let expected = (768 + index) as f32 / 1024.0;
            assert!(
                (frames[index * 2] - expected).abs() < 1e-5,
                "pre-wrap frame {index}: {} vs {expected}",
                frames[index * 2],
            );
        }
        // Fade-out: content × linear ramp down to zero at the wrap point.
        for step in 0..fade {
            let index = wrap - fade + step;
            let gain = (fade - 1 - step) as f32 / fade as f32;
            let expected = (768 + index) as f32 / 1024.0 * gain;
            assert!(
                (frames[index * 2] - expected).abs() < 1e-5,
                "fade-out frame {index}: {} vs {expected}",
                frames[index * 2],
            );
        }
        // Fade-in: wrapped content (from loop_start) × linear ramp up.
        let fade_in = 256 - wrap; // 32 frames of post-wrap audio in the block.
        for step in 0..fade_in {
            let index = wrap + step;
            let gain = (step + 1) as f32 / fade_in as f32;
            let expected = (480 + step) as f32 / 1024.0 * gain;
            assert!(
                (frames[index * 2] - expected).abs() < 1e-5,
                "fade-in frame {index}: {} vs {expected}",
                frames[index * 2],
            );
        }

        // The next block continues linearly from loop_start + remainder.
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 768);
        let expected = (512 + 100) as f32 / 1024.0;
        assert!((frames[100 * 2] - expected).abs() < 1e-5);
    }

    #[test]
    fn seeking_outside_the_loop_region_plays_without_wrapping() {
        // Loop only triggers when crossing loop_end: a position past the
        // region renders linearly.
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_loop_region(Some((0, 256))).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 1); // Wraps once: 0..256 crosses 256.
        assert_eq!(controller.position_frames(), 0);

        controller.seek(96_000).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames); // Ramp-out block; seek lands.
        executor.render_block(&mut frames);
        executor.render_block(&mut frames);
        // Past the region: the clock advances linearly, no wrap.
        assert_eq!(controller.position_frames(), 96_000 + 512);
    }

    #[test]
    fn clearing_the_loop_region_restores_linear_playback() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_loop_region(Some((0, 256))).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 1);
        assert_eq!(controller.position_frames(), 0);

        controller.set_loop_region(None).unwrap();
        warm_up(&mut executor, 2);
        assert_eq!(controller.position_frames(), 512);
    }

    #[test]
    fn looped_stream_clips_underrun_at_most_transiently_across_wraps() {
        // A short loop over a streamed clip: the wrap jumps wanted_frame
        // backwards, the feeder re-serves it like any seek, and underruns
        // stay transient per the documented stream semantics. Lenient by
        // design — the exact underrun count depends on chunk timing.
        let (mut controller, mut executor) = render_plane();
        let total = 48_000u64;
        let (feeder, handle) = render_stream(48_000, total);
        controller
            .install_plan(&stream_spec(&handle, 0, total))
            .unwrap();
        controller.set_loop_region(Some((0, 4_096))).unwrap();
        controller.set_playing(true).unwrap();
        feed_ramp(&feeder, total, 0, 4_096, 512);

        let mut frames = [0.0f32; 512];
        let mut audible_blocks = 0usize;
        let mut rendered_frames = 0u64;
        // Cursor-based feeding mirroring the production feeder (pulse's
        // `service_streams`): sequential decode toward wanted + lookahead,
        // cursor reset on seek-shaped jumps, no duplicate re-sends.
        let chunk_frames = 512u64;
        let mut cursor = 4_096u64;
        for _ in 0..64 {
            executor.render_block(&mut frames);
            rendered_frames += 256;
            if frames.iter().any(|sample| sample.abs() > 1e-4) {
                audible_blocks += 1;
            }
            drop(feeder.collect_retired());
            let wanted = feeder.wanted_frame().min(total);
            let aligned = wanted - wanted % chunk_frames;
            let target = (wanted + 2_048).min(total);
            if cursor < aligned || cursor > target + chunk_frames {
                cursor = aligned;
            }
            while cursor < target {
                let count = chunk_frames.min(total - cursor);
                let mut data = Vec::with_capacity(count as usize * 2);
                for frame in cursor..cursor + count {
                    let value = frame as f32 / total as f32;
                    data.push(value);
                    data.push(value);
                }
                if feeder
                    .try_send_chunk(StreamChunk {
                        start_frame: cursor,
                        frames: data.into(),
                    })
                    .is_err()
                {
                    break;
                }
                cursor += count;
            }
        }
        // 64 × 256 = 16_384 frames over a 4_096-frame loop: four wraps.
        // Most blocks must have been audible and underruns must stay a
        // fraction of the rendered span (transient, not systemic).
        assert!(audible_blocks > 48, "only {audible_blocks} audible blocks");
        assert!(
            handle.underrun_frames() < rendered_frames / 4,
            "underruns not transient: {} of {rendered_frames}",
            handle.underrun_frames(),
        );
    }

    fn max_left_step(frames: &[f32]) -> f32 {
        frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max)
    }

    #[test]
    fn master_limiter_caps_a_hot_mix_at_the_boundary() {
        let (mut controller, mut executor) = render_plane();
        // Two full-scale tones summed at unity: peaks near 2.0 unlimited.
        let mut spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: Some(RenderLimiterSpec {
                threshold: 0.7,
                knee_width: 0.3,
                release_seconds: 0.05,
            }),
            stages: vec![
                lane_node(1, 1.0, vec![tone_clip(330.0)]),
                lane_node(2, 1.0, vec![tone_clip(331.0)]),
                master_node(vec![identity_edge(1), identity_edge(2)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        let mut frames = [0.0f32; 1024];
        for _ in 0..20 {
            executor.render_block(&mut frames);
            assert!(
                frames.iter().all(|sample| sample.abs() <= 1.0),
                "limited master exceeded 0 dBFS",
            );
        }
        // Without the limiter the same mix clips past 1.0 — prove the test
        // has teeth.
        spec.master_limiter = None;
        controller.install_plan(&spec).unwrap();
        let mut hot = false;
        for _ in 0..20 {
            executor.render_block(&mut frames);
            hot |= frames.iter().any(|sample| sample.abs() > 1.0);
        }
        assert!(hot, "unlimited reference mix never exceeded 1.0");
    }

    #[test]
    fn set_stage_gain_retargets_without_recompile() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Fast path: no install, just a retarget; the smoothing ramp keeps
        // the transition step-free.
        controller.set_stage_gain(LANE_ID, 1.0).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        assert!(step < 0.08, "fast-path gain stepped audio by {step}");

        // Unknown stage: typed error, callers fall back to install.
        assert!(controller.set_stage_gain(999, 0.5).is_err());
    }

    #[test]
    fn gain_automation_follows_the_envelope_sample_accurately() {
        let (mut controller, mut executor) = render_plane();
        // Constant-amplitude source (DC-ish loopable samples) under a gain
        // ramp envelope 0.0 -> 1.0 over 9600 frames, then hold.
        let values = vec![0.5f32; 480];
        let mut spec = samples_spec(&values, 0, u64::MAX, true);
        spec.stages[0].gain_automation = Some(vec![(0, 0.0), (9_600, 1.0), (19_200, 0.25)]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Render 19_200 frames in 256-frame blocks; spot-check the envelope.
        let mut output = Vec::new();
        let mut frames = [0.0f32; 512];
        for _ in 0..75 {
            executor.render_block(&mut frames);
            output.extend(frames.chunks_exact(2).map(|frame| frame[0]));
        }
        // At frame 9_600 the gain is 1.0: sample value 0.5 * 1.0.
        let mid = output[9_600];
        assert!((mid - 0.5).abs() < 0.02, "envelope peak read {mid}");
        // At frame 14_400 (halfway down to 0.25): gain ≈ 0.625.
        let down = output[14_400];
        assert!(
            (down - 0.5 * 0.625).abs() < 0.02,
            "envelope descent read {down}"
        );
        // Monotonic rise across the first segment (block-ramped).
        assert!(output[2_000] < output[4_000] && output[4_000] < output[8_000]);
    }

    #[test]
    fn envelope_swap_mid_play_stays_continuous() {
        let (mut controller, mut executor) = render_plane();
        let values = vec![0.5f32; 480];
        let mut spec = samples_spec(&values, 0, u64::MAX, true);
        spec.stages[0].gain_automation = Some(vec![(0, 1.0)]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Swap to a very different envelope mid-play: the block ramp anchors
        // at the inherited smoothed gain, so no step.
        let mut louder = samples_spec(&values, 0, u64::MAX, true);
        louder.stages[0].gain_automation = Some(vec![(0, 0.1)]);
        controller.install_plan(&louder).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        assert!(step < 0.05, "envelope swap stepped audio by {step}");
    }

    #[test]
    fn gain_only_spec_diffs_take_the_fast_path() {
        let base = tone_spec(440.0);
        let mut louder = base.clone();
        louder.stages[0].gain = 0.9;
        assert_eq!(
            base.differs_only_in_gains(&louder),
            Some(vec![(LANE_ID, 0.9)])
        );
        // Structural change: no fast path.
        let mut reshaped = base.clone();
        reshaped.stages[0].gain_automation = Some(vec![(0, 1.0)]);
        assert_eq!(base.differs_only_in_gains(&reshaped), None);
        assert_eq!(base.differs_only_in_gains(&base), Some(vec![]));
    }

    #[test]
    fn mid_lane_clip_insert_preserves_neighbour_state() {
        // A tone clip keeps its phase when a new clip is inserted BEFORE it
        // in the lane's clip list — the clip-id inheritance map prevents the
        // zip-index cross-wiring the old code had.
        let (mut controller, mut executor) = render_plane();
        let survivor = RenderClipSpec {
            clip_id: 7,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone {
                frequency_hz: 440.0,
            },
            loop_source: false,
        };
        controller
            .install_plan(&lane_master_spec(0.5, vec![survivor.clone()]))
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Insert a silent clip at index 0; survivor moves to index 1.
        let inserted = RenderClipSpec {
            clip_id: 8,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Silence,
            loop_source: false,
        };
        controller
            .install_plan(&lane_master_spec(0.5, vec![inserted, survivor]))
            .unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        // Phase carried: the 440 Hz tone continues without a step.
        let step = max_left_step(&frames);
        assert!(step < 0.05, "clip insert stepped audio by {step}");
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn stage_reorder_preserves_state_through_the_identity_map() {
        // Two tone lanes swap positions in the stage list across a plan
        // swap; both keep phase and smoothed gain (no audible step).
        let (mut controller, mut executor) = render_plane();
        let lane_a = lane_node(10, 0.4, vec![tone_clip(330.0)]);
        let lane_b = lane_node(11, 0.4, vec![tone_clip(550.0)]);
        let master = master_node(vec![identity_edge(10), identity_edge(11)]);
        controller
            .install_plan(&RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                master_limiter: None,
                stages: vec![lane_a.clone(), lane_b.clone(), master.clone()],
            })
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        controller
            .install_plan(&RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                master_limiter: None,
                stages: vec![lane_b, lane_a, master],
            })
            .unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        // Two tones at 0.4 gain: their combined slope stays well under this
        // bound only if both phases carried.
        assert!(step < 0.07, "stage reorder stepped audio by {step}");
    }

    #[test]
    fn plan_churn_keeps_a_surviving_tone_continuous() {
        // Property-style: a seeded LCG drives 24 plan installs mid-play —
        // adding/removing extra lanes, inserting silent clips around the
        // survivor, jittering other lanes' gains. The surviving tone lane
        // must never step.
        let (mut controller, mut executor) = render_plane();
        let survivor_clip = RenderClipSpec {
            clip_id: 1,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone {
                frequency_hz: 440.0,
            },
            loop_source: false,
        };
        let mut seed: u64 = 0x5EED_CAFE;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let build = |extra_lanes: u32, clips_before: u32, extra_gain: f32| -> RenderPlanSpec {
            let mut clips = Vec::new();
            for index in 0..clips_before {
                clips.push(RenderClipSpec {
                    clip_id: 100 + index as u64,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Silence,
                    loop_source: false,
                });
            }
            clips.push(survivor_clip.clone());
            let mut stages = vec![lane_node(LANE_ID, 0.5, clips)];
            let mut edges = vec![identity_edge(LANE_ID)];
            for index in 0..extra_lanes {
                let stage_id = 50 + index as u64;
                // Extra lanes are silent so the survivor's continuity is the
                // only signal under test.
                stages.push(lane_node(stage_id, extra_gain, vec![]));
                edges.push(identity_edge(stage_id));
            }
            stages.push(master_node(edges));
            RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                master_limiter: None,
                stages,
            }
        };
        controller.install_plan(&build(0, 0, 0.3)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut worst_step = 0.0f32;
        let mut previous_tail = None::<f32>;
        for _ in 0..24 {
            let extra = next() % 4;
            let before = next() % 3;
            let gain = (next() % 100) as f32 / 100.0;
            controller
                .install_plan(&build(extra, before, gain))
                .unwrap();
            let mut frames = [0.0f32; 512];
            executor.render_block(&mut frames);
            if let Some(tail) = previous_tail {
                worst_step = worst_step.max((frames[0] - tail).abs());
            }
            worst_step = worst_step.max(max_left_step(&frames));
            previous_tail = Some(frames[frames.len() - 2]);
        }
        assert!(
            worst_step < 0.05,
            "plan churn stepped the surviving tone by {worst_step}",
        );
    }

    #[test]
    fn plan_swap_inherits_smoothed_gain_without_stepping() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Same plan with lane gain doubled: swap mid-play. Stage ids are
        // stable, so the smoothed gain carries over and ramps.
        let louder = lane_master_spec(1.0, vec![tone_clip(440.0)]);
        controller.install_plan(&louder).unwrap();

        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let max_step = frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max);
        // 440 Hz at 48k moves at most ~0.058/sample at unity; the gain ramp
        // must not add a visible step on top.
        assert!(max_step < 0.08, "gain swap produced a step of {max_step}");
    }

    #[test]
    fn rate_converted_clips_play_through_the_sinc_path() {
        // 1 kHz sine at 44.1k played on a 48k stream: after the edge ramp
        // and clip fade, output must track the analytic sine to ~60 dB
        // (linear interpolation fails this at ~35-40 dB).
        let (mut controller, mut executor) = render_plane();
        let source_rate = 44_100u32;
        let frequency = 1_000.0f64;
        let mut data = Vec::new();
        for n in 0..44_100 {
            let value =
                (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
            data.push(value);
            data.push(value);
        }
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1005,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Samples(RenderSampleBuffer::stereo(source_rate, data.into())),
                loop_source: false,
            }],
        );
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 4); // 1024 frames: ramp open, fades passed.

        let mut frames = vec![0.0f32; 2048];
        executor.render_block(&mut frames);
        let step = source_rate as f64 / 48_000.0;
        let mut error = 0.0f64;
        let mut power = 0.0f64;
        for frame_index in 0..1024usize {
            let stream_frame = 1024 + frame_index as u64;
            let position = stream_frame as f64 * step;
            let expected =
                (std::f64::consts::TAU * frequency * position / source_rate as f64).sin();
            let actual = frames[frame_index * 2] as f64;
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "rate-converted playback SNR {snr:.1} dB");
    }

    // ── Warped (rate-multiplied) sources (g12.027) ──────────────────────

    #[test]
    fn warped_samples_clip_plays_at_the_rate_multiplied_step() {
        // 440 Hz sine at the stream rate warped by 1.5: playback must track
        // the analytic sine advanced at 1.5 source frames per stream frame
        // (i.e. sound at 660 Hz), through the same sinc path as SRC.
        let (mut controller, mut executor) = render_plane();
        let rate = 1.5f64;
        let frequency = 440.0f64;
        let mut data = Vec::new();
        for n in 0..96_000 {
            let value = (std::f64::consts::TAU * frequency * n as f64 / 48_000.0).sin() as f32;
            data.push(value);
            data.push(value);
        }
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1006,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Warped {
                    source: Box::new(RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into()))),
                    rate,
                },
                loop_source: false,
            }],
        );
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 4);

        let mut frames = vec![0.0f32; 2048];
        executor.render_block(&mut frames);
        let mut error = 0.0f64;
        let mut power = 0.0f64;
        for frame_index in 0..1024usize {
            let stream_frame = 1024 + frame_index as u64;
            let position = stream_frame as f64 * rate;
            let expected = (std::f64::consts::TAU * frequency * position / 48_000.0).sin();
            let actual = frames[frame_index * 2] as f64;
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "warped playback SNR {snr:.1} dB");
    }

    #[test]
    fn warped_source_exhausts_early_at_faster_rates() {
        // A 1 s buffer at rate 2.0 runs out of source after 0.5 s of stream
        // time: the second half of the clip window renders silence.
        let (mut controller, mut executor) = render_plane();
        let data: Vec<f32> = std::iter::repeat([0.5f32, 0.5f32])
            .take(48_000)
            .flatten()
            .collect();
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1007,
                start_frames: 0,
                end_frames: 96_000,
                source: RenderSource::Warped {
                    source: Box::new(RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into()))),
                    rate: 2.0,
                },
                loop_source: false,
            }],
        );
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        // Render to just past the source-exhaustion point (24 000 stream
        // frames) and confirm content, then silence.
        let mut audible_at_20k = 0.0f32;
        let mut audible_at_30k = 0.0f32;
        let mut frames = vec![0.0f32; 512];
        for block in 0..128u64 {
            executor.render_block(&mut frames);
            let block_start = block * 256;
            if block_start == 19_968 {
                audible_at_20k = frames.iter().fold(0.0f32, |a, s| a.max(s.abs()));
            }
            if block_start == 29_952 {
                audible_at_30k = frames.iter().fold(0.0f32, |a, s| a.max(s.abs()));
            }
        }
        assert!(
            audible_at_20k > 0.1,
            "expected audible content before exhaustion, peak {audible_at_20k}"
        );
        assert!(
            audible_at_30k < 1.0e-3,
            "expected silence after source exhaustion, peak {audible_at_30k}"
        );
    }

    #[test]
    fn warped_source_over_non_media_rejects_at_compile() {
        let (mut controller, _executor) = render_plane();
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1008,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Warped {
                    source: Box::new(RenderSource::TestTone {
                        frequency_hz: 440.0,
                    }),
                    rate: 1.5,
                },
                loop_source: false,
            }],
        );
        let error = controller.install_plan(&spec).unwrap_err();
        let expected = RenderPlanCompileError::WarpedSourceUnsupported {
            stage_id: LANE_ID,
            clip_id: 1008,
        };
        assert!(
            error.message.contains(&expected.to_string()),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn warped_source_sanitizes_degenerate_rates_to_identity() {
        // NaN/zero/negative rates compile as 1.0: playback matches the
        // unwarped source sample for sample.
        for rate in [f64::NAN, 0.0, -2.0] {
            let (mut controller, mut executor) = render_plane();
            let data: Vec<f32> = (0..9_600)
                .flat_map(|n| {
                    let value = ((n as f32 * 0.13).sin()) * 0.5;
                    [value, value]
                })
                .collect();
            let buffer = RenderSampleBuffer::stereo(48_000, data.into());
            let warped = lane_master_spec(
                1.0,
                vec![RenderClipSpec {
                    clip_id: 1009,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Warped {
                        source: Box::new(RenderSource::Samples(buffer.clone())),
                        rate,
                    },
                    loop_source: false,
                }],
            );
            controller.install_plan(&warped).unwrap();
            controller.set_playing(true).unwrap();
            let mut warped_frames = vec![0.0f32; 2048];
            executor.render_block(&mut warped_frames);

            let (mut controller, mut executor) = render_plane();
            let plain = lane_master_spec(
                1.0,
                vec![RenderClipSpec {
                    clip_id: 1009,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Samples(buffer),
                    loop_source: false,
                }],
            );
            controller.install_plan(&plain).unwrap();
            controller.set_playing(true).unwrap();
            let mut plain_frames = vec![0.0f32; 2048];
            executor.render_block(&mut plain_frames);

            assert_eq!(warped_frames, plain_frames, "rate {rate}");
        }
    }

    // ── Plugin processors (g11.012) ─────────────────────────────────────

    /// Fake in-process backend: multiplies every sample by `gain` and counts
    /// calls. Stands in for a live plugin without any child process.
    struct FakeGainProcessor {
        gain: f32,
        calls: AtomicU64,
    }

    impl PluginBlockProcessor for FakeGainProcessor {
        fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
            self.calls.fetch_add(1, Ordering::Relaxed);
            for sample in &mut scratch[..frame_count * channels] {
                *sample *= self.gain;
            }
            true
        }
    }

    /// Fake backend that always misses: returns `false` and must leave the
    /// scratch untouched (the bypass contract under test).
    struct AlwaysMissProcessor {
        misses: AtomicU64,
    }

    impl PluginBlockProcessor for AlwaysMissProcessor {
        fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }

    /// Minimal alloc-free instrument backend: note-on starts a constant
    /// signal at the event velocity; note-off returns to silence.
    struct EventInstrumentProcessor {
        amplitude_bits: AtomicU32,
    }

    impl EventInstrumentProcessor {
        fn render(
            &self,
            scratch: &mut [f32],
            frame_count: usize,
            channels: usize,
            events: &[RenderBlockPluginEvent],
        ) -> bool {
            let mut amplitude = f32::from_bits(self.amplitude_bits.load(Ordering::Relaxed));
            let mut event_index = 0;
            for frame in 0..frame_count {
                while event_index < events.len()
                    && events[event_index].offset_frames as usize == frame
                {
                    amplitude = match events[event_index].kind {
                        RenderPluginEventKind::NoteOn { velocity, .. } => velocity,
                        RenderPluginEventKind::NoteOff { .. } => 0.0,
                        RenderPluginEventKind::ControlChange { .. }
                        | RenderPluginEventKind::PitchBend { .. }
                        | RenderPluginEventKind::ChannelPressure { .. }
                        | RenderPluginEventKind::NoteExpression { .. } => amplitude,
                    };
                    event_index += 1;
                }
                for channel in 0..channels {
                    scratch[frame * channels + channel] = amplitude;
                }
            }
            self.amplitude_bits
                .store(amplitude.to_bits(), Ordering::Relaxed);
            true
        }
    }

    impl PluginBlockProcessor for EventInstrumentProcessor {
        fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
            self.render(scratch, frame_count, channels, &[])
        }

        fn process_with_events(
            &self,
            scratch: &mut [f32],
            frame_count: usize,
            channels: usize,
            events: &[RenderBlockPluginEvent],
        ) -> bool {
            self.render(scratch, frame_count, channels, events)
        }
    }

    /// Constant-content plan with a Sum insert stage carrying `processor`.
    fn processor_spec(processor: Option<RenderPluginProcessor>) -> RenderPlanSpec {
        let values = vec![0.5f32; 480];
        let mut data = Vec::new();
        for value in &values {
            data.push(*value);
            data.push(*value);
        }
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(
                    LANE_ID,
                    1.0,
                    vec![RenderClipSpec {
                        clip_id: 2001,
                        start_frames: 0,
                        end_frames: u64::MAX,
                        source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                        loop_source: true,
                    }],
                ),
                RenderStageSpec {
                    processor,
                    events: None,
                    stage_id: 77,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(LANE_ID)],
                },
                master_node(vec![identity_edge(77)]),
            ],
        }
    }

    fn impulse_delay_spec(delay_frames: u32) -> RenderPlanSpec {
        let mut data = vec![0.0f32; 512 * 2];
        data[100 * 2] = 1.0;
        data[100 * 2 + 1] = 1.0;
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(
                    LANE_ID,
                    1.0,
                    vec![RenderClipSpec {
                        clip_id: 2002,
                        start_frames: 0,
                        end_frames: 512,
                        source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                        loop_source: false,
                    }],
                ),
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 78,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Delay {
                        frames: delay_frames,
                    },
                    inputs: vec![identity_edge(LANE_ID)],
                },
                master_node(vec![identity_edge(78)]),
            ],
        }
    }

    #[test]
    fn delay_stage_moves_an_impulse_by_exact_stream_frames() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&impulse_delay_spec(7)).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = [0.0f32; 256 * 2];
        executor.render_block(&mut frames);

        assert_eq!(frames[100 * 2], 0.0);
        assert!(frames[107 * 2] > 0.0);
        assert_eq!(frames[107 * 2], frames[107 * 2 + 1]);
        assert_eq!(frames[106 * 2], 0.0);
        assert_eq!(frames[108 * 2], 0.0);
    }

    #[test]
    fn delay_stage_carries_ring_state_across_plan_swap() {
        let (mut controller, mut executor) = render_plane();
        let spec = impulse_delay_spec(300);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        let mut first = [0.0f32; 256 * 2];
        executor.render_block(&mut first);
        assert!(first.iter().all(|sample| *sample == 0.0));

        controller.install_plan(&spec).unwrap();
        let mut second = [0.0f32; 256 * 2];
        executor.render_block(&mut second);
        assert_eq!(second[144 * 2], 1.0);
        assert_eq!(second[144 * 2 + 1], 1.0);
    }

    #[test]
    fn sum_stage_processor_transforms_the_summed_scratch() {
        let (mut controller, mut executor) = render_plane();
        let backend = Arc::new(FakeGainProcessor {
            gain: 0.5,
            calls: AtomicU64::new(0),
        });
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        controller
            .install_plan(&processor_spec(Some(handle)))
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 4);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Past the edge ramp and clip fade: 0.5 content × 0.5 plugin gain.
        assert!(
            (frames[100 * 2] - 0.25).abs() < 1e-5,
            "processed sample read {}",
            frames[100 * 2],
        );
        assert!(backend.calls.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn processor_miss_bypasses_and_counts_without_touching_scratch() {
        let (mut controller, mut executor) = render_plane();
        let backend = Arc::new(AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        });
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        controller
            .install_plan(&processor_spec(Some(handle)))
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 4);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Bypass: dry content flows through the insert untouched.
        assert!(
            (frames[100 * 2] - 0.5).abs() < 1e-5,
            "bypassed sample read {}",
            frames[100 * 2],
        );
        assert!(backend.misses.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn processor_absent_and_bypassed_render_identically() {
        let render = |processor: Option<RenderPluginProcessor>| -> Vec<f32> {
            let (mut controller, mut executor) = render_plane();
            controller.install_plan(&processor_spec(processor)).unwrap();
            controller.set_playing(true).unwrap();
            let mut collected = Vec::new();
            let mut frames = [0.0f32; 512];
            for _ in 0..8 {
                executor.render_block(&mut frames);
                collected.extend_from_slice(&frames);
            }
            collected
        };
        let dry = render(None);
        let bypassed = render(Some(RenderPluginProcessor::new(Arc::new(
            AlwaysMissProcessor {
                misses: AtomicU64::new(0),
            },
        ))));
        assert_eq!(dry, bypassed, "bypass must be bit-identical to absent");
    }

    #[test]
    fn compile_rejects_processors_on_non_sum_stages() {
        let handle = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        }));
        let mut spec = tone_spec(440.0);
        spec.stages[0].processor = Some(handle);
        let (mut controller, _executor) = render_plane();
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("not a Sum stage"), "{error}");
    }

    #[test]
    fn processor_swap_is_structural_not_a_gain_fast_path() {
        let handle_a = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        }));
        let handle_b = RenderPluginProcessor::new(Arc::new(AlwaysMissProcessor {
            misses: AtomicU64::new(0),
        }));
        let with_a = processor_spec(Some(handle_a.clone()));
        // Clone (same sample-buffer Arc) so only the processor may differ.
        let with_a_again = with_a.clone();
        let mut with_b = with_a.clone();
        with_b.stages[1].processor = Some(handle_b);
        let _ = handle_a;
        // Same handle: gain-only diff logic sees no change.
        assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
        // Different handle (same everything else): structural.
        assert_eq!(with_a.differs_only_in_gains(&with_b), None);
    }

    // ── Plugin event delivery (g12.034 follow-up) ───────────────────────

    /// Recording backend: captures the event slice of every
    /// `process_with_events` call. A bare `process` call records a sentinel
    /// (`offset_frames == u32::MAX`) — stages carrying an event buffer must
    /// never take that path.
    struct RecordingEventProcessor {
        calls: std::sync::Mutex<Vec<Vec<RenderBlockPluginEvent>>>,
    }

    impl RecordingEventProcessor {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                calls: std::sync::Mutex::new(Vec::new()),
            })
        }

        fn calls(&self) -> Vec<Vec<RenderBlockPluginEvent>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl PluginBlockProcessor for RecordingEventProcessor {
        fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
            self.calls
                .lock()
                .unwrap()
                .push(vec![RenderBlockPluginEvent {
                    offset_frames: u32::MAX,
                    channel: 0,
                    kind: RenderPluginEventKind::NoteOff { key: 0 },
                }]);
            true
        }

        fn process_with_events(
            &self,
            _scratch: &mut [f32],
            _frames: usize,
            _channels: usize,
            events: &[RenderBlockPluginEvent],
        ) -> bool {
            self.calls.lock().unwrap().push(events.to_vec());
            true
        }
    }

    fn event_buffer(events: Vec<RenderPluginEvent>) -> RenderPluginEventBuffer {
        RenderPluginEventBuffer {
            events: events.into(),
        }
    }

    /// `processor_spec` with an event stream on the insert stage.
    fn events_spec(
        handle: RenderPluginProcessor,
        events: RenderPluginEventBuffer,
    ) -> RenderPlanSpec {
        let mut spec = processor_spec(Some(handle));
        spec.stages[1].events = Some(events);
        spec
    }

    #[test]
    fn processor_stage_delivers_events_at_intra_block_sample_offsets() {
        let (mut controller, mut executor) = render_plane();
        let backend = RecordingEventProcessor::new();
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let buffer = event_buffer(vec![
            RenderPluginEvent {
                frame: 100,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 64,
                    velocity: 0.75,
                },
            },
            RenderPluginEvent {
                frame: 519,
                channel: 0,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.33,
                },
            },
            RenderPluginEvent {
                frame: 700,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ]);
        controller
            .install_plan(&events_spec(handle, buffer))
            .unwrap();
        controller.set_playing(true).unwrap();

        // Two 512-frame blocks from position 0.
        let mut frames = vec![0.0f32; 1024];
        executor.render_block(&mut frames);
        executor.render_block(&mut frames);

        let calls = backend.calls();
        assert_eq!(calls.len(), 2, "one delivery per rendered block");
        assert_eq!(
            calls[0],
            vec![RenderBlockPluginEvent {
                offset_frames: 100,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 64,
                    velocity: 0.75,
                },
            }],
            "block 1 carries the note-on at its absolute frame",
        );
        assert_eq!(
            calls[1],
            vec![
                RenderBlockPluginEvent {
                    offset_frames: 7,
                    channel: 0,
                    kind: RenderPluginEventKind::ControlChange {
                        controller: 74,
                        value: 0.33,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 188,
                    channel: 0,
                    kind: RenderPluginEventKind::NoteOff { key: 64 },
                },
            ],
            "block 2 events land at frame − block start",
        );
    }

    #[test]
    fn hosted_instrument_events_generate_audio_from_a_silent_lane() {
        let (mut controller, mut executor) = render_plane();
        let backend = Arc::new(EventInstrumentProcessor {
            amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
        });
        let handle = RenderPluginProcessor::new(backend as Arc<_>);
        let events = event_buffer(vec![
            RenderPluginEvent {
                frame: 64,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            },
            RenderPluginEvent {
                frame: 320,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 60 },
            },
        ]);
        let mut spec = events_spec(handle, events);
        let RenderStageKind::Source { clips } = &mut spec.stages[0].kind else {
            panic!("fixture lane source");
        };
        let RenderSource::Samples(samples) = &mut clips[0].source else {
            panic!("fixture sample source");
        };
        samples.frames = vec![0.0; samples.frames.len()].into();
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = vec![0.0f32; 512 * 2];
        executor.render_block(&mut frames);
        assert!(frames[..64 * 2].iter().all(|sample| *sample == 0.0));
        assert!(frames[96 * 2..256 * 2].iter().all(|sample| *sample > 0.0));
        assert!(frames[320 * 2..].iter().all(|sample| *sample == 0.0));
        assert!(controller.meters().iter().any(|(_, peak, _)| *peak > 0.0));

        let offline = crate::offline::render_plan_to_pcm(
            &spec,
            &crate::offline::OfflineRenderOptions {
                start_frame: 0,
                frame_count: 512,
                block_frames: 128,
                capture_stage_ids: Vec::new(),
            },
        )
        .expect("offline hosted instrument render");
        assert!(offline.master[..64 * 2].iter().all(|sample| *sample == 0.0));
        assert!(offline.master[96 * 2..256 * 2]
            .iter()
            .all(|sample| *sample > 0.0));
        assert!(offline.master[320 * 2..]
            .iter()
            .all(|sample| *sample == 0.0));
    }

    #[test]
    fn event_delivery_is_playback_gated() {
        let (mut controller, mut executor) = render_plane();
        let backend = RecordingEventProcessor::new();
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let buffer = event_buffer(vec![RenderPluginEvent {
            frame: 0,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 1.0,
            },
        }]);
        controller
            .install_plan(&events_spec(handle, buffer))
            .unwrap();
        controller.set_playing(true).unwrap();
        let mut frames = vec![0.0f32; 1024];
        executor.render_block(&mut frames);
        // Stop: the edge ramp keeps rendering blocks briefly, but the
        // position no longer advances — re-delivering the same events would
        // double-trigger notes, so delivery gates on playback.
        controller.set_playing(false).unwrap();
        executor.render_block(&mut frames);

        let calls = backend.calls();
        assert!(calls.len() >= 2, "ramp-out still processes audio");
        assert_eq!(calls[0].len(), 1, "playing block delivers");
        for call in &calls[1..] {
            assert!(call.is_empty(), "stopped blocks must deliver no events");
        }
    }

    #[test]
    fn seek_chases_held_plugin_note_controller_and_expression_state() {
        let (mut controller, mut executor) = render_plane();
        let backend = RecordingEventProcessor::new();
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let buffer = event_buffer(vec![
            RenderPluginEvent {
                frame: 50,
                channel: 2,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.25,
                },
            },
            RenderPluginEvent {
                frame: 50,
                channel: 2,
                kind: RenderPluginEventKind::PitchBend { value: 0.4 },
            },
            RenderPluginEvent {
                frame: 50,
                channel: 2,
                kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
            },
            RenderPluginEvent {
                frame: 100,
                channel: 2,
                kind: RenderPluginEventKind::NoteOn {
                    key: 64,
                    velocity: 0.75,
                },
            },
            RenderPluginEvent {
                frame: 150,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Pressure,
                    value: 0.7,
                },
            },
            RenderPluginEvent {
                frame: 150,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.8,
                },
            },
            RenderPluginEvent {
                frame: 150,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Tuning,
                    value: 12.0,
                },
            },
            RenderPluginEvent {
                frame: 500,
                channel: 2,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ]);
        controller
            .install_plan(&events_spec(handle, buffer))
            .unwrap();
        controller.seek(300).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = vec![0.0f32; 1024];
        executor.render_block(&mut frames);
        assert_eq!(
            backend.calls()[0],
            vec![
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::ControlChange {
                        controller: 74,
                        value: 0.25,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::PitchBend { value: 0.4 },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::NoteOn {
                        key: 64,
                        velocity: 0.75,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::NoteExpression {
                        key: 64,
                        expression: RenderNoteExpressionKind::Pressure,
                        value: 0.7,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::NoteExpression {
                        key: 64,
                        expression: RenderNoteExpressionKind::Timbre,
                        value: 0.8,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 0,
                    channel: 2,
                    kind: RenderPluginEventKind::NoteExpression {
                        key: 64,
                        expression: RenderNoteExpressionKind::Tuning,
                        value: 12.0,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 200,
                    channel: 2,
                    kind: RenderPluginEventKind::NoteOff { key: 64 },
                },
            ],
        );
    }

    #[test]
    fn loop_wrap_delivers_both_segments_with_buffer_relative_offsets() {
        let (mut controller, mut executor) = render_plane();
        let backend = RecordingEventProcessor::new();
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let buffer = event_buffer(vec![
            RenderPluginEvent {
                frame: 100,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            },
            RenderPluginEvent {
                frame: 550,
                channel: 1,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 1,
                    value: 1.0,
                },
            },
        ]);
        controller
            .install_plan(&events_spec(handle, buffer))
            .unwrap();
        controller.set_loop_region(Some((0, 600))).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = vec![0.0f32; 1024];
        executor.render_block(&mut frames); // [0, 512): note at 100
        executor.render_block(&mut frames); // [512, 600) + wrap [0, 424)

        let calls = backend.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].len(), 1);
        assert_eq!(calls[0][0].offset_frames, 100);
        assert_eq!(
            calls[1],
            vec![
                RenderBlockPluginEvent {
                    offset_frames: 38, // 550 − 512, first segment
                    channel: 1,
                    kind: RenderPluginEventKind::ControlChange {
                        controller: 1,
                        value: 1.0,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 88, // wrap: release note active at loop end
                    channel: 0,
                    kind: RenderPluginEventKind::NoteOff { key: 60 },
                },
                RenderBlockPluginEvent {
                    offset_frames: 188, // 88 wrap offset + frame 100
                    channel: 0,
                    kind: RenderPluginEventKind::NoteOn {
                        key: 60,
                        velocity: 0.5,
                    },
                },
            ],
            "wrapped block delivers both segments, buffer-relative",
        );
    }

    #[test]
    fn loop_wrap_chases_held_note_controller_and_expression_at_wrap_offset() {
        let (mut controller, mut executor) = render_plane();
        let backend = RecordingEventProcessor::new();
        let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
        let buffer = event_buffer(vec![
            RenderPluginEvent {
                frame: 50,
                channel: 3,
                kind: RenderPluginEventKind::PitchBend { value: -0.25 },
            },
            RenderPluginEvent {
                frame: 100,
                channel: 3,
                kind: RenderPluginEventKind::NoteOn {
                    key: 67,
                    velocity: 0.9,
                },
            },
            RenderPluginEvent {
                frame: 150,
                channel: 3,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 67,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.45,
                },
            },
        ]);
        controller
            .install_plan(&events_spec(handle, buffer))
            .unwrap();
        controller.set_loop_region(Some((300, 600))).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = vec![0.0f32; 1024];
        executor.render_block(&mut frames); // [0, 512)
        executor.render_block(&mut frames); // [512, 600) + wrap [300, 724)

        assert_eq!(
            backend.calls()[1],
            vec![
                RenderBlockPluginEvent {
                    offset_frames: 88,
                    channel: 3,
                    kind: RenderPluginEventKind::NoteOff { key: 67 },
                },
                RenderBlockPluginEvent {
                    offset_frames: 88,
                    channel: 3,
                    kind: RenderPluginEventKind::PitchBend { value: -0.25 },
                },
                RenderBlockPluginEvent {
                    offset_frames: 88,
                    channel: 3,
                    kind: RenderPluginEventKind::NoteOn {
                        key: 67,
                        velocity: 0.9,
                    },
                },
                RenderBlockPluginEvent {
                    offset_frames: 88,
                    channel: 3,
                    kind: RenderPluginEventKind::NoteExpression {
                        key: 67,
                        expression: RenderNoteExpressionKind::Timbre,
                        value: 0.45,
                    },
                },
            ],
        );
    }

    #[test]
    fn compile_rejects_events_without_processor_and_unsorted_events() {
        let buffer = event_buffer(vec![RenderPluginEvent {
            frame: 0,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        }]);
        let mut spec = processor_spec(None);
        spec.stages[1].events = Some(buffer);
        let (mut controller, _executor) = render_plane();
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(
            error.message.contains("without a plugin processor"),
            "{error}"
        );

        let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
        let unsorted = event_buffer(vec![
            RenderPluginEvent {
                frame: 10,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 0 },
            },
            RenderPluginEvent {
                frame: 5,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 0 },
            },
        ]);
        let spec = events_spec(handle, unsorted);
        let (mut controller, _executor) = render_plane();
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("not sorted by frame"), "{error}");
    }

    #[test]
    fn event_buffer_swap_is_structural_not_a_gain_fast_path() {
        let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
        let event = RenderPluginEvent {
            frame: 0,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        };
        let with_a = events_spec(handle, event_buffer(vec![event]));
        // Clone shares the Arc: gain-only diff logic sees no change.
        let with_a_again = with_a.clone();
        assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
        // A rebuilt buffer (same content, new Arc) is structural.
        let mut with_b = with_a.clone();
        with_b.stages[1].events = Some(event_buffer(vec![event]));
        assert_eq!(with_a.differs_only_in_gains(&with_b), None);
    }

    #[test]
    fn sample_buffers_compare_by_pointer_for_cheap_spec_equality() {
        let data: Arc<[f32]> = vec![0.0f32; 8].into();
        let a = RenderSampleBuffer::stereo(48_000, Arc::clone(&data));
        let b = RenderSampleBuffer::stereo(48_000, data);
        let c = RenderSampleBuffer::stereo(48_000, vec![0.0f32; 8].into());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ── Disk-streaming sources ──────────────────────────────────────────────

    /// Spec with one stream clip windowed `[start, end)` at lane gain 1.
    fn stream_spec(
        handle: &RenderStreamHandle,
        start_frames: u64,
        end_frames: u64,
    ) -> RenderPlanSpec {
        lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1006,
                start_frames,
                end_frames,
                source: RenderSource::Stream(handle.clone()),
                loop_source: false,
            }],
        )
    }

    /// Feed `[from, to)` of a ramp (value = frame / total) in fixed chunks.
    fn feed_ramp(feeder: &StreamFeeder, total: u64, from: u64, to: u64, chunk_frames: u64) {
        let mut start = from - from % chunk_frames;
        while start < to.min(total) {
            let count = chunk_frames.min(total - start);
            let mut data = Vec::with_capacity(count as usize * 2);
            for frame in start..start + count {
                let value = frame as f32 / total as f32;
                data.push(value);
                data.push(value);
            }
            if feeder
                .try_send_chunk(StreamChunk {
                    start_frame: start,
                    frames: data.into(),
                })
                .is_err()
            {
                return; // Mailbox full: enough read-ahead for the test.
            }
            start += count;
        }
    }

    #[test]
    fn stream_clips_play_fed_chunks_sample_accurately() {
        let (mut controller, mut executor) = render_plane();
        let total = 4_096u64;
        let (feeder, handle) = render_stream(48_000, total);
        // Window starts at frame 512, well past the edge ramp warm-up.
        let spec = stream_spec(&handle, 512, 512 + total);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        feed_ramp(&feeder, total, 0, 1_024, 256);

        // Two 256-frame blocks open the edge ramp and reach frame 512.
        warm_up(&mut executor, 2);
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 512+128 plays source frame 128, past the clip edge fade.
        let index = 128usize;
        let expected = 128.0 / total as f32;
        assert!((frames[index * 2] - expected).abs() < 1e-6);
        // 1:1 streaming: identical channels, zero underruns.
        assert_eq!(frames[index * 2], frames[index * 2 + 1]);
        assert_eq!(handle.underrun_frames(), 0);
        // The next block starts past the clip anchor: the read hint follows.
        executor.render_block(&mut frames);
        assert_eq!(feeder.wanted_frame(), 256);
    }

    #[test]
    fn stream_underruns_render_silence_and_count() {
        let (mut controller, mut executor) = render_plane();
        let (feeder, handle) = render_stream(48_000, 48_000);
        controller
            .install_plan(&stream_spec(&handle, 0, 48_000))
            .unwrap();
        controller.set_playing(true).unwrap();

        // Nothing fed: every in-window frame is an underrun, output silent.
        let mut frames = [0.1f32; 512];
        executor.render_block(&mut frames);
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(handle.underrun_frames(), 512);

        // Feed the region the executor wants: audio resumes, count holds.
        feed_ramp(
            &feeder,
            48_000,
            feeder.wanted_frame(),
            feeder.wanted_frame() + 2_048,
            512,
        );
        let before = handle.underrun_frames();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.001));
        assert_eq!(handle.underrun_frames(), before);
    }

    #[test]
    fn stream_seek_retires_stale_chunks_and_resumes_once_fed() {
        let (mut controller, mut executor) = render_plane();
        let total = 1_000_000u64;
        let (feeder, handle) = render_stream(48_000, total);
        controller
            .install_plan(&stream_spec(&handle, 0, total))
            .unwrap();
        controller.set_playing(true).unwrap();
        feed_ramp(&feeder, total, 0, 1_024, 256);
        warm_up(&mut executor, 3);
        assert_eq!(handle.underrun_frames(), 0);

        // Seek far past the retire lookahead: held chunks for the old
        // region must come back via the retired mailbox.
        let target = 600_000u64;
        controller.seek(target).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames); // Ramp-out block; seek lands.
        executor.render_block(&mut frames); // First block at the new region.
        assert!(feeder.wanted_frame() >= target);
        // Old-region chunks retire within a few blocks (stale arrivals can
        // sit one block in a held slot first).
        let mut retired = Vec::new();
        for _ in 0..4 {
            retired.extend(feeder.collect_retired());
            executor.render_block(&mut frames);
        }
        retired.extend(feeder.collect_retired());
        assert!(
            retired.iter().all(|chunk| chunk.start_frame < 1_024),
            "only old-region chunks should retire",
        );
        assert!(!retired.is_empty(), "stale chunks should have retired");

        // Feed the new region: playback resumes with the right content.
        let wanted = feeder.wanted_frame();
        feed_ramp(&feeder, total, wanted, wanted + 4_096, 512);
        let before = handle.underrun_frames();
        assert!(before > 0, "seek without data should have underrun");
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert_eq!(handle.underrun_frames(), before);
        let position = controller.position_frames() - 256;
        let expected = position as f32 / total as f32;
        assert!(
            (frames[0] - expected).abs() < 1e-5,
            "resumed at the wrong content"
        );
    }

    #[test]
    fn rate_converted_streams_play_through_the_sinc_path() {
        // 1 kHz sine at 44.1k streamed onto a 48k plan: same SNR bar as the
        // in-memory rate-converted test — proof the stream path shares the
        // polyphase interpolation.
        let (mut controller, mut executor) = render_plane();
        let source_rate = 44_100u32;
        let total = 44_100u64;
        let frequency = 1_000.0f64;
        let (feeder, handle) = render_stream(source_rate, total);
        controller
            .install_plan(&stream_spec(&handle, 0, u64::MAX))
            .unwrap();
        controller.set_playing(true).unwrap();
        // Feed the whole second up front in 8 large chunks.
        let chunk_frames = total.div_ceil(8);
        let mut start = 0u64;
        while start < total {
            let count = chunk_frames.min(total - start);
            let mut data = Vec::with_capacity(count as usize * 2);
            for n in start..start + count {
                let value = (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64)
                    .sin() as f32;
                data.push(value);
                data.push(value);
            }
            feeder
                .try_send_chunk(StreamChunk {
                    start_frame: start,
                    frames: data.into(),
                })
                .unwrap();
            start += count;
        }
        warm_up(&mut executor, 4); // 1024 frames: ramp open, fades passed.

        let mut frames = vec![0.0f32; 2048];
        executor.render_block(&mut frames);
        let step = source_rate as f64 / 48_000.0;
        let mut error = 0.0f64;
        let mut power = 0.0f64;
        for frame_index in 0..1024usize {
            let stream_frame = 1024 + frame_index as u64;
            let position = stream_frame as f64 * step;
            let expected =
                (std::f64::consts::TAU * frequency * position / source_rate as f64).sin();
            let actual = frames[frame_index * 2] as f64;
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "rate-converted stream SNR {snr:.1} dB");
        assert_eq!(handle.underrun_frames(), 0);
    }

    #[test]
    fn plan_swap_mid_stream_keeps_held_chunks_without_underrun() {
        let (mut controller, mut executor) = render_plane();
        let total = 48_000u64;
        let (feeder, handle) = render_stream(48_000, total);
        let spec = stream_spec(&handle, 0, total);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        // Feed only what fits in the mailbox + held slots; after the swap no
        // further feeding happens, so continuity proves the held chunks
        // moved across the plan boundary via the clip inheritance map.
        feed_ramp(&feeder, total, 0, 2_048, 256);
        warm_up(&mut executor, 2); // 512 frames consumed, chunks held.

        // Identity recompile mid-stream (the handle is pointer-equal, so
        // the spec is too — hosts would skip this install; force it).
        controller.install_plan(&spec.clone()).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert_eq!(handle.underrun_frames(), 0, "swap dropped held chunks");
        // Content continues exactly: frame 512 plays source frame 512.
        let expected = 512.0 / total as f32;
        assert!((frames[0] - expected).abs() < 1e-6);
    }

    // ── Live input monitor sources ──────────────────────────────────────────

    /// Spec with one live-input clip windowed `[0, u64::MAX)` at `lane_gain`.
    fn live_input_spec(handle: &RenderLiveInputHandle, lane_gain: f32) -> RenderPlanSpec {
        lane_master_spec(
            lane_gain,
            vec![RenderClipSpec {
                clip_id: 1007,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::LiveInput(handle.clone()),
                loop_source: false,
            }],
        )
    }

    /// Push `count` stereo frames of a ramp starting at `value_base`
    /// (value = (value_base + i) / 10_000) and return the next base.
    fn push_ramp(feeder: &LiveInputFeeder, value_base: u64, count: usize) -> u64 {
        let mut data = Vec::with_capacity(count * 2);
        for index in 0..count {
            let value = (value_base + index as u64) as f32 / 10_000.0;
            data.push(value);
            data.push(value);
        }
        assert_eq!(feeder.push_slice(&data), count, "test ring overflowed");
        value_base + count as u64
    }

    #[test]
    fn live_input_clips_render_pushed_audio_through_the_chain() {
        let (mut controller, mut executor) = render_plane();
        let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
        // Lane gain 0.5: the chain's fader applies to monitored input like
        // any other source.
        controller
            .install_plan(&live_input_spec(&handle, 0.5))
            .unwrap();
        controller.set_playing(true).unwrap();

        // Feed exactly one block per render so content is deterministic.
        let mut base = 0u64;
        let mut frames = [0.0f32; 512];
        // Warm-up: edge ramp opens, clip edge fade passes.
        for _ in 0..2 {
            base = push_ramp(&feeder, base, 256);
            executor.render_block(&mut frames);
        }
        assert_eq!(handle.underrun_frames(), 0);

        base = push_ramp(&feeder, base, 256);
        executor.render_block(&mut frames);
        // Block 2 renders pushed frames 512..768 at lane gain 0.5.
        for index in [0usize, 100, 255] {
            let expected = (512 + index) as f32 / 10_000.0 * 0.5;
            assert!(
                (frames[index * 2] - expected).abs() < 1e-6,
                "frame {index}: {} vs {expected}",
                frames[index * 2],
            );
            // Stereo feed: identical channels.
            assert_eq!(frames[index * 2], frames[index * 2 + 1]);
        }
        assert_eq!(handle.underrun_frames(), 0);
        let _ = base;
    }

    #[test]
    fn live_input_underruns_render_silence_and_count() {
        let (mut controller, mut executor) = render_plane();
        let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
        controller
            .install_plan(&live_input_spec(&handle, 1.0))
            .unwrap();
        controller.set_playing(true).unwrap();

        // Nothing fed: the whole block underruns, output stays silent.
        let mut frames = [0.1f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(handle.underrun_frames(), 256);

        // Half a block fed: 128 frames render, 128 count as underrun.
        push_ramp(&feeder, 50_000, 128);
        executor.render_block(&mut frames);
        assert_eq!(handle.underrun_frames(), 256 + 128);
        assert!(frames.iter().any(|sample| sample.abs() > 0.001));

        // Fed again: audio resumes, the count holds.
        push_ramp(&feeder, 50_128, 256);
        executor.render_block(&mut frames);
        assert_eq!(handle.underrun_frames(), 256 + 128);
    }

    #[test]
    fn live_input_survives_plan_swaps_without_dropping_audio() {
        let (mut controller, mut executor) = render_plane();
        let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
        let spec = live_input_spec(&handle, 1.0);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        let mut base = 0u64;
        let mut frames = [0.0f32; 512];
        for _ in 0..2 {
            base = push_ramp(&feeder, base, 256);
            executor.render_block(&mut frames);
        }

        // Recompile mid-feed (identity spec — pointer-equal handle keeps the
        // spec equal too; force the install). The ring lives in the shared
        // handle, so the feed continues without a gap or reset.
        controller.install_plan(&spec.clone()).unwrap();
        base = push_ramp(&feeder, base, 256);
        executor.render_block(&mut frames);
        assert_eq!(handle.underrun_frames(), 0, "swap dropped live audio");
        // Content continues exactly: first frame plays pushed frame 512.
        let expected = 512.0 / 10_000.0;
        assert!((frames[0] - expected).abs() < 1e-6);
        let _ = base;
    }

    #[test]
    fn live_input_trims_stale_backlog_to_bound_monitoring_latency() {
        let (mut controller, mut executor) = render_plane();
        let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
        controller
            .install_plan(&live_input_spec(&handle, 1.0))
            .unwrap();

        // Feeder ran while the executor was stopped: a deep stale backlog.
        let mut base = 0u64;
        for _ in 0..16 {
            base = push_ramp(&feeder, base, 256); // 4_096 frames total.
        }
        controller.set_playing(true).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // The executor discarded down to span + LIVE_INPUT_MAX_BACKLOG and
        // rendered from there — near the END of the pushed data, not frame
        // 0. Sample index 250 sits past both the transport edge ramp
        // (240 frames at 48 kHz) and the clip edge fade, so the raw pushed
        // value reads back unscaled.
        let value = f64::from(frames[250 * 2]) * 10_000.0;
        let cutoff = (4_096 - 256 - LIVE_INPUT_MAX_BACKLOG_FRAMES) as f64;
        assert!(
            value >= cutoff,
            "stale backlog replayed pushed frame {value} (cutoff {cutoff})",
        );
        // What remains buffered is bounded (latency stays shallow).
        assert!(
            handle.buffered_frames() <= LIVE_INPUT_MAX_BACKLOG_FRAMES,
            "backlog left at {} frames",
            handle.buffered_frames(),
        );
        let _ = base;
    }

    #[test]
    fn live_input_handles_compare_by_pointer_for_cheap_spec_equality() {
        let (_feeder_a, a) = render_live_input(1_024);
        let b = a.clone();
        let (_feeder_c, c) = render_live_input(1_024);
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Spec equality follows handle equality: idempotent recompiles.
        assert_eq!(live_input_spec(&a, 1.0), live_input_spec(&b, 1.0));
        assert_ne!(live_input_spec(&a, 1.0), live_input_spec(&c, 1.0));
    }

    #[test]
    fn stream_handles_compare_by_pointer_for_cheap_spec_equality() {
        let (_feeder_a, a) = render_stream(48_000, 1_000);
        let b = a.clone();
        let (_feeder_c, c) = render_stream(48_000, 1_000);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.source_sample_rate_hz(), 48_000);
        assert_eq!(a.total_frames(), 1_000);
    }

    // ── Note sources (built-in instrument) ─────────────────────────────────

    fn note(start_frame: u64, duration_frames: u64, degree: i32, velocity: f32) -> RenderNote {
        RenderNote {
            start_frame,
            duration_frames,
            degree,
            pitch_intent: None,
            velocity,
        }
    }

    #[test]
    fn degree_frequency_derivation_is_bit_identical_to_the_u8_pitch_formula() {
        // g12.034 widening compatibility pin: a degree with no pitch intent
        // must derive the EXACT bits the pre-widening
        // `440 * 2^((pitch - 69) / 12)` path produced for every u8 pitch.
        for pitch in 0u8..=127 {
            let old = 440.0 * f64::powf(2.0, (f64::from(pitch) - 69.0) / 12.0);
            let new = note(0, 1, i32::from(pitch), 1.0).frequency_hz();
            assert_eq!(old.to_bits(), new.to_bits(), "diverged at pitch {pitch}");
        }
    }

    #[test]
    fn pitch_intent_overrides_or_offsets_the_degree_frequency() {
        let mut absolute = note(0, 1, 69, 1.0);
        absolute.pitch_intent = Some(RenderPitchIntent::FrequencyHz(432.0));
        assert_eq!(absolute.frequency_hz(), 432.0);

        let mut offset = note(0, 1, 69, 1.0);
        offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(1200.0));
        assert!((offset.frequency_hz() - 880.0).abs() < 1e-9);

        let mut zero_offset = note(0, 1, 69, 1.0);
        zero_offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(0.0));
        assert_eq!(zero_offset.frequency_hz(), 440.0);
    }

    fn note_buffer(notes: Vec<RenderNote>) -> RenderNoteBuffer {
        RenderNoteBuffer {
            notes: notes.into(),
        }
    }

    /// Spec with one notes clip windowed `[start, end)` at lane gain 1.
    fn notes_spec(buffer: &RenderNoteBuffer, start_frames: u64, end_frames: u64) -> RenderPlanSpec {
        lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1008,
                start_frames,
                end_frames,
                source: RenderSource::Notes(buffer.clone()),
                loop_source: false,
            }],
        )
    }

    /// Offline-render `frame_count` frames of `spec` from `start_frame` and
    /// return the LEFT channel (channels are identical for note sources).
    fn render_notes_left(spec: &RenderPlanSpec, start_frame: u64, frame_count: u64) -> Vec<f32> {
        let output = crate::render_plan_to_pcm(
            spec,
            &crate::OfflineRenderOptions {
                start_frame,
                frame_count,
                ..crate::OfflineRenderOptions::default()
            },
        )
        .expect("offline note render");
        output
            .master
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect()
    }

    #[test]
    fn note_clips_render_at_the_note_frequency() {
        // A4 (pitch 69) sustained for one second: past the attack and clip
        // edge fade, the output must be a 440 Hz sine at the velocity
        // amplitude. Quadrature projection at 440 Hz recovers the amplitude
        // and the residual bounds everything that is not that sine.
        let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
        let spec = notes_spec(&buffer, 0, u64::MAX);
        let left = render_notes_left(&spec, 0, 24_000);

        let start = 4_800usize; // Past attack (144 frames) and edge fade.
        let count = 14_400usize; // Whole periods of 440 at 48k every 300.
        let mut in_phase = 0.0f64;
        let mut quadrature = 0.0f64;
        for index in 0..count {
            let n = (start + index) as f64;
            let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
            let sample = f64::from(left[start + index]);
            in_phase += sample * angle.sin();
            quadrature += sample * angle.cos();
        }
        let amplitude = 2.0 * (in_phase * in_phase + quadrature * quadrature).sqrt() / count as f64;
        assert!(
            (amplitude - 1.0).abs() < 0.01,
            "440 Hz amplitude read {amplitude}",
        );
        // Residual after removing the projected 440 Hz component: > 60 dB
        // below the tone (proof the output is that sine, not something else).
        let sine_gain = 2.0 * in_phase / count as f64;
        let cosine_gain = 2.0 * quadrature / count as f64;
        let mut error = 0.0f64;
        let mut power = 0.0f64;
        for index in 0..count {
            let n = (start + index) as f64;
            let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
            let expected = sine_gain * angle.sin() + cosine_gain * angle.cos();
            let actual = f64::from(left[start + index]);
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "note tone SNR {snr:.1} dB");
    }

    #[test]
    fn note_envelope_is_silent_before_attacks_and_releases() {
        // Note at frame 4_800, 4_800 frames long: silence before the start,
        // ramping attack (3 ms = 144 frames), full velocity through the
        // sustain, and a 40 ms release tail that ends in exact silence.
        let buffer = note_buffer(vec![note(4_800, 4_800, 69, 1.0)]);
        let spec = notes_spec(&buffer, 0, u64::MAX);
        let left = render_notes_left(&spec, 0, 24_000);

        assert!(
            left[..4_800].iter().all(|sample| *sample == 0.0),
            "audio before the note start",
        );
        let peak = |range: std::ops::Range<usize>| {
            left[range].iter().fold(0.0f32, |max, s| max.max(s.abs()))
        };
        // Attack: the first 72 frames stay under the half-ramped level.
        assert!(peak(4_800..4_872) < 0.55, "attack did not ramp");
        // Sustain: full velocity once the attack completes.
        let sustain_peak = peak(5_200..9_000);
        assert!(
            (sustain_peak - 1.0).abs() < 0.05,
            "sustain peak {sustain_peak}",
        );
        // Release: decaying after the note end...
        let release_start = 4_800 + 4_800;
        let release_end = release_start + 1_920; // 40 ms at 48 kHz.
        assert!(peak(release_start + 960..release_end) < 0.6, "release flat");
        // ...and exactly silent once the tail ends.
        assert!(
            left[release_end + 1..].iter().all(|sample| *sample == 0.0),
            "audio after the release tail",
        );
    }

    #[test]
    fn chords_render_as_the_sum_of_their_notes() {
        let pitches = [60i32, 64, 67];
        let chord = note_buffer(pitches.iter().map(|p| note(0, 24_000, *p, 0.3)).collect());
        let chord_left = render_notes_left(&notes_spec(&chord, 0, u64::MAX), 0, 12_000);

        let mut summed = vec![0.0f32; 12_000];
        for pitch in pitches {
            let single = note_buffer(vec![note(0, 24_000, pitch, 0.3)]);
            let left = render_notes_left(&notes_spec(&single, 0, u64::MAX), 0, 12_000);
            for (accumulator, sample) in summed.iter_mut().zip(left.iter()) {
                *accumulator += *sample;
            }
        }
        for (index, (chord_sample, sum_sample)) in chord_left.iter().zip(summed.iter()).enumerate()
        {
            assert!(
                (chord_sample - sum_sample).abs() < 1e-5,
                "chord diverged from note sum at {index}: {chord_sample} vs {sum_sample}",
            );
        }
    }

    #[test]
    fn seeking_into_a_sustained_note_reproduces_played_through_samples() {
        // Statelessness proof: rendering from a mid-note seek must produce
        // the SAME bits as playing through from zero — there is no voice
        // state whose absence a seek could expose.
        let buffer = note_buffer(vec![
            note(0, 48_000, 57, 0.8),
            note(6_000, 30_000, 64, 0.6),
            note(12_000, 12_000, 72, 1.0),
        ]);
        let spec = notes_spec(&buffer, 0, u64::MAX);
        let played_through = render_notes_left(&spec, 0, 48_000);
        let seeked = render_notes_left(&spec, 24_000, 4_800);
        for (index, (seek_sample, through_sample)) in seeked
            .iter()
            .zip(played_through[24_000..24_000 + 4_800].iter())
            .enumerate()
        {
            assert_eq!(
                seek_sample.to_bits(),
                through_sample.to_bits(),
                "seek diverged from play-through at offset {index}",
            );
        }
    }

    #[test]
    fn note_polyphony_caps_at_the_limit_keeping_earliest_started() {
        // 33 simultaneous notes: the render must equal the first 32 alone
        // (the 33rd — latest in sorted order — is skipped), and dropping to
        // 31 must change the output (the cap has teeth).
        let make = |count: usize| {
            note_buffer(
                (0..count)
                    .map(|index| note(index as u64, 24_000, 40 + index as i32, 0.02))
                    .collect(),
            )
        };
        let render =
            |count: usize| render_notes_left(&notes_spec(&make(count), 0, u64::MAX), 0, 12_000);
        let with_33 = render(33);
        let with_32 = render(32);
        let with_31 = render(31);
        assert_eq!(
            with_33
                .iter()
                .zip(with_32.iter())
                .filter(|(a, b)| a.to_bits() != b.to_bits())
                .count(),
            0,
            "the 33rd simultaneous note leaked past the polyphony cap",
        );
        assert!(
            with_32
                .iter()
                .zip(with_31.iter())
                .any(|(a, b)| a.to_bits() != b.to_bits()),
            "32nd note should be audible (cap test has no teeth)",
        );
    }

    #[test]
    fn unsorted_note_buffers_are_rejected_at_compile() {
        let (mut controller, _executor) = render_plane();
        let buffer = note_buffer(vec![note(1_000, 100, 60, 1.0), note(0, 100, 62, 1.0)]);
        let error = controller
            .install_plan(&notes_spec(&buffer, 0, u64::MAX))
            .unwrap_err();
        assert!(error.message.contains("sorted"), "{}", error.message);
    }

    #[test]
    fn note_buffers_compare_by_pointer_for_cheap_spec_equality() {
        let notes: Arc<[RenderNote]> = vec![note(0, 100, 60, 1.0)].into();
        let a = RenderNoteBuffer {
            notes: Arc::clone(&notes),
        };
        let b = RenderNoteBuffer { notes };
        let c = note_buffer(vec![note(0, 100, 60, 1.0)]);
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Spec equality follows buffer equality: idempotent recompiles.
        assert_eq!(notes_spec(&a, 0, 100), notes_spec(&b, 0, 100));
        assert_ne!(notes_spec(&a, 0, 100), notes_spec(&c, 0, 100));
    }

    #[test]
    fn note_clip_windows_gate_notes_on_the_stream_clock() {
        // Clip windowed [1_000, 2_000): a clip-relative note at 0 sounds at
        // stream frame 1_000, and nothing sounds past the window end even
        // though the note's tail extends beyond it.
        let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
        let spec = notes_spec(&buffer, 1_000, 2_000);
        let left = render_notes_left(&spec, 0, 4_000);
        assert!(left[..1_000].iter().all(|sample| *sample == 0.0));
        assert!(
            left[1_100..1_900].iter().any(|sample| sample.abs() > 0.5),
            "windowed note inaudible",
        );
        assert!(left[2_000..].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn retired_plans_return_to_the_control_side() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        let mut frames = [0.0f32; 64];
        executor.render_block(&mut frames);

        controller.install_plan(&tone_spec(880.0)).unwrap();
        executor.render_block(&mut frames);

        assert_eq!(controller.collect_retired(), 1);
        assert_eq!(controller.retired_parked_blocks(), 0);
    }

    // ── Graph-shaped plans ──────────────────────────────────────────────────

    #[test]
    fn compile_rejects_cycles() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 1,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(2)],
                },
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 2,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(1)],
                },
                master_node(vec![identity_edge(1)]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("cycle"), "{}", error.message);
    }

    #[test]
    fn compile_rejects_duplicate_node_ids() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(7, 1.0, vec![]),
                lane_node(7, 1.0, vec![]),
                master_node(vec![identity_edge(7)]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("duplicate"), "{}", error.message);
    }

    #[test]
    fn compile_rejects_wrong_master_count() {
        let (mut controller, _executor) = render_plane();
        let no_master = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![lane_node(1, 1.0, vec![])],
        };
        let error = controller.install_plan(&no_master).unwrap_err();
        assert!(
            error.message.contains("exactly one output stage"),
            "{}",
            error.message
        );

        let mut two_masters = master_node(vec![]);
        two_masters.stage_id = MASTER_ID + 1;
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![master_node(vec![]), two_masters],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(
            error.message.contains("exactly one output stage"),
            "{}",
            error.message
        );
    }

    #[test]
    fn compile_rejects_unknown_inputs_and_bad_matrices() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![master_node(vec![identity_edge(99)])],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("unknown input"), "{}", error.message);

        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(1, 1.0, vec![]),
                master_node(vec![RenderEdgeSpec {
                    source_stage_id: 1,
                    gain: 1.0,
                    matrix: Some(vec![1.0, 0.0, 0.0]), // 2x2 edge needs 4.
                }]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(
            error.message.contains("matrix") && error.message.contains("expected 4"),
            "{}",
            error.message
        );
    }

    #[test]
    fn bus_chain_renders_in_topological_order() {
        // Stages listed deliberately out of order: output first, then the
        // Sum chain, then the source. The schedule must still run source →
        // sum A → sum B → output, with each stage's gain applied at
        // consumption.
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                master_node(vec![identity_edge(20)]),
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 20,
                    format: ChannelFormat::stereo(),
                    gain: 0.5,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(10)],
                },
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 10,
                    format: ChannelFormat::stereo(),
                    gain: 0.5,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(LANE_ID)],
                },
                lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Reference: the same tone through a single lane at the chain's
        // composite gain (0.5 × 0.5 = 0.25).
        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut chained = [0.0f32; 512];
        let mut reference = [0.0f32; 512];
        executor.render_block(&mut chained);
        reference_executor.render_block(&mut reference);
        for (a, b) in chained.iter().zip(reference.iter()) {
            assert!((a - b).abs() < 1e-6, "chain diverged: {a} vs {b}");
        }
    }

    #[test]
    fn pan_matrix_places_a_lane_in_the_stereo_field() {
        // The pan primitive per chorus a14: an explicit 2×2 equal-power
        // matrix on the lane → master edge.
        let render_with_pan = |pan: f32| -> [f32; 512] {
            let (mut controller, mut executor) = render_plane();
            let spec = RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                master_limiter: None,
                stages: vec![
                    lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
                    master_node(vec![RenderEdgeSpec {
                        source_stage_id: LANE_ID,
                        gain: 1.0,
                        matrix: Some(equal_power_pan_matrix(pan).to_vec()),
                    }]),
                ],
            };
            controller.install_plan(&spec).unwrap();
            controller.set_playing(true).unwrap();
            warm_up(&mut executor, 2);
            let mut frames = [0.0f32; 512];
            executor.render_block(&mut frames);
            frames
        };

        let hard_left = render_with_pan(-1.0);
        assert!(hard_left.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
        assert!(hard_left.chunks_exact(2).all(|frame| frame[1] == 0.0));

        let hard_right = render_with_pan(1.0);
        assert!(hard_right
            .chunks_exact(2)
            .all(|frame| frame[0].abs() < 1e-6));
        assert!(hard_right.chunks_exact(2).any(|frame| frame[1].abs() > 0.1));

        let center = render_with_pan(0.0);
        let minus_3db = std::f32::consts::FRAC_1_SQRT_2;
        for frame in center.chunks_exact(2) {
            assert!((frame[0] - frame[1]).abs() < 1e-6);
        }
        // Center sits -3 dB against the hard-left reference.
        let left_peak = hard_left
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        let center_peak = center
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        assert!((center_peak - left_peak * minus_3db).abs() < 1e-3);
    }

    #[test]
    fn mono_lane_upmixes_to_stereo_through_the_default_adapter() {
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                master_node(vec![identity_edge(LANE_ID)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        let peak = frames
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        assert!(peak > 0.1, "mono lane should be audible after upmix");
        for frame in frames.chunks_exact(2) {
            // Equal distribution at -3 dB: both channels identical.
            assert_eq!(frame[0], frame[1]);
        }
        // -3 dB against a stereo lane at unity.
        assert!((peak - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.05);
    }

    #[test]
    fn send_topology_sums_both_paths() {
        // Source feeds Sum A and Sum B (a send-like fan-out); both feed the
        // output. The
        // output must be exactly double the single-path render.
        let (mut controller, mut executor) = render_plane();
        let sum_stage = |stage_id: u64| RenderStageSpec {
            processor: None,
            events: None,
            stage_id,
            format: ChannelFormat::stereo(),
            gain: 1.0,
            gain_automation: None,
            kind: RenderStageKind::Sum,
            inputs: vec![identity_edge(LANE_ID)],
        };
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane_node(LANE_ID, 0.25, vec![tone_clip(440.0)]),
                sum_stage(10),
                sum_stage(11),
                master_node(vec![identity_edge(10), identity_edge(11)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut sent = [0.0f32; 512];
        let mut single = [0.0f32; 512];
        executor.render_block(&mut sent);
        reference_executor.render_block(&mut single);
        for (doubled, reference) in sent.iter().zip(single.iter()) {
            assert!(
                (doubled - reference * 2.0).abs() < 1e-6,
                "send sum diverged: {doubled} vs 2×{reference}",
            );
        }
    }

    #[test]
    fn wider_master_downmixes_at_the_hardware_boundary() {
        // 4-channel master on a 2-channel stream: the boundary matrix
        // (compiled at install, when the stream is known) folds channels
        // 0/2 onto left and 1/3 onto right at equal weight.
        let (mut controller, mut executor) = render_plane();
        controller.set_stream_channels(2).unwrap();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: MASTER_ID,
                    format: ChannelFormat {
                        channels: 4,
                        layout: ChannelLayout::Generic,
                    },
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Output,
                    // Distinct synthetic spread: [1.0, 0.5, 0.25, 0.75].
                    inputs: vec![RenderEdgeSpec {
                        source_stage_id: LANE_ID,
                        gain: 1.0,
                        matrix: Some(vec![1.0, 0.5, 0.25, 0.75]),
                    }],
                },
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Mono reference at unity for the same tone.
        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(1.0, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut downmixed = [0.0f32; 512];
        let mut reference = [0.0f32; 512];
        executor.render_block(&mut downmixed);
        reference_executor.render_block(&mut reference);
        // Boundary fold (4→2): L = (c0 + c2)/2 = (1.0 + 0.25)/2 = 0.625×tone,
        // R = (c1 + c3)/2 = (0.5 + 0.75)/2 = 0.625×tone.
        for (frame, reference_frame) in downmixed.chunks_exact(2).zip(reference.chunks_exact(2)) {
            let tone = reference_frame[0];
            assert!((frame[0] - tone * 0.625).abs() < 1e-5);
            assert!((frame[1] - tone * 0.625).abs() < 1e-5);
        }
        // Clock advances by stream frames (2-channel framing): 512+256.
        assert_eq!(controller.position_frames(), 768);
    }

    #[test]
    fn narrower_master_leaves_extra_stream_channels_silent() {
        // Mono master on a stereo stream: the hardware stage never invents
        // an upmix — channel 0 carries the master, channel 1 stays silent.
        let (mut controller, mut executor) = render_plane();
        controller.set_stream_channels(2).unwrap();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: MASTER_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Output,
                    inputs: vec![identity_edge(LANE_ID)],
                },
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
        assert!(frames.chunks_exact(2).all(|frame| frame[1] == 0.0));
    }

    #[test]
    fn meters_publish_per_stage_levels_and_zero_when_silent() {
        let (mut controller, mut executor) = render_plane();
        // No install yet: nothing to resolve against.
        assert!(controller.meters().is_empty());

        controller.install_plan(&tone_spec(440.0)).unwrap();
        // Installed but not yet rendered: the shared table still carries the
        // previous generation, so the controller refuses to mislabel slots.
        assert!(controller.meters().is_empty());

        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let meters = controller.meters();
        assert_eq!(meters.len(), 2);
        let (lane_id, lane_peak, lane_rms) = meters[0];
        let (master_id, master_peak, _) = meters[1];
        assert_eq!(lane_id, LANE_ID);
        assert_eq!(master_id, MASTER_ID);
        assert!(lane_peak > 0.01, "lane peak {lane_peak}");
        assert!(lane_rms > 0.001 && lane_rms <= lane_peak);
        assert!(master_peak > 0.01, "master peak {master_peak}");

        // Stop: after the ramp-out, silent blocks publish zeros.
        controller.set_playing(false).unwrap();
        warm_up(&mut executor, 4);
        let meters = controller.meters();
        assert!(meters
            .iter()
            .all(|(_, peak, rms)| *peak == 0.0 && *rms == 0.0));
    }

    #[test]
    fn callback_health_counters_advance_and_infer_xruns() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = [0.0f32; 512]; // 256 frames ≈ 5.3 ms at 48 kHz.
        executor.render_block(&mut frames);
        executor.render_block(&mut frames);
        assert_eq!(controller.callback_count(), 2);
        assert!(
            controller.max_callback_duration_micros() >= controller.last_callback_duration_micros()
        );
        // Back-to-back blocks are far faster than the deadline: no xruns.
        assert_eq!(controller.xrun_count(), 0);

        // Starve the callback past 1.5 × the block duration: one xrun.
        std::thread::sleep(std::time::Duration::from_millis(20));
        executor.render_block(&mut frames);
        assert_eq!(controller.xrun_count(), 1);
    }

    /// FNV-1a 64 over the bit pattern of rendered samples.
    fn fnv1a_hash_pcm(frames: &[f32]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for sample in frames {
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    }

    #[test]
    fn golden_render_hash_is_stable() {
        // Reference plan: two tone lanes panned hard left/right through a
        // Sum stage, mixed with a centered mono source at the output. Renders 8 ×
        // 256-frame blocks from transport start (edge ramp included) and
        // hashes the PCM. Gates every render-plane change: any behavioral
        // drift in declick, smoothing, scheduling, or matrix application
        // moves the hash.
        //
        // Regenerating after an INTENTIONAL change: run with the assert
        // relaxed (or print `hash`), paste the new value, and justify the
        // change in the commit. Never regenerate to silence a failure you
        // cannot explain.
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 0.8,
            master_limiter: None,
            stages: vec![
                lane_node(1, 0.5, vec![tone_clip(440.0)]),
                lane_node(2, 0.4, vec![tone_clip(660.0)]),
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 3,
                    format: ChannelFormat::mono(),
                    gain: 0.3,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(220.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    processor: None,
                    events: None,
                    stage_id: 10,
                    format: ChannelFormat::stereo(),
                    gain: 0.9,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![
                        RenderEdgeSpec {
                            source_stage_id: 1,
                            gain: 1.0,
                            matrix: Some(equal_power_pan_matrix(-1.0).to_vec()),
                        },
                        RenderEdgeSpec {
                            source_stage_id: 2,
                            gain: 0.8,
                            matrix: Some(equal_power_pan_matrix(1.0).to_vec()),
                        },
                    ],
                },
                master_node(vec![identity_edge(10), identity_edge(3)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        let mut frames = [0.0f32; 512];
        for _ in 0..8 {
            executor.render_block(&mut frames);
            // Chain the per-block hashes by re-seeding from the running value.
            hash ^= fnv1a_hash_pcm(&frames);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Recorded on first run (see regeneration note above).
        assert_eq!(
            hash, GOLDEN_RENDER_HASH,
            "golden render drifted: {hash:#018x}"
        );
    }

    /// Recorded output hash for `golden_render_hash_is_stable` (captured on
    /// the test's first run; see the regeneration note in the test body).
    const GOLDEN_RENDER_HASH: u64 = 0x494b_7128_ef17_1a6a;
}
