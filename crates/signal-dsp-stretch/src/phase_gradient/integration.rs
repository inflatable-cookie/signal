use std::{cmp::Ordering, collections::BinaryHeap};

use super::{
    analysis::Analysis, trace_hash_value, EvidenceAccumulator, BINS, RELATIVE_TOLERANCE,
    SYNTHESIS_HOP,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeapFrame {
    Previous,
    Current,
}

#[derive(Clone, Copy, Debug)]
struct HeapEntry {
    magnitude: f32,
    frame: HeapFrame,
    bin: usize,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.magnitude.to_bits() == other.magnitude.to_bits()
            && self.frame == other.frame
            && self.bin == other.bin
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.magnitude
            .total_cmp(&other.magnitude)
            .then_with(|| heap_frame_rank(self.frame).cmp(&heap_frame_rank(other.frame)))
            .then_with(|| other.bin.cmp(&self.bin))
    }
}

fn heap_frame_rank(frame: HeapFrame) -> u8 {
    match frame {
        HeapFrame::Previous => 0,
        HeapFrame::Current => 1,
    }
}

pub(super) fn integrate_frame(
    analysis: &Analysis,
    time_derivatives: &[Vec<f32>],
    frequency_derivative: &[f32],
    frame: usize,
    previous_synthesis_phase: &[f32],
    ratio: f64,
    accumulator: &mut EvidenceAccumulator,
) -> Vec<f32> {
    let previous_magnitude = &analysis.magnitudes[frame - 1];
    let current_magnitude = &analysis.magnitudes[frame];
    let absolute_tolerance = RELATIVE_TOLERANCE
        * previous_magnitude
            .iter()
            .chain(current_magnitude)
            .copied()
            .fold(0.0_f32, f32::max);
    let mut significant = current_magnitude
        .iter()
        .map(|magnitude| *magnitude > absolute_tolerance)
        .collect::<Vec<_>>();
    let significant_count = significant.iter().filter(|owned| **owned).count();
    accumulator.significant_bins += significant_count;
    accumulator.insignificant_bins += BINS - significant_count;

    let mut phase = analysis.phases[frame].clone();
    let mut assigned = vec![false; BINS];
    let mut heap = BinaryHeap::with_capacity(BINS * 2);
    for bin in 0..BINS {
        if significant[bin] {
            heap.push(HeapEntry {
                magnitude: previous_magnitude[bin],
                frame: HeapFrame::Previous,
                bin,
            });
        }
    }
    accumulator.heap_high_water = accumulator.heap_high_water.max(heap.len());

    let frequency_scale = ratio as f32;
    let mut remaining = significant_count;
    while remaining > 0 {
        let Some(entry) = heap.pop() else {
            break;
        };
        match entry.frame {
            HeapFrame::Previous if significant[entry.bin] => {
                phase[entry.bin] = previous_synthesis_phase[entry.bin]
                    + SYNTHESIS_HOP as f32
                        * 0.5
                        * (time_derivatives[frame - 1][entry.bin]
                            + time_derivatives[frame][entry.bin]);
                remaining -= usize::from(assign_bin(
                    entry.bin,
                    HeapFrame::Previous,
                    current_magnitude,
                    &mut significant,
                    &mut assigned,
                    &mut heap,
                    accumulator,
                ));
            }
            HeapFrame::Current => {
                if entry.bin + 1 < BINS && significant[entry.bin + 1] {
                    phase[entry.bin + 1] = phase[entry.bin]
                        + frequency_scale
                            * 0.5
                            * (frequency_derivative[entry.bin]
                                + frequency_derivative[entry.bin + 1]);
                    remaining -= usize::from(assign_bin(
                        entry.bin + 1,
                        HeapFrame::Current,
                        current_magnitude,
                        &mut significant,
                        &mut assigned,
                        &mut heap,
                        accumulator,
                    ));
                }
                if entry.bin > 0 && significant[entry.bin - 1] {
                    phase[entry.bin - 1] = phase[entry.bin]
                        - frequency_scale
                            * 0.5
                            * (frequency_derivative[entry.bin]
                                + frequency_derivative[entry.bin - 1]);
                    remaining -= usize::from(assign_bin(
                        entry.bin - 1,
                        HeapFrame::Current,
                        current_magnitude,
                        &mut significant,
                        &mut assigned,
                        &mut heap,
                        accumulator,
                    ));
                }
            }
            HeapFrame::Previous => {}
        }
    }
    accumulator.missing_assignments += significant.iter().filter(|owned| **owned).count();
    phase
}

#[allow(clippy::too_many_arguments)]
fn assign_bin(
    bin: usize,
    direction: HeapFrame,
    current_magnitude: &[f32],
    significant: &mut [bool],
    assigned: &mut [bool],
    heap: &mut BinaryHeap<HeapEntry>,
    accumulator: &mut EvidenceAccumulator,
) -> bool {
    if assigned[bin] {
        accumulator.duplicate_assignments += 1;
        return false;
    }
    assigned[bin] = true;
    significant[bin] = false;
    match direction {
        HeapFrame::Previous => accumulator.horizontal_assignments += 1,
        HeapFrame::Current => accumulator.vertical_assignments += 1,
    }
    trace_hash_value(&mut accumulator.trace_hash, bin as u64);
    trace_hash_value(
        &mut accumulator.trace_hash,
        heap_frame_rank(direction) as u64,
    );
    heap.push(HeapEntry {
        magnitude: current_magnitude[bin],
        frame: HeapFrame::Current,
        bin,
    });
    accumulator.heap_high_water = accumulator.heap_high_water.max(heap.len());
    true
}
