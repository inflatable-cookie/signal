use signal_dsp_spectral::Spectrogram;
use signal_primitives::SampleRate;

use crate::normalize;

mod meter_cues;

pub(crate) use meter_cues::{band_profile_change, low_band_flux};

fn spectral_flux(spectrogram: &Spectrogram) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(spectrogram.frames.len());
    let mut previous: Option<&[f32]> = None;

    for frame in &spectrogram.frames {
        let current = frame.magnitudes.as_slice();
        let flux = if let Some(last) = previous {
            current
                .iter()
                .zip(last.iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum()
        } else {
            0.0
        };
        envelope.push(flux);
        previous = Some(current);
    }

    normalize(&mut envelope);
    envelope
}

fn high_frequency_content(spectrogram: &Spectrogram) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(spectrogram.frames.len());
    for frame in &spectrogram.frames {
        let hfc = frame
            .magnitudes
            .iter()
            .enumerate()
            .map(|(index, magnitude)| index as f32 * magnitude)
            .sum();
        envelope.push(hfc);
    }
    normalize(&mut envelope);
    envelope
}

fn bandwise_spectral_flux(spectrogram: &Spectrogram, bands: usize) -> Vec<f32> {
    if spectrogram.frames.is_empty() || bands == 0 {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let band_width = (bin_count - 1).div_ceil(bands);
    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for (frame_index, frame) in spectrogram.frames.iter().enumerate().skip(1) {
        let current = frame.magnitudes.as_slice();
        let previous = spectrogram.frames[frame_index - 1].magnitudes.as_slice();

        let mut score = 0.0;
        let mut active_bands = 0usize;
        let mut band_start = 1usize;

        while band_start < bin_count {
            let band_end = (band_start + band_width).min(bin_count);
            let band_flux: f32 = current[band_start..band_end]
                .iter()
                .zip(previous[band_start..band_end].iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum();
            if band_flux > 0.0 {
                score += band_flux / (band_end - band_start) as f32;
                active_bands += 1;
            }
            band_start = band_end;
        }

        envelope[frame_index] = if active_bands > 0 {
            score / active_bands as f32
        } else {
            0.0
        };
    }

    normalize(&mut envelope);
    envelope
}

fn complex_domain_difference(spectrogram: &Spectrogram) -> Vec<f32> {
    if spectrogram.frames.is_empty() {
        return Vec::new();
    }

    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for (frame_index, current) in spectrogram.frames.iter().enumerate().skip(1) {
        let previous = &spectrogram.frames[frame_index - 1];
        let older = frame_index
            .checked_sub(2)
            .map(|index| &spectrogram.frames[index]);

        let bin_count = current
            .magnitudes
            .len()
            .min(previous.magnitudes.len())
            .min(current.phases.len())
            .min(previous.phases.len());

        let mut score = 0.0;
        for bin_index in 1..bin_count {
            let current_magnitude = current.magnitudes[bin_index];
            let previous_magnitude = previous.magnitudes[bin_index];
            let predicted_phase = older
                .and_then(|frame| frame.phases.get(bin_index).copied())
                .map(|older_phase| 2.0 * previous.phases[bin_index] - older_phase)
                .unwrap_or(previous.phases[bin_index]);
            let phase_delta = current.phases[bin_index] - predicted_phase;
            let distance = (current_magnitude * current_magnitude
                + previous_magnitude * previous_magnitude
                - 2.0 * current_magnitude * previous_magnitude * phase_delta.cos())
            .max(0.0)
            .sqrt();
            score += distance;
        }

        envelope[frame_index] = score;
    }

    normalize(&mut envelope);
    envelope
}

fn energy_flux(samples: &[f32], sample_rate: SampleRate, hop_size: usize) -> Vec<f32> {
    if samples.is_empty() || sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    let window_size = hop_size * 2;
    let mut energies = Vec::new();
    let mut start = 0usize;

    while start < samples.len() {
        let end = (start + window_size).min(samples.len());
        let window = &samples[start..end];
        if window.is_empty() {
            break;
        }
        let rms =
            (window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32).sqrt();
        energies.push(rms);
        if end == samples.len() {
            break;
        }
        start = start.saturating_add(hop_size);
    }

    let mut flux = Vec::with_capacity(energies.len());
    let mut previous: Option<f32> = None;
    for energy in energies {
        let delta = previous.map(|last| (energy - last).max(0.0)).unwrap_or(0.0);
        flux.push(delta);
        previous = Some(energy);
    }

    normalize(&mut flux);
    flux
}

pub(crate) fn multifeature_onset_envelope(
    spectrogram: &Spectrogram,
    mono_samples: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
    reduced_features: bool,
) -> Vec<f32> {
    if reduced_features {
        let (flux, band_flux, energy) = std::thread::scope(|s| {
            let flux_handle = s.spawn(|| spectral_flux(spectrogram));
            let band_flux_handle = s.spawn(|| bandwise_spectral_flux(spectrogram, 6));
            let energy_handle = s.spawn(|| energy_flux(mono_samples, sample_rate, hop_size));

            (
                flux_handle.join().unwrap(),
                band_flux_handle.join().unwrap(),
                energy_handle.join().unwrap(),
            )
        });

        let len = flux.len().max(band_flux.len()).max(energy.len());
        let mut combined = vec![0.0; len];
        for (index, value) in combined.iter_mut().enumerate().take(len) {
            let flux_value = flux.get(index).copied().unwrap_or(0.0);
            let band_flux_value = band_flux.get(index).copied().unwrap_or(0.0);
            let energy_value = energy.get(index).copied().unwrap_or(0.0);
            *value = 0.48 * flux_value + 0.38 * band_flux_value + 0.14 * energy_value;
        }

        sharpen_onset_envelope(&mut combined);
        normalize(&mut combined);
        return combined;
    }

    let (flux, band_flux, complex, hfc, energy) = std::thread::scope(|s| {
        let flux_handle = s.spawn(|| spectral_flux(spectrogram));
        let band_flux_handle = s.spawn(|| bandwise_spectral_flux(spectrogram, 6));
        let complex_handle = s.spawn(|| complex_domain_difference(spectrogram));
        let hfc_handle = s.spawn(|| high_frequency_content(spectrogram));
        let energy_handle = s.spawn(|| energy_flux(mono_samples, sample_rate, hop_size));

        (
            flux_handle.join().unwrap(),
            band_flux_handle.join().unwrap(),
            complex_handle.join().unwrap(),
            hfc_handle.join().unwrap(),
            energy_handle.join().unwrap(),
        )
    });

    let len = flux
        .len()
        .max(band_flux.len())
        .max(complex.len())
        .max(hfc.len())
        .max(energy.len());

    let mut combined = vec![0.0; len];
    for (index, value) in combined.iter_mut().enumerate().take(len) {
        let flux_value = flux.get(index).copied().unwrap_or(0.0);
        let band_flux_value = band_flux.get(index).copied().unwrap_or(0.0);
        let complex_value = complex.get(index).copied().unwrap_or(0.0);
        let hfc_value = hfc.get(index).copied().unwrap_or(0.0);
        let energy_value = energy.get(index).copied().unwrap_or(0.0);
        *value = 0.28 * flux_value
            + 0.22 * band_flux_value
            + 0.30 * complex_value
            + 0.12 * hfc_value
            + 0.08 * energy_value;
    }

    sharpen_onset_envelope(&mut combined);
    normalize(&mut combined);
    combined
}

fn sharpen_onset_envelope(values: &mut [f32]) {
    if values.is_empty() {
        return;
    }

    let source = values.to_vec();
    let radius = 8usize.min(source.len().saturating_sub(1)).max(1);
    let mut prefix = vec![0.0; source.len() + 1];

    for (index, value) in source.iter().copied().enumerate() {
        prefix[index + 1] = prefix[index] + value;
    }

    for index in 0..source.len() {
        let start = index.saturating_sub(radius);
        let end = (index + radius + 1).min(source.len());
        let local_mean = (prefix[end] - prefix[start]) / (end - start) as f32;
        let previous = index.checked_sub(1).map(|i| source[i]).unwrap_or(0.0);
        let rising_edge = (source[index] - previous).max(0.0);
        values[index] = (source[index] - 0.65 * local_mean).max(0.0) + 0.2 * rising_edge;
    }
}
