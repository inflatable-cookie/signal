use rustfft::num_complex::Complex64;

use super::super::super::{interpolate, FrequencyBoundaryPolicy};

pub(super) struct BinResult {
    pub(super) output: [Complex64; 2],
    pub(super) reference: usize,
    pub(super) corrected: bool,
    pub(super) active_tie: bool,
    pub(super) unilateral_non_silent_completion: bool,
}

pub(super) fn reference_relative_bin(
    bin: usize,
    bins: usize,
    long_distance: usize,
    time_factor: f64,
    current: &[Vec<Complex64>; 2],
    preliminary: &[Vec<Complex64>; 2],
    corrected: &[Vec<Complex64>; 2],
    significant_energy: f64,
) -> BinResult {
    let target_energy = [current[0][bin].norm_sqr(), current[1][bin].norm_sqr()];
    let reference = usize::from(target_energy[1] > target_energy[0]);
    let peer = 1 - reference;
    let prediction = vertical_prediction(
        bin,
        bins,
        long_distance,
        time_factor,
        &current[reference],
        &preliminary[reference],
        &corrected[reference],
    );
    let prediction_energy = prediction.norm_sqr();
    let floor = target_energy[reference] * f64::EPSILON * 64.0;
    let corrected_reference = prediction_energy > floor;
    let output = if corrected_reference {
        let reference_output = prediction * (target_energy[reference] / prediction_energy).sqrt();
        let (peer_output, peer_fallback) = project_peer(
            reference_output,
            current[reference][bin],
            current[peer][bin],
            target_energy[peer],
        );
        let output = if reference == 0 {
            [reference_output, peer_output]
        } else {
            [peer_output, reference_output]
        };
        (output, peer_fallback)
    } else {
        ([current[0][bin], current[1][bin]], false)
    };
    let unilateral_non_silent_completion =
        corrected_reference && target_energy[peer] > significant_energy && output.1;
    BinResult {
        output: output.0,
        reference,
        corrected: corrected_reference,
        active_tie: target_energy[0] == target_energy[1] && target_energy[0] > 0.0,
        unilateral_non_silent_completion,
    }
}

fn project_peer(
    reference_output: Complex64,
    reference_input: Complex64,
    peer_input: Complex64,
    peer_target_energy: f64,
) -> (Complex64, bool) {
    if peer_target_energy == 0.0 {
        return (Complex64::new(0.0, 0.0), false);
    }
    if peer_input == reference_input {
        return (reference_output, false);
    }
    let projected = reference_output * peer_input * reference_input.conj();
    let projected_energy = projected.norm_sqr();
    if projected_energy > peer_target_energy * f64::EPSILON * 64.0 {
        (
            projected * (peer_target_energy / projected_energy).sqrt(),
            false,
        )
    } else {
        (peer_input, true)
    }
}

fn vertical_prediction(
    bin: usize,
    bins: usize,
    long_distance: usize,
    time_factor: f64,
    current: &[Complex64],
    preliminary: &[Complex64],
    corrected: &[Complex64],
) -> Complex64 {
    let mut prediction = Complex64::new(0.0, 0.0);
    if bin >= 1 {
        let lower = interpolate(
            current,
            bin as f64 - time_factor,
            FrequencyBoundaryPolicy::Clamp,
        );
        let twist = current[bin] * lower.conj();
        prediction += corrected[bin - 1] * twist;
    }
    if bin + 1 < bins {
        let lower = interpolate(
            current,
            bin as f64 + 1.0 - time_factor,
            FrequencyBoundaryPolicy::Clamp,
        );
        let twist = current[bin + 1] * lower.conj();
        prediction += preliminary[bin + 1] * twist.conj();
    }
    if bin >= long_distance {
        let lower = interpolate(
            current,
            bin as f64 - long_distance as f64 * time_factor,
            FrequencyBoundaryPolicy::Clamp,
        );
        let twist = current[bin] * lower.conj();
        prediction += corrected[bin - long_distance] * twist;
    }
    if bin + long_distance < bins {
        let lower = interpolate(
            current,
            bin as f64 + long_distance as f64 - long_distance as f64 * time_factor,
            FrequencyBoundaryPolicy::Clamp,
        );
        let twist = current[bin + long_distance] * lower.conj();
        prediction += preliminary[bin + long_distance] * twist.conj();
    }
    prediction
}
