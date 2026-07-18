use super::{hash_u64, hash_usize, paired_max_error, HASH_OFFSET};

mod geometry;
mod render;
mod report;

use geometry::{prepare, validate_capacity, CapacityRequest, Geometry, PrepareError};
use render::{boundary_token_review, required_slice_count, Renderer};

const PROOF_RATES: [usize; 3] = [8_000, 44_100, 48_000];
const CHANNEL_CAPACITY: usize = 2;
const SIGNED_ATOM_CAPACITY: usize = 1_260;
const POSITIVE_ATOM_CAPACITY: usize = 631;
const COEFFICIENT_CAPACITY: usize = 32;
const REGION_CAPACITY: usize = 631;
const SOURCE_SLICE_CAPACITY: usize = 6;
const OUTPUT_SLICE_CAPACITY: usize = 2;
const MATERIAL_HALO_FRAMES: usize = 19;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MemoryCounts {
    coefficient_complex: usize,
    transform_complex: usize,
    outer_samples: usize,
    guidance_values: usize,
    phase_values: usize,
    region_records: usize,
    static_values: usize,
    tap_records: usize,
    band_records: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WorkCounts {
    full_transforms: usize,
    band_transforms: usize,
    tap_visits: usize,
    coefficient_visits: usize,
    sample_visits: usize,
    conjugate_visits: usize,
}

impl WorkCounts {
    fn scaled(self, factor: usize) -> Self {
        Self {
            full_transforms: self.full_transforms * factor,
            band_transforms: self.band_transforms * factor,
            tap_visits: self.tap_visits * factor,
            coefficient_visits: self.coefficient_visits * factor,
            sample_visits: self.sample_visits * factor,
            conjugate_visits: self.conjugate_visits * factor,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TokenReview {
    expected_updates: usize,
    updates: usize,
    final_value: usize,
    duplicate_updates: usize,
    reset_failures: usize,
    capacity_failures: usize,
    slice_creations: usize,
    slice_retirements: usize,
    active_high_water: usize,
    boundary_crossings: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct RateReview {
    sample_rate: usize,
    geometry: [usize; 4],
    supports: [usize; 3],
    atom_counts: [usize; 3],
    owner_counts: [usize; 3],
    structural_failures: [usize; 8],
    maximum_errors: [f64; 7],
    mechanics_errors: [f64; 4],
    mechanics_failures: [usize; 5],
    bounded_lengths: [usize; 3],
    bounded_slices: [usize; 3],
    bounded_tokens: [TokenReview; 3],
    memory: MemoryCounts,
    per_slice_work: WorkCounts,
    total_work: [WorkCounts; 3],
    non_finite_values: usize,
    hashes: [u64; 3],
}

#[derive(Clone, Debug, PartialEq)]
struct StageAReview {
    rates: Vec<RateReview>,
    overflow_failures: usize,
    hash: u64,
}

fn review_hash(review: &StageAReview) -> u64 {
    let mut hash = HASH_OFFSET;
    for rate in &review.rates {
        for value in rate
            .geometry
            .into_iter()
            .chain(rate.supports)
            .chain(rate.atom_counts)
            .chain(rate.owner_counts)
            .chain(rate.structural_failures)
            .chain(rate.mechanics_failures)
            .chain(rate.bounded_lengths)
            .chain(rate.bounded_slices)
        {
            hash_usize(&mut hash, value);
        }
        for value in rate.maximum_errors.into_iter().chain(rate.mechanics_errors) {
            hash_u64(&mut hash, value.to_bits());
        }
        hash_memory(&mut hash, rate.memory);
        hash_work(&mut hash, rate.per_slice_work);
        for token in rate.bounded_tokens {
            for value in [
                token.expected_updates,
                token.updates,
                token.final_value,
                token.duplicate_updates,
                token.reset_failures,
                token.capacity_failures,
                token.slice_creations,
                token.slice_retirements,
                token.active_high_water,
                token.boundary_crossings,
            ] {
                hash_usize(&mut hash, value);
            }
        }
        for work in rate.total_work {
            hash_work(&mut hash, work);
        }
        hash_usize(&mut hash, rate.non_finite_values);
        for value in rate.hashes {
            hash_u64(&mut hash, value);
        }
    }
    hash_usize(&mut hash, review.overflow_failures);
    hash
}

fn hash_memory(hash: &mut u64, memory: MemoryCounts) {
    for value in [
        memory.coefficient_complex,
        memory.transform_complex,
        memory.outer_samples,
        memory.guidance_values,
        memory.phase_values,
        memory.region_records,
        memory.static_values,
        memory.tap_records,
        memory.band_records,
    ] {
        hash_usize(hash, value);
    }
}

fn hash_work(hash: &mut u64, work: WorkCounts) {
    for value in [
        work.full_transforms,
        work.band_transforms,
        work.tap_visits,
        work.coefficient_visits,
        work.sample_visits,
        work.conjugate_visits,
    ] {
        hash_usize(hash, value);
    }
}

#[cfg(test)]
mod tests {
    use super::report::stage_a_review;

    #[test]
    fn normalized_sliced_frame_stage_a_passes_rule_31t() {
        let review = stage_a_review();
        assert_eq!(review.rates.len(), 3, "{review:#?}");
        assert_eq!(review.overflow_failures, 0, "{review:#?}");
        for rate in &review.rates {
            assert_eq!(rate.structural_failures, [0; 8], "{rate:#?}");
            assert!(
                rate.maximum_errors.iter().all(|error| *error <= 1.0e-12),
                "{rate:#?}"
            );
            assert!(
                rate.mechanics_errors.iter().all(|error| *error <= 1.0e-12),
                "{rate:#?}"
            );
            assert_eq!(rate.mechanics_failures, [0; 5], "{rate:#?}");
            assert_eq!(rate.non_finite_values, 0, "{rate:#?}");
            for token in rate.bounded_tokens {
                assert_eq!(token.expected_updates, token.updates, "{rate:#?}");
                assert_eq!(token.final_value, token.updates, "{rate:#?}");
                assert_eq!(token.duplicate_updates, 0, "{rate:#?}");
                assert_eq!(token.reset_failures, 0, "{rate:#?}");
                assert_eq!(token.capacity_failures, 0, "{rate:#?}");
                assert_eq!(token.active_high_water, 2, "{rate:#?}");
                assert!(token.boundary_crossings > 0, "{rate:#?}");
            }
        }
        assert_eq!(review.rates[2].memory.coefficient_complex, 645_120);
        assert_eq!(review.hash, 0x0407_f765_c7d8_4375);
        eprintln!("normalized_sliced_frame_stage_a {review:#?}");
    }

    #[test]
    fn normalized_sliced_frame_stage_a_repeats() {
        assert_eq!(stage_a_review(), stage_a_review());
    }
}
