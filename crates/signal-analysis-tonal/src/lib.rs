//! Tonal analysis surfaces for Signal.
//!
//! The crate currently exposes offline whole-track key detection driven by
//! STFT-based chroma accumulation.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_tonal::{KeyDetector, KeyDetectorConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut detector = KeyDetector::new(KeyDetectorConfig::default());
//! let result = detector.analyze(&audio);
//!
//! assert_eq!(detector.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.chroma.len(), 12);
//! ```

#![warn(missing_docs)]


use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode, AnalysisStage,
};
use signal_dsp_spectral::Stft;
use signal_primitives::{AudioBuffer, Sample, SampleRate, Seconds};

mod local_tracking;
mod profile_scoring;
mod tuning;
mod types;

use local_tracking::analyze_local_tonal_tracking;
use profile_scoring::score_chroma;
use tuning::estimate_tuning;

pub use types::{
    AnalysisProfile, HarmonicChangeKind, HarmonicChangeSummary, Key, KeyDetectorConfig, KeyMode,
    KeyProfile, LocalTonalAmbiguitySummary, LocalTonalTrackingSummary, TonalAmbiguityKind,
    TonalAnalysisResult, TonalProfileCandidate, TonalScoringSummary, TonalSegmentAmbiguitySummary,
    TonalSegmentSummary, Tonic, TuningCandidate, TuningEstimate, TuningReferenceMode,
    TuningReferenceSource,
};

/// Offline detector for global key and chroma summaries.
#[derive(Debug, Default)]
pub struct KeyDetector {
    config: KeyDetectorConfig,
}

impl KeyDetector {
    /// Create a key detector with the provided config.
    pub fn new(config: KeyDetectorConfig) -> Self {
        Self { config }
    }

    /// Return the current detector config.
    pub fn config(&self) -> KeyDetectorConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        let prepared =
            prepare_mono_analysis(sample_rate, mono_samples, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }

    fn analysis_input_config(&self) -> AnalysisInputConfig {
        AnalysisInputConfig {
            max_duration: self
                .config
                .analysis_duration_seconds
                .map(|seconds| Seconds(seconds as f32)),
            target_sample_rate: Some(self.config.analysis_sample_rate),
            ..AnalysisInputConfig::default()
        }
    }

    fn analyze_prepared(
        &self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let tuning = estimate_tuning(&spectrogram, self.config);
        let chroma = spectrogram.chroma_with_reference(tuning.reference_hz);
        let local_tracking =
            analyze_local_tonal_tracking(&spectrogram, self.config, tuning.reference_hz);

        score_chroma(chroma, self.config.profile)
            .with_tuning(tuning)
            .with_local_tracking(local_tracking)
    }
}

impl AnalysisStage<TonalAnalysisResult> for KeyDetector {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> TonalAnalysisResult {
        let prepared = prepare_audio_analysis(audio, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
