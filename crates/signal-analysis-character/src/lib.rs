//! Character analysis descriptor packs for Signal.
//!
//! The crate computes reusable offline descriptor packs from mono audio:
//! spectral shape, spectral contrast, coarse mel-profile shape, temporal
//! activity, and amplitude-domain dynamics.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
//! let result = analyzer.analyze(&audio);
//!
//! assert_eq!(analyzer.mode(), signal_analysis::AnalysisMode::Offline);
//! assert!(result.dynamics.rms_energy >= 0.0 && result.dynamics.rms_energy <= 1.0);
//! ```

use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode, AnalysisStage,
};
use signal_dsp_spectral::Stft;
use signal_primitives::{AudioBuffer, Sample, SampleRate, Seconds};

mod spectral;
mod stats;
mod temporal;
mod types;

use temporal::{
    character_confidence, compute_dynamics_pack, compute_event_density, compute_sustain_ratio,
    compute_temporal_shape_pack, compute_transient_density, compute_zcr, detect_peak_indices,
    frame_rms_envelope, spectral_flux_envelope,
};
pub use types::{
    CharacterAnalysisResult, CharacterAnalyzerConfig, CharacterDescriptorReductionPolicy,
    DescriptorReduction, DynamicsDescriptorPack, SpectralContrastDescriptorPack,
    SpectralProfileDescriptorPack, SpectralShapeDescriptorPack, TemporalDescriptorPack,
    TemporalShapeDescriptorPack,
};

/// Offline character analyzer.
#[derive(Debug, Default)]
pub struct CharacterAnalyzer {
    config: CharacterAnalyzerConfig,
}

impl CharacterAnalyzer {
    /// Create a character analyzer with the provided config.
    pub fn new(config: CharacterAnalyzerConfig) -> Self {
        Self { config }
    }

    /// Return the current analyzer config.
    pub fn config(&self) -> CharacterAnalyzerConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> CharacterAnalysisResult {
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
    ) -> CharacterAnalysisResult {
        if mono_samples.is_empty() || sample_rate.0 == 0 {
            return CharacterAnalysisResult::zero();
        }

        let duration_seconds = mono_samples.len() as f32 / sample_rate.0 as f32;
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let spectral_flux = spectral_flux_envelope(&spectrogram);
        let transient_peak_indices =
            detect_peak_indices(&spectral_flux, self.config.onset_threshold);
        let frame_envelope = frame_rms_envelope(mono_samples, self.config.stft);

        let temporal = TemporalDescriptorPack {
            onset_density: compute_event_density(transient_peak_indices.len(), duration_seconds),
            zero_crossing_rate_hz: compute_zcr(mono_samples, sample_rate),
            transient_density: compute_transient_density(
                mono_samples,
                sample_rate,
                duration_seconds,
            ),
            sustain_ratio: compute_sustain_ratio(mono_samples),
        };
        let temporal_shape = compute_temporal_shape_pack(
            sample_rate,
            self.config.stft,
            &spectral_flux,
            &frame_envelope,
            &transient_peak_indices,
        );

        CharacterAnalysisResult {
            spectral_shape: spectral::compute_spectral_shape_pack(&spectrogram),
            spectral_contrast: spectral::compute_spectral_contrast_pack(&spectrogram),
            spectral_profile: spectral::compute_spectral_profile_pack(&spectrogram),
            temporal,
            temporal_shape,
            dynamics: compute_dynamics_pack(mono_samples),
            reduction_policy: CharacterDescriptorReductionPolicy::default(),
            confidence: character_confidence(sample_rate, mono_samples.len()),
        }
    }
}

impl AnalysisStage<CharacterAnalysisResult> for CharacterAnalyzer {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> CharacterAnalysisResult {
        let prepared = prepare_audio_analysis(audio, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
