mod external;
mod metrics;
mod report;

use std::{fs, path::PathBuf};

use super::super::{coherent_representation, render};
use external::ExternalEngines;
use metrics::{control, evaluate, ControlKind, Metrics};

const SAMPLE_RATE: usize = 8_000;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];
const LENGTHS: [usize; 2] = [8_000, 16_384];
const PHASES: [f64; 2] = [0.0, 0.37];
const ALIGNMENTS: [bool; 2] = [true, false];
const CALIBRATED_TONE_IPD_RADIANS: f64 = 0.006;
const CALIBRATED_IMAGE_MID_SIDE_DB: f64 = 0.05;
const CALIBRATED_IMAGE_CORRELATION: f64 = 0.002;
const CALIBRATED_IMAGE_RELATION_RESIDUAL: f64 = 0.002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum GateCalibrationDirection {
    RetainExactGate,
    ReviseGate,
    RepairSignal,
    ReviseGateAndRepairSignal,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CalibrationRow {
    pub ratio: f64,
    pub source_frames: usize,
    pub phase: f64,
    pub frequency_hz: f64,
    pub bin_aligned: bool,
    pub control: &'static str,
    pub engine: &'static str,
    pub whole: Metrics,
    pub interior: Metrics,
    pub structural_failures: usize,
    pub hashes: [u64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct GateCalibrationReview {
    pub signalsmith_revision: String,
    pub signalsmith_version: String,
    pub rubber_band_version: String,
    pub rows: Vec<CalibrationRow>,
    pub repeated: bool,
    pub negative_control_sensitive: bool,
    pub negative_control_residuals: [f64; 2],
    pub direction: GateCalibrationDirection,
}

pub(in crate::frequency_adaptive) fn review() -> GateCalibrationReview {
    let engines = ExternalEngines::discover();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-linked-stereo-gate-calibration");
    external::replace_directory(&root);
    let first = run(&engines, &root.join("first"));
    let second = run(&engines, &root.join("second"));
    let repeated = first == second;
    let negative_control_residuals = metrics::negative_control_residuals();
    let negative_control_sensitive =
        negative_control_residuals[0] > 0.05 && negative_control_residuals[1] > 0.01;
    let exact_references_pass = first
        .iter()
        .filter(|row| matches!(row.engine, "ideal" | "rubber-band-r3"))
        .all(exact_gate);
    let calibrated_references_pass = first
        .iter()
        .filter(|row| matches!(row.engine, "ideal" | "rubber-band-r3"))
        .all(calibrated_gate);
    let calibrated_signal_pass = first
        .iter()
        .filter(|row| row.engine == "signal")
        .all(calibrated_gate);
    let direction = if exact_references_pass && calibrated_signal_pass {
        GateCalibrationDirection::RetainExactGate
    } else if exact_references_pass {
        GateCalibrationDirection::RepairSignal
    } else if calibrated_references_pass && !calibrated_signal_pass {
        GateCalibrationDirection::ReviseGateAndRepairSignal
    } else {
        GateCalibrationDirection::ReviseGate
    };
    report::write(
        &root,
        &engines,
        &first,
        repeated,
        negative_control_sensitive,
        negative_control_residuals,
        direction,
    );
    GateCalibrationReview {
        signalsmith_revision: engines.signalsmith_revision,
        signalsmith_version: engines.signalsmith_version,
        rubber_band_version: engines.rubber_band_version,
        rows: first,
        repeated,
        negative_control_sensitive,
        negative_control_residuals,
        direction,
    }
}

fn run(engines: &ExternalEngines, root: &std::path::Path) -> Vec<CalibrationRow> {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let output_trim = geometry[0];
    let bin_spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
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
                        let files = engines.render(root, &stem, &source, ratio, SAMPLE_RATE);
                        let quantized = &files.input;
                        let ideal = control(
                            kind,
                            (source_frames as f64 * ratio).round() as usize,
                            frequency,
                            phase,
                        );
                        let signal =
                            render::linked([&quantized[0], &quantized[1]], ratio, SAMPLE_RATE);
                        let ideal_hash = audio_hash(&ideal);
                        let candidates = [
                            ("ideal", ideal, [files.input_hash, ideal_hash]),
                            ("signal", signal.channels, [files.input_hash, signal.hash]),
                            (
                                "signalsmith",
                                files.signalsmith,
                                [files.input_hash, files.signalsmith_hash],
                            ),
                            (
                                "rubber-band-r3",
                                files.rubber_band,
                                [files.input_hash, files.rubber_band_hash],
                            ),
                        ];
                        for (engine, output, hashes) in candidates {
                            rows.push(row(
                                kind,
                                ratio,
                                source_frames,
                                phase,
                                frequency,
                                bin_aligned,
                                engine,
                                quantized,
                                output,
                                hashes,
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

fn audio_hash(channels: &[Vec<f64>; 2]) -> u64 {
    let mut hash = super::super::super::hash_samples(&channels[0]);
    super::super::hash_values(
        &mut hash,
        &[super::super::super::hash_samples(&channels[1])],
    );
    hash
}

#[allow(clippy::too_many_arguments)]
fn row(
    kind: ControlKind,
    ratio: f64,
    source_frames: usize,
    phase: f64,
    frequency: f64,
    bin_aligned: bool,
    engine: &'static str,
    input: &[Vec<f64>; 2],
    output: [Vec<f64>; 2],
    hashes: [u64; 2],
    output_trim: usize,
) -> CalibrationRow {
    let target = (source_frames as f64 * ratio).round() as usize;
    let structural_failures = usize::from(output.iter().any(|channel| channel.len() != target))
        + output
            .iter()
            .flatten()
            .filter(|sample| !sample.is_finite())
            .count();
    let source_trim = ((output_trim as f64 / ratio).ceil() as usize).min(source_frames / 3);
    let target_trim = output_trim.min(target / 3);
    CalibrationRow {
        ratio,
        source_frames,
        phase,
        frequency_hz: frequency,
        bin_aligned,
        control: kind.name(),
        engine,
        whole: evaluate(input, &output, frequency, SAMPLE_RATE),
        interior: evaluate(
            &metrics::crop(input, source_trim),
            &metrics::crop(&output, target_trim),
            frequency,
            SAMPLE_RATE,
        ),
        structural_failures,
        hashes,
    }
}

fn calibrated_gate(row: &CalibrationRow) -> bool {
    row.structural_failures == 0
        && (row.control != "tone"
            || (row.whole.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS
                && row.interior.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS))
        && (row.control != "image"
            || (row.whole.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                && row.interior.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                && row.whole.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                && row.interior.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                && row.whole.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL
                && row.interior.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL))
}

fn exact_gate(row: &CalibrationRow) -> bool {
    row.structural_failures == 0
        && (row.control != "tone"
            || (row.whole.ipd_error_radians <= 1.0e-9 && row.interior.ipd_error_radians <= 1.0e-9))
        && row.whole.mid_side_delta_db <= 0.25
        && row.interior.mid_side_delta_db <= 0.25
        && row.whole.correlation_delta <= 0.02
        && row.interior.correlation_delta <= 0.02
}
