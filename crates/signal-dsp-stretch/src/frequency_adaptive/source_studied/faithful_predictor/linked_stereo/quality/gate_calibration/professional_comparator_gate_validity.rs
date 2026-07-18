mod inputs;
mod measurement;
mod mechanics_measurement;
mod report;
mod specimen;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use specimen::{Render, RubberBand};

use super::{
    external::replace_directory, metrics::Metrics, CALIBRATED_IMAGE_CORRELATION,
    CALIBRATED_IMAGE_MID_SIDE_DB, CALIBRATED_IMAGE_RELATION_RESIDUAL, CALIBRATED_TONE_IPD_RADIANS,
};

const EXACT_MECHANICS_LIMIT: f64 = 1.0e-6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum ProfessionalComparatorGateDirection {
    RetainRules,
    SeparateExactDiagnostics,
    ReviseLocalConsistency,
    ReviseLocalAndExactMechanics,
    ReviseCalibratedGate,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ProfessionalComparatorGateReview {
    pub(in crate::frequency_adaptive) rubber_band_version: String,
    pub(in crate::frequency_adaptive) binary_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) stereo_rows: usize,
    pub(in crate::frequency_adaptive) calibrated_failures: usize,
    pub(in crate::frequency_adaptive) signal_relative_local_failures: usize,
    pub(in crate::frequency_adaptive) mechanics_errors: [f64; 6],
    pub(in crate::frequency_adaptive) exact_mechanics_failures: usize,
    pub(in crate::frequency_adaptive) input_hash: u64,
    pub(in crate::frequency_adaptive) output_hash: u64,
    pub(in crate::frequency_adaptive) command_hash: u64,
    pub(in crate::frequency_adaptive) measurement_hash: u64,
    pub(in crate::frequency_adaptive) comparator_envelope_hash: u64,
    pub(in crate::frequency_adaptive) evidence_hash: u64,
    pub(in crate::frequency_adaptive) direction: ProfessionalComparatorGateDirection,
}

#[derive(Clone, Debug, PartialEq)]
struct StereoRow {
    ratio: f64,
    source_frames: usize,
    phase: f64,
    bin_aligned: bool,
    control: &'static str,
    whole: Metrics,
    interior: Metrics,
    structural_failures: usize,
    local_windows_improved: usize,
    maximum_local_residuals: [f64; 2],
    local_residuals: [[f64; 8]; 2],
    input_hash: u64,
    output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: Vec<StereoRow>,
    mechanics_errors: [f64; 6],
    input_hash: u64,
    output_hash: u64,
    command_hash: u64,
    measurement_hash: u64,
    evidence_hash: u64,
}

pub(in crate::frequency_adaptive) fn review() -> ProfessionalComparatorGateReview {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-professional-comparator-gate-validity");
    replace_directory(&root);
    let prepared = inputs::prepare();
    let mechanics_inputs = mechanics_measurement::prepare();
    let specimen = RubberBand::discover();
    let first = measurement::run(&specimen, &root.join("first"), &prepared, &mechanics_inputs);
    let second = measurement::run(
        &specimen,
        &root.join("second"),
        &prepared,
        &mechanics_inputs,
    );
    let repeated = first == second;
    let calibrated_failures = first
        .rows
        .iter()
        .filter(|row| !calibrated_gate(row))
        .count();
    let signal_relative_local_failures = first
        .rows
        .iter()
        .filter(|row| {
            row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
                || row.local_windows_improved < 4
        })
        .count();
    let exact_mechanics_failures = first
        .mechanics_errors
        .iter()
        .filter(|error| **error > EXACT_MECHANICS_LIMIT)
        .count();
    let direction = if calibrated_failures > 0 {
        ProfessionalComparatorGateDirection::ReviseCalibratedGate
    } else if signal_relative_local_failures > 0 && exact_mechanics_failures > 0 {
        ProfessionalComparatorGateDirection::ReviseLocalAndExactMechanics
    } else if signal_relative_local_failures > 0 {
        ProfessionalComparatorGateDirection::ReviseLocalConsistency
    } else if exact_mechanics_failures > 0 {
        ProfessionalComparatorGateDirection::SeparateExactDiagnostics
    } else {
        ProfessionalComparatorGateDirection::RetainRules
    };
    report::write(
        &root,
        &specimen,
        &first,
        repeated,
        calibrated_failures,
        signal_relative_local_failures,
        exact_mechanics_failures,
        direction,
    );
    ProfessionalComparatorGateReview {
        rubber_band_version: specimen.version,
        binary_hash: specimen.binary_hash,
        repeated,
        stereo_rows: first.rows.len(),
        calibrated_failures,
        signal_relative_local_failures,
        mechanics_errors: first.mechanics_errors,
        exact_mechanics_failures,
        input_hash: first.input_hash,
        output_hash: first.output_hash,
        command_hash: first.command_hash,
        measurement_hash: first.measurement_hash,
        comparator_envelope_hash: comparator_envelope_hash(&first.rows),
        evidence_hash: first.evidence_hash,
        direction,
    }
}

fn calibrated_gate(row: &StereoRow) -> bool {
    row.structural_failures == 0
        && if row.control == "tone" {
            [row.whole, row.interior]
                .into_iter()
                .all(|value| value.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS)
        } else {
            [row.whole, row.interior].into_iter().all(|value| {
                value.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                    && value.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                    && value.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL
            })
        }
}

fn add_render_hashes(hashes: &mut [u64; 4], render: &Render) {
    mix(&mut hashes[0], render.input_hash);
    mix(&mut hashes[1], render.output_hash);
    mix(&mut hashes[2], render.command_hash);
}

fn comparator_envelope_hash(rows: &[StereoRow]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for row in rows {
        for value in row.local_residuals[1] {
            mix(&mut hash, value.to_bits());
        }
    }
    hash
}

fn mix(hash: &mut u64, value: u64) {
    *hash = (*hash ^ value).wrapping_mul(0x100_0000_01b3);
}
