use super::super::*;

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
