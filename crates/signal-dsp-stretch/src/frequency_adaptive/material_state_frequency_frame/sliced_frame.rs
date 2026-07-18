use super::*;

mod render;
mod report;

const OUTER_ADVANCE: usize = FFT_FRAMES / 2;
const IDENTITY_LENGTHS: [usize; 5] = [1, 4_095, 8_192, 12_289, 220_500];
const BOUNDED_LENGTHS: [usize; 3] = [8_192, 65_536, 220_500];

#[derive(Clone, Debug, PartialEq)]
struct SlicedStageAReview {
    geometry: [usize; 4],
    support_frames: [usize; 3],
    crossover_hz: [usize; 2],
    owner_counts: [usize; 3],
    structural_failures: [usize; 4],
    identity_lengths: [usize; 5],
    identity_slice_counts: [usize; 5],
    maximum_errors: [f64; 7],
    relation_errors: [f64; 4],
    mechanics_failures: [usize; 6],
    boundedness: [[usize; 4]; 3],
    per_slice_operations: usize,
    non_finite_values: usize,
    hashes: [u64; 4],
}

fn sliced_review_hash(review: &SlicedStageAReview) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in review
        .geometry
        .into_iter()
        .chain(review.support_frames)
        .chain(review.crossover_hz)
        .chain(review.owner_counts)
        .chain(review.structural_failures)
        .chain(review.identity_lengths)
        .chain(review.identity_slice_counts)
    {
        hash_usize(&mut hash, value);
    }
    for value in review.maximum_errors {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.relation_errors {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.mechanics_failures {
        hash_usize(&mut hash, value);
    }
    for row in review.boundedness {
        for value in row {
            hash_usize(&mut hash, value);
        }
    }
    hash_usize(&mut hash, review.per_slice_operations);
    hash_usize(&mut hash, review.non_finite_values);
    for value in &review.hashes[..3] {
        hash_u64(&mut hash, *value);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::report::stage_a_review;
    use super::*;

    #[test]
    fn frequency_adaptive_sliced_frame_stage_a_passes_frozen_gates() {
        let review = stage_a_review();
        assert_eq!(review.geometry, [16_384, 8_192, 512, 32]);
        assert_eq!(review.support_frames, [4_096, 2_048, 1_024]);
        assert_eq!(review.crossover_hz, [750, 6_000]);
        assert!(review.owner_counts.iter().all(|count| *count > 0));
        assert_eq!(review.structural_failures, [0; 4], "{review:?}");
        assert_eq!(review.identity_lengths, IDENTITY_LENGTHS);
        assert_eq!(review.identity_slice_counts, [2, 2, 2, 3, 28]);
        assert!(
            review.maximum_errors.iter().all(|error| *error <= 1.0e-12),
            "{review:?}"
        );
        assert!(
            review.relation_errors.iter().all(|error| *error <= 1.0e-12),
            "{review:?}"
        );
        assert_eq!(review.mechanics_failures, [0; 6], "{review:?}");
        assert_eq!(review.non_finite_values, 0, "{review:?}");
        let peak_coefficients = review.boundedness[0][2];
        for (row, length) in review.boundedness.iter().zip(BOUNDED_LENGTHS) {
            assert_eq!(row[0], render::required_slice_count(length));
            assert!(row[1] <= 2, "{review:?}");
            assert_eq!(row[2], peak_coefficients, "{review:?}");
            assert_eq!(row[3], row[0] * review.per_slice_operations, "{review:?}");
        }
        assert!(review.hashes.iter().all(|hash| *hash != 0), "{review:?}");
        eprintln!("frequency_adaptive_sliced_frame_stage_a {review:?}");
    }
}
