mod anchors;
mod attribution;
mod dense_attribution;
mod development_measurement;
mod development_objective;
mod mechanism_attribution;
mod overlap_ownership;
mod ownership;
pub(super) mod quality;
mod render;
mod resolution_attribution;
mod stage_attribution;
mod window_attribution;

pub(super) use attribution::{attribution_review, AttributionDirection};
pub(super) use dense_attribution::{dense_attribution_review, DenseAttributionDirection};
pub(super) use development_objective::{development_objective_review, DevelopmentDirection};
pub(super) use mechanism_attribution::{
    mechanism_attribution_review, MechanismAttributionDirection,
};
pub(super) use overlap_ownership::overlap_ownership_review;
pub(super) use ownership::{ownership_review, OwnershipDirection};
pub(super) use quality::{
    owned_successor_quality_review, quality_review, successor_quality_review, QualityDirection,
};
pub(super) use resolution_attribution::{
    resolution_attribution_review, ResolutionAttributionDirection,
};
pub(super) use stage_attribution::{stage_attribution_review, StageAttributionDirection};
pub(super) use window_attribution::{window_attribution_review, WindowAttributionDirection};

use render::{render, Mode, Render};

use super::study_local_schedule::{
    controls,
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, CONTROL_EVENTS, SOURCE_FRAMES,
};
use super::HASH_OFFSET;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    FixedRatioMonoObjectiveGate,
    OutputScheduleFrameCoupling,
    PhaseOrSynthesisRedesign,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Evidence {
    pub ratio: f64,
    pub selected_points: usize,
    pub frame_counts: [usize; 2],
    pub phase_state_counts: [usize; 2],
    pub coverage: [usize; 2],
    pub frame_values: [f64; 3],
    pub structural_failures: [usize; 8],
    pub identity_peak_error: f64,
    pub tone_frequency_error_hz: f64,
    pub maximum_event_error: usize,
    pub event_phase_changes: usize,
    pub vertical_phase_changes: usize,
    pub maximum_symmetry_error: f64,
    pub maximum_imaginary_residue: f64,
    pub non_finite_values: usize,
    pub hashes: [u64; 11],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub controls: Vec<Evidence>,
    pub evidence_hash: u64,
    pub direction: Direction,
}

pub(super) fn review() -> Review {
    let frozen = controls();
    let mut cases = Vec::with_capacity(frozen.len() + 1);
    cases.push((frozen[0].0.clone(), 1.0));
    cases.extend(frozen);
    let controls = cases
        .into_iter()
        .map(|(channels, ratio)| review_control(&channels, ratio))
        .collect::<Vec<_>>();
    let coverage_pass = controls.iter().all(|control| {
        control.coverage[0] == 0
            && control.frame_values[0].is_finite()
            && control.frame_values[0] > 0.0
            && control.frame_values[2].is_finite()
    });
    let phase_pass = controls.iter().all(|control| {
        control.structural_failures == [0; 8]
            && control.phase_state_counts[0] > 0
            && control.phase_state_counts[1] == 2
            && control.identity_peak_error <= 5.0e-12
            && control.tone_frequency_error_hz <= 2.0
            && control.maximum_event_error <= 256
            && control.event_phase_changes > 0
            && control.vertical_phase_changes > 0
            && control.maximum_symmetry_error <= 2.0e-10
            && control.maximum_imaginary_residue <= 2.0e-10
            && control.non_finite_values == 0
    });
    let direction = if !coverage_pass {
        Direction::OutputScheduleFrameCoupling
    } else if phase_pass {
        Direction::FixedRatioMonoObjectiveGate
    } else {
        Direction::PhaseOrSynthesisRedesign
    };
    let mut result = Review {
        controls,
        evidence_hash: 0,
        direction,
    };
    result.evidence_hash = review_hash(&result);
    result
}

fn review_control(channels: &[Vec<f64>], ratio: f64) -> Evidence {
    let study = analyze(channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let modes = [Mode::Ordinary, Mode::Event, Mode::Vertical, Mode::Both]
        .map(|mode| render(channels, ratio, &points, &schedule, mode));
    let mut reversed = channels.to_vec();
    reversed.reverse();
    let reversed_both = render(&reversed, ratio, &points, &schedule, Mode::Both);
    let baseline = &modes[0];
    let target_len = (ratio * SOURCE_FRAMES as f64).round() as usize;
    let structural_failures = [
        modes
            .iter()
            .filter(|render| {
                render.target_len != target_len
                    || render
                        .samples
                        .iter()
                        .any(|channel| channel.len() != target_len)
            })
            .count(),
        modes.iter().map(|render| render.uncovered).sum(),
        modes.iter().map(|render| render.boundary_failures).sum(),
        modes.iter().map(|render| render.event_order_failures).sum(),
        modes
            .iter()
            .filter(|render| {
                render.schedule_hash != baseline.schedule_hash
                    || render.frame_hash != baseline.frame_hash
            })
            .count(),
        modes
            .iter()
            .filter(|render| {
                render.coefficient_hash != baseline.coefficient_hash
                    || render.magnitude_hash != baseline.magnitude_hash
            })
            .count(),
        usize::from(reversed_both.decision_hash != modes[3].decision_hash),
        modes
            .iter()
            .filter(|render| render.phase_initializations != channels.len())
            .count(),
    ];
    let identity_peak_error = if ratio == 1.0 {
        modes
            .iter()
            .flat_map(|render| render.samples.iter().zip(channels))
            .flat_map(|(output, input)| output.iter().zip(input))
            .map(|(output, input)| (output - input).abs())
            .fold(0.0_f64, f64::max)
    } else {
        0.0
    };
    let tone_frequency_error_hz = (dominant_frequency(&modes[3].samples[0], 311, 12) - 311.0).abs();
    let maximum_event_error = CONTROL_EVENTS
        .into_iter()
        .map(|point| {
            let expected = schedule.positions[point / BASE_HOP];
            peak_index(&modes[3].samples[0], expected, 512).abs_diff(expected)
        })
        .max()
        .unwrap_or(0);
    let event_phase_changes = modes[1].event_phase_changes + modes[3].event_phase_changes;
    let vertical_phase_changes = modes[2].vertical_phase_changes + modes[3].vertical_phase_changes;
    let maximum_symmetry_error = modes
        .iter()
        .map(|render| render.symmetry_error)
        .fold(0.0_f64, f64::max);
    let maximum_imaginary_residue = modes
        .iter()
        .map(|render| render.imaginary_residue)
        .fold(0.0_f64, f64::max);
    let non_finite_values = modes.iter().map(|render| render.non_finite).sum();
    let mut hashes = [
        study.hash,
        points_hash(&points),
        schedule.hash,
        baseline.frame_hash,
        baseline.dual_hash,
        combined_hash(&modes, |render| render.coefficient_hash),
        combined_hash(&modes, |render| render.magnitude_hash),
        combined_hash(&modes, |render| render.phase_hash),
        combined_hash(&modes, |render| render.decision_hash),
        combined_hash(&modes, |render| render.output_hash),
        0,
    ];
    let frame_counts = [baseline.frame_count, baseline.resolution_changes];
    let phase_state_counts = [baseline.resolution_changes, baseline.phase_initializations];
    let coverage = [baseline.uncovered, baseline.covered];
    let frame_values = baseline.frame_values;
    let mut evidence = Evidence {
        ratio,
        selected_points: points.len(),
        frame_counts,
        phase_state_counts,
        coverage,
        frame_values,
        structural_failures,
        identity_peak_error,
        tone_frequency_error_hz,
        maximum_event_error,
        event_phase_changes,
        vertical_phase_changes,
        maximum_symmetry_error,
        maximum_imaginary_residue,
        non_finite_values,
        hashes,
    };
    hashes[10] = evidence_hash(&evidence);
    evidence.hashes = hashes;
    evidence
}

fn dominant_frequency(samples: &[f64], center: usize, radius: usize) -> f64 {
    let start = samples.len() * 55 / 100;
    let end = samples.len() * 72 / 100;
    let first_quarter_hz = center.saturating_sub(radius) * 4;
    let last_quarter_hz = (center + radius) * 4;
    (first_quarter_hz..=last_quarter_hz)
        .map(|quarter_hz| {
            let frequency = quarter_hz as f64 / 4.0;
            let omega = std::f64::consts::TAU * frequency / 48_000.0;
            let length = end - start;
            let coefficient =
                samples[start..end]
                    .iter()
                    .enumerate()
                    .fold((0.0, 0.0), |sum, (index, sample)| {
                        let weight = 0.5
                            - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos();
                        (
                            sum.0 + weight * sample * (omega * index as f64).cos(),
                            sum.1 - weight * sample * (omega * index as f64).sin(),
                        )
                    });
            (
                frequency,
                coefficient.0 * coefficient.0 + coefficient.1 * coefficient.1,
            )
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map(|result| result.0)
        .unwrap_or(0.0)
}

fn peak_index(samples: &[f64], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

fn points_hash(points: &[usize]) -> u64 {
    let mut state = HASH_OFFSET;
    for point in points {
        hash(&mut state, *point as u64);
    }
    state
}

fn combined_hash(modes: &[Render; 4], field: impl Fn(&Render) -> u64) -> u64 {
    let mut state = HASH_OFFSET;
    for mode in modes {
        hash(&mut state, field(mode));
    }
    state
}

fn evidence_hash(evidence: &Evidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, evidence.ratio.to_bits());
    for value in evidence
        .frame_counts
        .into_iter()
        .chain(evidence.phase_state_counts)
        .chain(evidence.coverage)
        .chain(evidence.structural_failures)
    {
        hash(&mut state, value as u64);
    }
    for value in evidence.frame_values {
        hash(&mut state, value.to_bits());
    }
    hash(&mut state, evidence.identity_peak_error.to_bits());
    hash(&mut state, evidence.tone_frequency_error_hz.to_bits());
    hash(&mut state, evidence.maximum_event_error as u64);
    hash(&mut state, evidence.event_phase_changes as u64);
    hash(&mut state, evidence.vertical_phase_changes as u64);
    hash(&mut state, evidence.maximum_symmetry_error.to_bits());
    hash(&mut state, evidence.maximum_imaginary_residue.to_bits());
    hash(&mut state, evidence.non_finite_values as u64);
    for value in &evidence.hashes[..10] {
        hash(&mut state, *value);
    }
    state
}

fn review_hash(review: &Review) -> u64 {
    let mut state = HASH_OFFSET;
    for control in &review.controls {
        hash(&mut state, control.hashes[10]);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
