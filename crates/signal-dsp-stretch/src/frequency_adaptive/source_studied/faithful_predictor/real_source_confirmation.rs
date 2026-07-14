use std::{env, ffi::OsString, fs, path::PathBuf};

use super::{coherent_representation, hash_samples};
use crate::frequency_adaptive::{
    adaptive_single_frame_synthesis::development_measurement,
    complete_system_tuning::listening_export::audio::{read_mono, write_mono},
};

use super::super::{confirmation, long_form, HASH_OFFSET};

const SAMPLE_RATE: usize = 44_100;
const INPUT_FRAMES: usize = 220_500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum RealSourceConfirmationDirection {
    ConcealedMusicalComparison,
    RepresentationResearch,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RealSourceRowEvidence {
    pub(in crate::frequency_adaptive) id: &'static str,
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) coherent_mean_event_offset: f64,
    pub(in crate::frequency_adaptive) pinned_mean_event_offset: f64,
    pub(in crate::frequency_adaptive) coherent_replica_ratio: f64,
    pub(in crate::frequency_adaptive) pinned_replica_ratio: f64,
    pub(in crate::frequency_adaptive) coherent_static_residual: f64,
    pub(in crate::frequency_adaptive) pinned_static_residual: f64,
    pub(in crate::frequency_adaptive) coherent_boundary_growth_db: f64,
    pub(in crate::frequency_adaptive) pinned_boundary_growth_db: f64,
    pub(in crate::frequency_adaptive) coherent_peak_growth_db: f64,
    pub(in crate::frequency_adaptive) pinned_peak_growth_db: f64,
    pub(in crate::frequency_adaptive) coherent_hash: u64,
    pub(in crate::frequency_adaptive) pinned_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RealSourceConfirmationReview {
    pub(in crate::frequency_adaptive) rows: Vec<RealSourceRowEvidence>,
    pub(in crate::frequency_adaptive) geometry: [usize; 4],
    pub(in crate::frequency_adaptive) window_hash: u64,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 5],
    pub(in crate::frequency_adaptive) coherent_hard_failures: usize,
    pub(in crate::frequency_adaptive) pinned_hard_failures: usize,
    pub(in crate::frequency_adaptive) coherent_regression_rows: [usize; 4],
    pub(in crate::frequency_adaptive) hashes: [u64; 5],
    pub(in crate::frequency_adaptive) signalsmith_version: String,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) pinned_repeated: bool,
    pub(in crate::frequency_adaptive) direction: RealSourceConfirmationDirection,
}

pub(in crate::frequency_adaptive) fn review() -> RealSourceConfirmationReview {
    let root = output_root();
    confirmation::replace_directory(&root);
    for directory in [
        root.join("inputs"),
        root.join("coherent-signal"),
        root.join("signalsmith"),
        root.join("signalsmith-repeat"),
    ] {
        fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("create {}: {error}", directory.display()));
    }
    let binary = env::var_os("SIGNALSMITH_STRETCH_BIN")
        .map(PathBuf::from)
        .expect("set SIGNALSMITH_STRETCH_BIN to the pinned fixed-seed Signalsmith 1.3.2 CLI");
    let signalsmith_version = confirmation::version(&binary, &["-v"]);
    assert_eq!(signalsmith_version, "1.3.2");

    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let window = coherent_representation::source_kaiser_window(geometry[0], geometry[1]);
    let window_hash = hash_samples(&window);
    let mut structural_failures = [
        usize::from(geometry != [5_292, 1_323, 6_144, 3_072]),
        0,
        0,
        0,
        0,
    ];
    let mut measurements = Vec::with_capacity(12);
    let mut rows = Vec::with_capacity(6);
    let mut manifest_hash = HASH_OFFSET;
    let mut coherent_hash = HASH_OFFSET;
    let mut pinned_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut pinned_repeated = true;

    for case in long_form::cases() {
        let original = read_mono(&long_form::source_root().join(case.source));
        assert_eq!(original.len(), INPUT_FRAMES, "{}", case.source);
        let input_path = root.join("inputs").join(format!("{}.wav", case.id));
        confirmation::write_input(&input_path, &original);
        let source = confirmation::read_exact(&input_path, INPUT_FRAMES, 16);
        let target = (INPUT_FRAMES as f64 * case.ratio).round() as usize;
        mix(&mut manifest_hash, confirmation::file_hash(&input_path));

        let first = coherent_representation::render(&source, case.ratio, SAMPLE_RATE);
        let second = coherent_representation::render(&source, case.ratio, SAMPLE_RATE);
        structural_failures[1] += usize::from(first.samples.len() != target);
        structural_failures[2] += first.non_finite + first.uncovered + first.boundary_failures;
        structural_failures[3] += usize::from(first.hash != second.hash);
        let coherent_samples = first
            .samples
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        write_mono(
            &root
                .join("coherent-signal")
                .join(format!("{}.wav", case.id)),
            SAMPLE_RATE as u32,
            &first.samples,
        );

        let pinned_path = root.join("signalsmith").join(format!("{}.wav", case.id));
        confirmation::run_command(
            &binary,
            &[
                input_path.as_os_str().to_os_string(),
                pinned_path.as_os_str().to_os_string(),
                OsString::from(format!("--time={:.6}", case.ratio)),
            ],
        );
        let pinned_samples = confirmation::read_exact(&pinned_path, target, 0);
        let pinned_repeat_path = root
            .join("signalsmith-repeat")
            .join(format!("{}.wav", case.id));
        confirmation::run_command(
            &binary,
            &[
                input_path.as_os_str().to_os_string(),
                pinned_repeat_path.as_os_str().to_os_string(),
                OsString::from(format!("--time={:.6}", case.ratio)),
            ],
        );
        let pinned_repeat = confirmation::read_exact(&pinned_repeat_path, target, 0);
        pinned_repeated &= pinned_samples == pinned_repeat;
        let pinned_samples = pinned_samples
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        structural_failures[4] += pinned_samples
            .iter()
            .filter(|sample| !sample.is_finite())
            .count();

        let source_samples = source
            .iter()
            .map(|sample| *sample as f32)
            .collect::<Vec<_>>();
        let coherent = development_measurement::measure(
            case.id,
            case.ratio,
            "coherent-signal",
            &source_samples,
            &coherent_samples,
        );
        let pinned = development_measurement::measure(
            case.id,
            case.ratio,
            "signalsmith-1.3.2",
            &source_samples,
            &pinned_samples,
        );
        mix(&mut coherent_hash, coherent.render_hash);
        mix(&mut pinned_hash, pinned.render_hash);
        mix(&mut measurement_hash, coherent.measurement_hash);
        mix(&mut measurement_hash, pinned.measurement_hash);
        rows.push(RealSourceRowEvidence {
            id: case.id,
            ratio: case.ratio,
            coherent_mean_event_offset: coherent.mean_event_offset,
            pinned_mean_event_offset: pinned.mean_event_offset,
            coherent_replica_ratio: coherent.replica_ratio,
            pinned_replica_ratio: pinned.replica_ratio,
            coherent_static_residual: coherent.static_residual,
            pinned_static_residual: pinned.static_residual,
            coherent_boundary_growth_db: coherent.boundary_growth_db,
            pinned_boundary_growth_db: pinned.boundary_growth_db,
            coherent_peak_growth_db: coherent.peak_growth_db,
            pinned_peak_growth_db: pinned.peak_growth_db,
            coherent_hash: coherent.render_hash,
            pinned_hash: pinned.render_hash,
        });
        measurements.push(coherent);
        measurements.push(pinned);
    }

    let report = development_measurement::report(&measurements);
    fs::write(root.join("objective-report.tsv"), &report).expect("write objective report");
    let mut report_hash = HASH_OFFSET;
    hash_bytes(&mut report_hash, report.as_bytes());
    let coherent_hard_failures = measurements
        .iter()
        .step_by(2)
        .filter(|item| !development_measurement::hard_pass(item))
        .count();
    let pinned_hard_failures = measurements
        .iter()
        .skip(1)
        .step_by(2)
        .filter(|item| !development_measurement::hard_pass(item))
        .count();
    let coherent_regression_rows = [
        rows.iter()
            .filter(|row| row.coherent_mean_event_offset > row.pinned_mean_event_offset)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_replica_ratio > row.pinned_replica_ratio)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_static_residual > row.pinned_static_residual)
            .count(),
        rows.iter()
            .filter(|row| row.coherent_boundary_growth_db > row.pinned_boundary_growth_db)
            .count(),
    ];
    let broad_regression = coherent_regression_rows[0] >= 4
        && coherent_regression_rows[1] >= 4
        && coherent_regression_rows[2] == 6
        && coherent_regression_rows[3] == 6;
    let repeated = structural_failures[3] == 0;
    let passed = structural_failures == [0; 5]
        && coherent_hard_failures == 0
        && pinned_hard_failures == 0
        && repeated
        && pinned_repeated
        && !broad_regression;
    RealSourceConfirmationReview {
        rows,
        geometry,
        window_hash,
        structural_failures,
        coherent_hard_failures,
        pinned_hard_failures,
        coherent_regression_rows,
        hashes: [
            manifest_hash,
            coherent_hash,
            pinned_hash,
            measurement_hash,
            report_hash,
        ],
        signalsmith_version,
        repeated,
        pinned_repeated,
        direction: if passed {
            RealSourceConfirmationDirection::ConcealedMusicalComparison
        } else {
            RealSourceConfirmationDirection::RepresentationResearch
        },
    }
}

pub(super) fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-cz-confirmation")
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        mix(state, u64::from(*byte));
    }
}
