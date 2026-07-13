pub(super) mod control;
mod evidence;
pub(super) mod measurement;

use super::super::study_local_schedule::{
    schedule::{build_schedule, Schedule},
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::HASH_OFFSET;
use super::render::{render, Mode, Render};
use control::{controls, RATIOS};
pub(in crate::frequency_adaptive) use evidence::QualityDirection;
use evidence::{case_hash, QualityReview};
use measurement::{
    angular_frequency_error, crest_db, dense_event_errors, error, peak, peak_index, projected,
    replica_ratio, rms_prefix, rms_suffix, texture,
};

const TIMING_SEARCH: usize = 512;
pub(in crate::frequency_adaptive) use control::Control;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ModeEvidence {
    pub hard_failures: [usize; 10],
    pub lengths: [usize; 2],
    pub coverage: [usize; 2],
    pub assembly_actions: [usize; 2],
    pub frame_condition: f64,
    pub symmetry_error: f64,
    pub imaginary_residue: f64,
    pub non_finite_values: usize,
    pub identity_error: [f64; 4],
    pub endpoint_rms: [f64; 2],
    pub tone_angular_error: f64,
    pub isolated_error: usize,
    pub dense_errors: [usize; 2],
    pub dense_unmatched: usize,
    pub impulse_crest_db: f64,
    pub replica_ratio: f64,
    pub texture: [f64; 6],
    pub silence_peak: f64,
    pub phase_assignments: [usize; 2],
    pub hashes: [u64; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CaseEvidence {
    pub control: Control,
    pub ratio: f64,
    pub selected_points: usize,
    pub modes: [ModeEvidence; 2],
    pub mode_deltas: [f64; 6],
    pub ownership_failures: usize,
    pub combined_regressions: usize,
    pub hash: u64,
}

pub(in crate::frequency_adaptive) fn quality_review() -> QualityReview {
    let mut cases = Vec::with_capacity(controls().len() * RATIOS.len());
    for (control, input) in controls() {
        for ratio in RATIOS {
            cases.push(review_case(control, &input, ratio));
        }
    }
    let hard_failures = cases
        .iter()
        .map(|case| {
            case.ownership_failures
                + case
                    .modes
                    .iter()
                    .flat_map(|mode| mode.hard_failures)
                    .sum::<usize>()
        })
        .sum();
    let combined_regressions = cases.iter().map(|case| case.combined_regressions).sum();
    let direction = if hard_failures == 0 && combined_regressions == 0 {
        QualityDirection::FrozenMonoDevelopmentObjective
    } else {
        QualityDirection::MeasuredPhaseEventVerticalOrSynthesisStage
    };
    let mut evidence_hash = HASH_OFFSET;
    for case in &cases {
        hash(&mut evidence_hash, case.hash);
    }
    QualityReview {
        cases,
        hard_failures,
        combined_regressions,
        evidence_hash,
        direction,
    }
}

fn review_case(control: Control, input: &[f64], ratio: f64) -> CaseEvidence {
    let channels = [input.to_vec()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let renders = [
        render(&channels, ratio, &points, &schedule, Mode::Ordinary),
        render(&channels, ratio, &points, &schedule, Mode::Both),
    ];
    let modes = [
        measure(control, input, ratio, &schedule, &renders[0]),
        measure(control, input, ratio, &schedule, &renders[1]),
    ];
    let combined_regressions = modes[0]
        .hard_failures
        .iter()
        .zip(modes[1].hard_failures)
        .filter(|(ordinary, combined)| **ordinary == 0 && *combined != 0)
        .count();
    let ownership_failures = usize::from(
        renders[0].schedule_hash != renders[1].schedule_hash
            || renders[0].frame_hash != renders[1].frame_hash
            || renders[0].coefficient_hash != renders[1].coefficient_hash
            || renders[0].magnitude_hash != renders[1].magnitude_hash,
    );
    let mode_deltas = [
        modes[1].impulse_crest_db - modes[0].impulse_crest_db,
        modes[1].replica_ratio - modes[0].replica_ratio,
        modes[1].texture[0] - modes[0].texture[0],
        modes[1].texture[1] - modes[0].texture[1],
        modes[1].texture[4] - modes[0].texture[4],
        modes[1].texture[5] - modes[0].texture[5],
    ];
    let mut result = CaseEvidence {
        control,
        ratio,
        selected_points: points.len(),
        modes,
        mode_deltas,
        ownership_failures,
        combined_regressions,
        hash: 0,
    };
    result.hash = case_hash(&result);
    result
}

fn measure(
    control: Control,
    input: &[f64],
    ratio: f64,
    schedule: &Schedule,
    render: &Render,
) -> ModeEvidence {
    let output = &render.samples[0];
    let identity_error = if ratio == 1.0 {
        error(input, output)
    } else {
        [0.0; 4]
    };
    let tone_angular_error = control
        .tone_hz()
        .map(|hz| angular_frequency_error(output, hz))
        .unwrap_or(0.0);
    let isolated_error = if control == Control::IsolatedImpulse {
        let expected = projected(schedule, SOURCE_FRAMES / 2);
        peak_index(output, expected, TIMING_SEARCH).abs_diff(expected)
    } else {
        0
    };
    let (dense_errors, dense_unmatched) = if control == Control::DenseEvent {
        dense_event_errors(
            output,
            [
                projected(schedule, SOURCE_FRAMES / 2 - 128),
                projected(schedule, SOURCE_FRAMES / 2 + 128),
            ],
        )
    } else {
        ([0; 2], 0)
    };
    let impulse_center = match control {
        Control::IsolatedImpulse | Control::Mixed => Some(projected(schedule, SOURCE_FRAMES / 2)),
        _ => None,
    };
    let impulse_crest_db = impulse_center
        .map(|center| crest_db(output, center, 256))
        .unwrap_or(0.0);
    let replica_ratio = impulse_center
        .map(|center| replica_ratio(output, center))
        .unwrap_or(0.0);
    let texture = if control.texture() {
        texture(input, output, ratio)
    } else {
        [0.0; 6]
    };
    let silence_peak = if control == Control::Silence {
        peak(output)
    } else {
        0.0
    };
    let endpoint_rms = [rms_prefix(output, 256), rms_suffix(output, 256)];
    let target_len = (SOURCE_FRAMES as f64 * ratio).round() as usize;
    let hard_failures = [
        usize::from(render.target_len != target_len || output.len() != target_len),
        render.uncovered,
        render.boundary_failures,
        render.non_finite,
        usize::from(render.symmetry_error > 1.0e-9),
        usize::from(render.imaginary_residue > 1.0e-9),
        usize::from(
            ratio == 1.0
                && (identity_error[0] > 1.0e-5
                    || identity_error[1] > 1.0e-6
                    || identity_error[2] > 1.0e-5
                    || identity_error[3] > 1.0e-5),
        ),
        usize::from(control.tone_hz().is_some() && tone_angular_error > 1.0e-6),
        usize::from(control == Control::IsolatedImpulse && isolated_error > 1),
        usize::from(
            (control == Control::DenseEvent
                && (dense_unmatched != 0 || dense_errors.into_iter().any(|error| error > 256)))
                || (control == Control::Silence && silence_peak > 1.0e-12),
        ),
    ];
    let mut hashes = [
        render.coefficient_hash,
        render.magnitude_hash,
        render.phase_hash,
        render.output_hash,
    ];
    // The zero-fill and post-fade counts are structurally zero: this renderer
    // crops the covered diagonal-dual sum directly and has no such assembly path.
    hash(&mut hashes[2], 0);
    hash(&mut hashes[2], 0);
    ModeEvidence {
        hard_failures,
        lengths: [output.len(), target_len],
        coverage: [render.uncovered, render.covered],
        assembly_actions: [0, 0],
        frame_condition: render.frame_values[2],
        symmetry_error: render.symmetry_error,
        imaginary_residue: render.imaginary_residue,
        non_finite_values: render.non_finite,
        identity_error,
        endpoint_rms,
        tone_angular_error,
        isolated_error,
        dense_errors,
        dense_unmatched,
        impulse_crest_db,
        replica_ratio,
        texture,
        silence_peak,
        phase_assignments: [render.event_phase_changes, render.vertical_phase_changes],
        hashes,
    }
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
