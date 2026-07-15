use std::{fs, path::PathBuf};

use super::real_source_confirmation::{
    self, RealSourceConfirmationDirection, RealSourceConfirmationReview,
};
use crate::frequency_adaptive::complete_system_tuning::listening_export::{
    audio::{level_match, read_mono, write_mono},
    manifest::assignment,
};

use super::super::{confirmation, hash_bytes, long_form, HASH_OFFSET};

const SAMPLE_RATE: u32 = 44_100;
const INPUT_FRAMES: usize = 220_500;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ConcealedComparisonReview {
    pub(in crate::frequency_adaptive) rows: usize,
    pub(in crate::frequency_adaptive) candidates_per_row: usize,
    pub(in crate::frequency_adaptive) audio_files: usize,
    pub(in crate::frequency_adaptive) holdout_reads: usize,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 7],
    pub(in crate::frequency_adaptive) maximum_candidate_rms_delta_db: f64,
    pub(in crate::frequency_adaptive) hashes: [u64; 7],
    pub(in crate::frequency_adaptive) confirmation: RealSourceConfirmationReview,
}

pub(in crate::frequency_adaptive) fn export() -> ConcealedComparisonReview {
    let confirmation = real_source_confirmation::review();
    let confirmation_root = real_source_confirmation::output_root();
    let root = output_root();
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
    let mut structural_failures = [
        usize::from(
            confirmation.direction != RealSourceConfirmationDirection::ConcealedMusicalComparison,
        ),
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut audio_hash = HASH_OFFSET;
    let mut assignment_hash = HASH_OFFSET;
    let mut gain_hash = HASH_OFFSET;
    let mut audio_files = 0_usize;
    let mut processed_rows = 0_usize;
    let mut maximum_candidate_rms_delta_db = 0.0_f64;

    for case in long_form::cases() {
        processed_rows += 1;
        let source_path = confirmation_root
            .join("inputs")
            .join(format!("{}.wav", case.id));
        let coherent_path = confirmation_root
            .join("coherent-signal")
            .join(format!("{}.wav", case.id));
        let pinned_path = confirmation_root
            .join("signalsmith")
            .join(format!("{}.wav", case.id));
        let source = confirmation::read_exact(&source_path, INPUT_FRAMES, 16);
        let output_frames = (INPUT_FRAMES as f64 * case.ratio).round() as usize;
        let coherent = confirmation::read_exact(&coherent_path, output_frames, 0);
        let pinned = confirmation::read_exact(&pinned_path, output_frames, 0);
        structural_failures[1] += usize::from(source.len() != INPUT_FRAMES);
        structural_failures[1] += usize::from(coherent.len() != output_frames);
        structural_failures[1] += usize::from(pinned.len() != output_frames);
        structural_failures[2] += source
            .iter()
            .chain(&coherent)
            .chain(&pinned)
            .filter(|sample| !sample.is_finite())
            .count();

        let raw_hashes = [
            confirmation::file_hash(&coherent_path),
            confirmation::file_hash(&pinned_path),
        ];
        let matched = level_match(
            &source,
            vec![
                ("coherent-signal".to_string(), coherent),
                ("signalsmith-stretch-1.3.2-seed-0".to_string(), pinned),
            ],
        );
        let source_name = format!("{}-source.wav", case.id);
        let packed_source_path = root.join("references").join(&source_name);
        write_mono(&packed_source_path, SAMPLE_RATE, &matched.source);
        append_receipt(
            &mut receipt,
            &mut audio_hash,
            case.id,
            "source",
            &format!("references/{source_name}"),
            &packed_source_path,
            INPUT_FRAMES,
            &mut structural_failures,
        );
        audio_files += 1;

        let letters = assignment(case.id, matched.candidates.len());
        let mut trial_names = Vec::with_capacity(letters.len());
        let mut candidate_rms = Vec::with_capacity(letters.len());
        for (letter_index, candidate_index) in letters.into_iter().enumerate() {
            let letter = char::from(b'A' + letter_index as u8);
            let candidate = &matched.candidates[candidate_index];
            let trial_name = format!("{}-{letter}.wav", case.id);
            let relative_path = format!("trials/{trial_name}");
            let packed_path = root.join("trials").join(&trial_name);
            write_mono(&packed_path, SAMPLE_RATE, &candidate.samples);
            let (packed_hash, packed_rms, _) = append_receipt(
                &mut receipt,
                &mut audio_hash,
                case.id,
                &letter.to_string(),
                &relative_path,
                &packed_path,
                output_frames,
                &mut structural_failures,
            );
            audio_files += 1;
            trial_names.push(relative_path);
            candidate_rms.push(packed_rms);
            key.push_str(&format!(
                "{}\t{:.6}\t{letter}\t{}\t{:.9}\t{:016x}\t{packed_hash:016x}\n",
                case.id,
                case.ratio,
                candidate.identity,
                candidate.gain,
                raw_hashes[candidate_index],
            ));
            hash_bytes(&mut assignment_hash, case.id.as_bytes());
            hash_bytes(&mut assignment_hash, &[letter as u8]);
            hash_bytes(&mut assignment_hash, candidate.identity.as_bytes());
            hash_bytes(&mut gain_hash, &candidate.gain.to_bits().to_le_bytes());
        }
        let rms_delta_db = 20.0 * (candidate_rms[0] / candidate_rms[1]).log10().abs();
        maximum_candidate_rms_delta_db = maximum_candidate_rms_delta_db.max(rms_delta_db);
        structural_failures[6] += usize::from(rms_delta_db > 1.0e-5);
        manifest.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{output_frames}\n",
            case.id, case.ratio, trial_names[0], trial_names[1]
        ));
        notes.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t\t\t\t\t\t\t\t\t\tfalse\n",
            case.id, case.ratio, trial_names[0], trial_names[1]
        ));
    }
    structural_failures[4] = processed_rows.abs_diff(6);
    structural_failures[5] = audio_files.abs_diff(18);

    fs::write(root.join("listening-manifest.tsv"), &manifest).expect("write listening manifest");
    fs::write(root.join("listening-notes.tsv"), &notes).expect("write listening notes");
    fs::write(root.join("listening-key.tsv"), &key).expect("write listening key");
    fs::write(root.join("audio-receipt.tsv"), &receipt).expect("write audio receipt");
    fs::write(
        root.join("README.md"),
        "# Coherent Source Concealed Comparison\n\nStatus: ready for concealed operator listening\n\nSix five-second mono source references. Each row has two level-matched concealed candidates: coherent Signal and pinned Signalsmith Stretch 1.3.2 with seed 0. Judge musical continuity, transient definition, grain or ringing, tonal stability, and start/end artifacts. Keep `listening-key.tsv` closed until all six `listening-notes.tsv` rows are complete. No holdout, stereo, dynamic-ratio, or product audio is present.\n",
    )
    .expect("write listening readme");

    ConcealedComparisonReview {
        rows: processed_rows,
        candidates_per_row: 2,
        audio_files,
        holdout_reads: 0,
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
        confirmation,
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
    structural_failures: &mut [usize; 7],
) -> (u64, f64, f64) {
    let reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let specification = reader.spec();
    let frames = reader.duration() as usize;
    structural_failures[1] += usize::from(frames != expected_frames);
    structural_failures[3] +=
        usize::from(specification.sample_rate != SAMPLE_RATE || specification.channels != 1);
    drop(reader);
    let samples = read_mono(path);
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
    (hash, rms, peak)
}

fn text_hash(text: &str) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_bytes(&mut hash, text.as_bytes());
    hash
}

pub(in crate::frequency_adaptive) fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-da-concealed-pack")
}
