use super::super::super::study_local_schedule::{schedule::Schedule, BASE_HOP, SOURCE_FRAMES};
use super::super::super::time_adaptive_painless::adaptive_schedule_for_points;
use super::super::super::HASH_OFFSET;
use super::super::anchors::projected;
use super::{window, Frame};

pub(super) fn legacy(ratio: f64, points: &[usize], schedule: &Schedule) -> Vec<Frame> {
    adaptive_schedule_for_points(SOURCE_FRAMES, points)
        .into_iter()
        .map(|(source, length)| Frame {
            source,
            output: if source < 0 || source > SOURCE_FRAMES as isize {
                (ratio * source as f64).round() as isize
            } else {
                schedule.positions[source as usize / BASE_HOP] as isize
            },
            length,
        })
        .collect()
}

pub(super) fn successor(
    ratio: f64,
    resolution_points: &[usize],
    anchors: &[usize],
    schedule: &Schedule,
) -> Vec<Frame> {
    let mut geometry = resolution_points.to_vec();
    geometry.extend_from_slice(anchors);
    geometry.sort_unstable();
    geometry.dedup();
    let mut frames = adaptive_schedule_for_points(SOURCE_FRAMES, &geometry)
        .into_iter()
        .map(|(source, length)| Frame {
            source,
            output: if source < 0 || source > SOURCE_FRAMES as isize {
                (ratio * source as f64).round() as isize
            } else {
                projected(schedule, source as usize)
            },
            length,
        })
        .collect::<Vec<_>>();
    let mut claimed = Vec::with_capacity(anchors.len());
    for anchor in anchors {
        let (index, _) = frames
            .iter()
            .enumerate()
            .filter(|(index, _)| !claimed.contains(index))
            .min_by_key(|(_, frame)| frame.source.abs_diff(*anchor as isize))
            .expect("adaptive anchor frame");
        frames[index].source = *anchor as isize;
        frames[index].output = projected(schedule, *anchor);
        claimed.push(index);
    }
    frames.sort_by_key(|frame| frame.source);
    frames
}

pub(super) fn frame_hash(frames: &[Frame]) -> u64 {
    let mut state = HASH_OFFSET;
    for frame in frames {
        hash(&mut state, frame.source as i64 as u64);
        hash(&mut state, frame.output as i64 as u64);
        hash(&mut state, frame.length as u64);
    }
    state
}

pub(super) fn dual_hash(frames: &[Frame], operator: &[f64], output_start: isize) -> u64 {
    let mut state = HASH_OFFSET;
    for frame in frames {
        for (offset, weight) in window(frame.length).into_iter().enumerate() {
            if weight == 0.0 {
                continue;
            }
            let logical = frame.output - frame.length as isize / 2 + offset as isize;
            let domain = (logical - output_start) as usize;
            hash(&mut state, (weight / operator[domain]).to_bits());
        }
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
