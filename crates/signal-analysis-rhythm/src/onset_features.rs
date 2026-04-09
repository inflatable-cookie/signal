use signal_dsp_spectral::Spectrogram;
use signal_primitives::SampleRate;

use crate::normalize;

mod meter_cues;

pub(crate) use meter_cues::{band_profile_change, low_band_flux};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnsetFeatureKind {
    Flux,
    BandFlux,
    Complex,
    HighFrequencyContent,
    Energy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnsetFeatureAvailability {
    Ready,
    WorkerPanicked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnsetFeatureStatus {
    kind: OnsetFeatureKind,
    availability: OnsetFeatureAvailability,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OnsetFeatureAvailabilityReport {
    statuses: Vec<OnsetFeatureStatus>,
}

impl OnsetFeatureAvailabilityReport {
    fn push(&mut self, status: OnsetFeatureStatus) {
        self.statuses.push(status);
    }

    #[cfg(test)]
    fn degraded_count(&self) -> usize {
        self.statuses
            .iter()
            .filter(|status| status.availability != OnsetFeatureAvailability::Ready)
            .count()
    }
}

#[derive(Debug, Clone)]
struct OnsetFeatureWorkerResult {
    values: Vec<f32>,
    status: OnsetFeatureStatus,
}

#[derive(Debug, Clone)]
struct OnsetEnvelopeComputation {
    envelope: Vec<f32>,
    #[cfg_attr(not(test), allow(dead_code))]
    availability: OnsetFeatureAvailabilityReport,
}

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

fn expected_energy_flux_len(samples: &[f32], sample_rate: SampleRate, hop_size: usize) -> usize {
    if samples.is_empty() || sample_rate.0 == 0 || hop_size == 0 {
        return 0;
    }

    let window_size = hop_size * 2;
    let mut count = 0usize;
    let mut start = 0usize;

    while start < samples.len() {
        let end = (start + window_size).min(samples.len());
        if end <= start {
            break;
        }
        count += 1;
        if end == samples.len() {
            break;
        }
        start = start.saturating_add(hop_size);
    }

    count
}

fn worker_result_from_join(
    kind: OnsetFeatureKind,
    expected_len: usize,
    join_result: std::thread::Result<Vec<f32>>,
) -> OnsetFeatureWorkerResult {
    match join_result {
        Ok(values) => OnsetFeatureWorkerResult {
            values,
            status: OnsetFeatureStatus {
                kind,
                availability: OnsetFeatureAvailability::Ready,
            },
        },
        Err(_) => OnsetFeatureWorkerResult {
            values: vec![0.0; expected_len],
            status: OnsetFeatureStatus {
                kind,
                availability: OnsetFeatureAvailability::WorkerPanicked,
            },
        },
    }
}

fn combine_reduced_feature_results(
    flux: OnsetFeatureWorkerResult,
    band_flux: OnsetFeatureWorkerResult,
    energy: OnsetFeatureWorkerResult,
) -> OnsetEnvelopeComputation {
    let len = flux
        .values
        .len()
        .max(band_flux.values.len())
        .max(energy.values.len());
    let mut combined = vec![0.0; len];
    for (index, value) in combined.iter_mut().enumerate().take(len) {
        let flux_value = flux.values.get(index).copied().unwrap_or(0.0);
        let band_flux_value = band_flux.values.get(index).copied().unwrap_or(0.0);
        let energy_value = energy.values.get(index).copied().unwrap_or(0.0);
        *value = 0.48 * flux_value + 0.38 * band_flux_value + 0.14 * energy_value;
    }

    sharpen_onset_envelope(&mut combined);
    normalize(&mut combined);

    let mut availability = OnsetFeatureAvailabilityReport::default();
    availability.push(flux.status);
    availability.push(band_flux.status);
    availability.push(energy.status);

    OnsetEnvelopeComputation {
        envelope: combined,
        availability,
    }
}

fn combine_full_feature_results(
    flux: OnsetFeatureWorkerResult,
    band_flux: OnsetFeatureWorkerResult,
    complex: OnsetFeatureWorkerResult,
    hfc: OnsetFeatureWorkerResult,
    energy: OnsetFeatureWorkerResult,
) -> OnsetEnvelopeComputation {
    let len = flux
        .values
        .len()
        .max(band_flux.values.len())
        .max(complex.values.len())
        .max(hfc.values.len())
        .max(energy.values.len());

    let mut combined = vec![0.0; len];
    for (index, value) in combined.iter_mut().enumerate().take(len) {
        let flux_value = flux.values.get(index).copied().unwrap_or(0.0);
        let band_flux_value = band_flux.values.get(index).copied().unwrap_or(0.0);
        let complex_value = complex.values.get(index).copied().unwrap_or(0.0);
        let hfc_value = hfc.values.get(index).copied().unwrap_or(0.0);
        let energy_value = energy.values.get(index).copied().unwrap_or(0.0);
        *value = 0.28 * flux_value
            + 0.22 * band_flux_value
            + 0.30 * complex_value
            + 0.12 * hfc_value
            + 0.08 * energy_value;
    }

    sharpen_onset_envelope(&mut combined);
    normalize(&mut combined);

    let mut availability = OnsetFeatureAvailabilityReport::default();
    availability.push(flux.status);
    availability.push(band_flux.status);
    availability.push(complex.status);
    availability.push(hfc.status);
    availability.push(energy.status);

    OnsetEnvelopeComputation {
        envelope: combined,
        availability,
    }
}

fn compute_reduced_onset_with_workers<FFlux, FBandFlux, FEnergy>(
    spectral_feature_len: usize,
    energy_feature_len: usize,
    flux_worker: FFlux,
    band_flux_worker: FBandFlux,
    energy_worker: FEnergy,
) -> OnsetEnvelopeComputation
where
    FFlux: FnOnce() -> Vec<f32> + Send,
    FBandFlux: FnOnce() -> Vec<f32> + Send,
    FEnergy: FnOnce() -> Vec<f32> + Send,
{
    let (flux, band_flux, energy) = std::thread::scope(|s| {
        let flux_handle = s.spawn(flux_worker);
        let band_flux_handle = s.spawn(band_flux_worker);
        let energy_handle = s.spawn(energy_worker);

        (
            worker_result_from_join(
                OnsetFeatureKind::Flux,
                spectral_feature_len,
                flux_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::BandFlux,
                spectral_feature_len,
                band_flux_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::Energy,
                energy_feature_len,
                energy_handle.join(),
            ),
        )
    });

    combine_reduced_feature_results(flux, band_flux, energy)
}

fn compute_full_onset_with_workers<FFlux, FBandFlux, FComplex, FHfc, FEnergy>(
    spectral_feature_len: usize,
    energy_feature_len: usize,
    flux_worker: FFlux,
    band_flux_worker: FBandFlux,
    complex_worker: FComplex,
    hfc_worker: FHfc,
    energy_worker: FEnergy,
) -> OnsetEnvelopeComputation
where
    FFlux: FnOnce() -> Vec<f32> + Send,
    FBandFlux: FnOnce() -> Vec<f32> + Send,
    FComplex: FnOnce() -> Vec<f32> + Send,
    FHfc: FnOnce() -> Vec<f32> + Send,
    FEnergy: FnOnce() -> Vec<f32> + Send,
{
    let (flux, band_flux, complex, hfc, energy) = std::thread::scope(|s| {
        let flux_handle = s.spawn(flux_worker);
        let band_flux_handle = s.spawn(band_flux_worker);
        let complex_handle = s.spawn(complex_worker);
        let hfc_handle = s.spawn(hfc_worker);
        let energy_handle = s.spawn(energy_worker);

        (
            worker_result_from_join(
                OnsetFeatureKind::Flux,
                spectral_feature_len,
                flux_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::BandFlux,
                spectral_feature_len,
                band_flux_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::Complex,
                spectral_feature_len,
                complex_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::HighFrequencyContent,
                spectral_feature_len,
                hfc_handle.join(),
            ),
            worker_result_from_join(
                OnsetFeatureKind::Energy,
                energy_feature_len,
                energy_handle.join(),
            ),
        )
    });

    combine_full_feature_results(flux, band_flux, complex, hfc, energy)
}

fn multifeature_onset_envelope_with_report(
    spectrogram: &Spectrogram,
    mono_samples: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
    reduced_features: bool,
) -> OnsetEnvelopeComputation {
    let spectral_feature_len = spectrogram.frames.len();
    let energy_feature_len = expected_energy_flux_len(mono_samples, sample_rate, hop_size);

    if reduced_features {
        return compute_reduced_onset_with_workers(
            spectral_feature_len,
            energy_feature_len,
            || spectral_flux(spectrogram),
            || bandwise_spectral_flux(spectrogram, 6),
            || energy_flux(mono_samples, sample_rate, hop_size),
        );
    }

    compute_full_onset_with_workers(
        spectral_feature_len,
        energy_feature_len,
        || spectral_flux(spectrogram),
        || bandwise_spectral_flux(spectrogram, 6),
        || complex_domain_difference(spectrogram),
        || high_frequency_content(spectrogram),
        || energy_flux(mono_samples, sample_rate, hop_size),
    )
}

pub(crate) fn multifeature_onset_envelope(
    spectrogram: &Spectrogram,
    mono_samples: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
    reduced_features: bool,
) -> Vec<f32> {
    multifeature_onset_envelope_with_report(
        spectrogram,
        mono_samples,
        sample_rate,
        hop_size,
        reduced_features,
    )
    .envelope
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

#[cfg(test)]
mod tests {
    use super::{
        compute_full_onset_with_workers, compute_reduced_onset_with_workers,
        OnsetFeatureAvailability, OnsetFeatureKind,
    };

    #[test]
    fn reduced_onset_containment_recovers_from_worker_panic() {
        let computation = compute_reduced_onset_with_workers(
            4,
            3,
            || vec![0.0, 0.7, 0.2, 0.1],
            || panic!("band flux worker lost"),
            || vec![0.0, 0.2, 0.1],
        );

        assert_eq!(computation.envelope.len(), 4);
        assert_eq!(computation.availability.degraded_count(), 1);
        assert_eq!(
            computation
                .availability
                .statuses
                .iter()
                .find(|status| status.kind == OnsetFeatureKind::BandFlux)
                .map(|status| status.availability),
            Some(OnsetFeatureAvailability::WorkerPanicked)
        );
        assert!(computation.envelope.iter().all(|value| value.is_finite()));
        assert!(computation.envelope.iter().any(|value| *value > 0.0));
    }

    #[test]
    fn full_onset_containment_zero_fills_multiple_failed_workers_deterministically() {
        let computation = compute_full_onset_with_workers(
            5,
            2,
            || vec![0.0, 0.3, 0.7, 0.1, 0.0],
            || vec![0.0, 0.4, 0.1, 0.0, 0.0],
            || panic!("complex worker lost"),
            || panic!("hfc worker lost"),
            || vec![0.0, 0.5],
        );

        assert_eq!(computation.envelope.len(), 5);
        assert_eq!(computation.availability.degraded_count(), 2);
        assert_eq!(
            computation
                .availability
                .statuses
                .iter()
                .filter(|status| status.availability == OnsetFeatureAvailability::WorkerPanicked)
                .count(),
            2
        );
        assert!(computation.envelope.iter().all(|value| value.is_finite()));
        assert!(computation.envelope.iter().any(|value| *value > 0.0));
    }
}
