use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::{development_cases, execute, hash_bytes, render_current, Architecture, HASH_OFFSET};
use crate::frequency_adaptive::complete_system_tuning::listening_export::{
    audio::{level_match, read_mono, write_mono},
    manifest::{assignment, rows},
};

pub(super) const SAMPLE_RATE: u32 = 44_100;
const INPUT_FRAMES: usize = 16_384;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ConfirmationReview {
    pub rows: usize,
    pub candidates_per_row: usize,
    pub input_files: usize,
    pub external_files: usize,
    pub audio_files: usize,
    pub holdout_reads: usize,
    pub structural_failures: [usize; 4],
    pub hashes: [u64; 5],
    pub rubber_band_version: String,
    pub signalsmith_version: String,
}

pub(in crate::frequency_adaptive) fn run() -> ConfirmationReview {
    let external_root = external_root();
    let pack_root = pack_root();
    replace_directory(&external_root);
    replace_directory(&pack_root);
    for directory in [
        external_root.join("inputs"),
        external_root.join("rubber-band-r3"),
        external_root.join("signalsmith-stretch-1.3.2"),
        pack_root.join("references"),
        pack_root.join("trials"),
    ] {
        fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("create {}: {error}", directory.display()));
    }

    let rubber_band = env::var_os("RUBBERBAND_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("rubberband"));
    let signalsmith = env::var_os("SIGNALSMITH_STRETCH_BIN")
        .map(PathBuf::from)
        .expect("set SIGNALSMITH_STRETCH_BIN to the pinned Signalsmith Stretch 1.3.2 CLI");
    let rubber_band_version = version(&rubber_band, &["--version"]);
    let signalsmith_version = version(&signalsmith, &["-v"]);
    assert_eq!(rubber_band_version, "4.0.0");
    assert_eq!(signalsmith_version, "1.3.2");

    let mut input_manifest =
        String::from("row\tratio\tpath\tsample_rate\tchannels\tframes\tfile_hash\n");
    let mut external_receipt =
        String::from("row\tratio\tengine\tversion\tinput_hash\toutput_hash\toutput_frames\n");
    let mut inputs = Vec::with_capacity(9);
    let mut input_files = 0;
    let mut external_files = 0;
    let mut input_hash = HASH_OFFSET;
    let mut external_hash = HASH_OFFSET;
    let manifest_rows = rows();
    for (row, case) in manifest_rows.iter().zip(development_cases()) {
        let source = &case.channels[0];
        assert_eq!(source.len(), INPUT_FRAMES);
        assert_eq!(case.ratio, row.ratio);
        let input_path = external_root.join("inputs").join(format!("{}.wav", row.id));
        write_input(&input_path, source);
        input_files += 1;
        let source = read_exact(&input_path, INPUT_FRAMES, 16);
        let row_input_hash = file_hash(&input_path);
        input_manifest.push_str(&format!(
            "{}\t{:.6}\tinputs/{}.wav\t{}\t1\t{}\t{:016x}\n",
            row.id, row.ratio, row.id, SAMPLE_RATE, INPUT_FRAMES, row_input_hash
        ));
        hash_bytes(&mut input_hash, &row_input_hash.to_le_bytes());

        let target = (INPUT_FRAMES as f64 * row.ratio).round() as usize;
        let rubber_path = external_root
            .join("rubber-band-r3")
            .join(format!("{}.wav", row.id));
        run_command(
            &rubber_band,
            &[
                "-q".into(),
                "-3".into(),
                "-t".into(),
                format!("{:.6}", row.ratio).into(),
                input_path.as_os_str().into(),
                rubber_path.as_os_str().into(),
            ],
        );
        let rubber = read_exact(&rubber_path, target, 0);
        external_files += 1;
        let rubber_hash = file_hash(&rubber_path);
        external_receipt.push_str(&receipt_row(
            row.id,
            row.ratio,
            "rubber-band-r3",
            &rubber_band_version,
            row_input_hash,
            rubber_hash,
            target,
        ));
        hash_bytes(&mut external_hash, &rubber_hash.to_le_bytes());

        let signalsmith_path = external_root
            .join("signalsmith-stretch-1.3.2")
            .join(format!("{}.wav", row.id));
        run_command(
            &signalsmith,
            &[
                input_path.as_os_str().into(),
                signalsmith_path.as_os_str().into(),
                format!("--time={:.6}", row.ratio).into(),
            ],
        );
        let signalsmith_samples = read_exact(&signalsmith_path, target, 0);
        external_files += 1;
        let signalsmith_hash = file_hash(&signalsmith_path);
        external_receipt.push_str(&receipt_row(
            row.id,
            row.ratio,
            "signalsmith-stretch-1.3.2",
            &signalsmith_version,
            row_input_hash,
            signalsmith_hash,
            target,
        ));
        hash_bytes(&mut external_hash, &signalsmith_hash.to_le_bytes());
        inputs.push((source, rubber, signalsmith_samples));
    }
    fs::write(external_root.join("input-manifest.tsv"), &input_manifest)
        .expect("write exact-input manifest");
    fs::write(
        external_root.join("external-render-receipt.tsv"),
        &external_receipt,
    )
    .expect("write external render receipt");
    fs::write(
        external_root.join("README.md"),
        "# Exact-Excerpt External Stretch Renders\n\nNine row-specific 44.1 kHz, mono, 16-bit, 16384-frame inputs. Rubber Band R3 4.0.0 and Signalsmith Stretch 1.3.2 outputs were invoked from these files by the Signal confirmation runner. `input-manifest.tsv` and `external-render-receipt.tsv` freeze file hashes and frame counts.\n",
    )
    .expect("write external render readme");

    let mut notes = String::from(
        "row\tratio\tsource\tA\tB\tC\tD\ttransient\ttonal\tgrain_ringing\tboundary\tpreference\tbroad_defect\tnotes\tcompleted\n",
    );
    let mut key = String::from("row\tratio\tletter\tidentity\tgain\n");
    let mut assignment_hash = HASH_OFFSET;
    let mut gain_hash = HASH_OFFSET;
    let mut structural_failures = [0; 4];
    let mut audio_files = 0;
    let mut processed_rows = 0_usize;
    for (row, (source, rubber, signalsmith)) in manifest_rows.iter().zip(inputs) {
        processed_rows += 1;
        let target = (INPUT_FRAMES as f64 * row.ratio).round() as usize;
        let weighted = execute(
            std::slice::from_ref(&source),
            row.ratio,
            Architecture::WeightedPredictor,
        )
        .samples
        .remove(0);
        let current = render_current(&source, row.ratio);
        let candidates = vec![
            ("signal-weighted-predictor".to_string(), weighted),
            ("current-signal".to_string(), current),
            ("rubber-band-r3".to_string(), rubber),
            ("signalsmith-stretch-1.3.2".to_string(), signalsmith),
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
        let source_name = format!("{}-source.wav", row.id);
        write_mono(
            &pack_root.join("references").join(&source_name),
            SAMPLE_RATE,
            &matched.source,
        );
        audio_files += 1;
        let letters = assignment(row.id, matched.candidates.len());
        let mut trial_names = Vec::with_capacity(letters.len());
        for (letter_index, candidate_index) in letters.into_iter().enumerate() {
            let letter = char::from(b'A' + letter_index as u8);
            let candidate = &matched.candidates[candidate_index];
            let trial_name = format!("{}-{letter}.wav", row.id);
            write_mono(
                &pack_root.join("trials").join(&trial_name),
                SAMPLE_RATE,
                &candidate.samples,
            );
            audio_files += 1;
            trial_names.push(format!("trials/{trial_name}"));
            key.push_str(&format!(
                "{}\t{:.6}\t{letter}\t{}\t{:.9}\n",
                row.id, row.ratio, candidate.identity, candidate.gain
            ));
            hash_bytes(&mut assignment_hash, candidate.identity.as_bytes());
            hash_bytes(&mut gain_hash, &candidate.gain.to_bits().to_le_bytes());
        }
        notes.push_str(&format!(
            "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{}\t{}\t\t\t\t\t\t\t\tfalse\n",
            row.id, row.ratio, trial_names[0], trial_names[1], trial_names[2], trial_names[3],
        ));
    }
    structural_failures[2] = processed_rows.abs_diff(9);
    structural_failures[3] = usize::from(audio_files != 45);
    fs::write(pack_root.join("development-listening-notes.tsv"), &notes)
        .expect("write confirmation notes");
    fs::write(pack_root.join("development-listening-key.tsv"), &key)
        .expect("write confirmation key");
    fs::write(
        pack_root.join("comparator-receipt.tsv"),
        format!("{input_manifest}\n{external_receipt}"),
    )
    .expect("write confirmation receipt");
    fs::write(
        pack_root.join("README.md"),
        "# Exact-Excerpt Stretch Confirmation Pack\n\nStatus: ready for concealed operator listening\n\nNine mono rows. Each row has source plus candidates A-D: Signal weighted predictor, current Signal, Rubber Band R3, and Signalsmith Stretch 1.3.2. Every engine consumed the same row-specific 44.1 kHz, mono, 16-bit, 16384-frame input. Compare transient integrity, tonal stability, grain/ringing, and boundaries. Keep `development-listening-key.tsv` and `comparator-receipt.tsv` closed until every row is complete. No holdout audio is present.\n",
    )
    .expect("write confirmation readme");
    let mut notes_hash = HASH_OFFSET;
    hash_bytes(&mut notes_hash, notes.as_bytes());

    ConfirmationReview {
        rows: 9,
        candidates_per_row: 4,
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
        signalsmith_version,
    }
}

fn external_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-cj-external")
}

fn pack_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-cj-development-pack")
}

pub(super) fn replace_directory(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("replace {}: {error}", path.display()));
    }
    fs::create_dir_all(path).unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
}

pub(super) fn write_input(path: &Path, samples: &[f64]) {
    let specification = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, specification)
        .unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
    for sample in samples {
        let quantized =
            (sample.clamp(-1.0, f64::from(i16::MAX) / 32_768.0) * 32_768.0).round() as i16;
        writer.write_sample(quantized).expect("write input sample");
    }
    writer.finalize().expect("finalize exact input");
}

pub(super) fn read_exact(path: &Path, expected_frames: usize, expected_bits: u16) -> Vec<f64> {
    let reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let specification = reader.spec();
    assert_eq!(specification.sample_rate, SAMPLE_RATE, "{}", path.display());
    assert_eq!(specification.channels, 1, "{}", path.display());
    if expected_bits != 0 {
        assert_eq!(
            specification.bits_per_sample,
            expected_bits,
            "{}",
            path.display()
        );
        assert_eq!(
            specification.sample_format,
            hound::SampleFormat::Int,
            "{}",
            path.display()
        );
    }
    assert_eq!(
        reader.duration() as usize,
        expected_frames,
        "{}",
        path.display()
    );
    drop(reader);
    let samples = read_mono(path);
    assert_eq!(samples.len(), expected_frames, "{}", path.display());
    assert!(samples.iter().all(|sample| sample.is_finite()));
    samples
}

pub(super) fn version(program: &Path, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", program.display()));
    assert!(
        output.status.success(),
        "{} version failed: {}",
        program.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout
        .lines()
        .chain(stderr.lines())
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

pub(super) fn run_command(program: &Path, arguments: &[std::ffi::OsString]) {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", program.display()));
    assert!(
        output.status.success(),
        "{} failed: {}",
        program.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn receipt_row(
    row: &str,
    ratio: f64,
    engine: &str,
    version: &str,
    input_hash: u64,
    output_hash: u64,
    output_frames: usize,
) -> String {
    format!(
        "{row}\t{ratio:.6}\t{engine}\t{version}\t{input_hash:016x}\t{output_hash:016x}\t{output_frames}\n"
    )
}

pub(super) fn file_hash(path: &Path) -> u64 {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut hash = HASH_OFFSET;
    hash_bytes(&mut hash, &bytes);
    hash
}
