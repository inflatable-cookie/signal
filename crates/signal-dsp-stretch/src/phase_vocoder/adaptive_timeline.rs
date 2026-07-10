use signal_primitives::Sample;

#[derive(Clone, Debug)]
pub(super) struct AdaptiveTimelineSchedule {
    pub(super) positions: Vec<usize>,
    pub(super) reinitialize_phase: Vec<bool>,
    pub(super) protected_onset_count: usize,
    pub(super) dense_conflict_count: usize,
    pub(super) max_anchor_error_frames: f64,
    pub(super) min_synthesis_hop_frames: usize,
    pub(super) max_synthesis_hop_frames: usize,
    pub(super) schedule_fallback: bool,
}

pub(crate) struct AdaptiveTimelineEngineRender {
    pub(crate) samples: Vec<Sample>,
    pub(crate) synthesis_positions: Vec<usize>,
    pub(crate) reinitialized_frames: Vec<usize>,
    pub(crate) protected_onset_count: usize,
    pub(crate) dense_conflict_count: usize,
    pub(crate) max_anchor_error_frames: f64,
    pub(crate) min_synthesis_hop_frames: usize,
    pub(crate) max_synthesis_hop_frames: usize,
    pub(crate) uncovered_output_frames: usize,
    pub(crate) schedule_fallback: bool,
}

#[derive(Clone, Copy, Debug)]
struct OnsetIsland {
    onset_frame: usize,
    first_frame: usize,
    last_frame: usize,
}

pub(super) fn build_adaptive_timeline_schedule(
    frame_count: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    onset_frames: &[usize],
) -> AdaptiveTimelineSchedule {
    let uniform = uniform_positions(frame_count, ratio, analysis_hop);
    if frame_count < 2 || onset_frames.is_empty() {
        return schedule_from_positions(uniform, vec![false; frame_count], 0, 0, 0.0, false);
    }

    let mut islands = onset_frames
        .iter()
        .copied()
        .map(|onset_frame| onset_island(onset_frame, frame_count, window_size, analysis_hop))
        .collect::<Vec<_>>();
    islands.sort_by_key(|island| (island.first_frame, island.onset_frame));
    islands.dedup_by_key(|island| island.onset_frame);

    let mut fixed = vec![None; frame_count];
    fixed[0] = Some(uniform[0]);
    fixed[frame_count - 1] = Some(uniform[frame_count - 1]);
    let mut reinitialize_phase = vec![false; frame_count];
    let mut protected_onset_count = 0usize;
    let mut dense_conflict_count = 0usize;
    let mut max_anchor_error_frames = 0.0f64;
    let mut index = 0usize;
    while index < islands.len() {
        let mut end = index + 1;
        let mut group_last = islands[index].last_frame;
        while end < islands.len() && islands[end].first_frame <= group_last {
            group_last = group_last.max(islands[end].last_frame);
            end += 1;
        }
        if end - index > 1 {
            dense_conflict_count += end - index;
            for frame in islands[index].first_frame..=group_last {
                fixed[frame] = Some(uniform[frame]);
            }
        } else {
            let island = islands[index];
            protected_onset_count += 1;
            for frame in island.first_frame..=island.last_frame {
                let source_center = frame * analysis_hop;
                let local_position = island.onset_frame as f64 * ratio + source_center as f64
                    - island.onset_frame as f64;
                fixed[frame] = Some(local_position.round().max(0.0) as usize);
                reinitialize_phase[frame] = true;
                let mapped_onset = fixed[frame].expect("protected position") as f64
                    - source_center as f64
                    + island.onset_frame as f64;
                max_anchor_error_frames = max_anchor_error_frames
                    .max((mapped_onset - island.onset_frame as f64 * ratio).abs());
            }
        }
        index = end;
    }

    let Some(positions) = interpolate_fixed_positions(&fixed) else {
        return schedule_from_positions(
            uniform,
            vec![false; frame_count],
            0,
            onset_frames.len(),
            0.0,
            true,
        );
    };
    schedule_from_positions(
        positions,
        reinitialize_phase,
        protected_onset_count,
        dense_conflict_count,
        max_anchor_error_frames,
        false,
    )
}

fn onset_island(
    onset_frame: usize,
    frame_count: usize,
    window_size: usize,
    analysis_hop: usize,
) -> OnsetIsland {
    let half_window = window_size / 2;
    let first_frame = onset_frame
        .saturating_sub(half_window)
        .checked_div(analysis_hop)
        .unwrap_or(0)
        .saturating_add(1)
        .min(frame_count - 1);
    let last_frame = onset_frame
        .saturating_add(half_window)
        .checked_div(analysis_hop)
        .unwrap_or(0)
        .min(frame_count - 1);
    OnsetIsland {
        onset_frame,
        first_frame,
        last_frame: last_frame.max(first_frame),
    }
}

fn uniform_positions(frame_count: usize, ratio: f64, analysis_hop: usize) -> Vec<usize> {
    (0..frame_count)
        .map(|frame| (frame as f64 * analysis_hop as f64 * ratio).round() as usize)
        .collect()
}

fn interpolate_fixed_positions(fixed: &[Option<usize>]) -> Option<Vec<usize>> {
    let anchors = fixed
        .iter()
        .enumerate()
        .filter_map(|(index, position)| position.map(|position| (index, position)))
        .collect::<Vec<_>>();
    if anchors.windows(2).any(|pair| pair[0].1 >= pair[1].1) {
        return None;
    }
    let mut positions = vec![0; fixed.len()];
    for pair in anchors.windows(2) {
        let (left_frame, left_position) = pair[0];
        let (right_frame, right_position) = pair[1];
        let frame_span = right_frame - left_frame;
        let position_span = right_position - left_position;
        for frame in left_frame..=right_frame {
            let offset = frame - left_frame;
            positions[frame] = left_position
                + ((position_span as f64 * offset as f64 / frame_span as f64).round() as usize);
        }
    }
    positions
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        .then_some(positions)
}

fn schedule_from_positions(
    positions: Vec<usize>,
    reinitialize_phase: Vec<bool>,
    protected_onset_count: usize,
    dense_conflict_count: usize,
    max_anchor_error_frames: f64,
    schedule_fallback: bool,
) -> AdaptiveTimelineSchedule {
    let mut hops = positions.windows(2).map(|pair| pair[1] - pair[0]);
    let first_hop = hops.next().unwrap_or(0);
    let (min_synthesis_hop_frames, max_synthesis_hop_frames) = hops
        .fold((first_hop, first_hop), |(minimum, maximum), hop| {
            (minimum.min(hop), maximum.max(hop))
        });
    AdaptiveTimelineSchedule {
        positions,
        reinitialize_phase,
        protected_onset_count,
        dense_conflict_count,
        max_anchor_error_frames,
        min_synthesis_hop_frames,
        max_synthesis_hop_frames,
        schedule_fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_onset_is_monotonic_and_uses_local_unity_hops() {
        for ratio in [0.75, 1.25, 1.5] {
            let schedule = build_adaptive_timeline_schedule(32, ratio, 2_048, 512, &[6_144]);
            let protected = schedule
                .reinitialize_phase
                .iter()
                .enumerate()
                .filter_map(|(index, protected)| protected.then_some(index))
                .collect::<Vec<_>>();

            assert!(!schedule.schedule_fallback);
            assert_eq!(schedule.protected_onset_count, 1);
            assert_eq!(protected.len(), 4);
            assert!(schedule.positions.windows(2).all(|pair| pair[0] < pair[1]));
            assert!(protected
                .windows(2)
                .all(|pair| { schedule.positions[pair[1]] - schedule.positions[pair[0]] == 512 }));
        }
    }

    #[test]
    fn overlapping_onset_islands_use_uniform_dense_fallback() {
        let schedule = build_adaptive_timeline_schedule(32, 0.75, 2_048, 512, &[6_144, 6_400]);

        assert_eq!(schedule.protected_onset_count, 0);
        assert_eq!(schedule.dense_conflict_count, 2);
        assert!(schedule
            .reinitialize_phase
            .iter()
            .all(|protected| !protected));
        assert_eq!(schedule.positions, uniform_positions(32, 0.75, 512));
    }
}
