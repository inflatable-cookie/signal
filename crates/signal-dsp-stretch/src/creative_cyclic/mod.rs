//! Admitted fixed-ratio centred compressed-anchor Cyclic renderer.

mod interpolate;
mod plan;
mod schedule;
mod synthesis;

#[cfg(test)]
mod tests;

use plan::Plan;

#[derive(Clone, Copy, Debug)]
pub(super) struct Request<'a> {
    pub(super) input: &'a [f32],
    pub(super) channels: usize,
    pub(super) sample_rate: u32,
    pub(super) target_frames: usize,
    pub(super) cycle_us: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidateError {
    InvalidChannels,
    PartialFrame,
    InvalidSampleRate,
    NonFiniteInput,
    InvalidCycle,
    InvalidEmptyTarget,
    UnsupportedCompression,
    UnsupportedRatio,
    ExactIntegerLimit,
    ArithmeticOverflow,
    AllocationOverflow,
}

pub(super) fn render(request: Request<'_>) -> Result<Vec<f32>, CandidateError> {
    let plan = Plan::new(request)?;
    if plan.identity {
        return Ok(request.input.to_vec());
    }
    synthesis::render(request, &plan)
}
