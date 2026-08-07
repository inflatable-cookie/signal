use super::args::SelectorArgs;
use super::select::{FmaCandidate, FmaCorpusFamily};

pub fn format_fma_selection_report(args: &SelectorArgs, candidates: &[FmaCandidate]) -> String {
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

pub fn format_fma_selection_tsv(candidates: &[FmaCandidate]) -> Result<String, String> {
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

pub(crate) fn genre_summary(raw_genres: &str) -> String {
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
