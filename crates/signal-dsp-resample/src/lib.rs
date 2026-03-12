//! Deterministic mono resampling helpers for the Signal workspace.
//!
//! The crate exposes one pure-Rust resampling contract that both offline and
//! bounded chunked analyzers can share without open-coding sample-rate math.
//! Supported quality modes are deliberately explicit:
//! - [`ResampleQuality::Nearest`] for the cheapest deterministic stepping
//! - [`ResampleQuality::Linear`] for interpolation that preserves continuity
//!   across chunk boundaries
//!
//! The output is deterministic for a given input sample stream, chunking
//! pattern, and configuration.

use signal_primitives::{Sample, SampleRate};

/// Quality / cost trade-off for resampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResampleQuality {
    /// Fastest mode: nearest-neighbour lookup.
    Nearest,
    /// Linear interpolation between adjacent source samples.
    Linear,
}

/// Resampling configuration for one mono stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResampleConfig {
    pub input_rate: SampleRate,
    pub output_rate: SampleRate,
    pub quality: ResampleQuality,
}

impl ResampleConfig {
    pub fn new(input_rate: SampleRate, output_rate: SampleRate, quality: ResampleQuality) -> Self {
        Self {
            input_rate,
            output_rate,
            quality,
        }
    }
}

/// Stateful chunked mono resampler.
pub struct StreamingResampler {
    config: ResampleConfig,
    step: f64,
    pending: Vec<Sample>,
    next_source_index: f64,
}

impl StreamingResampler {
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

    pub fn config(&self) -> ResampleConfig {
        self.config
    }

    pub fn reset(&mut self) {
        self.pending.clear();
        self.next_source_index = 0.0;
    }

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

        let limit = if final_chunk {
            self.pending.len() as f64
        } else if self.pending.len() < 2 {
            return Vec::new();
        } else {
            self.pending.len() as f64 - 1.0
        };

        let mut output = Vec::new();
        while self.next_source_index < limit {
            output.push(sample_at(
                &self.pending,
                self.next_source_index,
                self.config.quality,
            ));
            self.next_source_index += self.step;
        }

        if final_chunk {
            return output;
        }

        let drain_up_to = self.next_source_index.floor() as usize;
        let drain_count = drain_up_to.saturating_sub(1).min(self.pending.len());
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

fn sample_at(samples: &[Sample], source_index: f64, quality: ResampleQuality) -> Sample {
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
}
