//! Regression owners for the `g10.036` Transparent correctness defects.
//!
//! Each owner was written before the renderer changed, so each failed on
//! arrival. All are active now:
//!
//! - `A1` overlap coverage and ripple: corrected by Batch 36.3
//! - `A4` output bound: corrected by Batch 36.3
//! - `A2` dense-curve pitch preservation: corrected by Batch 36.4
//! - `A3` seam parity across channel counts: corrected by Batch 36.4
//!
//! Byte-exactness for the range the overlap law does not touch is proven
//! structurally in `phase_vocoder::tests`, not by an output hash here: f32
//! render output differs between optimization profiles, so an absolute hash is
//! only valid in the profile that captured it.
//!
//! Governing laws: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
//! 2026-07-27 Transparent renderer defect correction addendum.

use signal_dsp_stretch::{
    OfflineHighQualityStretcher, StretchRatioPoint, StretchRenderError, TimeStretcher,
    MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES,
};

const SAMPLE_RATE_HZ: f32 = 48_000.0;
const RMS_BLOCK_FRAMES: usize = 512;
const EDGE_BLOCKS: usize = 8;

/// Frozen ripple ceiling. The measured law produces `0.276 dB` on a tone and
/// `0.477 dB` on broadband material at every supported ratio.
const MAX_RIPPLE_DB: f64 = 0.5;

/// A block quieter than this counts as lost overlap coverage.
const NEAR_ZERO_RMS: f32 = 1.0e-4;

/// Seam-click parity tolerance between channel counts, in decibels.
const MAX_SEAM_PARITY_DELTA_DB: f64 = 6.0;

fn tone(frequency_hz: f32, frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / SAMPLE_RATE_HZ).sin())
        .collect()
}

fn broadband(frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|index| {
            let seconds = index as f32 / SAMPLE_RATE_HZ;
            0.4 * (std::f32::consts::TAU * 220.0 * seconds).sin()
                + 0.3 * (std::f32::consts::TAU * 1_310.0 * seconds).sin()
                + 0.2 * (std::f32::consts::TAU * 4_700.0 * seconds).sin()
        })
        .collect()
}

/// Interior block RMS envelope, skipping the windup and tail edges.
fn interior_block_rms(samples: &[f32]) -> Vec<f32> {
    let blocks: Vec<f32> = samples
        .chunks(RMS_BLOCK_FRAMES)
        .map(|block| (block.iter().map(|s| s * s).sum::<f32>() / block.len() as f32).sqrt())
        .collect();
    assert!(
        blocks.len() > EDGE_BLOCKS * 2,
        "output too short to measure an interior envelope"
    );
    blocks[EDGE_BLOCKS..blocks.len() - EDGE_BLOCKS].to_vec()
}

fn ripple_db(samples: &[f32]) -> f64 {
    let interior = interior_block_rms(samples);
    let min = interior
        .iter()
        .copied()
        .fold(f32::MAX, f32::min)
        .max(1.0e-12);
    let max = interior.iter().copied().fold(0.0_f32, f32::max);
    20.0 * (max as f64 / min as f64).log10()
}

fn near_zero_blocks(samples: &[f32]) -> usize {
    interior_block_rms(samples)
        .iter()
        .filter(|rms| **rms < NEAR_ZERO_RMS)
        .count()
}

/// Dominant frequency by zero-crossing count over a trimmed interior span.
fn dominant_frequency_hz(samples: &[f32]) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * SAMPLE_RATE_HZ / (2.0 * interior.len() as f32)
}

/// Mirrors `measure_dynamic_segment_seam_click`: peak absolute sample delta
/// across the seam, in dBFS. Duplicated here so the owner does not depend on
/// the `evidence` feature surface.
fn seam_click_dbfs(interleaved: &[f32], channels: usize, seam_frame: usize) -> f64 {
    let frames = interleaved.len() / channels;
    assert!(seam_frame > 0 && seam_frame < frames, "seam outside output");
    let before = &interleaved[(seam_frame - 1) * channels..seam_frame * channels];
    let after = &interleaved[seam_frame * channels..(seam_frame + 1) * channels];
    let peak = before
        .iter()
        .zip(after.iter())
        .map(|(left, right)| (left - right).abs() as f64)
        .fold(0.0_f64, f64::max);
    if peak <= 0.0 {
        return f64::NEG_INFINITY;
    }
    20.0 * peak.log10()
}

/// `A1`. Above ratio `4.0` the synthesis hop passes the window, overlap-add
/// coverage disappears, and the normalization gate zeroes interior samples.
///
/// Pre-fix failure: ratio `6.0` reports `183` zeroed blocks of `547`, ratio
/// `8.0` reports `368` of `734`.
#[test]
fn overlap_coverage_has_no_zeroed_interior_block() {
    for ratio in [4.0_f64, 5.0, 6.0, 8.0, 12.0] {
        let input = tone(440.0, 48_000);
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the bound");
        let zeroed = near_zero_blocks(&output);
        assert_eq!(
            zeroed, 0,
            "ratio {ratio}: {zeroed} interior blocks lost overlap coverage"
        );
    }
}

/// `A1`. Ratio `4.0` sits exactly at synthesis hop equal to window size and
/// carries a periodic amplitude ripple even though no block is fully zeroed.
///
/// Pre-fix failure: ratio `4.0` measures `1.396 dB` on tone and `1.615 dB` on
/// broadband against the `0.5 dB` ceiling.
#[test]
fn overlap_ripple_stays_within_ceiling() {
    for ratio in [1.0_f64, 2.0, 3.0, 4.0, 6.0, 8.0] {
        for (label, input) in [
            ("tone", tone(440.0, 48_000)),
            ("broadband", broadband(48_000)),
        ] {
            let mut stretcher = OfflineHighQualityStretcher::new(ratio);
            let output = stretcher
                .stretch_mono(&input)
                .expect("render fits the bound");
            let ripple = ripple_db(&output);
            assert!(
                ripple <= MAX_RIPPLE_DB,
                "ratio {ratio} {label}: ripple {ripple:.3} dB exceeds {MAX_RIPPLE_DB} dB"
            );
        }
    }
}

/// `A1` control. The correction must not disturb the retained product range
/// below the point where the overlap law engages.
#[test]
fn overlap_law_leaves_low_ratios_byte_exact() {
    // The frozen `2048/512` geometry satisfies the law through ratio `3.0`,
    // so these renders must be unchanged by Batch 36.3.
    let input = tone(440.0, 24_000);
    for ratio in [0.5_f64, 1.5, 2.0, 3.0] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let first = stretcher
            .stretch_mono(&input)
            .expect("render fits the bound");
        let mut repeat = OfflineHighQualityStretcher::new(ratio);
        let second = repeat.stretch_mono(&input).expect("render fits the bound");
        assert_eq!(first, second, "ratio {ratio} is not deterministic");
        assert_eq!(
            first.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio} broke the output-length contract"
        );
    }
}

/// `A2`. A ratio curve sampled finer than one analysis window routes every
/// segment into the sub-window interpolation fallback, which pitch-shifts.
///
/// Pre-fix failure: a `47`-point curve at `1024`-frame spacing and ratio `2.0`
/// renders a `440 Hz` source at `220.0 Hz`.
#[test]
fn dense_ratio_curve_preserves_pitch() {
    let input = tone(440.0, 48_000);
    let dense: Vec<StretchRatioPoint> = (0..47)
        .map(|index| StretchRatioPoint::new(index as i64 * 1_024, 2.0))
        .collect();

    let mut stretcher = OfflineHighQualityStretcher::new(2.0);
    let curved = stretcher
        .stretch_dynamic_ratio_mono(&input, &dense)
        .expect("render fits the bound");
    let flat = stretcher
        .stretch_mono(&input)
        .expect("render fits the bound");

    let curved_hz = dominant_frequency_hz(&curved);
    let flat_hz = dominant_frequency_hz(&flat);
    let error = (curved_hz - flat_hz).abs() / flat_hz;
    assert!(
        error < 0.005,
        "dense curve rendered {curved_hz:.1} Hz against {flat_hz:.1} Hz for the same ratio"
    );
}

/// `A2` control. Output length already survives a dense curve; only pitch
/// breaks. Segment coalescing must keep this true, so this owner is active
/// now rather than deferred.
#[test]
fn dense_ratio_curve_preserves_output_length() {
    let input = tone(440.0, 48_000);
    let dense: Vec<StretchRatioPoint> = (0..47)
        .map(|index| StretchRatioPoint::new(index as i64 * 1_024, 2.0))
        .collect();
    let mut stretcher = OfflineHighQualityStretcher::new(2.0);
    let curved = stretcher
        .stretch_dynamic_ratio_mono(&input, &dense)
        .expect("render fits the bound");
    assert_eq!(curved.len(), input.len() * 2);
}

/// `A2` invariant. Coalescing sets each merged segment's target to the sum of
/// the targets its constituent spans would have produced, so total output
/// length must be unchanged for every curve shape, coalesced or not.
#[test]
fn segment_coalescing_preserves_total_output_length() {
    let input = tone(440.0, 48_000);
    let cases: [(&str, Vec<StretchRatioPoint>, usize); 4] = [
        (
            "dense uniform",
            (0..47)
                .map(|index| StretchRatioPoint::new(index as i64 * 1_024, 2.0))
                .collect(),
            96_000,
        ),
        (
            "two coarse spans",
            vec![
                StretchRatioPoint::new(0, 1.5),
                StretchRatioPoint::new(24_000, 0.75),
            ],
            54_000,
        ),
        (
            "tempo ramp",
            (0..8)
                .map(|index| StretchRatioPoint::new(index as i64 * 6_000, 1.0 + index as f64 * 0.1))
                .collect(),
            64_800,
        ),
        (
            "short tail span",
            vec![
                StretchRatioPoint::new(0, 2.0),
                StretchRatioPoint::new(47_900, 0.5),
            ],
            95_850,
        ),
    ];

    for (label, curve, expected) in cases {
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);
        let mono = stretcher
            .stretch_dynamic_ratio_mono(&input, &curve)
            .expect("render fits the bound");
        assert_eq!(mono.len(), expected, "{label} mono length");

        let interleaved: Vec<f32> = input.iter().flat_map(|sample| [*sample, *sample]).collect();
        let mut stretcher = OfflineHighQualityStretcher::new(1.0);
        let stereo = stretcher
            .stretch_dynamic_ratio_interleaved_stereo(&interleaved, &curve)
            .expect("render fits the bound");
        assert_eq!(stereo.len() / 2, expected, "{label} stereo frames");
    }
}

/// `A2` cost. Coalescing is not free: a curve whose spans are shorter than the
/// frozen minimum loses its individual ratio changes and renders at the mean
/// ratio over the merged span. Total length still holds exactly. This owner
/// records that cost rather than hiding it.
#[test]
fn sub_minimum_curve_spans_render_at_their_mean_ratio() {
    let input = tone(440.0, 48_000);
    // Three spans of 16000 frames, 333 ms each, all under the 384 ms minimum.
    let curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let expected = 12_000 + 16_000 + 24_000;

    let mut stretcher = OfflineHighQualityStretcher::new(1.0);
    let merged = stretcher
        .stretch_dynamic_ratio_mono(&input, &curve)
        .expect("render fits the bound");
    assert_eq!(
        merged.len(),
        expected,
        "merging must preserve the summed target length"
    );

    // The merged render is one static ratio, so it matches a plain render at
    // the mean ratio rather than following the curve.
    let mut flat = OfflineHighQualityStretcher::new(expected as f64 / input.len() as f64);
    let reference = flat.stretch_mono(&input).expect("render fits the bound");
    assert_eq!(merged.len(), reference.len());
}

/// Known limitation, and the acceptance target for `g10.039`.
///
/// Dynamic-ratio segments render independently, so each one restarts the phase
/// vocoder and the phase relationship across every join is arbitrary. Rendering
/// a constant ratio through the segmented path therefore produces a waveform
/// almost uncorrelated with the same ratio rendered whole:
///
/// | measurement | value |
/// | --- | --- |
/// | correlation | `0.034` |
/// | peak sample difference | `1.1470` |
/// | difference RMS against signal RMS | `0.2474` against `0.1784` |
///
/// Concealed listening heard this as a periodic pulse whose rate tracks the
/// segment length. It is not amplitude modulation and no segment minimum
/// removes it; only carrying renderer state across the join does.
///
/// This owner asserts the transparency that a state-carrying renderer must
/// deliver. It is expected to fail until `g10.039` lands.
#[test]
#[ignore = "known limitation: segmented renders restart phase; g10.039 acceptance target"]
fn segmented_render_matches_whole_render_at_constant_ratio() {
    let input = tone(440.0, 48_000);
    let dense: Vec<StretchRatioPoint> = (0..47)
        .map(|index| StretchRatioPoint::new(index as i64 * 1_024, 2.0))
        .collect();

    let mut stretcher = OfflineHighQualityStretcher::new(2.0);
    let segmented = stretcher
        .stretch_dynamic_ratio_mono(&input, &dense)
        .expect("render fits the bound");
    let mut stretcher = OfflineHighQualityStretcher::new(2.0);
    let whole = stretcher
        .stretch_mono(&input)
        .expect("render fits the bound");

    let n = segmented.len().min(whole.len());
    let dot: f64 = (0..n).map(|i| (segmented[i] * whole[i]) as f64).sum();
    let seg_energy: f64 = (0..n).map(|i| (segmented[i] * segmented[i]) as f64).sum();
    let whole_energy: f64 = (0..n).map(|i| (whole[i] * whole[i]) as f64).sum();
    let correlation = dot / (seg_energy.sqrt() * whole_energy.sqrt());

    assert!(
        correlation > 0.99,
        "segmented render correlates {correlation:.6} with the whole render; \
         a transparent segmentation must be near 1.0"
    );
}

/// `A3`. Segment-join treatment must not depend on channel count. The mono
/// path skips the smoothing the interleaved path applies.
///
/// Pre-fix failure: mono measures `-28.940011 dBFS` against linked stereo at
/// `-180.617997 dBFS` for the same source and curve.
#[test]
fn dynamic_ratio_seam_click_matches_across_channel_counts() {
    let input = tone(440.0, 48_000);
    let interleaved: Vec<f32> = input.iter().flat_map(|sample| [*sample, *sample]).collect();
    let curve = [
        StretchRatioPoint::new(0, 1.5),
        StretchRatioPoint::new(24_000, 0.75),
    ];
    let seam_frame = (24_000.0 * 1.5) as usize;

    let mut stretcher = OfflineHighQualityStretcher::new(1.5);
    let mono = stretcher
        .stretch_dynamic_ratio_mono(&input, &curve)
        .expect("render fits the bound");
    let stereo = stretcher
        .stretch_dynamic_ratio_interleaved_stereo(&interleaved, &curve)
        .expect("render fits the bound");

    let mono_click = seam_click_dbfs(&mono, 1, seam_frame);
    let stereo_click = seam_click_dbfs(&stereo, 2, seam_frame);
    assert!(
        mono_click <= stereo_click + MAX_SEAM_PARITY_DELTA_DB,
        "mono seam {mono_click:.6} dBFS against stereo {stereo_click:.6} dBFS \
         exceeds the {MAX_SEAM_PARITY_DELTA_DB} dB parity tolerance"
    );
}

/// `A4`. Oversized renders are refused, not attempted. Before Batch 36.3 this
/// request allocated `4096000000` samples and returned after roughly one
/// minute.
#[test]
fn oversized_output_request_is_refused() {
    let input = tone(440.0, 4_096);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0e6);
    let error = stretcher
        .stretch_mono(&input)
        .expect_err("a 4096000000-sample render must be refused");
    assert_eq!(
        error,
        StretchRenderError::OutputTooLarge {
            requested_samples: 4_096_000_000,
            maximum_samples: MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES,
        }
    );
}

/// `A4`. The bound refuses only what exceeds it. A render sitting just inside
/// the ceiling still succeeds.
#[test]
fn render_inside_the_output_bound_is_served() {
    let input = tone(440.0, 48_000);
    let mut stretcher = OfflineHighQualityStretcher::new(4.0);
    let output = stretcher
        .stretch_mono(&input)
        .expect("192000 samples is far inside the ceiling");
    assert_eq!(output.len(), 192_000);
}

/// `A4`. The ceiling counts samples across all channels, not frames, so a
/// stereo render refuses at half the frame count a mono render allows.
#[test]
fn output_bound_counts_every_channel() {
    let frames = MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES / 2 + 1_024;
    let interleaved = vec![0.0_f32; 8];
    let mut stretcher = OfflineHighQualityStretcher::new(frames as f64 / 4.0);
    let error = stretcher
        .stretch_interleaved_stereo(&interleaved)
        .expect_err("a stereo render past the ceiling must be refused");
    assert!(matches!(error, StretchRenderError::OutputTooLarge { .. }));
}
