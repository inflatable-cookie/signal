//! Reusable DSP kernels and control primitives for the Signal workspace.
//!
//! The crate now exposes a small realtime-safe kernel layer that higher-level
//! graph, runtime, and host code can share instead of rebuilding block-local
//! DSP helpers at every boundary.
//!
//! Current module groups:
//! - block-level helpers for applying sample-accurate control streams to gain,
//!   low-pass, and delay kernels
//! - control ramps and segment playback for block-local automation
//! - basic delay, filter, and level-tracking kernels
//! - a direct-form FIR convolution kernel ([`FirConvolver`]) for HRTF and
//!   short/medium impulse responses
//! - RBJ cookbook biquads ([`BiquadCoefficients`] + per-channel
//!   [`BiquadState`]) and a soft-knee master limiter ([`LimiterState`])
//! - stateless block mixing helpers and deterministic signal fixtures
//! - [`flush_denormal`] and [`flush_denormals_in_place`] helpers for clearing
//!   subnormal values from kernel state between blocks, plus the scoped
//!   hardware [`DenormalGuard`] for the audio callback
//!
//! ```no_run
//! use signal_dsp::{DspKernel, Gain, SmoothedValue};
//!
//! let mut gain = Gain::new(0.5);
//! let mut smoothing = SmoothedValue::new(0.0);
//! smoothing.set_linear_target(1.0, 4);
//!
//! let mut block = [0.8, -0.4, 0.2];
//! gain.process_in_place(&mut block);
//!
//! assert_eq!(block, [0.4, -0.2, 0.1]);
//! assert!(smoothing.next_value() > 0.0);
//! ```

#![warn(missing_docs)]

mod binaural;
mod biquad;
mod block;
mod control;
mod convolution;
mod delay;
mod denormal;
mod filter;
mod fixtures;
mod level;
mod limiter;
mod mix;
mod mix_matrix;
mod polyphase;
pub mod ramp;

pub use binaural::{BinauralConvolver, DEFAULT_HRIR_CROSSFADE_SAMPLES};
pub use biquad::{BiquadCoefficients, BiquadState};
pub use block::{
    apply_gain_control, process_delay_with_feedback_control, process_low_pass_with_cutoff_control,
};
pub use control::SmoothedValue;
pub use control::{ControlPlan, ControlSegment, ControlSegmentPlayer, ControlSegmentShape};
pub use convolution::FirConvolver;
pub use delay::DelayLine;
pub use denormal::DenormalGuard;
pub use filter::OnePoleLowPass;
pub use fixtures::SignalFixture;
pub use level::{EnvelopeFollower, PeakMeter, RmsMeter};
pub use limiter::{LimiterState, LIMITER_CEILING};
pub use mix::{apply_gain_in_place, clear_block, mix_in_place, sum_in_place, Gain};
pub use mix_matrix::{default_adapter_matrix, equal_power_pan_matrix};
pub use polyphase::PolyphaseInterpolationTable;
pub use ramp::{ExponentialRamp, LinearRamp};

use signal_primitives::Sample;

/// Magnitude threshold below which a sample value is flushed to zero between
/// blocks. Deliberately far above the f32 subnormal boundary (~1.2e-38): it
/// also clears vanishing-but-normal feedback tails that would otherwise decay
/// for thousands of blocks at inaudible levels.
pub const DENORMAL_THRESHOLD: Sample = 1.0e-20;

/// Common trait for reusable in-place DSP processors.
pub trait DspKernel {
    /// Reset any internal state carried across blocks.
    fn reset(&mut self);

    /// Control whether the kernel should stay transparent while preserving
    /// continuity rules for any state it carries.
    fn set_bypassed(&mut self, bypassed: bool);

    /// Report the current bypass flag for the kernel.
    fn is_bypassed(&self) -> bool;

    /// Process a mutable block of samples in place.
    fn process_in_place(&mut self, block: &mut [Sample]);
}

/// Flush a single subnormal value to zero so stateful kernels do not carry
/// denormals between blocks.
pub fn flush_denormal(sample: Sample) -> Sample {
    if sample.abs() < DENORMAL_THRESHOLD {
        0.0
    } else {
        sample
    }
}

/// Flush any subnormal values in a block to zero in place.
pub fn flush_denormals_in_place(block: &mut [Sample]) {
    for sample in block {
        *sample = flush_denormal(*sample);
    }
}

#[cfg(test)]
mod tests {
    use super::{flush_denormal, DENORMAL_THRESHOLD};

    #[test]
    fn flush_denormal_zeroes_subnormal_samples() {
        assert_eq!(flush_denormal(DENORMAL_THRESHOLD * 0.5), 0.0);
        assert_eq!(flush_denormal(0.125), 0.125);
    }
}
