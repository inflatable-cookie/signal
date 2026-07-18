use super::{CoefficientContributionTrace, SynthesisRelationTrace, TrackedPeakPhaseTrace};

#[derive(Clone, Debug)]
pub(in crate::frequency_adaptive) struct StereoRender {
    pub(in crate::frequency_adaptive) channels: [Vec<f64>; 2],
    pub(in crate::frequency_adaptive) uncovered: usize,
    pub(in crate::frequency_adaptive) non_finite: usize,
    pub(in crate::frequency_adaptive) boundary_failures: usize,
    pub(in crate::frequency_adaptive) shared_corrected: usize,
    pub(in crate::frequency_adaptive) shared_fallback: usize,
    pub(in crate::frequency_adaptive) unilateral_non_silent_completions: usize,
    pub(in crate::frequency_adaptive) reference_bins: [usize; 2],
    pub(in crate::frequency_adaptive) active_reference_ties: usize,
    pub(in crate::frequency_adaptive) reference_switches: usize,
    pub(in crate::frequency_adaptive) maximum_projected_relation_error: f64,
    pub(in crate::frequency_adaptive) maximum_constrained_relation_error: f64,
    pub(in crate::frequency_adaptive) synthesis_relation_trace: Option<SynthesisRelationTrace>,
    pub(in crate::frequency_adaptive) coefficient_contribution_trace:
        Option<CoefficientContributionTrace>,
    pub(in crate::frequency_adaptive) peak_region_counts: [usize; 4],
    pub(in crate::frequency_adaptive) tracked_peak_phase_trace: TrackedPeakPhaseTrace,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish(
    channels: [Vec<f64>; 2],
    uncovered: usize,
    shared_corrected: usize,
    shared_fallback: usize,
    unilateral_non_silent_completions: usize,
    reference_bins: [usize; 2],
    active_reference_ties: usize,
    reference_switches: usize,
    maximum_projected_relation_error: f64,
    maximum_constrained_relation_error: f64,
    synthesis_relation_trace: Option<SynthesisRelationTrace>,
    coefficient_contribution_trace: Option<CoefficientContributionTrace>,
    peak_region_counts: [usize; 4],
    tracked_peak_phase_trace: TrackedPeakPhaseTrace,
) -> StereoRender {
    let non_finite = channels
        .iter()
        .flat_map(|channel| channel.iter())
        .filter(|sample| !sample.is_finite())
        .count();
    let boundary_failures = channels
        .iter()
        .map(|channel| {
            usize::from(channel.first().is_none_or(|sample| !sample.is_finite()))
                + usize::from(channel.last().is_none_or(|sample| !sample.is_finite()))
        })
        .sum();
    let mut hash = super::super::super::hash_samples(&channels[0]);
    super::super::hash_values(
        &mut hash,
        &[super::super::super::hash_samples(&channels[1])],
    );
    StereoRender {
        channels,
        uncovered,
        non_finite,
        boundary_failures,
        shared_corrected,
        shared_fallback,
        unilateral_non_silent_completions,
        reference_bins,
        active_reference_ties,
        reference_switches,
        maximum_projected_relation_error,
        maximum_constrained_relation_error,
        synthesis_relation_trace,
        coefficient_contribution_trace,
        peak_region_counts,
        tracked_peak_phase_trace,
        hash,
    }
}
