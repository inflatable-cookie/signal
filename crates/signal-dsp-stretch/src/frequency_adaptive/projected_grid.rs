use std::{cmp::Ordering, collections::BinaryHeap};

use signal_primitives::Sample;

use super::{
    common_grid::{
        analyze_coefficients, digital_delay, hash_u64, wrap_phase, CHANNELS, HASH_OFFSET, HOP,
    },
    types::StretchCommonGridProjectedPhaseEvidence,
};

const RELATIVE_TOLERANCE: f64 = 1.0e-6;
const HEAP_CAPACITY: usize = CHANNELS * 2;
const CHANNEL_INTERVAL: f64 = std::f64::consts::PI / (CHANNELS - 1) as f64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    magnitude: f64,
    direction: Direction,
    channel: usize,
    predecessor: usize,
    phase: f64,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.magnitude.to_bits() == other.magnitude.to_bits()
            && self.direction == other.direction
            && self.channel == other.channel
            && self.predecessor == other.predecessor
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.magnitude
            .total_cmp(&other.magnitude)
            .then_with(|| direction_rank(self.direction).cmp(&direction_rank(other.direction)))
            .then_with(|| other.channel.cmp(&self.channel))
            .then_with(|| other.predecessor.cmp(&self.predecessor))
    }
}

fn direction_rank(direction: Direction) -> u8 {
    match direction {
        Direction::Vertical => 0,
        Direction::Horizontal => 1,
    }
}

struct SourceFields {
    columns: usize,
    magnitudes: Vec<f64>,
    frequencies: Vec<f64>,
    vertical_derivatives: Vec<f64>,
    phases: Vec<f64>,
}

struct ProjectedColumn {
    magnitudes: Vec<f64>,
    frequencies: Vec<f64>,
    vertical_derivatives: Vec<f64>,
    phases: Vec<f64>,
}

pub(crate) fn common_grid_projected_phase_review_mono(
    input: &[Sample],
    ratio: f64,
) -> StretchCommonGridProjectedPhaseEvidence {
    let ratio = ratio.clamp(0.25, 4.0);
    let target_frames = (input.len() as f64 * ratio).round() as usize;
    let output_columns = target_frames.div_ceil(HOP) + 1;
    let source = source_fields(input);
    let mut max_coordinate_error = 0.0_f64;
    let mut coordinates_monotonic = true;
    let mut fractional_columns = 0;
    let mut boundary_pad_reads = 0;
    let mut projected_field_values = 0;
    let mut projected_field_hash = HASH_OFFSET;
    let mut assignment_hash = HASH_OFFSET;
    let mut seed_assignments = 0;
    let mut horizontal_assignments = 0;
    let mut vertical_assignments = 0;
    let duplicate_assignments = 0;
    let mut missing_assignments = 0;
    let mut insignificant_cells = 0;
    let mut heap_high_water = 0;
    let mut non_finite_values = 0;
    let mut previous_coordinate = None;
    let mut previous: Option<ProjectedColumn> = None;
    let mut previous_phase = vec![0.0; CHANNELS];

    for output_column in 0..output_columns {
        let coordinate = output_column as f64 / ratio;
        let lower = coordinate.floor() as usize;
        let fraction = coordinate - lower as f64;
        let upper = lower + 1;
        max_coordinate_error =
            max_coordinate_error.max(((lower as f64 + fraction) - coordinate).abs());
        if let Some(previous_coordinate) = previous_coordinate {
            coordinates_monotonic &= coordinate > previous_coordinate;
        }
        previous_coordinate = Some(coordinate);
        fractional_columns += usize::from(fraction > 0.0);
        boundary_pad_reads +=
            usize::from(lower >= source.columns) + usize::from(upper >= source.columns);

        let projected = project_column(
            &source,
            lower,
            upper,
            fraction,
            &mut projected_field_hash,
            &mut projected_field_values,
            &mut non_finite_values,
        );
        let maximum = projected.magnitudes.iter().copied().fold(0.0_f64, f64::max);
        let threshold = maximum * RELATIVE_TOLERANCE;
        let significant = projected
            .magnitudes
            .iter()
            .map(|magnitude| *magnitude > threshold)
            .collect::<Vec<_>>();
        insignificant_cells += significant.iter().filter(|value| !**value).count();

        if output_column == 0 {
            for channel in 0..CHANNELS {
                previous_phase[channel] = projected.phases[channel];
                if significant[channel] {
                    seed_assignments += 1;
                    hash_assignment(
                        &mut assignment_hash,
                        output_column,
                        channel,
                        Direction::Horizontal,
                        previous_phase[channel],
                    );
                }
            }
        } else if let Some(previous_fields) = previous.as_ref() {
            let mut assigned = vec![false; CHANNELS];
            let mut phase = projected.phases.clone();
            let mut heap = BinaryHeap::with_capacity(HEAP_CAPACITY);
            for channel in 0..CHANNELS {
                if significant[channel] {
                    heap.push(Candidate {
                        magnitude: previous_fields.magnitudes[channel],
                        direction: Direction::Horizontal,
                        channel,
                        predecessor: channel,
                        phase: previous_phase[channel]
                            + HOP as f64
                                * 0.5
                                * (previous_fields.frequencies[channel]
                                    + projected.frequencies[channel]),
                    });
                }
            }
            heap_high_water = heap_high_water.max(heap.len());
            while let Some(candidate) = heap.pop() {
                if assigned[candidate.channel] || !significant[candidate.channel] {
                    continue;
                }
                assigned[candidate.channel] = true;
                phase[candidate.channel] = candidate.phase;
                match candidate.direction {
                    Direction::Horizontal => horizontal_assignments += 1,
                    Direction::Vertical => vertical_assignments += 1,
                }
                hash_assignment(
                    &mut assignment_hash,
                    output_column,
                    candidate.channel,
                    candidate.direction,
                    candidate.phase,
                );
                push_vertical_neighbors(
                    candidate.channel,
                    candidate.phase,
                    &projected,
                    &significant,
                    &assigned,
                    &mut heap,
                );
                heap_high_water = heap_high_water.max(heap.len());
            }
            missing_assignments += significant
                .iter()
                .zip(&assigned)
                .filter(|(significant, assigned)| **significant && !**assigned)
                .count();
            non_finite_values += phase.iter().filter(|value| !value.is_finite()).count();
            previous_phase = phase;
        }
        previous = Some(projected);
    }

    StretchCommonGridProjectedPhaseEvidence {
        ratio,
        target_frames,
        source_columns: source.columns,
        output_columns,
        max_coordinate_error,
        coordinates_monotonic,
        fractional_columns,
        boundary_pad_reads,
        projected_field_values,
        seed_assignments,
        horizontal_assignments,
        vertical_assignments,
        duplicate_assignments,
        missing_assignments,
        insignificant_cells,
        heap_high_water,
        heap_capacity: HEAP_CAPACITY,
        non_finite_values,
        projected_field_hash,
        assignment_hash,
    }
}

fn source_fields(input: &[Sample]) -> SourceFields {
    let fft_frames = input.len().max(HOP).div_ceil(HOP) * HOP;
    let columns = fft_frames / HOP;
    let (coefficients, derivatives) = analyze_coefficients(input, fft_frames);
    let mut magnitudes = vec![0.0; coefficients.len()];
    let mut frequencies = vec![0.0; coefficients.len()];
    let mut phases = vec![0.0; coefficients.len()];
    for channel in 0..CHANNELS {
        let center = std::f64::consts::PI * channel as f64 / (CHANNELS - 1) as f64;
        for column in 0..columns {
            let index = channel * columns + column;
            let energy = coefficients[index].norm_sqr();
            magnitudes[index] = energy.sqrt();
            frequencies[index] = if energy > f64::MIN_POSITIVE {
                (derivatives[index] * coefficients[index].conj()).im / energy
            } else {
                center
            };
            phases[index] = wrap_phase(
                coefficients[index].arg()
                    - frequencies[index] * HOP as f64 * digital_delay(channel),
            );
        }
    }
    let mut vertical_derivatives = vec![0.0; coefficients.len()];
    for channel in 0..CHANNELS - 1 {
        for column in 0..columns {
            let index = channel * columns + column;
            let right = (channel + 1) * columns + column;
            vertical_derivatives[index] =
                wrap_phase(phases[right] - phases[index]) / CHANNEL_INTERVAL;
        }
    }
    for column in 0..columns {
        vertical_derivatives[(CHANNELS - 1) * columns + column] =
            vertical_derivatives[(CHANNELS - 2) * columns + column];
    }
    SourceFields {
        columns,
        magnitudes,
        frequencies,
        vertical_derivatives,
        phases,
    }
}

#[allow(clippy::too_many_arguments)]
fn project_column(
    source: &SourceFields,
    lower: usize,
    upper: usize,
    fraction: f64,
    field_hash: &mut u64,
    field_values: &mut usize,
    non_finite_values: &mut usize,
) -> ProjectedColumn {
    let lower_sample = lower.min(source.columns - 1);
    let upper_sample = upper.min(source.columns - 1);
    let nearest = if fraction <= 0.5 {
        lower_sample
    } else {
        upper_sample
    };
    let mut projected = ProjectedColumn {
        magnitudes: Vec::with_capacity(CHANNELS),
        frequencies: Vec::with_capacity(CHANNELS),
        vertical_derivatives: Vec::with_capacity(CHANNELS),
        phases: Vec::with_capacity(CHANNELS),
    };
    for channel in 0..CHANNELS {
        let lower_index = channel * source.columns + lower_sample;
        let upper_index = channel * source.columns + upper_sample;
        let magnitude = interpolate(
            source.magnitudes[lower_index],
            source.magnitudes[upper_index],
            fraction,
        );
        let frequency = interpolate(
            source.frequencies[lower_index],
            source.frequencies[upper_index],
            fraction,
        );
        let vertical = interpolate(
            source.vertical_derivatives[lower_index],
            source.vertical_derivatives[upper_index],
            fraction,
        );
        let phase = source.phases[channel * source.columns + nearest];
        for value in [magnitude, frequency, vertical] {
            hash_u64(field_hash, value.to_bits());
            *field_values += 1;
            *non_finite_values += usize::from(!value.is_finite());
        }
        projected.magnitudes.push(magnitude);
        projected.frequencies.push(frequency);
        projected.vertical_derivatives.push(vertical);
        projected.phases.push(phase);
    }
    projected
}

fn interpolate(lower: f64, upper: f64, fraction: f64) -> f64 {
    lower + fraction * (upper - lower)
}

fn push_vertical_neighbors(
    channel: usize,
    phase: f64,
    projected: &ProjectedColumn,
    significant: &[bool],
    assigned: &[bool],
    heap: &mut BinaryHeap<Candidate>,
) {
    if channel + 1 < CHANNELS && significant[channel + 1] && !assigned[channel + 1] {
        if heap.len() < HEAP_CAPACITY {
            heap.push(Candidate {
                magnitude: projected.magnitudes[channel + 1],
                direction: Direction::Vertical,
                channel: channel + 1,
                predecessor: channel,
                phase: phase
                    + 0.5
                        * (projected.vertical_derivatives[channel]
                            + projected.vertical_derivatives[channel + 1])
                        * CHANNEL_INTERVAL,
            });
        }
    }
    if channel > 0 && significant[channel - 1] && !assigned[channel - 1] {
        if heap.len() < HEAP_CAPACITY {
            heap.push(Candidate {
                magnitude: projected.magnitudes[channel - 1],
                direction: Direction::Vertical,
                channel: channel - 1,
                predecessor: channel,
                phase: phase
                    - 0.5
                        * (projected.vertical_derivatives[channel]
                            + projected.vertical_derivatives[channel - 1])
                        * CHANNEL_INTERVAL,
            });
        }
    }
}

fn hash_assignment(
    hash: &mut u64,
    column: usize,
    channel: usize,
    direction: Direction,
    phase: f64,
) {
    hash_u64(hash, column as u64);
    hash_u64(hash, channel as u64);
    hash_u64(hash, u64::from(direction_rank(direction)));
    hash_u64(hash, phase.to_bits());
}
