//! Plugin block processor trait and thin Arc wrapper.

use std::sync::Arc;

use crate::{RenderBlockPluginEvent, RenderPluginEventSupport};

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

    /// Set one plugin parameter to a normalized `0..=1` value, returning
    /// `true` when the backend accepted it. This is the OFFLINE mirror of
    /// the host's live parameter forwarding (the same block-boundary
    /// set-parameter cadence, driven by the offline renderer instead of the
    /// host's playback poll). The default rejects the write (`false`), so
    /// backends without parameter transport stay honest: the envelope is
    /// simply not applied and the audio path is untouched. Never called on
    /// the realtime audio thread — only the offline driver uses it, between
    /// blocks.
    fn set_parameter_normalized(&self, parameter_id: u32, normalized: f32) -> bool {
        let _ = (parameter_id, normalized);
        false
    }

    /// Switch the backend between realtime and offline waiting, returning the
    /// previous setting. OFFLINE-DRIVER SEAM, never called on the audio
    /// thread; the default is a no-op returning `false`.
    ///
    /// Backends that wait on another process bound that wait, and the bound is
    /// a realtime one: the callback must return before its output buffer
    /// drains, so a slow block is bypassed rather than waited for. Bypass is
    /// the right answer there — a late block is worse than an unprocessed one.
    ///
    /// An offline render has no buffer to drain, and there bypass is the wrong
    /// answer: it does not cost latency, it silently drops the insert for that
    /// block and writes a render that differs from the one the host would
    /// play. Under machine load that happens at unpredictable block
    /// boundaries, so the damage is neither reproducible nor visible. Offline
    /// backends should therefore wait as long as the block takes, bounded only
    /// generously enough to still notice a genuinely dead child.
    ///
    /// The setting is per-backend and the protocol is single-flight
    /// ([`RenderPluginProcessor`] is documented one-caller-at-a-time), so a
    /// handle cannot be driven live and offline at once regardless.
    fn set_offline_waiting(&self, enabled: bool) -> bool {
        let _ = enabled;
        false
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

    /// Set one plugin parameter to a normalized `0..=1` value; `true` when
    /// the backend accepted it. Offline-driver seam only (see
    /// [`PluginBlockProcessor::set_parameter_normalized`]).
    pub fn set_parameter_normalized(&self, parameter_id: u32, normalized: f32) -> bool {
        self.inner
            .set_parameter_normalized(parameter_id, normalized)
    }

    /// Switch this backend between realtime and offline waiting, returning the
    /// previous setting. See
    /// [`PluginBlockProcessor::set_offline_waiting`] — offline-driver seam,
    /// never called on the audio thread.
    pub fn set_offline_waiting(&self, enabled: bool) -> bool {
        self.inner.set_offline_waiting(enabled)
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
