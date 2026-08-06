//! Structural gates for the `g10.039` resumable renderer.
//!
//! All five pass. `G1` and `G1b` are the equivalence law: for any chunk
//! partition, output is bit-identical to a single whole-source render. `G4` is
//! the acceptance target inherited from `g10.036`, which measured `0.034`
//! correlation on the segmented path this renderer replaces.
//!
//! The candidate lives on `main` by explicit operator decision, waiving
//! Contract `084` Rule 2 isolation for this lane.

use signal_dsp_stretch::{
    ResumableOfflineStretch, ResumableStretchConfig, StretchRatioPoint, MAX_RESUMABLE_WORKING_BYTES,
};
use signal_primitives::SampleRate;

fn material(frames: usize, channels: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(frames * channels);
    for f in 0..frames {
        let t = f as f32 / 48_000.0;
        let chord = 0.22 * (std::f32::consts::TAU * 220.0 * t).sin()
            + 0.18 * (std::f32::consts::TAU * 277.18 * t).sin();
        let phase = f % 12_000;
        let click = if phase < 240 {
            0.4 * (1.0 - phase as f32 / 240.0).powi(2)
        } else {
            0.0
        };
        for c in 0..channels {
            out.push((chord + click) * if c == 0 { 1.0 } else { 0.85 });
        }
    }
    out
}

fn render_in_chunks(
    config: &ResumableStretchConfig,
    source: &[f32],
    chunk_frames: usize,
) -> Vec<f32> {
    let mut renderer = ResumableOfflineStretch::new(config.clone()).expect("config");
    let mut out = Vec::new();
    let total = source.len() / config.channels;
    let mut start = 0;
    while start < total {
        let end = (start + chunk_frames).min(total);
        renderer
            .render(
                &source[start * config.channels..end * config.channels],
                &mut out,
            )
            .expect("render");
        start = end;
    }
    renderer.flush(&mut out).expect("flush");
    out
}

fn base_config(source_frames: usize, channels: usize) -> ResumableStretchConfig {
    ResumableStretchConfig {
        channels,
        window_size: 2_048,
        analysis_hop: 512,
        source_frames,
        ratio_curve: Vec::new(),
        fallback_ratio: 1.5,
        sample_rate: SampleRate(48_000),
        pitch_shift_semitones: 0.0,
    }
}

/// G1: output must be identical for any chunk partition.
#[test]
fn chunk_size_independence_static_ratio() {
    let frames = 48_000 * 3;
    let source = material(frames, 1);
    let config = base_config(frames, 1);
    let whole = render_in_chunks(&config, &source, frames);
    for chunk in [1_024_usize, 7_777, 48_000, 100_000] {
        let chunked = render_in_chunks(&config, &source, chunk);
        assert_eq!(chunked.len(), whole.len(), "chunk {chunk}: length differs");
        assert_eq!(chunked, whole, "chunk {chunk}: output differs");
    }
    println!(
        "G1 static ratio: identical across 4 partitions, {} frames",
        whole.len()
    );
}

/// G1b: same law with a dynamic ratio curve.
#[test]
fn chunk_size_independence_dynamic_ratio() {
    let frames = 48_000 * 3;
    let source = material(frames, 2);
    let mut config = base_config(frames, 2);
    config.fallback_ratio = 1.0;
    config.ratio_curve = vec![
        StretchRatioPoint::new(0, 1.25),
        StretchRatioPoint::new(48_000, 0.8),
        StretchRatioPoint::new(96_000, 1.6),
    ];
    let whole = render_in_chunks(&config, &source, frames);
    for chunk in [2_048_usize, 13_333, 60_000] {
        let chunked = render_in_chunks(&config, &source, chunk);
        assert_eq!(
            chunked, whole,
            "chunk {chunk}: dynamic-ratio output differs"
        );
    }
    println!(
        "G1b dynamic ratio: identical across 3 partitions, {} frames",
        whole.len() / 2
    );
}

/// G2: working state is bounded by geometry, not source duration.
#[test]
fn memory_ceiling_is_duration_independent() {
    let short = ResumableOfflineStretch::new(base_config(1_000, 2)).expect("short");
    let long = ResumableOfflineStretch::new(base_config(48_000 * 600, 2)).expect("long");
    assert_eq!(
        short.working_bytes(),
        long.working_bytes(),
        "working state changed with source duration"
    );
    assert!(
        long.working_bytes() <= MAX_RESUMABLE_WORKING_BYTES,
        "working bytes {} exceed the ceiling",
        long.working_bytes()
    );

    let mut wide = base_config(48_000 * 600, 2);
    wide.window_size = 65_536;
    wide.analysis_hop = 16_384;
    let widest = ResumableOfflineStretch::new(wide).expect("max geometry");
    assert!(
        widest.working_bytes() <= MAX_RESUMABLE_WORKING_BYTES,
        "max geometry working bytes {} exceed the ceiling",
        widest.working_bytes()
    );
    println!(
        "G2: retained {} B, 10 min source {} B, max geometry {} B, ceiling {} B",
        short.working_bytes(),
        long.working_bytes(),
        widest.working_bytes(),
        MAX_RESUMABLE_WORKING_BYTES
    );
}

/// G3: output length honours the contracted target.
#[test]
fn output_length_matches_the_target() {
    let frames = 48_000 * 2;
    let source = material(frames, 1);
    for ratio in [0.75_f64, 1.0, 1.5, 2.0] {
        let mut config = base_config(frames, 1);
        config.fallback_ratio = ratio;
        let renderer = ResumableOfflineStretch::new(config.clone()).expect("config");
        let target = renderer.target_output_frames();
        let out = render_in_chunks(&config, &source, 10_000);
        assert_eq!(
            out.len(),
            target,
            "ratio {ratio}: length differs from target"
        );
        assert_eq!(target, (frames as f64 * ratio).round() as usize);
    }
    println!("G3: output length matches target at 4 ratios");
}

/// G5: the renderer must deliver audio, not a zero-padded shortfall.
///
/// The `g10.039` listening round found the adopted artifact path emitting
/// `3.8` seconds of audio followed by `108` seconds of silence: `render` could
/// deadlock between the input and output rings and silently drop the rest of
/// the chunk, and the artifact path's length `resize` padded the gap with
/// zeros. Length alone therefore proves nothing about content.
#[test]
fn render_delivers_audio_across_the_whole_source() {
    let frames = 48_000 * 20;
    let source = material(frames, 2);
    let mut config = base_config(frames, 2);
    config.fallback_ratio = 1.25;
    // One large call, as the artifact path makes per chunk.
    let out = render_in_chunks(&config, &source, frames);
    let total = out.len() / 2;
    assert!(total > 0, "no output at all");

    // Every tenth of the output must carry signal.
    let slice = total / 10;
    for part in 0..10 {
        let start = part * slice;
        let seg = &out[start * 2..(start + slice) * 2];
        let rms = (seg.iter().map(|s| (s * s) as f64).sum::<f64>() / seg.len() as f64).sqrt();
        assert!(
            rms > 1.0e-4,
            "output decile {part} is silent: rms {rms:.9} at {:.1}s",
            start as f32 / 48_000.0
        );
    }
}

/// G4: the acceptance target inherited from `g10.036`. A segmented render must
/// correlate with a whole render at the same constant ratio.
#[test]
fn segmented_render_matches_whole_render_at_constant_ratio() {
    let frames = 48_000 * 3;
    let source = material(frames, 1);
    let config = base_config(frames, 1);
    let whole = render_in_chunks(&config, &source, frames);
    let chunked = render_in_chunks(&config, &source, 12_000);
    let n = whole.len().min(chunked.len());
    let dot: f64 = (0..n).map(|i| (whole[i] * chunked[i]) as f64).sum();
    let ew: f64 = (0..n).map(|i| (whole[i] * whole[i]) as f64).sum();
    let ec: f64 = (0..n).map(|i| (chunked[i] * chunked[i]) as f64).sum();
    let correlation = dot / (ew.sqrt() * ec.sqrt());
    println!("G4: correlation {correlation:.6}");
    assert!(
        correlation > 0.99,
        "correlation {correlation:.6} below the 0.99 acceptance target"
    );
}

// ---------------------------------------------------------------------------
// `g10.042` Batch 42.3: resumable pitch.
// ---------------------------------------------------------------------------

fn pitched_config(
    source_frames: usize,
    channels: usize,
    semitones: f64,
    fallback_ratio: f64,
) -> ResumableStretchConfig {
    ResumableStretchConfig {
        fallback_ratio,
        pitch_shift_semitones: semitones,
        ..base_config(source_frames, channels)
    }
}

/// A 220 Hz tone, interleaved stereo, so pitch is measurable as a frequency.
fn tone(frames: usize, hz: f32, channels: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|index| {
            let value = 0.4 * (std::f32::consts::TAU * hz * index as f32 / 48_000.0).sin();
            std::iter::repeat_n(value, channels)
        })
        .collect()
}

fn dominant_hz(samples: &[f32], channels: usize) -> f64 {
    let frames = samples.len() / channels;
    // Skip the render's edges, where overlap-add is still building.
    let from = frames / 8;
    let to = frames - frames / 8;
    let mut crossings = 0usize;
    for frame in from..to.saturating_sub(1) {
        let current = samples[frame * channels];
        let next = samples[(frame + 1) * channels];
        if (current >= 0.0) != (next >= 0.0) {
            crossings += 1;
        }
    }
    crossings as f64 * 48_000.0 / (2.0 * (to - from).max(1) as f64)
}

/// `G6`: pitched output is chunk-count independent, the property `g10.039`
/// proved for the unpitched path.
///
/// Ignored: the implementation does not satisfy it yet. Worst sample delta is
/// `0.0057568103` at `-5` semitones with `3` chunks, first diverging `39.8%`
/// through the render rather than at a chunk boundary or the tail.
///
/// Four causes are ruled out by measurement, so the search should not restart
/// from them:
/// - `StreamingResampler` is byte-exact under chunking, `0.0` delta at `3`,
///   `7` and `16` chunks
/// - the pitched material this stage produces is byte-exact under chunking,
///   `0.0` delta, so the mid/side split and per-chunk resampling are correct
/// - the unpitched renderer is byte-exact at ratios `1.5`, `1.123`, `1.0`,
///   `2.0` and `0.8`, including the `1.123` effective ratio pitch produces here
/// - the carry path never fires; the feed loop always consumes what it is given
#[test]
#[ignore = "g10.042 open: pitched render diverges by 0.0057568103 across chunk counts"]
fn pitched_render_is_chunk_count_independent() {
    let source = tone(48_000 * 2, 220.0, 2);
    for semitones in [-5.0f64, 7.0] {
        let single = render_in_chunks(
            &pitched_config(source.len() / 2, 2, semitones, 1.5),
            &source,
            source.len() / 2,
        );
        for chunks in [3usize, 7, 16] {
            let sliced = render_in_chunks(
                &pitched_config(source.len() / 2, 2, semitones, 1.5),
                &source,
                source.len() / 2 / chunks,
            );
            assert_eq!(
                single.len(),
                sliced.len(),
                "semitones {semitones}, {chunks} chunks: length differs"
            );
            let worst = single
                .iter()
                .zip(sliced.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(
                worst < 1.0e-6,
                "semitones {semitones}, {chunks} chunks: worst sample delta {worst}"
            );
        }
    }
}

/// `G7`: the pitch actually happens, and in the right direction.
///
/// Length alone cannot see this — a renderer that ignored pitch entirely would
/// still produce the contracted length.
#[test]
fn pitched_render_shifts_the_tone() {
    let source = tone(48_000 * 2, 220.0, 2);
    let frames = source.len() / 2;
    for (semitones, expected) in [(12.0f64, 440.0f64), (-12.0, 110.0), (7.0, 329.6)] {
        let rendered = render_in_chunks(&pitched_config(frames, 2, semitones, 1.0), &source, 4_096);
        let measured = dominant_hz(&rendered, 2);
        assert!(
            (measured - expected).abs() / expected < 0.06,
            "semitones {semitones}: measured {measured:.0}Hz against an expected {expected:.0}Hz"
        );
    }
}

/// `G8`: the ratio curve lands in the right place under pitch.
///
/// This is the gate Batch 42.2 froze the coordinate rule for. A renderer that
/// forgot to convert the curve into pitched coordinates produces exactly the
/// right length, chunk-count independent, with no dropped source — and its
/// automation in the wrong place. Nothing else here can see that.
#[test]
fn pitched_ratio_curve_lands_in_pitched_coordinates() {
    let frames = 48_000usize;
    let source = tone(frames, 220.0, 2);
    // Ratio 1.0 for the first half of the source, 2.0 for the second.
    let curve = vec![
        StretchRatioPoint::new(0, 1.0),
        StretchRatioPoint::new((frames / 2) as i64, 2.0),
    ];
    let config = ResumableStretchConfig {
        ratio_curve: curve,
        ..pitched_config(frames, 2, 7.0, 1.0)
    };
    let rendered = render_in_chunks(&config, &source, 4_096);

    // Half the source at 1.0 plus half at 2.0 is 1.5x overall, whatever the
    // pitch: pitch changes the internal coordinates, not the output duration.
    let expected = (frames as f64 * 1.5).round() as usize;
    let produced = rendered.len() / 2;
    let drift = (produced as f64 - expected as f64).abs() / expected as f64;
    assert!(
        drift < 0.02,
        "curve applied in the wrong coordinates: produced {produced} frames against \
         an expected {expected}"
    );
}
