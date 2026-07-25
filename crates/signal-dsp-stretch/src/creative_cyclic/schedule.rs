use super::{plan::Plan, CandidateError};

#[derive(Clone, Copy, Debug)]
pub(super) struct Anchor {
    pub(super) index: i128,
    pub(super) numerator: i128,
}

pub(super) fn anchor(plan: &Plan, index: i128) -> Result<Anchor, CandidateError> {
    let h = i128::try_from(plan.cycle_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let input =
        i128::try_from(plan.input_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let target =
        i128::try_from(plan.target_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let numerator = index
        .checked_mul(h)
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(1))
        .and_then(|value| value.checked_mul(input))
        .and_then(|value| value.checked_sub(target))
        .ok_or(CandidateError::ArithmeticOverflow)?;
    Ok(Anchor { index, numerator })
}

pub(super) fn position_numerator(
    plan: &Plan,
    anchor: Anchor,
    output: usize,
) -> Result<i128, CandidateError> {
    let output = i128::try_from(output).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let h = i128::try_from(plan.cycle_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let delta = output
        .checked_sub(
            anchor
                .index
                .checked_mul(h)
                .ok_or(CandidateError::ArithmeticOverflow)?,
        )
        .ok_or(CandidateError::ArithmeticOverflow)?;
    anchor
        .numerator
        .checked_add(
            plan.denominator
                .checked_mul(delta)
                .ok_or(CandidateError::ArithmeticOverflow)?,
        )
        .ok_or(CandidateError::ArithmeticOverflow)
}

pub(super) fn output_anchors(
    plan: &Plan,
    output: usize,
) -> Result<(Anchor, Anchor, usize), CandidateError> {
    let k0 = output / plan.cycle_frames;
    let remainder = output - k0 * plan.cycle_frames;
    let k0 = i128::try_from(k0).map_err(|_| CandidateError::ArithmeticOverflow)?;
    Ok((anchor(plan, k0)?, anchor(plan, k0 + 1)?, remainder))
}

#[cfg(test)]
pub(super) fn ideal_map_numerator(plan: &Plan, output: usize) -> Result<i128, CandidateError> {
    let output = i128::try_from(output).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let input =
        i128::try_from(plan.input_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    let target =
        i128::try_from(plan.target_frames).map_err(|_| CandidateError::ArithmeticOverflow)?;
    output
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .and_then(|value| value.checked_mul(input))
        .and_then(|value| value.checked_sub(target))
        .ok_or(CandidateError::ArithmeticOverflow)
}
