//! Admitted centred compressed-anchor Cyclic renderer.

mod interpolate;
mod plan;
mod schedule;
mod synthesis;

#[cfg(test)]
mod tests;

use plan::Plan;

#[cfg(test)]
pub(super) const CONTINUOUS_BEHAVIOR_ID: &str = "signal-creative-continuous-event-ledger-cyclic-v1";

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

pub(super) fn render_continuous(request: Request<'_>) -> Result<Vec<f32>, CandidateError> {
    let plan = Plan::new(request)?;
    if plan.input_frames == 0 {
        return Ok(Vec::new());
    }
    let minimum = plan
        .input_frames
        .checked_mul(2)
        .ok_or(CandidateError::ArithmeticOverflow)?;
    if plan.target_frames < minimum {
        return Err(CandidateError::UnsupportedRatio);
    }
    synthesis::render(request, &plan)
}
