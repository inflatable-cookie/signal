//! Select local FMA candidates for Signal stretch corpus listening runs.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_FMA_ROOT: &str = "/Users/tom/Downloads/FMA";
const DEFAULT_PER_FAMILY: usize = 5;
const DEFAULT_REVIEW_SEED_PER_FAMILY: usize = 2;
const DEFAULT_OUTPUT: &str = "target/stretch-corpus-fma-selection.md";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FmaCorpusFamily {
    DrumsPercussion,
    Bass,
    Vocals,
    PadsSustains,
    FullMix,
}

impl FmaCorpusFamily {
    const ALL: [Self; 5] = [
        Self::DrumsPercussion,
        Self::Bass,
        Self::Vocals,
        Self::PadsSustains,
        Self::FullMix,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::DrumsPercussion => "drums_percussion",
            Self::Bass => "bass",
            Self::Vocals => "vocals",
            Self::PadsSustains => "pads_sustains",
            Self::FullMix => "full_mix",
        }
    }

    fn signal_case_id(self) -> &'static str {
        match self {
            Self::DrumsPercussion => "stretch:drums_percussion",
            Self::Bass => "stretch:bass",
            Self::Vocals => "stretch:vocals",
            Self::PadsSustains => "stretch:pads_sustains",
            Self::FullMix => "stretch:full_mix",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SelectorArgs {
    fma_root: PathBuf,
    metadata: PathBuf,
    output: PathBuf,
    tsv_output: Option<PathBuf>,
    review_seed_tsv_output: Option<PathBuf>,
    per_family: usize,
    review_seed_per_family: usize,
}

impl Default for SelectorArgs {
    fn default() -> Self {
        let fma_root = PathBuf::from(DEFAULT_FMA_ROOT);
        Self {
            metadata: fma_root.join("fma_metadata/raw_tracks.csv"),
            fma_root,
            output: PathBuf::from(DEFAULT_OUTPUT),
            tsv_output: None,
            review_seed_tsv_output: None,
            per_family: DEFAULT_PER_FAMILY,
            review_seed_per_family: DEFAULT_REVIEW_SEED_PER_FAMILY,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FmaCandidate {
    family: FmaCorpusFamily,
    track_id: u32,
    artist_name: String,
    track_title: String,
    album_title: String,
    duration: String,
    license_title: String,
    license_url: String,
    track_url: String,
    genres: String,
    local_path: PathBuf,
}

fn main() {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return;
    }

    let args = match parse_args(raw_args) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            process::exit(2);
        }
    };

    let candidates = match select_fma_candidates(&args) {
        Ok(candidates) => candidates,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };
    let report = format_fma_selection_report(&args, &candidates);

    if let Some(parent) = args.output.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create {}: {error}", parent.display());
            process::exit(1);
        }
    }
    if let Err(error) = fs::write(&args.output, report) {
        eprintln!("failed to write {}: {error}", args.output.display());
        process::exit(1);
    }
    println!("wrote {}", args.output.display());

    if let Some(tsv_output) = &args.tsv_output {
        let tsv = match format_fma_selection_tsv(&candidates) {
            Ok(tsv) => tsv,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        if let Some(parent) = tsv_output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            }
        }
        if let Err(error) = fs::write(tsv_output, tsv) {
            eprintln!("failed to write {}: {error}", tsv_output.display());
            process::exit(1);
        }
        println!("wrote {}", tsv_output.display());
    }

    if let Some(review_seed_tsv_output) = &args.review_seed_tsv_output {
        let review_seed = review_seed_candidates(&candidates, args.review_seed_per_family);
        let tsv = match format_fma_selection_tsv(&review_seed) {
            Ok(tsv) => tsv,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        if let Some(parent) = review_seed_tsv_output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            }
        }
        if let Err(error) = fs::write(review_seed_tsv_output, tsv) {
            eprintln!(
                "failed to write {}: {error}",
                review_seed_tsv_output.display()
            );
            process::exit(1);
        }
        println!("wrote {}", review_seed_tsv_output.display());
    }
}

fn parse_args<I>(args: I) -> Result<SelectorArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = SelectorArgs::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--fma-root" => {
                parsed.fma_root = PathBuf::from(next_value(&mut iter, "--fma-root")?);
                parsed.metadata = parsed.fma_root.join("fma_metadata/raw_tracks.csv");
            }
            "--metadata" => {
                parsed.metadata = PathBuf::from(next_value(&mut iter, "--metadata")?);
            }
            "--output" => {
                parsed.output = PathBuf::from(next_value(&mut iter, "--output")?);
            }
            "--tsv-output" => {
                parsed.tsv_output = Some(PathBuf::from(next_value(&mut iter, "--tsv-output")?));
            }
            "--review-seed-tsv-output" => {
                parsed.review_seed_tsv_output = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--review-seed-tsv-output",
                )?));
            }
            "--per-family" => {
                parsed.per_family = next_value(&mut iter, "--per-family")?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --per-family value: {error}"))?;
                if parsed.per_family == 0 {
                    return Err("--per-family must be greater than zero".to_string());
                }
            }
            "--review-seed-per-family" => {
                parsed.review_seed_per_family = next_value(&mut iter, "--review-seed-per-family")?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --review-seed-per-family value: {error}"))?;
                if parsed.review_seed_per_family == 0 {
                    return Err("--review-seed-per-family must be greater than zero".to_string());
                }
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}"));
            }
        }
    }
    Ok(parsed)
}

fn next_value<I>(iter: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing value for {name}"))
}

fn usage() -> &'static str {
    "usage: fma-stretch-corpus-select [--fma-root PATH] [--metadata CSV] [--per-family N] [--output PATH] [--tsv-output PATH] [--review-seed-tsv-output PATH] [--review-seed-per-family N]"
}

fn select_fma_candidates(args: &SelectorArgs) -> Result<Vec<FmaCandidate>, String> {
    let mut reader = csv::Reader::from_path(&args.metadata)
        .map_err(|error| format!("failed to open {}: {error}", args.metadata.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read FMA headers: {error}"))?
        .clone();
    let mut selected = Vec::new();
    let mut selected_track_ids = HashSet::new();

    for row in reader.records() {
        let record = row.map_err(|error| format!("failed to read FMA row: {error}"))?;
        let Some(track_id) = field(&headers, &record, "track_id").and_then(parse_track_id) else {
            continue;
        };
        if selected_track_ids.contains(&track_id) {
            continue;
        }
        let Some(local_path) =
            fma_large_path(&args.fma_root, track_id).filter(|path| path.exists())
        else {
            continue;
        };
        let duration = field_or_empty(&headers, &record, "track_duration");
        if !duration_is_in_candidate_window(&duration) {
            continue;
        }
        let genres = field_or_empty(&headers, &record, "track_genres");
        let artist_name = field_or_empty(&headers, &record, "artist_name");
        let Some(family) = classify_family(&genres, &artist_name, &selected, args.per_family)
        else {
            continue;
        };

        selected_track_ids.insert(track_id);
        selected.push(FmaCandidate {
            family,
            track_id,
            artist_name,
            track_title: field_or_empty(&headers, &record, "track_title"),
            album_title: field_or_empty(&headers, &record, "album_title"),
            duration,
            license_title: field_or_empty(&headers, &record, "license_title"),
            license_url: field_or_empty(&headers, &record, "license_url"),
            track_url: field_or_empty(&headers, &record, "track_url"),
            genres,
            local_path,
        });

        if FmaCorpusFamily::ALL
            .iter()
            .all(|family| family_count(&selected, *family) >= args.per_family)
        {
            break;
        }
    }

    Ok(selected)
}

fn field<'a>(
    headers: &csv::StringRecord,
    record: &'a csv::StringRecord,
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .position(|header| header == name)
        .and_then(|index| record.get(index))
        .filter(|value| !value.is_empty())
}

fn field_or_empty(headers: &csv::StringRecord, record: &csv::StringRecord, name: &str) -> String {
    field(headers, record, name).unwrap_or("").to_string()
}

fn parse_track_id(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

fn fma_large_path(fma_root: &Path, track_id: u32) -> Option<PathBuf> {
    let padded = format!("{track_id:06}");
    Some(
        fma_root
            .join("fma_large")
            .join(&padded[..3])
            .join(format!("{padded}.mp3")),
    )
}

fn classify_family(
    genres: &str,
    artist_name: &str,
    selected: &[FmaCandidate],
    per_family: usize,
) -> Option<FmaCorpusFamily> {
    FmaCorpusFamily::ALL.into_iter().find(|family| {
        family_count(selected, *family) < per_family
            && !family_has_artist(selected, *family, artist_name)
            && family_matches(*family, genres)
    })
}

fn family_count(selected: &[FmaCandidate], family: FmaCorpusFamily) -> usize {
    selected
        .iter()
        .filter(|candidate| candidate.family == family)
        .count()
}

fn family_has_artist(
    selected: &[FmaCandidate],
    family: FmaCorpusFamily,
    artist_name: &str,
) -> bool {
    selected.iter().any(|candidate| {
        candidate.family == family && candidate.artist_name.eq_ignore_ascii_case(artist_name)
    })
}

fn duration_is_in_candidate_window(duration: &str) -> bool {
    let Some(seconds) = parse_duration_seconds(duration) else {
        return false;
    };
    (60..=480).contains(&seconds)
}

fn parse_duration_seconds(duration: &str) -> Option<u32> {
    let parts = duration
        .split(':')
        .map(|part| part.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    match parts.as_slice() {
        [minutes, seconds] if *seconds < 60 => Some(minutes * 60 + seconds),
        [hours, minutes, seconds] if *minutes < 60 && *seconds < 60 => {
            Some(hours * 3_600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}

fn family_matches(family: FmaCorpusFamily, genres: &str) -> bool {
    let genres = genres.to_ascii_lowercase();
    match family {
        FmaCorpusFamily::DrumsPercussion => contains_any(
            &genres,
            &[
                "hip-hop",
                "breakbeat",
                "breaks",
                "drum",
                "percussion",
                "punk",
                "dance",
            ],
        ),
        FmaCorpusFamily::Bass => contains_any(
            &genres,
            &["dub", "bass", "electronic", "techno", "house", "hip-hop"],
        ),
        FmaCorpusFamily::Vocals => contains_any(
            &genres,
            &["singer-songwriter", "folk", "pop", "hip-hop", "soul"],
        ),
        FmaCorpusFamily::PadsSustains => contains_any(
            &genres,
            &[
                "ambient",
                "drone",
                "soundtrack",
                "electroacoustic",
                "instrumental",
                "classical",
            ],
        ),
        FmaCorpusFamily::FullMix => contains_any(
            &genres,
            &["rock", "pop", "electronic", "jazz", "folk", "hip-hop"],
        ),
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn review_seed_candidates(candidates: &[FmaCandidate], per_family: usize) -> Vec<FmaCandidate> {
    let mut selected = Vec::new();
    let mut selected_track_ids = HashSet::new();
    let mut selected_artists = HashSet::new();

    for family in FmaCorpusFamily::ALL {
        for candidate in candidates {
            if candidate.family != family || family_count(&selected, family) >= per_family {
                continue;
            }
            if selected_artists.contains(&candidate.artist_name.to_ascii_lowercase()) {
                continue;
            }
            selected_track_ids.insert(candidate.track_id);
            selected_artists.insert(candidate.artist_name.to_ascii_lowercase());
            selected.push(candidate.clone());
        }
    }

    for family in FmaCorpusFamily::ALL {
        for candidate in candidates {
            if candidate.family != family || family_count(&selected, family) >= per_family {
                continue;
            }
            if selected_track_ids.contains(&candidate.track_id) {
                continue;
            }
            selected_track_ids.insert(candidate.track_id);
            selected.push(candidate.clone());
        }
    }

    selected
}

fn format_fma_selection_report(args: &SelectorArgs, candidates: &[FmaCandidate]) -> String {
    let mut lines = Vec::new();
    lines.push("# FMA Stretch Corpus Local Selection".to_string());
    lines.push(String::new());
    lines.push("Status: local-only generated evidence".to_string());
    lines.push(format!("FMA root: `{}`", args.fma_root.display()));
    lines.push(format!("Metadata: `{}`", args.metadata.display()));
    lines.push(format!("Per family target: `{}`", args.per_family));
    lines.push(String::new());
    lines.push("## Source Boundary".to_string());
    lines.push(String::new());
    lines.push("- Do not commit FMA audio files.".to_string());
    lines.push("- Each selected track keeps its FMA artist license metadata.".to_string());
    lines.push(
        "- MP3 candidates are listening/evidence material, not final golden-quality proof."
            .to_string(),
    );
    lines.push(String::new());

    for family in FmaCorpusFamily::ALL {
        lines.push(format!("## {}", family.label()));
        lines.push(String::new());
        lines.push(format!("Signal case: `{}`", family.signal_case_id()));
        lines.push(String::new());
        lines.push(
            "| Track id | Artist | Title | Album | Genres | Duration | License | Local path | URL |"
                .to_string(),
        );
        lines.push("| --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_string());
        for candidate in candidates
            .iter()
            .filter(|candidate| candidate.family == family)
        {
            lines.push(format!(
                "| `{}` | {} | {} | {} | {} | `{}` | [{}]({}) | `{}` | {} |",
                candidate.track_id,
                markdown_cell(&candidate.artist_name),
                markdown_cell(&candidate.track_title),
                markdown_cell(&candidate.album_title),
                markdown_cell(&genre_summary(&candidate.genres)),
                markdown_cell(&candidate.duration),
                markdown_cell(&candidate.license_title),
                candidate.license_url,
                candidate.local_path.display(),
                candidate.track_url
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

fn format_fma_selection_tsv(candidates: &[FmaCandidate]) -> Result<String, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(Vec::new());
    writer
        .write_record([
            "case_id",
            "family",
            "track_id",
            "artist",
            "title",
            "album",
            "genres",
            "duration",
            "license_title",
            "license_url",
            "track_url",
            "local_path",
        ])
        .map_err(|error| format!("failed to write FMA TSV header: {error}"))?;

    for candidate in candidates {
        writer
            .write_record([
                candidate.family.signal_case_id(),
                candidate.family.label(),
                &candidate.track_id.to_string(),
                &candidate.artist_name,
                &candidate.track_title,
                &candidate.album_title,
                &genre_summary(&candidate.genres),
                &candidate.duration,
                &candidate.license_title,
                &candidate.license_url,
                &candidate.track_url,
                &candidate.local_path.display().to_string(),
            ])
            .map_err(|error| format!("failed to write FMA TSV row: {error}"))?;
    }

    let bytes = writer
        .into_inner()
        .map_err(|error| format!("failed to finish FMA TSV: {error}"))?;
    String::from_utf8(bytes).map_err(|error| format!("failed to encode FMA TSV: {error}"))
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn genre_summary(raw_genres: &str) -> String {
    let mut titles = Vec::new();
    let mut rest = raw_genres;
    let marker = "'genre_title': '";
    while let Some(start) = rest.find(marker) {
        let title_start = start + marker.len();
        let Some(title_end) = rest[title_start..].find('\'') else {
            break;
        };
        titles.push(rest[title_start..title_start + title_end].to_string());
        rest = &rest[title_start + title_end..];
    }
    if titles.is_empty() {
        raw_genres.to_string()
    } else {
        titles.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn test_candidate_with(
        family: FmaCorpusFamily,
        track_id: u32,
        artist_name: &str,
    ) -> FmaCandidate {
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
}
