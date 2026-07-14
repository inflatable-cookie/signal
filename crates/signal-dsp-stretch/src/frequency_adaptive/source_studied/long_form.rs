use std::{env, fs, path::PathBuf};

use super::{
    confirmation::{
        file_hash, read_exact, replace_directory, run_command, version, write_input, SAMPLE_RATE,
    },
    execute, hash_bytes, render_current, Architecture, HASH_OFFSET,
};
use crate::frequency_adaptive::complete_system_tuning::listening_export::{
    audio::{level_match, read_mono, write_mono},
    manifest::assignment,
};

const INPUT_FRAMES: usize = 220_500;

pub(in crate::frequency_adaptive) struct LongCase {
    pub(in crate::frequency_adaptive) id: &'static str,
    pub(in crate::frequency_adaptive) source: &'static str,
    pub(in crate::frequency_adaptive) ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LongFormReview {
    pub rows: usize,
    pub candidates_per_row: usize,
    pub input_files: usize,
    pub external_files: usize,
    pub audio_files: usize,
    pub holdout_reads: usize,
    pub structural_failures: [usize; 4],
    pub hashes: [u64; 5],
    pub rubber_band_version: String,
}

pub(in crate::frequency_adaptive) fn run() -> LongFormReview {
    let external_root = external_root();
    let pack_root = pack_root();
    replace_directory(&external_root);
    replace_directory(&pack_root);
    for directory in [
        external_root.join("inputs"),
        external_root.join("rubber-band-r3"),
        pack_root.join("references"),
        pack_root.join("trials"),
    ] {
        fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("create {}: {error}", directory.display()));
    }

    let rubber_band = env::var_os("RUBBERBAND_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("rubberband"));
    let rubber_band_version = version(&rubber_band, &["--version"]);
    assert_eq!(rubber_band_version, "4.0.0");

    let cases = cases();
    let mut input_manifest =
        String::from("row\tratio\tpath\tsample_rate\tchannels\tframes\tfile_hash\n");
    let mut external_receipt =
        String::from("row\tratio\tengine\tversion\tinput_hash\toutput_hash\toutput_frames\n");
    let mut rendered = Vec::with_capacity(cases.len());
    let mut input_hash = HASH_OFFSET;
    let mut external_hash = HASH_OFFSET;
    let mut input_files = 0;
    let mut external_files = 0;
    for case in &cases {
        let original = read_mono(&source_root().join(case.source));
        assert_eq!(original.len(), INPUT_FRAMES, "{}", case.source);
        let input_path = external_root
            .join("inputs")
            .join(format!("{}.wav", case.id));
        write_input(&input_path, &original);
        input_files += 1;
        let source = read_exact(&input_path, INPUT_FRAMES, 16);
        let row_input_hash = file_hash(&input_path);
        input_manifest.push_str(&format!(
            "{}\t{:.6}\tinputs/{}.wav\t{}\t1\t{}\t{:016x}\n",
            case.id, case.ratio, case.id, SAMPLE_RATE, INPUT_FRAMES, row_input_hash
        ));
        hash_bytes(&mut input_hash, &row_input_hash.to_le_bytes());

        let target = (INPUT_FRAMES as f64 * case.ratio).round() as usize;
        let rubber_path = external_root
            .join("rubber-band-r3")
            .join(format!("{}.wav", case.id));
        run_command(
            &rubber_band,
            &[
                "-q".into(),
                "-3".into(),
                "-t".into(),
                format!("{:.6}", case.ratio).into(),
                input_path.as_os_str().into(),
                rubber_path.as_os_str().into(),
            ],
        );
        let rubber = read_exact(&rubber_path, target, 0);
        external_files += 1;
        let rubber_hash = file_hash(&rubber_path);
        external_receipt.push_str(&format!(
            "{}\t{:.6}\trubber-band-r3\t{}\t{:016x}\t{:016x}\t{}\n",
            case.id, case.ratio, rubber_band_version, row_input_hash, rubber_hash, target
        ));
        hash_bytes(&mut external_hash, &rubber_hash.to_le_bytes());
        rendered.push((source, rubber));
    }
    fs::write(external_root.join("input-manifest.tsv"), &input_manifest)
        .expect("write long-form input manifest");
    fs::write(
        external_root.join("external-render-receipt.tsv"),
        &external_receipt,
    )
    .expect("write long-form external receipt");
    fs::write(
        external_root.join("README.md"),
        "# Long-Form External Stretch Renders\n\nSix row-specific 44.1 kHz, mono, 16-bit, five-second inputs. Rubber Band R3 4.0.0 outputs were invoked from these exact files by the Signal long-form runner. Hashes and frame counts are frozen in the TSV receipts.\n",
    )
    .expect("write long-form external readme");

    let mut notes = String::from(
        "row\tratio\tsource\tA\tB\tC\ttransient\ttonal\tgrain_ringing\tcontinuity\tboundary\tpreference\tbroad_defect\tnotes\tcompleted\n",
    );
    let mut key = String::from("row\tratio\tletter\tidentity\tgain\n");
    let mut assignment_hash = HASH_OFFSET;
    let mut gain_hash = HASH_OFFSET;
    let mut structural_failures = [0; 4];
    let mut audio_files = 0;
    let mut processed_rows = 0_usize;
    for (case, (source, rubber)) in cases.iter().zip(rendered) {
        processed_rows += 1;
        let target = (INPUT_FRAMES as f64 * case.ratio).round() as usize;
        let weighted = execute(
            std::slice::from_ref(&source),
            case.ratio,
            Architecture::WeightedPredictor,
        )
        .samples
        .remove(0);
        let current = render_current(&source, case.ratio);
        let candidates = vec![
            ("signal-weighted-predictor".to_string(), weighted),
            ("current-signal".to_string(), current),
            ("rubber-band-r3".to_string(), rubber),
        ];
        structural_failures[0] += candidates
            .iter()
            .filter(|(_, samples)| samples.len() != target)
            .count();
        structural_failures[1] += candidates
            .iter()
            .flat_map(|(_, samples)| samples)
            .filter(|sample| !sample.is_finite())
            .count();
        let matched = level_match(&source, candidates);
        let source_name = format!("{}-source.wav", case.id);
        write_mono(
            &pack_root.join("references").join(&source_name),
            SAMPLE_RATE,
            &matched.source,
        );
        audio_files += 1;
        let letters = assignment(case.id, matched.candidates.len());
        let mut trial_names = Vec::with_capacity(letters.len());
        for (letter_index, candidate_index) in letters.into_iter().enumerate() {
            let letter = char::from(b'A' + letter_index as u8);
            let candidate = &matched.candidates[candidate_index];
            let trial_name = format!("{}-{letter}.wav", case.id);
            write_mono(
                &pack_root.join("trials").join(&trial_name),
                SAMPLE_RATE,
                &candidate.samples,
            );
            audio_files += 1;
            trial_names.push(format!("trials/{trial_name}"));
            key.push_str(&format!(
                "{}\t{:.6}\t{letter}\t{}\t{:.9}\n",
                case.id, case.ratio, candidate.identity, candidate.gain
            ));
            hash_bytes(&mut assignment_hash, candidate.identity.as_bytes());
            hash_bytes(&mut gain_hash, &candidate.gain.to_bits().to_le_bytes());
        }
        notes.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{}\t\t\t\t\t\t\t\t\tfalse\n",
            case.id, case.ratio, trial_names[0], trial_names[1], trial_names[2]
        ));
    }
    structural_failures[2] = processed_rows.abs_diff(cases.len());
    structural_failures[3] = usize::from(audio_files != 24);
    fs::write(pack_root.join("development-listening-notes.tsv"), &notes)
        .expect("write long-form notes");
    fs::write(pack_root.join("development-listening-key.tsv"), &key).expect("write long-form key");
    fs::write(
        pack_root.join("comparator-receipt.tsv"),
        format!("{input_manifest}\n{external_receipt}"),
    )
    .expect("write long-form comparator receipt");
    fs::write(
        pack_root.join("README.md"),
        "# Long-Form Stretch Listening Pack\n\nStatus: ready for concealed operator listening\n\nSix five-second mono rows stretched to 1.5x or 2.0x. Each row has source plus candidates A-C: Signal weighted predictor, current Signal, and Rubber Band R3. Every engine consumed the same 44.1 kHz mono 16-bit input. Judge musical continuity, sustained grain/ringing, tonal stability, transient integrity, and boundaries. Keep the key and comparator receipt closed until all rows are complete. No holdout audio is present.\n",
    )
    .expect("write long-form readme");
    let mut notes_hash = HASH_OFFSET;
    hash_bytes(&mut notes_hash, notes.as_bytes());

    LongFormReview {
        rows: cases.len(),
        candidates_per_row: 3,
        input_files,
        external_files,
        audio_files,
        holdout_reads: 0,
        structural_failures,
        hashes: [
            input_hash,
            external_hash,
            assignment_hash,
            gain_hash,
            notes_hash,
        ],
        rubber_band_version,
    }
}

pub(in crate::frequency_adaptive) fn cases() -> [LongCase; 6] {
    [
        LongCase {
            id: "M001",
            source: "0000-drums_percussion-000002.wav",
            ratio: 1.5,
        },
        LongCase {
            id: "M002",
            source: "0004-bass-000236.wav",
            ratio: 1.5,
        },
        LongCase {
            id: "M003",
            source: "0008-vocals-000010.wav",
            ratio: 2.0,
        },
        LongCase {
            id: "M004",
            source: "0012-pads_sustains-000423.wav",
            ratio: 2.0,
        },
        LongCase {
            id: "M005",
            source: "0016-full_mix-000144.wav",
            ratio: 1.5,
        },
        LongCase {
            id: "M006",
            source: "0016-full_mix-000144.wav",
            ratio: 2.0,
        },
    ]
}

pub(in crate::frequency_adaptive) fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-corpus-external-benchmark-pack-fma-broad/sources")
}

fn external_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-ck-external")
}

fn pack_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-ck-long-form-pack")
}
