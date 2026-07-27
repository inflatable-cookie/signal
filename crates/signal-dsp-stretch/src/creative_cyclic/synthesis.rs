use super::{interpolate, plan::Plan, schedule, CandidateError, Request};

#[cfg(test)]
use std::cell::Cell;

// Thread-scoped on purpose. A process-global counter is shared with every
// other test thread, so a concurrent Cyclic render would be counted against
// whichever test happened to be measuring. This is the same defect class the
// creative allocation gate carried before `g10.036` Batch 36.2.
#[cfg(test)]
thread_local! {
    static OUTPUT_ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(super) fn reset_output_allocation_count() {
    OUTPUT_ALLOCATION_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(super) fn output_allocation_count() -> usize {
    OUTPUT_ALLOCATION_COUNT.with(Cell::get)
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
    OUTPUT_ALLOCATION_COUNT.with(|count| count.set(count.get().saturating_add(1)));
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
