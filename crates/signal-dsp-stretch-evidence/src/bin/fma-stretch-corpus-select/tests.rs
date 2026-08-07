use std::path::{Path, PathBuf};

use super::args::SelectorArgs;
use super::report::{format_fma_selection_report, format_fma_selection_tsv, genre_summary};
use super::select::{
    classify_family, duration_is_in_candidate_window, family_count, fma_large_path,
    review_seed_candidates, FmaCandidate, FmaCorpusFamily,
};

#[test]
fn fma_large_path_uses_padded_directory_shape() {
    assert_eq!(
        fma_large_path(Path::new("/fma"), 135054).unwrap(),
        PathBuf::from("/fma/fma_large/135/135054.mp3")
    );
    assert_eq!(
        fma_large_path(Path::new("/fma"), 2).unwrap(),
        PathBuf::from("/fma/fma_large/000/000002.mp3")
    );
}

#[test]
fn family_classification_is_deterministic_by_priority() {
    let selected = Vec::new();

    assert_eq!(
        classify_family("genre_title': 'Hip-Hop'", "Artist", &selected, 1),
        Some(FmaCorpusFamily::DrumsPercussion)
    );
    assert_eq!(
        classify_family("genre_title': 'Ambient'", "Artist", &selected, 1),
        Some(FmaCorpusFamily::PadsSustains)
    );
}

#[test]
fn duration_window_accepts_practical_tracks() {
    assert!(duration_is_in_candidate_window("01:00"));
    assert!(duration_is_in_candidate_window("08:00"));
    assert!(!duration_is_in_candidate_window("00:59"));
    assert!(!duration_is_in_candidate_window("08:01"));
    assert!(!duration_is_in_candidate_window(""));
}

#[test]
fn report_includes_source_boundary_and_cases() {
    let args = SelectorArgs::default();
    let candidates = vec![test_candidate()];

    let report = format_fma_selection_report(&args, &candidates);

    assert!(report.contains("Do not commit FMA audio files."));
    assert!(report.contains("Signal case: `stretch:full_mix`"));
    assert!(report.contains("`/fma/fma_large/000/000010.mp3`"));
}

#[test]
fn tsv_includes_report_manifest_fields() {
    let tsv = format_fma_selection_tsv(&[test_candidate()]).expect("format tsv");

    assert!(tsv.starts_with("case_id\tfamily\ttrack_id\tartist"));
    assert!(tsv.contains("stretch:full_mix\tfull_mix\t10\tArtist\tTrack"));
    assert!(tsv.contains("Attribution\thttps://example.test/license"));
    assert!(tsv.contains("https://example.test/track\t/fma/fma_large/000/000010.mp3"));
}

#[test]
fn review_seed_caps_family_count_and_deduplicates_artists_when_possible() {
    let candidates = vec![
        test_candidate_with(FmaCorpusFamily::DrumsPercussion, 1, "Shared"),
        test_candidate_with(FmaCorpusFamily::DrumsPercussion, 2, "Percussion Two"),
        test_candidate_with(FmaCorpusFamily::Bass, 3, "Shared"),
        test_candidate_with(FmaCorpusFamily::Bass, 4, "Bass Two"),
        test_candidate_with(FmaCorpusFamily::Vocals, 5, "Vocal One"),
        test_candidate_with(FmaCorpusFamily::Vocals, 6, "Vocal Two"),
    ];

    let seed = review_seed_candidates(&candidates, 1);

    assert_eq!(family_count(&seed, FmaCorpusFamily::DrumsPercussion), 1);
    assert_eq!(family_count(&seed, FmaCorpusFamily::Bass), 1);
    assert_eq!(family_count(&seed, FmaCorpusFamily::Vocals), 1);
    assert!(seed
        .iter()
        .any(|candidate| candidate.family == FmaCorpusFamily::Bass
            && candidate.artist_name == "Bass Two"));
}

#[test]
fn genre_summary_extracts_fma_titles() {
    let raw = "[{'genre_id': '10', 'genre_title': 'Pop'}, {'genre_title': 'Rock'}]";

    assert_eq!(genre_summary(raw), "Pop, Rock");
}

fn test_candidate() -> FmaCandidate {
    test_candidate_with(FmaCorpusFamily::FullMix, 10, "Artist")
}

fn test_candidate_with(family: FmaCorpusFamily, track_id: u32, artist_name: &str) -> FmaCandidate {
    FmaCandidate {
        family,
        track_id,
        artist_name: artist_name.to_string(),
        track_title: "Track".to_string(),
        album_title: "Album".to_string(),
        duration: "01:00".to_string(),
        license_title: "Attribution".to_string(),
        license_url: "https://example.test/license".to_string(),
        track_url: "https://example.test/track".to_string(),
        genres: "Rock".to_string(),
        local_path: PathBuf::from(format!("/fma/fma_large/000/{track_id:06}.mp3")),
    }
}
