use signal_primitives::{Sample, SampleRate};

use crate::phase_vocoder::{phase_vocoder, transient_reset_phase_vocoder};
use crate::transient_smear::{
    measure_transient_smear, transient_smear_nan, StretchTransientSmearMeasurement,
    StretchTransientSmearPolicies,
};
use crate::{
    dynamic_ratio_output_boundaries, dynamic_ratio_output_frames, OfflineHighQualityStretcher,
    RealtimePreviewStretcher, TimeStretcher,
};

use super::measure::{
    compare_sustained_material_coherence, loop_boundary_nan, measure_draft_loop_boundary_click,
    measure_draft_stereo_image_delta, measure_draft_transient_smear,
    measure_dynamic_segment_seam_click, measure_loop_boundary_click,
    measure_pitch_shift_error_cents, measure_stereo_image_delta,
    measure_transient_reset_loop_boundary_click, measure_transient_reset_stereo_image_delta,
    measure_transient_reset_transient_smear, output_length_drift_samples,
    smooth_loop_boundary_interleaved, stereo_image_nan,
};
use super::synthetic::{
    generate_synthetic_stretch_audio, stretch_dynamic_ratio_stereo_independent,
    stretch_stereo_synthetic, synthetic_extreme_ratio, synthetic_loop_seam,
    synthetic_pitch_shift_tone, synthetic_tempo_ramp, synthetic_tempo_ramp_ratio_curve,
};
use super::types::{
    StretchBenchmarkBackend, StretchBenchmarkComparisonOutcome, StretchBenchmarkPath,
    StretchCorpusFamily, StretchCorpusSource, StretchLoopBoundaryMeasurement, StretchMetric,
    StretchStereoImageMeasurement, StretchSyntheticBenchmarkComparison,
    StretchSyntheticBenchmarkComparisonReport, STRETCH_BENCHMARK_CORPUS,
};

/// Compare the OfflineHighQuality prototype against the draft baseline across
/// all repository-local synthetic stretch corpus cases.
///
/// This is a measurement report, not a promotion decision. It intentionally
/// includes both improvements and regressions so later promotion work can tune
/// thresholds from evidence.
pub fn compare_synthetic_stretch_backends() -> StretchSyntheticBenchmarkComparisonReport {
    let mut comparisons = Vec::new();

    for case in STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter(|case| case.source == StretchCorpusSource::Synthetic)
    {
        for ratio in case.ratios {
            comparisons.push(compare_metric(
                case.case_id,
                *ratio,
                StretchMetric::TimingDriftSamples,
                measure_synthetic_length_drift(case.family, *ratio, phase_vocoder),
                measure_synthetic_length_drift(case.family, *ratio, transient_reset_phase_vocoder),
            ));

            match case.family {
                StretchCorpusFamily::LoopSeam => {
                    comparisons.push(compare_metric(
                        case.case_id,
                        *ratio,
                        StretchMetric::LoopBoundaryClickDbfs,
                        measure_draft_loop_boundary_click(*ratio).metric.value,
                        measure_transient_reset_loop_boundary_click(*ratio)
                            .metric
                            .value,
                    ));
                    comparisons.push(
                        compare_metric(
                            case.case_id,
                            *ratio,
                            StretchMetric::StereoImageDelta,
                            measure_draft_stereo_image_delta(*ratio).metric.value,
                            measure_transient_reset_stereo_image_delta(*ratio)
                                .metric
                                .value,
                        )
                        .with_path(StretchBenchmarkPath::LinkedStereo),
                    );
                }
                StretchCorpusFamily::ExtremeRatio => {
                    comparisons.push(compare_metric(
                        case.case_id,
                        *ratio,
                        StretchMetric::TransientSmearFrames,
                        measure_draft_transient_smear(*ratio).metric.value,
                        measure_transient_reset_transient_smear(*ratio).metric.value,
                    ));
                }
                _ => {}
            }
        }

        if case.family == StretchCorpusFamily::TempoRamp {
            comparisons.extend(compare_dynamic_tempo_ramp(case.case_id));
        }
    }
    comparisons.extend(compare_pitch_shift());
    comparisons.extend(compare_sustained_coherence());

    finish_synthetic_benchmark_report(comparisons)
}

/// Compare the RealtimePreview prototype against the draft baseline across
/// the synthetic corpus subset relevant to low-latency preview.
pub fn compare_synthetic_realtime_preview_backends() -> StretchSyntheticBenchmarkComparisonReport {
    let mut comparisons = Vec::new();

    for case in STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter(|case| case.source == StretchCorpusSource::Synthetic)
    {
        for ratio in case.ratios {
            comparisons.push(compare_metric_for_backend(
                case.case_id,
                *ratio,
                StretchMetric::TimingDriftSamples,
                StretchBenchmarkBackend::RealtimePreviewPrototype,
                measure_synthetic_length_drift(case.family, *ratio, phase_vocoder),
                measure_synthetic_length_drift_realtime_preview(case.family, *ratio),
            ));

            match case.family {
                StretchCorpusFamily::LoopSeam => {
                    comparisons.push(compare_metric_for_backend(
                        case.case_id,
                        *ratio,
                        StretchMetric::LoopBoundaryClickDbfs,
                        StretchBenchmarkBackend::RealtimePreviewPrototype,
                        measure_draft_loop_boundary_click(*ratio).metric.value,
                        measure_realtime_preview_loop_boundary_click(*ratio)
                            .metric
                            .value,
                    ));
                    comparisons.push(
                        compare_metric_for_backend(
                            case.case_id,
                            *ratio,
                            StretchMetric::StereoImageDelta,
                            StretchBenchmarkBackend::RealtimePreviewPrototype,
                            measure_draft_stereo_image_delta(*ratio).metric.value,
                            measure_realtime_preview_stereo_image_delta(*ratio)
                                .metric
                                .value,
                        )
                        .with_path(StretchBenchmarkPath::LinkedStereo),
                    );
                }
                StretchCorpusFamily::ExtremeRatio => {
                    comparisons.push(compare_metric_for_backend(
                        case.case_id,
                        *ratio,
                        StretchMetric::TransientSmearFrames,
                        StretchBenchmarkBackend::RealtimePreviewPrototype,
                        measure_draft_transient_smear(*ratio).metric.value,
                        measure_realtime_preview_transient_smear(*ratio)
                            .metric
                            .value,
                    ));
                }
                _ => {}
            }
        }

        if case.family == StretchCorpusFamily::TempoRamp {
            comparisons.extend(compare_dynamic_tempo_ramp_realtime_preview(case.case_id));
        }
    }
    comparisons.extend(compare_pitch_shift_realtime_preview());

    finish_synthetic_benchmark_report(comparisons)
}
fn finish_synthetic_benchmark_report(
    comparisons: Vec<StretchSyntheticBenchmarkComparison>,
) -> StretchSyntheticBenchmarkComparisonReport {
    let mut report = StretchSyntheticBenchmarkComparisonReport {
        comparisons,
        improved_count: 0,
        regressed_count: 0,
        unchanged_count: 0,
        inconclusive_count: 0,
    };
    for comparison in &report.comparisons {
        match comparison.outcome {
            StretchBenchmarkComparisonOutcome::Improved => report.improved_count += 1,
            StretchBenchmarkComparisonOutcome::Regressed => report.regressed_count += 1,
            StretchBenchmarkComparisonOutcome::Unchanged => report.unchanged_count += 1,
            StretchBenchmarkComparisonOutcome::Inconclusive => report.inconclusive_count += 1,
        }
    }
    report
}
fn compare_metric(
    case_id: &'static str,
    ratio: f64,
    metric: StretchMetric,
    baseline_value: f64,
    candidate_value: f64,
) -> StretchSyntheticBenchmarkComparison {
    compare_metric_for_backend(
        case_id,
        ratio,
        metric,
        StretchBenchmarkBackend::OfflineHighQualityPrototype,
        baseline_value,
        candidate_value,
    )
}

fn compare_metric_for_backend(
    case_id: &'static str,
    ratio: f64,
    metric: StretchMetric,
    candidate_backend: StretchBenchmarkBackend,
    baseline_value: f64,
    candidate_value: f64,
) -> StretchSyntheticBenchmarkComparison {
    let delta = candidate_value - baseline_value;
    let outcome =
        if !baseline_value.is_finite() || !candidate_value.is_finite() || !delta.is_finite() {
            StretchBenchmarkComparisonOutcome::Inconclusive
        } else if delta < -comparison_tolerance(metric) {
            StretchBenchmarkComparisonOutcome::Improved
        } else if delta > comparison_tolerance(metric) {
            StretchBenchmarkComparisonOutcome::Regressed
        } else {
            StretchBenchmarkComparisonOutcome::Unchanged
        };

    StretchSyntheticBenchmarkComparison {
        case_id,
        ratio,
        metric,
        path: StretchBenchmarkPath::FixedRatio,
        pitch_shift_semitones: None,
        baseline_backend: StretchBenchmarkBackend::Draft,
        candidate_backend,
        baseline_value,
        candidate_value,
        delta,
        outcome,
    }
}

fn comparison_tolerance(metric: StretchMetric) -> f64 {
    match metric {
        StretchMetric::TransientSmearFrames => 1.0,
        _ => 1.0e-9,
    }
}

impl StretchSyntheticBenchmarkComparison {
    fn with_path(mut self, path: StretchBenchmarkPath) -> Self {
        self.path = path;
        self
    }

    fn with_pitch_shift(mut self, pitch_shift_semitones: f64) -> Self {
        self.path = StretchBenchmarkPath::PitchShift;
        self.pitch_shift_semitones = Some(pitch_shift_semitones);
        self
    }
}

fn measure_synthetic_length_drift(
    family: StretchCorpusFamily,
    ratio: f64,
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> f64 {
    let Some(input) = generate_synthetic_stretch_audio(family) else {
        return f64::NAN;
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }

    let input_frames = input.frame_count();
    let output_frames = match input.channels {
        1 => {
            let target_len = (input_frames as f64 * ratio).round() as usize;
            stretcher(&input.samples, target_len, ratio, 2_048, 512).len()
        }
        2 => stretch_stereo_synthetic(&input, ratio, stretcher).len() / 2,
        _ => return f64::NAN,
    };

    output_length_drift_samples(input_frames, output_frames, ratio)
}

fn measure_synthetic_length_drift_realtime_preview(family: StretchCorpusFamily, ratio: f64) -> f64 {
    let Some(input) = generate_synthetic_stretch_audio(family) else {
        return f64::NAN;
    };
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }

    let input_frames = input.frame_count();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output_frames = match input.channels {
        1 => preview
            .stretch_mono(&input.samples)
            .expect("corpus render fits the offline output bound")
            .len(),
        2 => {
            preview
                .stretch_interleaved_stereo(&input.samples)
                .expect("corpus render fits the offline output bound")
                .len()
                / 2
        }
        _ => return f64::NAN,
    };

    output_length_drift_samples(input_frames, output_frames, ratio)
}

fn measure_realtime_preview_loop_boundary_click(ratio: f64) -> StretchLoopBoundaryMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return loop_boundary_nan(ratio, 2);
    }

    let input = synthetic_loop_seam();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let mut output = preview
        .stretch_interleaved_stereo(&input.samples)
        .expect("corpus render fits the offline output bound");
    smooth_loop_boundary_interleaved(&mut output, input.channels, 128);
    measure_loop_boundary_click(&output, input.channels, ratio)
}

fn measure_realtime_preview_stereo_image_delta(ratio: f64) -> StretchStereoImageMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return stereo_image_nan(ratio);
    }

    let input = synthetic_loop_seam();
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output = preview
        .stretch_interleaved_stereo(&input.samples)
        .expect("corpus render fits the offline output bound");
    measure_stereo_image_delta(&input.samples, &output, ratio)
}

fn measure_realtime_preview_transient_smear(ratio: f64) -> StretchTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 {
        return transient_smear_nan(ratio);
    }

    const WINDOW_SIZE: usize = 1_024;
    const HOP_SIZE: usize = 256;
    let input = synthetic_extreme_ratio().samples;
    let mut preview = RealtimePreviewStretcher::new(ratio);
    let output = preview
        .stretch_mono(&input)
        .expect("corpus render fits the offline output bound");
    measure_transient_smear(
        &input,
        &output,
        ratio,
        WINDOW_SIZE,
        HOP_SIZE,
        StretchTransientSmearPolicies::production(),
    )
}

fn compare_dynamic_tempo_ramp(case_id: &'static str) -> Vec<StretchSyntheticBenchmarkComparison> {
    let input = synthetic_tempo_ramp();
    let ratio_curve = synthetic_tempo_ramp_ratio_curve(input.frame_count());
    let expected_frames = dynamic_ratio_output_frames(input.frame_count(), &ratio_curve, 1.0);
    let seam_frames = dynamic_ratio_output_boundaries(input.frame_count(), &ratio_curve, 1.0);
    let effective_ratio = expected_frames as f64 / input.frame_count() as f64;
    let draft_output =
        stretch_dynamic_ratio_stereo_independent(&input, &ratio_curve, phase_vocoder);
    let mut offline_high_quality = OfflineHighQualityStretcher::new(1.0);
    let offline_high_quality_output = offline_high_quality
        .stretch_dynamic_ratio_interleaved_stereo(&input.samples, &ratio_curve)
        .expect("corpus render fits the offline output bound");

    vec![
        compare_metric(
            case_id,
            effective_ratio,
            StretchMetric::TimingDriftSamples,
            (draft_output.len() / 2).abs_diff(expected_frames) as f64,
            (offline_high_quality_output.len() / 2).abs_diff(expected_frames) as f64,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
        compare_metric(
            case_id,
            effective_ratio,
            StretchMetric::DynamicSegmentSeamClickDbfs,
            measure_dynamic_segment_seam_click(
                &draft_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
            measure_dynamic_segment_seam_click(
                &offline_high_quality_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
    ]
}

fn compare_dynamic_tempo_ramp_realtime_preview(
    case_id: &'static str,
) -> Vec<StretchSyntheticBenchmarkComparison> {
    let input = synthetic_tempo_ramp();
    let ratio_curve = synthetic_tempo_ramp_ratio_curve(input.frame_count());
    let expected_frames = dynamic_ratio_output_frames(input.frame_count(), &ratio_curve, 1.0);
    let seam_frames = dynamic_ratio_output_boundaries(input.frame_count(), &ratio_curve, 1.0);
    let effective_ratio = expected_frames as f64 / input.frame_count() as f64;
    let draft_output =
        stretch_dynamic_ratio_stereo_independent(&input, &ratio_curve, phase_vocoder);
    let mut preview = RealtimePreviewStretcher::new(1.0);
    let preview_output = preview
        .stretch_dynamic_ratio_interleaved_stereo(&input.samples, &ratio_curve)
        .expect("corpus render fits the offline output bound");

    vec![
        compare_metric_for_backend(
            case_id,
            effective_ratio,
            StretchMetric::TimingDriftSamples,
            StretchBenchmarkBackend::RealtimePreviewPrototype,
            (draft_output.len() / 2).abs_diff(expected_frames) as f64,
            (preview_output.len() / 2).abs_diff(expected_frames) as f64,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
        compare_metric_for_backend(
            case_id,
            effective_ratio,
            StretchMetric::DynamicSegmentSeamClickDbfs,
            StretchBenchmarkBackend::RealtimePreviewPrototype,
            measure_dynamic_segment_seam_click(
                &draft_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
            measure_dynamic_segment_seam_click(
                &preview_output,
                input.channels,
                &seam_frames,
                effective_ratio,
            )
            .metric
            .value,
        )
        .with_path(StretchBenchmarkPath::DynamicRatio),
    ]
}

fn compare_pitch_shift() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:pitch_shift";
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const SOURCE_FREQUENCY_HZ: f64 = 440.0;

    let input = synthetic_pitch_shift_tone(SOURCE_FREQUENCY_HZ, SAMPLE_RATE_HZ, 48_000);
    [(1.0, 12.0), (1.25, -5.0)]
        .into_iter()
        .map(|(ratio, semitones)| {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft_output = phase_vocoder(&input, target_len, ratio, 2_048, 512);
            let mut offline_high_quality = OfflineHighQualityStretcher::new(ratio);
            let offline_high_quality_output = offline_high_quality
                .stretch_pitch_mono(&input, SampleRate(SAMPLE_RATE_HZ), semitones)
                .expect("corpus render fits the offline output bound");

            compare_metric(
                CASE_ID,
                ratio,
                StretchMetric::PitchErrorCents,
                measure_pitch_shift_error_cents(
                    &draft_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
                measure_pitch_shift_error_cents(
                    &offline_high_quality_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
            )
            .with_pitch_shift(semitones)
        })
        .collect()
}

fn compare_pitch_shift_realtime_preview() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:pitch_shift";
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const SOURCE_FREQUENCY_HZ: f64 = 440.0;

    let input = synthetic_pitch_shift_tone(SOURCE_FREQUENCY_HZ, SAMPLE_RATE_HZ, 48_000);
    [(1.0, 12.0), (1.25, -5.0)]
        .into_iter()
        .map(|(ratio, semitones)| {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft_output = phase_vocoder(&input, target_len, ratio, 2_048, 512);
            let mut preview = RealtimePreviewStretcher::new(ratio);
            let preview_output = preview
                .stretch_pitch_mono(&input, SampleRate(SAMPLE_RATE_HZ), semitones)
                .expect("corpus render fits the offline output bound");

            compare_metric_for_backend(
                CASE_ID,
                ratio,
                StretchMetric::PitchErrorCents,
                StretchBenchmarkBackend::RealtimePreviewPrototype,
                measure_pitch_shift_error_cents(
                    &draft_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
                measure_pitch_shift_error_cents(
                    &preview_output,
                    SAMPLE_RATE_HZ,
                    SOURCE_FREQUENCY_HZ,
                    semitones,
                    ratio,
                )
                .metric
                .value,
            )
            .with_pitch_shift(semitones)
        })
        .collect()
}

fn compare_sustained_coherence() -> Vec<StretchSyntheticBenchmarkComparison> {
    const CASE_ID: &str = "stretch:sustained_coherence";

    [0.75, 1.25, 1.5]
        .into_iter()
        .map(|ratio| {
            let coherence = compare_sustained_material_coherence(ratio);
            compare_metric(
                CASE_ID,
                ratio,
                StretchMetric::VerticalCoherenceDelta,
                coherence.draft_vertical_coherence_score,
                coherence.phase_locked_vertical_coherence_score,
            )
            .with_path(StretchBenchmarkPath::PhaseLocked)
        })
        .collect()
}
