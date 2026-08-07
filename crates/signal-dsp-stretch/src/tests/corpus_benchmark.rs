use super::*;

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
