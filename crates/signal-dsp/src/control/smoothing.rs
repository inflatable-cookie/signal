use crate::ramp::{ExponentialRamp, LinearRamp};
use signal_primitives::Sample;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RampKind {
    Idle,
    Linear(LinearRamp),
    Exponential(ExponentialRamp),
}

/// Smoothed value that can transition between targets using linear or
/// exponential ramps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothedValue {
    current: Sample,
    target: Sample,
    ramp: RampKind,
}

impl SmoothedValue {
    /// Create a new smoothed value starting at the given initial value.
    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            ramp: RampKind::Idle,
        }
    }

    /// Reset to a specific value, clearing any active transition.
    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.ramp = RampKind::Idle;
    }

    /// Return the current value.
    pub fn current(&self) -> Sample {
        self.current
    }

    /// Return the target value.
    pub fn target(&self) -> Sample {
        self.target
    }

    /// Return true if a transition is in progress.
    pub fn is_smoothing(&self) -> bool {
        !matches!(self.ramp, RampKind::Idle)
    }

    /// Set the value immediately without smoothing.
    pub fn set_immediate(&mut self, value: Sample) {
        self.reset(value);
    }

    /// Set a linear ramp target over the given number of samples.
    pub fn set_linear_target(&mut self, target: Sample, samples: usize) {
        self.target = target;
        if samples == 0 {
            self.set_immediate(target);
            return;
        }

        let mut ramp = LinearRamp::new(self.current);
        ramp.set_target(target, samples);
        self.ramp = RampKind::Linear(ramp);
    }

    /// Set an exponential ramp target over the given number of samples.
    pub fn set_exponential_target(&mut self, target: Sample, samples: usize) {
        self.target = target.max(ExponentialRamp::MIN_MAGNITUDE);
        if samples == 0 {
            self.set_immediate(self.target);
            return;
        }

        let mut ramp = ExponentialRamp::new(self.current.max(ExponentialRamp::MIN_MAGNITUDE));
        ramp.set_target(self.target, samples);
        self.ramp = RampKind::Exponential(ramp);
    }

    /// Advance by one sample and return the new value.
    pub fn next_value(&mut self) -> Sample {
        self.current = match &mut self.ramp {
            RampKind::Idle => self.target,
            RampKind::Linear(ramp) => {
                let next = ramp.next_value();
                if !ramp.is_active() {
                    self.ramp = RampKind::Idle;
                }
                next
            }
            RampKind::Exponential(ramp) => {
                let next = ramp.next_value();
                if !ramp.is_active() {
                    self.ramp = RampKind::Idle;
                }
                next
            }
        };

        self.current
    }

    /// Fill a block of samples with smoothed values.
    pub fn fill_block(&mut self, block: &mut [Sample]) {
        for sample in block {
            *sample = self.next_value();
        }
    }
}
