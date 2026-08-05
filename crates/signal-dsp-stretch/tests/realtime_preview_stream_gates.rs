//! `g10.040` Batch 40.3 evidence gates, in the blocking order Batch 40.2 froze.
//!
//! `G1` allocation-free callback execution
//! `G2` bounded work across the frozen ratio range
//! `G3` sustained ratios either side of `1.0` produce continuous output with no
//!      dropped source
//! `G4` reported source consumption equals actual kernel consumption
//! `G5` underrun reports silence rather than a normal-looking block
//! `G6` dynamic-ratio changes land inside the frozen alignment tolerance

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use signal_dsp_stretch::{
    RealtimePreviewStreamConfig, RealtimePreviewStreamError, RealtimePreviewStreamState,
    REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MIN_RATIO,
};
use signal_primitives::SampleRate;

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
    static CALLBACK_ALLOCS: Cell<usize> = const { Cell::new(0) };
}
static ARMED: AtomicBool = AtomicBool::new(false);

struct CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ARMED.load(Ordering::Relaxed) {
            IN_CALLBACK.with(|flag| {
                if flag.get() {
                    CALLBACK_ALLOCS.with(|count| count.set(count.get() + 1));
                }
            });
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator;

const RATE: u32 = 48_000;
const BLOCK: usize = 128;
const CHANNELS: usize = 2;

fn state(block: usize) -> RealtimePreviewStreamState {
    RealtimePreviewStreamState::new(RealtimePreviewStreamConfig::new(
        SampleRate(RATE),
        CHANNELS,
        block,
    ))
    .expect("preview stream should plan")
}

/// A 220 Hz tone with a click every 250 ms, interleaved stereo.
fn source(frames: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|index| {
            let phase = std::f32::consts::TAU * 220.0 * index as f32 / RATE as f32;
            let tone = 0.4 * phase.sin();
            let click = if index % (RATE as usize / 4) < 32 {
                0.5
            } else {
                0.0
            };
            [tone + click, tone + click]
        })
        .collect()
}

/// Keep the producer ahead of demand, the way a non-realtime filler would.
fn top_up(state: &mut RealtimePreviewStreamState, source: &[f32], cursor: &mut usize) {
    let demand = state.source_demand_frame() as usize;
    if demand <= *cursor {
        return;
    }
    let wanted = (demand - *cursor).min(source.len() / CHANNELS - *cursor);
    if wanted == 0 {
        return;
    }
    let slice = &source[*cursor * CHANNELS..(*cursor + wanted) * CHANNELS];
    let accepted = state.push_source(slice);
    *cursor += accepted;
}

#[test]
fn g1_callback_render_allocates_nothing() {
    let mut state = state(BLOCK);
    let source = source(RATE as usize * 4);
    let mut cursor = 0usize;
    let mut output = vec![0.0f32; BLOCK * CHANNELS];

    // Warm the rings and the producer outside the measured region.
    for _ in 0..64 {
        top_up(&mut state, &source, &mut cursor);
        let _ = state.render(&mut output, BLOCK, 1.0);
    }

    ARMED.store(true, Ordering::Relaxed);
    CALLBACK_ALLOCS.with(|count| count.set(0));
    for iteration in 0..256 {
        // The producer is not the callback, so it stays outside the arm.
        top_up(&mut state, &source, &mut cursor);
        let ratio = if iteration % 2 == 0 { 0.5 } else { 2.0 };
        IN_CALLBACK.with(|flag| flag.set(true));
        let result = state.render(&mut output, BLOCK, ratio);
        IN_CALLBACK.with(|flag| flag.set(false));
        result.expect("render should succeed");
    }
    ARMED.store(false, Ordering::Relaxed);

    assert_eq!(
        CALLBACK_ALLOCS.with(Cell::get),
        0,
        "preview stream callback path allocated"
    );
}

#[test]
fn g2_work_is_bounded_across_the_frozen_ratio_range() {
    let mut state = state(BLOCK);
    let source = source(RATE as usize * 8);
    let mut cursor = 0usize;
    let mut output = vec![0.0f32; BLOCK * CHANNELS];

    // Work per callback is `block / (analysis_hop * ratio)` spectral frames.
    // The floor exists so this cannot grow without bound; check the worst case
    // the frozen range allows, with one frame of slack for cursor phase.
    let hop = state.config().analysis_hop as f64;
    let worst = (BLOCK as f64 / (hop * REALTIME_PREVIEW_STREAM_MIN_RATIO)).ceil() as usize + 1;

    for _ in 0..64 {
        top_up(&mut state, &source, &mut cursor);
        let _ = state.render(&mut output, BLOCK, REALTIME_PREVIEW_STREAM_MIN_RATIO);
    }
    let mut peak = 0usize;
    for _ in 0..512 {
        top_up(&mut state, &source, &mut cursor);
        let report = state
            .render(&mut output, BLOCK, REALTIME_PREVIEW_STREAM_MIN_RATIO)
            .expect("render should succeed");
        peak = peak.max(report.spectral_frames);
    }
    assert!(
        peak <= worst,
        "spectral frames per callback {peak} exceeded the bound {worst} at the minimum ratio"
    );

    // Out-of-range ratios are rejected rather than clamped: a silent clamp
    // would make reported and actual source advance disagree.
    let below = REALTIME_PREVIEW_STREAM_MIN_RATIO / 2.0;
    assert!(matches!(
        state.render(&mut output, BLOCK, below),
        Err(RealtimePreviewStreamError::RatioOutOfRange { .. })
    ));
    let above = REALTIME_PREVIEW_STREAM_MAX_RATIO * 2.0;
    assert!(matches!(
        state.render(&mut output, BLOCK, above),
        Err(RealtimePreviewStreamError::RatioOutOfRange { .. })
    ));
}

/// A linear 200 -> 3000 Hz sweep. Position in the sweep encodes position in
/// the source, so the frequency the output reaches says exactly how much source
/// was consumed — and a backward jump says source was skipped.
fn sweep(frames: usize) -> Vec<f32> {
    let mut phase = 0.0f32;
    (0..frames)
        .flat_map(|index| {
            let progress = index as f32 / frames as f32;
            phase += std::f32::consts::TAU * (200.0 + 2800.0 * progress) / RATE as f32;
            let sample = 0.4 * phase.sin();
            [sample, sample]
        })
        .collect()
}

/// Zero-crossing frequency estimate per chunk of rendered output.
fn frequency_per_chunk(rendered: &[f32], chunk: usize) -> Vec<f32> {
    let frames = rendered.len() / CHANNELS;
    let mut estimates = Vec::new();
    let mut start = 0usize;
    while start + chunk <= frames {
        let mut crossings = 0usize;
        for frame in start..start + chunk - 1 {
            let current = rendered[frame * CHANNELS];
            let next = rendered[(frame + 1) * CHANNELS];
            if (current >= 0.0) != (next >= 0.0) {
                crossings += 1;
            }
        }
        estimates.push(crossings as f32 * RATE as f32 / (2.0 * chunk as f32));
        start += chunk;
    }
    estimates
}

#[test]
fn g3_sustained_ratios_consume_source_at_the_rate_the_ratio_implies() {
    // A decile "is it silent" check does NOT discriminate here: the shipped
    // quantum-locked kernel drops source and still emits audio, so it passes
    // that test. Measured against this sweep it ends at 2895 Hz where the ratio
    // implies 1600 Hz, having consumed almost the whole source to fill an
    // output span that should have taken half of it, with 141 Hz of backward
    // jump where the ring guard skipped ahead. This gate sees that; a loudness
    // gate cannot.
    const SOURCE_SECONDS: usize = 12;
    const OUTPUT_SECONDS: usize = 4;
    let source_frames = RATE as usize * SOURCE_SECONDS;
    let source = sweep(source_frames);

    for ratio in [0.5f64, 1.0, 2.0] {
        let mut state = state(BLOCK);
        let mut cursor = 0usize;
        let mut output = vec![0.0f32; BLOCK * CHANNELS];
        let mut rendered: Vec<f32> = Vec::new();
        let mut underrun_total = 0usize;

        let blocks = RATE as usize * OUTPUT_SECONDS / BLOCK;
        for _ in 0..blocks {
            top_up(&mut state, &source, &mut cursor);
            let report = state
                .render(&mut output, BLOCK, ratio)
                .expect("render should succeed");
            underrun_total += report.underrun_frames;
            rendered.extend_from_slice(&output);
        }

        assert_eq!(
            underrun_total, 0,
            "ratio {ratio}: the producer kept up, so no underrun should be reported"
        );

        let estimates = frequency_per_chunk(&rendered, 2048);
        let usable = &estimates[estimates.len() / 8..];

        // Source consumed is output/ratio, so the sweep should have advanced
        // that far and no further. This number is predicted from the ratio, not
        // fitted to the measurement.
        let consumed_fraction = (OUTPUT_SECONDS as f64 / ratio) / SOURCE_SECONDS as f64;
        let expected_hz = 200.0 + 2800.0 * consumed_fraction as f32;
        let reached = *usable.last().expect("chunks");
        assert!(
            (reached - expected_hz).abs() < 250.0,
            "ratio {ratio}: output reached {reached:.0}Hz, ratio implies {expected_hz:.0}Hz \
             (consuming source faster or slower than the ratio means source was dropped or stalled)"
        );

        // A monotone source must stay monotone. Backward jumps are skipped
        // source; the shipped kernel shows 141 Hz here.
        let mut max_backward = 0.0f32;
        for pair in usable.windows(2) {
            max_backward = max_backward.max(pair[0] - pair[1]);
        }
        assert!(
            max_backward < 60.0,
            "ratio {ratio}: frequency jumped backward by {max_backward:.0}Hz, so source was skipped"
        );
    }
}

#[test]
fn g4_reported_source_consumption_equals_actual() {
    for ratio in [0.5f64, 1.0, 2.0] {
        let mut state = state(BLOCK);
        let source = source(RATE as usize * 8);
        let mut cursor = 0usize;
        let mut output = vec![0.0f32; BLOCK * CHANNELS];
        let mut summed = 0u64;
        let mut last_total = 0u64;

        for _ in 0..300 {
            top_up(&mut state, &source, &mut cursor);
            let report = state
                .render(&mut output, BLOCK, ratio)
                .expect("render should succeed");
            summed += report.source_frames_consumed;
            last_total = report.total_source_frames_consumed;
        }

        assert_eq!(
            summed, last_total,
            "ratio {ratio}: per-block consumption must sum to the running total"
        );

        // The kernel consumes `output / ratio` source frames. Allow one
        // analysis hop of slack for where the cursor sits at the boundary.
        let produced = 300.0 * BLOCK as f64;
        let expected = produced / ratio;
        let slack = state.config().analysis_hop as f64 * 2.0;
        assert!(
            (last_total as f64 - expected).abs() <= slack,
            "ratio {ratio}: consumed {last_total} source frames, ratio implies {expected:.0}"
        );
    }
}

#[test]
fn g5_underrun_is_reported_rather_than_hidden() {
    let mut state = state(BLOCK);
    let source = source(RATE as usize);
    let mut output = vec![0.0f32; BLOCK * CHANNELS];

    // Deliberately starve the kernel: give it one window and nothing more.
    let window = state.config().window_size;
    state.push_source(&source[..window * CHANNELS]);

    let mut saw_underrun = false;
    let mut underrun_frames = 0usize;
    for _ in 0..64 {
        let report = state
            .render(&mut output, BLOCK, 1.0)
            .expect("render should succeed even when starved");
        if report.underrun_frames > 0 {
            saw_underrun = true;
            underrun_frames += report.underrun_frames;
        }
    }

    assert!(
        saw_underrun,
        "a starved kernel must report underrun, not return a normal-looking block"
    );
    assert!(underrun_frames > 0);

    // Starving must not advance past source the producer never delivered.
    assert!(
        state.source_demand_frame() >= state.source_write_frame(),
        "demand must stay ahead of what the producer has supplied"
    );
}

#[test]
fn g6_ratio_changes_land_inside_the_frozen_alignment_tolerance() {
    let mut state = state(BLOCK);
    let source = source(RATE as usize * 8);
    let mut cursor = 0usize;
    let mut output = vec![0.0f32; BLOCK * CHANNELS];
    let tolerance = state.config().analysis_hop as u64;

    for _ in 0..32 {
        top_up(&mut state, &source, &mut cursor);
        let _ = state.render(&mut output, BLOCK, 1.0);
    }

    let mut changes = 0u64;
    for iteration in 0..200 {
        top_up(&mut state, &source, &mut cursor);
        let ratio = match iteration % 4 {
            0 => 1.0,
            1 => 1.5,
            2 => 0.75,
            _ => 2.0,
        };
        let report = state
            .render(&mut output, BLOCK, ratio)
            .expect("render should succeed");
        assert!(
            report.ratio_change_alignment_error_frames <= tolerance,
            "alignment error {} exceeded the frozen tolerance {tolerance}",
            report.ratio_change_alignment_error_frames
        );
        changes = report.ratio_change_count;
    }
    assert!(changes > 0, "ratio changes should have been applied");
}

/// `G7` preview quality against the whole-buffer preview at the same geometry.
///
/// The threshold is not a constant. It is the score the *same* whole-buffer
/// algorithm gets against itself with the source shifted by half an analysis
/// hop — the metric's own sensitivity to frame-grid phase. Measured, that
/// control is `0.0639` at ratio `0.5` and about `0.62` at `1.5` and `2.0`, so
/// waveform correlation cannot resolve phase-vocoder quality away from unity
/// ratio and a hard-coded threshold here would be measuring grid phase.
///
/// A self-calibrating gate instead: the candidate must beat the control. It
/// does, by `11x` at ratio `0.5`.
#[test]
fn g7_quality_matches_the_whole_buffer_preview_at_the_same_geometry() {
    use signal_dsp_stretch::RealtimePreviewStretcher;

    let source: Vec<f32> = (0..RATE as usize * 8)
        .flat_map(|index| {
            let t = index as f32 / RATE as f32;
            let chord = 0.22 * (std::f32::consts::TAU * 220.0 * t).sin()
                + 0.18 * (std::f32::consts::TAU * 277.0 * t).sin()
                + 0.15 * (std::f32::consts::TAU * 330.0 * t).sin();
            let click = if index % (RATE as usize / 4) < 24 {
                0.45
            } else {
                0.0
            };
            [chord + click, chord + click]
        })
        .collect();

    for ratio in [0.5f64, 1.0, 1.5, 2.0] {
        let mut state = state(BLOCK);
        let mut cursor = 0usize;
        let mut block_out = vec![0.0f32; BLOCK * CHANNELS];
        let mut streaming: Vec<f32> = Vec::new();
        let mut underrun_total = 0usize;
        for _ in 0..(RATE as usize * 3 / BLOCK) {
            top_up(&mut state, &source, &mut cursor);
            let report = state
                .render(&mut block_out, BLOCK, ratio)
                .expect("render should succeed");
            underrun_total += report.underrun_frames;
            streaming.extend_from_slice(&block_out);
        }
        assert_eq!(underrun_total, 0, "ratio {ratio}: unexpected underrun");

        let whole = RealtimePreviewStretcher::new(ratio)
            .stretch_interleaved_stereo(&source)
            .expect("whole-buffer preview");
        let shifted = RealtimePreviewStretcher::new(ratio)
            .stretch_interleaved_stereo(&source[64 * CHANNELS..])
            .expect("grid-shifted whole-buffer preview");

        let n = streaming.len().min(whole.len()).min(shifted.len());
        let candidate = correlation(&streaming[..n], &whole[..n]);
        let control = correlation(&whole[..n], &shifted[..n]);

        // The control decides which standard applies rather than being the bar
        // directly. Where it scores near-perfect the metric is reliable and the
        // candidate must be near-perfect too; where a half-hop grid shift
        // destroys it, beating that floor is the only meaningful claim
        // available. At ratio 1.0 the control is degenerate — identity ratio
        // returns the input verbatim, so a shifted source still scores 1.0.
        let bar = if control >= 0.99 { 0.99 } else { control };
        assert!(
            candidate > bar,
            "ratio {ratio}: candidate correlation {candidate:.4} did not clear {bar:.4} \
             (grid-phase control {control:.4})"
        );

        // Level and spectrum are what waveform correlation cannot see, and they
        // must match closely.
        let (rms_candidate, rms_whole) = (rms(&streaming[..n]), rms(&whole[..n]));
        assert!(
            (rms_candidate - rms_whole).abs() / rms_whole.max(1.0e-9) < 0.05,
            "ratio {ratio}: RMS {rms_candidate:.4} vs whole-buffer {rms_whole:.4}"
        );
    }
}

fn rms(samples: &[f32]) -> f64 {
    (samples
        .iter()
        .map(|s| (*s as f64) * (*s as f64))
        .sum::<f64>()
        / samples.len().max(1) as f64)
        .sqrt()
}

/// Best correlation over a symmetric lag search, so a constant offset between
/// two renders is not mistaken for divergence.
fn correlation(a: &[f32], b: &[f32]) -> f64 {
    let frames = (a.len() / CHANNELS).min(b.len() / CHANNELS);
    let mut best = -1.0f64;
    for signed in -6000i64..6000 {
        let (offset_a, offset_b) = if signed >= 0 {
            (0usize, signed as usize)
        } else {
            ((-signed) as usize, 0usize)
        };
        if offset_a.max(offset_b) + 4096 >= frames {
            continue;
        }
        let len = frames - offset_a.max(offset_b);
        let (mut num, mut da, mut db) = (0.0f64, 0.0f64, 0.0f64);
        for index in 0..len {
            let x = a[(index + offset_a) * CHANNELS] as f64;
            let y = b[(index + offset_b) * CHANNELS] as f64;
            num += x * y;
            da += x * x;
            db += y * y;
        }
        let denom = (da * db).sqrt();
        if denom > 0.0 {
            best = best.max(num / denom);
        }
    }
    best
}

/// `G8` the contract opens only inside the envelope the gates prove.
#[test]
fn g8_contract_reports_callback_safe_streaming_within_the_envelope() {
    use signal_dsp_stretch::{
        RealtimePreviewCallbackTimelineMode, RealtimePreviewIntegrationMode,
        REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES,
    };

    let state = state(BLOCK);
    let contract = state.contract();
    assert!(
        contract.audio_thread_processing_supported,
        "the streaming kernel should open the callback tier"
    );
    assert_eq!(
        contract.integration_mode,
        RealtimePreviewIntegrationMode::CallbackSafeStreaming
    );
    assert_eq!(
        contract.callback_timeline_mode,
        RealtimePreviewCallbackTimelineMode::SourceProjected
    );
    assert_eq!(contract.unsupported_mode, None);
    assert_eq!(
        contract.ratio_change_alignment_tolerance_frames,
        state.config().analysis_hop,
        "the reported tolerance must be the one G6 proves, not a looser one"
    );

    // The frozen ceiling, checked against the real allocation rather than an
    // estimate. Stereo at MAX_BLOCK_FRAMES is the worst case Batch 40.2 sized.
    let widest = RealtimePreviewStreamState::new(RealtimePreviewStreamConfig::new(
        SampleRate(RATE),
        2,
        4096,
    ))
    .expect("widest supported configuration should plan");
    eprintln!(
        "G8: widest supported working set {} bytes ({:.1} KiB) against a {:.1} KiB ceiling",
        widest.working_bytes(),
        widest.working_bytes() as f64 / 1024.0,
        REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES as f64 / 1024.0
    );
    assert!(
        widest.working_bytes() <= REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES,
        "working set {} exceeds the frozen ceiling {}",
        widest.working_bytes(),
        REALTIME_PREVIEW_STREAM_MAX_WORKING_BYTES
    );
    assert!(widest.contract().audio_thread_processing_supported);
}

/// `G9` no callback deadline miss under sustained load.
///
/// Wall-clock, so it lives in the soak lane: it asserts the callback finishes
/// inside its real-time budget, which is a claim about the host. `A20`, `A21`
/// and `A22` were all this kind of assertion running in the default suite.
#[test]
fn g9_no_callback_deadline_miss_under_soak() {
    if std::env::var("SIGNAL_SOAK_TESTS").as_deref() != Ok("1") {
        eprintln!(
            "SKIPPED: wall-clock soak test; set SIGNAL_SOAK_TESTS=1 (or run `effigy test:soak`)"
        );
        return;
    }

    let mut state = state(BLOCK);
    let source = source(RATE as usize * 30);
    let mut cursor = 0usize;
    let mut output = vec![0.0f32; BLOCK * CHANNELS];
    let budget = std::time::Duration::from_secs_f64(BLOCK as f64 / RATE as f64);

    for _ in 0..256 {
        top_up(&mut state, &source, &mut cursor);
        let _ = state.render(&mut output, BLOCK, 1.0);
    }

    let mut worst = std::time::Duration::ZERO;
    let mut misses = 0usize;
    for iteration in 0..20_000 {
        top_up(&mut state, &source, &mut cursor);
        if cursor >= source.len() / CHANNELS {
            cursor = 0;
        }
        // Sweep the whole frozen range, so the worst case is exercised.
        let ratio = match iteration % 4 {
            0 => REALTIME_PREVIEW_STREAM_MIN_RATIO,
            1 => 1.0,
            2 => 2.0,
            _ => REALTIME_PREVIEW_STREAM_MAX_RATIO,
        };
        let started = std::time::Instant::now();
        let _ = state.render(&mut output, BLOCK, ratio);
        let elapsed = started.elapsed();
        worst = worst.max(elapsed);
        if elapsed > budget {
            misses += 1;
        }
    }

    eprintln!(
        "G9: worst callback {:.1}us against a {:.1}us budget ({:.1}% of deadline), {misses} misses",
        worst.as_secs_f64() * 1e6,
        budget.as_secs_f64() * 1e6,
        worst.as_secs_f64() / budget.as_secs_f64() * 100.0
    );
    assert_eq!(misses, 0, "callback missed its deadline {misses} times");
}
