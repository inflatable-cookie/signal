use std::collections::HashMap;
use std::path::Path;

use super::{REQUIRED_FAMILIES, REQUIRED_RATIOS};

const FINDING_FIELDS: [&str; 6] = [
    "transient",
    "tonal",
    "stereo",
    "formant",
    "boundary",
    "preference",
];

pub(crate) fn format_blind_listening_note_status(manifest: &Path) -> Result<String, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(manifest)
        .map_err(|error| format!("failed to open {}: {error}", manifest.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to read {} headers: {error}", manifest.display()))?
        .clone();
    let mut stats = HashMap::<String, ListeningFamilyStatus>::new();
    let mut pair_count = 0usize;
    for row in reader.records() {
        let row = row.map_err(|error| format!("failed to read {}: {error}", manifest.display()))?;
        let case_id = tsv_field(&headers, &row, "case_id")?;
        if !REQUIRED_FAMILIES.contains(&case_id) {
            return Err(format!(
                "{} contains unsupported listening family {case_id}",
                manifest.display()
            ));
        }
        pair_count += 1;
        let family = stats.entry(case_id.to_string()).or_default();
        family.pair_count += 1;
        let ratio = tsv_field(&headers, &row, "ratio")?
            .parse::<f64>()
            .map_err(|error| format!("invalid blind listening ratio: {error}"))?;
        let ratio_index = REQUIRED_RATIOS
            .iter()
            .position(|required| (ratio - required).abs() < 1.0e-9)
            .ok_or_else(|| format!("unsupported blind listening ratio {ratio}"))?;
        family.ratio_mask |= 1 << ratio_index;
        if tsv_field(&headers, &row, "completed")? != "true" {
            continue;
        }
        if FINDING_FIELDS
            .iter()
            .all(|field| tsv_field(&headers, &row, field).is_ok_and(|value| !value.is_empty()))
        {
            family.completed_pair_count += 1;
        } else {
            family.invalid_completed_pair_count += 1;
        }
    }

    let completed_family_count = REQUIRED_FAMILIES
        .iter()
        .filter(|family| {
            stats
                .get(**family)
                .is_some_and(ListeningFamilyStatus::complete)
        })
        .count();
    let invalid_completed_pair_count = stats
        .values()
        .map(|family| family.invalid_completed_pair_count)
        .sum::<usize>();
    let status =
        if completed_family_count == REQUIRED_FAMILIES.len() && invalid_completed_pair_count == 0 {
            "Complete"
        } else {
            "Incomplete"
        };
    let mut lines = vec![format!(
        "blind_listening_note_status manifest={:?} status={} pairs={} completed_families={} required_families={} invalid_completed_pairs={}",
        manifest.display().to_string(),
        status,
        pair_count,
        completed_family_count,
        REQUIRED_FAMILIES.len(),
        invalid_completed_pair_count,
    )];
    for family in REQUIRED_FAMILIES {
        let family_status = stats.get(family).copied().unwrap_or_default();
        lines.push(format!(
            "blind_listening_family_status case={} status={} pairs={} completed_pairs={} invalid_completed_pairs={}",
            family,
            if family_status.complete() {
                "Complete"
            } else {
                "Incomplete"
            },
            family_status.pair_count,
            family_status.completed_pair_count,
            family_status.invalid_completed_pair_count,
        ));
    }
    Ok(lines.join("\n"))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ListeningFamilyStatus {
    pair_count: usize,
    completed_pair_count: usize,
    invalid_completed_pair_count: usize,
    ratio_mask: u8,
}

impl ListeningFamilyStatus {
    fn complete(&self) -> bool {
        self.pair_count == REQUIRED_RATIOS.len()
            && self.ratio_mask == (1 << REQUIRED_RATIOS.len()) - 1
            && self.completed_pair_count == self.pair_count
            && self.invalid_completed_pair_count == 0
    }
}

fn tsv_field<'a>(
    headers: &csv::StringRecord,
    row: &'a csv::StringRecord,
    name: &str,
) -> Result<&'a str, String> {
    headers
        .iter()
        .position(|header| header == name)
        .and_then(|index| row.get(index))
        .ok_or_else(|| format!("blind listening notes are missing field {name}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn listening_note_status_requires_classified_findings_for_every_family() {
        let path = PathBuf::from(format!(
            "target/stretch-blind-note-status-test-{}.tsv",
            std::process::id()
        ));
        let mut contents =
            "case_id\tratio\ttransient\ttonal\tstereo\tformant\tboundary\tpreference\tcompleted\n"
                .to_string();
        for family in REQUIRED_FAMILIES {
            for ratio in REQUIRED_RATIOS {
                contents.push_str(&format!(
                    "{family}\t{ratio:.6}\tnone\tnone\tnone\tnone\tnone\tA\ttrue\n"
                ));
            }
        }
        fs::write(&path, contents).expect("write completed notes");

        let status = format_blind_listening_note_status(&path).expect("inspect completed notes");

        assert!(status.contains("status=Complete"));
        assert!(status.contains("completed_families=5 required_families=5"));
        let _ = fs::remove_file(path);
    }
}
