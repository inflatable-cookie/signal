use super::*;

#[test]
fn guided_frequency_partitioned_linked_phase_stage_a_passes() {
    let review = stage_a_review();
    assert_eq!(review.geometry, [16_384, 8_192, 512, 4_096, 2_048, 1_024]);
    assert_eq!(review.capacities, [2, 1_344, 673, 32, 673]);
    assert!(
        review.owner_counts.iter().all(|count| *count > 0),
        "{review:#?}"
    );
    assert_eq!(review.structural_failures, [0; 5], "{review:#?}");
    assert!(
        review.identity_errors.iter().all(|error| *error <= 1.0e-12),
        "{review:#?}"
    );
    assert!(
        review.mechanics_errors.iter().all(|error| *error <= 1.0e-6),
        "{review:#?}"
    );
    assert!(
        review.state_counts.iter().all(|count| *count > 0),
        "{review:#?}"
    );
    assert!(review.linked_regions > 0, "{review:#?}");
    assert!(review.unlinked_regions > 0, "{review:#?}");
    assert!(review.region_high_water <= REGION_CAPACITY, "{review:#?}");
    assert_eq!(review.overflow_failures, 0, "{review:#?}");
    assert_eq!(review.non_finite_values, 0, "{review:#?}");
    assert!(review.hashes.iter().all(|hash| *hash != 0), "{review:#?}");
    eprintln!("guided_frequency_partitioned_linked_phase_stage_a {review:#?}");
}

#[test]
fn guided_frequency_partitioned_linked_phase_stage_a_repeats() {
    assert_eq!(stage_a_review(), stage_a_review());
}

#[test]
fn guided_frequency_partitioned_linked_phase_stage_b_gate_exceeds_frozen_capacity() {
    let representation = super::super::build_representation_for(16_384, 8_000, COMMON_HOP);
    let positive = representation
        .bands
        .iter()
        .filter(|band| band.center <= representation.fft_frames / 2)
        .count();
    eprintln!(
        "guided_frequency_partitioned_linked_phase_stage_b_capacity signed={} positive={} coefficients={}",
        representation.bands.len(),
        positive,
        representation.common_coefficients
    );
    assert!(representation.bands.len() > SIGNED_ATOM_CAPACITY);
    assert!(positive > POSITIVE_ATOM_CAPACITY);
}
