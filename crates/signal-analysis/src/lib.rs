//! Shared analysis traits, result types, and confidence models for Signal.

use signal_primitives::AudioBuffer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisMode {
    Offline,
    Streaming,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

pub trait AnalysisStage<Output> {
    fn mode(&self) -> AnalysisMode;
    fn analyze(&mut self, audio: &AudioBuffer) -> Output;
}
