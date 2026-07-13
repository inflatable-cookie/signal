mod audio;
pub(in crate::frequency_adaptive) mod manifest;

use std::fs;

use audio::{level_match, read_mono, write_mono};
use manifest::{
    assignment, candidate_configurations, export_root, readme, render_root, rows, source_root,
};

use super::{Configuration, Sensitivity, HASH_OFFSET};
use crate::frequency_adaptive::{
    complete_phase_synthesis::render::{render_configured, Mode},
    study_local_schedule::{
        schedule::build_schedule_with_strength,
        study::{analyze_with_geometry, select},
    },
};
use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

const SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Debug)]
pub(crate) struct ExportReview {
    pub rows: usize,
    pub candidates_per_row: usize,
    pub audio_files: usize,
    pub holdout_reads: usize,
    pub structural_failures: [usize; 5],
    pub hashes: [u64; 3],
}

pub(crate) fn export_development_pack() -> ExportReview {
    let root = export_root();
    if root.exists() {
        fs::remove_dir_all(&root).expect("replace development pack");
    }
    fs::create_dir_all(root.join("references")).expect("create references");
    fs::create_dir_all(root.join("trials")).expect("create trials");
    let configurations = candidate_configurations();
    let mut notes = String::from("row\tratio\tsource\tA\tB\tC\tD\tE\ttransient\ttonal\tgrain_ringing\tboundary\tpreference\tbroad_defect\tnotes\tcompleted\n");
    let mut key = String::from("row\tratio\tletter\tidentity\tgain\n");
    let mut structural_failures = [0; 5];
    let mut hashes = [HASH_OFFSET; 3];
    let mut audio_files = 0;
    for row in rows() {
        let mut source = read_mono(&source_root().join(row.source));
        source.truncate(source.len() / 128 * 128);
        let target_len = (source.len() as f64 * row.ratio).round() as usize;
        let mut candidates = Vec::new();
        for configuration in configurations.iter().copied() {
            candidates.push((
                configuration.stable_id(),
                successor(&source, row.ratio, configuration),
            ));
        }
        let mut current =
            OfflineHighQualityStretcher::with_path(row.ratio, OfflineHighQualityPath::Default);
        let source_f32 = source
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        let current = current
            .stretch_mono(&source_f32)
            .into_iter()
            .map(f64::from)
            .collect::<Vec<_>>();
        candidates.push(("current-signal".to_string(), current));
        let mut rubber_band = read_mono(&render_root().join(row.rubber_band));
        rubber_band.truncate(target_len);
        candidates.push(("rubber-band-r3".to_string(), rubber_band));
        structural_failures[0] += candidates
            .iter()
            .filter(|(_, samples)| samples.len() != target_len)
            .count();
        structural_failures[1] += candidates
            .iter()
            .flat_map(|(_, samples)| samples)
            .filter(|sample| !sample.is_finite())
            .count();
        let matched = level_match(&source, candidates);
        let assignment = assignment(row.id, matched.candidates.len());
        let source_name = format!("{}-source.wav", row.id);
        write_mono(
            &root.join("references").join(&source_name),
            SAMPLE_RATE,
            &matched.source,
        );
        audio_files += 1;
        let mut trial_names = Vec::new();
        for (letter_index, candidate_index) in assignment.into_iter().enumerate() {
            let letter = char::from(b'A' + letter_index as u8);
            let candidate = &matched.candidates[candidate_index];
            let name = format!("{}-{letter}.wav", row.id);
            write_mono(
                &root.join("trials").join(&name),
                SAMPLE_RATE,
                &candidate.samples,
            );
            audio_files += 1;
            trial_names.push(format!("trials/{name}"));
            key.push_str(&format!(
                "{}\t{:.6}\t{letter}\t{}\t{:.9}\n",
                row.id, row.ratio, candidate.identity, candidate.gain
            ));
            hash_bytes(&mut hashes[0], candidate.identity.as_bytes());
            hash_bytes(&mut hashes[1], &candidate.gain.to_bits().to_le_bytes());
        }
        notes.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{}\t{}\t{}\t\t\t\t\t\t\t\tfalse\n",
            row.id,
            row.ratio,
            trial_names[0],
            trial_names[1],
            trial_names[2],
            trial_names[3],
            trial_names[4]
        ));
    }
    fs::write(root.join("development-listening-notes.tsv"), notes).expect("write notes");
    fs::write(root.join("development-listening-key.tsv"), key).expect("write key");
    fs::write(root.join("README.md"), readme()).expect("write readme");
    hash_bytes(
        &mut hashes[2],
        fs::read(root.join("development-listening-notes.tsv"))
            .expect("read notes hash")
            .as_slice(),
    );
    structural_failures[2] = usize::from(audio_files != 54);
    structural_failures[3] = usize::from(configurations.len() != 3);
    structural_failures[4] = 0;
    ExportReview {
        rows: 9,
        candidates_per_row: 5,
        audio_files,
        holdout_reads: 0,
        structural_failures,
        hashes,
    }
}

fn successor(source: &[f64], ratio: f64, configuration: Configuration) -> Vec<f64> {
    let channels = vec![source.to_vec()];
    let study = analyze_with_geometry(&channels, source.len(), configuration.geometry);
    let (threshold, agreement) = match configuration.sensitivity {
        Sensitivity::Responsive => (3.0, 2),
        Sensitivity::Conservative => (6.0, 3),
    };
    let points = select(&study, threshold, agreement);
    let schedule = build_schedule_with_strength(
        source.len(),
        128,
        ratio,
        &points,
        configuration.unity_strength(),
    );
    render_configured(
        &channels,
        ratio,
        &points,
        &schedule,
        Mode::Both,
        configuration,
    )
    .samples
    .remove(0)
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *state = (*state ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}
