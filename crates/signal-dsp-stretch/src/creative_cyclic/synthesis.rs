use super::{interpolate, plan::Plan, schedule, CandidateError, Request};

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
static OUTPUT_ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
pub(super) fn reset_output_allocation_count() {
    OUTPUT_ALLOCATION_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(super) fn output_allocation_count() -> usize {
    OUTPUT_ALLOCATION_COUNT.load(Ordering::SeqCst)
}

pub(super) fn render(request: Request<'_>, plan: &Plan) -> Result<Vec<f32>, CandidateError> {
    if plan.target_frames == 0 {
        return Ok(Vec::new());
    }
    let sample_count = plan
        .target_frames
        .checked_mul(plan.channels)
        .ok_or(CandidateError::AllocationOverflow)?;
    #[cfg(test)]
    OUTPUT_ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut output = vec![0.0_f32; sample_count];
    for output_frame in 0..plan.target_frames {
        let (left_anchor, right_anchor, remainder) = schedule::output_anchors(plan, output_frame)?;
        let right_weight = plan.window[remainder];
        let left_weight = 1.0 - right_weight;
        let left_position = schedule::position_numerator(plan, left_anchor, output_frame)?;
        let right_position = schedule::position_numerator(plan, right_anchor, output_frame)?;
        for channel in 0..plan.channels {
            let left = interpolate::sample(
                request.input,
                plan.input_frames,
                plan.channels,
                channel,
                left_position,
                plan.denominator,
            );
            let right = interpolate::sample(
                request.input,
                plan.input_frames,
                plan.channels,
                channel,
                right_position,
                plan.denominator,
            );
            output[output_frame * plan.channels + channel] =
                (left_weight * left + right_weight * right) as f32;
        }
    }
    Ok(output)
}
