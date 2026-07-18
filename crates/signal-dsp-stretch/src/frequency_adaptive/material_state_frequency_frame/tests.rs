use super::*;

#[test]
fn frequency_adaptive_material_frame_stage_a_passes_identity_and_mechanics() {
    let review = stage_a_review();
    assert_eq!(review.geometry, [16_384, 8_192, 4_096, 32, 512]);
    assert_eq!(review.support_frames, [4_096, 2_048, 1_024]);
    assert_eq!(review.crossover_hz, [750, 6_000]);
    assert!(
        review.owner_counts.iter().all(|count| *count > 0),
        "{review:?}"
    );
    assert_eq!(review.structural_failures, [0; 4], "{review:?}");
    assert!(review.frame_values[0] > 0.0, "{review:?}");
    assert!(review.frame_values[1].is_finite(), "{review:?}");
    assert!(review.frame_values[2] <= 1.0 + 1.0e-12, "{review:?}");
    assert!(review.maximum_errors[0] <= 1.0e-12, "{review:?}");
    assert!(review.maximum_errors[1] <= 1.0e-13, "{review:?}");
    assert!(review.maximum_errors[2] <= 1.0e-12, "{review:?}");
    assert!(review.maximum_errors[3] <= 1.0e-12, "{review:?}");
    assert!(review.maximum_errors[4] <= 1.0e-12, "{review:?}");
    assert!(review.maximum_errors[5] <= 1.0e-12, "{review:?}");
    assert!(
        review.relation_errors.iter().all(|error| *error <= 1.0e-12),
        "{review:?}"
    );
    assert_eq!(review.mechanics_failures, [0; 4], "{review:?}");
    assert_eq!(review.reflected_reads, 8_192);
    assert_eq!(review.non_finite_values, 0, "{review:?}");
    assert!(review.hashes.iter().all(|hash| *hash != 0), "{review:?}");
    eprintln!("frequency_adaptive_material_frame_stage_a {review:?}");
}

#[test]
fn frequency_adaptive_material_frame_stage_a_is_deterministic() {
    let first = stage_a_review();
    let repeated = stage_a_review();
    assert_eq!(first, repeated);
}
