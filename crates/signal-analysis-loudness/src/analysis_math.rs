use signal_primitives::{ChannelCount, ChannelLayout, SampleRate};

use crate::types::{LoudnessChannelWeightSource, LoudnessSampleRateSupport};

mod aggregation;
mod trace_support;

pub(crate) use aggregation::{
    aggregate_weighted_energies, deinterleave_channels, empty_loudness_result,
    gated_integrated_loudness, loudness_confidence, loudness_range_from_energies, true_peak_dbtp,
};
pub(crate) use trace_support::{
    dynamics_summary, loudness_trace_from_energies, trace_latest_loudness, trace_tail,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, coeffs: BiquadCoefficients, input: f32) -> f32 {
        let output = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

pub(crate) const PRE_FILTER_48K: BiquadCoefficients = BiquadCoefficients {
    b0: 1.5351249,
    b1: -2.6916962,
    b2: 1.1983929,
    a1: -1.6906593,
    a2: 0.73248076,
};

// ITU-R BS.1770-4 stage-2 high-pass filter at 48 kHz.
// Previous coefficients (b0=1.005, b1=-1.990, b2=0.985) were incorrect and
// caused filter instability on longer signals (growing output -> overflow).
pub(crate) const HIGH_SHELF_48K: BiquadCoefficients = BiquadCoefficients {
    b0: 1.0,
    b1: -2.0,
    b2: 1.0,
    a1: -1.990_047_5,
    a2: 0.990_072_25,
};

pub(crate) fn apply_loudness_weighting(sample_rate: SampleRate, samples: &[f32]) -> Vec<f32> {
    if sample_rate.0 != 48_000 {
        return samples.to_vec();
    }

    let mut stage_one = BiquadState::default();
    let mut stage_two = BiquadState::default();
    let mut weighted = Vec::with_capacity(samples.len());

    for sample in samples {
        let first = stage_one.process(PRE_FILTER_48K, *sample);
        let second = stage_two.process(HIGH_SHELF_48K, first);
        weighted.push(second);
    }

    weighted
}

pub(crate) fn loudness_channel_weights(
    layout: ChannelLayout,
    channel_count: ChannelCount,
) -> (Vec<f32>, LoudnessChannelWeightSource) {
    match layout {
        ChannelLayout::Mono => (vec![1.0], LoudnessChannelWeightSource::MonoDirect),
        ChannelLayout::Stereo => (
            vec![1.0, 1.0],
            LoudnessChannelWeightSource::StereoEqualWeight,
        ),
        ChannelLayout::Count(count) => (
            vec![1.0; count.0.min(channel_count.0)],
            LoudnessChannelWeightSource::GenericCountFallback,
        ),
    }
}

pub(crate) fn loudness_sample_rate_support(
    source_sample_rate: SampleRate,
    analysis_sample_rate: SampleRate,
) -> LoudnessSampleRateSupport {
    if analysis_sample_rate.0 == 48_000 {
        if source_sample_rate.0 == 48_000 {
            LoudnessSampleRateSupport::Native48kKWeighted
        } else {
            LoudnessSampleRateSupport::ResampledTo48kKWeighted
        }
    } else {
        LoudnessSampleRateSupport::UnweightedFallback
    }
}

pub(crate) fn true_peak_oversample_factor(sample_rate: SampleRate) -> usize {
    match sample_rate.0 {
        0..=48_000 => 4,
        48_001..=96_000 => 2,
        _ => 1,
    }
}

pub(crate) fn seconds_to_frames(sample_rate: SampleRate, seconds: f32) -> usize {
    (sample_rate.0 as f32 * seconds).round().max(0.0) as usize
}

pub(crate) fn window_mean_square(samples: &[f32], window_size: usize, hop_size: usize) -> Vec<f32> {
    if samples.is_empty() || window_size == 0 || hop_size == 0 {
        return Vec::new();
    }

    let mut energies = Vec::new();
    let mut start = 0usize;
    while start < samples.len() {
        let end = (start + window_size).min(samples.len());
        let window = &samples[start..end];
        if window.is_empty() {
            break;
        }
        let mean_square =
            window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32;
        energies.push(mean_square);

        if end == samples.len() {
            break;
        }
        start = start.saturating_add(hop_size);
    }
    energies
}

pub(crate) fn lufs_from_mean_square(mean_square: f32) -> f32 {
    if mean_square <= 0.0 {
        f32::NEG_INFINITY
    } else {
        -0.691 + 10.0 * mean_square.log10()
    }
}
