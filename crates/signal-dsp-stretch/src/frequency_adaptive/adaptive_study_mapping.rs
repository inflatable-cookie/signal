use std::collections::BTreeSet;

use super::study_local_schedule::{
    controls,
    schedule::{build_schedule, Schedule},
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::time_adaptive_painless::adaptive_schedule_for_points;
use super::types::{
    StretchAdaptiveStudyMappingDirection as Direction,
    StretchAdaptiveStudyMappingEvidence as Evidence, StretchAdaptiveStudyMappingReview as Review,
};
use super::HASH_OFFSET;

const WINDOW_LENGTHS: [usize; 4] = [512, 1_024, 2_048, 4_096];

pub(crate) fn adaptive_study_time_map_review() -> Review {
    let controls = controls()
        .into_iter()
        .map(|(channels, ratio)| review_control(&channels, ratio))
        .collect::<Vec<_>>();
    let pass = controls.iter().all(|control| {
        control.level_mapping_failures == [0; 4]
            && control.structural_failures == [0; 8]
            && control.maximum_event_movement <= 256
            && control.non_finite_values == 0
    });
    let mut review = Review {
        controls,
        evidence_hash: 0,
        direction: if pass {
            Direction::SingleFramePhaseContract
        } else {
            Direction::StudyMappingRedesign
        },
    };
    review.evidence_hash = review_hash(&review);
    review
}

fn review_control(channels: &[Vec<f64>], ratio: f64) -> Evidence {
    let study = analyze(channels, SOURCE_FRAMES);
    let selected_points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &selected_points);
    let frames = adaptive_schedule_for_points(SOURCE_FRAMES, &selected_points);
    let output_centres = project_centres(&frames, ratio, &schedule);

    let mut reversed = channels.to_vec();
    reversed.reverse();
    let reversed_study = analyze(&reversed, SOURCE_FRAMES);
    let reversed_points = select(&reversed_study, 3.0, 2);
    let reversed_schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &reversed_points);
    let reversed_frames = adaptive_schedule_for_points(SOURCE_FRAMES, &reversed_points);
    let reversed_output = project_centres(&reversed_frames, ratio, &reversed_schedule);

    let source_centres = frames.iter().map(|frame| frame.0).collect::<Vec<_>>();
    let mut window_counts = [0; 4];
    for frame in &frames {
        window_counts[level(frame.1)] += 1;
    }
    let in_range_indices = source_centres
        .iter()
        .enumerate()
        .filter_map(|(index, center)| {
            (*center >= 0 && *center <= SOURCE_FRAMES as isize).then_some(index)
        })
        .collect::<Vec<_>>();
    let reflected = source_centres.len() - in_range_indices.len();
    let source_hops = in_range_indices
        .windows(2)
        .map(|pair| source_centres[pair[1]].abs_diff(source_centres[pair[0]]));
    let output_hops = in_range_indices
        .windows(2)
        .map(|pair| output_centres[pair[1]].abs_diff(output_centres[pair[0]]));
    let mut level_mapping_failures = [0; 4];
    for index in &in_range_indices {
        let source = source_centres[*index] as usize;
        let expected = schedule.positions[source / BASE_HOP] as isize;
        if output_centres[*index] != expected {
            level_mapping_failures[level(frames[*index].1)] += 1;
        }
    }
    let duplicate_sources = duplicate_count(&source_centres);
    let in_range_outputs = in_range_indices
        .iter()
        .map(|index| output_centres[*index])
        .collect::<Vec<_>>();
    let duplicate_outputs = duplicate_count(&in_range_outputs);
    let off_grid = source_centres
        .iter()
        .filter(|center| **center % BASE_HOP as isize != 0)
        .count();
    let illegal_transitions = frames
        .windows(2)
        .filter(|pair| {
            level(pair[0].1).abs_diff(level(pair[1].1)) > 1
                || pair[1].0 - pair[0].0 != pair[0].1.min(pair[1].1) as isize / 4
        })
        .count();
    let non_positive_source_hops = source_centres
        .windows(2)
        .filter(|pair| pair[1] <= pair[0])
        .count();
    let non_positive_output_hops = in_range_outputs
        .windows(2)
        .filter(|pair| pair[1] <= pair[0])
        .count();
    let target = (ratio * SOURCE_FRAMES as f64).round() as usize;
    let endpoint_mismatch = usize::from(schedule.positions.last() != Some(&target));
    let linked_order_mismatch = usize::from(
        study != reversed_study
            || selected_points != reversed_points
            || schedule.positions != reversed_schedule.positions
            || frames != reversed_frames
            || output_centres != reversed_output,
    );
    let maximum_event_movement = selected_points
        .iter()
        .map(|point| {
            schedule.positions[*point / BASE_HOP].abs_diff((ratio * *point as f64).round() as usize)
        })
        .max()
        .unwrap_or(0);
    let ownership_hash = frame_hash(&frames);
    let mapping_hash = mapping_hash(&source_centres, &output_centres);
    let mut evidence = Evidence {
        ratio,
        selected_points: selected_points.clone(),
        window_counts,
        frame_counts: [frames.len(), in_range_indices.len(), reflected],
        hop_extrema: [
            source_hops.clone().min().unwrap_or(0),
            source_hops.max().unwrap_or(0),
            output_hops.clone().min().unwrap_or(0),
            output_hops.max().unwrap_or(0),
        ],
        source_centres,
        output_centres,
        level_mapping_failures,
        structural_failures: [
            duplicate_sources,
            duplicate_outputs,
            off_grid,
            illegal_transitions,
            non_positive_source_hops,
            non_positive_output_hops,
            endpoint_mismatch,
            linked_order_mismatch,
        ],
        maximum_event_movement,
        non_finite_values: usize::from(!ratio.is_finite()),
        hashes: [
            study.hash,
            points_hash(&selected_points),
            schedule.hash,
            ownership_hash,
            mapping_hash,
            0,
        ],
    };
    evidence.hashes[5] = evidence_hash(&evidence);
    evidence
}

fn project_centres(frames: &[(isize, usize)], ratio: f64, schedule: &Schedule) -> Vec<isize> {
    frames
        .iter()
        .map(|frame| {
            if frame.0 < 0 || frame.0 > SOURCE_FRAMES as isize {
                (ratio * frame.0 as f64).round() as isize
            } else {
                schedule.positions[frame.0 as usize / BASE_HOP] as isize
            }
        })
        .collect()
}

fn duplicate_count(values: &[isize]) -> usize {
    values.len() - values.iter().collect::<BTreeSet<_>>().len()
}

fn level(length: usize) -> usize {
    WINDOW_LENGTHS
        .iter()
        .position(|candidate| *candidate == length)
        .unwrap_or(WINDOW_LENGTHS.len() - 1)
}

fn points_hash(points: &[usize]) -> u64 {
    let mut state = HASH_OFFSET;
    for point in points {
        hash(&mut state, *point as u64);
    }
    state
}

fn frame_hash(frames: &[(isize, usize)]) -> u64 {
    let mut state = HASH_OFFSET;
    for frame in frames {
        hash(&mut state, frame.0 as i64 as u64);
        hash(&mut state, frame.1 as u64);
    }
    state
}

fn mapping_hash(source: &[isize], output: &[isize]) -> u64 {
    let mut state = HASH_OFFSET;
    for (source, output) in source.iter().zip(output) {
        hash(&mut state, *source as i64 as u64);
        hash(&mut state, *output as i64 as u64);
    }
    state
}

fn evidence_hash(evidence: &Evidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, evidence.ratio.to_bits());
    for value in evidence
        .window_counts
        .into_iter()
        .chain(evidence.frame_counts)
        .chain(evidence.hop_extrema)
        .chain(evidence.level_mapping_failures)
        .chain(evidence.structural_failures)
    {
        hash(&mut state, value as u64);
    }
    hash(&mut state, evidence.maximum_event_movement as u64);
    hash(&mut state, evidence.non_finite_values as u64);
    for value in &evidence.hashes[..5] {
        hash(&mut state, *value);
    }
    state
}

fn review_hash(review: &Review) -> u64 {
    let mut state = HASH_OFFSET;
    for control in &review.controls {
        hash(&mut state, control.hashes[5]);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
