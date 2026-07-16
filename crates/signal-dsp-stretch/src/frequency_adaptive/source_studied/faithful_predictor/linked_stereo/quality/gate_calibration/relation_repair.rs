mod mechanics;
mod report;
mod transform;

use std::{fs, path::PathBuf};

use super::{
    external::{self, ExternalEngines},
    metrics::{self, control, evaluate, ControlKind, Metrics},
    ALIGNMENTS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    coherent_representation, render,
};
use transform::{local_evidence, repair};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum RelationRepairDirection {
    Accept,
    Reject,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RelationRepairRow {
    pub ratio: f64,
    pub source_frames: usize,
    pub phase: f64,
    pub bin_aligned: bool,
    pub control: &'static str,
    pub applied: bool,
    pub matrix: [[f64; 2]; 2],
    pub current: [Metrics; 2],
    pub repaired: [Metrics; 2],
    pub ideal: [Metrics; 2],
    pub rubber_band: [Metrics; 2],
    pub local_windows_improved: usize,
    pub local_windows: usize,
    pub maximum_local_residuals: [f64; 2],
    pub energy_error: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct RelationRepairReview {
    pub signalsmith_revision: String,
    pub rubber_band_version: String,
    pub rows: Vec<RelationRepairRow>,
    pub mechanics_errors: [f64; 5],
    pub silent_peer_peak: f64,
    pub reference_failures: usize,
    pub repaired_failures: usize,
    pub localization_failures: usize,
    pub repeated: bool,
    pub direction: RelationRepairDirection,
}

pub(in crate::frequency_adaptive) fn review() -> RelationRepairReview {
    let engines = ExternalEngines::discover();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-linked-stereo-relation-repair");
    external::replace_directory(&root);
    let first = run(&engines, &root.join("first"));
    let second = run(&engines, &root.join("second"));
    let repeated = first == second;
    let (mechanics_errors, silent_peer_peak) = mechanics::review();
    let reference_failures = first
        .iter()
        .filter(|row| !gate(row.control, row.ideal) || !gate(row.control, row.rubber_band))
        .count();
    let repaired_failures = first
        .iter()
        .filter(|row| !gate(row.control, row.repaired))
        .count();
    let localization_failures = first
        .iter()
        .filter(|row| row.applied)
        .filter(|row| {
            row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
                || row.local_windows_improved * 2 < row.local_windows
                || row.energy_error > 1.0e-12
        })
        .count();
    let mechanics_pass =
        mechanics_errors.iter().all(|error| *error <= 1.0e-12) && silent_peer_peak == 0.0;
    let direction = if repeated
        && reference_failures == 0
        && repaired_failures == 0
        && localization_failures == 0
        && mechanics_pass
    {
        RelationRepairDirection::Accept
    } else {
        RelationRepairDirection::Reject
    };
    report::write(
        &root,
        &engines,
        &first,
        mechanics_errors,
        silent_peer_peak,
        reference_failures,
        repaired_failures,
        localization_failures,
        repeated,
        direction,
    );
    RelationRepairReview {
        signalsmith_revision: engines.signalsmith_revision,
        rubber_band_version: engines.rubber_band_version,
        rows: first,
        mechanics_errors,
        silent_peer_peak,
        reference_failures,
        repaired_failures,
        localization_failures,
        repeated,
        direction,
    }
}

fn run(engines: &ExternalEngines, root: &std::path::Path) -> Vec<RelationRepairRow> {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let trim = geometry[0];
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut rows = Vec::new();
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let stem = format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}",
                            kind.name()
                        );
                        let files = engines.render(root, &stem, &source, ratio, SAMPLE_RATE);
                        let signal =
                            render::linked([&files.input[0], &files.input[1]], ratio, SAMPLE_RATE);
                        let repaired = repair(&files.input, signal.channels);
                        let ideal = control(
                            kind,
                            (source_frames as f64 * ratio).round() as usize,
                            frequency,
                            phase,
                        );
                        rows.push(measure(
                            kind,
                            ratio,
                            source_frames,
                            phase,
                            bin_aligned,
                            frequency,
                            trim,
                            &files.input,
                            &repaired.before,
                            &repaired.channels,
                            &ideal,
                            &files.rubber_band,
                            repaired.applied,
                            repaired.matrix,
                            repaired.energy_error,
                        ));
                    }
                }
            }
        }
    }
    rows
}

#[allow(clippy::too_many_arguments)]
fn measure(
    kind: ControlKind,
    ratio: f64,
    source_frames: usize,
    phase: f64,
    bin_aligned: bool,
    frequency: f64,
    trim: usize,
    input: &[Vec<f64>; 2],
    current: &[Vec<f64>; 2],
    repaired: &[Vec<f64>; 2],
    ideal: &[Vec<f64>; 2],
    rubber_band: &[Vec<f64>; 2],
    applied: bool,
    matrix: [[f64; 2]; 2],
    energy_error: f64,
) -> RelationRepairRow {
    let source_trim = ((trim as f64 / ratio).ceil() as usize).min(source_frames / 3);
    let output_trim = trim.min(repaired[0].len() / 3);
    let pair = |output: &[Vec<f64>; 2]| {
        [
            evaluate(input, output, frequency, SAMPLE_RATE),
            evaluate(
                &metrics::crop(input, source_trim),
                &metrics::crop(output, output_trim),
                frequency,
                SAMPLE_RATE,
            ),
        ]
    };
    let (improved, windows, local) = local_evidence(input, current, repaired);
    RelationRepairRow {
        ratio,
        source_frames,
        phase,
        bin_aligned,
        control: kind.name(),
        applied,
        matrix,
        current: pair(current),
        repaired: pair(repaired),
        ideal: pair(ideal),
        rubber_band: pair(rubber_band),
        local_windows_improved: improved,
        local_windows: windows,
        maximum_local_residuals: local,
        energy_error,
    }
}

fn gate(control: &str, metrics: [Metrics; 2]) -> bool {
    metrics.into_iter().all(|metrics| {
        if control == "tone" {
            metrics.ipd_error_radians <= super::CALIBRATED_TONE_IPD_RADIANS
        } else {
            metrics.mid_side_delta_db <= super::CALIBRATED_IMAGE_MID_SIDE_DB
                && metrics.correlation_delta <= super::CALIBRATED_IMAGE_CORRELATION
                && metrics.relation_residual <= super::CALIBRATED_IMAGE_RELATION_RESIDUAL
        }
    })
}
