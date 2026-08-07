//! Render-plane command mailbox and shared meter state.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

use crate::plan::RenderPlan;
use crate::{RenderPluginEvent, LIVE_EVENT_PUSH_CAPACITY, METER_SLOT_CAPACITY};

#[allow(clippy::large_enum_variant)]
pub(crate) enum RenderCommand {
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
    /// Live render posture (g13.018): while active the executor renders
    /// stages even when the transport is stopped — live-input monitoring and
    /// live-pushed events stay audible. Compiled timeline content stays
    /// `playing`-gated and the transport position does not advance.
    SetLiveRender {
        active: bool,
    },
    /// Live-event fast path (g13.018), mirror of the gain fast path:
    /// `stage_index` addresses the ACTIVE plan's topological stage list
    /// (resolved control-side against `last_topology`). The batch is inline
    /// and fixed-size so the command allocates nothing after the
    /// controller-side copy; only `events[..len]` is meaningful.
    PushLiveEvents {
        stage_index: usize,
        events: [RenderPluginEvent; LIVE_EVENT_PUSH_CAPACITY],
        len: usize,
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
pub(crate) struct MeterSlot {
    pub(crate) peak_bits: AtomicU32,
    pub(crate) rms_bits: AtomicU32,
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
pub(crate) struct SharedState {
    /// Stream-clock position in frames, written by the executor.
    pub(crate) position_frames: AtomicU64,
    /// Transport gate as last applied by the executor.
    pub(crate) playing: AtomicBool,
    /// Blocks rendered while a retired plan could not be returned because the
    /// retired mailbox was full (plan held in the parking slot instead —
    /// never dropped on the audio thread).
    pub(crate) retired_parked_blocks: AtomicU64,
    /// Fixed-capacity per-stage meter table: slot `i` holds the meter for
    /// the active plan's topological stage `i`. Stages past
    /// [`METER_SLOT_CAPACITY`] are silently unmetered.
    pub(crate) meter_slots: [MeterSlot; METER_SLOT_CAPACITY],
    /// Generation stamp of the plan the meter slots currently describe
    /// (assigned control-side per install, written by the executor alongside
    /// the slots). Readers compare it against their last install to map
    /// slots → stage ids; a mismatch means the executor has not switched to
    /// the latest plan yet and the slots describe the previous topology.
    pub(crate) meter_generation: AtomicU64,
    /// Total render callbacks observed (incremented once per `render_block`).
    pub(crate) callback_count: AtomicU64,
    /// Wall-clock duration of the most recent callback, in microseconds.
    pub(crate) last_callback_duration_micros: AtomicU64,
    /// Maximum observed callback duration, in microseconds.
    pub(crate) max_callback_duration_micros: AtomicU64,
    /// Inferred missed deadlines: callbacks whose interval since the
    /// previous callback exceeded [`XRUN_INTERVAL_FACTOR`] × the block
    /// duration at the plan rate.
    pub(crate) xrun_count: AtomicU64,
    /// Live render posture as last applied by the executor (g13.018).
    pub(crate) live_render: AtomicBool,
    /// Live events dropped instead of delivered (g13.018): ring overflow,
    /// per-block scratch overflow, pushes addressing a stage that no longer
    /// accepts them, or pushes with no plan installed. Monotonic; surfaces
    /// on the controller like the xrun counter.
    pub(crate) live_event_drop_count: AtomicU64,
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
            live_render: AtomicBool::new(false),
            live_event_drop_count: AtomicU64::new(0),
        }
    }
}

/// Identity snapshot of one installed stage in topological order: enough for
/// the controller to resolve fast-path commands (`set_stage_gain`,
/// `push_live_events`) and label meter slots without recompiling.
#[derive(Debug, Clone)]
pub(crate) struct TopologyStage {
    pub(crate) stage_id: u64,
    pub(crate) clip_ids: Vec<u64>,
    pub(crate) accepts_live_events: bool,
}
