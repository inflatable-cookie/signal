use rustfft::num_complex::Complex64;

use super::common_grid::{
    build_boundary_candidate_filters, filter_hash, hash_u64, CHANNELS, HASH_OFFSET, HOP,
};
use super::conditioning_attribution::{frame_matrix, residue_bins, FFT_FRAMES};
use super::hermitian_jacobi::jacobi_solution;
use super::types::{
    StretchCommonGridJacobiEvidence as JacobiEvidence,
    StretchCommonGridThreeRowNyquistDirection as Direction,
    StretchCommonGridThreeRowNyquistResidueEvidence as ResidueEvidence,
    StretchCommonGridThreeRowNyquistReview as Review,
};

const COMPLETION_DELAYS: [i32; 3] = [-128, 0, 128];
const CANDIDATE_ROWS: usize = CHANNELS + 2;

pub(crate) fn common_grid_three_row_nyquist_review() -> Review {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let raw = build_boundary_candidate_filters(FFT_FRAMES);
    let preserved_end = (CHANNELS - 1) * positive_bins;
    let preserved_hash = filter_hash(&raw[..preserved_end]);
    let original_completion = &raw[preserved_end..];
    let mut filters = Vec::with_capacity(CANDIDATE_ROWS * positive_bins);
    filters.extend_from_slice(&raw[..preserved_end]);
    let scale = 3.0_f64.sqrt().recip();
    let mut completion_hashes = [0; 3];
    for (row, delay) in COMPLETION_DELAYS.into_iter().enumerate() {
        let start = filters.len();
        for (bin, value) in original_completion.iter().enumerate() {
            let phase = -std::f64::consts::TAU * bin as f64 * delay as f64 / FFT_FRAMES as f64;
            filters.push(*value * scale * Complex64::from_polar(1.0, phase));
        }
        completion_hashes[row] = filter_hash(&filters[start..]);
    }

    let construction_errors = construction_errors(
        original_completion,
        &filters[preserved_end..],
        positive_bins,
    );
    let mut residues = Vec::with_capacity(FFT_FRAMES / HOP);
    let mut minimum = (f64::INFINITY, 0_usize);
    let mut maximum = (f64::NEG_INFINITY, 0_usize);
    let mut maximum_proof_errors = [0.0_f64; 4];
    for residue in 0..FFT_FRAMES / HOP {
        let bins = residue_bins(residue, positive_bins);
        let matrix = frame_matrix(&filters, &bins, positive_bins);
        let solution = jacobi_solution(&matrix);
        for (slot, value) in maximum_proof_errors
            .iter_mut()
            .zip(solution.evidence.proof_errors)
        {
            *slot = slot.max(value);
        }
        let eigenvalues = [
            solution.eigenvalues[0],
            solution.eigenvalues[bins.len() - 1],
        ];
        if eigenvalues[0] < minimum.0 {
            minimum = (eigenvalues[0], residue);
        }
        if eigenvalues[1] > maximum.0 {
            maximum = (eigenvalues[1], residue);
        }
        residues.push(ResidueEvidence {
            residue,
            bin_count: bins.len(),
            eigenvalues,
            condition_ratio: eigenvalues[1] / eigenvalues[0],
            jacobi: solution.evidence,
            hashes: [hash_usizes(&bins), hash_complex(&matrix)],
        });
    }
    let eigenvalues = [minimum.0, maximum.0];
    let condition_ratio = maximum.0 / minimum.0;
    let passed = filters.len() == CANDIDATE_ROWS * positive_bins
        && completion_hashes.iter().all(|hash| *hash != 0)
        && construction_errors.iter().all(|error| *error <= 1.0e-12)
        && residues.iter().all(|row| passes(&row.jacobi))
        && condition_ratio <= 1.25;
    let direction = if passed {
        Direction::IdentityReconstructionProof
    } else {
        Direction::BoundaryGeometry
    };
    let mut review = Review {
        row_count: filters.len() / positive_bins,
        hop_frames: HOP,
        completion_delays: COMPLETION_DELAYS,
        preserved_hash,
        completion_hashes,
        construction_errors,
        residues,
        eigenvalues,
        limiting_residues: [minimum.1, maximum.1],
        condition_ratio,
        maximum_proof_errors,
        evidence_hash: 0,
        direction,
    };
    review.evidence_hash = evidence_hash(&review);
    review
}

fn construction_errors(
    original: &[Complex64],
    completion_rows: &[Complex64],
    positive_bins: usize,
) -> [f64; 4] {
    let spacing = 0.5 / (CHANNELS - 1) as f64;
    let support_start = 0.5 - 16.0 * spacing;
    let mut support_error = 0.0_f64;
    let mut diagonal_error = 0.0_f64;
    let mut off_diagonal_error = 0.0_f64;
    for bin in 0..positive_bins {
        let frequency = bin as f64 / FFT_FRAMES as f64;
        let values = completion_values(completion_rows, positive_bins, bin);
        if frequency < support_start {
            support_error = support_error.max(values.iter().map(|value| value.norm()).sum());
        }
        let diagonal = values.iter().map(|value| value.norm_sqr()).sum::<f64>();
        diagonal_error = diagonal_error.max((diagonal - original[bin].norm_sqr()).abs());
    }
    for residue in 0..FFT_FRAMES / HOP {
        let bins = residue_bins(residue, positive_bins);
        for (left_index, left) in bins.iter().enumerate() {
            for right in bins.iter().skip(left_index + 1) {
                let cross = (0..3)
                    .map(|row| {
                        completion_rows[row * positive_bins + *left]
                            * completion_rows[row * positive_bins + *right].conj()
                    })
                    .sum::<Complex64>();
                off_diagonal_error = off_diagonal_error.max(cross.norm());
            }
        }
    }
    let nyquist = positive_bins - 1;
    let real_nyquist_error = completion_values(completion_rows, positive_bins, nyquist)
        .iter()
        .map(|value| value.im.abs())
        .fold(0.0, f64::max);
    [
        support_error,
        diagonal_error,
        off_diagonal_error,
        real_nyquist_error,
    ]
}

fn completion_values(
    completion_rows: &[Complex64],
    positive_bins: usize,
    bin: usize,
) -> [Complex64; 3] {
    std::array::from_fn(|row| completion_rows[row * positive_bins + bin])
}

fn passes(row: &JacobiEvidence) -> bool {
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
    hash_u64(&mut hash, review.preserved_hash);
    for value in review.completion_hashes {
        hash_u64(&mut hash, value);
    }
    for value in review.construction_errors {
        hash_u64(&mut hash, value.to_bits());
    }
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
    hash
}
