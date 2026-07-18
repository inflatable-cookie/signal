use std::{fs, path::PathBuf};

use super::{
    peak_region_feasibility::{self, PeakRegionReview, PeakRegionScreen},
    shared_rotation_region_locked_proof::{
        corpus::review as corpus_review,
        mechanics::review as mechanics_review,
        SharedRotationCorpusReview, SharedRotationMechanicsReview,
    },
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::state_complete_linked_phase_vocoder::{
    self, Policy,
};

const MAX_FINALISTS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ScreenedCandidate {
    pub(in crate::frequency_adaptive) index: usize,
    pub(in crate::frequency_adaptive) policy: Policy,
    pub(in crate::frequency_adaptive) screen: PeakRegionScreen,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct Finalist {
    pub(in crate::frequency_adaptive) index: usize,
    pub(in crate::frequency_adaptive) policy: Policy,
    pub(in crate::frequency_adaptive) stereo: PeakRegionReview,
    pub(in crate::frequency_adaptive) mechanics: SharedRotationMechanicsReview,
    pub(in crate::frequency_adaptive) mono: SharedRotationCorpusReview,
    pub(in crate::frequency_adaptive) passed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct CalibrationReview {
    pub(in crate::frequency_adaptive) screens: Vec<ScreenedCandidate>,
    pub(in crate::frequency_adaptive) finalists: Vec<Finalist>,
    pub(in crate::frequency_adaptive) frozen: Option<usize>,
}

pub(in crate::frequency_adaptive) fn review() -> CalibrationReview {
    let mut screens = state_complete_linked_phase_vocoder::candidates()
        .into_iter()
        .enumerate()
        .map(|(index, policy)| {
            let screen = peak_region_feasibility::screen_candidate(
                "stretch-state-complete-screen",
                |inputs, ratio, sample_rate| {
                    state_complete_linked_phase_vocoder::stereo_adapter(
                        inputs,
                        ratio,
                        sample_rate,
                        policy,
                    )
                },
            );
            ScreenedCandidate {
                index,
                policy,
                screen,
            }
        })
        .collect::<Vec<_>>();
    screens.sort_by_key(screen_key);

    let finalists = select_finalists(&screens)
        .into_iter()
        .map(|screened| complete_finalist(screened.index, screened.policy))
        .collect::<Vec<_>>();
    let frozen = finalists
        .iter()
        .filter(|finalist| finalist.passed)
        .min_by_key(|finalist| finalist.index)
        .map(|finalist| finalist.index);
    let review = CalibrationReview {
        screens,
        finalists,
        frozen,
    };
    write_ledger(&review);
    review
}

fn select_finalists(screens: &[ScreenedCandidate]) -> Vec<&ScreenedCandidate> {
    let mut selected = Vec::with_capacity(MAX_FINALISTS);
    for frequency_level in state_complete_linked_phase_vocoder::POLICY_LEVELS[1] {
        for history_level in state_complete_linked_phase_vocoder::POLICY_LEVELS[5] {
            let candidate = screens
                .iter()
                .find(|candidate| {
                    candidate.policy.predecessor_tolerance_region_widths == frequency_level
                        && candidate.policy.history_tolerance_radians == history_level
                })
                .expect("candidate stratum");
            selected.push(candidate);
        }
    }
    selected
}

fn screen_key(candidate: &ScreenedCandidate) -> (usize, usize, usize, usize, usize) {
    (
        candidate.screen.structural_failures,
        candidate.screen.candidate_failures,
        candidate.screen.local_consistency_failures,
        candidate.screen.metric_regressions,
        candidate.index,
    )
}

fn complete_finalist(index: usize, policy: Policy) -> Finalist {
    let stereo = peak_region_feasibility::review_candidate(
        &format!("stretch-state-complete-finalist-{index:02}"),
        |inputs, ratio, sample_rate| {
            state_complete_linked_phase_vocoder::stereo_adapter(inputs, ratio, sample_rate, policy)
        },
    );
    let mechanics = mechanics_review(|inputs, ratio, sample_rate| {
        state_complete_linked_phase_vocoder::render(inputs, ratio, sample_rate, policy)
    });
    let mono = corpus_review(
        |inputs, ratio, sample_rate| {
            state_complete_linked_phase_vocoder::render(inputs, ratio, sample_rate, policy)
        },
        "state-complete-linked-phase-vocoder",
    );
    let passed = stereo.repeated
        && stereo.candidate_failures == 0
        && stereo.local_consistency_failures == 0
        && stereo.rows.iter().all(|row| row.structural_failures == 0)
        && mechanics.repeated
        && mechanics.structural_failures == 0
        && mechanics.identity_mismatches == 0
        && mechanics.errors.iter().all(|error| *error <= 1.0e-12)
        && mechanics.silent_peer_peak == 0.0
        && mechanics.states.reset > 0
        && mechanics.states.locked > 0
        && mechanics.states.diffuse > 0
        && mono.repeated
        && mono.candidate_hard_failures == 0
        && mono.row_complete_regressions == 0;
    Finalist {
        index,
        policy,
        stereo,
        mechanics,
        mono,
        passed,
    }
}

fn write_ledger(review: &CalibrationReview) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-state-complete-linked-phase-vocoder");
    fs::create_dir_all(&root).expect("create state-complete calibration report");
    let mut ledger = format!(
        "definition\t4\ncandidate_count\t{}\nfinalist_count\t{}\nfrozen\t{}\nindex\tprominence\tfrequency_tolerance_region_widths\ttransient_rise_db\treset_support_frames\tunlock_coherence\thistory_tolerance_radians\trows\tstructural_failures\tcandidate_failures\tlocal_failures\tmetric_regressions\tregions\tlocked\treset\tunlocked\tevidence_hash\n",
        review.screens.len(),
        review.finalists.len(),
        review
            .frozen
            .map_or_else(|| "none".to_owned(), |index| index.to_string()),
    );
    for candidate in &review.screens {
        let policy = candidate.policy;
        let screen = &candidate.screen;
        ledger.push_str(&format!(
            "{}\t{:.3}\t{}\t{:.3}\t{}\t{:.3}\t{:.12}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:016x}\n",
            candidate.index,
            policy.peak_prominence,
            policy.predecessor_tolerance_region_widths,
            policy.transient_rise_db,
            policy.reset_support_frames,
            policy.unlock_coherence,
            policy.history_tolerance_radians,
            screen.rows,
            screen.structural_failures,
            screen.candidate_failures,
            screen.local_consistency_failures,
            screen.metric_regressions,
            screen.peak_region_counts[0],
            screen.peak_region_counts[1],
            screen.peak_region_counts[2],
            screen.peak_region_counts[3],
            screen.evidence_hash,
        ));
    }
    fs::write(root.join("calibration-ledger.tsv"), ledger)
        .expect("write state-complete calibration ledger");

    let mut finalists = "index\tpassed\tstereo_failures\tlocal_failures\tmechanics_structural\tmechanics_errors\tmono_hard_failures\tmono_row_complete_regressions\tstereo_hash\tmechanics_hash\tmono_hash\n".to_owned();
    for finalist in &review.finalists {
        finalists.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:e},{:e},{:e},{:e},{:e}\t{}\t{}\t{:016x}\t{:016x}\t{:016x}\n",
            finalist.index,
            finalist.passed,
            finalist.stereo.candidate_failures,
            finalist.stereo.local_consistency_failures,
            finalist.mechanics.structural_failures,
            finalist.mechanics.errors[0],
            finalist.mechanics.errors[1],
            finalist.mechanics.errors[2],
            finalist.mechanics.errors[3],
            finalist.mechanics.errors[4],
            finalist.mono.candidate_hard_failures,
            finalist.mono.row_complete_regressions,
            finalist.stereo.evidence_hash,
            finalist.mechanics.hash,
            finalist.mono.hash,
        ));
    }
    fs::write(root.join("finalists.tsv"), finalists).expect("write state-complete finalist report");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_complete_screen_order_is_failure_first_and_deterministic() {
        let policy = state_complete_linked_phase_vocoder::candidates()[0];
        let candidate = |index, failures, local, regressions| ScreenedCandidate {
            index,
            policy,
            screen: PeakRegionScreen {
                rows: 12,
                structural_failures: 0,
                candidate_failures: failures,
                metric_regressions: regressions,
                local_consistency_failures: local,
                peak_region_counts: [1; 4],
                evidence_hash: 0,
            },
        };
        let mut rows = [
            candidate(3, 1, 0, 0),
            candidate(2, 0, 1, 0),
            candidate(1, 0, 0, 1),
        ];
        rows.sort_by_key(screen_key);
        assert_eq!(rows.map(|row| row.index), [1, 2, 3]);
    }

    #[test]
    fn state_complete_finalists_cover_frequency_and_history_strata() {
        let mut screens = state_complete_linked_phase_vocoder::candidates()
            .into_iter()
            .enumerate()
            .map(|(index, policy)| ScreenedCandidate {
                index,
                policy,
                screen: PeakRegionScreen {
                    rows: 12,
                    structural_failures: 0,
                    candidate_failures: index,
                    metric_regressions: 0,
                    local_consistency_failures: 0,
                    peak_region_counts: [1; 4],
                    evidence_hash: index as u64,
                },
            })
            .collect::<Vec<_>>();
        screens.sort_by_key(screen_key);
        let finalists = select_finalists(&screens);
        assert_eq!(finalists.len(), 4);
        assert_eq!(
            finalists
                .iter()
                .map(|candidate| (
                    candidate.policy.predecessor_tolerance_region_widths,
                    candidate.policy.history_tolerance_radians,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[1][0],
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[5][0],
                ),
                (
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[1][0],
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[5][1],
                ),
                (
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[1][1],
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[5][0],
                ),
                (
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[1][1],
                    state_complete_linked_phase_vocoder::POLICY_LEVELS[5][1],
                ),
            ]
        );
    }

    #[test]
    #[ignore = "requires frozen exact-source development corpus"]
    fn state_complete_linked_phase_vocoder_bounded_calibration() {
        let result = review();
        assert_eq!(result.screens.len(), 64);
        assert_eq!(result.finalists.len(), 4);
        assert_eq!(
            result
                .finalists
                .iter()
                .map(|finalist| (
                    finalist.index,
                    finalist.stereo.candidate_failures,
                    finalist.stereo.local_consistency_failures,
                    finalist.mono.row_complete_regressions,
                    finalist.stereo.evidence_hash,
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, 11, 0, 0x2dcd_edcd_293b_c82d),
                (1, 1, 17, 0, 0x5fc1_5fbe_ab44_0c89),
                (16, 1, 15, 0, 0xe2b6_5145_a109_4142),
                (17, 1, 13, 0, 0x8fc9_5547_c4d9_5682),
            ]
        );
        assert!(result.finalists.iter().all(|finalist| {
            finalist.mechanics.repeated
                && finalist.mechanics.structural_failures == 0
                && finalist
                    .mechanics
                    .errors
                    .iter()
                    .all(|error| *error <= 1.0e-12)
                && finalist.mono.candidate_hard_failures == 0
        }));
        assert_eq!(result.frozen, None);
    }
}
