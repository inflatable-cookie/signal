//! Linear and exponential ramps for control signal smoothing.
//!
//! This module provides [`LinearRamp`] and [`ExponentialRamp`] for generating
//! smooth parameter transitions in DSP contexts.

use crate::flush_denormal;
use signal_primitives::Sample;

/// Linear interpolation ramp between two values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearRamp {
    current: Sample,
    target: Sample,
    step: Sample,
    remaining_samples: usize,
}

impl LinearRamp {
    /// Create a new ramp starting at the given value.
    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            step: 0.0,
            remaining_samples: 0,
        }
    }

    /// Reset the ramp to a specific value, clearing any active transition.
    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.remaining_samples = 0;
    }

    /// Set a new target value to reach over the given number of samples.
    pub fn set_target(&mut self, target: Sample, samples: usize) {
        self.target = target;
        if samples == 0 {
            self.reset(target);
            return;
        }

        self.remaining_samples = samples;
        self.step = (target - self.current) / samples as Sample;
    }

    /// Return the current value.
    pub fn current(&self) -> Sample {
        self.current
    }

    /// Return the target value.
    pub fn target(&self) -> Sample {
        self.target
    }

    /// Return true if the ramp is still transitioning.
    pub fn is_active(&self) -> bool {
        self.remaining_samples > 0
    }

    /// Advance the ramp by one sample and return the new value.
    pub fn next_value(&mut self) -> Sample {
        if self.remaining_samples == 0 {
            self.current = self.target;
            return self.target;
        }

        self.remaining_samples -= 1;
        if self.remaining_samples == 0 {
            self.current = self.target;
        } else {
            self.current = flush_denormal(self.current + self.step);
        }

        self.current
    }
}

/// Exponential interpolation ramp between two values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialRamp {
    current: Sample,
    target: Sample,
    multiplier: Sample,
    remaining_samples: usize,
}

impl ExponentialRamp {
    /// Minimum magnitude to avoid numerical issues near zero.
    pub const MIN_MAGNITUDE: Sample = 1.0e-6;

    /// Create a new ramp starting at the given value.
    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            multiplier: 1.0,
            remaining_samples: 0,
        }
    }

    /// Reset the ramp to a specific value, clearing any active transition.
    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.multiplier = 1.0;
        self.remaining_samples = 0;
    }

    /// Set a new target value to reach over the given number of samples.
    ///
    /// The ramp is magnitude-domain: an exponential trajectory cannot cross
    /// zero, so values are clamped to the positive domain
    /// (`MIN_MAGNITUDE..`). Negative inputs are rejected in debug builds and
    /// use their magnitude in release builds; use [`LinearRamp`] for signed
    /// trajectories.
    pub fn set_target(&mut self, target: Sample, samples: usize) {
        debug_assert!(
            target >= 0.0,
            "ExponentialRamp is magnitude-domain; got negative target {target}",
        );
        let current = self.current.abs().max(Self::MIN_MAGNITUDE);
        let target = target.abs().max(Self::MIN_MAGNITUDE);
        self.target = target;

        if samples == 0 {
            self.reset(target);
            return;
        }

        self.remaining_samples = samples;
        self.multiplier = (target / current).powf(1.0 / samples as Sample);
        if !self.multiplier.is_finite() || self.multiplier == 0.0 {
            self.multiplier = 1.0;
            self.remaining_samples = 0;
            self.current = target;
            self.target = target;
        }
    }

    /// Return the current value.
    pub fn current(&self) -> Sample {
        self.current
    }

    /// Return the target value.
    pub fn target(&self) -> Sample {
        self.target
    }

    /// Return true if the ramp is still transitioning.
    pub fn is_active(&self) -> bool {
        self.remaining_samples > 0
    }

    /// Advance the ramp by one sample and return the new value.
    pub fn next_value(&mut self) -> Sample {
        if self.remaining_samples == 0 {
            self.current = self.target;
            return self.target;
        }

        self.remaining_samples -= 1;
        if self.remaining_samples == 0 {
            self.current = self.target;
        } else {
            self.current = flush_denormal(self.current * self.multiplier);
        }

        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::{ExponentialRamp, LinearRamp};

    #[test]
    fn linear_ramp_hits_target_exactly() {
        let mut ramp = LinearRamp::new(0.0);
        ramp.set_target(1.0, 4);

        let values = [
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
        ];

        assert_eq!(values, [0.25, 0.5, 0.75, 1.0]);
        assert!(!ramp.is_active());
    }

    #[test]
    fn exponential_ramp_converges_monotonically() {
        let mut ramp = ExponentialRamp::new(1.0);
        ramp.set_target(16.0, 4);

        let values = [
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
        ];

        assert!(values[0] > 1.0);
        assert!(values[1] > values[0]);
        assert!(values[2] > values[1]);
        assert_eq!(values[3], 16.0);
    }
}
