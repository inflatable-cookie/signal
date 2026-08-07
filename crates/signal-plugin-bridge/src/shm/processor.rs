use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

use signal_ipc::{
    MappedSharedMemoryRegion, PluginAudioBlockLayout, PluginAudioBlockView, SharedMemoryBroker,
    SharedMemoryTransportKind, SharedMemoryTransportPayload, PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY,
};
use signal_plugin::{write_event_to_slice, PluginEvent};
use signal_render_plane::{PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventSupport};

use super::budget::{
    plugin_process_wait_budget, PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT,
    PLUGIN_PROCESS_OFFLINE_WAIT_BUDGET,
};

/// Parent-side shared-memory plugin processor (DedicatedSandbox tier).
///
/// `process` is alloc-free and lock-free: it copies the scratch into the
/// region's input area, publishes a request stamp, spin-waits within the
/// bounded budget for the child's response stamp, and copies the output
/// back on success. On a miss or a dead child it leaves the scratch
/// untouched (bypass) and increments [`Self::miss_count`].
///
/// One caller at a time: exactly one render executor may call `process` on
/// a given handle (the request/response protocol is single-flight). The
/// owning host service marks the handle dead ([`Self::mark_dead`]) when the
/// child process dies, so subsequent blocks bypass immediately without
/// burning their wait budget.
pub struct ShmPluginProcessor {
    /// Keeps the mapping alive for the view's lifetime. Never read directly.
    _region: MappedSharedMemoryRegion,
    view: PluginAudioBlockView,
    layout: PluginAudioBlockLayout,
    sample_rate_hz: u32,
    /// Last request stamp published by this side.
    request_counter: AtomicU32,
    /// Blocks bypassed on budget miss or dead child.
    misses: AtomicU64,
    /// Events rejected because the v1 shared-memory block carries audio only.
    unsupported_events: AtomicU64,
    /// Fatal processing deadline runs. Reaching the consecutive-miss limit
    /// trips the backend dead; later blocks bypass until the host replaces it.
    timeouts: AtomicU64,
    consecutive_misses: AtomicU32,
    /// Cleared by the owning service on child death.
    alive: AtomicBool,
    /// Set by the offline driver; swaps the realtime wait budget for
    /// [`PLUGIN_PROCESS_OFFLINE_WAIT_BUDGET`] and the spin for a yield.
    offline_waiting: AtomicBool,
    event_support: RenderPluginEventSupport,
}

impl ShmPluginProcessor {
    /// Attach the audio block region the sandbox child leased at plugin
    /// activation. `sample_rate_hz` is the rate the plugin was activated at
    /// (it scales the wait budget).
    pub fn attach(
        region_id: &str,
        shm_path: &str,
        shm_bytes: u32,
        max_frames: u32,
        channels: u32,
        sample_rate_hz: u32,
    ) -> Result<Self, String> {
        Self::attach_with_event_support(
            region_id,
            shm_path,
            shm_bytes,
            max_frames,
            channels,
            sample_rate_hz,
            RenderPluginEventSupport::default(),
        )
    }

    /// Attach with the format-specific event delivery support published by
    /// the owning host for this backend.
    pub fn attach_with_event_support(
        region_id: &str,
        shm_path: &str,
        shm_bytes: u32,
        max_frames: u32,
        channels: u32,
        sample_rate_hz: u32,
        event_support: RenderPluginEventSupport,
    ) -> Result<Self, String> {
        let layout = PluginAudioBlockLayout {
            max_frames,
            channels,
        };
        if layout.region_bytes() != shm_bytes {
            return Err(format!(
                "audio block region size mismatch: lease says {shm_bytes} bytes, layout needs {}",
                layout.region_bytes(),
            ));
        }
        let transport = SharedMemoryTransportPayload {
            region_id: region_id.to_string(),
            transport_kind: SharedMemoryTransportKind::MappedFile,
            backing_path: shm_path.to_string(),
            total_bytes: shm_bytes,
        };
        let broker = SharedMemoryBroker::default();
        let mut region = broker
            .attach_region(&transport)
            .map_err(|error| error.to_string())?;
        // Safety: the mapping lives in `_region` for the processor's whole
        // lifetime and is exactly `layout.region_bytes()` long (validated
        // by the attach path).
        let view = unsafe { PluginAudioBlockView::new(region.as_mut_slice().as_mut_ptr(), layout) };
        let request_counter = AtomicU32::new(view.request_seq().load(Ordering::Acquire));
        Ok(Self {
            _region: region,
            view,
            layout,
            sample_rate_hz,
            request_counter,
            misses: AtomicU64::new(0),
            unsupported_events: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            consecutive_misses: AtomicU32::new(0),
            alive: AtomicBool::new(true),
            offline_waiting: AtomicBool::new(false),
            event_support,
        })
    }

    /// Blocks bypassed so far (budget miss or dead child), cumulative.
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Processing deadline misses, cumulative for this backend generation.
    pub fn timeout_count(&self) -> u64 {
        self.timeouts.load(Ordering::Relaxed)
    }

    /// Mark the backend dead (child process gone): every subsequent block
    /// bypasses immediately without waiting.
    pub fn mark_dead(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }

    /// See [`PluginBlockProcessor::set_offline_waiting`]. Inherent so callers
    /// holding a concrete `ShmPluginProcessor` need not import the trait.
    pub fn set_offline_waiting(&self, enabled: bool) -> bool {
        self.offline_waiting.swap(enabled, Ordering::Relaxed)
    }

    /// Whether the backend still considers its child alive.
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    fn process_block(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        let samples = frame_count * channels;
        unsafe { self.view.write_input(&scratch[..samples]) };
        self.view
            .frame_count()
            .store(frame_count as u32, Ordering::Relaxed);
        let request = self.request_counter.fetch_add(1, Ordering::Relaxed) + 1;
        self.view.request_seq().store(request, Ordering::Release);

        let offline = self.offline_waiting.load(Ordering::Relaxed);
        let budget = if offline {
            PLUGIN_PROCESS_OFFLINE_WAIT_BUDGET
        } else {
            plugin_process_wait_budget(frame_count, self.sample_rate_hz)
        };
        let deadline = Instant::now() + budget;
        loop {
            if self.view.response_seq().load(Ordering::Acquire) == request {
                self.consecutive_misses.store(0, Ordering::Relaxed);
                unsafe { self.view.read_output(&mut scratch[..samples]) };
                return true;
            }
            if Instant::now() >= deadline {
                self.misses.fetch_add(1, Ordering::Relaxed);
                let consecutive = self.consecutive_misses.fetch_add(1, Ordering::Relaxed) + 1;
                if consecutive >= PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT {
                    self.timeouts.fetch_add(1, Ordering::Relaxed);
                    self.alive.store(false, Ordering::Release);
                }
                return false;
            }
            if offline {
                // Seconds, not microseconds, and not on the audio thread:
                // spinning here would burn a core for the whole wait and
                // starve the very child we are waiting on.
                std::thread::yield_now();
            } else {
                std::hint::spin_loop();
            }
        }
    }
}

impl PluginBlockProcessor for ShmPluginProcessor {
    fn event_support(&self) -> RenderPluginEventSupport {
        self.event_support
    }
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        if !self.alive.load(Ordering::Relaxed) {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        // v1 hosts stereo blocks only; anything else bypasses (the plan
        // compiled a format the bridge does not carry).
        if channels != self.layout.channels as usize
            || frame_count > self.layout.max_frames as usize
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        self.view.event_count().store(0, Ordering::Relaxed);
        self.process_block(scratch, frame_count, channels)
    }

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
    }

    fn set_offline_waiting(&self, enabled: bool) -> bool {
        ShmPluginProcessor::set_offline_waiting(self, enabled)
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        if !self.alive.load(Ordering::Relaxed)
            || channels != self.layout.channels as usize
            || frame_count > self.layout.max_frames as usize
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let event_count = events.len().min(PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY);
        self.unsupported_events.fetch_add(
            events.len().saturating_sub(event_count) as u64,
            Ordering::Relaxed,
        );
        for (index, event) in events.iter().take(event_count).enumerate() {
            let Some(plugin_event): Option<PluginEvent> =
                crate::in_process::convert_block_event(event)
            else {
                self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                continue;
            };
            let mut encoded = [0u8; PluginEvent::ENCODED_BYTES];
            if write_event_to_slice(&plugin_event, &mut encoded).is_err() {
                self.unsupported_events.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            unsafe { self.view.write_event(index, &encoded) };
        }
        self.view
            .event_count()
            .store(event_count as u32, Ordering::Relaxed);
        self.process_block(scratch, frame_count, channels)
    }
}
