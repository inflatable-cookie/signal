use super::super::study_local_schedule::{schedule::Schedule, BASE_HOP};
use super::super::HASH_OFFSET;

const CONTRAST_LIMIT: f64 = 12.0;
const LOCAL_RADIUS: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Anchors {
    pub(super) positions: Vec<usize>,
    pub(super) grid_hash: u64,
    pub(super) anchor_hash: u64,
}

pub(super) fn detect(channels: &[Vec<f64>], source_frames: usize) -> Anchors {
    let energy = linked_energy(channels, source_frames);
    let difference = linked_difference(channels, source_frames);
    let frame_count = source_frames / BASE_HOP + 1;
    let mut positions = Vec::new();
    let mut grid_hash = HASH_OFFSET;
    for frame in 0..frame_count {
        let center = frame * BASE_HOP;
        let start = center.saturating_sub(BASE_HOP / 2);
        let end = (center + BASE_HOP / 2).min(source_frames);
        let candidate = (start..end)
            .max_by(|left, right| {
                score(*left, &energy, &difference).total_cmp(&score(*right, &energy, &difference))
            })
            .unwrap_or(start);
        let contrast = contrast(candidate, &difference);
        hash(&mut grid_hash, center as u64);
        hash(&mut grid_hash, candidate as u64);
        hash(&mut grid_hash, contrast.to_bits());
        if difference[candidate] > 0.0 && contrast >= CONTRAST_LIMIT {
            positions.push(candidate);
        }
    }
    positions.sort_unstable();
    positions.dedup();
    let mut anchor_hash = HASH_OFFSET;
    for position in &positions {
        hash(&mut anchor_hash, *position as u64);
    }
    Anchors {
        positions,
        grid_hash,
        anchor_hash,
    }
}

pub(super) fn projected(schedule: &Schedule, source: usize) -> isize {
    let left = (source / BASE_HOP).min(schedule.positions.len() - 1);
    let right = (left + 1).min(schedule.positions.len() - 1);
    if left == right {
        return schedule.positions[left] as isize;
    }
    let fraction = (source % BASE_HOP) as f64 / BASE_HOP as f64;
    ((1.0 - fraction) * schedule.positions[left] as f64
        + fraction * schedule.positions[right] as f64)
        .round() as isize
}

fn linked_energy(channels: &[Vec<f64>], source_frames: usize) -> Vec<f64> {
    (0..source_frames)
        .map(|index| {
            channels
                .iter()
                .map(|channel| channel[index] * channel[index])
                .sum()
        })
        .collect()
}

fn linked_difference(channels: &[Vec<f64>], source_frames: usize) -> Vec<f64> {
    (0..source_frames)
        .map(|index| {
            channels
                .iter()
                .map(|channel| {
                    let previous = index
                        .checked_sub(1)
                        .map(|prior| channel[prior])
                        .unwrap_or(0.0);
                    (channel[index] - previous).powi(2)
                })
                .sum()
        })
        .collect()
}

fn score(index: usize, energy: &[f64], difference: &[f64]) -> f64 {
    difference[index] * (energy[index] + 0.25 * difference[index])
}

fn contrast(index: usize, difference: &[f64]) -> f64 {
    let start = index.saturating_sub(LOCAL_RADIUS);
    let end = (index + LOCAL_RADIUS + 1).min(difference.len());
    let neighbors = end.saturating_sub(start + 1);
    let sum = difference[start..end].iter().sum::<f64>() - difference[index];
    difference[index] / (sum / neighbors.max(1) as f64).max(1.0e-12)
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
