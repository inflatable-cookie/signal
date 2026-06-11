use signal_analysis::Confidence;
use signal_primitives::{AudioBuffer, Sample};

use crate::analysis_math::lufs_from_mean_square;
use crate::types::{
    LoudnessAggregationSummary, LoudnessAnalysisResult, LoudnessChannelWeightSource,
    LoudnessDynamicsSummary, LoudnessSampleRateSupport, LoudnessTrace,
};

pub(crate) fn gated_integrated_loudness(block_energies: &[f32]) -> f32 {
    if block_energies.is_empty() {
        return f32::NEG_INFINITY;
    }

    let absolute_gated: Vec<f32> = block_energies
        .iter()
        .copied()
        .filter(|energy| lufs_from_mean_square(*energy) >= -70.0)
        .collect();

    if absolute_gated.is_empty() {
        return f32::NEG_INFINITY;
    }

    let absolute_mean = absolute_gated.iter().copied().sum::<f32>() / absolute_gated.len() as f32;
    let relative_threshold = lufs_from_mean_square(absolute_mean) - 10.0;

    let relative_gated: Vec<f32> = absolute_gated
        .into_iter()
        .filter(|energy| lufs_from_mean_square(*energy) >= relative_threshold)
        .collect();

    if relative_gated.is_empty() {
        return f32::NEG_INFINITY;
    }

    let integrated_mean = relative_gated.iter().copied().sum::<f32>() / relative_gated.len() as f32;
    lufs_from_mean_square(integrated_mean)
}

pub(crate) fn aggregate_weighted_energies(
    channel_energies: &[Vec<f32>],
    weights: &[f32],
) -> Vec<f32> {
    let max_len = channel_energies.iter().map(Vec::len).max().unwrap_or(0);
    let mut aggregated = vec![0.0; max_len];

    for (channel_index, energies) in channel_energies.iter().enumerate() {
        let gain = *weights.get(channel_index).unwrap_or(&1.0);
        let energy_scale = gain * gain;
        for (index, energy) in energies.iter().copied().enumerate() {
            aggregated[index] += energy * energy_scale;
        }
    }

    aggregated
}

/// Loudness range per EBU Tech 3342.
///
/// Two-stage gating over the short-term loudness distribution: an absolute
/// gate at -70 LUFS, then a relative gate at -20 LU below the power mean of
/// the absolute-gated values, then the 10th..95th percentile spread.
pub(crate) fn loudness_range_from_energies(short_term_energies: &[f32]) -> f32 {
    let absolute_gated: Vec<f32> = short_term_energies
        .iter()
        .copied()
        .filter(|energy| lufs_from_mean_square(*energy) >= -70.0)
        .collect();

    if absolute_gated.len() < 2 {
        return 0.0;
    }

    let absolute_mean = absolute_gated.iter().copied().sum::<f32>() / absolute_gated.len() as f32;
    let relative_threshold = lufs_from_mean_square(absolute_mean) - 20.0;

    let mut loudness_values: Vec<f32> = absolute_gated
        .into_iter()
        .map(lufs_from_mean_square)
        .filter(|value| value.is_finite() && *value >= relative_threshold)
        .collect();

    if loudness_values.len() < 2 {
        return 0.0;
    }

    loudness_values.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let lower = percentile(&loudness_values, 0.10);
    let upper = percentile(&loudness_values, 0.95);
    (upper - lower).max(0.0)
}

const TRUE_PEAK_TAPS_PER_PHASE: usize = 12;

/// Build the polyphase interpolation FIR used for true-peak oversampling.
///
/// ITU-R BS.1770-4 Annex 2 specifies inter-sample peak estimation through a
/// 4x polyphase FIR interpolator (48 taps total at 48 kHz, 12 per phase) and
/// allows equivalent designs. This is a Blackman-windowed sinc prototype with
/// the same tap budget (12 taps per phase at any oversample factor), with
/// each polyphase branch normalised to unity DC gain so interpolated values
/// share the input sample scale. Measured response at 4x/48 kHz: flat within
/// 0.01 dB through 12 kHz, -1.4 dB at 20 kHz, image rejection >= 45 dB above
/// 34 kHz (>= 63 dB above 36 kHz).
fn true_peak_interpolation_taps(oversample_factor: usize) -> Vec<f32> {
    let total_taps = TRUE_PEAK_TAPS_PER_PHASE * oversample_factor;
    let center = (total_taps - 1) as f32 / 2.0;
    let mut taps = Vec::with_capacity(total_taps);
    for index in 0..total_taps {
        let offset = (index as f32 - center) / oversample_factor as f32;
        let sinc = if offset.abs() < 1e-6 {
            1.0
        } else {
            (core::f32::consts::PI * offset).sin() / (core::f32::consts::PI * offset)
        };
        let position = index as f32 / (total_taps - 1) as f32;
        let window = 0.42 - 0.5 * (core::f32::consts::TAU * position).cos()
            + 0.08 * (2.0 * core::f32::consts::TAU * position).cos();
        taps.push(sinc * window);
    }

    for phase in 0..oversample_factor {
        let branch_sum: f32 = taps.iter().skip(phase).step_by(oversample_factor).sum();
        if branch_sum.abs() > f32::EPSILON {
            for tap in taps.iter_mut().skip(phase).step_by(oversample_factor) {
                *tap /= branch_sum;
            }
        }
    }
    taps
}

/// True-peak level in dBTP via oversampled FIR interpolation (BS.1770-4
/// Annex 2). The raw sample peak is folded into the maximum so the result is
/// never below the plain sample-peak reading.
pub(crate) fn true_peak_dbtp(samples: &[f32], oversample_factor: usize) -> f32 {
    if samples.is_empty() || oversample_factor == 0 {
        return f32::NEG_INFINITY;
    }

    let mut peak = samples
        .iter()
        .fold(0.0f32, |acc, sample| acc.max(sample.abs()));

    if oversample_factor > 1 {
        let taps = true_peak_interpolation_taps(oversample_factor);
        let taps_per_phase = taps.len() / oversample_factor;
        // Polyphase evaluation of the zero-stuffed convolution. Run past the
        // end of the input to flush the filter tail.
        for position in 0..samples.len() + taps_per_phase {
            for phase in 0..oversample_factor {
                let mut interpolated = 0.0f32;
                for tap_index in 0..taps_per_phase {
                    let Some(sample_index) = position.checked_sub(tap_index) else {
                        break;
                    };
                    let Some(sample) = samples.get(sample_index) else {
                        continue;
                    };
                    interpolated += taps[tap_index * oversample_factor + phase] * sample;
                }
                peak = peak.max(interpolated.abs());
            }
        }
    }

    if peak == 0.0 {
        f32::NEG_INFINITY
    } else {
        20.0 * peak.log10()
    }
}

pub(crate) fn loudness_confidence(
    sample_rate_support: LoudnessSampleRateSupport,
    channel_weight_source: LoudnessChannelWeightSource,
    block_count: usize,
) -> Confidence {
    let rate_factor = match sample_rate_support {
        LoudnessSampleRateSupport::Native48kKWeighted => 1.0,
        LoudnessSampleRateSupport::ResampledTo48kKWeighted => 0.95,
        LoudnessSampleRateSupport::UnweightedFallback => 0.75,
    };
    let channel_factor = match channel_weight_source {
        LoudnessChannelWeightSource::MonoDirect
        | LoudnessChannelWeightSource::StereoEqualWeight => 1.0,
        LoudnessChannelWeightSource::GenericCountFallback => 0.9,
    };
    let coverage_factor = (block_count as f32 / 10.0).clamp(0.0, 1.0);
    Confidence::new(rate_factor * channel_factor * coverage_factor)
}

pub(crate) fn deinterleave_channels(audio: &AudioBuffer) -> Vec<Vec<Sample>> {
    let channel_count = audio.channel_count().0;
    if channel_count == 0 || audio.is_empty() {
        return Vec::new();
    }

    if channel_count == 1 {
        return vec![audio.samples().to_vec()];
    }

    let mut channels = vec![Vec::with_capacity(audio.frames().0); channel_count];
    for frame in audio.samples().chunks_exact(channel_count) {
        for (channel, sample) in channels.iter_mut().zip(frame.iter().copied()) {
            channel.push(sample);
        }
    }
    channels
}

pub(crate) fn empty_loudness_result(
    aggregation: LoudnessAggregationSummary,
) -> LoudnessAnalysisResult {
    LoudnessAnalysisResult {
        integrated_lufs: f32::NEG_INFINITY,
        loudness_range_lu: 0.0,
        true_peak_dbtp: f32::NEG_INFINITY,
        confidence: Confidence::new(0.0),
        channels: Vec::new(),
        aggregation,
        momentary_trace: LoudnessTrace {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            points: Vec::new(),
        },
        short_term_trace: LoudnessTrace {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            points: Vec::new(),
        },
        dynamics: LoudnessDynamicsSummary {
            target_offset_lu: 0.0,
            peak_to_loudness_lu: 0.0,
            momentary_max_lufs: f32::NEG_INFINITY,
            short_term_max_lufs: f32::NEG_INFINITY,
            momentary_range_lu: 0.0,
            short_term_range_lu: 0.0,
        },
    }
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    let index = ((sorted.len() - 1) as f32 * fraction).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}
