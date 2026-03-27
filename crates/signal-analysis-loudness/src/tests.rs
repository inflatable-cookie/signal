// Tests for signal-analysis-loudness
mod tests {
    use crate::{
        LoudnessAnalysisResult, LoudnessChannelWeightSource, LoudnessMeter, LoudnessMeterConfig,
        LoudnessSampleRateSupport, RUNTIME_MOMENTARY_TAIL_POINTS, RUNTIME_SHORT_TERM_TAIL_POINTS,
    };
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, SampleRate};

    fn sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = Vec::with_capacity(frames);
        for index in 0..frames {
            let t = index as f32 / sample_rate as f32;
            samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn sine_sequence(sample_rate: u32, sections: &[(f32, f32, f32)]) -> AudioBuffer {
        let mut samples = Vec::new();
        for (frequency, amplitude, seconds) in sections {
            samples
                .extend_from_slice(sine(sample_rate, *frequency, *amplitude, *seconds).samples());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn loudness_metrics(result: &LoudnessAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("integrated_lufs", result.integrated_lufs),
            AnalysisMetricValue::new("true_peak_dbtp", result.true_peak_dbtp),
            AnalysisMetricValue::new("loudness_range_lu", result.loudness_range_lu),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("momentary_range_lu", result.dynamics.momentary_range_lu),
        ]
    }

    fn loudness_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:quiet-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Quiet tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.1, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-20.5),
                    Some(-19.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:loud-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Loud tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.5, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-6.5),
                    Some(-5.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:level-step",
                    AnalysisCorpusFamily::Loudness,
                    "Two-section level-step range reference",
                ),
                sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "loudness_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "momentary_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    #[test]
    fn silence_reports_negative_infinity() {
        let audio = AudioBuffer::new(
            SampleRate(48_000),
            ChannelLayout::Mono,
            signal_primitives::FrameCount(48_000),
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);

        assert!(!result.integrated_lufs.is_finite());
        assert!(!result.true_peak_dbtp.is_finite());
        assert_eq!(result.loudness_range_lu, 0.0);
        assert_eq!(result.confidence.0, 0.0);
        assert_eq!(result.aggregation.channel_count, ChannelCount(1));
        assert!(result
            .momentary_trace
            .points
            .iter()
            .all(|point| !point.loudness_lufs.is_finite()));
        assert!(result
            .short_term_trace
            .points
            .iter()
            .all(|point| !point.loudness_lufs.is_finite()));
    }

    #[test]
    fn louder_signal_matches_expected_decibel_scaling() {
        let quiet = sine(48_000, 1_000.0, 0.1, 4.0);
        let loud = sine(48_000, 1_000.0, 0.5, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let quiet_result = meter.analyze(&quiet);
        let loud_result = meter.analyze(&loud);
        let expected_delta = 20.0 * (0.5f32 / 0.1f32).log10();

        assert!(loud_result.integrated_lufs > quiet_result.integrated_lufs);
        assert!(loud_result.true_peak_dbtp > quiet_result.true_peak_dbtp);
        assert!(loud_result.confidence.0 > 0.9);
        assert!(loud_result.dynamics.momentary_max_lufs > quiet_result.dynamics.momentary_max_lufs);
        assert!(
            (loud_result.integrated_lufs - quiet_result.integrated_lufs - expected_delta).abs()
                < 0.3
        );
        assert!(
            (loud_result.true_peak_dbtp - quiet_result.true_peak_dbtp - expected_delta).abs()
                < 0.05
        );
    }

    #[test]
    fn non_native_input_rate_is_resampled_without_material_drift() {
        let supported = sine(48_000, 440.0, 0.2, 4.0);
        let unsupported = sine(44_100, 440.0, 0.2, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let supported_result = meter.analyze(&supported);
        let unsupported_result = meter.analyze(&unsupported);

        assert_eq!(
            supported_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::Native48kKWeighted
        );
        assert_eq!(
            unsupported_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::ResampledTo48kKWeighted
        );
        assert!(supported_result.confidence.0 > unsupported_result.confidence.0);
        assert!(unsupported_result.confidence.0 >= 0.85);
        assert!(
            (supported_result.integrated_lufs - unsupported_result.integrated_lufs).abs() < 1.0
        );
    }

    #[test]
    fn low_profile_produces_finite_results() {
        let audio = sine(48_000, 440.0, 0.3, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::low());
        let result = meter.analyze(&audio);

        assert!(result.integrated_lufs.is_finite());
        assert!(result.true_peak_dbtp.is_finite());
        assert!(result.confidence.0 > 0.0);
        assert!(!result.momentary_trace.points.is_empty());
    }

    #[test]
    fn stereo_inputs_use_explicit_equal_weight_aggregation() {
        let mono = sine(48_000, 440.0, 0.25, 4.0);
        let stereo_samples: Vec<f32> = mono
            .samples()
            .iter()
            .flat_map(|sample| [*sample, *sample])
            .collect();
        let stereo = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            stereo_samples,
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let mono_result = meter.analyze(&mono);
        let stereo_result = meter.analyze(&stereo);

        assert_eq!(
            stereo_result.aggregation.channel_weight_source,
            LoudnessChannelWeightSource::StereoEqualWeight
        );
        assert_eq!(stereo_result.channels.len(), 2);
        assert!(
            (stereo_result.integrated_lufs - mono_result.integrated_lufs - 3.0103).abs() < 0.25
        );
        assert!((mono_result.true_peak_dbtp - stereo_result.true_peak_dbtp).abs() < 0.1);
        assert_eq!(
            stereo_result.short_term_trace.window_seconds,
            LoudnessMeterConfig::default().short_term_seconds
        );
    }

    #[test]
    fn generic_multichannel_layout_uses_deterministic_fallback_weights() {
        let mono = sine(48_000, 440.0, 0.2, 4.0);
        let quad_samples: Vec<f32> = mono
            .samples()
            .iter()
            .flat_map(|sample| [*sample, *sample, *sample, *sample])
            .collect();
        let quad = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Count(ChannelCount(4)),
            quad_samples,
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let mono_result = meter.analyze(&mono);
        let result = meter.analyze(&quad);

        assert_eq!(
            result.aggregation.channel_weight_source,
            LoudnessChannelWeightSource::GenericCountFallback
        );
        assert_eq!(result.channels.len(), 4);
        assert!(result.channels.iter().all(|channel| channel.weight == 1.0));
        assert!(result.confidence.0 < 1.0);
        assert!((result.integrated_lufs - mono_result.integrated_lufs - 6.0206).abs() < 0.35);
    }

    #[test]
    fn non_48k_analysis_rate_reports_unweighted_fallback() {
        let audio = sine(44_100, 1_000.0, 0.2, 4.0);
        let mut config = LoudnessMeterConfig::default();
        config.analysis_sample_rate = SampleRate(44_100);
        let mut fallback_meter = LoudnessMeter::new(config);
        let mut default_meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let fallback_result = fallback_meter.analyze(&audio);
        let default_result = default_meter.analyze(&audio);

        assert_eq!(
            fallback_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::UnweightedFallback
        );
        assert_eq!(fallback_result.aggregation.true_peak_oversample_factor, 4);
        assert!(fallback_result.confidence.0 < default_result.confidence.0);
        assert!(fallback_result.integrated_lufs.is_finite());
    }

    #[test]
    fn harness_loudness_cases_meet_frozen_acceptance_thresholds() {
        let cases = loudness_acceptance_cases();
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_loudness_acceptance_report_remains_interpretable_for_closeout() {
        let cases = loudness_acceptance_cases();
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

        println!("loudness_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }

    #[test]
    fn loudness_traces_capture_level_step_and_dynamics_summary() {
        let audio = sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);

        assert!(result.momentary_trace.points.len() > result.short_term_trace.points.len());
        assert!(result.momentary_trace.points.len() > 10);
        assert!(result.short_term_trace.points.len() >= 2);
        assert!(result.dynamics.momentary_max_lufs >= result.integrated_lufs);
        assert!(result.dynamics.short_term_max_lufs >= result.integrated_lufs);
        assert!(result.dynamics.momentary_range_lu > 0.0);
        assert!(result.dynamics.short_term_range_lu > 0.0);
        assert!(result.dynamics.target_offset_lu.is_finite());

        let loudest_momentary = result
            .momentary_trace
            .points
            .iter()
            .max_by(|lhs, rhs| {
                lhs.loudness_lufs
                    .partial_cmp(&rhs.loudness_lufs)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .expect("loudest momentary point");
        assert!(loudest_momentary.start_seconds >= 3.0);
    }

    #[test]
    fn runtime_diagnostics_summary_uses_bounded_recent_trace_tails() {
        let audio = sine_sequence(
            48_000,
            &[(440.0, 0.05, 3.0), (440.0, 0.2, 3.0), (440.0, 0.35, 3.0)],
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);
        let diagnostics = result.runtime_diagnostics_summary();

        assert!(diagnostics.recent_momentary.points.len() <= RUNTIME_MOMENTARY_TAIL_POINTS);
        assert!(diagnostics.recent_short_term.points.len() <= RUNTIME_SHORT_TERM_TAIL_POINTS);
        assert_eq!(
            diagnostics.current_momentary_lufs,
            diagnostics
                .recent_momentary
                .points
                .last()
                .expect("recent momentary point")
                .loudness_lufs
        );
        assert_eq!(
            diagnostics.current_short_term_lufs,
            diagnostics
                .recent_short_term
                .points
                .last()
                .expect("recent short-term point")
                .loudness_lufs
        );
        assert_eq!(diagnostics.integrated_lufs, result.integrated_lufs);
        assert_eq!(diagnostics.true_peak_dbtp, result.true_peak_dbtp);
        assert_eq!(
            diagnostics.target_offset_lu,
            result.dynamics.target_offset_lu
        );
        assert_eq!(
            diagnostics.momentary_max_lufs,
            result.dynamics.momentary_max_lufs
        );
        assert_eq!(
            diagnostics.short_term_max_lufs,
            result.dynamics.short_term_max_lufs
        );
    }
}
