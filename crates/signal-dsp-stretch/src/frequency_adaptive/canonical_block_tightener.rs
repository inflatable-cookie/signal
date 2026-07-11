use rustfft::num_complex::Complex64;

use super::common_grid::{filter_hash, hash_u64, HASH_OFFSET, HOP};
use super::conditioning_attribution::{frame_matrix, residue_bins, FFT_FRAMES};
use super::hermitian_jacobi::jacobi_solution;
use super::three_row_nyquist::three_row_candidate_filters;
use super::types::{
    StretchCommonGridCanonicalTightenerDirection as Direction,
    StretchCommonGridCanonicalTightenerReview as Review,
};

const ROWS: usize = 1_538;

pub(crate) fn common_grid_canonical_tightener_review() -> Review {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let filters = three_row_candidate_filters().0;
    let input_hash = filter_hash(&filters);
    let mut tighteners = Vec::with_capacity(FFT_FRAMES / HOP);
    let mut tightener_hash = HASH_OFFSET;
    let mut extrema = [f64::INFINITY, f64::NEG_INFINITY];
    let mut maximum_proof_errors = [0.0_f64; 5];

    for residue in 0..FFT_FRAMES / HOP {
        let bins = residue_bins(residue, positive_bins);
        let frame = frame_matrix(&filters, &bins, positive_bins);
        let solution = jacobi_solution(&frame);
        if solution
            .eigenvalues
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return rejected(input_hash);
        }
        let tightener = inverse_sqrt(&solution.eigenvalues, &solution.eigenvectors, bins.len());
        let transformed = multiply(
            &multiply(&tightener, &frame, bins.len()),
            &tightener,
            bins.len(),
        );
        let transformed_solution = jacobi_solution(&transformed);
        extrema[0] = extrema[0].min(transformed_solution.eigenvalues[0]);
        extrema[1] = extrema[1].max(transformed_solution.eigenvalues[bins.len() - 1]);
        for (slot, value) in maximum_proof_errors
            .iter_mut()
            .zip(transformed_solution.evidence.proof_errors)
        {
            *slot = slot.max(value);
        }
        maximum_proof_errors[4] =
            maximum_proof_errors[4].max(identity_error(&transformed, bins.len()));
        for value in &tightener {
            hash_u64(&mut tightener_hash, value.re.to_bits());
            hash_u64(&mut tightener_hash, value.im.to_bits());
        }
        tighteners.push(tightener);
    }

    let condition = extrema[1] / extrema[0];
    let numerical_pass = condition <= 1.0 + 1.0e-10
        && maximum_proof_errors[0] <= 1.0e-8
        && maximum_proof_errors[1] <= 1.0e-10
        && maximum_proof_errors[2] <= 1.0e-12
        && maximum_proof_errors[3] <= 1.0e-10
        && maximum_proof_errors[4] <= 1.0e-10;
    if !numerical_pass {
        return Review {
            evaluated_rows: 0,
            first_violating_row: usize::MAX,
            frame_values: [extrema[0], extrema[1], condition],
            maximum_proof_errors,
            localization_errors: [f64::INFINITY; 3],
            limiting_support_bins: [0; 2],
            hashes: [input_hash, tightener_hash, 0, 0],
            direction: Direction::Inconclusive,
        };
    }

    let mut row_hash = HASH_OFFSET;
    let mut localization_errors = [0.0_f64; 3];
    let mut limiting_support_bins = [0; 2];
    let mut evaluated_rows = 0;
    let mut first_violating_row = usize::MAX;
    for row in 0..ROWS {
        let transformed = transform_row(&filters, row, positive_bins, &tighteners);
        let original = &filters[row * positive_bins..(row + 1) * positive_bins];
        let total = transformed
            .iter()
            .map(|value| value.norm_sqr())
            .sum::<f64>();
        let mut leaked = 0.0;
        let mut peak = 0.0_f64;
        let mut original_support = 0;
        let mut transformed_support = 0;
        for (before, after) in original.iter().zip(&transformed) {
            if before.norm_sqr() == 0.0 {
                leaked += after.norm_sqr();
                peak = peak.max(after.norm());
            } else {
                original_support += 1;
            }
            transformed_support += usize::from(after.norm_sqr() > 0.0);
            hash_u64(&mut row_hash, after.re.to_bits());
            hash_u64(&mut row_hash, after.im.to_bits());
        }
        let leakage = leaked / total.max(f64::MIN_POSITIVE);
        let endpoint = transformed[0]
            .im
            .abs()
            .max(transformed[positive_bins - 1].im.abs());
        localization_errors = [
            localization_errors[0].max(leakage),
            localization_errors[1].max(peak),
            localization_errors[2].max(endpoint),
        ];
        limiting_support_bins = [original_support, transformed_support];
        evaluated_rows += 1;
        if leakage > 1.0e-12 || peak > 1.0e-12 || endpoint > 1.0e-12 {
            first_violating_row = row;
            break;
        }
    }
    let direction = if first_violating_row == usize::MAX && evaluated_rows == ROWS {
        Direction::LargeProbeLocalization
    } else {
        Direction::TransformFamilyReassessment
    };
    let mut review = Review {
        evaluated_rows,
        first_violating_row,
        frame_values: [extrema[0], extrema[1], condition],
        maximum_proof_errors,
        localization_errors,
        limiting_support_bins,
        hashes: [input_hash, tightener_hash, row_hash, 0],
        direction,
    };
    review.hashes[3] = evidence_hash(&review);
    review
}

fn inverse_sqrt(values: &[f64], vectors: &[Complex64], size: usize) -> Vec<Complex64> {
    let mut result = vec![Complex64::new(0.0, 0.0); size * size];
    for row in 0..size {
        for column in 0..size {
            result[row * size + column] = (0..size)
                .map(|mode| {
                    vectors[row * size + mode]
                        * values[mode].sqrt().recip()
                        * vectors[column * size + mode].conj()
                })
                .sum();
        }
    }
    result
}

fn transform_row(
    filters: &[Complex64],
    row: usize,
    positive_bins: usize,
    tighteners: &[Vec<Complex64>],
) -> Vec<Complex64> {
    let mut output = vec![Complex64::new(0.0, 0.0); positive_bins];
    for (residue, tightener) in tighteners.iter().enumerate() {
        let bins = residue_bins(residue, positive_bins);
        for (target, target_bin) in bins.iter().enumerate() {
            output[*target_bin] = bins
                .iter()
                .enumerate()
                .map(|(source, source_bin)| {
                    tightener[target * bins.len() + source]
                        * filters[row * positive_bins + *source_bin]
                })
                .sum();
        }
    }
    output
}

fn multiply(left: &[Complex64], right: &[Complex64], size: usize) -> Vec<Complex64> {
    let mut result = vec![Complex64::new(0.0, 0.0); size * size];
    for row in 0..size {
        for column in 0..size {
            result[row * size + column] = (0..size)
                .map(|inner| left[row * size + inner] * right[inner * size + column])
                .sum();
        }
    }
    result
}

fn identity_error(matrix: &[Complex64], size: usize) -> f64 {
    (0..size)
        .flat_map(|row| (0..size).map(move |column| (row, column)))
        .map(|(row, column)| {
            (matrix[row * size + column] - Complex64::new((row == column) as u8 as f64, 0.0)).norm()
        })
        .fold(0.0, f64::max)
}

fn evidence_hash(review: &Review) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in review
        .frame_values
        .into_iter()
        .chain(review.maximum_proof_errors)
    {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.localization_errors {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in &review.hashes[..3] {
        hash_u64(&mut hash, *value);
    }
    hash
}

fn rejected(input_hash: u64) -> Review {
    Review {
        evaluated_rows: 0,
        first_violating_row: usize::MAX,
        frame_values: [f64::NAN; 3],
        maximum_proof_errors: [f64::INFINITY; 5],
        localization_errors: [f64::INFINITY; 3],
        limiting_support_bins: [0; 2],
        hashes: [input_hash, 0, 0, 0],
        direction: Direction::Inconclusive,
    }
}
