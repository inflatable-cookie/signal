//! Shared analysis traits, result types, and confidence models for Signal.
//!
//! Analysis crates in the workspace implement [`AnalysisStage`] and return
//! confidence-scored result types through this shared contract layer.
//!
//! ```no_run
//! use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
//! use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};
//!
//! struct EmptyStage;
//!
//! impl AnalysisStage<Confidence> for EmptyStage {
//!     fn mode(&self) -> AnalysisMode {
//!         AnalysisMode::Offline
//!     }
//!
//!     fn analyze(&mut self, _audio: &AudioBuffer) -> Confidence {
//!         Confidence::new(0.25)
//!     }
//! }
//!
//! let audio = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Mono, FrameCount(128));
//! let mut stage = EmptyStage;
//! assert_eq!(stage.analyze(&audio), Confidence::new(0.25));
//! ```

use signal_primitives::AudioBuffer;

/// Execution mode for an analysis stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisMode {
    Offline,
    Streaming,
}

/// Confidence score normalized to the inclusive range `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    /// Construct a confidence value, clamping it into `0.0..=1.0`.
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

/// Shared trait for analysis stages that consume an [`AudioBuffer`].
pub trait AnalysisStage<Output> {
    /// Report whether the stage is intended for offline or streaming use.
    fn mode(&self) -> AnalysisMode;

    /// Analyze an audio buffer and return the stage-specific output.
    fn analyze(&mut self, audio: &AudioBuffer) -> Output;
}
