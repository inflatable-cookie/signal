#[cfg(test)]
mod tests {
    use crate::{
        cents_offset_from_standard, reference_hz_from_cents, HarmonicChangeKind, KeyDetector,
        KeyDetectorConfig, KeyMode, KeyProfile, TonalAmbiguityKind, Tonic, TuningReferenceMode,
        TuningReferenceSource,
    };
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn tonal_mix(sample_rate: u32, freqs: &[f32], seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = vec![0.0f32; frames];
        let scale = if freqs.is_empty() {
            0.0
        } else {
            1.0 / freqs.len() as f32
        };

        for (index, sample) in samples.iter_mut().enumerate() {
            let t = index as f32 / sample_rate as f32;
            let mut value = 0.0;
            for freq in freqs {
                value += (core::f32::consts::TAU * *freq * t).sin();
            }
            *sample = value * scale;
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn detuned_tonal_mix(
        sample_rate: u32,
        freqs: &[f32],
        seconds: f32,
        reference_hz: f32,
    ) -> AudioBuffer {
        let ratio = reference_hz / 440.0;
        let detuned: Vec<f32> = freqs.iter().map(|frequency| frequency * ratio).collect();
        tonal_mix(sample_rate, &detuned, seconds)
    }

    fn tonal_sequence_mix(sample_rate: u32, sections: &[(&[f32], f32)]) -> AudioBuffer {
        let mut samples = Vec::new();
        for (freqs, seconds) in sections {
            samples.extend_from_slice(tonal_mix(sample_rate, freqs, *seconds).samples());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn tonic_metric(key: Option<crate::Key>) -> f32 {
        match key.map(|key| key.tonic) {
            Some(Tonic::C) => 0.0,
            Some(Tonic::Cs) => 1.0,
            Some(Tonic::D) => 2.0,
            Some(Tonic::Ds) => 3.0,
            Some(Tonic::E) => 4.0,
            Some(Tonic::F) => 5.0,
            Some(Tonic::Fs) => 6.0,
            Some(Tonic::G) => 7.0,
            Some(Tonic::Gs) => 8.0,
            Some(Tonic::A) => 9.0,
            Some(Tonic::As) => 10.0,
            Some(Tonic::B) => 11.0,
            None => -1.0,
        }
    }

    fn mode_metric(key: Option<crate::Key>) -> f32 {
        match key.map(|key| key.mode) {
            Some(KeyMode::Major) => 0.0,
            Some(KeyMode::Minor) => 1.0,
            None => -1.0,
        }
    }

    fn count_ambiguities(result: &crate::TonalAnalysisResult, kind: TonalAmbiguityKind) -> usize {
        result
            .local_tracking
            .ambiguities
            .iter()
            .filter(|ambiguity| ambiguity.kind == kind)
            .count()
    }

    fn tonal_metrics(result: &crate::TonalAnalysisResult) -> Vec<AnalysisMetricValue> {
        let first_segment = result
            .local_tracking
            .segments
            .first()
            .and_then(|segment| segment.key);
        let last_segment = result
            .local_tracking
            .segments
            .last()
            .and_then(|segment| segment.key);

        vec![
            AnalysisMetricValue::new("key_tonic", tonic_metric(result.key)),
            AnalysisMetricValue::new("key_mode", mode_metric(result.key)),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("tuning_reference_hz", result.tuning.reference_hz),
            AnalysisMetricValue::new("tuning_cents_offset", result.tuning.cents_offset),
            AnalysisMetricValue::new(
                "local_segment_count",
                result.local_tracking.segments.len() as f32,
            ),
            AnalysisMetricValue::new(
                "local_change_count",
                result.local_tracking.changes.len() as f32,
            ),
            AnalysisMetricValue::new(
                "local_ambiguity_count",
                result.local_tracking.ambiguities.len() as f32,
            ),
            AnalysisMetricValue::new(
                "modulation_ambiguity_count",
                count_ambiguities(result, TonalAmbiguityKind::Modulation) as f32,
            ),
            AnalysisMetricValue::new("first_segment_tonic", tonic_metric(first_segment)),
            AnalysisMetricValue::new("last_segment_tonic", tonic_metric(last_segment)),
        ]
    }

    fn tonal_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:c-major-triad",
                    AnalysisCorpusFamily::Tonal,
                    "Stable C-major global and local key reference",
                ),
                tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "key_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "key_mode",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.01),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_reference_hz",
                    Some(438.0),
                    Some(442.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "local_ambiguity_count",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:detuned-c-major-432",
                    AnalysisCorpusFamily::RatePolicy,
                    "Detuned tuning-reference reference",
                ),
                detuned_tonal_mix(48_000, &[261.63, 329.63, 392.0], 5.0, 432.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "key_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "key_mode",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_reference_hz",
                    Some(429.5),
                    Some(434.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_cents_offset",
                    Some(-40.0),
                    Some(-20.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:modulation-c-to-g",
                    AnalysisCorpusFamily::Tonal,
                    "Section-local modulation and ambiguity reference",
                ),
                tonal_sequence_mix(
                    48_000,
                    &[
                        (&[261.63, 329.63, 392.0], 6.0),
                        (&[196.0, 246.94, 293.66], 6.0),
                    ],
                ),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "local_segment_count",
                    Some(2.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "local_change_count",
                    Some(1.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "modulation_ambiguity_count",
                    Some(1.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "first_segment_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "last_segment_tonic",
                    Some(7.0),
                    Some(7.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    #[test]
    fn key_detector_finds_c_major_triad() {
        let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::C);
        assert_eq!(result.key.unwrap().mode, KeyMode::Major);
        assert!(result.confidence.0 > 0.01);
        assert_eq!(result.tuning.source, TuningReferenceSource::Estimated);
        assert!((result.tuning.reference_hz - 440.0).abs() <= 2.0);
        assert_eq!(result.scoring.profile, KeyProfile::Krumhansl);
        assert_eq!(result.scoring.best.unwrap().key.tonic, Tonic::C);
    }

    #[test]
    fn key_detector_finds_a_minor_triad() {
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::A);
        assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
        assert!(result.confidence.0 > 0.001);
    }

    #[test]
    fn low_profile_still_detects_key() {
        let audio = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::low());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::C);
        assert_eq!(result.key.unwrap().mode, KeyMode::Major);
        assert_eq!(
            detector.config().tuning_reference,
            TuningReferenceMode::Estimate
        );
        assert_eq!(detector.config().tuning_step_cents, 10);
    }

    #[test]
    fn medium_profile_still_detects_key() {
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::A);
        assert_eq!(result.key.unwrap().mode, KeyMode::Minor);
        assert_eq!(detector.config().tuning_step_cents, 5);
    }

    #[test]
    fn pearson_distinguishes_relative_major_minor() {
        // A minor chord (A-C-E) should be detected as A minor, not C major,
        // even though they share the same pitch classes.
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(key.tonic, Tonic::A);
        assert_eq!(key.mode, KeyMode::Minor);

        // The A minor correlation (index 12+9=21) should exceed C major (index 0).
        assert!(
            result.correlations[21] > result.correlations[0],
            "A minor correlation ({}) should exceed C major ({})",
            result.correlations[21],
            result.correlations[0],
        );
    }

    #[test]
    fn b_minor_bass_detected_correctly_at_44100() {
        // B minor triad rooted in bass register (B2-D4-F#4), at 44100 Hz.
        // With a 4096-point FFT, B2 (123.47 Hz) falls between bins that map
        // to A# and C — no bin maps to B.  The 8192-point FFT used by
        // medium/high profiles fixes this.
        let audio = tonal_mix(44_100, &[123.47, 293.66, 369.99], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(
            key.tonic,
            Tonic::B,
            "Expected B but got {:?}; chroma = {:?}",
            key.tonic,
            result.chroma,
        );
        assert_eq!(key.mode, KeyMode::Minor);
    }

    #[test]
    fn b_minor_bass_detected_correctly_at_48000() {
        let audio = tonal_mix(48_000, &[123.47, 293.66, 369.99], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        let key = result.key.unwrap();
        assert_eq!(
            key.tonic,
            Tonic::B,
            "Expected B but got {:?}; chroma = {:?}",
            key.tonic,
            result.chroma,
        );
        assert_eq!(key.mode, KeyMode::Minor);
    }

    #[test]
    fn non_native_input_rate_preserves_key_under_frozen_analysis_rate() {
        let native = tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0);
        let non_native = tonal_mix(44_100, &[261.63, 329.63, 392.0], 4.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::default());

        let native_result = detector.analyze(&native);
        let non_native_result = detector.analyze(&non_native);

        assert_eq!(native_result.key, non_native_result.key);
        assert!(
            (native_result.confidence.0 - non_native_result.confidence.0).abs() < 0.1,
            "confidence drifted from {} to {}",
            native_result.confidence.0,
            non_native_result.confidence.0,
        );
    }

    #[test]
    fn detector_estimates_detuned_reference_for_c_major_material() {
        let audio = detuned_tonal_mix(48_000, &[261.63, 329.63, 392.0], 5.0, 432.0);
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        assert_eq!(result.key.unwrap().tonic, Tonic::C);
        assert_eq!(result.key.unwrap().mode, KeyMode::Major);
        assert_eq!(result.tuning.source, TuningReferenceSource::Estimated);
        assert!((result.tuning.reference_hz - 432.0).abs() <= 2.5);
        assert!(result.tuning.cents_offset < -20.0);
        assert!(result.tuning.runner_up.is_some());
        assert!(result.scoring.runner_up.is_some());
    }

    #[test]
    fn fixed_tuning_reference_is_reported_explicitly() {
        let audio = tonal_mix(48_000, &[220.0, 261.63, 329.63], 4.0);
        let mut config = KeyDetectorConfig::medium();
        config.tuning_reference = TuningReferenceMode::Fixed(442.0);
        let mut detector = KeyDetector::new(config);
        let result = detector.analyze(&audio);

        assert_eq!(result.tuning.source, TuningReferenceSource::FixedReference);
        assert!((result.tuning.reference_hz - 442.0).abs() < 0.01);
        assert!(result.tuning.confidence.0 >= 1.0);
        assert!((result.tuning.cents_offset - cents_offset_from_standard(442.0)).abs() < 0.01);
    }

    #[test]
    fn tuning_reference_helpers_round_trip_standard_offsets() {
        let offset = -31.766;
        let reference = reference_hz_from_cents(offset);

        assert!((reference - 432.0).abs() < 1.5);
        assert!((cents_offset_from_standard(reference) - offset).abs() < 0.1);
    }

    #[test]
    fn detector_exposes_stable_local_key_tracking_for_c_major_sections() {
        let audio = tonal_sequence_mix(
            48_000,
            &[
                (&[261.63, 329.63, 392.0], 6.0),
                (&[261.63, 329.63, 392.0], 6.0),
            ],
        );
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        assert!(result.local_tracking.segments.len() >= 2);
        assert!(result.local_tracking.changes.is_empty());
        assert!(
            result.local_tracking.ambiguities.is_empty(),
            "unexpected ambiguities: {:?}",
            result.local_tracking
        );
        assert!(result
            .local_tracking
            .segments
            .iter()
            .all(|segment| segment.key
                == Some(crate::Key {
                    tonic: Tonic::C,
                    mode: KeyMode::Major
                })));
    }

    #[test]
    fn detector_exposes_local_key_shift_and_harmonic_change_for_modulation() {
        let audio = tonal_sequence_mix(
            48_000,
            &[
                (&[261.63, 329.63, 392.0], 6.0),
                (&[196.0, 246.94, 293.66], 6.0),
            ],
        );
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        assert!(result.local_tracking.segments.len() >= 2);
        let first = result
            .local_tracking
            .segments
            .first()
            .expect("first local segment");
        let last = result
            .local_tracking
            .segments
            .last()
            .expect("last local segment");
        assert_eq!(
            first.key,
            Some(crate::Key {
                tonic: Tonic::C,
                mode: KeyMode::Major,
            })
        );
        assert_eq!(
            last.key,
            Some(crate::Key {
                tonic: Tonic::G,
                mode: KeyMode::Major,
            })
        );
        let change = result
            .local_tracking
            .changes
            .iter()
            .find(|change| change.kind == HarmonicChangeKind::ConfirmedKeyChange)
            .expect("confirmed key change");
        assert_eq!(
            change.from_key,
            Some(crate::Key {
                tonic: Tonic::C,
                mode: KeyMode::Major,
            })
        );
        assert_eq!(
            change.to_key,
            Some(crate::Key {
                tonic: Tonic::G,
                mode: KeyMode::Major,
            })
        );
        assert!(change.confidence.0 > 0.1);
        assert!(change.chroma_distance.0 > 0.2);
        let ambiguity = result
            .local_tracking
            .ambiguities
            .iter()
            .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::Modulation)
            .expect("modulation ambiguity");
        assert_eq!(
            ambiguity.primary_key,
            Some(crate::Key {
                tonic: Tonic::C,
                mode: KeyMode::Major,
            })
        );
        assert_eq!(
            ambiguity.alternate_key,
            Some(crate::Key {
                tonic: Tonic::G,
                mode: KeyMode::Major,
            })
        );
    }

    #[test]
    fn detector_surfaces_weak_tonal_centre_ambiguity() {
        let audio = tonal_mix(
            48_000,
            &[
                261.63, 277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 392.0, 415.3, 440.0,
                466.16, 493.88,
            ],
            8.0,
        );
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
        let result = detector.analyze(&audio);

        let ambiguity = result
            .local_tracking
            .ambiguities
            .iter()
            .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::WeakTonalCenter)
            .unwrap_or_else(|| panic!("weak tonal-centre ambiguity: {:?}", result.local_tracking));
        assert!(ambiguity.confidence.0 >= 0.5);
        assert!(result
            .local_tracking
            .segments
            .iter()
            .all(|segment| matches!(
                segment.ambiguity,
                Some(crate::TonalSegmentAmbiguitySummary {
                    kind: TonalAmbiguityKind::WeakTonalCenter,
                    ..
                })
            )));
    }

    #[test]
    fn detector_surfaces_mixed_tonality_ambiguity_for_competing_sections() {
        let audio = tonal_sequence_mix(
            48_000,
            &[
                (&[261.63, 329.63, 392.0], 4.0),
                (&[196.0, 246.94, 293.66], 4.0),
                (&[261.63, 329.63, 392.0], 4.0),
            ],
        );
        let mut config = KeyDetectorConfig::medium();
        config.section_window_seconds = 4;
        config.section_hop_seconds = 2;
        let mut detector = KeyDetector::new(config);
        let result = detector.analyze(&audio);

        let ambiguity = result
            .local_tracking
            .ambiguities
            .iter()
            .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::MixedTonality)
            .unwrap_or_else(|| panic!("mixed-tonality ambiguity: {:?}", result.local_tracking));
        assert!(
            ambiguity.confidence.0 > 0.1,
            "mixed ambiguity too weak: {:?}",
            result.local_tracking
        );
        assert_eq!(
            ambiguity.primary_key,
            Some(crate::Key {
                tonic: Tonic::C,
                mode: KeyMode::Major,
            })
        );
        assert_eq!(
            ambiguity.alternate_key,
            Some(crate::Key {
                tonic: Tonic::G,
                mode: KeyMode::Major,
            })
        );
    }

    #[test]
    fn harness_tonal_cases_meet_frozen_acceptance_thresholds() {
        let cases = tonal_acceptance_cases();
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());

        let report =
            run_audio_acceptance_harness(&cases, |audio| detector.analyze(audio), tonal_metrics);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_tonal_acceptance_report_remains_interpretable_for_closeout() {
        let cases = tonal_acceptance_cases();
        let mut detector = KeyDetector::new(KeyDetectorConfig::medium());

        let report =
            run_audio_acceptance_harness(&cases, |audio| detector.analyze(audio), tonal_metrics);

        println!("tonal_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }
}
