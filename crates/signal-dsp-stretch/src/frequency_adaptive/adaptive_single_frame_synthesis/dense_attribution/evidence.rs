use super::super::super::HASH_OFFSET;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DenseMode {
    Ordinary,
    Successor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    PassingControl,
    AnchorPlacement,
    EventReset,
    ActiveOwnerTransport,
    OverlapSynthesis,
    MetricAssociation,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SampleContribution {
    pub(in crate::frequency_adaptive) frame_source: isize,
    pub(in crate::frequency_adaptive) frame_output: isize,
    pub(in crate::frequency_adaptive) frame_length: usize,
    pub(in crate::frequency_adaptive) dual_weight: f64,
    pub(in crate::frequency_adaptive) value: [f64; 2],
    pub(in crate::frequency_adaptive) frame_peak_output: isize,
    pub(in crate::frequency_adaptive) frame_peak_magnitude: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct EventEvidence {
    pub(super) source: usize,
    pub(super) target: usize,
    pub(super) attached: bool,
    pub(super) phase_found: bool,
    pub(super) event_assignment: bool,
    pub(super) owner_counts: [usize; 4],
    pub(super) active_state_hash: u64,
    pub(super) actual_peak: usize,
    pub(super) peak_error: usize,
    pub(super) target_value: f64,
    pub(super) peak_value: f64,
    pub(super) local_peaks: [usize; 3],
    pub(super) closure_error: [f64; 2],
    pub(super) cancellation_ratio: f64,
    pub(super) contributions: Vec<SampleContribution>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RowEvidence {
    pub(super) ratio: f64,
    pub(super) mode: DenseMode,
    pub(super) hard_failure: bool,
    pub(super) stage: Stage,
    pub(super) errors: [usize; 2],
    pub(super) unmatched: usize,
    pub(super) events: [EventEvidence; 2],
    pub(super) peak_contributions: [Vec<SampleContribution>; 2],
    pub(super) hashes: [u64; 8],
}

pub(super) fn row_hash(row: &RowEvidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, row.ratio.to_bits());
    hash(&mut state, row.mode as u64);
    hash(&mut state, row.hard_failure as u64);
    hash(&mut state, row.stage as u64);
    for value in row.errors.into_iter().chain([row.unmatched]) {
        hash(&mut state, value as u64);
    }
    for event in &row.events {
        for value in [
            event.source,
            event.target,
            event.actual_peak,
            event.peak_error,
        ]
        .into_iter()
        .chain(event.owner_counts)
        .chain(event.local_peaks)
        {
            hash(&mut state, value as u64);
        }
        for value in [event.attached, event.phase_found, event.event_assignment] {
            hash(&mut state, value as u64);
        }
        hash(&mut state, event.active_state_hash);
        for value in [
            event.target_value,
            event.peak_value,
            event.closure_error[0],
            event.closure_error[1],
            event.cancellation_ratio,
        ] {
            hash(&mut state, value.to_bits());
        }
        for contribution in &event.contributions {
            hash(&mut state, contribution.frame_source as i64 as u64);
            hash(&mut state, contribution.frame_output as i64 as u64);
            hash(&mut state, contribution.frame_length as u64);
            hash(&mut state, contribution.dual_weight.to_bits());
            for value in contribution.value {
                hash(&mut state, value.to_bits());
            }
            hash(&mut state, contribution.frame_peak_output as i64 as u64);
            hash(&mut state, contribution.frame_peak_magnitude.to_bits());
        }
    }
    for value in &row.hashes[..7] {
        hash(&mut state, *value);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
