use super::super::super::HASH_OFFSET;
use super::super::quality::Control;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RowEvidence {
    pub(super) control: Control,
    pub(super) ratio: f64,
    pub(super) detected: Vec<usize>,
    pub(super) expected: Vec<usize>,
    pub(super) failures: [usize; 8],
    pub(super) identity_error: [f64; 4],
    pub(super) tone_errors: [f64; 2],
    pub(super) event_errors: [usize; 3],
    pub(super) owner_counts: [usize; 4],
    pub(super) resolution_transitions: usize,
    pub(super) matched_resolution_transitions: usize,
    pub(super) frame_counts: [usize; 3],
    pub(super) phase_limits: [f64; 2],
    pub(super) silence_peak: f64,
    pub(super) hashes: [u64; 9],
}

pub(super) fn row_hash(row: &RowEvidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, row.control as u64);
    hash(&mut state, row.ratio.to_bits());
    for value in row.detected.iter().chain(&row.expected) {
        hash(&mut state, *value as u64);
    }
    for value in row
        .failures
        .into_iter()
        .chain(row.event_errors)
        .chain(row.owner_counts)
        .chain(row.frame_counts)
    {
        hash(&mut state, value as u64);
    }
    for value in row
        .identity_error
        .into_iter()
        .chain(row.tone_errors)
        .chain(row.phase_limits)
        .chain([row.silence_peak])
    {
        hash(&mut state, value.to_bits());
    }
    for value in &row.hashes[..8] {
        hash(&mut state, *value);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
