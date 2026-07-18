use super::*;

pub(super) fn deterministic_probe() -> Vec<f64> {
    (0..8_192)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE_HZ as f64;
            (std::f64::consts::TAU * 55.0 * time).sin() * 0.23
                + (std::f64::consts::TAU * 440.0 * time + 0.31).sin() * 0.19
                + (std::f64::consts::TAU * 4_000.0 * time + 0.73).sin() * 0.11
                + ((index * 73 % 509) as f64 - 254.0) / 4_096.0
        })
        .collect()
}

pub(super) fn hash_channels(channels: &[Vec<f64>; CHANNEL_CAPACITY]) -> u64 {
    let mut hash = HASH_OFFSET;
    for channel in channels {
        for sample in channel {
            hash_u64(&mut hash, sample.to_bits());
        }
    }
    hash
}

pub(super) fn review_hash(review: &StageAReview) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in review.geometry.into_iter().chain(review.capacities) {
        hash_usize(&mut hash, value);
    }
    for value in review
        .owner_counts
        .into_iter()
        .chain(review.structural_failures)
    {
        hash_usize(&mut hash, value);
    }
    for value in review
        .identity_errors
        .into_iter()
        .chain(review.mechanics_errors)
    {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in review.state_counts {
        hash_usize(&mut hash, value);
    }
    for value in [
        review.linked_regions,
        review.unlinked_regions,
        review.owner_switches,
        review.region_high_water,
        review.overflow_failures,
        review.non_finite_values,
    ] {
        hash_usize(&mut hash, value);
    }
    hash_u64(&mut hash, review.hashes[0]);
    hash_u64(&mut hash, review.hashes[1]);
    hash
}
