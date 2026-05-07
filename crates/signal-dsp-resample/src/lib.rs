//! Deterministic mono resampling helpers for the Signal workspace.
//!
//! The crate exposes one pure-Rust resampling contract that both offline and
//! bounded chunked analyzers can share without open-coding sample-rate math.
//! Supported quality modes are deliberately explicit:
//! - [`ResampleQuality::Nearest`] for the cheapest deterministic stepping
//! - [`ResampleQuality::Linear`] for interpolation that preserves continuity
//!   across chunk boundaries
//! - [`ResampleQuality::BandLimited`] for a higher-quality windowed-sinc path
//!   that applies low-pass smoothing instead of interpolation alone
//!
//! The output is deterministic for a given input sample stream, chunking
//! pattern, and configuration.

#![warn(missing_docs)]

use signal_primitives::{Sample, SampleRate};

/// Quality / cost trade-off for resampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResampleQuality {
    /// Fastest mode: nearest-neighbour lookup.
    Nearest,
    /// Linear interpolation between adjacent source samples.
    Linear,
    /// Windowed-sinc interpolation with a cutoff scaled for downsampling.
    BandLimited,
}

/// Resampling configuration for one mono stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResampleConfig {
    /// Sample rate of the input signal.
    pub input_rate: SampleRate,
    /// Desired sample rate of the output signal.
    pub output_rate: SampleRate,
    /// Quality level for the resampling kernel.
    pub quality: ResampleQuality,
}

impl ResampleConfig {
    /// Construct a `ResampleConfig` from the given rates and quality level.
    pub fn new(input_rate: SampleRate, output_rate: SampleRate, quality: ResampleQuality) -> Self {
        Self {
            input_rate,
            output_rate,
            quality,
        }
    }
}

/// Summary metrics for one resampled output stream.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResampleArtifactMetrics {
    /// Number of samples in the resampled output.
    pub output_len: usize,
    /// Absolute peak amplitude of the output.
    pub peak_amplitude: f32,
    /// Root-mean-square amplitude of the output.
    pub rms_amplitude: f32,
    /// Mean absolute difference between adjacent output samples.
    pub mean_abs_step: f32,
    /// Mean squared difference between adjacent output samples.
    pub step_energy: f32,
}

/// Frozen comparison surface for the crate's three quality tiers.
#[derive(Clone, Debug, PartialEq)]
pub struct ResampleQualityComparisonReport {
    /// Sample rate of the input signal used for the comparison.
    pub input_rate: SampleRate,
    /// Target sample rate used for the comparison.
    pub output_rate: SampleRate,
    /// Number of input samples used for the comparison.
    pub input_len: usize,
    /// Artifact metrics for the [`ResampleQuality::Nearest`] tier.
    pub nearest: ResampleArtifactMetrics,
    /// Artifact metrics for the [`ResampleQuality::Linear`] tier.
    pub linear: ResampleArtifactMetrics,
    /// Artifact metrics for the [`ResampleQuality::BandLimited`] tier.
    pub band_limited: ResampleArtifactMetrics,
}

/// Stateful chunked mono resampler.
pub struct StreamingResampler {
    config: ResampleConfig,
    step: f64,
    pending: Vec<Sample>,
    next_source_index: f64,
}

impl StreamingResampler {
    /// Create a new `StreamingResampler` with the given configuration.
    pub fn new(config: ResampleConfig) -> Self {
        let step = if config.input_rate.0 == 0 || config.output_rate.0 == 0 {
            0.0
        } else {
            config.input_rate.0 as f64 / config.output_rate.0 as f64
        };

        Self {
            config,
            step,
            pending: Vec::new(),
            next_source_index: 0.0,
        }
    }

    /// Return the current resampling configuration.
    pub fn config(&self) -> ResampleConfig {
        self.config
    }

    /// Reset internal state, discarding any buffered samples.
    pub fn reset(&mut self) {
        self.pending.clear();
        self.next_source_index = 0.0;
    }

    /// Feed one chunk of input samples and return all newly available output samples.
    pub fn process_chunk(&mut self, input: &[Sample]) -> Vec<Sample> {
        if input.is_empty() || self.config.input_rate.0 == 0 || self.config.output_rate.0 == 0 {
            return Vec::new();
        }

        if self.config.input_rate == self.config.output_rate {
            return input.to_vec();
        }

        self.pending.extend_from_slice(input);
        self.drain_available(false)
    }

    /// Flush remaining buffered samples and return any final output samples.
    ///
    /// Resets internal state. Call once after the last [`process_chunk`](Self::process_chunk).
    pub fn finish(&mut self) -> Vec<Sample> {
        if self.config.input_rate.0 == 0 || self.config.output_rate.0 == 0 {
            self.reset();
            return Vec::new();
        }

        if self.config.input_rate == self.config.output_rate {
            self.reset();
            return Vec::new();
        }

        let output = self.drain_available(true);
        self.reset();
        output
    }

    fn drain_available(&mut self, final_chunk: bool) -> Vec<Sample> {
        if self.pending.is_empty() || self.step <= 0.0 {
            return Vec::new();
        }

        let radius = sample_radius(self.config.quality);
        let limit = if final_chunk {
            self.pending.len() as f64
        } else {
            let Some(limit) = required_nonfinal_limit(self.config.quality, self.pending.len())
            else {
                return Vec::new();
            };
            limit
        };

        let mut output = Vec::new();
        while self.next_source_index < limit {
            output.push(sample_at_with_step(
                &self.pending,
                self.next_source_index,
                self.step,
                self.config.quality,
            ));
            self.next_source_index += self.step;
        }

        if final_chunk {
            return output;
        }

        let drain_up_to = self.next_source_index.floor() as usize;
        let drain_count = drain_up_to
            .saturating_sub(radius)
            .min(self.pending.len().saturating_sub(radius));
        if drain_count > 0 {
            self.pending.drain(..drain_count);
            self.next_source_index -= drain_count as f64;
        }

        output
    }
}

/// Resample a mono slice in one offline call.
pub fn resample_mono(config: ResampleConfig, input: &[Sample]) -> Vec<Sample> {
    if input.is_empty() || config.input_rate.0 == 0 || config.output_rate.0 == 0 {
        return Vec::new();
    }

    if config.input_rate == config.output_rate {
        return input.to_vec();
    }

    let mut resampler = StreamingResampler::new(config);
    let mut output = resampler.process_chunk(input);
    output.extend(resampler.finish());
    output
}

/// Compare the artifact posture of the crate's quality tiers for one mono
/// input and rate conversion pair.
pub fn compare_quality_tiers(
    input_rate: SampleRate,
    output_rate: SampleRate,
    input: &[Sample],
) -> ResampleQualityComparisonReport {
    let nearest = resample_mono(
        ResampleConfig::new(input_rate, output_rate, ResampleQuality::Nearest),
        input,
    );
    let linear = resample_mono(
        ResampleConfig::new(input_rate, output_rate, ResampleQuality::Linear),
        input,
    );
    let band_limited = resample_mono(
        ResampleConfig::new(input_rate, output_rate, ResampleQuality::BandLimited),
        input,
    );

    ResampleQualityComparisonReport {
        input_rate,
        output_rate,
        input_len: input.len(),
        nearest: artifact_metrics(&nearest),
        linear: artifact_metrics(&linear),
        band_limited: artifact_metrics(&band_limited),
    }
}

fn sample_at_with_step(
    samples: &[Sample],
    source_index: f64,
    step: f64,
    quality: ResampleQuality,
) -> Sample {
    let left_index = source_index.floor() as usize;
    let right_index = (left_index + 1).min(samples.len().saturating_sub(1));
    let left = samples[left_index];
    let right = samples[right_index];

    match quality {
        ResampleQuality::Nearest => {
            if source_index - left_index as f64 >= 0.5 {
                right
            } else {
                left
            }
        }
        ResampleQuality::Linear => {
            let fraction = (source_index - left_index as f64) as f32;
            left + (right - left) * fraction
        }
        ResampleQuality::BandLimited => band_limited_sample(samples, source_index, step),
    }
}

fn artifact_metrics(samples: &[Sample]) -> ResampleArtifactMetrics {
    let peak_amplitude = samples
        .iter()
        .copied()
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    let rms_amplitude = if samples.is_empty() {
        0.0
    } else {
        let sum = samples
            .iter()
            .copied()
            .map(|sample| sample * sample)
            .sum::<f32>();
        (sum / samples.len() as f32).sqrt()
    };
    let step_count = samples.len().saturating_sub(1);
    let (mean_abs_step, step_energy) = if step_count == 0 {
        (0.0, 0.0)
    } else {
        let mut abs_sum = 0.0f32;
        let mut energy_sum = 0.0f32;
        for pair in samples.windows(2) {
            let step = pair[1] - pair[0];
            abs_sum += step.abs();
            energy_sum += step * step;
        }
        (abs_sum / step_count as f32, energy_sum / step_count as f32)
    };

    ResampleArtifactMetrics {
        output_len: samples.len(),
        peak_amplitude,
        rms_amplitude,
        mean_abs_step,
        step_energy,
    }
}

fn sample_radius(quality: ResampleQuality) -> usize {
    match quality {
        ResampleQuality::Nearest | ResampleQuality::Linear => 1,
        ResampleQuality::BandLimited => 8,
    }
}

fn required_nonfinal_limit(quality: ResampleQuality, pending_len: usize) -> Option<f64> {
    match quality {
        ResampleQuality::Nearest | ResampleQuality::Linear => {
            if pending_len < 2 {
                None
            } else {
                Some(pending_len as f64 - 1.0)
            }
        }
        ResampleQuality::BandLimited => {
            let radius = sample_radius(quality);
            if pending_len <= radius * 2 {
                None
            } else {
                Some((pending_len - radius) as f64)
            }
        }
    }
}

fn band_limited_sample(samples: &[Sample], source_index: f64, step: f64) -> Sample {
    let radius = sample_radius(ResampleQuality::BandLimited) as isize;
    let center = source_index.floor() as isize;
    let normalized_cutoff = if step > 1.0 { 1.0 / step } else { 1.0 };
    let mut weighted_sum = 0.0f64;
    let mut weight_sum = 0.0f64;

    for tap in -radius + 1..=radius {
        let sample_index = center + tap;
        let Some(sample) = clamped_sample(samples, sample_index) else {
            continue;
        };
        let distance = source_index - sample_index as f64;
        let weight = sinc(distance * normalized_cutoff)
            * hann_window(distance / radius as f64)
            * normalized_cutoff;
        weighted_sum += sample as f64 * weight;
        weight_sum += weight;
    }

    if weight_sum.abs() < 1.0e-9 {
        0.0
    } else {
        (weighted_sum / weight_sum) as Sample
    }
}

fn clamped_sample(samples: &[Sample], index: isize) -> Option<Sample> {
    if samples.is_empty() {
        return None;
    }
    let clamped = index.clamp(0, samples.len() as isize - 1) as usize;
    Some(samples[clamped])
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1.0e-12 {
        1.0
    } else {
        let angle = core::f64::consts::PI * x;
        angle.sin() / angle
    }
}

fn hann_window(normalized_distance: f64) -> f64 {
    let abs = normalized_distance.abs();
    if abs >= 1.0 {
        0.0
    } else {
        0.5 * (1.0 + (core::f64::consts::PI * abs).cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_identity_passthrough_preserves_samples() {
        let samples = vec![0.0, 0.25, -0.5, 1.0];
        let output = resample_mono(
            ResampleConfig::new(
                SampleRate(48_000),
                SampleRate(48_000),
                ResampleQuality::Linear,
            ),
            &samples,
        );

        assert_eq!(output, samples);
    }

    #[test]
    fn offline_linear_upsample_interpolates_between_samples() {
        let output = resample_mono(
            ResampleConfig::new(SampleRate(4), SampleRate(8), ResampleQuality::Linear),
            &[0.0, 1.0, 0.0, -1.0],
        );

        assert_eq!(output.len(), 8);
        assert!((output[1] - 0.5).abs() < 1.0e-6, "sample was {}", output[1]);
        assert!((output[3] - 0.5).abs() < 1.0e-6, "sample was {}", output[3]);
        assert!((output[7] + 1.0).abs() < 1.0e-6, "sample was {}", output[7]);
    }

    #[test]
    fn offline_linear_downsample_reduces_output_length() {
        let output = resample_mono(
            ResampleConfig::new(SampleRate(8), SampleRate(4), ResampleQuality::Linear),
            &[0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25],
        );

        assert_eq!(output.len(), 4);
        assert!((output[0] - 0.0).abs() < 1.0e-6);
        assert!((output[1] - 0.5).abs() < 1.0e-6);
        assert!((output[2] - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn streaming_and_offline_linear_outputs_match() {
        let config = ResampleConfig::new(
            SampleRate(48_000),
            SampleRate(16_000),
            ResampleQuality::Linear,
        );
        let input: Vec<f32> = (0..97)
            .map(|index| (index as f32 * 0.03125).sin())
            .collect();

        let offline = resample_mono(config, &input);

        let mut streaming = StreamingResampler::new(config);
        let mut chunked = Vec::new();
        chunked.extend(streaming.process_chunk(&input[..13]));
        chunked.extend(streaming.process_chunk(&input[13..41]));
        chunked.extend(streaming.process_chunk(&input[41..64]));
        chunked.extend(streaming.process_chunk(&input[64..]));
        chunked.extend(streaming.finish());

        assert_eq!(chunked.len(), offline.len());
        for (index, (lhs, rhs)) in chunked.iter().zip(offline.iter()).enumerate() {
            assert!(
                (lhs - rhs).abs() < 1.0e-6,
                "mismatch at {index}: {lhs} vs {rhs}"
            );
        }
    }

    #[test]
    fn nearest_quality_snaps_to_source_samples() {
        let output = resample_mono(
            ResampleConfig::new(SampleRate(4), SampleRate(8), ResampleQuality::Nearest),
            &[0.0, 1.0, 0.0, -1.0],
        );

        assert_eq!(output, vec![0.0, 1.0, 1.0, 0.0, 0.0, -1.0, -1.0, -1.0]);
    }

    #[test]
    fn streaming_and_offline_band_limited_outputs_match() {
        let config = ResampleConfig::new(
            SampleRate(48_000),
            SampleRate(12_000),
            ResampleQuality::BandLimited,
        );
        let input: Vec<f32> = (0..193)
            .map(|index| ((index as f32 * 0.19).sin() * 0.7) + ((index as f32 * 1.7).sin() * 0.3))
            .collect();

        let offline = resample_mono(config, &input);

        let mut streaming = StreamingResampler::new(config);
        let mut chunked = Vec::new();
        chunked.extend(streaming.process_chunk(&input[..31]));
        chunked.extend(streaming.process_chunk(&input[31..79]));
        chunked.extend(streaming.process_chunk(&input[79..141]));
        chunked.extend(streaming.process_chunk(&input[141..]));
        chunked.extend(streaming.finish());

        assert_eq!(chunked.len(), offline.len());
        for (index, (lhs, rhs)) in chunked.iter().zip(offline.iter()).enumerate() {
            assert!(
                (lhs - rhs).abs() < 1.0e-5,
                "band-limited mismatch at {index}: {lhs} vs {rhs}"
            );
        }
    }

    #[test]
    fn band_limited_quality_attenuates_alias_prone_downsampling_content() {
        let input: Vec<f32> = (0..128)
            .map(|index| if index % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let report = compare_quality_tiers(SampleRate(48_000), SampleRate(12_000), &input);

        assert!(
            report.band_limited.peak_amplitude < report.linear.peak_amplitude * 0.6,
            "expected band-limited attenuation, got linear_peak={} band_limited_peak={}",
            report.linear.peak_amplitude,
            report.band_limited.peak_amplitude
        );
        assert!(
            report.band_limited.rms_amplitude < report.linear.rms_amplitude * 0.6,
            "expected band-limited rms attenuation, got linear={} band_limited={}",
            report.linear.rms_amplitude,
            report.band_limited.rms_amplitude
        );
    }

    #[test]
    fn quality_comparison_report_is_stable_and_machine_readable() {
        let input: Vec<f32> = (0..160)
            .map(|index| {
                ((index as f32 * 0.13).sin() * 0.65)
                    + ((index as f32 * 0.79).sin() * 0.25)
                    + ((index as f32 * 1.83).sin() * 0.10)
            })
            .collect();
        let report = compare_quality_tiers(SampleRate(48_000), SampleRate(16_000), &input);

        println!("resample_quality_report={report:#?}");

        assert_eq!(report.input_rate, SampleRate(48_000));
        assert_eq!(report.output_rate, SampleRate(16_000));
        assert_eq!(report.input_len, 160);
        assert_eq!(report.nearest.output_len, report.linear.output_len);
        assert_eq!(report.linear.output_len, report.band_limited.output_len);
        assert!(report.band_limited.step_energy <= report.linear.step_energy);
        assert!(report.linear.step_energy <= report.nearest.step_energy);
    }
}
