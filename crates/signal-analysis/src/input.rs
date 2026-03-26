//! Audio input preparation for analysis stages.
//!
//! This module provides utilities for preparing audio buffers for analysis,
//! including channel reduction (mix-to-mono or first-channel), duration
//! trimming, and sample rate conversion.

use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{AudioBuffer, Sample, SampleRate, Seconds};

/// Shared mono reduction policy for analyzer input preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisChannelPolicy {
    /// Average all input channels into a mono stream.
    MixToMono,
    /// Use only the first channel of the input stream.
    FirstChannel,
}

/// Shared preprocessing configuration for offline or chunk-owned analyzers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalysisInputConfig {
    pub channel_policy: AnalysisChannelPolicy,
    pub max_duration: Option<Seconds>,
    pub target_sample_rate: Option<SampleRate>,
    pub resample_quality: ResampleQuality,
}

impl Default for AnalysisInputConfig {
    fn default() -> Self {
        Self {
            channel_policy: AnalysisChannelPolicy::MixToMono,
            max_duration: None,
            target_sample_rate: None,
            resample_quality: ResampleQuality::Linear,
        }
    }
}

/// Prepared mono analysis input after optional duration limiting and resampling.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedAnalysisBuffer {
    pub sample_rate: SampleRate,
    pub samples: Vec<Sample>,
}

/// Prepare an [`AudioBuffer`] for mono analyzer consumption.
pub fn prepare_audio_analysis(
    audio: &AudioBuffer,
    config: AnalysisInputConfig,
) -> PreparedAnalysisBuffer {
    let mono_samples = match config.channel_policy {
        AnalysisChannelPolicy::MixToMono => audio.to_mono(),
        AnalysisChannelPolicy::FirstChannel => first_channel_samples(audio),
    };

    prepare_mono_analysis(audio.sample_rate(), &mono_samples, config)
}

/// Prepare a mono slice for analyzer consumption.
pub fn prepare_mono_analysis(
    sample_rate: SampleRate,
    mono_samples: &[Sample],
    config: AnalysisInputConfig,
) -> PreparedAnalysisBuffer {
    let mut samples = mono_samples.to_vec();

    if let Some(max_duration) = config.max_duration {
        let max_frames = sample_rate.seconds_to_frames(max_duration).0;
        if max_frames > 0 && samples.len() > max_frames {
            let start = (samples.len() - max_frames) / 2;
            samples = samples[start..start + max_frames].to_vec();
        }
    }

    let output_rate = config.target_sample_rate.unwrap_or(sample_rate);
    if output_rate != sample_rate && !samples.is_empty() {
        samples = resample_mono(
            ResampleConfig::new(sample_rate, output_rate, config.resample_quality),
            &samples,
        );
    }

    PreparedAnalysisBuffer {
        sample_rate: output_rate,
        samples,
    }
}

fn first_channel_samples(audio: &AudioBuffer) -> Vec<Sample> {
    let channels = audio.channel_count().0;
    if channels == 0 || audio.is_empty() {
        return Vec::new();
    }

    if channels == 1 {
        return audio.samples().to_vec();
    }

    audio
        .samples()
        .chunks_exact(channels)
        .map(|frame| frame[0])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_primitives::{ChannelLayout, FrameCount};

    #[test]
    fn prepare_audio_analysis_mixes_channels_by_default() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );

        let prepared = prepare_audio_analysis(&audio, AnalysisInputConfig::default());

        assert_eq!(prepared.sample_rate, SampleRate(48_000));
        assert_eq!(prepared.samples, vec![0.0, 0.5]);
    }

    #[test]
    fn prepare_audio_analysis_can_take_first_channel() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );
        let config = AnalysisInputConfig {
            channel_policy: AnalysisChannelPolicy::FirstChannel,
            ..AnalysisInputConfig::default()
        };

        let prepared = prepare_audio_analysis(&audio, config);

        assert_eq!(prepared.samples, vec![1.0, 0.25]);
    }

    #[test]
    fn prepare_mono_analysis_center_trims_to_duration() {
        let samples: Vec<f32> = (0..10).map(|index| index as f32).collect();
        let prepared = prepare_mono_analysis(
            SampleRate(10),
            &samples,
            AnalysisInputConfig {
                max_duration: Some(Seconds(0.4)),
                ..AnalysisInputConfig::default()
            },
        );

        assert_eq!(prepared.samples, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn prepare_mono_analysis_resamples_to_target_rate() {
        let prepared = prepare_mono_analysis(
            SampleRate(8),
            &[0.0, 1.0, 0.0, -1.0],
            AnalysisInputConfig {
                target_sample_rate: Some(SampleRate(4)),
                ..AnalysisInputConfig::default()
            },
        );

        assert_eq!(prepared.sample_rate, SampleRate(4));
        assert_eq!(prepared.samples.len(), 2);
    }

    #[test]
    fn prepare_audio_analysis_handles_empty_buffers() {
        let audio = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Mono, FrameCount(0));
        let prepared = prepare_audio_analysis(&audio, AnalysisInputConfig::default());

        assert!(prepared.samples.is_empty());
    }
}
