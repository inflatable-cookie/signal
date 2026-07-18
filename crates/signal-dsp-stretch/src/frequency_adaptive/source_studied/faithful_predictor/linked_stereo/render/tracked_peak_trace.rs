use rustfft::num_complex::Complex64;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in super::super) struct PhaseFieldClassTrace {
    pub(in super::super) coefficients: usize,
    pub(in super::super) phase_delta_squared_sum: f64,
    pub(in super::super) maximum_phase_delta: f64,
    pub(in super::super) relation_bins: usize,
    pub(in super::super) relation_before_squared_sum: f64,
    pub(in super::super) relation_after_squared_sum: f64,
    pub(in super::super) maximum_relation_before: f64,
    pub(in super::super) maximum_relation_after: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::frequency_adaptive) struct TrackedPeakPhaseTrace {
    pub(in super::super) classes: [PhaseFieldClassTrace; 3],
}

impl TrackedPeakPhaseTrace {
    pub(super) fn merge(&mut self, other: Self) {
        for (total, frame) in self.classes.iter_mut().zip(other.classes) {
            total.coefficients += frame.coefficients;
            total.phase_delta_squared_sum += frame.phase_delta_squared_sum;
            total.maximum_phase_delta = total.maximum_phase_delta.max(frame.maximum_phase_delta);
            total.relation_bins += frame.relation_bins;
            total.relation_before_squared_sum += frame.relation_before_squared_sum;
            total.relation_after_squared_sum += frame.relation_after_squared_sum;
            total.maximum_relation_before = total
                .maximum_relation_before
                .max(frame.maximum_relation_before);
            total.maximum_relation_after = total
                .maximum_relation_after
                .max(frame.maximum_relation_after);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_phase_field(
    trace: &mut TrackedPeakPhaseTrace,
    bin: usize,
    pair: [Option<usize>; 2],
    region_states: &[Option<[Option<usize>; 2]>],
    applied: [bool; 2],
    current: &[Vec<Complex64>; 2],
    relational: &[Vec<Complex64>; 2],
    output: &[Vec<Complex64>; 2],
) {
    let boundary = bin == 0
        || bin + 1 == region_states.len()
        || region_states[bin - 1] != region_states[bin]
        || region_states[bin + 1] != region_states[bin];
    let class = |channel: usize| {
        if pair[channel] == Some(bin) {
            0
        } else if boundary {
            2
        } else {
            1
        }
    };
    for channel in 0..2 {
        if !applied[channel] {
            continue;
        }
        let phase_delta =
            wrap((output[channel][bin] * relational[channel][bin].conj()).arg()).abs();
        let class = &mut trace.classes[class(channel)];
        class.coefficients += 1;
        class.phase_delta_squared_sum += phase_delta * phase_delta;
        class.maximum_phase_delta = class.maximum_phase_delta.max(phase_delta);
    }
    if !applied.into_iter().any(|value| value)
        || current[0][bin].norm_sqr() == 0.0
        || current[1][bin].norm_sqr() == 0.0
    {
        return;
    }
    let relation = |field: &[Vec<Complex64>; 2]| {
        wrap(
            (field[1][bin] * field[0][bin].conj()).arg()
                - (current[1][bin] * current[0][bin].conj()).arg(),
        )
        .abs()
    };
    let before = relation(relational);
    let after = relation(output);
    let relation_class = if pair[0] == Some(bin) || pair[1] == Some(bin) {
        0
    } else if boundary {
        2
    } else {
        1
    };
    let class = &mut trace.classes[relation_class];
    class.relation_bins += 1;
    class.relation_before_squared_sum += before * before;
    class.relation_after_squared_sum += after * after;
    class.maximum_relation_before = class.maximum_relation_before.max(before);
    class.maximum_relation_after = class.maximum_relation_after.max(after);
}

fn wrap(phase: f64) -> f64 {
    (phase + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
