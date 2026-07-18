use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::guided_frequency_partitioned_linked_phase::StateCounts;

mod fixtures;
mod mechanics;
mod report;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LengthReview {
    length: usize,
    slices: usize,
    expected_updates: usize,
    updates: usize,
    active_high_water: usize,
    dual_layer_updates: usize,
    decision_updates: [usize; 5],
    boundary_decisions: [[usize; 5]; 4],
    state: StateCounts,
    region_high_water: usize,
    atom_visits: usize,
    region_visits: usize,
    maximum_errors: [f64; 6],
    failures: [usize; 5],
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct GuidedRateReview {
    sample_rate: usize,
    geometry: [usize; 5],
    lengths: [LengthReview; 3],
}

#[derive(Clone, Debug, PartialEq)]
struct GuidedStageReview {
    representation_hash: u64,
    rates: Vec<GuidedRateReview>,
    overflow_failures: usize,
    hash: u64,
}

fn hash_length(hash: &mut u64, review: LengthReview) {
    for value in [
        review.length,
        review.slices,
        review.expected_updates,
        review.updates,
        review.active_high_water,
        review.dual_layer_updates,
        review.region_high_water,
        review.atom_visits,
        review.region_visits,
    ]
    .into_iter()
    .chain(review.decision_updates)
    .chain(review.boundary_decisions.into_iter().flatten())
    .chain(review.state.states)
    .chain([
        review.state.linked_regions,
        review.state.unlinked_regions,
        review.state.owner_switches,
    ])
    .chain(review.failures)
    {
        hash_usize(hash, value);
    }
    for error in review.maximum_errors {
        hash_u64(hash, error.to_bits());
    }
    hash_u64(hash, review.hash);
}

#[cfg(test)]
mod tests {
    use super::report::guided_stage_review;

    #[test]
    fn normalized_sliced_guided_state_boundary_mechanics_pass_rule_31u() {
        let review = guided_stage_review();
        assert_eq!(review.representation_hash, 0x0407_f765_c7d8_4375);
        assert_eq!(review.rates.len(), 3, "{review:#?}");
        assert_eq!(review.overflow_failures, 0, "{review:#?}");
        for rate in &review.rates {
            for length in rate.lengths {
                assert_eq!(length.expected_updates, length.updates, "{rate:#?}");
                assert_eq!(length.active_high_water, 2, "{rate:#?}");
                assert!(length.dual_layer_updates > 0, "{rate:#?}");
                assert!(length.decision_updates.iter().all(|count| *count > 0));
                assert!(length.state.states.iter().all(|count| *count > 0));
                assert!(length.state.linked_regions > 0, "{rate:#?}");
                assert!(length.region_high_water <= rate.geometry[4], "{rate:#?}");
                assert_eq!(length.atom_visits, length.updates * rate.geometry[4]);
                assert!(length.region_visits <= length.atom_visits, "{rate:#?}");
                assert!(
                    length.maximum_errors.iter().all(|error| *error <= 1.0e-6),
                    "{rate:#?}"
                );
                assert_eq!(length.failures, [0; 5], "{rate:#?}");
                assert_ne!(length.hash, 0);
            }
            assert!(
                rate.lengths[2]
                    .boundary_decisions
                    .iter()
                    .flatten()
                    .all(|count| *count > 0),
                "{rate:#?}"
            );
        }
        assert_eq!(review.hash, 0x90c1_0cd2_e66d_4faf);
        eprintln!("normalized_sliced_guided_state {review:#?}");
    }

    #[test]
    fn normalized_sliced_guided_state_boundary_mechanics_repeat() {
        assert_eq!(guided_stage_review(), guided_stage_review());
    }
}
