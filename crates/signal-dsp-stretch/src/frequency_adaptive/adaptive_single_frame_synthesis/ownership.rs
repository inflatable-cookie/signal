mod evidence;
mod measurement;

use super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::HASH_OFFSET;
use super::anchors::{detect, projected};
use super::quality::{
    control::controls,
    measurement::{error, peak},
    Control,
};
use super::render::render_successor;
use evidence::{row_hash, RowEvidence};
use measurement::{event_errors, tone_errors};

const RATIOS: [f64; 4] = [1.0, 0.75, 1.5, 2.0];
const CONTROLS: [Control; 8] = [
    Control::LowTone,
    Control::MidTone,
    Control::HighTone,
    Control::IsolatedImpulse,
    Control::DenseEvent,
    Control::Mixed,
    Control::Boundary,
    Control::Silence,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum OwnershipDirection {
    SuccessorSyntheticQualityGate,
    ActivePeakOrTransientAnchorRedesign,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct OwnershipReview {
    rows: Vec<RowEvidence>,
    pub failure_counts: [usize; 8],
    pub maximum_identity_error: [f64; 4],
    pub maximum_tone_errors: [f64; 2],
    pub maximum_event_errors: [usize; 3],
    pub owner_counts: [usize; 4],
    pub detected_anchors: usize,
    pub expected_anchors: usize,
    pub evidence_hash: u64,
    pub direction: OwnershipDirection,
}

pub(in crate::frequency_adaptive) fn ownership_review() -> OwnershipReview {
    let controls = controls();
    let mut rows = Vec::with_capacity(CONTROLS.len() * RATIOS.len());
    for control in CONTROLS {
        let input = &controls
            .iter()
            .find(|(candidate, _)| *candidate == control)
            .expect("ownership control")
            .1;
        for ratio in RATIOS {
            rows.push(review_row(control, input, ratio));
        }
    }
    let failure_counts =
        std::array::from_fn(|index| rows.iter().map(|row| row.failures[index]).sum::<usize>());
    let maximum_identity_error = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.identity_error[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_tone_errors = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.tone_errors[index])
            .fold(0.0_f64, f64::max)
    });
    let maximum_event_errors = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.event_errors[index])
            .max()
            .unwrap_or(0)
    });
    let owner_counts = std::array::from_fn(|index| {
        rows.iter()
            .map(|row| row.owner_counts[index])
            .sum::<usize>()
    });
    let detected_anchors = rows.iter().map(|row| row.detected.len()).sum();
    let expected_anchors = rows.iter().map(|row| row.expected.len()).sum();
    let mut evidence_hash = HASH_OFFSET;
    for row in &rows {
        hash(&mut evidence_hash, row.hashes[8]);
    }
    let pass = failure_counts == [0; 8];
    OwnershipReview {
        rows,
        failure_counts,
        maximum_identity_error,
        maximum_tone_errors,
        maximum_event_errors,
        owner_counts,
        detected_anchors,
        expected_anchors,
        evidence_hash,
        direction: if pass {
            OwnershipDirection::SuccessorSyntheticQualityGate
        } else {
            OwnershipDirection::ActivePeakOrTransientAnchorRedesign
        },
    }
}

fn review_row(control: Control, input: &[f64], ratio: f64) -> RowEvidence {
    let channels = [input.to_vec()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let resolution_points = select(&study, 3.0, 2);
    let anchors = detect(&channels, SOURCE_FRAMES);
    let expected = expected(control);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &resolution_points);
    let render = render_successor(
        &channels,
        ratio,
        &resolution_points,
        &anchors.positions,
        &schedule,
    );
    let identity_error = if ratio == 1.0 {
        error(input, &render.samples[0])
    } else {
        [0.0; 4]
    };
    let tone_errors = tone_errors(control, &render);
    let event_errors = event_errors(control, &expected, &schedule, &render);
    let attachment_failures = anchors
        .positions
        .iter()
        .filter(|anchor| {
            let output = projected(&schedule, **anchor);
            !render
                .synthesis_trace
                .iter()
                .any(|frame| frame.source == **anchor as isize && frame.output == output)
        })
        .count();
    let detection_failures = detection_failures(&anchors.positions, &expected);
    let owner_counts = [
        render
            .phase_trace
            .iter()
            .map(|frame| frame.phase.owner_births)
            .sum(),
        render
            .phase_trace
            .iter()
            .map(|frame| frame.phase.owner_matches)
            .sum(),
        render
            .phase_trace
            .iter()
            .map(|frame| frame.phase.owner_retirements)
            .sum(),
        render
            .phase_trace
            .iter()
            .map(|frame| frame.phase.region_assignments)
            .sum(),
    ];
    let frame_counts = [
        render.frame_count,
        render.phase_trace.len(),
        render.synthesis_trace.len(),
    ];
    let phase_limits = [render.symmetry_error, render.imaginary_residue];
    let silence_peak = if control == Control::Silence {
        peak(&render.samples[0])
    } else {
        0.0
    };
    let target = (ratio * SOURCE_FRAMES as f64).round() as usize;
    let failures = [
        usize::from(
            render.target_len != target
                || render.samples[0].len() != target
                || render.uncovered != 0
                || render.boundary_failures != 0
                || render.non_finite != 0,
        ),
        usize::from(
            ratio == 1.0
                && (identity_error[0] > 1.0e-5
                    || identity_error[1] > 1.0e-6
                    || identity_error[2] > 1.0e-5
                    || identity_error[3] > 1.0e-5),
        ),
        usize::from(control.tone_hz().is_some() && tone_errors[0] > 1.0e-6),
        usize::from(control.tone_hz().is_some() && tone_errors[1] > 1.0e-6),
        detection_failures,
        attachment_failures,
        usize::from(
            phase_limits[0] > 1.0e-9
                || phase_limits[1] > 1.0e-9
                || frame_counts[0] != frame_counts[1]
                || frame_counts[0] != frame_counts[2]
                || (control != Control::Silence && owner_counts[3] == 0),
        ),
        usize::from(silence_peak > 1.0e-12),
    ];
    let mut row = RowEvidence {
        control,
        ratio,
        detected: anchors.positions,
        expected,
        failures,
        identity_error,
        tone_errors,
        event_errors,
        owner_counts,
        frame_counts,
        phase_limits,
        silence_peak,
        hashes: [
            anchors.grid_hash,
            anchors.anchor_hash,
            render.frame_hash,
            render.coefficient_hash,
            render.magnitude_hash,
            render.phase_hash,
            render.trace_hashes[0],
            render.output_hash,
            0,
        ],
    };
    row.hashes[8] = row_hash(&row);
    row
}

fn expected(control: Control) -> Vec<usize> {
    match control {
        Control::IsolatedImpulse | Control::Mixed => vec![SOURCE_FRAMES / 2],
        Control::DenseEvent => vec![SOURCE_FRAMES / 2 - 128, SOURCE_FRAMES / 2 + 128],
        Control::Boundary => vec![0, SOURCE_FRAMES - 1],
        _ => Vec::new(),
    }
}

fn detection_failures(detected: &[usize], expected: &[usize]) -> usize {
    detected.len().abs_diff(expected.len())
        + detected
            .iter()
            .zip(expected)
            .filter(|(actual, expected)| actual.abs_diff(**expected) > 1)
            .count()
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
