use super::{hash, HASH_OFFSET};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Schedule {
    pub positions: Vec<usize>,
    pub hash: u64,
}

pub(super) fn build_schedule(
    source_frames: usize,
    base_hop: usize,
    ratio: f64,
    points: &[usize],
) -> Schedule {
    let frame_count = source_frames / base_hop + 1;
    let mut anchors = points
        .iter()
        .map(|point| point / base_hop)
        .collect::<Vec<_>>();
    anchors.push(0);
    anchors.push(frame_count - 1);
    anchors.sort_unstable();
    anchors.dedup();
    let mut positions = vec![0; frame_count];
    for pair in anchors.windows(2) {
        allocate_interval(&mut positions, pair[0], pair[1], base_hop, ratio);
    }
    let mut result = Schedule { positions, hash: 0 };
    result.hash = schedule_hash(&result);
    result
}

fn allocate_interval(
    positions: &mut [usize],
    start: usize,
    end: usize,
    base_hop: usize,
    ratio: f64,
) {
    let start_position = (ratio * (start * base_hop) as f64).round() as usize;
    let end_position = (ratio * (end * base_hop) as f64).round() as usize;
    positions[start] = start_position;
    if start == end {
        return;
    }
    let count = end - start;
    let ideal = ratio * base_hop as f64;
    let mut weights = (0..count)
        .map(|index| {
            let distance = index.min(count - index - 1);
            let blend = match distance {
                0 => 1.0,
                1 => 0.5,
                _ => 0.0,
            };
            ideal * (1.0 - blend) + base_hop as f64 * blend
        })
        .collect::<Vec<_>>();
    let total = (end_position - start_position) as f64;
    let scale = total / weights.iter().sum::<f64>();
    for weight in &mut weights {
        *weight *= scale;
    }
    let mut hops = weights
        .iter()
        .map(|weight| weight.floor().max(1.0) as usize)
        .collect::<Vec<_>>();
    let assigned = hops.iter().sum::<usize>();
    let remainder = end_position - start_position - assigned;
    let mut order = (0..count).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        weights[*right]
            .fract()
            .total_cmp(&weights[*left].fract())
            .then(left.cmp(right))
    });
    for index in order.into_iter().take(remainder) {
        hops[index] += 1;
    }
    for (offset, hop) in hops.into_iter().enumerate() {
        positions[start + offset + 1] = positions[start + offset] + hop;
    }
}

fn schedule_hash(schedule: &Schedule) -> u64 {
    let mut state = HASH_OFFSET;
    for position in &schedule.positions {
        hash(&mut state, *position as u64);
    }
    state
}
