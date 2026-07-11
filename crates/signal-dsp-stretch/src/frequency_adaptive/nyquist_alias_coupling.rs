use rustfft::num_complex::Complex64;

use super::common_grid::{build_boundary_candidate_filters, hash_u64, CHANNELS, HASH_OFFSET, HOP};
use super::conditioning_attribution::{frame_matrix, normalize_exact, residue_bins, FFT_FRAMES};
use super::hermitian_jacobi::{jacobi_solution, JacobiSolution};
use super::types::{
    StretchCommonGridNyquistAblationDirection as Direction,
    StretchCommonGridNyquistAblationGlobalEvidence as GlobalEvidence,
    StretchCommonGridNyquistAblationModeEvidence as ModeEvidence,
    StretchCommonGridNyquistAblationOperator as Operator,
    StretchCommonGridNyquistAblationResidueEvidence as ResidueEvidence,
    StretchCommonGridNyquistAblationReview as Review,
};

const COMPLETION_CHANNEL: usize = CHANNELS - 1;

pub(crate) fn common_grid_nyquist_alias_coupling_review() -> Review {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let (filters, filter_hash) =
        normalize_exact(build_boundary_candidate_filters(FFT_FRAMES), positive_bins);
    let operators = [
        Operator::Full,
        Operator::CompletionRemoved,
        Operator::CompletionDiagonalized,
    ];
    let mut residues = Vec::with_capacity(3 * FFT_FRAMES / HOP);
    let mut extrema = [
        [(f64::INFINITY, 0_usize), (f64::NEG_INFINITY, 0_usize)],
        [(f64::INFINITY, 0_usize), (f64::NEG_INFINITY, 0_usize)],
        [(f64::INFINITY, 0_usize), (f64::NEG_INFINITY, 0_usize)],
    ];
    let mut full_modes: [Option<(usize, f64, Vec<Complex64>, Vec<usize>)>; 2] = [None, None];
    let mut maximum_errors = [0.0_f64; 5];

    for residue in 0..FFT_FRAMES / HOP {
        let bins = residue_bins(residue, positive_bins);
        let full = frame_matrix(&filters, &bins, positive_bins);
        let completion = completion_vector(&filters, &bins, positive_bins);
        let removed = subtract_completion(&full, &completion, false);
        let diagonalized = subtract_completion(&full, &completion, true);
        let matrices = [full, removed, diagonalized];
        let energy = completion_energy(&completion);

        for (operator_index, (operator, matrix)) in
            operators.into_iter().zip(matrices.iter()).enumerate()
        {
            let solution = jacobi_solution(matrix);
            record_jacobi_errors(&mut maximum_errors, &solution);
            let minimum = solution.eigenvalues[0];
            let maximum = solution.eigenvalues[bins.len() - 1];
            if minimum < extrema[operator_index][0].0 {
                extrema[operator_index][0] = (minimum, residue);
            }
            if maximum > extrema[operator_index][1].0 {
                extrema[operator_index][1] = (maximum, residue);
            }
            if operator == Operator::Full {
                update_mode(
                    &mut full_modes[0],
                    residue,
                    minimum,
                    &solution,
                    &bins,
                    false,
                );
                update_mode(&mut full_modes[1], residue, maximum, &solution, &bins, true);
            }
            residues.push(ResidueEvidence {
                operator,
                residue,
                bin_count: bins.len(),
                eigenvalues: [minimum, maximum],
                condition_ratio: maximum / minimum,
                jacobi: solution.evidence,
                completion_energy: energy,
                hashes: [hash_usizes(&bins), hash_complex(matrix)],
            });
        }
    }

    let globals = std::array::from_fn(|index| GlobalEvidence {
        operator: operators[index],
        eigenvalues: [extrema[index][0].0, extrema[index][1].0],
        residues: [extrema[index][0].1, extrema[index][1].1],
        condition_ratio: extrema[index][1].0 / extrema[index][0].0,
    });
    let modes = std::array::from_fn(|index| {
        let (residue, eigenvalue, vector, bins) = full_modes[index].take().expect("full extrema");
        mode_evidence(
            index == 1,
            residue,
            eigenvalue,
            &vector,
            &bins,
            &filters,
            positive_bins,
        )
    });
    maximum_errors[4] = modes
        .iter()
        .flat_map(|mode| mode.closure_errors)
        .fold(0.0, f64::max);
    let numerical_pass = residues.iter().all(|row| passes(&row.jacobi))
        && maximum_errors[4] <= 1.0e-8
        && globals.iter().all(|row| {
            row.eigenvalues.iter().all(|value| value.is_finite()) && row.condition_ratio.is_finite()
        });
    let direction = if !numerical_pass {
        Direction::Inconclusive
    } else if globals[2].condition_ratio <= 1.25 {
        Direction::OrthogonalOrMultiRowCompletion
    } else if globals[1].condition_ratio <= 1.25 {
        Direction::ReplacementCompletion
    } else {
        Direction::CompleteHighEdgeGeometry
    };
    let mut review = Review {
        residues,
        globals,
        modes,
        maximum_errors,
        hashes: [filter_hash, 0],
        direction,
    };
    review.hashes[1] = evidence_hash(&review);
    review
}

fn completion_vector(
    filters: &[Complex64],
    bins: &[usize],
    positive_bins: usize,
) -> Vec<Complex64> {
    bins.iter()
        .map(|bin| filters[COMPLETION_CHANNEL * positive_bins + *bin])
        .collect()
}

fn subtract_completion(
    full: &[Complex64],
    completion: &[Complex64],
    retain_diagonal: bool,
) -> Vec<Complex64> {
    let size = completion.len();
    let mut matrix = full.to_vec();
    for row in 0..size {
        for column in 0..size {
            if !retain_diagonal || row != column {
                matrix[row * size + column] -= completion[row] * completion[column].conj();
            }
        }
    }
    matrix
}

fn completion_energy(completion: &[Complex64]) -> [f64; 2] {
    let diagonal = completion.iter().map(|value| value.norm_sqr()).sum();
    let off_diagonal = completion
        .iter()
        .enumerate()
        .flat_map(|(row, value)| {
            completion
                .iter()
                .enumerate()
                .filter(move |(column, _)| *column != row)
                .map(move |(_, other)| (*value * other.conj()).norm_sqr())
        })
        .sum::<f64>()
        .sqrt();
    [diagonal, off_diagonal]
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

fn mode_evidence(
    maximum: bool,
    residue: usize,
    eigenvalue: f64,
    vector: &[Complex64],
    bins: &[usize],
    filters: &[Complex64],
    positive_bins: usize,
) -> ModeEvidence {
    let full = frame_matrix(filters, bins, positive_bins);
    let completion = completion_vector(filters, bins, positive_bins);
    let removed = subtract_completion(&full, &completion, false);
    let diagonalized = subtract_completion(&full, &completion, true);
    let rayleigh = [full, removed, diagonalized]
        .each_ref()
        .map(|matrix| dot(vector, &multiply(matrix, vector, bins.len())).re);
    let diagonal = completion
        .iter()
        .zip(vector)
        .map(|(h, v)| h.norm_sqr() * v.norm_sqr())
        .sum::<f64>();
    let projection = completion
        .iter()
        .zip(vector)
        .map(|(h, v)| h.conj() * v)
        .sum::<Complex64>()
        .norm_sqr();
    let scale = eigenvalue.abs().max(f64::MIN_POSITIVE);
    ModeEvidence {
        maximum,
        residue,
        eigenvalue,
        rayleigh,
        changes: [rayleigh[1] - rayleigh[0], rayleigh[2] - rayleigh[0]],
        closure_errors: [
            (rayleigh[0] - rayleigh[1] - projection).abs() / scale,
            (rayleigh[0] - rayleigh[2] - (projection - diagonal)).abs() / scale,
        ],
        vector_hash: hash_complex(vector),
    }
}

fn record_jacobi_errors(maximum: &mut [f64; 5], solution: &JacobiSolution) {
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

fn multiply(matrix: &[Complex64], vector: &[Complex64], size: usize) -> Vec<Complex64> {
    (0..size)
        .map(|row| {
            (0..size)
                .map(|column| matrix[row * size + column] * vector[column])
                .sum()
        })
        .collect()
}

fn dot(left: &[Complex64], right: &[Complex64]) -> Complex64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left.conj() * right)
        .sum()
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
        for value in row.jacobi.proof_errors {
            hash_u64(&mut hash, value.to_bits());
        }
        for value in row.hashes {
            hash_u64(&mut hash, value);
        }
    }
    for mode in &review.modes {
        for value in mode.rayleigh {
            hash_u64(&mut hash, value.to_bits());
        }
        hash_u64(&mut hash, mode.vector_hash);
    }
    hash
}
