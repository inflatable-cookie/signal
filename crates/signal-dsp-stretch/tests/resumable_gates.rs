//! Structural gates for the `g10.039` resumable renderer.
//!
//! Four of these are the candidate's acceptance targets and currently fail;
//! they are `#[ignore]`d so `main` stays green while the owners exist, exactly
//! as the `g10.036` pre-fix owners were. Run them with
//! `cargo test -p signal-dsp-stretch --test resumable_gates -- --ignored`.
//!
//! The candidate lives on `main` by explicit operator decision, waiving
//! Contract `084` Rule 2 isolation for this lane.

use signal_dsp_stretch::{
    ResumableOfflineStretch, ResumableStretchConfig, StretchRatioPoint,
    MAX_RESUMABLE_WORKING_BYTES,
};

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
    }
}

/// G1: output must be identical for any chunk partition.
#[test]
#[ignore = "g10.039 Batch 39.3 acceptance target: fails at chunk 1024"]
fn chunk_size_independence_static_ratio() {
    let frames = 48_000 * 3;
    let source = material(frames, 1);
    let config = base_config(frames, 1);
    let whole = render_in_chunks(&config, &source, frames);
    for chunk in [1_024_usize, 7_777, 48_000, 100_000] {
        let chunked = render_in_chunks(&config, &source, chunk);
        assert_eq!(
            chunked.len(),
            whole.len(),
            "chunk {chunk}: length differs"
        );
        assert_eq!(chunked, whole, "chunk {chunk}: output differs");
    }
    println!("G1 static ratio: identical across 4 partitions, {} frames", whole.len());
}

/// G1b: same law with a dynamic ratio curve.
#[test]
#[ignore = "g10.039 Batch 39.3 acceptance target: fails at chunk 2048"]
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
        assert_eq!(chunked, whole, "chunk {chunk}: dynamic-ratio output differs");
    }
    println!("G1b dynamic ratio: identical across 3 partitions, {} frames", whole.len() / 2);
}

/// G2: working state is bounded by geometry, not source duration.
#[test]
#[ignore = "g10.039 Batch 39.3 acceptance target: 11665468 B against the 8388608 B ceiling"]
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
        assert_eq!(out.len(), target, "ratio {ratio}: length differs from target");
        assert_eq!(target, (frames as f64 * ratio).round() as usize);
    }
    println!("G3: output length matches target at 4 ratios");
}

/// G4: the acceptance target inherited from `g10.036`. A segmented render must
/// correlate with a whole render at the same constant ratio.
#[test]
#[ignore = "g10.039 Batch 39.3 acceptance target: correlation -0.082711 against 0.99"]
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
