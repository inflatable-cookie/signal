//! Sandboxed (DedicatedSandbox tier) backend: the parent half of the
//! shared-memory audio block bridge.
//!
//! The child (sandbox broker) created the region at plugin activation and
//! runs its audio thread against it; this side attaches the same mapping,
//! posts input blocks, and bounded-spin-waits for the child's response. A
//! miss (budget exhausted) or a dead child bypasses: the caller's scratch is
//! left untouched and a miss counter increments — the engine callback never
//! blocks past the budget.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use signal_ipc::{
    MappedSharedMemoryRegion, PluginAudioBlockLayout, PluginAudioBlockView, SharedMemoryBroker,
    SharedMemoryTransportKind, SharedMemoryTransportPayload, PLUGIN_AUDIO_BLOCK_EVENT_CAPACITY,
};
use signal_plugin::{write_event_to_slice, PluginEvent};
use signal_render_plane::{PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventSupport};

/// Hard ceiling of the per-block wait budget, in microseconds.
///
/// The effective budget is `min(1 ms, 50 % of the block duration at the
/// plan rate)` — see [`plugin_process_wait_budget`]. Rationale: at typical
/// block sizes (128–1024 frames at 44.1–96 kHz, ~1.3–23 ms) half a block
/// leaves the rest of the callback comfortably inside its deadline even if
/// EVERY insert misses, and 1 ms caps the damage on very large blocks where
/// half a block would be a pointless long stall.
pub const PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS: u64 = 1_000;

/// A single scheduling miss is not evidence that the sandbox child died.
/// Keep bypass bounded, but require a short run of misses before retiring
/// the backend generation.
pub const PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT: u32 = 3;

/// Effective bounded wait for one block: `min(1 ms, 50 % of the block
/// duration at `sample_rate_hz`)`.
pub fn plugin_process_wait_budget(frame_count: usize, sample_rate_hz: u32) -> Duration {
    let half_block_micros =
        (frame_count as u64).saturating_mul(500_000) / u64::from(sample_rate_hz.max(1));
    Duration::from_micros(half_block_micros.min(PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS))
}

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

        let budget = plugin_process_wait_budget(frame_count, self.sample_rate_hz);
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
            std::hint::spin_loop();
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

#[cfg(test)]
mod tests {
    use super::*;
    use signal_render_plane::{
        RenderPluginEventKind, RenderPluginEventSupport, RenderPluginProcessor,
    };
    use std::sync::Arc;

    fn test_region(
        layout: PluginAudioBlockLayout,
    ) -> (SharedMemoryBroker, MappedSharedMemoryRegion) {
        let broker = SharedMemoryBroker::default();
        let region = broker
            .create_region("bridge-test", layout.region_bytes())
            .expect("test region should create");
        (broker, region)
    }

    #[test]
    fn dead_shared_memory_backend_bypasses_before_event_delivery() {
        let layout = PluginAudioBlockLayout {
            max_frames: 128,
            channels: 2,
        };
        let (broker, region) = test_region(layout);
        let metadata = region.metadata().clone();
        let processor = Arc::new(
            ShmPluginProcessor::attach(
                &metadata.region_id,
                &metadata.backing_path,
                metadata.total_bytes,
                layout.max_frames,
                layout.channels,
                48_000,
            )
            .expect("attach processor"),
        );
        processor.mark_dead();
        let handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);
        assert_eq!(handle.event_support(), RenderPluginEventSupport::default());
        let mut scratch = vec![0.0; 256];
        assert!(!handle.process_with_events(
            &mut scratch,
            128,
            2,
            &[RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 1.0,
                },
            }],
        ));
        assert_eq!(handle.unsupported_event_count(), 0);
        drop(handle);
        drop(processor);
        drop(region);
        let _ = broker.destroy_region(&metadata);
    }

    #[test]
    fn wait_budget_is_half_a_block_capped_at_one_millisecond() {
        // 128 frames at 48 kHz ≈ 2.67 ms block → half = ~1333 µs → capped.
        assert_eq!(
            plugin_process_wait_budget(128, 48_000),
            Duration::from_micros(1_000)
        );
        // 64 frames at 48 kHz ≈ 1.33 ms block → half ≈ 666 µs.
        assert_eq!(
            plugin_process_wait_budget(64, 48_000),
            Duration::from_micros(666)
        );
        // Tiny blocks: budget shrinks with the block.
        assert!(plugin_process_wait_budget(16, 96_000) < Duration::from_micros(100));
    }

    #[test]
    fn unanswered_request_misses_within_budget_and_leaves_scratch_untouched() {
        let layout = PluginAudioBlockLayout {
            max_frames: 256,
            channels: 2,
        };
        let (broker, mut region) = test_region(layout);
        let metadata = region.metadata().clone();
        // Child-side init, then nobody serves requests (a fake handle that
        // never responds).
        let child_view =
            unsafe { PluginAudioBlockView::new(region.as_mut_slice().as_mut_ptr(), layout) };
        child_view.initialize();

        let processor = ShmPluginProcessor::attach(
            &metadata.region_id,
            &metadata.backing_path,
            metadata.total_bytes,
            256,
            2,
            48_000,
        )
        .expect("attach should succeed");

        let mut scratch: Vec<f32> = (0..256).map(|index| index as f32).collect();
        let reference = scratch.clone();
        let budget = plugin_process_wait_budget(128, 48_000);
        let start = Instant::now();
        let processed = processor.process(&mut scratch, 128, 2);
        let elapsed = start.elapsed();
        assert!(!processed, "no child: the block must miss");
        assert_eq!(
            scratch, reference,
            "missed block must leave scratch untouched"
        );
        assert_eq!(processor.miss_count(), 1);
        assert_eq!(processor.timeout_count(), 0);
        assert!(processor.is_alive(), "one miss keeps the backend live");
        // The wait respected its budget (generous ceiling for CI jitter).
        assert!(
            elapsed < budget + Duration::from_millis(10),
            "bounded wait overran: {elapsed:?} vs budget {budget:?}",
        );

        // Repeated unanswered requests eventually retire the epoch.
        for _ in 1..PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT {
            assert!(!processor.process(&mut scratch, 128, 2));
        }
        assert_eq!(processor.timeout_count(), 1);
        assert!(!processor.is_alive());

        // Timed-out epoch: every later block bypasses immediately instead
        // of repeatedly spending the wait budget.
        let start = Instant::now();
        assert!(!processor.process(&mut scratch, 128, 2));
        assert!(start.elapsed() < Duration::from_millis(2));
        assert_eq!(
            processor.miss_count(),
            u64::from(PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT) + 1,
        );

        drop(processor);
        drop(region);
        let _ = broker.destroy_region(&metadata);
    }

    #[test]
    fn served_request_round_trips_through_the_region() {
        let layout = PluginAudioBlockLayout {
            max_frames: 64,
            channels: 2,
        };
        let (broker, mut region) = test_region(layout);
        let metadata = region.metadata().clone();
        let child_view =
            unsafe { PluginAudioBlockView::new(region.as_mut_slice().as_mut_ptr(), layout) };
        child_view.initialize();

        let attach = || {
            Arc::new(
                ShmPluginProcessor::attach(
                    &metadata.region_id,
                    &metadata.backing_path,
                    metadata.total_bytes,
                    64,
                    2,
                    48_000,
                )
                .expect("attach should succeed"),
            )
        };
        let mut processor = attach();
        let mut handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);

        // Fake child thread: serve requests by doubling samples, until the
        // client says it is done.
        //
        // It must keep serving rather than answer once and exit. The client
        // retries on a miss, and every retry issues a *new* request sequence.
        // A serve-once server that answers request N while the client has
        // already moved to N+1 leaves a stale response and then exits, so no
        // later request can ever be answered and the client spins to its
        // deadline. That is the second half of `A19`: the original 200-retry
        // loop had the same race but a window short enough to usually win it.
        let stop_serving = Arc::new(AtomicBool::new(false));
        let server_stop = Arc::clone(&stop_serving);
        let server = std::thread::spawn(move || {
            // Longer than the client's 30s bound on purpose, so the client
            // decides the outcome. A shorter server deadline would make this
            // thread panic first under contention and report "never saw a
            // request" when the real answer is "the host was busy".
            let deadline = Instant::now() + Duration::from_secs(60);
            let mut served = 0u32;
            loop {
                if server_stop.load(Ordering::Relaxed) {
                    return served;
                }
                let request = child_view.request_seq().load(Ordering::Acquire);
                if request != child_view.response_seq().load(Ordering::Relaxed) {
                    let frames = child_view.frame_count().load(Ordering::Relaxed) as usize;
                    let mut samples = vec![0.0f32; frames * 2];
                    unsafe { child_view.read_input(&mut samples) };
                    for sample in &mut samples {
                        *sample *= 2.0;
                    }
                    unsafe { child_view.write_output(&samples) };
                    child_view.response_seq().store(request, Ordering::Release);
                    served += 1;
                    continue;
                }
                assert!(
                    served > 0 || Instant::now() < deadline,
                    "server never saw a request"
                );
                // Yield, do not sleep. The client's wait budget is half a
                // block -- 333us at 32 frames and 48 kHz -- so a server that
                // sleeps even 1ms polls three times slower than the window it
                // has to answer within, and misses almost every request.
                // `yield_now` stays inside that window while still giving up
                // the CPU voluntarily, which a bare spin loop does not.
                std::thread::yield_now();
            }
        });

        let mut scratch: Vec<f32> = (0..64).map(|index| index as f32 / 64.0).collect();
        let reference = scratch.clone();
        // The server may take a scheduler quantum to see the request; retry
        // like the engine would (each miss bypasses cleanly). Bounded by a
        // deadline rather than an iteration count: a fixed 200 retries assumes
        // the other thread gets scheduled within 200 of this one's iterations,
        // which is a claim about host contention, not about the bridge.
        let mut processed = false;
        let mut epochs = 1u32;
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if handle.process(&mut scratch, 32, 2) {
                processed = true;
                break;
            }
            // Retrying the same handle forever cannot work. After
            // `PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` misses the processor
            // retires its epoch and every later `process` returns false
            // immediately, so a long retry loop against a retired backend is
            // futile by construction. That is what failed on CI: the first
            // three attempts missed the wait budget, the epoch retired, and
            // the remaining 30s could not have succeeded however long it ran.
            //
            // The budget is half a block capped at 1ms, which a contended
            // three-core runner can miss three times in a row through no fault
            // of the protocol. Re-attach and give the round trip a fresh epoch.
            if !processor.is_alive() {
                processor = attach();
                handle = RenderPluginProcessor::new(Arc::clone(&processor) as Arc<_>);
                epochs += 1;
            }
            // Sleep between attempts, do not spin. The thread this loop is
            // waiting for may share a core with it: CI runners have a handful
            // of cores and `cargo test --workspace` oversubscribes them badly.
            // Spinning hot here starves the server thread and the deadline
            // expires having prevented the very work it was waiting on.
            std::thread::sleep(Duration::from_millis(1));
        }

        // Join before asserting anything. `child_view` is a raw pointer into
        // `region`'s mapping, so if an assertion below panics while the server
        // thread is still spinning, unwinding drops `region`, unmaps the
        // backing memory underneath that thread, and the process dies with
        // SIGSEGV instead of reporting the failed assertion. That is finding
        // `A19`: it presented as an intermittent failure and an intermittent
        // segfault because it was both, from one cause.
        stop_serving.store(true, Ordering::Relaxed);
        let served = server.join();

        assert!(
            processed,
            "server should have answered within 30s (across {epochs} epochs)",
        );
        assert!(
            served.expect("server thread joins") >= 1,
            "server should have served at least one request",
        );
        for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
            assert!(
                (output - input * 2.0).abs() < 1e-7,
                "sample {index}: {output} vs {input} * 2",
            );
        }

        drop(handle);
        drop(processor);
        drop(region);
        let _ = broker.destroy_region(&metadata);
    }
}
