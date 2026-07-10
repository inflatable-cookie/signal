use signal_primitives::Sample;

use super::StretchHybridTransitionRejection;

const MIN_TRANSITION_CORRELATION: f64 = 0.50;
pub(super) const MAX_NORMALIZATION_GAIN_DB: f64 = 1.0;
const ALIGNMENT_REVIEW_RADIUS_FRAMES: i64 = 256;

#[derive(Clone, Copy, Debug)]
pub(super) struct TransitionEvaluation {
    pub(super) correlation: f64,
    pub(super) max_normalization_gain_db: f64,
    pub(super) best_lag_frames: i64,
    pub(super) best_lag_correlation: f64,
    pub(super) best_lag_normalization_gain_db: f64,
    pub(super) rejection: Option<StretchHybridTransitionRejection>,
}

pub(super) fn evaluate_transition(
    outgoing: &[Sample],
    incoming: &[Sample],
    range: (usize, usize),
) -> TransitionEvaluation {
    let correlation =
        normalized_correlation(&outgoing[range.0..range.1], &incoming[range.0..range.1]);
    let zero_lag_normalization_gain_db = max_normalization_gain_db(correlation);
    let (best_lag_frames, best_lag_correlation) = best_lag_correlation(outgoing, incoming, range);
    let best_lag_normalization_gain_db = max_normalization_gain_db(best_lag_correlation);
    let rejection = if correlation < MIN_TRANSITION_CORRELATION {
        Some(StretchHybridTransitionRejection::LowCorrelation)
    } else if zero_lag_normalization_gain_db > MAX_NORMALIZATION_GAIN_DB {
        Some(StretchHybridTransitionRejection::ExcessNormalization)
    } else {
        None
    };
    TransitionEvaluation {
        correlation,
        max_normalization_gain_db: zero_lag_normalization_gain_db,
        best_lag_frames,
        best_lag_correlation,
        best_lag_normalization_gain_db,
        rejection,
    }
}

fn best_lag_correlation(
    outgoing: &[Sample],
    incoming: &[Sample],
    range: (usize, usize),
) -> (i64, f64) {
    let mut best_lag = 0i64;
    let mut best_correlation =
        normalized_correlation(&outgoing[range.0..range.1], &incoming[range.0..range.1]);
    for lag in -ALIGNMENT_REVIEW_RADIUS_FRAMES..=ALIGNMENT_REVIEW_RADIUS_FRAMES {
        let Some(correlation) = correlation_at_lag(outgoing, incoming, range, lag) else {
            continue;
        };
        if correlation > best_correlation
            || (correlation == best_correlation && lag.abs() < best_lag.abs())
        {
            best_lag = lag;
            best_correlation = correlation;
        }
    }
    (best_lag, best_correlation)
}

fn correlation_at_lag(
    outgoing: &[Sample],
    incoming: &[Sample],
    range: (usize, usize),
    lag: i64,
) -> Option<f64> {
    let mut start = range.0;
    let mut end = range.1.min(outgoing.len());
    if lag < 0 {
        start = start.max(lag.unsigned_abs() as usize);
    } else {
        end = end.min(incoming.len().saturating_sub(lag as usize));
    }
    let minimum_overlap = range.1.saturating_sub(range.0).div_ceil(2).max(16);
    if end.saturating_sub(start) < minimum_overlap {
        return None;
    }
    let incoming_start = (start as i64 + lag) as usize;
    let incoming_end = incoming_start + end.saturating_sub(start);
    Some(normalized_correlation(
        &outgoing[start..end],
        &incoming[incoming_start..incoming_end],
    ))
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
