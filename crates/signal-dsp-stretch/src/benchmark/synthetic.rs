use signal_primitives::Sample;

use crate::phase_vocoder::transient_reset_phase_vocoder_linked_stereo;
use crate::{stretch_dynamic_ratio_mono_with_engine, StretchRatioPoint};

use super::types::{
    StretchCorpusCase, StretchCorpusFamily, StretchCorpusSource, STRETCH_BENCHMARK_CORPUS,
};

/// Inline synthetic audio generated for stretch benchmark bootstrap cases.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchSyntheticAudio {
    /// Sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Interleaved channel count.
    pub channels: u16,
    /// Interleaved sample frames.
    pub samples: Vec<Sample>,
}

impl StretchSyntheticAudio {
    /// Number of sample frames.
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels.max(1) as usize
    }
}

/// Generate the synthetic benchmark audio for a corpus family.
pub fn generate_synthetic_stretch_audio(
    family: StretchCorpusFamily,
) -> Option<StretchSyntheticAudio> {
    match family {
        StretchCorpusFamily::TempoRamp => Some(synthetic_tempo_ramp()),
        StretchCorpusFamily::LoopSeam => Some(synthetic_loop_seam()),
        StretchCorpusFamily::ExtremeRatio => Some(synthetic_extreme_ratio()),
        _ => None,
    }
}

/// Generate all inline synthetic benchmark cases declared in the corpus
/// blueprint.
pub fn synthetic_stretch_corpus_cases() -> Vec<(StretchCorpusCase, StretchSyntheticAudio)> {
    STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter_map(|case| {
            if case.source == StretchCorpusSource::Synthetic {
                generate_synthetic_stretch_audio(case.family).map(|audio| (*case, audio))
            } else {
                None
            }
        })
        .collect()
}
pub(super) fn synthetic_tempo_ramp_ratio_curve(input_frames: usize) -> Vec<StretchRatioPoint> {
    vec![
        StretchRatioPoint::new(0, 0.75),
        StretchRatioPoint::new((input_frames / 3) as i64, 1.0),
        StretchRatioPoint::new((input_frames * 2 / 3) as i64, 1.5),
    ]
}

pub(super) fn synthetic_tempo_ramp() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let progress = frame as f32 / FRAMES as f32;
        let frequency = 220.0 + 220.0 * progress;
        let carrier = (std::f32::consts::TAU * frequency * frame as f32 / SAMPLE_RATE as f32).sin();
        let pulse = if frame % 12_000 < 96 { 0.7 } else { 0.0 };
        let sample = (carrier * 0.25 + pulse) * (1.0 - 0.25 * progress);
        samples.push(sample);
        samples.push(sample);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

pub(super) fn synthetic_loop_seam() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let phase = frame as f32 / FRAMES as f32;
        let body = (std::f32::consts::TAU * 110.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.2;
        let boundary_probe = if !(128..FRAMES - 128).contains(&frame) {
            0.8 * (1.0 - frame.min(FRAMES - 1 - frame) as f32 / 128.0)
        } else {
            0.0
        };
        let left = body + boundary_probe;
        let right = body * (0.95 + 0.05 * phase) + boundary_probe;
        samples.push(left);
        samples.push(right);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

pub(super) fn synthetic_extreme_ratio() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
        let tonal =
            (std::f32::consts::TAU * 330.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.25;
        let transient = if frame % 8_000 < 64 {
            0.9 * (1.0 - (frame % 8_000) as f32 / 64.0)
        } else {
            0.0
        };
        samples.push(tonal + transient);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 1,
        samples,
    }
}

pub(super) fn synthetic_pitch_shift_tone(
    source_frequency_hz: f64,
    sample_rate_hz: u32,
    frames: usize,
) -> Vec<Sample> {
    (0..frames)
        .map(|frame| {
            let time = frame as f64 / sample_rate_hz as f64;
            let fade_in = (frame as f32 / 1_024.0).min(1.0);
            let fade_out = ((frames - 1 - frame) as f32 / 1_024.0).min(1.0);
            let fade = fade_in.min(fade_out);
            (std::f64::consts::TAU * source_frequency_hz * time).sin() as f32 * 0.7 * fade
        })
        .collect()
}

pub(super) fn synthetic_sustained_material() -> Vec<Sample> {
    const SAMPLE_RATE: usize = 48_000;
    const FRAMES: usize = SAMPLE_RATE * 2;
    const FADE_FRAMES: usize = 1_024;
    let bin_frequency = SAMPLE_RATE as f32 / 2048.0;
    let partials = [
        (9.0 * bin_frequency, 0.38),
        (17.0 * bin_frequency, 0.24),
        (29.0 * bin_frequency, 0.16),
        (43.0 * bin_frequency, 0.10),
    ];

    (0..FRAMES)
        .map(|frame| {
            let time = frame as f32 / SAMPLE_RATE as f32;
            let fade_in = (frame as f32 / FADE_FRAMES as f32).min(1.0);
            let fade_out = ((FRAMES - 1 - frame) as f32 / FADE_FRAMES as f32).min(1.0);
            let fade = fade_in.min(fade_out);
            let motion = 0.78 + 0.12 * (std::f32::consts::TAU * 0.35 * time).sin();
            partials
                .iter()
                .map(|(frequency, gain)| gain * (std::f32::consts::TAU * frequency * time).sin())
                .sum::<f32>()
                * motion
                * fade
        })
        .collect()
}
pub(super) fn stretch_stereo_synthetic(
    input: &StretchSyntheticAudio,
    ratio: f64,
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let channel_count = input.channels as usize;
    if channel_count != 2 {
        return Vec::new();
    }
    let target_len = (input.frame_count() as f64 * ratio).round() as usize;
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(stretcher(&mono, target_len, ratio, 2_048, 512));
    }
    interleave_channels(&output_channels)
}

pub(super) fn stretch_dynamic_ratio_stereo_independent(
    input: &StretchSyntheticAudio,
    ratio_curve: &[StretchRatioPoint],
    stretcher: fn(&[Sample], usize, f64, usize, usize) -> Vec<Sample>,
) -> Vec<Sample> {
    let channel_count = input.channels as usize;
    if channel_count != 2 {
        return Vec::new();
    }
    let mut output_channels = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        let mono = deinterleave_channel(&input.samples, channel_count, channel);
        output_channels.push(
            stretch_dynamic_ratio_mono_with_engine(&mono, ratio_curve, 1.0, 2_048, 512, stretcher)
                .expect("corpus render fits the offline output bound"),
        );
    }
    interleave_channels(&output_channels)
}

pub(super) fn stretch_stereo_synthetic_linked(
    input: &StretchSyntheticAudio,
    ratio: f64,
) -> Vec<Sample> {
    if input.channels != 2 {
        return Vec::new();
    }
    let target_len = (input.frame_count() as f64 * ratio).round() as usize;
    transient_reset_phase_vocoder_linked_stereo(&input.samples, target_len, ratio, 2_048, 512)
}

pub(super) fn deinterleave_channel(
    samples: &[Sample],
    channels: usize,
    channel: usize,
) -> Vec<Sample> {
    samples
        .chunks_exact(channels)
        .map(|frame| frame[channel])
        .collect()
}

pub(super) fn interleave_channels(channels: &[Vec<Sample>]) -> Vec<Sample> {
    let Some(first) = channels.first() else {
        return Vec::new();
    };
    let frames = channels.iter().map(Vec::len).min().unwrap_or(first.len());
    let mut output = Vec::with_capacity(frames * channels.len());
    for frame in 0..frames {
        for channel in channels {
            output.push(channel[frame]);
        }
    }
    output
}
