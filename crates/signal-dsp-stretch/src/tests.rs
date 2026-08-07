//! Unit tests for signal-dsp-stretch.

use super::*;
use signal_primitives::SampleRate;

fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
    (0..len)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin())
        .collect()
}

/// Dominant frequency estimate by zero-crossing count over a trimmed
/// interior span (skips windup/tail edges).
fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
}

fn rms(samples: &[Sample]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
    let mut input = vec![0.0; len];
    input[..edge_frames].fill(0.5);
    input[len - edge_frames..].fill(-0.5);
    input
}

fn add_decaying_burst(samples: &mut [Sample], start: usize, frames: usize, amplitude: f32) {
    for offset in 0..frames {
        let Some(sample) = samples.get_mut(start + offset) else {
            break;
        };
        let envelope = 1.0 - offset as f32 / frames as f32;
        let polarity = if offset % 2 == 0 { 1.0 } else { -1.0 };
        *sample += amplitude * envelope * polarity;
    }
}

fn masked_soft_attack_probe(soft_attack_amplitude: f32) -> Vec<Sample> {
    let mut input = sine(180.0, 48_000.0, 48_000)
        .into_iter()
        .map(|sample| sample * 0.06)
        .collect::<Vec<_>>();
    add_decaying_burst(&mut input, 8_000, 96, 1.0);
    add_decaying_burst(&mut input, 24_000, 96, soft_attack_amplitude);
    input
}

#[test]
fn identity_ratio_is_passthrough() {
    let input = sine(440.0, 48_000.0, 10_000);
    let mut stretcher = PhaseVocoderStretcher::new(1.0);
    assert_eq!(
        stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        input
    );
}

#[test]
fn ratio_clamps_invalid_values_to_identity() {
    let mut stretcher = PhaseVocoderStretcher::new(f64::NAN);
    assert_eq!(stretcher.ratio(), 1.0);
    stretcher.set_ratio(-2.0);
    assert_eq!(stretcher.ratio(), 1.0);
    stretcher.set_ratio(1.5);
    assert_eq!(stretcher.ratio(), 1.5);
}

#[test]
fn stretch_honors_output_length_contract() {
    let input = sine(440.0, 48_000.0, 48_000);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut stretcher = PhaseVocoderStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        assert_eq!(
            output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}"
        );
    }
}

#[test]
fn offline_high_quality_reports_target_quality() {
    let stretcher = OfflineHighQualityStretcher::new(1.25);

    assert_eq!(stretcher.quality(), StretchQuality::OfflineHighQuality);
    assert_eq!(stretcher.ratio(), 1.25);
    assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
}

#[test]
fn offline_high_quality_path_can_be_selected_explicitly() {
    let mut stretcher = OfflineHighQualityStretcher::with_path(
        0.75,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );

    assert_eq!(
        stretcher.path(),
        OfflineHighQualityPath::CompressionShortWindowSelector
    );
    stretcher.set_path(OfflineHighQualityPath::Default);
    assert_eq!(stretcher.path(), OfflineHighQualityPath::Default);
    stretcher.set_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
    assert_eq!(
        stretcher.path(),
        OfflineHighQualityPath::ExpansionShortWindowSelector
    );
}

#[test]
fn offline_high_quality_is_deterministic_and_honors_output_length() {
    let input = sine(440.0, 48_000.0, 48_000);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut first = OfflineHighQualityStretcher::new(ratio);
        let mut repeated = OfflineHighQualityStretcher::new(ratio);
        let first_output = first
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let repeated_output = repeated
            .stretch_mono(&input)
            .expect("render fits the offline output bound");

        assert_eq!(
            first_output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}"
        );
        assert_eq!(first_output, repeated_output, "ratio {ratio}");
    }
}

#[test]
fn offline_high_quality_boundary_preserves_endpoint_content() {
    let input = boundary_content_probe(48_000, 384);
    for ratio in [0.5, 2.0] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let edge_span = 2_048.min(output.len());

        assert_eq!(output.len(), (input.len() as f64 * ratio).round() as usize);
        assert!(
            rms(&output[..edge_span]) > 0.01,
            "ratio {ratio}: silent head"
        );
        assert!(
            rms(&output[output.len() - edge_span..]) > 0.01,
            "ratio {ratio}: silent tail"
        );
    }
}

#[test]
fn compression_short_window_selector_matches_gate_decision() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 0.75;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut short_window = OfflineHighQualityStretcher::with_window(
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    let short_window_output = short_window
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );
    let selector_output = selector
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let default_smear = measure_transient_smear(
        &input,
        &default_output,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        StretchTransientSmearPolicies::production(),
    );
    let accepted = default_smear.missed_transients
        >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
        || default_smear.max_smear_frames
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES;

    let expected = if accepted {
        &short_window_output
    } else {
        &default_output
    };
    assert_eq!(selector_output, *expected);
    assert_eq!(
        selector_output.len(),
        (input.len() as f64 * ratio).round() as usize
    );
}

#[test]
fn compression_short_window_selector_does_not_switch_expansion_ratios() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::CompressionShortWindowSelector,
    );

    assert_eq!(
        selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        default_output
    );
}

#[test]
fn expansion_short_window_selector_matches_gate_decision() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut short_window = OfflineHighQualityStretcher::with_window(
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    let short_window_output = short_window
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::ExpansionShortWindowSelector,
    );
    let selector_output = selector
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let accepted = should_select_expansion_short_window(&input, &default_output, ratio);

    let expected = if accepted {
        &short_window_output
    } else {
        &default_output
    };
    assert_eq!(selector_output, *expected);
    assert_eq!(
        selector_output.len(),
        (input.len() as f64 * ratio).round() as usize
    );
}

#[test]
fn expansion_short_window_selector_rejects_compression_ratios() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 0.75;
    let mut default = OfflineHighQualityStretcher::new(ratio);
    let default_output = default
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    let mut selector = OfflineHighQualityStretcher::with_path(
        ratio,
        OfflineHighQualityPath::ExpansionShortWindowSelector,
    );

    assert_eq!(
        selector
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        default_output
    );
}

#[test]
fn expansion_short_window_gate_accepts_current_misses() {
    let input = masked_soft_attack_probe(0.35);
    let ratio = 1.25;
    let silent_current = vec![0.0; (input.len() as f64 * ratio).round() as usize];

    assert!(should_select_expansion_short_window(
        &input,
        &silent_current,
        ratio
    ));
    assert!(!should_select_expansion_short_window(
        &input,
        &silent_current,
        0.75
    ));
}

#[test]
fn offline_high_quality_identity_ratio_is_passthrough() {
    let input = sine(330.0, 48_000.0, 8_192);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    assert_eq!(
        stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound"),
        input
    );
}

#[test]
fn stretch_preserves_pitch_within_tolerance() {
    let sample_rate = 48_000.0;
    let input = sine(440.0, sample_rate, 48_000);
    for ratio in [0.75, 1.5, 2.0] {
        let mut stretcher = PhaseVocoderStretcher::new(ratio);
        let output = stretcher
            .stretch_mono(&input)
            .expect("render fits the offline output bound");
        let frequency = dominant_frequency_hz(&output, sample_rate);
        assert!(
            (frequency - 440.0).abs() < 15.0,
            "ratio {ratio}: dominant frequency {frequency} Hz, expected ~440 Hz"
        );
        assert!(
            rms(&output) > 0.3,
            "ratio {ratio}: stretched output lost energy (rms {})",
            rms(&output)
        );
    }
}

#[test]
fn sub_window_input_scales_by_linear_fallback() {
    let input: Vec<f32> = (0..100).map(|index| index as f32 / 100.0).collect();
    let mut stretcher = PhaseVocoderStretcher::new(2.0);
    let output = stretcher
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    assert_eq!(output.len(), 200);
    // Monotone ramp stays monotone under linear scaling.
    assert!(output.windows(2).all(|pair| pair[1] >= pair[0] - 1.0e-6));
}

#[test]
fn offline_high_quality_linked_stereo_honors_output_length_contract() {
    let sample_rate = 48_000.0;
    let left = sine(440.0, sample_rate, 48_000);
    let right = sine(660.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound");

        assert_eq!(
            output.len(),
            ((left.len() as f64 * ratio).round() as usize) * 2,
            "ratio {ratio}"
        );
    }
}

#[test]
fn offline_high_quality_linked_stereo_is_identity_passthrough() {
    let frames = [0.0, 0.1, 0.2, 0.3, 0.4];
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    assert_eq!(
        stretcher
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound"),
        frames[..4]
    );
}

#[test]
fn offline_high_quality_linked_stereo_is_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(330.0, sample_rate, 48_000);
    let right = sine(550.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    let mut first = OfflineHighQualityStretcher::new(1.5);
    let mut repeated = OfflineHighQualityStretcher::new(1.5);

    assert_eq!(
        first
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound"),
        repeated
            .stretch_interleaved_stereo(&frames)
            .expect("render fits the offline output bound")
    );
}

#[test]
fn offline_high_quality_pitch_shift_preserves_tempo_length_contract() {
    let input = sine(440.0, 48_000.0, 48_000);
    for (ratio, semitones) in [(1.0, 12.0), (1.5, -7.0), (0.75, 5.0)] {
        let mut stretcher = OfflineHighQualityStretcher::new(ratio);
        let output = stretcher
            .stretch_pitch_mono(&input, SampleRate(48_000), semitones)
            .expect("render fits the offline output bound");

        assert_eq!(
            output.len(),
            (input.len() as f64 * ratio).round() as usize,
            "ratio {ratio}, semitones {semitones}"
        );
    }
}

#[test]
fn offline_high_quality_pitch_shift_raises_tonal_pitch() {
    let sample_rate = 48_000.0;
    let input = sine(440.0, sample_rate, 48_000);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);

    let output = stretcher
        .stretch_pitch_mono(&input, SampleRate(48_000), 12.0)
        .expect("render fits the offline output bound");
    let frequency = dominant_frequency_hz(&output, sample_rate);

    assert_eq!(output.len(), input.len());
    assert!(
        (frequency - 880.0).abs() < 35.0,
        "expected pitch near 880 Hz, got {frequency} Hz"
    );
}

#[test]
fn offline_high_quality_pitch_shift_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }

    let mut first = OfflineHighQualityStretcher::new(1.25);
    let mut repeated = OfflineHighQualityStretcher::new(1.25);
    let first_output = first
        .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_pitch_interleaved_stereo(&frames, SampleRate(48_000), -5.0)
        .expect("render fits the offline output bound");

    assert_eq!(first_output.len(), (48_000f64 * 1.25).round() as usize * 2);
    assert_eq!(first_output, repeated_output);
}

#[test]
fn offline_high_quality_dynamic_ratio_mono_sums_segment_targets() {
    let input = sine(440.0, 48_000.0, 48_000);
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);
    let output = stretcher
        .stretch_dynamic_ratio_mono(&input, &ratio_curve)
        .expect("render fits the offline output bound");

    assert_eq!(
        output.len(),
        dynamic_ratio_output_frames(input.len(), &ratio_curve, 1.0)
    );
    assert_eq!(output.len(), 52_000);
}

#[test]
fn offline_high_quality_dynamic_ratio_ignores_invalid_points() {
    let input = sine(440.0, 48_000.0, 8_000);
    let ratio_curve = [
        StretchRatioPoint::new(-128, 0.5),
        StretchRatioPoint::new(2_000, f64::NAN),
        StretchRatioPoint::new(4_000, -2.0),
    ];
    let mut dynamic = OfflineHighQualityStretcher::new(1.25);
    let mut fixed = OfflineHighQualityStretcher::new(1.25);

    // Invalid points are ignored, so this must render as the stretcher's
    // own static ratio. Compared through the same renderer with an empty
    // curve, because the invariant is about the curve, not about which
    // renderer runs.
    let curved = dynamic
        .stretch_dynamic_ratio_mono(&input, &ratio_curve)
        .expect("render fits the offline output bound");
    let empty_curve = dynamic
        .stretch_dynamic_ratio_mono(&input, &[])
        .expect("render fits the offline output bound");
    assert_eq!(curved, empty_curve);

    // `stretch_dynamic_ratio_mono` renders resumably and `stretch_mono` in
    // one shot, so they are close rather than identical at a static ratio.
    // Recorded as a bound because it is a real consequence of the dynamic
    // API moving to the resumable renderer: same length, same algorithm,
    // different state handling.
    let flat = fixed
        .stretch_mono(&input)
        .expect("render fits the offline output bound");
    assert_eq!(curved.len(), flat.len());
    let worst = curved
        .iter()
        .zip(flat.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1.0e-4,
        "resumable and one-shot static renders drifted apart: {worst}",
    );
}

#[test]
fn offline_high_quality_dynamic_ratio_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut first = OfflineHighQualityStretcher::new(1.0);
    let mut repeated = OfflineHighQualityStretcher::new(1.0);
    let first_output = first
        .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_dynamic_ratio_interleaved_stereo(&frames, &ratio_curve)
        .expect("render fits the offline output bound");

    assert_eq!(
        first_output.len(),
        dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
    );
    assert_eq!(first_output, repeated_output);
}

#[test]
fn dynamic_segment_seam_smoothing_is_not_neutral_on_continuous_material() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    // Explicit boundaries: this owner tests the smoother, not the
    // segmentation law. Deriving them from a ratio curve coupled it to the
    // Contract `046` minimum segment length, and it broke when that
    // minimum grew past the curve's span length.
    let boundaries = vec![12_000, 28_000];
    let mut raw = frames.clone();
    let before = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);
    smooth_dynamic_segment_boundaries_interleaved(&mut raw, 2, &boundaries, 64);
    let after = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);

    // Continuous material with no join: the smoother has nothing to fix,
    // and drags 64 frames either side of each nominated frame toward the
    // midpoint of the pair it straddles. That is a discontinuity it
    // introduces, not one it removes. Measured -240 dBFS (nothing) before,
    // -70.9 dBFS after.
    assert!(
        before.click_dbfs <= -240.0,
        "clean sines should show no seam, got {:.2} dBFS",
        before.click_dbfs,
    );
    assert!(
        after.click_dbfs > before.click_dbfs + 100.0,
        "smoothing continuous material should introduce a measurable \
         discontinuity, got {:.2} dBFS",
        after.click_dbfs,
    );
    assert_eq!(raw.len(), frames.len());
}

#[test]
fn offline_high_quality_dynamic_ratio_pitch_stereo_is_exact_and_deterministic() {
    let sample_rate = 48_000.0;
    let left = sine(220.0, sample_rate, 48_000);
    let right = sine(440.0, sample_rate, 48_000);
    let mut frames = Vec::with_capacity(left.len() * 2);
    for (l, r) in left.iter().zip(right.iter()) {
        frames.push(*l);
        frames.push(*r);
    }
    let ratio_curve = [
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new(16_000, 1.0),
        StretchRatioPoint::new(32_000, 1.5),
    ];
    let mut first = OfflineHighQualityStretcher::new(1.0);
    let mut repeated = OfflineHighQualityStretcher::new(1.0);
    let first_output = first
        .stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        )
        .expect("render fits the offline output bound");
    let repeated_output = repeated
        .stretch_dynamic_ratio_pitch_interleaved_stereo(
            &frames,
            &ratio_curve,
            SampleRate(48_000),
            2.0,
        )
        .expect("render fits the offline output bound");

    assert_eq!(
        first_output.len(),
        dynamic_ratio_output_frames(left.len(), &ratio_curve, 1.0) * 2
    );
    assert_eq!(first_output, repeated_output);
}

#[test]
fn backend_plan_tracks_signal_owned_tiers() {
    assert_eq!(SIGNAL_STRETCH_BACKEND_PLAN.len(), 3);
    assert_eq!(
        stretch_backend_plan(StretchBackendTier::Repitch).status,
        StretchBackendStatus::Implemented
    );
    let preview = stretch_backend_plan(StretchBackendTier::RealtimePreview);
    assert_eq!(preview.status, StretchBackendStatus::Prototype);
    assert!(preview.independent_tempo_and_pitch);
    assert!(preview.dynamic_ratio);
    assert!(!preview.audio_thread_safe);

    let offline = stretch_backend_plan(StretchBackendTier::OfflineHighQuality);
    assert_eq!(offline.status, StretchBackendStatus::Implemented);
    assert!(offline.transient_preservation);
    assert!(offline.vertical_phase_coherence);
    assert!(offline.deterministic_output);
}

#[test]
fn benchmark_corpus_covers_required_material_families() {
    let required = [
        StretchCorpusFamily::DrumsPercussion,
        StretchCorpusFamily::Bass,
        StretchCorpusFamily::Vocals,
        StretchCorpusFamily::PadsSustains,
        StretchCorpusFamily::FullMix,
        StretchCorpusFamily::TempoRamp,
        StretchCorpusFamily::LoopSeam,
        StretchCorpusFamily::ExtremeRatio,
    ];

    for family in required {
        assert!(
            STRETCH_BENCHMARK_CORPUS
                .iter()
                .any(|case| case.family == family),
            "missing corpus family {family:?}"
        );
    }
    assert!(STRETCH_BENCHMARK_CORPUS.iter().all(|case| case
        .ratios
        .iter()
        .all(|ratio| ratio.is_finite() && *ratio > 0.0)));
}

#[test]
fn real_corpus_manifest_covers_required_families_and_source_policy() {
    assert_eq!(STRETCH_CORPUS_MANIFEST.manifest_id, "stretch-corpus-v1");
    assert_eq!(STRETCH_CORPUS_MANIFEST.schema_version, 1);
    assert_eq!(STRETCH_CORPUS_MANIFEST.sample_rate_hz, 48_000);
    assert_eq!(STRETCH_CORPUS_MANIFEST.channels, 2);
    assert_eq!(
        STRETCH_CORPUS_MANIFEST.source_policy,
        STRETCH_CORPUS_SOURCE_POLICY
    );
    assert_eq!(
        STRETCH_CORPUS_MANIFEST.entries.len(),
        STRETCH_BENCHMARK_CORPUS.len()
    );

    for benchmark_case in STRETCH_BENCHMARK_CORPUS {
        let manifest_entry = STRETCH_CORPUS_MANIFEST
            .entries
            .iter()
            .find(|entry| entry.case.case_id == benchmark_case.case_id)
            .expect("benchmark case should have manifest entry");
        assert_eq!(manifest_entry.case.family, benchmark_case.family);
        assert_eq!(manifest_entry.case.ratios, benchmark_case.ratios);
        assert!(!manifest_entry.source_path_hint.is_empty());
        assert!(!manifest_entry.provenance_note.is_empty());
    }
}

#[test]
fn real_corpus_manifest_keeps_licensed_audio_out_of_repo() {
    for entry in STRETCH_CORPUS_MANIFEST.entries {
        match entry.case.source {
            StretchCorpusSource::Synthetic => {
                assert_eq!(
                    entry.asset_requirement,
                    StretchCorpusAssetRequirement::InlineSynthetic
                );
                assert_eq!(
                    entry.missing_asset_behavior,
                    StretchCorpusMissingAssetBehavior::GenerateInlineSynthetic
                );
                assert!(entry.source_path_hint.starts_with("inline:"));
                assert!(generate_synthetic_stretch_audio(entry.case.family).is_some());
            }
            StretchCorpusSource::LicensedListening => {
                assert_eq!(
                    entry.asset_requirement,
                    StretchCorpusAssetRequirement::OperatorProvidedAudio
                );
                assert_eq!(
                    entry.missing_asset_behavior,
                    StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase
                );
                assert!(entry
                    .source_path_hint
                    .starts_with("fixtures/stretch-corpus/licensed-listening/"));
                assert!(entry.provenance_note.contains("licensed"));
            }
            StretchCorpusSource::ExternalBenchmark => {
                assert_eq!(
                    entry.asset_requirement,
                    StretchCorpusAssetRequirement::OptionalExternalBenchmark
                );
                assert_eq!(
                    entry.missing_asset_behavior,
                    StretchCorpusMissingAssetBehavior::SkipOptionalBenchmark
                );
            }
            StretchCorpusSource::LocalFixture => {
                panic!("stretch corpus v1 must not rely on checked-in licensed fixtures");
            }
        }
    }
    assert!(STRETCH_CORPUS_SOURCE_POLICY
        .licensed_audio_policy
        .contains("do not commit source audio"));
}

#[test]
fn output_length_drift_tracks_fixed_ratio_contract() {
    assert_eq!(output_length_drift_samples(1_000, 1_500, 1.5), 0.0);
    assert_eq!(output_length_drift_samples(1_001, 1_502, 1.5), 0.0);
    assert_eq!(output_length_drift_samples(1_001, 1_503, 1.5), 1.0);
    assert!(output_length_drift_samples(1_000, 1_000, f64::NAN).is_nan());
}

#[test]
fn metric_assessment_aggregates_warnings_and_failures() {
    let measurements = [
        StretchMetricValue::new(StretchMetric::TimingDriftSamples, 0.0),
        StretchMetricValue::new(StretchMetric::StereoImageDelta, 0.2),
        StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, -24.0),
    ];
    let limits = [
        StretchMetricLimit::max(
            StretchMetric::TimingDriftSamples,
            1.0,
            StretchAcceptanceSeverity::Fail,
        ),
        StretchMetricLimit::max(
            StretchMetric::StereoImageDelta,
            0.1,
            StretchAcceptanceSeverity::Warn,
        ),
        StretchMetricLimit::max(
            StretchMetric::LoopBoundaryClickDbfs,
            -60.0,
            StretchAcceptanceSeverity::Fail,
        ),
    ];

    let report = assess_stretch_metrics(&measurements, &limits);

    assert_eq!(report.status, StretchAcceptanceStatus::Fail);
    assert_eq!(report.metrics[0].status, StretchAcceptanceStatus::Pass);
    assert_eq!(report.metrics[1].status, StretchAcceptanceStatus::Warn);
    assert_eq!(report.metrics[2].status, StretchAcceptanceStatus::Fail);
}

#[test]
fn dynamic_segment_seam_metric_reports_excess_over_the_renders_own_floor() {
    // Too short to hold any frame outside a seam window: there is no way to
    // tell a seam from the waveform, so the answer is "unmeasurable", not
    // "clean". The predecessor of this measurement answered "clean".
    let tiny = [0.0, 0.0, 0.1, 0.2, 0.9, -0.4, 1.0, -0.3];
    assert!(measure_dynamic_segment_seam_click(&tiny, 2, &[2], 1.0)
        .click_dbfs
        .is_nan());

    // A long, smooth ramp with one injected step. The step is 0.5 against a
    // per-frame background of 0.0001, so it must read close to 0.5 rather
    // than to the raw first difference.
    let frame_count = 8_000usize;
    let mut frames = Vec::with_capacity(frame_count * 2);
    for index in 0..frame_count {
        let value = index as f32 * 0.0001;
        frames.push(value);
        frames.push(value);
    }
    for sample in frames[4_000 * 2..].iter_mut() {
        *sample += 0.5;
    }
    let measurement = measure_dynamic_segment_seam_click(&frames, 2, &[4_000], 1.0);
    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.channels, 2);
    assert_eq!(measurement.seam_frames, vec![4_000]);
    assert!(
        (measurement.peak_seam_delta - 0.5).abs() < 1.0e-3,
        "expected the injected step less the floor, got {}",
        measurement.peak_seam_delta,
    );
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::DynamicSegmentSeamClickDbfs
    );
    assert_eq!(measurement.metric.value, measurement.click_dbfs);

    // And it stays visible through the smoother, which is the whole point:
    // the smoother sets the straddling pair equal, so a measurement that
    // read only that pair scored this -240 dBFS, the silence sentinel.
    // A linear ramp is the smoother's best case -- it really does spread
    // the 0.5 step over its 256-frame fade -- and even here the residue
    // reads -60.2 dBFS rather than silence.
    let mut smoothed = frames.clone();
    smooth_dynamic_segment_boundaries_interleaved(&mut smoothed, 2, &[4_000], 256);
    let after = measure_dynamic_segment_seam_click(&smoothed, 2, &[4_000], 1.0);
    assert!(
        after.click_dbfs > -120.0,
        "the smoother must not be able to hide the step, got {:.2} dBFS",
        after.click_dbfs,
    );
}

#[test]
fn pitch_shift_metric_reports_dominant_frequency_error() {
    let sample_rate_hz = 48_000;
    let sample_rate = sample_rate_hz as f32;
    let input = sine(440.0, sample_rate, sample_rate_hz as usize);
    let mut stretcher = OfflineHighQualityStretcher::new(1.0);
    let output = stretcher
        .stretch_pitch_mono(&input, SampleRate(sample_rate_hz), 12.0)
        .expect("render fits the offline output bound");
    let measurement = measure_pitch_shift_error_cents(&output, sample_rate_hz, 440.0, 12.0, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.pitch_shift_semitones, 12.0);
    assert!((measurement.expected_frequency_hz - 880.0).abs() < 1.0e-6);
    assert!(measurement.measured_frequency_hz > 850.0);
    assert!(measurement.measured_frequency_hz < 910.0);
    assert!(measurement.pitch_error_cents < 75.0);
    assert_eq!(measurement.metric.metric, StretchMetric::PitchErrorCents);
    assert_eq!(measurement.metric.value, measurement.pitch_error_cents);
}

#[test]
fn synthetic_corpus_cases_run_without_file_io() {
    let cases = synthetic_stretch_corpus_cases();
    assert_eq!(cases.len(), 3);
    for (case, audio) in cases {
        assert_eq!(case.source, StretchCorpusSource::Synthetic);
        assert!(audio.sample_rate_hz > 0);
        assert!(audio.channels > 0);
        assert_eq!(audio.samples.len() % audio.channels as usize, 0);
        assert!(audio.samples.iter().any(|sample| sample.abs() > 0.01));
    }
}

#[test]
fn synthetic_backend_comparison_covers_all_synthetic_cases() {
    let report = compare_synthetic_stretch_backends();

    assert_eq!(report.comparisons.len(), 27);
    assert_eq!(
        report.improved_count
            + report.regressed_count
            + report.unchanged_count
            + report.inconclusive_count,
        report.comparisons.len()
    );
    for comparison in &report.comparisons {
        assert_eq!(comparison.baseline_backend, StretchBenchmarkBackend::Draft);
        assert_eq!(
            comparison.candidate_backend,
            StretchBenchmarkBackend::OfflineHighQualityPrototype
        );
        assert!(comparison.ratio.is_finite());
        assert!(comparison.ratio > 0.0);
        assert!(matches!(
            comparison.case_id,
            "stretch:tempo_ramp"
                | "stretch:loop_seam"
                | "stretch:extreme_ratio"
                | "stretch:pitch_shift"
                | "stretch:sustained_coherence"
        ));
    }
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::TimingDriftSamples
            && comparison.ratio > 1.0
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:tempo_ramp"
            && comparison.metric == StretchMetric::DynamicSegmentSeamClickDbfs
            && comparison.ratio > 1.0
            && comparison.path == StretchBenchmarkPath::DynamicRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::LoopBoundaryClickDbfs
            && comparison.path == StretchBenchmarkPath::FixedRatio
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:loop_seam"
            && comparison.metric == StretchMetric::StereoImageDelta
            && comparison.path == StretchBenchmarkPath::LinkedStereo
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:extreme_ratio"
            && comparison.metric == StretchMetric::TransientSmearFrames
            && comparison.path == StretchBenchmarkPath::FixedRatio
    }));
    let expanded_transient = report
        .comparisons
        .iter()
        .find(|comparison| {
            comparison.case_id == "stretch:extreme_ratio"
                && comparison.metric == StretchMetric::TransientSmearFrames
                && comparison.path == StretchBenchmarkPath::FixedRatio
                && comparison.ratio == 2.0
        })
        .expect("2x transient-smear comparison should remain covered");
    assert!(expanded_transient.baseline_value.is_finite());
    assert!(expanded_transient.candidate_value.is_finite());
    assert!(expanded_transient.delta.is_finite());
    assert_ne!(
        expanded_transient.outcome,
        StretchBenchmarkComparisonOutcome::Inconclusive
    );
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:pitch_shift"
            && comparison.metric == StretchMetric::PitchErrorCents
            && comparison.path == StretchBenchmarkPath::PitchShift
            && comparison.pitch_shift_semitones == Some(12.0)
    }));
    assert!(report.comparisons.iter().any(|comparison| {
        comparison.case_id == "stretch:sustained_coherence"
            && comparison.metric == StretchMetric::VerticalCoherenceDelta
            && comparison.path == StretchBenchmarkPath::PhaseLocked
    }));
}

#[test]
fn synthetic_backend_comparison_report_formats_deterministically() {
    let report = compare_synthetic_stretch_backends();
    let formatted = format_synthetic_stretch_comparison_report(&report);
    let repeated = format_synthetic_stretch_comparison_report(&report);

    assert_eq!(formatted, repeated);
    assert!(formatted.starts_with("synthetic_stretch_comparison improved="));
    assert!(formatted.contains("case=stretch:tempo_ramp"));
    assert!(formatted.contains("path=DynamicRatio"));
    assert!(formatted.contains("path=PhaseLocked"));
    assert!(formatted.contains("path=LinkedStereo"));
    assert!(formatted.contains("path=PitchShift"));
    assert!(formatted.contains("pitch_shift=12.000000"));
    assert!(formatted.contains("metric=TimingDriftSamples"));
    assert!(formatted.contains("metric=DynamicSegmentSeamClickDbfs"));
    assert!(formatted.contains("metric=VerticalCoherenceDelta"));
    assert!(formatted.contains("metric=PitchErrorCents"));
    assert!(formatted.contains("candidate_backend=OfflineHighQualityPrototype"));
    assert!(formatted.contains("candidate="));
    assert!(formatted.contains("outcome="));
}

#[test]
fn stretch_corpus_comparison_report_covers_manifest_and_note_slots() {
    let report =
        build_stretch_corpus_comparison_report("stretch-corpus-v1-local", "projection:unit");

    assert_eq!(report.report_name, "stretch-corpus-v1-local");
    assert_eq!(report.projection_epoch, "projection:unit");
    assert_eq!(report.manifest.manifest_id, "stretch-corpus-v1");
    assert_eq!(report.engine_version, SIGNAL_STRETCH_ENGINE_VERSION);
    assert_eq!(report.missing_assets.len(), 5);
    assert_eq!(report.optional_benchmark_skips.len(), 0);
    assert_eq!(report.synthetic_report.comparisons.len(), 27);
    assert_eq!(
        report.listening_note_slots.len(),
        report.missing_assets.len() + report.synthetic_report.comparisons.len()
    );
    assert!(report
        .missing_assets
        .iter()
        .all(|asset| asset.missing_asset_behavior
            == StretchCorpusMissingAssetBehavior::ReportMissingAndSkipCase));
    assert!(report.listening_note_slots.iter().any(|slot| slot.case_id
        == "stretch:drums_percussion"
        && slot.ratio.is_none()
        && slot
            .source_path_hint
            .starts_with("fixtures/stretch-corpus/licensed-listening/")));
    assert!(report.listening_note_slots.iter().any(|slot| {
        slot.case_id == "stretch:pitch_shift"
            && slot.pitch_shift_semitones == Some(12.0)
            && slot.source_path_hint == "inline:pitch-shift-tone"
    }));
}

#[test]
fn stretch_corpus_comparison_report_formats_deterministically() {
    let report =
        build_stretch_corpus_comparison_report("stretch-corpus-v1-local", "projection:unit");
    let formatted = format_stretch_corpus_comparison_report(&report);
    let repeated = format_stretch_corpus_comparison_report(&report);

    assert_eq!(formatted, repeated);
    assert!(formatted.starts_with(
        "stretch_corpus_report name=\"stretch-corpus-v1-local\" corpus=stretch-corpus-v1"
    ));
    // Assert against the constant, not a literal. The engine version
    // advances whenever renderer output changes, and this owner should
    // prove the report carries it, not pin a particular value.
    assert!(formatted.contains(&format!("engine={SIGNAL_STRETCH_ENGINE_VERSION}")));
    assert!(formatted.contains("projection_epoch=\"projection:unit\""));
    assert!(formatted.contains("source_policy synthetic="));
    assert!(formatted.contains(
        "summary comparisons=27 external_benchmark_comparisons=0 operator_listening_sources=0 missing_assets=5"
    ));
    assert!(formatted.contains("asset case=stretch:drums_percussion status=missing_required"));
    assert!(formatted.contains("comparison case=stretch:tempo_ramp"));
    assert!(formatted.contains("ratio_curve=synthetic_tempo_ramp:"));
    assert!(formatted.contains("pitch_curve=constant:12.000000"));
    assert!(formatted.contains("metric=DynamicSegmentSeamClickDbfs"));
    assert!(formatted.contains("listening_note case=stretch:pitch_shift"));
    assert!(formatted
        .contains("prompt=\"operator-note: record audible artifacts beside objective metrics\""));
}

#[test]
fn stretch_corpus_report_accepts_operator_listening_sources() {
    let report = build_stretch_corpus_comparison_report_with_sources(
        "stretch-corpus-v1-local",
        "projection:unit",
        &[],
        &[StretchCorpusListeningSource {
            case_id: "stretch:vocals".to_string(),
            source_path: "/Users/tom/Downloads/FMA/fma_large/000/000010.mp3".to_string(),
            source_label: "Kurt Vile - Freeway".to_string(),
            license_title: "Attribution-NonCommercial-NoDerivatives".to_string(),
            license_url: "https://example.test/license".to_string(),
            provenance_url: "https://example.test/track".to_string(),
        }],
    );

    assert_eq!(report.operator_listening_sources.len(), 1);
    assert_eq!(report.missing_assets.len(), 4);
    assert!(report
        .missing_assets
        .iter()
        .all(|asset| asset.case.case_id != "stretch:vocals"));
    assert!(report
        .listening_note_slots
        .iter()
        .any(|slot| slot.case_id == "stretch:vocals"
            && slot.source_path_hint == "/Users/tom/Downloads/FMA/fma_large/000/000010.mp3"
            && slot.prompt
                == "operator-note: record real-source listening artifacts before promotion"));

    let formatted = format_stretch_corpus_comparison_report(&report);

    assert!(formatted.contains("operator_listening_sources=1 missing_assets=4"));
    assert!(formatted.contains("operator_listening_source case=stretch:vocals"));
    assert!(formatted.contains("label=\"Kurt Vile - Freeway\""));
    assert!(formatted.contains(
        "source_boundary=\"operator-provided licensed local audio; no source audio committed\""
    ));
}

#[test]
fn stretch_corpus_report_accepts_optional_external_benchmark_render() {
    let loop_frames = generate_synthetic_stretch_audio(StretchCorpusFamily::LoopSeam)
        .expect("loop seam synthetic exists")
        .frame_count();
    let report = build_stretch_corpus_comparison_report_with_external(
        "stretch-corpus-v1-local",
        "projection:unit",
        &[StretchExternalBenchmarkRender {
            case_id: "stretch:loop_seam".to_string(),
            ratio: 1.0,
            pitch_shift_semitones: None,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: "fixtures/stretch-corpus/external-benchmark/loop.wav".to_string(),
            rendered_frames: loop_frames + 2,
            sample_rate_hz: 48_000,
            channels: 2,
        }],
    );
    let comparison = &report.external_benchmark_comparisons[0];

    assert_eq!(comparison.case_id, "stretch:loop_seam");
    assert_eq!(comparison.tool_name, "rubberband-cli");
    assert_eq!(comparison.expected_frames, Some(loop_frames));
    assert_eq!(comparison.timing_drift_samples, Some(2.0));
    assert_eq!(
        comparison.source_boundary,
        "rendered-output-only; no external source or library dependency"
    );

    let formatted = format_stretch_corpus_comparison_report(&report);
    assert!(formatted.contains("external_benchmark case=stretch:loop_seam"));
    assert!(formatted.contains("tool=\"rubberband-cli\""));
    assert!(formatted.contains(
        "source_boundary=\"rendered-output-only; no external source or library dependency\""
    ));
    assert!(formatted.contains("timing_drift_samples=2.000000"));
}

#[test]
fn stretch_corpus_report_keeps_unknown_external_benchmark_metadata_only() {
    let report = build_stretch_corpus_comparison_report_with_external(
        "stretch-corpus-v1-local",
        "projection:unit",
        &[StretchExternalBenchmarkRender {
            case_id: "stretch:licensed-only".to_string(),
            ratio: 1.25,
            pitch_shift_semitones: None,
            tool_name: "rubberband-cli".to_string(),
            rendered_path: "fixtures/stretch-corpus/external-benchmark/licensed.wav".to_string(),
            rendered_frames: 60_000,
            sample_rate_hz: 48_000,
            channels: 2,
        }],
    );
    let comparison = &report.external_benchmark_comparisons[0];

    assert_eq!(comparison.expected_frames, None);
    assert_eq!(comparison.timing_drift_samples, None);
    assert_eq!(comparison.rendered_frames, 60_000);
    assert_eq!(comparison.sample_rate_hz, 48_000);
    assert_eq!(comparison.channels, 2);
}

#[test]
fn stretch_quality_priorities_are_regression_only_and_sorted() {
    let report = compare_synthetic_stretch_backends();
    let priorities = prioritize_stretch_quality_work(&report, 8);
    let formatted = format_stretch_quality_priority_report(&priorities);

    assert!(priorities.is_empty());
    for priority in &priorities {
        assert!(matches!(
            priority.outcome,
            StretchBenchmarkComparisonOutcome::Regressed
                | StretchBenchmarkComparisonOutcome::Inconclusive
        ));
        assert!(priority.priority_score.is_finite());
        assert!(priority.priority_score > 0.0);
    }
    for pair in priorities.windows(2) {
        assert!(pair[0].priority_score >= pair[1].priority_score);
    }
    assert!(formatted.starts_with("stretch_quality_priorities count="));
    assert_eq!(formatted, "stretch_quality_priorities count=0");
}

#[test]
fn acceptance_report_format_is_deterministic() {
    let report = assess_stretch_metrics(
        &[StretchMetricValue::new(
            StretchMetric::TimingDriftSamples,
            0.0,
        )],
        &[StretchMetricLimit::max(
            StretchMetric::TimingDriftSamples,
            1.0,
            StretchAcceptanceSeverity::Fail,
        )],
    );

    assert_eq!(
        format_stretch_acceptance_report("stretch:tempo_ramp", &report),
        "case=stretch:tempo_ramp status=Pass\nmetric=TimingDriftSamples value=0.000000 max=1.000000 status=Pass"
    );
}

#[test]
fn sustained_material_coherence_comparison_logs_measured_gap() {
    let comparison = compare_sustained_material_coherence(1.5);

    assert_eq!(comparison.ratio, 1.5);
    assert!(comparison.draft_vertical_coherence_score.is_finite());
    assert!(comparison.phase_locked_vertical_coherence_score.is_finite());
    assert_eq!(
        comparison.metric.metric,
        StretchMetric::VerticalCoherenceDelta
    );
    assert!(
        (comparison.metric.value
            - (comparison.phase_locked_vertical_coherence_score
                - comparison.draft_vertical_coherence_score))
            .abs()
            < 1.0e-12
    );
}

#[test]
fn sustained_material_coherence_gap_formats_as_acceptance_metric() {
    let comparison = compare_sustained_material_coherence(1.25);
    let report = assess_stretch_metrics(
        &[comparison.metric],
        &[StretchMetricLimit::max(
            StretchMetric::VerticalCoherenceDelta,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:pads_sustains", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=VerticalCoherenceDelta"));
    assert!(formatted.contains("status=Pass"));
}

#[test]
fn transient_detector_finds_synthetic_attack_frames() {
    let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
        .expect("extreme-ratio synthetic audio exists");
    let events = detect_stretch_transients(&audio.samples, 1024, 256);

    assert!(
        events.len() >= 10,
        "expected repeated synthetic attacks, got {events:?}"
    );
    for expected in [8_000usize, 16_000, 24_000, 32_000, 40_000] {
        assert!(
            events
                .iter()
                .any(|event| event.frame_index.abs_diff(expected) <= 768),
            "missing transient near frame {expected}, got {events:?}"
        );
    }
    assert!(events.iter().all(|event| event.energy_score.is_finite()
        && event.spectral_flux_score.is_finite()
        && event.combined_score.is_finite()));
}

#[test]
fn transient_detector_default_policy_matches_production_entry_point() {
    let audio = generate_synthetic_stretch_audio(StretchCorpusFamily::ExtremeRatio)
        .expect("extreme-ratio synthetic audio exists");

    assert_eq!(
        detect_stretch_transients(&audio.samples, 1024, 256),
        detect_stretch_transients_with_policy(
            &audio.samples,
            1024,
            256,
            StretchTransientDetectorPolicy::production()
        )
    );
}

#[test]
fn candidate_transient_detector_recovers_masked_soft_attack() {
    let input = masked_soft_attack_probe(0.25);
    let production = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::production(),
    );
    let candidate = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::candidate_review(),
    );

    assert!(
        production
            .iter()
            .all(|event| event.frame_index.abs_diff(24_000) > 768),
        "production policy should miss the softened probe attack: {production:?}"
    );
    assert!(
        candidate
            .iter()
            .any(|event| event.frame_index.abs_diff(24_000) <= 768),
        "candidate policy should recover the softened probe attack: {candidate:?}"
    );
}

#[test]
fn transient_detector_stays_quiet_on_plain_sustain() {
    let input = sine(440.0, 48_000.0, 48_000);
    let events = detect_stretch_transients(&input, 1024, 256);

    assert!(
        events.len() <= 1,
        "plain sustain should not generate repeated transient events: {events:?}"
    );
}

#[test]
fn candidate_transient_detector_stays_quiet_on_plain_sustain() {
    let input = sine(440.0, 48_000.0, 48_000);
    let events = detect_stretch_transients_with_policy(
        &input,
        1024,
        256,
        StretchTransientDetectorPolicy::candidate_review(),
    );

    assert!(
        events.len() <= 1,
        "candidate policy should not generate repeated sustain events: {events:?}"
    );
}

#[test]
fn transient_smear_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_transient_smear(1.5);

    assert_eq!(measurement.ratio, 1.5);
    assert!(measurement.input_transients >= 10);
    assert!(measurement.output_transients > 0);
    assert!(measurement.matched_transients > 0);
    assert_eq!(
        measurement.input_transients,
        measurement.matched_transients + measurement.missed_transients
    );
    assert!(measurement.mean_smear_frames.is_finite());
    assert!(measurement.max_smear_frames.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::TransientSmearFrames
    );
    assert_eq!(measurement.metric.value, measurement.max_smear_frames);
}

#[test]
fn transient_reset_smear_metric_reports_synthetic_case() {
    let draft = measure_draft_transient_smear(1.5);
    let reset = measure_transient_reset_transient_smear(1.5);

    assert_eq!(reset.ratio, 1.5);
    assert_eq!(reset.input_transients, draft.input_transients);
    assert!(reset.output_transients > 0);
    assert!(reset.matched_transients > 0);
    assert_eq!(
        reset.input_transients,
        reset.matched_transients + reset.missed_transients
    );
    assert!(reset.max_smear_frames.is_finite());
    assert_eq!(reset.metric.metric, StretchMetric::TransientSmearFrames);
}

#[test]
fn transient_smear_metric_penalizes_missing_matches() {
    let mut input = vec![0.0; 64];
    input[20] = 1.0;
    input[21] = 0.5;
    input[22] = 0.25;
    let output = vec![0.0; 64];
    let measurement = measure_transient_smear(
        &input,
        &output,
        1.0,
        16,
        4,
        StretchTransientSmearPolicies::production(),
    );

    assert!(measurement.input_transients > 0);
    assert_eq!(measurement.output_transients, 0);
    assert_eq!(measurement.matched_transients, 0);
    assert_eq!(measurement.missed_transients, measurement.input_transients);
    assert_eq!(measurement.mean_smear_frames, 16.0);
    assert_eq!(measurement.max_smear_frames, 16.0);
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::TransientSmearFrames
    );
    assert_eq!(measurement.metric.value, 16.0);
}

#[test]
fn transient_smear_entry_point_uses_promoted_output_recovery_policy() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let promoted = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::production(),
    );
    let strict = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
    );
    let recovery = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
        },
    );

    assert_eq!(promoted, recovery);
    assert!(promoted.matched_transients > strict.matched_transients);
    assert!(promoted.missed_transients < strict.missed_transients);
}

#[test]
fn candidate_transient_smear_counts_masked_soft_attack() {
    let input = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &input,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::production()),
    );
    let candidate = measure_transient_smear(
        &input,
        &input,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies::symmetric(StretchTransientDetectorPolicy::candidate_review()),
    );

    assert!(candidate.input_transients > production.input_transients);
    assert!(candidate.matched_transients > production.matched_transients);
    assert_eq!(candidate.missed_transients, 0);
    assert_eq!(candidate.max_smear_frames, 0.0);
}

#[test]
fn candidate_output_policy_recovers_production_input_match() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: None,
        },
    );
    let candidate_output = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::candidate_review(),
            output_recovery: None,
        },
    );

    assert_eq!(
        candidate_output.input_transients,
        production.input_transients
    );
    assert!(candidate_output.matched_transients > production.matched_transients);
    assert!(candidate_output.missed_transients < production.missed_transients);
}

#[test]
fn output_recovery_policy_keeps_primary_matches_before_candidate_recovery() {
    let input = masked_soft_attack_probe(1.0);
    let output = masked_soft_attack_probe(0.25);
    let production = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: None,
        },
    );
    let recovery = measure_transient_smear(
        &input,
        &output,
        1.0,
        1024,
        256,
        StretchTransientSmearPolicies {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
        },
    );

    assert_eq!(recovery.input_transients, production.input_transients);
    assert_eq!(recovery.output_transients, production.output_transients);
    assert!(recovery.matched_transients > production.matched_transients);
    assert!(recovery.missed_transients < production.missed_transients);
    assert!(recovery.max_smear_frames <= production.max_smear_frames);
}

#[test]
fn transient_smear_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_transient_smear(1.25);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::TransientSmearFrames,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:extreme_ratio", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=TransientSmearFrames"));
    assert!(formatted.contains("status=Pass"));
}

#[test]
fn loop_boundary_metric_reports_direct_discontinuity() {
    let frames = [0.1, -0.2, 0.3, 0.1];
    let measurement = measure_loop_boundary_click(&frames, 2, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert_eq!(measurement.channels, 2);
    assert!((measurement.peak_boundary_delta - 0.3).abs() < 1.0e-6);
    assert!((measurement.click_dbfs - (20.0f64 * 0.3f64.log10())).abs() < 1.0e-6);
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
    assert_eq!(measurement.metric.value, measurement.click_dbfs);
}

#[test]
fn loop_boundary_smoothing_equalizes_endpoints() {
    let mut frames = [1.0, -0.5, 0.25, 0.25, -1.0, 0.75];

    benchmark::smooth_loop_boundary_interleaved(&mut frames, 2, 1);

    assert!((frames[0] - frames[4]).abs() < 1.0e-6);
    assert!((frames[1] - frames[5]).abs() < 1.0e-6);
    assert!((frames[2] - 0.25).abs() < 1.0e-6);
    assert!((frames[3] - 0.25).abs() < 1.0e-6);
}

#[test]
fn dynamic_segment_boundary_smoothing_equalizes_join_edges() {
    let mut frames = [0.0, 0.0, 1.0, -1.0, -1.0, 1.0, 0.0, 0.0];

    smooth_dynamic_segment_boundaries_interleaved(&mut frames, 2, &[2], 1);

    assert!((frames[2] - frames[4]).abs() < 1.0e-6);
    assert!((frames[3] - frames[5]).abs() < 1.0e-6);
    assert_eq!(frames[0], 0.0);
    assert_eq!(frames[1], 0.0);
    assert_eq!(frames[6], 0.0);
    assert_eq!(frames[7], 0.0);
}

#[test]
fn loop_boundary_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_loop_boundary_click(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert_eq!(measurement.channels, 2);
    assert!(measurement.peak_boundary_delta.is_finite());
    assert!(measurement.click_dbfs.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
}

#[test]
fn transient_reset_loop_boundary_metric_reports_synthetic_case() {
    let measurement = measure_transient_reset_loop_boundary_click(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert_eq!(measurement.channels, 2);
    assert!(measurement.peak_boundary_delta.is_finite());
    assert!(measurement.click_dbfs.is_finite());
    assert_eq!(
        measurement.metric.metric,
        StretchMetric::LoopBoundaryClickDbfs
    );
}

#[test]
fn loop_boundary_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_loop_boundary_click(1.5);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::LoopBoundaryClickDbfs,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:loop_seam", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=LoopBoundaryClickDbfs"));
    assert!(formatted.contains("status=Pass"));
}

#[test]
fn stereo_image_metric_reports_direct_movement() {
    let input = [0.5, 0.5, 0.25, 0.25, -0.25, -0.25, -0.5, -0.5];
    let output = [0.5, -0.5, 0.25, -0.25, -0.25, 0.25, -0.5, 0.5];
    let measurement = measure_stereo_image_delta(&input, &output, 1.0);

    assert_eq!(measurement.ratio, 1.0);
    assert!(measurement.input_correlation > 0.99);
    assert!(measurement.output_correlation < -0.99);
    assert!(measurement.image_delta > 1.0);
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
    assert_eq!(measurement.metric.value, measurement.image_delta);
}

#[test]
fn stereo_image_metric_reports_synthetic_draft_case() {
    let measurement = measure_draft_stereo_image_delta(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert!(measurement.input_correlation.is_finite());
    assert!(measurement.output_correlation.is_finite());
    assert!(measurement.input_side_mid_ratio.is_finite());
    assert!(measurement.output_side_mid_ratio.is_finite());
    assert!(measurement.image_delta.is_finite());
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
}

#[test]
fn transient_reset_stereo_image_metric_reports_synthetic_case() {
    let measurement = measure_transient_reset_stereo_image_delta(1.25);

    assert_eq!(measurement.ratio, 1.25);
    assert!(measurement.input_correlation.is_finite());
    assert!(measurement.output_correlation.is_finite());
    assert!(measurement.image_delta.is_finite());
    assert_eq!(measurement.metric.metric, StretchMetric::StereoImageDelta);
}

#[test]
fn stereo_image_metric_formats_as_acceptance_metric() {
    let measurement = measure_draft_stereo_image_delta(1.5);
    let report = assess_stretch_metrics(
        &[measurement.metric],
        &[StretchMetricLimit::max(
            StretchMetric::StereoImageDelta,
            f64::INFINITY,
            StretchAcceptanceSeverity::Warn,
        )],
    );
    let formatted = format_stretch_acceptance_report("stretch:full_mix", &report);

    assert_eq!(report.status, StretchAcceptanceStatus::Pass);
    assert!(formatted.contains("metric=StereoImageDelta"));
    assert!(formatted.contains("status=Pass"));
}
