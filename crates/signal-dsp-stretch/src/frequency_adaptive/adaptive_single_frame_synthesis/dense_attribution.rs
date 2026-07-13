mod evidence;
mod measurement;
mod summary;

use super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::anchors::{detect, projected};
use super::quality::{control::controls, Control};
use super::render::{render_ordinary_traced, render_successor, render_successor_traced, Render};
pub(super) use evidence::SampleContribution;
use evidence::{row_hash, DenseMode, RowEvidence, Stage};
use measurement::event_evidence;
pub(super) use measurement::{matched_peaks, sample_contributions};

const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];
const SOURCES: [usize; 2] = [SOURCE_FRAMES / 2 - 128, SOURCE_FRAMES / 2 + 128];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum DenseAttributionDirection {
    SuccessorSyntheticQualityGate,
    AnchorPlacementRedesign,
    EventResetRedesign,
    ActiveOwnerTransportRedesign,
    OverlapSynthesisRedesign,
    MetricAssociationReview,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct DenseAttributionReview {
    rows: Vec<RowEvidence>,
    pub row_count: usize,
    pub failing_rows: usize,
    pub stage_counts: [usize; 5],
    pub maximum_errors: [usize; 2],
    pub row_errors: [[[usize; 2]; 3]; 2],
    pub anchor_failures: usize,
    pub reset_failures: usize,
    pub owner_failures: usize,
    pub maximum_closure_error: [f64; 2],
    pub maximum_cancellation_ratio: f64,
    pub traced_contributions: usize,
    pub failure_targets: [usize; 2],
    pub failure_peaks: [usize; 2],
    pub failure_target_values: [f64; 2],
    pub failure_peak_values: [f64; 2],
    pub failure_local_peaks: [[usize; 3]; 2],
    pub target_contributions: [Vec<evidence::SampleContribution>; 2],
    pub replica_contributions: Vec<evidence::SampleContribution>,
    pub evidence_hash: u64,
    pub direction: DenseAttributionDirection,
}

pub(in crate::frequency_adaptive) fn dense_attribution_review() -> DenseAttributionReview {
    let input = &controls()
        .into_iter()
        .find(|(control, _)| *control == Control::DenseEvent)
        .expect("dense attribution control")
        .1;
    let mut rows = Vec::with_capacity(RATIOS.len() * 2);
    for ratio in RATIOS {
        let channels = [input.clone()];
        let study = analyze(&channels, SOURCE_FRAMES);
        let points = select(&study, 3.0, 2);
        let anchors = detect(&channels, SOURCE_FRAMES);
        let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
        rows.push(review_row(
            ratio,
            DenseMode::Ordinary,
            &schedule,
            render_ordinary_traced(&channels, ratio, &points, &SOURCES, &schedule),
        ));
        let successor = render_successor(&channels, ratio, &points, &anchors.positions, &schedule);
        let targets = SOURCES.map(|source| projected(&schedule, source) as usize);
        let peaks = matched_peaks(&successor.samples[0], targets).0;
        rows.push(review_row(
            ratio,
            DenseMode::Successor,
            &schedule,
            render_successor_traced(
                &channels,
                ratio,
                &points,
                &anchors.positions,
                &peaks.map(|peak| peak as isize),
                &schedule,
            ),
        ));
    }
    summary::summarize(rows)
}

fn review_row(
    ratio: f64,
    mode: DenseMode,
    schedule: &super::super::study_local_schedule::schedule::Schedule,
    render: Render,
) -> RowEvidence {
    let targets = SOURCES.map(|source| projected(schedule, source) as usize);
    let (peaks, unmatched) = matched_peaks(&render.samples[0], targets);
    let errors = [peaks[0].abs_diff(targets[0]), peaks[1].abs_diff(targets[1])];
    let events = std::array::from_fn(|index| {
        event_evidence(SOURCES[index], targets[index], peaks[index], &render)
    });
    let peak_contributions = peaks.map(|peak| sample_contributions(&render, peak));
    let hard_failure = mode == DenseMode::Successor
        && (unmatched != 0 || errors.into_iter().any(|error| error > 256));
    let stage = classify(mode, hard_failure, &events);
    let mut row = RowEvidence {
        ratio,
        mode,
        hard_failure,
        stage,
        errors,
        unmatched,
        events,
        peak_contributions,
        hashes: [
            render.schedule_hash,
            render.frame_hash,
            render.coefficient_hash,
            render.phase_hash,
            render.trace_hashes[0],
            render.trace_hashes[1],
            render.output_hash,
            0,
        ],
    };
    row.hashes[7] = row_hash(&row);
    row
}

fn classify(mode: DenseMode, failure: bool, events: &[evidence::EventEvidence; 2]) -> Stage {
    if mode == DenseMode::Ordinary || !failure {
        return Stage::PassingControl;
    }
    if events
        .iter()
        .any(|event| !event.attached || !event.phase_found)
    {
        return Stage::AnchorPlacement;
    }
    if events.iter().any(|event| !event.event_assignment) {
        return Stage::EventReset;
    }
    if events.iter().any(|event| {
        event.active_state_hash == 0
            || event.owner_counts[3] == 0
            || event.owner_counts[0] + event.owner_counts[1] == 0
    }) {
        return Stage::ActiveOwnerTransport;
    }
    if events
        .iter()
        .any(|event| event.peak_error > 256 && event.peak_value.abs() > event.target_value.abs())
    {
        Stage::OverlapSynthesis
    } else {
        Stage::MetricAssociation
    }
}
