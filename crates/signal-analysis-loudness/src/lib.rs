//! Loudness analysis surfaces for Signal.
//!
//! The crate currently exposes an offline loudness meter with integrated
//! loudness, trace, true-peak estimation, and confidence reporting.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
//! let result = meter.analyze(&audio);
//!
//! assert_eq!(meter.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.loudness_range_lu, 0.0);
//! ```

#![warn(missing_docs)]

use signal_analysis::{
    prepare_mono_analysis, AnalysisInputConfig, AnalysisMode, AnalysisStage, Confidence,
};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, Sample, SampleRate, Seconds};

mod analysis_math;
mod types;

use analysis_math::{
    aggregate_weighted_energies, apply_loudness_weighting, deinterleave_channels, dynamics_summary,
    empty_loudness_result, gated_integrated_loudness, loudness_channel_weights,
    loudness_confidence, loudness_range_from_energies, loudness_sample_rate_support,
    loudness_trace_from_energies, seconds_to_frames, trace_latest_loudness, trace_tail,
    true_peak_dbtp, true_peak_oversample_factor, window_mean_square,
};
pub use types::{
    LoudnessAggregationSummary, LoudnessAnalysisResult, LoudnessChannelSummary,
    LoudnessChannelWeightSource, LoudnessDynamicsSummary, LoudnessMeterConfig,
    LoudnessRuntimeDiagnosticsSummary, LoudnessSampleRateSupport, LoudnessTrace,
    LoudnessTracePoint,
};

const RUNTIME_MOMENTARY_TAIL_POINTS: usize = 8;
const RUNTIME_SHORT_TERM_TAIL_POINTS: usize = 4;

/// Offline mono loudness meter.
#[derive(Debug, Default)]
pub struct LoudnessMeter {
    config: LoudnessMeterConfig,
}

impl LoudnessMeter {
    /// Create a meter with the provided config.
    pub fn new(config: LoudnessMeterConfig) -> Self {
        Self { config }
    }

    /// Return the current loudness config.
    pub fn config(&self) -> LoudnessMeterConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> LoudnessAnalysisResult {
        self.analyze_channels(sample_rate, &[mono_samples.to_vec()], ChannelLayout::Mono)
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

    fn analyze_channels(
        &self,
        source_sample_rate: SampleRate,
        channels: &[Vec<Sample>],
        layout: ChannelLayout,
    ) -> LoudnessAnalysisResult {
        let channel_count = ChannelCount(channels.len());
        let (weights, channel_weight_source) = loudness_channel_weights(layout, channel_count);
        let analysis_sample_rate = self.config.analysis_sample_rate;
        let sample_rate_support =
            loudness_sample_rate_support(source_sample_rate, analysis_sample_rate);
        let true_peak_oversample_factor = true_peak_oversample_factor(analysis_sample_rate);
        let aggregation = LoudnessAggregationSummary {
            channel_count,
            channel_weight_source,
            sample_rate_support,
            analysis_sample_rate,
            true_peak_oversample_factor,
        };

        if channels.is_empty() || source_sample_rate.0 == 0 {
            return empty_loudness_result(aggregation);
        }

        let prepared_channels: Vec<Vec<Sample>> = channels
            .iter()
            .map(|channel| {
                prepare_mono_analysis(source_sample_rate, channel, self.analysis_input_config())
                    .samples
            })
            .collect();
        if prepared_channels.iter().all(Vec::is_empty) {
            return empty_loudness_result(aggregation);
        }

        let analysis_sample_rate = self.config.analysis_sample_rate;
        let block_size = seconds_to_frames(analysis_sample_rate, self.config.block_seconds).max(1);
        let hop_size = seconds_to_frames(analysis_sample_rate, self.config.hop_seconds).max(1);
        let short_term_size =
            seconds_to_frames(analysis_sample_rate, self.config.short_term_seconds).max(block_size);

        let weighted_channels: Vec<Vec<f32>> = prepared_channels
            .iter()
            .map(|channel| apply_loudness_weighting(analysis_sample_rate, channel))
            .collect();
        let block_energies_by_channel: Vec<Vec<f32>> = weighted_channels
            .iter()
            .map(|channel| window_mean_square(channel, block_size, hop_size))
            .collect();
        let short_term_energies_by_channel: Vec<Vec<f32>> = weighted_channels
            .iter()
            .map(|channel| window_mean_square(channel, short_term_size, hop_size))
            .collect();

        let aggregated_block_energies =
            aggregate_weighted_energies(&block_energies_by_channel, &weights);
        let aggregated_short_term_energies =
            aggregate_weighted_energies(&short_term_energies_by_channel, &weights);
        let momentary_trace = loudness_trace_from_energies(
            &aggregated_block_energies,
            self.config.block_seconds,
            self.config.hop_seconds,
        );
        let short_term_trace = loudness_trace_from_energies(
            &aggregated_short_term_energies,
            self.config.short_term_seconds,
            self.config.hop_seconds,
        );
        let channels: Vec<LoudnessChannelSummary> = prepared_channels
            .iter()
            .enumerate()
            .map(|(index, channel)| LoudnessChannelSummary {
                index,
                weight: *weights.get(index).unwrap_or(&1.0),
                integrated_lufs: gated_integrated_loudness(&block_energies_by_channel[index]),
                true_peak_dbtp: true_peak_dbtp(channel, true_peak_oversample_factor),
            })
            .collect();

        let integrated_lufs = gated_integrated_loudness(&aggregated_block_energies);
        let loudness_range_lu = loudness_range_from_energies(&aggregated_short_term_energies);
        let true_peak_dbtp = channels
            .iter()
            .map(|channel| channel.true_peak_dbtp)
            .fold(f32::NEG_INFINITY, f32::max);
        let dynamics = dynamics_summary(
            self.config.target_lufs,
            integrated_lufs,
            true_peak_dbtp,
            &momentary_trace,
            &short_term_trace,
        );
        let has_signal = weighted_channels
            .iter()
            .flatten()
            .any(|sample| sample.abs() > 0.0);
        let confidence = if has_signal {
            loudness_confidence(
                sample_rate_support,
                channel_weight_source,
                aggregated_block_energies.len(),
            )
        } else {
            Confidence::new(0.0)
        };

        LoudnessAnalysisResult {
            integrated_lufs,
            loudness_range_lu,
            true_peak_dbtp,
            confidence,
            channels,
            aggregation,
            momentary_trace,
            short_term_trace,
            dynamics,
        }
    }
}

impl AnalysisStage<LoudnessAnalysisResult> for LoudnessMeter {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> LoudnessAnalysisResult {
        let channels = deinterleave_channels(audio);
        self.analyze_channels(audio.sample_rate(), &channels, audio.channels())
    }
}

impl LoudnessAnalysisResult {
    /// Return the bounded loudness subset intended for runtime diagnostics.
    ///
    /// This keeps the reusable contract compact: top-line delivery metrics,
    /// current windowed state, and bounded recent trace tails rather than the
    /// full offline timeline.
    pub fn runtime_diagnostics_summary(&self) -> LoudnessRuntimeDiagnosticsSummary {
        let recent_momentary = trace_tail(&self.momentary_trace, RUNTIME_MOMENTARY_TAIL_POINTS);
        let recent_short_term = trace_tail(&self.short_term_trace, RUNTIME_SHORT_TERM_TAIL_POINTS);

        LoudnessRuntimeDiagnosticsSummary {
            integrated_lufs: self.integrated_lufs,
            true_peak_dbtp: self.true_peak_dbtp,
            target_offset_lu: self.dynamics.target_offset_lu,
            peak_to_loudness_lu: self.dynamics.peak_to_loudness_lu,
            current_momentary_lufs: trace_latest_loudness(&recent_momentary),
            current_short_term_lufs: trace_latest_loudness(&recent_short_term),
            momentary_max_lufs: self.dynamics.momentary_max_lufs,
            short_term_max_lufs: self.dynamics.short_term_max_lufs,
            recent_momentary,
            recent_short_term,
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
