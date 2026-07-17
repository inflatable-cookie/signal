mod corpus;
mod mechanics;

use std::{fs, path::PathBuf};

use super::peak_region_feasibility;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::shared_rotation_region_locked;
use corpus::review as corpus_review;
use mechanics::review as mechanics_review;

pub(in crate::frequency_adaptive) use peak_region_feasibility::PeakRegionReview as SharedRotationStereoReview;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum SharedRotationProofDirection {
    ListeningCheckpoint,
    OperatorReview,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SharedRotationMechanicsReview {
    pub(in crate::frequency_adaptive) structural_failures: usize,
    pub(in crate::frequency_adaptive) identity_mismatches: usize,
    pub(in crate::frequency_adaptive) errors: [f64; 5],
    pub(in crate::frequency_adaptive) silent_peer_peak: f64,
    pub(in crate::frequency_adaptive) states: shared_rotation_region_locked::StateCounts,
    pub(in crate::frequency_adaptive) trajectory_break_resets: usize,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SharedRotationCorpusRow {
    pub(in crate::frequency_adaptive) id: &'static str,
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) current: [f64; 7],
    pub(in crate::frequency_adaptive) candidate: [f64; 7],
    pub(in crate::frequency_adaptive) rubber_band: [f64; 7],
    pub(in crate::frequency_adaptive) candidate_regressions: usize,
    pub(in crate::frequency_adaptive) hard_passes: [bool; 3],
    pub(in crate::frequency_adaptive) hashes: [u64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SharedRotationCorpusReview {
    pub(in crate::frequency_adaptive) rows: Vec<SharedRotationCorpusRow>,
    pub(in crate::frequency_adaptive) candidate_hard_failures: usize,
    pub(in crate::frequency_adaptive) row_complete_regressions: usize,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SharedRotationProofReview {
    pub(in crate::frequency_adaptive) stereo: SharedRotationStereoReview,
    pub(in crate::frequency_adaptive) mechanics: SharedRotationMechanicsReview,
    pub(in crate::frequency_adaptive) corpus: SharedRotationCorpusReview,
    pub(in crate::frequency_adaptive) direction: SharedRotationProofDirection,
}

pub(in crate::frequency_adaptive) fn review() -> SharedRotationProofReview {
    let stereo = peak_region_feasibility::review_candidate(
        "stretch-shared-rotation-region-locked",
        shared_rotation_region_locked::stereo_adapter,
    );
    let mechanics = mechanics_review();
    let corpus = corpus_review();
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
    let passed = stereo.repeated
        && stereo.candidate_failures == 0
        && stereo.local_consistency_failures == 0
        && mechanics_pass
        && corpus.repeated
        && corpus.candidate_hard_failures == 0
        && corpus.row_complete_regressions == 0;
    let direction = if passed {
        SharedRotationProofDirection::ListeningCheckpoint
    } else {
        SharedRotationProofDirection::OperatorReview
    };
    write_summary(&stereo, &mechanics, &corpus, direction);
    SharedRotationProofReview {
        stereo,
        mechanics,
        corpus,
        direction,
    }
}

fn write_summary(
    stereo: &SharedRotationStereoReview,
    mechanics: &SharedRotationMechanicsReview,
    corpus: &SharedRotationCorpusReview,
    direction: SharedRotationProofDirection,
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-shared-rotation-region-locked");
    let mut report = format!(
        "direction\t{direction:?}\nstereo_current_failures\t{}\nstereo_candidate_failures\t{}\nstereo_local_failures\t{}\nstereo_metric_regressions\t{}\nstereo_evidence_hash\t{:016x}\nmechanics_structural_failures\t{}\nmechanics_identity_mismatches\t{}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nmechanics_silent_peer_peak\t{:e}\nmechanics_states\t{},{},{},{},{}\nmechanics_hash\t{:016x}\ncorpus_candidate_hard_failures\t{}\ncorpus_row_complete_regressions\t{}\ncorpus_hash\t{:016x}\nrow\tratio\tregressions\tcurrent_hard\tcandidate_hard\trubber_hard\tcurrent_hash\tcandidate_hash\trubber_hash\n",
        stereo.current_failures,
        stereo.candidate_failures,
        stereo.local_consistency_failures,
        stereo.metric_regressions,
        stereo.evidence_hash,
        mechanics.structural_failures,
        mechanics.identity_mismatches,
        mechanics.errors[0], mechanics.errors[1], mechanics.errors[2],
        mechanics.errors[3], mechanics.errors[4], mechanics.silent_peer_peak,
        mechanics.states.tracked, mechanics.states.reset, mechanics.states.silent,
        mechanics.states.regions, mechanics.states.owner_switches, mechanics.hash,
        corpus.candidate_hard_failures, corpus.row_complete_regressions, corpus.hash,
    );
    for row in &corpus.rows {
        report.push_str(&format!(
            "{}\t{:.2}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{:016x}\n",
            row.id,
            row.ratio,
            row.candidate_regressions,
            row.hard_passes[0],
            row.hard_passes[1],
            row.hard_passes[2],
            row.hashes[0],
            row.hashes[1],
            row.hashes[2],
        ));
    }
    fs::write(root.join("proof-summary.tsv"), report).expect("write shared-rotation proof summary");
}
