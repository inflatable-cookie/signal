use std::{env, ffi::OsString, fs, path::PathBuf};

use super::coherent_representation;
use crate::frequency_adaptive::{
    adaptive_single_frame_synthesis::development_measurement,
    complete_system_tuning::listening_export::audio::{read_mono, write_mono},
};

use super::super::{confirmation, hash_bytes, long_form, HASH_OFFSET};

const SAMPLE_RATE: u32 = 44_100;
const INPUT_FRAMES: usize = 220_500;

mod pack;

struct RenderedRow {
    case: long_form::LongCase,
    source: Vec<f64>,
    coherent: Vec<f64>,
    rubber: Vec<f64>,
    raw_hashes: [u64; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum RubberBandComparisonDirection {
    ConcealedListening,
    ObjectiveFailure,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RubberBandComparisonRowEvidence {
    pub(in crate::frequency_adaptive) id: &'static str,
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) coherent_mean_event_offset: f64,
    pub(in crate::frequency_adaptive) rubber_mean_event_offset: f64,
    pub(in crate::frequency_adaptive) coherent_replica_ratio: f64,
    pub(in crate::frequency_adaptive) rubber_replica_ratio: f64,
    pub(in crate::frequency_adaptive) coherent_static_residual: f64,
    pub(in crate::frequency_adaptive) rubber_static_residual: f64,
    pub(in crate::frequency_adaptive) coherent_boundary_growth_db: f64,
    pub(in crate::frequency_adaptive) rubber_boundary_growth_db: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RubberBandComparisonReview {
    pub(in crate::frequency_adaptive) rows: Vec<RubberBandComparisonRowEvidence>,
    pub(in crate::frequency_adaptive) candidates_per_row: usize,
    pub(in crate::frequency_adaptive) audio_files: usize,
    pub(in crate::frequency_adaptive) holdout_reads: usize,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 9],
    pub(in crate::frequency_adaptive) coherent_hard_failures: usize,
    pub(in crate::frequency_adaptive) rubber_hard_failures: usize,
    pub(in crate::frequency_adaptive) coherent_regression_rows: [usize; 4],
    pub(in crate::frequency_adaptive) maximum_candidate_rms_delta_db: f64,
    pub(in crate::frequency_adaptive) hashes: [u64; 13],
    pub(in crate::frequency_adaptive) rubber_band_version: String,
    pub(in crate::frequency_adaptive) coherent_repeated: bool,
    pub(in crate::frequency_adaptive) rubber_repeated: bool,
    pub(in crate::frequency_adaptive) direction: RubberBandComparisonDirection,
}

pub(in crate::frequency_adaptive) fn export() -> RubberBandComparisonReview {
    let evidence_root = evidence_root();
    confirmation::replace_directory(&evidence_root);
    for directory in [
        evidence_root.join("inputs"),
        evidence_root.join("coherent-signal"),
        evidence_root.join("rubber-band-r3"),
        evidence_root.join("rubber-band-r3-repeat"),
    ] {
        fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("create {}: {error}", directory.display()));
    }

    let rubber_band = env::var_os("RUBBERBAND_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("rubberband"));
    let rubber_band_version = confirmation::version(&rubber_band, &["--version"]);
    assert_eq!(rubber_band_version, "4.0.0");

    let mut input_manifest =
        String::from("row\tratio\tpath\tsample_rate\tchannels\tframes\tfile_hash\n");
    let mut render_receipt =
        String::from("row\tratio\tengine\tversion\tinput_hash\toutput_hash\toutput_frames\n");
    let mut structural_failures = [0; 9];
    let mut input_hash = HASH_OFFSET;
    let mut coherent_hash = HASH_OFFSET;
    let mut rubber_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut measurements = Vec::with_capacity(12);
    let mut rows = Vec::with_capacity(6);
    let mut rendered = Vec::with_capacity(6);

    for case in long_form::cases() {
        let original = read_mono(&long_form::source_root().join(case.source));
        structural_failures[0] += usize::from(original.len() != INPUT_FRAMES);
        structural_failures[0] += original.iter().filter(|sample| !sample.is_finite()).count();
        let input_path = evidence_root
            .join("inputs")
            .join(format!("{}.wav", case.id));
        confirmation::write_input(&input_path, &original);
        let source = confirmation::read_exact(&input_path, INPUT_FRAMES, 16);
        let row_input_hash = confirmation::file_hash(&input_path);
        input_manifest.push_str(&format!(
            "{}\t{:.6}\tinputs/{}.wav\t{}\t1\t{}\t{row_input_hash:016x}\n",
            case.id, case.ratio, case.id, SAMPLE_RATE, INPUT_FRAMES,
        ));
        mix(&mut input_hash, row_input_hash);

        let target = (INPUT_FRAMES as f64 * case.ratio).round() as usize;
        let coherent = coherent_representation::render(&source, case.ratio, SAMPLE_RATE as usize);
        let coherent_repeat =
            coherent_representation::render(&source, case.ratio, SAMPLE_RATE as usize);
        structural_failures[1] += usize::from(coherent.samples.len() != target)
            + coherent.non_finite
            + coherent.uncovered
            + coherent.boundary_failures;
        structural_failures[2] += usize::from(coherent.hash != coherent_repeat.hash);
        let coherent_path = evidence_root
            .join("coherent-signal")
            .join(format!("{}.wav", case.id));
        write_mono(&coherent_path, SAMPLE_RATE, &coherent.samples);
        let coherent_file_hash = confirmation::file_hash(&coherent_path);
        render_receipt.push_str(&render_receipt_row(
            case.id,
            case.ratio,
            "coherent-signal",
            "signal-source-studied",
            row_input_hash,
            coherent_file_hash,
            target,
        ));

        let rubber_path = evidence_root
            .join("rubber-band-r3")
            .join(format!("{}.wav", case.id));
        render_rubber_band(&rubber_band, case.ratio, &input_path, &rubber_path);
        let rubber = confirmation::read_exact(&rubber_path, target, 0);
        structural_failures[3] += rubber.iter().filter(|sample| !sample.is_finite()).count();
        let rubber_file_hash = confirmation::file_hash(&rubber_path);
        render_receipt.push_str(&render_receipt_row(
            case.id,
            case.ratio,
            "rubber-band-r3",
            &rubber_band_version,
            row_input_hash,
            rubber_file_hash,
            target,
        ));

        let rubber_repeat_path = evidence_root
            .join("rubber-band-r3-repeat")
            .join(format!("{}.wav", case.id));
        render_rubber_band(&rubber_band, case.ratio, &input_path, &rubber_repeat_path);
        let rubber_repeat = confirmation::read_exact(&rubber_repeat_path, target, 0);
        structural_failures[4] += usize::from(rubber != rubber_repeat);

        let source_f32 = source
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        let coherent_f32 = coherent
            .samples
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        let rubber_f32 = rubber
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        let coherent_measurement = development_measurement::measure(
            case.id,
            case.ratio,
            "coherent-signal",
            &source_f32,
            &coherent_f32,
        );
        let rubber_measurement = development_measurement::measure(
            case.id,
            case.ratio,
            "rubber-band-r3-4.0.0",
            &source_f32,
            &rubber_f32,
        );
        mix(&mut coherent_hash, coherent_measurement.render_hash);
        mix(&mut rubber_hash, rubber_measurement.render_hash);
        mix(&mut measurement_hash, coherent_measurement.measurement_hash);
        mix(&mut measurement_hash, rubber_measurement.measurement_hash);
        rows.push(RubberBandComparisonRowEvidence {
            id: case.id,
            ratio: case.ratio,
            coherent_mean_event_offset: coherent_measurement.mean_event_offset,
            rubber_mean_event_offset: rubber_measurement.mean_event_offset,
            coherent_replica_ratio: coherent_measurement.replica_ratio,
            rubber_replica_ratio: rubber_measurement.replica_ratio,
            coherent_static_residual: coherent_measurement.static_residual,
            rubber_static_residual: rubber_measurement.static_residual,
            coherent_boundary_growth_db: coherent_measurement.boundary_growth_db,
            rubber_boundary_growth_db: rubber_measurement.boundary_growth_db,
        });
        measurements.push(coherent_measurement);
        measurements.push(rubber_measurement);
        rendered.push(RenderedRow {
            case,
            source,
            coherent: coherent.samples,
            rubber,
            raw_hashes: [coherent_file_hash, rubber_file_hash],
        });
    }

    let objective_report = development_measurement::report(&measurements);
    fs::write(evidence_root.join("input-manifest.tsv"), &input_manifest)
        .expect("write Rubber Band comparison input manifest");
    fs::write(evidence_root.join("render-receipt.tsv"), &render_receipt)
        .expect("write Rubber Band comparison render receipt");
    fs::write(
        evidence_root.join("objective-report.tsv"),
        &objective_report,
    )
    .expect("write Rubber Band comparison objective report");
    fs::write(
        evidence_root.join("README.md"),
        "# Exact-Source Rubber Band Evidence\n\nSix 44.1 kHz mono 16-bit five-second inputs feed coherent Signal and Rubber Band R3 4.0.0 at 1.5x or 2.0x. Both engines render exact target lengths twice. `input-manifest.tsv`, `render-receipt.tsv`, and `objective-report.tsv` freeze the source, engine, and measurement evidence used by the concealed pack.\n",
    )
    .expect("write Rubber Band comparison evidence readme");

    let coherent_hard_failures = measurements
        .iter()
        .step_by(2)
        .filter(|item| !development_measurement::hard_pass(item))
        .count();
    let rubber_hard_failures = measurements
        .iter()
        .skip(1)
        .step_by(2)
        .filter(|item| !development_measurement::hard_pass(item))
        .count();
    let coherent_regression_rows = [
        rows.iter()
            .filter(|row| row.coherent_mean_event_offset > row.rubber_mean_event_offset)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_replica_ratio > row.rubber_replica_ratio)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_static_residual > row.rubber_static_residual)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_boundary_growth_db > row.rubber_boundary_growth_db)
            .count(),
    ];

    let pack = pack::export(rendered);
    structural_failures[5] = pack.structural_failures[0];
    structural_failures[6] = pack.structural_failures[1];
    structural_failures[7] = pack.structural_failures[2];
    structural_failures[8] = pack.structural_failures[3];

    let coherent_repeated = structural_failures[2] == 0;
    let rubber_repeated = structural_failures[4] == 0;
    let ready = structural_failures == [0; 9]
        && coherent_hard_failures == 0
        && rubber_hard_failures == 0
        && coherent_repeated
        && rubber_repeated;
    RubberBandComparisonReview {
        rows,
        candidates_per_row: 2,
        audio_files: pack.audio_files,
        holdout_reads: 0,
        structural_failures,
        coherent_hard_failures,
        rubber_hard_failures,
        coherent_regression_rows,
        maximum_candidate_rms_delta_db: pack.maximum_candidate_rms_delta_db,
        hashes: [
            input_hash,
            coherent_hash,
            rubber_hash,
            measurement_hash,
            text_hash(&objective_report),
            text_hash(&render_receipt),
            pack.hashes[0],
            pack.hashes[1],
            pack.hashes[2],
            pack.hashes[3],
            pack.hashes[4],
            pack.hashes[5],
            pack.hashes[6],
        ],
        rubber_band_version,
        coherent_repeated,
        rubber_repeated,
        direction: if ready {
            RubberBandComparisonDirection::ConcealedListening
        } else {
            RubberBandComparisonDirection::ObjectiveFailure
        },
    }
}

fn render_rubber_band(
    binary: &std::path::Path,
    ratio: f64,
    input: &std::path::Path,
    output: &std::path::Path,
) {
    confirmation::run_command(
        binary,
        &[
            OsString::from("-q"),
            OsString::from("-3"),
            OsString::from("-t"),
            OsString::from(format!("{ratio:.6}")),
            input.as_os_str().to_os_string(),
            output.as_os_str().to_os_string(),
        ],
    );
}

fn render_receipt_row(
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

fn text_hash(text: &str) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_bytes(&mut hash, text.as_bytes());
    hash
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

fn evidence_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-db-exact-source")
}
