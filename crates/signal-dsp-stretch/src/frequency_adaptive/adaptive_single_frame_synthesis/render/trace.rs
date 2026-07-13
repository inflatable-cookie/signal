use super::phase::Trace as PhaseTrace;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PhaseFrameTrace {
    pub(in crate::frequency_adaptive) source: isize,
    pub(in crate::frequency_adaptive) output: isize,
    pub(in crate::frequency_adaptive) length: usize,
    pub(in crate::frequency_adaptive) phase: PhaseTrace,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SynthesisFrameTrace {
    pub(in crate::frequency_adaptive) source: isize,
    pub(in crate::frequency_adaptive) output: isize,
    pub(in crate::frequency_adaptive) length: usize,
    pub(in crate::frequency_adaptive) energy: f64,
    pub(in crate::frequency_adaptive) energy_center: f64,
    pub(in crate::frequency_adaptive) peak_output: isize,
    pub(in crate::frequency_adaptive) peak_magnitude: f64,
    pub(in crate::frequency_adaptive) hash: u64,
}

pub(super) fn hash_phase_trace(state: &mut u64, trace: &PhaseFrameTrace) {
    hash(state, trace.source as i64 as u64);
    hash(state, trace.output as i64 as u64);
    hash(state, trace.length as u64);
    hash(state, trace.phase.source_hop.to_bits());
    hash(state, trace.phase.output_hop.to_bits());
    hash(state, trace.phase.bin as u64);
    hash(state, trace.phase.prior_bin as u64);
    hash(state, trace.phase.peak_owner as u64);
    hash(state, trace.phase.analysis_advance.to_bits());
    hash(state, trace.phase.estimated_frequency.to_bits());
    hash(state, trace.phase.transported_advance.to_bits());
    hash(state, trace.phase.final_advance.to_bits());
    hash(state, trace.phase.event_assignment as u64);
    hash(state, trace.phase.vertical_assignment as u64);
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
