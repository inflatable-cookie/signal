use signal_dsp_spectral::Spectrogram;

use crate::normalize;

pub(crate) fn low_band_flux(spectrogram: &Spectrogram, max_frequency_hz: f32) -> Vec<f32> {
    if spectrogram.frames.is_empty()
        || spectrogram.sample_rate.0 == 0
        || spectrogram.config.window_size.0 == 0
    {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let max_bin = (((max_frequency_hz.max(0.0) * spectrogram.config.window_size.0 as f32)
        / spectrogram.sample_rate.0 as f32)
        .ceil() as usize)
        .clamp(1, bin_count - 1);
    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for (frame_index, frame) in spectrogram.frames.iter().enumerate().skip(1) {
        let current = &frame.magnitudes[..=max_bin];
        let previous = &spectrogram.frames[frame_index - 1].magnitudes[..=max_bin];
        envelope[frame_index] = current
            .iter()
            .zip(previous.iter())
            .map(|(now, then)| (now - then).max(0.0))
            .sum();
    }

    normalize(&mut envelope);
    envelope
}

pub(crate) fn band_profile_change(spectrogram: &Spectrogram, bands: usize) -> Vec<f32> {
    if spectrogram.frames.is_empty()
        || spectrogram.sample_rate.0 == 0
        || spectrogram.config.window_size.0 == 0
        || bands == 0
    {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let band_width = (bin_count - 1).div_ceil(bands);
    let mut profiles = Vec::with_capacity(spectrogram.frames.len());

    for frame in &spectrogram.frames {
        let mut profile = vec![0.0; bands];
        let mut band_start = 1usize;
        let mut band_index = 0usize;
        while band_start < bin_count && band_index < bands {
            let band_end = (band_start + band_width).min(bin_count);
            profile[band_index] = frame.magnitudes[band_start..band_end].iter().copied().sum();
            band_start = band_end;
            band_index += 1;
        }

        let total = profile.iter().copied().sum::<f32>();
        if total > 0.0 {
            for value in &mut profile {
                *value /= total;
            }
        }
        profiles.push(profile);
    }

    let mut envelope = vec![0.0; spectrogram.frames.len()];
    for frame_index in 1..profiles.len() {
        envelope[frame_index] = profiles[frame_index]
            .iter()
            .zip(profiles[frame_index - 1].iter())
            .map(|(now, then)| (now - then).abs())
            .sum();
    }

    normalize(&mut envelope);
    envelope
}
