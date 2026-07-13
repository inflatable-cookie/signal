mod evidence;
mod measurement;

use super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::HASH_OFFSET;
use super::quality::{
    control::{controls, Control},
    measurement::{dense_event_errors, projected},
};
use super::render::render;
use evidence::{row_hash, RowEvidence, Stage, TraceMode};
use measurement::{classify, event_evidence, tone_evidence};

const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum AttributionDirection {
    ActivePeakPhaseAndInjectedEventOwnershipContract,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AttributionReview {
    rows: Vec<RowEvidence>,
    pub failing_rows: usize,
    pub stage_counts: [usize; 5],
    pub maximum_tone_error: f64,
    pub maximum_phase_frequency_error: f64,
    pub resolution_frequency_error: [f64; 2],
    pub peak_owner_changes: usize,
    pub maximum_isolated_error: usize,
    pub maximum_dense_error: usize,
    pub selected_event_centres: usize,
    pub exact_event_centres: usize,
    pub traced_phase_frames: usize,
    pub traced_contributions: usize,
    pub evidence_hash: u64,
    pub direction: AttributionDirection,
}

pub(in crate::frequency_adaptive) fn attribution_review() -> AttributionReview {
    let controls = controls();
    let mut rows = Vec::with_capacity(30);
    for control in [
        Control::LowTone,
        Control::MidTone,
        Control::HighTone,
        Control::IsolatedImpulse,
        Control::DenseEvent,
    ] {
        let input = &controls
            .iter()
            .find(|(candidate, _)| *candidate == control)
            .expect("frozen attribution control")
            .1;
        for ratio in RATIOS {
            for mode in [TraceMode::Ordinary, TraceMode::Combined] {
                rows.push(review_row(control, input, ratio, mode));
            }
        }
    }
    for pair in rows.chunks_exact_mut(2) {
        if !pair[0].hard_failure && pair[1].hard_failure {
            let tone = pair[1].tone.as_ref().expect("combined-only tone failure");
            pair[1].stage = if tone.event_assignments > 0 {
                Stage::EventCorrection
            } else if tone.vertical_assignments > 0 {
                Stage::VerticalLocking
            } else {
                Stage::DiagonalDualSynthesis
            };
            pair[1].hashes[8] = row_hash(&pair[1]);
        }
    }
    let failing_rows = rows.iter().filter(|row| row.hard_failure).count();
    let mut stage_counts = [0; 5];
    for row in rows.iter().filter(|row| row.hard_failure) {
        let index = match row.stage {
            Stage::GlobalTimeMap => 0,
            Stage::PhysicalFrequencyPhaseTransport => 1,
            Stage::EventCorrection => 2,
            Stage::VerticalLocking => 3,
            Stage::DiagonalDualSynthesis => 4,
            Stage::PassingAblation => continue,
        };
        stage_counts[index] += 1;
    }
    let maximum_tone_error = rows
        .iter()
        .filter_map(|row| row.tone.as_ref())
        .map(|tone| tone.output_angular_error)
        .fold(0.0_f64, f64::max);
    let maximum_phase_frequency_error = rows
        .iter()
        .filter_map(|row| row.tone.as_ref())
        .map(|tone| tone.maximum_frequency_error)
        .fold(0.0_f64, f64::max);
    let resolution_frequency_error = [0, 1].map(|index| {
        rows.iter()
            .filter_map(|row| row.tone.as_ref())
            .map(|tone| tone.resolution_error[index])
            .fold(0.0_f64, f64::max)
    });
    let peak_owner_changes = rows
        .iter()
        .filter_map(|row| row.tone.as_ref())
        .map(|tone| tone.peak_owner_changes)
        .sum();
    let maximum_isolated_error = rows
        .iter()
        .filter(|row| row.control == Control::IsolatedImpulse)
        .flat_map(|row| row.events.iter())
        .map(|event| event.displacement[0])
        .max()
        .unwrap_or(0);
    let maximum_dense_error = rows
        .iter()
        .filter(|row| row.control == Control::DenseEvent)
        .flat_map(|row| row.dense_errors)
        .max()
        .unwrap_or(0);
    let exact_event_centres = rows
        .iter()
        .flat_map(|row| row.events.iter())
        .filter(|event| event.centered)
        .count();
    let selected_event_centres = rows
        .iter()
        .flat_map(|row| row.events.iter())
        .filter(|event| event.selected)
        .count();
    let traced_phase_frames = rows
        .iter()
        .filter_map(|row| row.tone.as_ref())
        .map(|tone| tone.frames.len())
        .sum();
    let traced_contributions = rows
        .iter()
        .flat_map(|row| row.events.iter())
        .map(|event| event.contributions.len())
        .sum();
    let mut evidence_hash = HASH_OFFSET;
    for row in &rows {
        hash(&mut evidence_hash, row.hashes[8]);
    }
    AttributionReview {
        rows,
        failing_rows,
        stage_counts,
        maximum_tone_error,
        maximum_phase_frequency_error,
        resolution_frequency_error,
        peak_owner_changes,
        maximum_isolated_error,
        maximum_dense_error,
        selected_event_centres,
        exact_event_centres,
        traced_phase_frames,
        traced_contributions,
        evidence_hash,
        direction: AttributionDirection::ActivePeakPhaseAndInjectedEventOwnershipContract,
    }
}

fn review_row(control: Control, input: &[f64], ratio: f64, mode: TraceMode) -> RowEvidence {
    let channels = [input.to_vec()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let render = render(&channels, ratio, &points, &schedule, mode.render_mode());
    let tone = control.tone_hz().map(|hz| tone_evidence(&render, hz));
    let sources: &[usize] = match control {
        Control::IsolatedImpulse => &[SOURCE_FRAMES / 2],
        Control::DenseEvent => &[SOURCE_FRAMES / 2 - 128, SOURCE_FRAMES / 2 + 128],
        _ => &[],
    };
    let events = sources
        .iter()
        .map(|source| event_evidence(*source, &points, &schedule, &render))
        .collect::<Vec<_>>();
    let (dense_errors, dense_unmatched) = if control == Control::DenseEvent {
        dense_event_errors(
            &render.samples[0],
            [
                projected(&schedule, sources[0]),
                projected(&schedule, sources[1]),
            ],
        )
    } else {
        ([0; 2], 0)
    };
    let hard_failure = tone
        .as_ref()
        .is_some_and(|tone| tone.output_angular_error > 1.0e-6)
        || (control == Control::IsolatedImpulse && events[0].displacement[0] > 1)
        || (control == Control::DenseEvent
            && (dense_unmatched != 0 || dense_errors.into_iter().any(|error| error > 256)));
    let stage = classify(hard_failure, mode, tone.as_ref(), &events);
    let mut row = RowEvidence {
        control,
        ratio,
        mode,
        hard_failure,
        stage,
        tone,
        events,
        dense_errors,
        dense_unmatched,
        hashes: [
            study.hash,
            points_hash(&points),
            schedule.hash,
            render.frame_hash,
            render.coefficient_hash,
            render.trace_hashes[0],
            render.trace_hashes[1],
            render.output_hash,
            0,
        ],
    };
    row.hashes[8] = row_hash(&row);
    row
}

fn points_hash(points: &[usize]) -> u64 {
    let mut state = HASH_OFFSET;
    for point in points {
        hash(&mut state, *point as u64);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
