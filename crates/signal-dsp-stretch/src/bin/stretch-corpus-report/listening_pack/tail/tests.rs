use super::*;
use std::path::PathBuf;

#[test]
fn shared_tail_gain_preserves_candidate_ratios_and_peak_ceiling() {
    let current = vec![0.1_f32; 100];
    let source = vec![0.2_f32; 100];
    let zero = vec![0.4_f32; 100];
    let gain = shared_tail_gain(&current, [&source, &zero]).expect("shared gain");

    assert!((gain - 1.5).abs() < 1.0e-6);
    assert!(((source[0] as f64 * gain) / (current[0] as f64 * gain) - 2.0).abs() < 1.0e-9);
    assert!(zero[0] as f64 * gain <= PEAK_CEILING);
}

#[test]
fn assignment_is_deterministic_and_contains_every_candidate() {
    let render = ExternalBenchmarkQualityRender {
        case_id: "stretch:test".to_string(),
        ratio: 1.5,
        tool_name: "unused".to_string(),
        rendered_path: "unused.wav".to_string(),
        source_wav: Some("source.wav".to_string()),
    };
    let first = stable_tail_assignment(&render, "source.wav");
    let second = stable_tail_assignment(&render, "source.wav");

    assert_eq!(first, second);
    assert!(first.contains(&TailCandidate::Current));
    assert!(first.contains(&TailCandidate::AdditiveZeroAnchor));
    assert!(first.contains(&TailCandidate::MultiplicativeZeroFade));
}

#[test]
fn selection_is_bounded_and_prioritizes_largest_endpoint_corrections() {
    let mut rows = (0..10)
        .map(|index| TailReviewRow {
            render_index: index,
            endpoint_correction: index as f64 / 10.0,
            source_path: format!("source-{index}"),
            spectral_centroid_hz: 1_000.0,
            centroid_band: TailCentroidBand::BelowThreshold,
        })
        .collect::<Vec<_>>();

    bound_tail_review_rows(&mut rows);

    assert_eq!(rows.len(), REVIEW_ROWS);
    assert_eq!(rows[0].endpoint_correction, 0.9);
    assert_eq!(rows.last().expect("last selected").endpoint_correction, 0.4);
}

#[test]
fn classifier_selection_balances_bands_and_deduplicates_sources() {
    let mut rows = Vec::new();
    for (index, centroid_band) in [
        TailCentroidBand::BelowThreshold,
        TailCentroidBand::AtOrAboveThreshold,
    ]
    .into_iter()
    .enumerate()
    {
        for source in 0..4 {
            for ratio in 0..2 {
                rows.push(TailReviewRow {
                    render_index: index * 100 + source * 10 + ratio,
                    endpoint_correction: (10 - source * 2 - ratio) as f64,
                    source_path: format!("band-{index}-source-{source}"),
                    spectral_centroid_hz: if index == 0 { 1_000.0 } else { 3_000.0 },
                    centroid_band,
                });
            }
        }
    }

    let selected = bound_classifier_validation_rows(rows);

    assert_eq!(selected.len(), 6);
    assert_eq!(
        selected
            .iter()
            .map(|row| row.source_path.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        6
    );
    assert_eq!(
        selected
            .iter()
            .filter(|row| row.centroid_band == TailCentroidBand::BelowThreshold)
            .count(),
        3
    );
}

#[test]
fn pack_exports_concealed_mono_tails_with_post_tail_silence() {
    let root = PathBuf::from(format!(
        "target/stretch-tail-pack-test-{}",
        std::process::id()
    ));
    let source_path = PathBuf::from(format!(
        "target/stretch-tail-pack-source-test-{}.wav",
        std::process::id()
    ));
    let samples = (0..4_096)
        .map(|frame| (std::f32::consts::TAU * frame as f32 / 61.0).sin() * 0.25)
        .collect::<Vec<_>>();
    super::super::write_float_wav(&source_path, 48_000, 1, &samples).expect("write source");
    let renders = vec![ExternalBenchmarkQualityRender {
        case_id: "stretch:test".to_string(),
        ratio: 1.5,
        tool_name: "unused".to_string(),
        rendered_path: "unused.wav".to_string(),
        source_wav: Some(source_path.display().to_string()),
    }];

    let report =
        export_tail_listening_pack(&[], &renders, 4_096, OfflineHighQualityPath::Default, &root)
            .expect("export tail pack");

    assert!(report.contains(
        "status=ReadyForOperator selection=worst-endpoints trials=1 candidates_per_trial=3"
    ));
    let notes = fs::read_to_string(root.join("tail-listening-notes.tsv")).expect("notes");
    let key = fs::read_to_string(root.join("tail-listening-key.tsv")).expect("key");
    assert!(!notes.contains("additive-zero-anchor"));
    assert!(!notes.contains("multiplicative-zero-fade"));
    assert!(key.contains("current"));
    assert!(key.contains("additive-zero-anchor"));
    assert!(key.contains("multiplicative-zero-fade"));
    assert_eq!(
        fs::read_dir(root.join("trials")).expect("trials").count(),
        3
    );
    let mut reader =
        hound::WavReader::open(root.join("trials/T001-A.wav")).expect("open candidate");
    assert_eq!(reader.spec().channels, 1);
    let rendered = reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .expect("read candidate");
    assert!(rendered.len() > 12_000);
    assert!(rendered[rendered.len() - 12_000..]
        .iter()
        .all(|sample| *sample == 0.0));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(source_path);
}
