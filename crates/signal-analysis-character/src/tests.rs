#[cfg(test)]
mod tests {
    use crate::*;
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine_audio(
        frequency_hz: f32,
        duration_seconds: f32,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        for (index, sample) in data.iter_mut().enumerate() {
            let time = index as f32 / sample_rate_hz as f32;
            *sample = amplitude * (core::f32::consts::TAU * frequency_hz * time).sin();
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn noise_audio(duration_seconds: f32, sample_rate_hz: u32, amplitude: f32) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        let mut state = 0x1234_5678u32;
        for sample in &mut data {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let unit = ((state >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sample = amplitude * unit;
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn adsr_pulse_audio(
        attack_ms: u32,
        sustain_ms: u32,
        decay_ms: u32,
        interval_ms: u32,
        event_count: usize,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let interval_samples = (interval_ms as usize * sample_rate_hz as usize) / 1_000;
        let attack_samples = (attack_ms as usize * sample_rate_hz as usize) / 1_000;
        let sustain_samples = (sustain_ms as usize * sample_rate_hz as usize) / 1_000;
        let decay_samples = (decay_ms as usize * sample_rate_hz as usize) / 1_000;
        let total_samples = interval_samples * event_count.max(1);
        let mut data = vec![0.0f32; total_samples.max(1)];

        for event_index in 0..event_count {
            let start = event_index * interval_samples;

            for offset in 0..attack_samples {
                let index = start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = (offset + 1) as f32 / attack_samples.max(1) as f32;
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }

            let sustain_start = start + attack_samples;
            for offset in 0..sustain_samples {
                let index = sustain_start + offset;
                if index >= data.len() {
                    break;
                }
                data[index] = amplitude;
            }

            let decay_start = sustain_start + sustain_samples;
            for offset in 0..decay_samples {
                let index = decay_start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = 1.0 - ((offset + 1) as f32 / decay_samples.max(1) as f32);
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn character_metrics(result: &CharacterAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("spectral_flatness", result.spectral_shape.flatness),
            AnalysisMetricValue::new("spectral_spread_hz", result.spectral_shape.spread_hz),
            AnalysisMetricValue::new("rms_energy", result.dynamics.rms_energy),
            AnalysisMetricValue::new("sustain_ratio", result.temporal.sustain_ratio),
            AnalysisMetricValue::new(
                "peak_transient_strength",
                result.temporal_shape.peak_transient_strength,
            ),
            AnalysisMetricValue::new("descriptor_confidence", result.confidence.0),
        ]
    }

    fn character_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:tone:sine440",
                    AnalysisCorpusFamily::Tonal,
                    "Sustained tonal descriptor reference",
                ),
                sine_audio(440.0, 2.0, 48_000, 1.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_flatness",
                    Some(0.0),
                    Some(0.05),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.65),
                    Some(0.75),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:noise:deterministic",
                    AnalysisCorpusFamily::Noise,
                    "Broadband descriptor reference",
                ),
                noise_audio(2.0, 48_000, 0.5),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_spread_hz",
                    Some(2_000.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.45),
                    Some(0.55),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:pulse:adsr",
                    AnalysisCorpusFamily::Pulse,
                    "Transient-heavy descriptor reference",
                ),
                adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "peak_transient_strength",
                    Some(0.80),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.25),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    #[test]
    fn spectral_shape_tracks_frequency_position() {
        let low = sine_audio(220.0, 2.0, 48_000, 1.0);
        let high = sine_audio(4_000.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let low_result = analyzer.analyze(&low);
        let high_result = analyzer.analyze(&high);

        assert!(high_result.spectral_shape.centroid_hz > low_result.spectral_shape.centroid_hz);
        assert!(high_result.spectral_shape.rolloff_95_hz > low_result.spectral_shape.rolloff_95_hz);
    }

    #[test]
    fn centroid_near_1khz_for_sine() {
        let audio = sine_audio(1_000.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(
            result.spectral_shape.centroid_hz > 800.0
                && result.spectral_shape.centroid_hz < 1_200.0,
            "centroid was {}",
            result.spectral_shape.centroid_hz,
        );
    }

    #[test]
    fn noise_is_flatter_than_sine() {
        let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
        let noise = noise_audio(2.0, 48_000, 0.5);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let tone_result = analyzer.analyze(&tone);
        let noise_result = analyzer.analyze(&noise);

        assert!(noise_result.spectral_shape.flatness > tone_result.spectral_shape.flatness);
    }

    #[test]
    fn normalized_mel_profile_is_bounded_and_sums_to_one() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        let profile = result.spectral_profile.normalized_mel_band_profile;
        let sum = profile.iter().copied().sum::<f32>();
        assert!((sum - 1.0).abs() < 1e-4, "profile sum was {}", sum);
        assert!(profile.iter().all(|value| *value >= 0.0 && *value <= 1.0));
    }

    #[test]
    fn rms_energy_near_expected_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.rms_energy > 0.6 && result.dynamics.rms_energy < 0.8,
            "rms was {}",
            result.dynamics.rms_energy,
        );
    }

    #[test]
    fn silence_produces_zero_results() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.0; 48_000],
        );
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.spectral_shape, SpectralShapeDescriptorPack::zero());
        assert_eq!(
            result.spectral_contrast,
            SpectralContrastDescriptorPack::zero()
        );
        assert_eq!(
            result.spectral_profile,
            SpectralProfileDescriptorPack::zero()
        );
        assert_eq!(result.temporal, TemporalDescriptorPack::zero());
        assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
        assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
    }

    #[test]
    fn empty_audio_yields_zero_confidence() {
        let audio =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, Vec::new());
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.confidence, Confidence::new(0.0));
        assert_eq!(result.temporal_shape, TemporalShapeDescriptorPack::zero());
        assert_eq!(result.dynamics, DynamicsDescriptorPack::zero());
    }

    #[test]
    fn zcr_near_expected_for_440hz_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.zero_crossing_rate_hz > 800.0
                && result.temporal.zero_crossing_rate_hz < 920.0,
            "zcr was {}",
            result.temporal.zero_crossing_rate_hz,
        );
    }

    #[test]
    fn onset_density_is_finite() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());
        let result = analyzer.analyze(&audio);

        assert!(result.temporal.onset_density.is_finite());
    }

    #[test]
    fn analysis_stage_trait_works() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = <CharacterAnalyzer as AnalysisStage<CharacterAnalysisResult>>::analyze(
            &mut analyzer,
            &audio,
        );

        assert!(result.dynamics.rms_energy > 0.0);
        assert_eq!(analyzer.mode(), AnalysisMode::Offline);
    }

    #[test]
    fn low_profile_still_produces_results() {
        let audio = sine_audio(440.0, 4.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::low());
        let result = analyzer.analyze(&audio);

        assert!(result.spectral_shape.centroid_hz > 0.0);
        assert!(result.dynamics.rms_energy > 0.0);
    }

    #[test]
    fn peak_amplitude_for_full_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.peak_amplitude > 0.95 && result.dynamics.peak_amplitude <= 1.0,
            "peak was {}",
            result.dynamics.peak_amplitude,
        );
    }

    #[test]
    fn peak_amplitude_for_half_scale_sine() {
        let audio = sine_audio(440.0, 1.0, 48_000, 0.5);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.dynamics.peak_amplitude > 0.45 && result.dynamics.peak_amplitude < 0.55,
            "peak was {}",
            result.dynamics.peak_amplitude,
        );
    }

    #[test]
    fn dynamic_range_is_peak_minus_rms() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        let expected = result.dynamics.peak_amplitude - result.dynamics.rms_energy;
        assert!(
            (result.dynamics.dynamic_range - expected).abs() < 1e-6,
            "dynamic_range {} != peak {} - rms {}",
            result.dynamics.dynamic_range,
            result.dynamics.peak_amplitude,
            result.dynamics.rms_energy,
        );
    }

    #[test]
    fn sustain_ratio_near_one_for_loud_signal() {
        let audio = sine_audio(440.0, 1.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.sustain_ratio > 0.95,
            "sustain_ratio was {}",
            result.temporal.sustain_ratio,
        );
    }

    #[test]
    fn sustain_ratio_near_zero_for_very_quiet_signal() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.001; 48_000],
        );
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert_eq!(result.temporal.sustain_ratio, 0.0);
    }

    #[test]
    fn transient_density_is_finite_and_non_negative() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(result.temporal.transient_density.is_finite());
        assert!(result.temporal.transient_density >= 0.0);
    }

    #[test]
    fn transient_density_increases_with_sharp_edges() {
        let sample_rate_hz = 48_000;
        let duration_seconds = 2.0;
        let count = (sample_rate_hz as f32 * duration_seconds) as usize;
        let mut data = vec![0.0f32; count];
        let spacing = 4_800;
        for index in (0..count).step_by(spacing) {
            if index + 1 < count {
                data[index + 1] = 0.5;
            }
        }

        let audio =
            AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
        let result = analyzer.analyze(&audio);

        assert!(
            result.temporal.transient_density > 1.0,
            "transient_density was {}",
            result.temporal.transient_density,
        );
    }

    #[test]
    fn transient_shape_strength_is_higher_for_pulses_than_steady_tone() {
        let pulse = adsr_pulse_audio(5, 10, 10, 350, 6, 48_000, 0.9);
        let tone = sine_audio(440.0, 2.2, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let pulse_result = analyzer.analyze(&pulse);
        let tone_result = analyzer.analyze(&tone);

        assert!(
            pulse_result.temporal_shape.peak_transient_strength
                > tone_result.temporal_shape.peak_transient_strength
        );
        assert!(
            pulse_result.temporal_shape.median_transient_strength
                >= tone_result.temporal_shape.median_transient_strength
        );
    }

    #[test]
    fn temporal_shape_attack_time_tracks_slower_attacks() {
        let sharp = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let slow = adsr_pulse_audio(80, 10, 10, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let sharp_result = analyzer.analyze(&sharp);
        let slow_result = analyzer.analyze(&slow);

        assert!(
            slow_result.temporal_shape.attack_time_ms > sharp_result.temporal_shape.attack_time_ms,
            "slow attack {} ms was not greater than sharp attack {} ms",
            slow_result.temporal_shape.attack_time_ms,
            sharp_result.temporal_shape.attack_time_ms,
        );
    }

    #[test]
    fn temporal_shape_decay_time_tracks_longer_decays() {
        let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let long = adsr_pulse_audio(5, 10, 120, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let short_result = analyzer.analyze(&short);
        let long_result = analyzer.analyze(&long);

        assert!(
            long_result.temporal_shape.decay_time_ms > short_result.temporal_shape.decay_time_ms
        );
    }

    #[test]
    fn temporal_shape_sustain_ratio_tracks_longer_plateaus() {
        let short = adsr_pulse_audio(5, 10, 10, 400, 6, 48_000, 0.9);
        let long = adsr_pulse_audio(5, 140, 10, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let short_result = analyzer.analyze(&short);
        let long_result = analyzer.analyze(&long);

        assert!(
            long_result.temporal_shape.sustain_plateau_ratio
                > short_result.temporal_shape.sustain_plateau_ratio
        );
    }

    #[test]
    fn harness_character_descriptor_cases_meet_frozen_acceptance_thresholds() {
        let cases = character_acceptance_cases();
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let report = run_audio_acceptance_harness(
            &cases,
            |audio| analyzer.analyze(audio),
            character_metrics,
        );

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_character_acceptance_report_remains_interpretable_for_closeout() {
        let cases = character_acceptance_cases();
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let report = run_audio_acceptance_harness(
            &cases,
            |audio| analyzer.analyze(audio),
            character_metrics,
        );

        println!("character_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }

    #[test]
    fn descriptor_pack_examples_remain_interpretable_for_closeout() {
        let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
        let noise = noise_audio(2.0, 48_000, 0.5);
        let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let tone_result = analyzer.analyze(&tone);
        let noise_result = analyzer.analyze(&noise);
        let pulse_result = analyzer.analyze(&pulse);

        println!("tone_result={:#?}", tone_result);
        println!("noise_result={:#?}", noise_result);
        println!("pulse_result={:#?}", pulse_result);

        assert!(tone_result.spectral_shape.flatness < noise_result.spectral_shape.flatness);
        assert!(noise_result.spectral_shape.spread_hz > tone_result.spectral_shape.spread_hz);
        assert!(
            noise_result.spectral_contrast.contrast_db < tone_result.spectral_contrast.contrast_db
        );
        assert!(
            pulse_result.temporal_shape.peak_transient_strength
                > tone_result.temporal_shape.peak_transient_strength
        );
        assert!(pulse_result.temporal.onset_density > tone_result.temporal.onset_density);
        assert!(pulse_result.temporal_shape.sustain_plateau_ratio > 0.0);
        assert!(
            pulse_result.temporal_shape.decay_time_ms > pulse_result.temporal_shape.attack_time_ms
        );
    }

    #[test]
    fn reduction_policy_is_frozen_to_expected_modes() {
        let policy = CharacterDescriptorReductionPolicy::default();

        assert_eq!(
            policy.spectral_centroid_hz,
            DescriptorReduction::MedianAcrossFrames
        );
        assert_eq!(
            policy.normalized_mel_band_profile,
            DescriptorReduction::MeanAcrossFramesNormalized
        );
        assert_eq!(policy.rms_energy, DescriptorReduction::WholeSignal);
        assert_eq!(
            policy.peak_transient_strength,
            DescriptorReduction::PeakAcrossEvents
        );
        assert_eq!(
            policy.attack_time_ms,
            DescriptorReduction::MedianAcrossEvents
        );
    }

    #[test]
    fn non_native_input_rate_preserves_descriptor_shape_under_frozen_analysis_rate() {
        let native = sine_audio(1_000.0, 2.0, 48_000, 1.0);
        let non_native = sine_audio(1_000.0, 2.0, 44_100, 1.0);
        let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::medium());

        let native_result = analyzer.analyze(&native);
        let non_native_result = analyzer.analyze(&non_native);

        assert!(
            (native_result.spectral_shape.centroid_hz
                - non_native_result.spectral_shape.centroid_hz)
                .abs()
                < 80.0,
            "centroid drifted from {} to {}",
            native_result.spectral_shape.centroid_hz,
            non_native_result.spectral_shape.centroid_hz,
        );
        assert!(
            (native_result.dynamics.rms_energy - non_native_result.dynamics.rms_energy).abs()
                < 0.05
        );
        assert!(
            (native_result.temporal.zero_crossing_rate_hz
                - non_native_result.temporal.zero_crossing_rate_hz)
                .abs()
                < 25.0
        );
    }
}
