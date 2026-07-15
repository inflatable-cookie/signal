use std::{fs, path::PathBuf};

use crate::frequency_adaptive::complete_system_tuning::listening_export::{
    audio::{level_match, read_mono, write_mono},
    manifest::assignment,
};
use crate::frequency_adaptive::source_studied::{confirmation, hash_bytes, HASH_OFFSET};

use super::{text_hash, RenderedRow, INPUT_FRAMES, SAMPLE_RATE};

pub(super) struct PackReview {
    pub(super) audio_files: usize,
    pub(super) structural_failures: [usize; 4],
    pub(super) maximum_candidate_rms_delta_db: f64,
    pub(super) hashes: [u64; 7],
}

pub(super) fn export(rendered: Vec<RenderedRow>) -> PackReview {
    let root = root();
    confirmation::replace_directory(&root);
    for directory in [root.join("references"), root.join("trials")] {
        fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("create {}: {error}", directory.display()));
    }
    let mut manifest = String::from("row\tratio\tsource\tA\tB\toutput_frames\n");
    let mut notes = String::from(
        "row\tratio\tsource\tA\tB\tcontinuity\ttransient\tgrain_ringing\ttonal_stability\tstart_boundary\tend_boundary\tpreference\tbroad_defect\tnotes\tcompleted\n",
    );
    let mut key = String::from("row\tratio\tletter\tidentity\tgain\traw_hash\tpacked_hash\n");
    let mut receipt =
        String::from("row\trole\tpath\tsample_rate\tchannels\tframes\trms\tpeak\tfile_hash\n");
    let mut structural_failures = [0; 4];
    let mut audio_hash = HASH_OFFSET;
    let mut assignment_hash = HASH_OFFSET;
    let mut gain_hash = HASH_OFFSET;
    let mut audio_files = 0_usize;
    let mut processed_rows = 0_usize;
    let mut maximum_candidate_rms_delta_db = 0.0_f64;

    for row in rendered {
        processed_rows += 1;
        let target = (INPUT_FRAMES as f64 * row.case.ratio).round() as usize;
        let matched = level_match(
            &row.source,
            vec![
                ("coherent-signal".to_string(), row.coherent),
                ("rubber-band-r3-4.0.0".to_string(), row.rubber),
            ],
        );
        let source_name = format!("{}-source.wav", row.case.id);
        let packed_source_path = root.join("references").join(&source_name);
        write_mono(&packed_source_path, SAMPLE_RATE, &matched.source);
        append_receipt(
            &mut receipt,
            &mut audio_hash,
            row.case.id,
            "source",
            &format!("references/{source_name}"),
            &packed_source_path,
            INPUT_FRAMES,
            &mut structural_failures,
        );
        audio_files += 1;

        let letters = assignment(row.case.id, matched.candidates.len());
        let mut trial_names = Vec::with_capacity(2);
        let mut candidate_rms = Vec::with_capacity(2);
        for (letter_index, candidate_index) in letters.into_iter().enumerate() {
            let letter = char::from(b'A' + letter_index as u8);
            let candidate = &matched.candidates[candidate_index];
            let trial_name = format!("{}-{letter}.wav", row.case.id);
            let relative_path = format!("trials/{trial_name}");
            let packed_path = root.join("trials").join(&trial_name);
            write_mono(&packed_path, SAMPLE_RATE, &candidate.samples);
            let (packed_hash, packed_rms) = append_receipt(
                &mut receipt,
                &mut audio_hash,
                row.case.id,
                &letter.to_string(),
                &relative_path,
                &packed_path,
                target,
                &mut structural_failures,
            );
            audio_files += 1;
            trial_names.push(relative_path);
            candidate_rms.push(packed_rms);
            key.push_str(&format!(
                "{}\t{:.6}\t{letter}\t{}\t{:.9}\t{:016x}\t{packed_hash:016x}\n",
                row.case.id,
                row.case.ratio,
                candidate.identity,
                candidate.gain,
                row.raw_hashes[candidate_index],
            ));
            hash_bytes(&mut assignment_hash, row.case.id.as_bytes());
            hash_bytes(&mut assignment_hash, &[letter as u8]);
            hash_bytes(&mut assignment_hash, candidate.identity.as_bytes());
            hash_bytes(&mut gain_hash, &candidate.gain.to_bits().to_le_bytes());
        }
        let rms_delta_db = 20.0 * (candidate_rms[0] / candidate_rms[1]).log10().abs();
        maximum_candidate_rms_delta_db = maximum_candidate_rms_delta_db.max(rms_delta_db);
        structural_failures[3] += usize::from(rms_delta_db > 1.0e-5);
        manifest.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{target}\n",
            row.case.id, row.case.ratio, trial_names[0], trial_names[1],
        ));
        notes.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t\t\t\t\t\t\t\t\t\tfalse\n",
            row.case.id, row.case.ratio, trial_names[0], trial_names[1],
        ));
    }
    structural_failures[1] = processed_rows.abs_diff(6);
    structural_failures[2] = audio_files.abs_diff(18);

    fs::write(root.join("listening-manifest.tsv"), &manifest).expect("write manifest");
    fs::write(root.join("listening-notes.tsv"), &notes).expect("write notes");
    fs::write(root.join("listening-key.tsv"), &key).expect("write key");
    fs::write(root.join("audio-receipt.tsv"), &receipt).expect("write audio receipt");
    fs::write(
        root.join("README.md"),
        "# Exact-Source Rubber Band Comparison\n\nStatus: ready for concealed operator listening\n\nSix five-second mono source references. Each row has two peak-safe RMS-matched concealed candidates: coherent Signal and Rubber Band R3 4.0.0. Both engines consumed the same written 44.1 kHz mono 16-bit input. Judge continuity, transient definition, grain or ringing, tonal stability, and both boundaries. Keep `listening-key.tsv` closed until all six rows are complete. No holdout, stereo, dynamic-ratio, or product audio is present.\n",
    )
    .expect("write readme");

    PackReview {
        audio_files,
        structural_failures,
        maximum_candidate_rms_delta_db,
        hashes: [
            audio_hash,
            assignment_hash,
            gain_hash,
            text_hash(&manifest),
            text_hash(&key),
            text_hash(&notes),
            text_hash(&receipt),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn append_receipt(
    receipt: &mut String,
    audio_hash: &mut u64,
    row: &str,
    role: &str,
    relative_path: &str,
    path: &std::path::Path,
    expected_frames: usize,
    structural_failures: &mut [usize; 4],
) -> (u64, f64) {
    let reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let specification = reader.spec();
    let frames = reader.duration() as usize;
    structural_failures[0] += usize::from(
        frames != expected_frames
            || specification.sample_rate != SAMPLE_RATE
            || specification.channels != 1,
    );
    drop(reader);
    let samples = read_mono(path);
    structural_failures[0] += samples.iter().filter(|sample| !sample.is_finite()).count();
    let rms =
        (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt();
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    let hash = confirmation::file_hash(path);
    hash_bytes(audio_hash, &hash.to_le_bytes());
    receipt.push_str(&format!(
        "{row}\t{role}\t{relative_path}\t{}\t{}\t{frames}\t{rms:.12}\t{peak:.12}\t{hash:016x}\n",
        specification.sample_rate, specification.channels,
    ));
    (hash, rms)
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-db-concealed-pack")
}
