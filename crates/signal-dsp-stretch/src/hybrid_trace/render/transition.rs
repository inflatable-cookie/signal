use signal_primitives::Sample;

use super::StretchHybridTransitionRejection;

const MIN_TRANSITION_CORRELATION: f64 = 0.50;
pub(super) const MAX_NORMALIZATION_GAIN_DB: f64 = 1.0;

#[derive(Clone, Copy, Debug)]
pub(super) struct TransitionEvaluation {
    pub(super) correlation: f64,
    pub(super) max_normalization_gain_db: f64,
    pub(super) rejection: Option<StretchHybridTransitionRejection>,
}

pub(super) fn evaluate_transition(
    outgoing: &[Sample],
    incoming: &[Sample],
    range: (usize, usize),
) -> TransitionEvaluation {
    let correlation =
        normalized_correlation(&outgoing[range.0..range.1], &incoming[range.0..range.1]);
    let max_normalization_gain_db = max_normalization_gain_db(correlation);
    let rejection = if correlation < MIN_TRANSITION_CORRELATION {
        Some(StretchHybridTransitionRejection::LowCorrelation)
    } else if max_normalization_gain_db > MAX_NORMALIZATION_GAIN_DB {
        Some(StretchHybridTransitionRejection::ExcessNormalization)
    } else {
        None
    };
    TransitionEvaluation {
        correlation,
        max_normalization_gain_db,
        rejection,
    }
}

fn normalized_correlation(left: &[Sample], right: &[Sample]) -> f64 {
    let (dot, left_energy, right_energy) = left.iter().zip(right.iter()).fold(
        (0.0f64, 0.0f64, 0.0f64),
        |(dot, left_energy, right_energy), (left, right)| {
            let left = f64::from(*left);
            let right = f64::from(*right);
            (
                dot + left * right,
                left_energy + left * left,
                right_energy + right * right,
            )
        },
    );
    if left_energy <= 1.0e-20 && right_energy <= 1.0e-20 {
        1.0
    } else if left_energy <= 1.0e-20 || right_energy <= 1.0e-20 {
        0.0
    } else {
        (dot / (left_energy * right_energy).sqrt()).clamp(-1.0, 1.0)
    }
}

pub(super) fn max_normalization_gain_db(correlation: f64) -> f64 {
    let midpoint_denominator = (0.5 + 0.5 * correlation).max(1.0e-12).sqrt();
    20.0 * (1.0 / midpoint_denominator).log10()
}

pub(super) fn apply_transition(
    output: &mut [Sample],
    outgoing: &[Sample],
    incoming: &[Sample],
    range: (usize, usize),
    correlation: f64,
) {
    let len = range.1.saturating_sub(range.0);
    for offset in 0..len {
        let progress = (offset + 1) as f64 / (len + 1) as f64;
        let incoming_weight = 0.5 - 0.5 * (std::f64::consts::PI * progress).cos();
        let outgoing_weight = 1.0 - incoming_weight;
        let denominator = (outgoing_weight * outgoing_weight
            + incoming_weight * incoming_weight
            + 2.0 * correlation * outgoing_weight * incoming_weight)
            .max(1.0e-12)
            .sqrt();
        let gain = 1.0 / denominator;
        let index = range.0 + offset;
        output[index] = ((f64::from(outgoing[index]) * outgoing_weight
            + f64::from(incoming[index]) * incoming_weight)
            * gain) as Sample;
    }
}
