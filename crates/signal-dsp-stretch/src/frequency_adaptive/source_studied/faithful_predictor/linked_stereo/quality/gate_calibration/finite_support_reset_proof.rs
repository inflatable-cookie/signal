use std::{fs, path::PathBuf};

use super::{
    peak_region_feasibility::{self, PeakRegionReview, PeakRegionRow},
    shared_rotation_region_locked_proof::{
        corpus, mechanics, SharedRotationCorpusReview, SharedRotationMechanicsReview,
    },
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    shared_rotation_finite_support_reset, shared_rotation_region_locked,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum FiniteSupportResetDirection {
    ListeningCheckpoint,
    FamilyReview,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct FiniteSupportResetProofReview {
    pub(in crate::frequency_adaptive) frozen: PeakRegionReview,
    pub(in crate::frequency_adaptive) candidate: PeakRegionReview,
    pub(in crate::frequency_adaptive) mechanics: SharedRotationMechanicsReview,
    pub(in crate::frequency_adaptive) corpus: SharedRotationCorpusReview,
    pub(in crate::frequency_adaptive) previously_passing_local_regressions: usize,
    pub(in crate::frequency_adaptive) direction: FiniteSupportResetDirection,
}

pub(in crate::frequency_adaptive) fn review() -> FiniteSupportResetProofReview {
    let frozen = peak_region_feasibility::review_candidate(
        "stretch-shared-rotation-region-locked-control",
        shared_rotation_region_locked::stereo_adapter,
    );
    let candidate = peak_region_feasibility::review_candidate(
        "stretch-shared-rotation-finite-support-reset",
        shared_rotation_finite_support_reset::stereo_adapter,
    );
    let mechanics = mechanics::review(shared_rotation_finite_support_reset::render);
    let corpus = corpus::review(
        shared_rotation_finite_support_reset::render,
        "shared-rotation-finite-support-reset",
    );
    let previously_passing_local_regressions = frozen
        .rows
        .iter()
        .zip(&candidate.rows)
        .filter(|(frozen, candidate)| !local_failure(frozen) && local_failure(candidate))
        .count();
    let mechanics_pass = mechanics.repeated
        && mechanics.structural_failures == 0
        && mechanics.identity_mismatches == 0
        && mechanics.errors.iter().all(|error| *error <= 1.0e-12)
        && mechanics.silent_peer_peak == 0.0
        && mechanics.states.tracked > 0
        && mechanics.states.reset > 0
        && mechanics.states.silent > 0
        && mechanics.states.owner_switches > 0
        && mechanics.trajectory_break_resets > 0;
    let passed = frozen.repeated
        && frozen.evidence_hash == 0xeff5_2feb_ad8c_0fb8
        && candidate.repeated
        && candidate.candidate_failures == 0
        && candidate.local_consistency_failures == 0
        && candidate
            .mechanics_errors
            .iter()
            .all(|error| *error <= 1.0e-12)
        && candidate.silent_peer_peak == 0.0
        && previously_passing_local_regressions == 0
        && mechanics_pass
        && corpus.repeated
        && corpus.candidate_hard_failures == 0
        && corpus.row_complete_regressions == 0;
    let direction = if passed {
        FiniteSupportResetDirection::ListeningCheckpoint
    } else {
        FiniteSupportResetDirection::FamilyReview
    };
    write_summary(
        &frozen,
        &candidate,
        &mechanics,
        &corpus,
        previously_passing_local_regressions,
        direction,
    );
    FiniteSupportResetProofReview {
        frozen,
        candidate,
        mechanics,
        corpus,
        previously_passing_local_regressions,
        direction,
    }
}

fn local_failure(row: &PeakRegionRow) -> bool {
    row.maximum_local_residuals[1] > row.maximum_local_residuals[0] + 1.0e-12
        || row.local_windows_improved < 4
}

fn write_summary(
    frozen: &PeakRegionReview,
    candidate: &PeakRegionReview,
    mechanics: &SharedRotationMechanicsReview,
    corpus: &SharedRotationCorpusReview,
    previously_passing_local_regressions: usize,
    direction: FiniteSupportResetDirection,
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-shared-rotation-finite-support-reset");
    let report = format!(
        "direction\t{direction:?}\nfrozen_failures\t{}\nfrozen_local_failures\t{}\nfrozen_hash\t{:016x}\ncandidate_failures\t{}\ncandidate_local_failures\t{}\ncandidate_metric_regressions\t{}\ncandidate_mono_parity_errors\t{:e},{:e},{:e},{:e},{:e}\ncandidate_hash\t{:016x}\npreviously_passing_local_regressions\t{previously_passing_local_regressions}\nmechanics_structural_failures\t{}\nmechanics_identity_mismatches\t{}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nmechanics_states\t{},{},{},{},{}\nmechanics_hash\t{:016x}\ncorpus_candidate_hard_failures\t{}\ncorpus_row_complete_regressions\t{}\ncorpus_hash\t{:016x}\n",
        frozen.candidate_failures,
        frozen.local_consistency_failures,
        frozen.evidence_hash,
        candidate.candidate_failures,
        candidate.local_consistency_failures,
        candidate.metric_regressions,
        candidate.mechanics_errors[0],
        candidate.mechanics_errors[1],
        candidate.mechanics_errors[2],
        candidate.mechanics_errors[3],
        candidate.mechanics_errors[4],
        candidate.evidence_hash,
        mechanics.structural_failures,
        mechanics.identity_mismatches,
        mechanics.errors[0],
        mechanics.errors[1],
        mechanics.errors[2],
        mechanics.errors[3],
        mechanics.errors[4],
        mechanics.states.tracked,
        mechanics.states.reset,
        mechanics.states.silent,
        mechanics.states.regions,
        mechanics.states.owner_switches,
        mechanics.hash,
        corpus.candidate_hard_failures,
        corpus.row_complete_regressions,
        corpus.hash,
    );
    fs::create_dir_all(&root).expect("create finite-support reset report directory");
    fs::write(root.join("proof-summary.tsv"), report)
        .expect("write finite-support reset proof summary");
}
