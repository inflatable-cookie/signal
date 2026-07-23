mod analysis;
mod plan;
mod stereo;
mod synthesis;

pub(super) const DIRECT_RENEWAL_DREAM_ENGINE_VERSION: &str =
    "signal-creative-direct-renewal-dream-v1";
pub(super) const DIRECT_RENEWAL_DREAM_RECEIPT_SCHEMA: &str =
    "signal.creative-direct-renewal.receipt.v1";
pub(super) const DIRECT_RENEWAL_DREAM_SUMMARY_SCHEMA: &str =
    "signal.creative-direct-renewal.summary.v1";

#[cfg(test)]
#[macro_use]
mod tests;

#[cfg(test)]
pub(super) use analysis::{cubic_coefficients, interpolated_sample, periodic_hann};
#[cfg(test)]
pub(super) use plan::{
    fft_size, nearest_power_of_two_ties_up, round_half_up_two_thirds, validate_dimensions,
    RenderPlan, MAX_EXACT_INTEGER,
};
pub(super) use stereo::ADMISSION_SEED;
#[cfg(test)]
pub(super) use stereo::{
    address, frequency_weight, high_53, mix64, phase, renew_mono, renew_stereo, BASE_TAG, BIN_TAG,
    FRAME_TAG, SPACE_TAG, TEST_TAG,
};
#[cfg(test)]
pub(super) use synthesis::{boundary_envelope, planned_working_bytes};

#[derive(Clone, Copy)]
pub(super) struct CandidateRequest<'a> {
    pub(super) input: &'a [f32],
    pub(super) channels: usize,
    pub(super) sample_rate: u32,
    pub(super) target_frames: usize,
    pub(super) seed: u64,
    pub(super) space: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidateError {
    InvalidChannels,
    InvalidSampleRate,
    PartialFrame,
    NonFiniteInput,
    InvalidSpace,
    EmptyInput,
    ZeroTarget,
    UnsupportedRatio,
    SizeOverflow,
    AllocationFailed,
    NonFiniteProcessing,
}

pub(super) fn render(request: CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError> {
    let plan = plan::RenderPlan::new(&request)?;
    if plan.target_frames == 0 {
        return Ok(Vec::new());
    }
    synthesis::render(&request, &plan)
}
