use super::*;
use signal_render_plane::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderPluginEventSupport,
    RenderPluginProcessor,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use signal_ipc::{
    MappedSharedMemoryRegion, PluginAudioBlockLayout, PluginAudioBlockView, SharedMemoryBroker,
};

fn test_region(layout: PluginAudioBlockLayout) -> (SharedMemoryBroker, MappedSharedMemoryRegion) {
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

/// A child that answers well past the realtime budget must still be heard
/// when the backend is driven offline.
///
/// `40 ms` is 40x the realtime budget for this block, and a plausible
/// stall for a sandbox child that lost its scheduling slot on a loaded
/// machine. Realtime bypasses it, correctly — the callback cannot wait.
/// Offline must not: bypass there writes a render missing the insert.
#[test]
fn offline_waiting_outlasts_a_stall_the_realtime_budget_bypasses() {
    let layout = PluginAudioBlockLayout {
        max_frames: 256,
        channels: 2,
    };
    let (broker, mut region) = test_region(layout);
    let metadata = region.metadata().clone();
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

    let stall = Duration::from_millis(40);
    assert!(
        stall > plugin_process_wait_budget(128, 48_000) * 20,
        "the stall must be far outside the realtime budget for this to mean anything",
    );

    // A deliberately late child: one request, served after the stall.
    let serving = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let request = child_view.request_seq().load(Ordering::Acquire);
            if request != 0 && child_view.response_seq().load(Ordering::Acquire) != request {
                std::thread::sleep(stall);
                let frames = child_view.frame_count().load(Ordering::Relaxed) as usize;
                let mut block = vec![0.0f32; frames * 2];
                unsafe { child_view.read_input(&mut block) };
                for sample in block.iter_mut() {
                    *sample *= 0.5;
                }
                unsafe { child_view.write_output(&block) };
                child_view.response_seq().store(request, Ordering::Release);
                return;
            }
            assert!(
                Instant::now() < deadline,
                "parent never published a request"
            );
            std::thread::yield_now();
        }
    });

    let mut scratch = vec![1.0f32; 256];
    assert!(
        !processor.set_offline_waiting(true),
        "backends attach in realtime waiting",
    );
    let processed = processor.process(&mut scratch, 128, 2);
    assert!(
        processed,
        "offline waiting must outlast a {stall:?} stall, not bypass it",
    );
    assert!(scratch[..256]
        .iter()
        .all(|sample| (*sample - 0.5).abs() < 1e-6));
    assert_eq!(processor.miss_count(), 0);
    assert!(
        processor.set_offline_waiting(false),
        "the swap returns the previous setting so callers can restore it",
    );

    serving.join().expect("serving thread");
    drop(processor);
    drop(region);
    let _ = broker.destroy_region(&metadata);
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
