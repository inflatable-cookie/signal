use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::args::SelectorArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FmaCorpusFamily {
    DrumsPercussion,
    Bass,
    Vocals,
    PadsSustains,
    FullMix,
}

impl FmaCorpusFamily {
    pub const ALL: [Self; 5] = [
        Self::DrumsPercussion,
        Self::Bass,
        Self::Vocals,
        Self::PadsSustains,
        Self::FullMix,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::DrumsPercussion => "drums_percussion",
            Self::Bass => "bass",
            Self::Vocals => "vocals",
            Self::PadsSustains => "pads_sustains",
            Self::FullMix => "full_mix",
        }
    }

    pub fn signal_case_id(self) -> &'static str {
        match self {
            Self::DrumsPercussion => "stretch:drums_percussion",
            Self::Bass => "stretch:bass",
            Self::Vocals => "stretch:vocals",
            Self::PadsSustains => "stretch:pads_sustains",
            Self::FullMix => "stretch:full_mix",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FmaCandidate {
    pub family: FmaCorpusFamily,
    pub track_id: u32,
    pub artist_name: String,
    pub track_title: String,
    pub album_title: String,
    pub duration: String,
    pub license_title: String,
    pub license_url: String,
    pub track_url: String,
    pub genres: String,
    pub local_path: PathBuf,
}

pub fn select_fma_candidates(args: &SelectorArgs) -> Result<Vec<FmaCandidate>, String> {
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

pub(crate) fn fma_large_path(fma_root: &Path, track_id: u32) -> Option<PathBuf> {
    let padded = format!("{track_id:06}");
    Some(
        fma_root
            .join("fma_large")
            .join(&padded[..3])
            .join(format!("{padded}.mp3")),
    )
}

pub(crate) fn classify_family(
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

pub(crate) fn family_count(selected: &[FmaCandidate], family: FmaCorpusFamily) -> usize {
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

pub(crate) fn duration_is_in_candidate_window(duration: &str) -> bool {
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

pub fn review_seed_candidates(candidates: &[FmaCandidate], per_family: usize) -> Vec<FmaCandidate> {
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
