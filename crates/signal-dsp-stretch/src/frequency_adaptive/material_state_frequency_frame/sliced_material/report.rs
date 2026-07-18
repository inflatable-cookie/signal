use super::*;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::quality::gate_calibration::{
    peak_region_feasibility::{self, PeakRegionReview},
    shared_rotation_region_locked_proof::{
        corpus, mechanics, SharedRotationCorpusReview, SharedRotationMechanicsReview,
    },
};

mod summary;
use summary::write_summary;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlicedMaterialDirection {
    ListeningCheckpoint,
    Closed,
}

#[derive(Clone, Debug, PartialEq)]
struct SyntheticReview {
    states: StateCounts,
    relations: RelationCounts,
    structural_failures: usize,
    hidden_gain_failures: usize,
    maximum_relation_error: f64,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
    repeated: bool,
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct SlicedMaterialReview {
    synthetic: SyntheticReview,
    mechanics: SharedRotationMechanicsReview,
    relation_mechanics: RelationCounts,
    relation_mechanics_error: f64,
    stereo: PeakRegionReview,
    corpus: Option<SharedRotationCorpusReview>,
    direction: SlicedMaterialDirection,
}

fn review() -> SlicedMaterialReview {
    let synthetic = synthetic_review();
    let mechanics = mechanics::review(render);
    let (relation_mechanics, relation_mechanics_error) = relation::mechanics_review();
    let stereo = peak_region_feasibility::review_candidate(
        "stretch-relation-owned-sliced-material",
        stereo_adapter,
    );
    let mechanics_pass = mechanics.repeated
        && mechanics.structural_failures == 0
        && mechanics.identity_mismatches == 0
        && mechanics.errors.iter().all(|error| *error <= 1.0e-12)
        && mechanics.silent_peer_peak == 0.0
        && mechanics.states.tracked > 0
        && mechanics.states.reset > 0
        && mechanics.states.silent > 0
        && mechanics.states.shoulder > 0
        && mechanics.states.locked > 0
        && mechanics.states.diffuse > 0;
    let relation_pass = relation_mechanics.as_array().iter().all(|count| *count > 0)
        && relation_mechanics_error <= 1.0e-12
        && stereo.relation_states[2] == 0
        && stereo.maximum_relation_error <= 1.0e-12;
    let premono_pass = synthetic.repeated
        && synthetic.structural_failures == 0
        && synthetic.hidden_gain_failures == 0
        && synthetic.maximum_relation_error <= 1.0e-12
        && synthetic.maximum_live_output_slices <= 2
        && mechanics_pass
        && relation_pass
        && stereo.repeated
        && stereo.candidate_failures == 0
        && stereo.local_consistency_failures == 0;
    let corpus = premono_pass.then(|| corpus::review(render, "relation-owned-sliced-material"));
    let passed = corpus.as_ref().is_some_and(|corpus| {
        corpus.repeated
            && corpus.candidate_hard_failures == 0
            && corpus.row_complete_regressions == 0
    });
    let direction = if passed {
        SlicedMaterialDirection::ListeningCheckpoint
    } else {
        SlicedMaterialDirection::Closed
    };
    write_summary(
        &synthetic,
        &mechanics,
        relation_mechanics,
        relation_mechanics_error,
        &stereo,
        corpus.as_ref(),
        direction,
    );
    SlicedMaterialReview {
        synthetic,
        mechanics,
        relation_mechanics,
        relation_mechanics_error,
        stereo,
        corpus,
        direction,
    }
}

fn synthetic_review() -> SyntheticReview {
    let run = || {
        let sample_rate = 48_000;
        let frames = 16_384;
        let silence = vec![0.0; frames];
        let tone = (0..frames)
            .map(|index| {
                (std::f64::consts::TAU * 440.0 * index as f64 / sample_rate as f64).sin() * 0.25
            })
            .collect::<Vec<_>>();
        let noise = (0..frames)
            .map(|index| deterministic_noise(index) * 0.1)
            .collect::<Vec<_>>();
        let mut impulse = vec![0.0; frames];
        impulse[frames / 2] = 1.0;
        let mixed = sum(&tone, &noise);
        let transient = sum(&tone, &impulse);
        let mut states = StateCounts::default();
        let mut relations = RelationCounts::default();
        let mut structural_failures = 0;
        let mut hidden_gain_failures = 0;
        let mut maximum_relation_error = 0.0_f64;
        let mut maximum_live_source_slices = 0;
        let mut maximum_live_output_slices = 0;
        let mut hash = HASH_OFFSET;
        for source in [&silence, &tone, &noise, &impulse, &mixed, &transient] {
            for ratio in RATIOS {
                let result = render::render_detailed([source, source], ratio, sample_rate);
                structural_failures += result.render.uncovered
                    + result.render.non_finite
                    + result.render.boundary_failures;
                hidden_gain_failures += usize::from(
                    source.iter().any(|sample| *sample != 0.0)
                        && peak(&result.render.channels[0]) > 4.0,
                );
                add_states(&mut states, result.render.states);
                relations.add(result.relations);
                maximum_relation_error = maximum_relation_error.max(result.maximum_relation_error);
                maximum_live_source_slices =
                    maximum_live_source_slices.max(result.maximum_live_source_slices);
                maximum_live_output_slices =
                    maximum_live_output_slices.max(result.maximum_live_output_slices);
                hash_u64(&mut hash, result.render.hash);
            }
        }
        (
            states,
            relations,
            structural_failures,
            hidden_gain_failures,
            maximum_relation_error,
            maximum_live_source_slices,
            maximum_live_output_slices,
            hash,
        )
    };
    let first = run();
    let second = run();
    SyntheticReview {
        states: first.0,
        relations: first.1,
        structural_failures: first.2,
        hidden_gain_failures: first.3,
        maximum_relation_error: first.4,
        maximum_live_source_slices: first.5,
        maximum_live_output_slices: first.6,
        repeated: first == second,
        hash: first.7,
    }
}

fn deterministic_noise(index: usize) -> f64 {
    let mut value = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    (value as f64 / u64::MAX as f64) * 2.0 - 1.0
}

fn sum(left: &[f64], right: &[f64]) -> Vec<f64> {
    left.iter()
        .zip(right)
        .map(|(left, right)| left + right)
        .collect()
}

fn peak(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}

fn add_states(target: &mut StateCounts, source: StateCounts) {
    target.tracked += source.tracked;
    target.reset += source.reset;
    target.silent += source.silent;
    target.regions += source.regions;
    target.owner_switches += source.owner_switches;
    target.shoulder += source.shoulder;
    target.locked += source.locked;
    target.diffuse += source.diffuse;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires frozen exact-source and Rubber Band corpus pack"]
    fn relation_owned_sliced_material_stage_b_objective_gate() {
        let result = review();
        eprintln!("relation_owned_sliced_material_stage_b {result:#?}");
        assert_eq!(result.synthetic.structural_failures, 0);
        assert_eq!(result.synthetic.hidden_gain_failures, 0);
        assert!(result.synthetic.repeated);
        assert!(result.synthetic.maximum_live_output_slices <= 2);
        assert!(result
            .relation_mechanics
            .as_array()
            .iter()
            .all(|count| *count > 0));
        assert!(result.relation_mechanics_error <= 1.0e-12);
        assert_eq!(result.mechanics.structural_failures, 0);
        assert_eq!(result.mechanics.identity_mismatches, 0);
        assert!(result
            .mechanics
            .errors
            .iter()
            .all(|error| *error <= 1.0e-12));
        assert_eq!(result.stereo.candidate_failures, 0);
        assert_eq!(result.stereo.local_consistency_failures, 0);
        assert_eq!(result.stereo.relation_states[2], 0);
        assert!(result.stereo.maximum_relation_error <= 1.0e-12);
        assert!(result.corpus.is_some());
        assert_eq!(
            result.direction,
            SlicedMaterialDirection::ListeningCheckpoint
        );
    }
}
