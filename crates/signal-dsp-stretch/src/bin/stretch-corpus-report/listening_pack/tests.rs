use super::*;

#[test]
fn level_matching_uses_common_rms_and_peak_ceiling() {
    let source = vec![0.25_f32; 256];
    let signal = vec![0.5_f32; 512];
    let external = vec![0.125_f32; 512];

    let matched = level_match_group(&source, &signal, &external).expect("level match");

    let source_rms = level_stats(&matched.source).expect("source stats").rms;
    let signal_rms = level_stats(&matched.signal).expect("signal stats").rms;
    let external_rms = level_stats(&matched.external).expect("external stats").rms;
    assert!((source_rms - signal_rms).abs() < 1.0e-6);
    assert!((source_rms - external_rms).abs() < 1.0e-6);
    assert!(matched
        .source
        .iter()
        .chain(&matched.signal)
        .chain(&matched.external)
        .all(|sample| sample.abs() <= PEAK_CEILING as f32));
}

#[test]
fn blind_pack_exports_one_level_matched_pair_per_required_family() {
    let root = PathBuf::from(format!(
        "target/stretch-blind-pack-test-{}",
        std::process::id()
    ));
    let source_path = PathBuf::from(format!(
        "target/stretch-blind-pack-source-test-{}.wav",
        std::process::id()
    ));
    let samples = (0..4_096)
        .flat_map(|frame| {
            let sample = (std::f32::consts::TAU * frame as f32 / 64.0).sin() * 0.25;
            [sample, sample * 0.8]
        })
        .collect::<Vec<_>>();
    write_float_wav(&source_path, 48_000, 2, &samples).expect("write test source");
    let renders = REQUIRED_FAMILIES
        .iter()
        .flat_map(|case_id| {
            REQUIRED_RATIOS.map(|ratio| ExternalBenchmarkQualityRender {
                case_id: (*case_id).to_string(),
                ratio,
                tool_name: "rubberband-cli".to_string(),
                rendered_path: source_path.display().to_string(),
                source_wav: Some(source_path.display().to_string()),
            })
        })
        .collect::<Vec<_>>();

    let report =
        export_blind_listening_pack(&[], &renders, 4_096, OfflineHighQualityPath::Default, &root)
            .expect("export blind pack");

    assert!(report.contains("status=ReadyForOperator pairs=15 families=5"));
    let notes =
        fs::read_to_string(root.join("blind-listening-notes.tsv")).expect("read notes manifest");
    let key =
        fs::read_to_string(root.join("blind-listening-key.tsv")).expect("read assignment key");
    assert_eq!(notes.lines().count(), 16);
    assert!(notes.contains("transient\ttonal\tstereo\tformant\tboundary"));
    assert!(key.contains("Signal::Default"));
    assert!(key.contains("rubberband-cli"));
    let status = format_blind_listening_note_status(&root.join("blind-listening-notes.tsv"))
        .expect("inspect empty notes");
    assert!(status.contains("status=Incomplete"));
    assert!(status.contains("completed_families=0 required_families=5"));
    assert_eq!(
        fs::read_dir(root.join("pairs"))
            .expect("pair files")
            .count(),
        30
    );

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(source_path);
}
