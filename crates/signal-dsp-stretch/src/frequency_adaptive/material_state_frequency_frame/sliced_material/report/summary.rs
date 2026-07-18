use std::{fs, path::PathBuf};

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn write_summary(
    synthetic: &SyntheticReview,
    mechanics: &SharedRotationMechanicsReview,
    relation_mechanics: RelationCounts,
    relation_mechanics_error: f64,
    stereo: &PeakRegionReview,
    corpus: Option<&SharedRotationCorpusReview>,
    direction: SlicedMaterialDirection,
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-relation-owned-sliced-material");
    fs::create_dir_all(&root).expect("create sliced material report directory");
    let (corpus_ran, corpus_failures, corpus_regressions, corpus_hash) = match corpus {
        Some(review) => (
            review.repeated.to_string(),
            review.candidate_hard_failures.to_string(),
            review.row_complete_regressions.to_string(),
            format!("{:016x}", review.hash),
        ),
        None => (
            "false".into(),
            "not-run".into(),
            "not-run".into(),
            "not-run".into(),
        ),
    };
    let report = format!(
        "direction\t{direction:?}\nsynthetic_failures\t{},{}\nsynthetic_relations\t{:?}\nsynthetic_relation_error\t{:e}\nmaximum_live_slices\t{},{}\nsynthetic_hash\t{:016x}\nmechanics_failures\t{},{}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nmechanics_hash\t{:016x}\nrelation_mechanics\t{:?}\nrelation_mechanics_error\t{:e}\nstereo_failures\t{}\nstereo_local_failures\t{}\nstereo_relations\t{:?}\nstereo_relation_error\t{:e}\nstereo_hash\t{:016x}\nmono_ran\t{}\nmono_hard_failures\t{}\nmono_row_complete_regressions\t{}\nmono_hash\t{}\n",
        synthetic.structural_failures, synthetic.hidden_gain_failures,
        synthetic.relations.as_array(), synthetic.maximum_relation_error,
        synthetic.maximum_live_source_slices, synthetic.maximum_live_output_slices,
        synthetic.hash, mechanics.structural_failures, mechanics.identity_mismatches,
        mechanics.errors[0], mechanics.errors[1], mechanics.errors[2], mechanics.errors[3],
        mechanics.errors[4], mechanics.hash, relation_mechanics.as_array(),
        relation_mechanics_error, stereo.candidate_failures,
        stereo.local_consistency_failures, stereo.relation_states,
        stereo.maximum_relation_error, stereo.evidence_hash, corpus_ran,
        corpus_failures, corpus_regressions, corpus_hash,
    );
    fs::write(root.join("proof-summary.tsv"), report).expect("write sliced material summary");
}
