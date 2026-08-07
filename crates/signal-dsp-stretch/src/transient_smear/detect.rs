use signal_primitives::Sample;

use super::features::{mean_plus_stddev, merge_nearby_transients, transient_frame_features};
use super::types::{StretchTransientDetectorPolicy, StretchTransientEvent};

/// Detect transient candidates from frame energy rise and positive spectral
/// flux. This is a measurement primitive only; it does not change synthesis.
#[cfg(any(test, feature = "evidence"))]
pub fn detect_stretch_transients(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
) -> Vec<StretchTransientEvent> {
    detect_stretch_transients_with_policy(
        samples,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
    )
}

/// Detect transient candidates using an explicit threshold policy.
///
/// This is a measurement primitive only. Candidate policies are for corpus
/// evidence and review gates; they do not change synthesis.
pub fn detect_stretch_transients_with_policy(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
    policy: StretchTransientDetectorPolicy,
) -> Vec<StretchTransientEvent> {
    if samples.len() < window_size || window_size < 16 || hop_size == 0 {
        return Vec::new();
    }

    let frame_features = transient_frame_features(samples, window_size, hop_size);
    if frame_features.len() < 3 {
        return Vec::new();
    }

    let mut energy_rises = Vec::with_capacity(frame_features.len());
    let mut fluxes = Vec::with_capacity(frame_features.len());
    energy_rises.push(0.0);
    fluxes.push(0.0);
    for pair in frame_features.windows(2) {
        energy_rises.push((pair[1].energy - pair[0].energy).max(0.0));
        fluxes.push(pair[1].spectral_flux);
    }

    let energy_scale = mean_plus_stddev(&energy_rises).max(1.0e-12);
    let flux_scale = mean_plus_stddev(&fluxes).max(1.0e-12);
    let mut events = Vec::new();

    for index in 1..frame_features.len() - 1 {
        let energy_score = energy_rises[index] / energy_scale;
        let flux_score = fluxes[index] / flux_scale;
        let combined_score = energy_score + flux_score;
        let previous_score =
            energy_rises[index - 1] / energy_scale + fluxes[index - 1] / flux_scale;
        let next_score = energy_rises[index + 1] / energy_scale + fluxes[index + 1] / flux_scale;
        if combined_score >= policy.minimum_combined_score
            && combined_score >= previous_score
            && combined_score > next_score
            && flux_score >= policy.minimum_spectral_flux_score
        {
            events.push(StretchTransientEvent {
                frame_index: frame_features[index].frame_index,
                energy_score,
                spectral_flux_score: flux_score,
                combined_score,
            });
        }
    }

    merge_nearby_transients(events, hop_size * 2)
}
