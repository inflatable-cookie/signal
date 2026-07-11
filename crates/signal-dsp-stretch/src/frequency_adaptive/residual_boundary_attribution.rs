use std::ops::Range;

use rustfft::num_complex::Complex64;

use super::common_grid::{hash_u64, CHANNELS, HASH_OFFSET, HOP, LOWPASS_CHANNELS};
use super::conditioning_attribution::{frame_matrix, residue_bins, FFT_FRAMES};
use super::hermitian_jacobi::{jacobi_solution, JacobiSolution};
use super::three_row_nyquist::three_row_candidate_filters;
use super::types::{
    StretchCommonGridConditioningBinEvidence as BinEvidence,
    StretchCommonGridConditioningChannelEvidence as ChannelEvidence,
    StretchCommonGridResidualBoundaryDirection as Direction,
    StretchCommonGridResidualBoundaryGroupEvidence as GroupEvidence,
    StretchCommonGridResidualBoundaryModeEvidence as ModeEvidence,
    StretchCommonGridResidualBoundaryOperator as Operator,
    StretchCommonGridResidualBoundaryResidueEvidence as ResidueEvidence,
    StretchCommonGridResidualBoundaryReview as Review,
};

const TOP: usize = 16;
const HIGH_EDGE_START: usize = CHANNELS - LOWPASS_CHANNELS;
const CANDIDATE_ROWS: usize = CHANNELS + 2;

pub(crate) fn common_grid_residual_boundary_attribution_review() -> Review {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let filters = three_row_candidate_filters().0;
    let operators = [
        Operator::Full,
        Operator::DcDiagonalized,
        Operator::HighEdgeDiagonalized,
        Operator::BothBoundaryDiagonalized,
    ];
    let mut residues = Vec::with_capacity(44);
    let mut extrema = [[f64::INFINITY, f64::NEG_INFINITY]; 4];
    let mut full_modes: [Option<(usize, f64, Vec<Complex64>, Vec<usize>)>; 2] = [None, None];
    let mut maximum_errors = [0.0_f64; 5];

    for residue in 0..FFT_FRAMES / HOP {
        let bins = residue_bins(residue, positive_bins);
        let full = frame_matrix(&filters, &bins, positive_bins);
        let dc = group_off_diagonal(&filters, &bins, positive_bins, 0..LOWPASS_CHANNELS);
        let high = group_off_diagonal(
            &filters,
            &bins,
            positive_bins,
            HIGH_EDGE_START..CHANNELS - 1,
        );
        let matrices = [
            full.clone(),
            subtract(&full, &dc),
            subtract(&full, &high),
            subtract(&subtract(&full, &dc), &high),
        ];
        let closures = [
            0.0,
            subtraction_closure(&full, &matrices[1], &dc),
            subtraction_closure(&full, &matrices[2], &high),
            subtraction_closure(&full, &matrices[3], &add(&dc, &high)),
        ];
        for (index, (operator, matrix)) in operators.into_iter().zip(&matrices).enumerate() {
            let solution = jacobi_solution(matrix);
            record_errors(&mut maximum_errors, &solution);
            maximum_errors[4] = maximum_errors[4].max(closures[index]);
            let eigenvalues = [
                solution.eigenvalues[0],
                solution.eigenvalues[bins.len() - 1],
            ];
            extrema[index][0] = extrema[index][0].min(eigenvalues[0]);
            extrema[index][1] = extrema[index][1].max(eigenvalues[1]);
            if operator == Operator::Full {
                update_mode(
                    &mut full_modes[0],
                    residue,
                    eigenvalues[0],
                    &solution,
                    &bins,
                    false,
                );
                update_mode(
                    &mut full_modes[1],
                    residue,
                    eigenvalues[1],
                    &solution,
                    &bins,
                    true,
                );
            }
            residues.push(ResidueEvidence {
                operator,
                residue,
                eigenvalues,
                condition_ratio: eigenvalues[1] / eigenvalues[0],
                jacobi: solution.evidence,
                hashes: [hash_usizes(&bins), hash_complex(matrix)],
                closure_error: closures[index],
            });
        }
    }
    let conditions = extrema.map(|values| values[1] / values[0]);
    let modes = std::array::from_fn(|index| {
        let (residue, eigenvalue, vector, bins) = full_modes[index].take().expect("full extrema");
        attribute_mode(
            index == 1,
            residue,
            eigenvalue,
            &vector,
            &bins,
            &filters,
            positive_bins,
        )
    });
    maximum_errors[4] = maximum_errors[4].max(
        modes
            .iter()
            .map(|mode| mode.closure_error)
            .fold(0.0, f64::max),
    );
    let numerical_pass = residues.iter().all(|row| passes(&row.jacobi))
        && maximum_errors[4] <= 1.0e-8
        && conditions.iter().all(|value| value.is_finite());
    let dc_pass = conditions[1] <= 1.25;
    let high_pass = conditions[2] <= 1.25;
    let both_pass = conditions[3] <= 1.25;
    let direction = if !numerical_pass {
        Direction::Inconclusive
    } else if high_pass && !dc_pass {
        Direction::HighEdgeGeometry
    } else if dc_pass && !high_pass {
        Direction::DcGeometry
    } else if both_pass {
        Direction::JointBoundaryGeometry
    } else {
        Direction::CompleteRawBank
    };
    let mut review = Review {
        residues,
        conditions,
        modes,
        maximum_errors,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = evidence_hash(&review);
    review
}

fn group_off_diagonal(
    filters: &[Complex64],
    bins: &[usize],
    positive_bins: usize,
    rows: Range<usize>,
) -> Vec<Complex64> {
    let size = bins.len();
    let mut matrix = vec![Complex64::new(0.0, 0.0); size * size];
    for channel in rows {
        for (row, bin) in bins.iter().enumerate() {
            for (column, other) in bins.iter().enumerate() {
                if row != column {
                    matrix[row * size + column] += filters[channel * positive_bins + *bin]
                        * filters[channel * positive_bins + *other].conj();
                }
            }
        }
    }
    matrix
}

fn attribute_mode(
    maximum: bool,
    residue: usize,
    eigenvalue: f64,
    vector: &[Complex64],
    bins: &[usize],
    filters: &[Complex64],
    positive_bins: usize,
) -> ModeEvidence {
    let full = frame_matrix(filters, bins, positive_bins);
    let dc = group_off_diagonal(filters, bins, positive_bins, 0..LOWPASS_CHANNELS);
    let high = group_off_diagonal(filters, bins, positive_bins, HIGH_EDGE_START..CHANNELS - 1);
    let matrices = [
        full,
        subtract(&frame_matrix(filters, bins, positive_bins), &dc),
        subtract(&frame_matrix(filters, bins, positive_bins), &high),
        subtract(
            &subtract(&frame_matrix(filters, bins, positive_bins), &dc),
            &high,
        ),
    ];
    let rayleigh = matrices
        .each_ref()
        .map(|matrix| rayleigh(matrix, vector, bins.len()));
    let width = LOWPASS_CHANNELS as f64 * 0.5 / (CHANNELS - 1) as f64;
    let mut region_mass = [0.0; 3];
    let mut top_bins = bins
        .iter()
        .zip(vector)
        .map(|(bin, value)| {
            let frequency = *bin as f64 / FFT_FRAMES as f64;
            region_mass[if frequency < width {
                0
            } else if frequency > 0.5 - width {
                2
            } else {
                1
            }] += value.norm_sqr();
            BinEvidence {
                bin: *bin,
                weight: value.norm_sqr(),
            }
        })
        .collect::<Vec<_>>();
    top_bins.sort_by(|a, b| {
        b.weight
            .total_cmp(&a.weight)
            .then_with(|| a.bin.cmp(&b.bin))
    });
    top_bins.truncate(TOP);
    let mut channels = channel_contributions(filters, bins, vector, positive_bins);
    let groups = [
        group_evidence(&channels, 0..LOWPASS_CHANNELS, eigenvalue),
        group_evidence(&channels, LOWPASS_CHANNELS..HIGH_EDGE_START, eigenvalue),
        group_evidence(&channels, HIGH_EDGE_START..CHANNELS - 1, eigenvalue),
        group_evidence(&channels, CHANNELS - 1..CANDIDATE_ROWS, eigenvalue),
    ];
    let total = channels.iter().map(|row| row.total).sum::<f64>();
    let closure_error = (total - eigenvalue).abs() / eigenvalue.abs().max(f64::MIN_POSITIVE);
    channels.sort_by(|a, b| {
        b.total
            .total_cmp(&a.total)
            .then_with(|| a.channel.cmp(&b.channel))
    });
    let mut top_total_channels = channels.clone();
    top_total_channels.truncate(TOP);
    channels.sort_by(|a, b| {
        b.cross
            .abs()
            .total_cmp(&a.cross.abs())
            .then_with(|| a.channel.cmp(&b.channel))
    });
    channels.truncate(TOP);
    ModeEvidence {
        maximum,
        residue,
        eigenvalue,
        region_mass,
        top_bins,
        top_total_channels,
        top_cross_channels: channels,
        groups,
        rayleigh,
        changes: [
            rayleigh[1] - rayleigh[0],
            rayleigh[2] - rayleigh[0],
            rayleigh[3] - rayleigh[0],
        ],
        closure_error,
        vector_hash: hash_complex(vector),
    }
}

fn channel_contributions(
    filters: &[Complex64],
    bins: &[usize],
    vector: &[Complex64],
    positive_bins: usize,
) -> Vec<ChannelEvidence> {
    (0..CANDIDATE_ROWS)
        .map(|channel| {
            let mut projection = Complex64::new(0.0, 0.0);
            let mut diagonal = 0.0;
            for (index, bin) in bins.iter().enumerate() {
                let h = filters[channel * positive_bins + *bin];
                projection += h.conj() * vector[index];
                diagonal += h.norm_sqr() * vector[index].norm_sqr();
            }
            let total = projection.norm_sqr();
            ChannelEvidence {
                channel,
                total,
                diagonal,
                cross: total - diagonal,
            }
        })
        .collect()
}

fn group_evidence(channels: &[ChannelEvidence], rows: Range<usize>, scale: f64) -> GroupEvidence {
    let start = rows.start;
    let end = rows.end;
    let total = channels[start..end]
        .iter()
        .map(|row| row.total)
        .sum::<f64>();
    let diagonal = channels[start..end]
        .iter()
        .map(|row| row.diagonal)
        .sum::<f64>();
    let cross = channels[start..end]
        .iter()
        .map(|row| row.cross)
        .sum::<f64>();
    GroupEvidence {
        rows: [start, end],
        contributions: [
            total,
            diagonal,
            cross,
            (total - diagonal - cross).abs() / scale.abs().max(f64::MIN_POSITIVE),
        ],
    }
}

fn update_mode(
    slot: &mut Option<(usize, f64, Vec<Complex64>, Vec<usize>)>,
    residue: usize,
    value: f64,
    solution: &JacobiSolution,
    bins: &[usize],
    maximum: bool,
) {
    let replace = slot.as_ref().is_none_or(|old| {
        if maximum {
            value > old.1
        } else {
            value < old.1
        }
    });
    if replace {
        let column = if maximum { bins.len() - 1 } else { 0 };
        let vector = (0..bins.len())
            .map(|row| solution.eigenvectors[row * bins.len() + column])
            .collect();
        *slot = Some((residue, value, vector, bins.to_vec()));
    }
}

fn subtract(left: &[Complex64], right: &[Complex64]) -> Vec<Complex64> {
    left.iter().zip(right).map(|(a, b)| a - b).collect()
}
fn add(left: &[Complex64], right: &[Complex64]) -> Vec<Complex64> {
    left.iter().zip(right).map(|(a, b)| a + b).collect()
}
fn subtraction_closure(full: &[Complex64], ablated: &[Complex64], expected: &[Complex64]) -> f64 {
    let numerator = full
        .iter()
        .zip(ablated)
        .zip(expected)
        .map(|((a, b), e)| (*a - *b - *e).norm_sqr())
        .sum::<f64>()
        .sqrt();
    let denominator = expected
        .iter()
        .map(|value| value.norm_sqr())
        .sum::<f64>()
        .sqrt()
        .max(f64::MIN_POSITIVE);
    numerator / denominator
}
fn rayleigh(matrix: &[Complex64], vector: &[Complex64], size: usize) -> f64 {
    (0..size)
        .map(|row| {
            vector[row].conj()
                * (0..size)
                    .map(|column| matrix[row * size + column] * vector[column])
                    .sum::<Complex64>()
        })
        .sum::<Complex64>()
        .re
}
fn record_errors(maximum: &mut [f64; 5], solution: &JacobiSolution) {
    for (slot, value) in maximum.iter_mut().zip(solution.evidence.proof_errors) {
        *slot = slot.max(value);
    }
}
fn passes(row: &super::types::StretchCommonGridJacobiEvidence) -> bool {
    row.converged
        && row.structural_errors[0] <= 1.0e-12
        && row.proof_errors[0] <= 1.0e-8
        && row.proof_errors[1] <= 1.0e-10
        && row.proof_errors[2] <= 1.0e-12
        && row.proof_errors[3] <= 1.0e-10
        && row.hashes.iter().all(|hash| *hash != 0)
}
fn hash_usizes(values: &[usize]) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in values {
        hash_u64(&mut hash, *value as u64);
    }
    hash
}
fn hash_complex(values: &[Complex64]) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in values {
        hash_u64(&mut hash, value.re.to_bits());
        hash_u64(&mut hash, value.im.to_bits());
    }
    hash
}
fn evidence_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for row in &review.residues {
        hash_u64(&mut hash, row.residue as u64);
        for value in row.eigenvalues {
            hash_u64(&mut hash, value.to_bits());
        }
        for value in row.hashes {
            hash_u64(&mut hash, value);
        }
    }
    for mode in &review.modes {
        hash_u64(&mut hash, mode.eigenvalue.to_bits());
        hash_u64(&mut hash, mode.vector_hash);
    }
    hash
}
