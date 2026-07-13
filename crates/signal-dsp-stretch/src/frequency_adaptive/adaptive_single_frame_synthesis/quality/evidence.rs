use super::super::super::HASH_OFFSET;
use super::CaseEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum QualityDirection {
    FrozenMonoDevelopmentObjective,
    MeasuredPhaseEventVerticalOrSynthesisStage,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct QualityReview {
    pub cases: Vec<CaseEvidence>,
    pub hard_failures: usize,
    pub combined_regressions: usize,
    pub evidence_hash: u64,
    pub direction: QualityDirection,
}

pub(super) fn case_hash(case: &CaseEvidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, case.control as u64);
    hash(&mut state, case.ratio.to_bits());
    hash(&mut state, case.selected_points as u64);
    hash(&mut state, case.ownership_failures as u64);
    hash(&mut state, case.combined_regressions as u64);
    for mode in &case.modes {
        for value in mode.hard_failures {
            hash(&mut state, value as u64);
        }
        for value in mode
            .lengths
            .into_iter()
            .chain(mode.coverage)
            .chain(mode.assembly_actions)
            .chain([mode.non_finite_values])
            .chain(mode.phase_assignments)
            .chain(mode.dense_errors)
            .chain([mode.dense_unmatched, mode.isolated_error])
        {
            hash(&mut state, value as u64);
        }
        for value in mode
            .identity_error
            .into_iter()
            .chain(mode.endpoint_rms)
            .chain([
                mode.frame_condition,
                mode.symmetry_error,
                mode.imaginary_residue,
                mode.tone_angular_error,
                mode.impulse_crest_db,
                mode.replica_ratio,
            ])
            .chain(mode.texture)
            .chain([mode.silence_peak])
        {
            hash(&mut state, value.to_bits());
        }
        for value in mode.hashes {
            hash(&mut state, value);
        }
    }
    for value in case.mode_deltas {
        hash(&mut state, value.to_bits());
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
