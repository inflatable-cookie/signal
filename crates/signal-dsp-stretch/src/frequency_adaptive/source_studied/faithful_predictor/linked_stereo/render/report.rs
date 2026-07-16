#[derive(Clone, Debug)]
pub(in super::super) struct StereoRender {
    pub(in super::super) channels: [Vec<f64>; 2],
    pub(in super::super) uncovered: usize,
    pub(in super::super) non_finite: usize,
    pub(in super::super) boundary_failures: usize,
    pub(in super::super) shared_corrected: usize,
    pub(in super::super) shared_fallback: usize,
    pub(in super::super) unilateral_non_silent_completions: usize,
    pub(in super::super) reference_bins: [usize; 2],
    pub(in super::super) active_reference_ties: usize,
    pub(in super::super) reference_switches: usize,
    pub(in super::super) maximum_projected_relation_error: f64,
    pub(in super::super) maximum_constrained_relation_error: f64,
    pub(in super::super) hash: u64,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish(
    channels: [Vec<f64>; 2],
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    shared_corrected: usize,
    shared_fallback: usize,
    unilateral_non_silent_completions: usize,
    reference_bins: [usize; 2],
    active_reference_ties: usize,
    reference_switches: usize,
    maximum_projected_relation_error: f64,
    maximum_constrained_relation_error: f64,
) -> StereoRender {
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
        hash,
    }
}
