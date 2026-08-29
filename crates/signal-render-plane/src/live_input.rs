//! Live input monitor sources (SPSC ring from the input callback).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use signal_primitives::SpscRing;

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
/// `LIVE_INPUT_MAX_BACKLOG_FRAMES` so monitoring latency stays bounded.
pub const LIVE_INPUT_DEFAULT_CAPACITY_FRAMES: usize = 8_192;

/// Deepest ring backlog the executor tolerates before discarding old input
/// (~21 ms at 48 kHz). A feeder that pushed while the transport was stopped
/// (the executor only renders while playing) would otherwise replay stale
/// audio as extra monitoring latency on the next play.
pub(crate) const LIVE_INPUT_MAX_BACKLOG_FRAMES: usize = 1_024;

/// Stack scratch frames for live-input drain/discard loops (alloc-free).
pub(crate) const LIVE_INPUT_CHUNK_FRAMES: usize = 256;

pub(crate) struct LiveInputInner {
    /// Interleaved stereo samples at the stream rate.
    pub(crate) ring: SpscRing,
    /// Output frames rendered as silence because the ring ran dry.
    pub(crate) underrun_frames: AtomicU64,
}

/// Executor-side handle to a live input feed. Arc-shared and pointer-equal
/// (like `RenderStreamHandle`): create one per monitored input and reuse
/// it across plan recompiles so specs stay idempotent. The ring lives inside
/// the shared handle, so plan swaps inherently keep the audio flowing —
/// there is no per-plan state to migrate.
#[derive(Clone)]
pub struct RenderLiveInputHandle {
    pub(crate) inner: Arc<LiveInputInner>,
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
    pub(crate) inner: Arc<LiveInputInner>,
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
/// into `RenderSource::LiveInput` specs; the feeder goes to the input
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
