use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use super::{
    calibrated_gate,
    external::{command_text, file_hash, read_stereo, replace_directory, run, write_stereo},
    metrics::{control, ControlKind},
    row, CalibrationRow, ALIGNMENTS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standard,
    CentreFocus,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Standard => "rubber-band-r3-standard",
            Self::CentreFocus => "rubber-band-r3-centre-focus",
        }
    }

    fn args(self, ratio: f64, input: &Path, output: &Path) -> Vec<OsString> {
        let mut args = vec!["-q".into(), "-3".into()];
        if self == Self::CentreFocus {
            args.push("--centre-focus".into());
        }
        args.extend([
            "-t".into(),
            format!("{ratio:.9}").into(),
            input.as_os_str().into(),
            output.as_os_str().into(),
        ]);
        args
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum MechanismStudyDirection {
    PeakTrajectoryRepair,
    Pause,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct MechanismStudyReview {
    pub rubber_band_version: String,
    pub rows: Vec<CalibrationRow>,
    pub repeated: bool,
    pub standard_failures: usize,
    pub centre_focus_failures: usize,
    pub changed_pairs: usize,
    pub direction: MechanismStudyDirection,
}

pub(in crate::frequency_adaptive) fn review() -> MechanismStudyReview {
    let rubber_band = env::var_os("RUBBERBAND_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| "rubberband".into());
    let rubber_band_version = command_text(&rubber_band, &["--version".into()]);
    assert_eq!(rubber_band_version, "4.0.0");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-rubber-band-linked-stereo-mechanism");
    replace_directory(&root);
    let first = matrix(&rubber_band, &root.join("first"));
    let second = matrix(&rubber_band, &root.join("second"));
    let repeated = first == second;
    let standard_failures = failures(&first, Mode::Standard);
    let centre_focus_failures = failures(&first, Mode::CentreFocus);
    let changed_pairs = first
        .chunks_exact(2)
        .filter(|pair| pair[0].hashes[1] != pair[1].hashes[1])
        .count();
    let direction = if repeated
        && standard_failures == 0
        && centre_focus_failures > 0
        && changed_pairs == first.len() / 2
    {
        MechanismStudyDirection::PeakTrajectoryRepair
    } else {
        MechanismStudyDirection::Pause
    };
    write_report(
        &root,
        &rubber_band_version,
        &first,
        repeated,
        standard_failures,
        centre_focus_failures,
        changed_pairs,
        direction,
    );
    MechanismStudyReview {
        rubber_band_version,
        rows: first,
        repeated,
        standard_failures,
        centre_focus_failures,
        changed_pairs,
        direction,
    }
}

fn matrix(rubber_band: &Path, root: &Path) -> Vec<CalibrationRow> {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let output_trim = super::super::super::coherent_representation::source_geometry(SAMPLE_RATE)[0];
    let bin_spacing = SAMPLE_RATE as f64
        / super::super::super::coherent_representation::source_geometry(SAMPLE_RATE)[2] as f64;
    let mut rows = Vec::new();
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * bin_spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let stem = format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}",
                            kind.name()
                        );
                        let input_path = root.join(format!("{stem}-input.wav"));
                        write_stereo(&input_path, &source, SAMPLE_RATE as u32);
                        let input = read_stereo(&input_path, source_frames, SAMPLE_RATE as u32);
                        let input_hash = file_hash(&input_path);
                        for mode in [Mode::Standard, Mode::CentreFocus] {
                            let output_path = root.join(format!("{stem}-{}.wav", mode.name()));
                            run(rubber_band, &mode.args(ratio, &input_path, &output_path));
                            let target = (source_frames as f64 * ratio).round() as usize;
                            let output = read_stereo(&output_path, target, SAMPLE_RATE as u32);
                            rows.push(row(
                                kind,
                                ratio,
                                source_frames,
                                phase,
                                frequency,
                                bin_aligned,
                                mode.name(),
                                &input,
                                output,
                                [input_hash, file_hash(&output_path)],
                                output_trim,
                            ));
                        }
                    }
                }
            }
        }
    }
    rows
}

fn failures(rows: &[CalibrationRow], mode: Mode) -> usize {
    rows.iter()
        .filter(|row| row.engine == mode.name() && !calibrated_gate(row))
        .count()
}

fn write_report(
    root: &Path,
    version: &str,
    rows: &[CalibrationRow],
    repeated: bool,
    standard_failures: usize,
    centre_focus_failures: usize,
    changed_pairs: usize,
    direction: MechanismStudyDirection,
) {
    let mut report = format!(
        "rubber_band_version\t{version}\nrepeated\t{repeated}\nstandard_failures\t{standard_failures}\ncentre_focus_failures\t{centre_focus_failures}\nchanged_pairs\t{changed_pairs}\ndirection\t{direction:?}\nratio\tframes\tphase\tfrequency\tbin_aligned\tcontrol\tengine\tscope\tipd_error\tmid_side_delta_db\tcorrelation_delta\trelation_residual\tstructural_failures\tinput_hash\toutput_hash\n"
    );
    for row in rows {
        for (scope, metrics) in [("whole", row.whole), ("interior", row.interior)] {
            report.push_str(&format!("{:.2}\t{}\t{:.2}\t{:.6}\t{}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{:016x}\t{:016x}\n", row.ratio, row.source_frames, row.phase, row.frequency_hz, row.bin_aligned, row.control, row.engine, scope, metrics.ipd_error_radians, metrics.mid_side_delta_db, metrics.correlation_delta, metrics.relation_residual, row.structural_failures, row.hashes[0], row.hashes[1]));
        }
    }
    fs::write(root.join("mechanism-study.tsv"), report)
        .expect("write Rubber Band linked-stereo mechanism report");
}
