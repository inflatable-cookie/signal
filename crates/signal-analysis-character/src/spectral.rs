use signal_dsp_spectral::{
    LogCompression, MelFilterNorm, MelFilterbankConfig, MelScale, MelSpectrogramConfig, Spectrogram,
};

use crate::stats::{normalize_slice, percentile_value, reduce_median, sort_f32};
use crate::types::{
    SpectralContrastDescriptorPack, SpectralProfileDescriptorPack, SpectralShapeDescriptorPack,
    MEL_PROFILE_BAND_COUNT,
};

const ROLLOFF_85_FRACTION: f32 = 0.85;
const ROLLOFF_95_FRACTION: f32 = 0.95;
const MEL_PROFILE_LOW_FREQUENCY_HZ: f32 = 30.0;
const MEL_PROFILE_HIGH_FREQUENCY_HZ: f32 = 12_000.0;

#[derive(Clone, Copy, Debug)]
struct FrameSpectralShape {
    centroid_hz: f32,
    spread_hz: f32,
    rolloff_85_hz: f32,
    rolloff_95_hz: f32,
    flatness: f32,
}

pub(crate) fn compute_spectral_shape_pack(
    spectrogram: &Spectrogram,
) -> SpectralShapeDescriptorPack {
    let mut centroids = Vec::new();
    let mut spreads = Vec::new();
    let mut rolloff_85 = Vec::new();
    let mut rolloff_95 = Vec::new();
    let mut flatness = Vec::new();

    for shape in spectrogram
        .frames
        .iter()
        .filter_map(|frame| frame_spectral_shape(spectrogram, &frame.magnitudes))
    {
        centroids.push(shape.centroid_hz);
        spreads.push(shape.spread_hz);
        rolloff_85.push(shape.rolloff_85_hz);
        rolloff_95.push(shape.rolloff_95_hz);
        flatness.push(shape.flatness);
    }

    if centroids.is_empty() {
        return SpectralShapeDescriptorPack::zero();
    }

    SpectralShapeDescriptorPack {
        centroid_hz: reduce_median(&mut centroids),
        spread_hz: reduce_median(&mut spreads),
        rolloff_85_hz: reduce_median(&mut rolloff_85),
        rolloff_95_hz: reduce_median(&mut rolloff_95),
        flatness: reduce_median(&mut flatness),
    }
}

pub(crate) fn compute_spectral_contrast_pack(
    spectrogram: &Spectrogram,
) -> SpectralContrastDescriptorPack {
    let mut frame_contrasts = Vec::new();

    for frame in &spectrogram.frames {
        let magnitudes: Vec<f32> = frame
            .magnitudes
            .iter()
            .copied()
            .skip(1)
            .filter(|magnitude| *magnitude > 0.0)
            .collect();
        if magnitudes.len() < 2 {
            continue;
        }

        let mut sorted = magnitudes;
        sort_f32(&mut sorted);
        let low = percentile_value(&sorted, 0.10);
        let high = percentile_value(&sorted, 0.90);
        if high <= 0.0 {
            continue;
        }

        let contrast_db = 20.0 * ((high + f32::EPSILON) / (low + f32::EPSILON)).log10();
        if contrast_db.is_finite() {
            frame_contrasts.push(contrast_db.max(0.0));
        }
    }

    if frame_contrasts.is_empty() {
        return SpectralContrastDescriptorPack::zero();
    }

    SpectralContrastDescriptorPack {
        contrast_db: reduce_median(&mut frame_contrasts),
    }
}

pub(crate) fn compute_spectral_profile_pack(
    spectrogram: &Spectrogram,
) -> SpectralProfileDescriptorPack {
    if spectrogram.is_empty() || spectrogram.sample_rate.0 == 0 {
        return SpectralProfileDescriptorPack::zero();
    }

    let nyquist_hz = spectrogram.sample_rate.0 as f32 * 0.5;
    let high_frequency_hz = nyquist_hz.clamp(
        MEL_PROFILE_LOW_FREQUENCY_HZ + 1.0,
        MEL_PROFILE_HIGH_FREQUENCY_HZ,
    );
    let mel = spectrogram.to_mel_spectrogram(&MelSpectrogramConfig {
        filterbank: MelFilterbankConfig {
            mel_bin_count: MEL_PROFILE_BAND_COUNT,
            low_frequency_hz: MEL_PROFILE_LOW_FREQUENCY_HZ,
            high_frequency_hz,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::UnitTri,
        },
        log_compression: LogCompression::None,
    });

    if mel.frames.is_empty() {
        return SpectralProfileDescriptorPack::zero();
    }

    let mut profile = [0.0; MEL_PROFILE_BAND_COUNT];
    for frame in &mel.frames {
        for (index, value) in frame
            .iter()
            .copied()
            .enumerate()
            .take(MEL_PROFILE_BAND_COUNT)
        {
            profile[index] += value;
        }
    }

    let frame_count = mel.frames.len() as f32;
    for value in &mut profile {
        *value /= frame_count;
    }

    normalize_slice(&mut profile);
    SpectralProfileDescriptorPack {
        normalized_mel_band_profile: profile,
    }
}

fn frame_spectral_shape(
    spectrogram: &Spectrogram,
    magnitudes: &[f32],
) -> Option<FrameSpectralShape> {
    if magnitudes.len() < 2
        || spectrogram.config.window_size.0 == 0
        || spectrogram.sample_rate.0 == 0
    {
        return None;
    }

    let mut weighted_sum = 0.0f32;
    let mut magnitude_sum = 0.0f32;
    let mut power_values = Vec::new();

    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        let frequency = signal_dsp_spectral::bin_frequency(
            bin_index,
            spectrogram.sample_rate,
            spectrogram.config.window_size.0,
        );
        weighted_sum += frequency * magnitude;
        magnitude_sum += magnitude;
        if bin_index > 0 && magnitude > 0.0 {
            power_values.push(magnitude * magnitude);
        }
    }

    if magnitude_sum <= 0.0 {
        return None;
    }

    let centroid_hz = weighted_sum / magnitude_sum;
    let mut spread_sum = 0.0f32;
    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        let frequency = signal_dsp_spectral::bin_frequency(
            bin_index,
            spectrogram.sample_rate,
            spectrogram.config.window_size.0,
        );
        spread_sum += (frequency - centroid_hz).powi(2) * magnitude;
    }

    let spread_hz = (spread_sum / magnitude_sum).sqrt();
    let rolloff_85_hz =
        frame_rolloff_frequency(magnitudes, spectrogram, ROLLOFF_85_FRACTION).unwrap_or(0.0);
    let rolloff_95_hz =
        frame_rolloff_frequency(magnitudes, spectrogram, ROLLOFF_95_FRACTION).unwrap_or(0.0);
    let flatness = spectral_flatness(&power_values);

    Some(FrameSpectralShape {
        centroid_hz,
        spread_hz,
        rolloff_85_hz,
        rolloff_95_hz,
        flatness,
    })
}

fn frame_rolloff_frequency(
    magnitudes: &[f32],
    spectrogram: &Spectrogram,
    fraction: f32,
) -> Option<f32> {
    let total_energy: f32 = magnitudes.iter().copied().sum();
    if total_energy <= 0.0 {
        return None;
    }

    let target = total_energy * fraction.clamp(0.0, 1.0);
    let mut cumulative = 0.0f32;
    for (bin_index, magnitude) in magnitudes.iter().copied().enumerate() {
        cumulative += magnitude;
        if cumulative >= target {
            return Some(signal_dsp_spectral::bin_frequency(
                bin_index,
                spectrogram.sample_rate,
                spectrogram.config.window_size.0,
            ));
        }
    }

    Some(signal_dsp_spectral::bin_frequency(
        magnitudes.len().saturating_sub(1),
        spectrogram.sample_rate,
        spectrogram.config.window_size.0,
    ))
}

fn spectral_flatness(power_values: &[f32]) -> f32 {
    if power_values.is_empty() {
        return 0.0;
    }

    let epsilon = 1e-12f32;
    let log_mean = power_values
        .iter()
        .copied()
        .map(|value| (value + epsilon).ln())
        .sum::<f32>()
        / power_values.len() as f32;
    let arithmetic_mean = power_values.iter().copied().sum::<f32>() / power_values.len() as f32;
    if arithmetic_mean <= 0.0 {
        0.0
    } else {
        (log_mean.exp() / arithmetic_mean).clamp(0.0, 1.0)
    }
}
