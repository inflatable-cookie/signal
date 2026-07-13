pub(super) mod schedule;
pub(super) mod study;

use super::HASH_OFFSET;
use schedule::{build_schedule, Schedule};
use study::{analyze, select, Study};

pub(crate) const SOURCE_FRAMES: usize = 16_384;
pub(crate) const BASE_HOP: usize = 128;
pub(crate) const CONTROL_EVENTS: [usize; 5] = [2_048, 4_096, 4_224, 8_192, 12_288];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    SyntheticPhaseAndSynthesisProof,
    StudyOrScheduleRedesign,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ControlEvidence {
    pub ratio: f64,
    pub study_frames: usize,
    pub selected_points: [usize; 2],
    pub dense_points_retained: usize,
    pub evidence_parity: bool,
    pub linked_decision_equivalence: bool,
    pub schedule_failures: [usize; 5],
    pub hop_extrema: [usize; 2],
    pub maximum_event_movement: usize,
    pub local_unity_improvement: f64,
    pub hashes: [u64; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub controls: Vec<ControlEvidence>,
    pub evidence_hash: u64,
    pub direction: Direction,
}

pub(super) fn review() -> Review {
    let controls = controls()
        .into_iter()
        .map(|(channels, ratio)| review_control(&channels, ratio))
        .collect::<Vec<_>>();
    let pass = controls.iter().all(|control| {
        control.evidence_parity
            && control.linked_decision_equivalence
            && control.selected_points[0] >= 2
            && control.dense_points_retained >= 2
            && control.local_unity_improvement > 0.0
            && control.schedule_failures == [0; 5]
    });
    let mut result = Review {
        controls,
        evidence_hash: 0,
        direction: if pass {
            Direction::SyntheticPhaseAndSynthesisProof
        } else {
            Direction::StudyOrScheduleRedesign
        },
    };
    result.evidence_hash = review_hash(&result);
    result
}

fn review_control(channels: &[Vec<f64>], ratio: f64) -> ControlEvidence {
    let enabled = analyze(channels, SOURCE_FRAMES);
    let disabled = analyze(channels, SOURCE_FRAMES);
    let mut reversed = channels.to_vec();
    reversed.reverse();
    let reversed_study = analyze(&reversed, SOURCE_FRAMES);
    let responsive = select(&enabled, 3.0, 2);
    let conservative = select(&enabled, 6.0, 3);
    let reversed_points = select(&reversed_study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &responsive);
    let evidence_parity = enabled == disabled;
    let linked_decision_equivalence = enabled == reversed_study && responsive == reversed_points;
    let schedule_failures = schedule_failures(&schedule, ratio, &responsive);
    let hops = schedule.positions.windows(2).map(|pair| pair[1] - pair[0]);
    let hop_extrema = [hops.clone().min().unwrap_or(0), hops.max().unwrap_or(0)];
    let maximum_event_movement = responsive
        .iter()
        .map(|point| {
            let index = point / BASE_HOP;
            schedule.positions[index].abs_diff((ratio * *point as f64).round() as usize)
        })
        .max()
        .unwrap_or(0);
    let local_unity_improvement = unity_improvement(&schedule, ratio, &responsive);
    ControlEvidence {
        ratio,
        study_frames: enabled.evidence.len(),
        selected_points: [responsive.len(), conservative.len()],
        dense_points_retained: dense_points(&responsive),
        evidence_parity,
        linked_decision_equivalence,
        schedule_failures,
        hop_extrema,
        maximum_event_movement,
        local_unity_improvement,
        hashes: [
            enabled.hash,
            points_hash(&responsive),
            schedule.hash,
            control_hash(&enabled, &schedule),
        ],
    }
}

fn unity_improvement(schedule: &Schedule, ratio: f64, points: &[usize]) -> f64 {
    let ideal = ratio * BASE_HOP as f64;
    let baseline_error = (ideal - BASE_HOP as f64).abs();
    let mut improvement = 0.0;
    let mut count = 0;
    for point in points
        .iter()
        .copied()
        .filter(|point| *point > 0 && *point < SOURCE_FRAMES)
    {
        let index = point / BASE_HOP;
        for hop in [
            schedule.positions[index] - schedule.positions[index - 1],
            schedule.positions[index + 1] - schedule.positions[index],
        ] {
            improvement += baseline_error - (hop as f64 - BASE_HOP as f64).abs();
            count += 1;
        }
    }
    improvement / count.max(1) as f64
}

fn schedule_failures(schedule: &Schedule, ratio: f64, points: &[usize]) -> [usize; 5] {
    let ideal = ratio * BASE_HOP as f64;
    let minimum = (ideal / 4.0).floor().max(1.0) as usize;
    let maximum = (ideal * 4.0).ceil() as usize;
    let non_positive = schedule
        .positions
        .windows(2)
        .filter(|pair| pair[1] <= pair[0])
        .count();
    let out_of_bounds = schedule
        .positions
        .windows(2)
        .filter(|pair| {
            let hop = pair[1] - pair[0];
            hop < minimum || hop > maximum
        })
        .count();
    let unordered = points.windows(2).filter(|pair| pair[1] <= pair[0]).count();
    let movement = points
        .iter()
        .filter(|point| {
            let index = **point / BASE_HOP;
            schedule.positions[index].abs_diff((ratio * **point as f64).round() as usize) > 256
        })
        .count();
    let target = (ratio * SOURCE_FRAMES as f64).round() as usize;
    [
        non_positive,
        out_of_bounds,
        unordered,
        movement,
        usize::from(schedule.positions.last() != Some(&target)),
    ]
}

fn dense_points(points: &[usize]) -> usize {
    points
        .windows(2)
        .filter(|pair| pair[1] - pair[0] <= 128)
        .flat_map(|pair| pair.iter())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

pub(crate) fn controls() -> Vec<(Vec<Vec<f64>>, f64)> {
    [0.75, 1.5, 2.0]
        .into_iter()
        .enumerate()
        .map(|(variant, ratio)| {
            let mut left = vec![0.0; SOURCE_FRAMES];
            let mut right = vec![0.0; SOURCE_FRAMES];
            for event in CONTROL_EVENTS {
                for offset in 0..32 {
                    let decay = (-(offset as f64) / 7.0).exp();
                    left[event + offset] += decay;
                    right[event + offset] += decay * (0.65 + variant as f64 * 0.1);
                }
            }
            for index in 0..SOURCE_FRAMES {
                left[index] +=
                    0.08 * (std::f64::consts::TAU * 311.0 * index as f64 / 48_000.0).sin();
                right[index] +=
                    0.06 * (std::f64::consts::TAU * 523.0 * index as f64 / 48_000.0).sin();
            }
            (vec![left, right], ratio)
        })
        .collect()
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
fn points_hash(points: &[usize]) -> u64 {
    let mut state = HASH_OFFSET;
    for point in points {
        hash(&mut state, *point as u64);
    }
    state
}
fn control_hash(study: &Study, schedule: &Schedule) -> u64 {
    let mut state = study.hash;
    hash(&mut state, schedule.hash);
    state
}
fn review_hash(review: &Review) -> u64 {
    let mut state = HASH_OFFSET;
    for control in &review.controls {
        for value in control.hashes {
            hash(&mut state, value);
        }
    }
    state
}
