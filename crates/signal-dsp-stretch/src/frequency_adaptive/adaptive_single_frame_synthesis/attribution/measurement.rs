use std::f64::consts::TAU;

use super::super::super::study_local_schedule::schedule::Schedule;
use super::super::quality::{
    control::SAMPLE_RATE,
    measurement::{angular_frequency_error, peak_index, projected},
};
use super::super::render::{Render, SynthesisFrameTrace};
use super::evidence::{
    ContributionEvidence, EventEvidence, Stage, ToneEvidence, ToneFrameEvidence, TraceMode,
};

const SEARCH: usize = 512;

pub(super) fn tone_evidence(render: &Render, hz: f64) -> ToneEvidence {
    let expected = TAU * hz / SAMPLE_RATE;
    let traces = render
        .phase_trace
        .windows(2)
        .map(|pair| {
            let trace = &pair[1];
            ToneFrameEvidence {
                source: trace.source,
                output: trace.output,
                length: trace.length,
                hops: [trace.phase.source_hop, trace.phase.output_hop],
                bins: [
                    trace.phase.prior_bin,
                    trace.phase.bin,
                    trace.phase.peak_owner,
                ],
                frequency_error: (trace.phase.estimated_frequency - expected).abs(),
                advance_error: [
                    wrap(trace.phase.transported_advance - expected * trace.phase.output_hop).abs(),
                    wrap(trace.phase.final_advance - expected * trace.phase.output_hop).abs(),
                ],
                assignments: [
                    trace.phase.event_assignment,
                    trace.phase.vertical_assignment,
                ],
            }
        })
        .collect::<Vec<_>>();
    let mut resolution_error = [0.0_f64; 2];
    for (pair, trace) in render.phase_trace.windows(2).zip(&traces) {
        let index = usize::from(pair[0].length != pair[1].length);
        resolution_error[index] = resolution_error[index].max(trace.frequency_error);
    }
    ToneEvidence {
        output_angular_error: angular_frequency_error(&render.samples[0], hz),
        maximum_frequency_error: maximum(&traces, |trace| trace.frequency_error),
        maximum_transport_advance_error: maximum(&traces, |trace| trace.advance_error[0]),
        maximum_final_advance_error: maximum(&traces, |trace| trace.advance_error[1]),
        peak_owner_changes: traces
            .iter()
            .filter(|trace| trace.bins[0] != trace.bins[1])
            .count(),
        event_assignments: traces.iter().filter(|trace| trace.assignments[0]).count(),
        vertical_assignments: traces.iter().filter(|trace| trace.assignments[1]).count(),
        resolution_error,
        frames: traces,
    }
}

pub(super) fn event_evidence(
    source: usize,
    points: &[usize],
    schedule: &Schedule,
    render: &Render,
) -> EventEvidence {
    let scheduled = projected(schedule, source);
    let overlaps = render
        .synthesis_trace
        .iter()
        .filter(|trace| trace.source.abs_diff(source as isize) <= trace.length / 2)
        .collect::<Vec<_>>();
    let dominant = overlaps
        .iter()
        .copied()
        .max_by(|left, right| left.energy.total_cmp(&right.energy))
        .expect("event overlap");
    let actual_peak = peak_index(&render.samples[0], scheduled, SEARCH);
    let output_energy_center = local_energy_center(&render.samples[0], scheduled, SEARCH);
    EventEvidence {
        source,
        scheduled,
        selected: points.contains(&source),
        centered: render
            .synthesis_trace
            .iter()
            .any(|trace| trace.source == source as isize),
        overlapping_frames: overlaps.len(),
        event_assignments: render
            .phase_trace
            .iter()
            .filter(|trace| trace.source == source as isize && trace.phase.event_assignment)
            .count(),
        vertical_assignments: overlaps
            .iter()
            .filter(|trace| has_vertical_assignment(render, trace))
            .count(),
        dominant_frame: [dominant.source, dominant.output, dominant.peak_output],
        displacement: [
            actual_peak.abs_diff(scheduled),
            (output_energy_center.round().max(0.0) as usize).abs_diff(scheduled),
            dominant.peak_output.abs_diff(scheduled as isize),
        ],
        replica_peaks: replica_peaks(&render.samples[0], scheduled),
        contributions: overlaps.iter().map(|trace| contribution(trace)).collect(),
    }
}

pub(super) fn classify(
    failure: bool,
    mode: TraceMode,
    tone: Option<&ToneEvidence>,
    events: &[EventEvidence],
) -> Stage {
    if !failure {
        return Stage::PassingAblation;
    }
    if let Some(tone) = tone {
        if tone.maximum_frequency_error > 1.0e-6 || tone.maximum_transport_advance_error > 1.0e-6 {
            return Stage::PhysicalFrequencyPhaseTransport;
        }
        if mode == TraceMode::Combined && tone.event_assignments > 0 {
            return Stage::EventCorrection;
        }
        if mode == TraceMode::Combined && tone.vertical_assignments > 0 {
            return Stage::VerticalLocking;
        }
        return Stage::DiagonalDualSynthesis;
    }
    if events.iter().any(|event| !event.centered) {
        Stage::GlobalTimeMap
    } else if events.iter().any(|event| event.displacement[2] > 1) {
        Stage::PhysicalFrequencyPhaseTransport
    } else {
        Stage::DiagonalDualSynthesis
    }
}

fn maximum(traces: &[ToneFrameEvidence], field: impl Fn(&ToneFrameEvidence) -> f64) -> f64 {
    traces.iter().map(field).fold(0.0_f64, f64::max)
}

fn contribution(trace: &&SynthesisFrameTrace) -> ContributionEvidence {
    ContributionEvidence {
        source: trace.source,
        output: trace.output,
        length: trace.length,
        energy: trace.energy,
        energy_center: trace.energy_center,
        peak_output: trace.peak_output,
        peak_magnitude: trace.peak_magnitude,
        hash: trace.hash,
    }
}

fn has_vertical_assignment(render: &Render, trace: &&SynthesisFrameTrace) -> bool {
    render
        .phase_trace
        .iter()
        .find(|phase| phase.source == trace.source)
        .is_some_and(|phase| phase.phase.vertical_assignment)
}

fn local_energy_center(samples: &[f64], center: usize, radius: usize) -> f64 {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    let (energy, moment) =
        samples[start..end]
            .iter()
            .enumerate()
            .fold((0.0, 0.0), |sum, (offset, sample)| {
                let square = sample * sample;
                (sum.0 + square, sum.1 + (start + offset) as f64 * square)
            });
    moment / energy
}

fn replica_peaks(samples: &[f64], center: usize) -> [usize; 3] {
    let start = center.saturating_sub(SEARCH);
    let end = (center + SEARCH + 1).min(samples.len());
    let mut candidates = (start..end).collect::<Vec<_>>();
    let mut result = [start; 3];
    for peak in &mut result {
        let index = *candidates
            .iter()
            .max_by(|left, right| samples[**left].abs().total_cmp(&samples[**right].abs()))
            .expect("replica candidates");
        *peak = index;
        candidates.retain(|candidate| candidate.abs_diff(index) > 32);
    }
    result
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(TAU) - std::f64::consts::PI
}
