use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    common_grid::{
        build_filters, conjugate_gradient, hash_u64, tighten_frequency_response, CHANNELS,
        HASH_OFFSET, HOP,
    },
    types::StretchCommonGridDualGuardEvidence,
};

const GUARD_CAP_FRAMES: usize = 16_384;
const TAIL_ENERGY_LIMIT: f64 = 1.0e-12;

pub(crate) fn common_grid_dual_guard_review(
    source_frames: usize,
) -> StretchCommonGridDualGuardEvidence {
    let probe_fft_frames =
        (source_frames.max(HOP) + 2 * (GUARD_CAP_FRAMES + HOP)).div_ceil(HOP) * HOP;
    let coefficient_frames = probe_fft_frames / HOP;
    let positive_bins = probe_fft_frames / 2 + 1;
    let mut filters = build_filters(probe_fft_frames);
    tighten_frequency_response(&mut filters, positive_bins);
    let mut planner = FftPlanner::<f64>::new();
    let inverse = planner.plan_fft_inverse(probe_fft_frames);
    let mut required_guard_lower_bound_frames = 0;
    let mut max_tail_energy_ratio = 0.0_f64;
    let mut limiting_channel = 0;
    let mut max_dual_residual = 0.0_f64;
    let mut non_finite_values = 0;
    let mut dual_atom_hash = HASH_OFFSET;
    let mut evaluated_channels = 0;
    let mut passed = true;

    for channel in channel_order() {
        let (mut spectrum, residual) =
            dual_atom_spectrum(channel, coefficient_frames, positive_bins, &filters);
        max_dual_residual = max_dual_residual.max(residual);
        for bin in 1..probe_fft_frames / 2 {
            spectrum[probe_fft_frames - bin] = spectrum[bin].conj();
        }
        spectrum[0].im = 0.0;
        spectrum[probe_fft_frames / 2].im = 0.0;
        non_finite_values += spectrum
            .iter()
            .filter(|value| !value.re.is_finite() || !value.im.is_finite())
            .count();
        inverse.process(&mut spectrum);
        for value in &mut spectrum {
            *value /= probe_fft_frames as f64;
            hash_u64(&mut dual_atom_hash, value.re.to_bits());
            hash_u64(&mut dual_atom_hash, value.im.to_bits());
        }
        non_finite_values += spectrum
            .iter()
            .filter(|value| !value.re.is_finite() || !value.im.is_finite())
            .count();
        let (guard, tail_ratio) = required_guard(&spectrum);
        evaluated_channels += 1;
        if guard > required_guard_lower_bound_frames {
            required_guard_lower_bound_frames = guard;
            limiting_channel = channel;
        }
        max_tail_energy_ratio = max_tail_energy_ratio.max(tail_ratio);
        if guard > GUARD_CAP_FRAMES || !tail_ratio.is_finite() || non_finite_values > 0 {
            passed = false;
            break;
        }
    }

    StretchCommonGridDualGuardEvidence {
        probe_fft_frames,
        channel_count: CHANNELS,
        evaluated_channels,
        guard_cap_frames: GUARD_CAP_FRAMES,
        required_guard_lower_bound_frames,
        max_tail_energy_ratio,
        limiting_channel,
        max_dual_residual,
        non_finite_values,
        passed: passed && evaluated_channels == CHANNELS,
        dual_atom_hash,
    }
}

fn channel_order() -> impl Iterator<Item = usize> {
    (0..16).chain(16..CHANNELS)
}

fn dual_atom_spectrum(
    channel: usize,
    coefficient_frames: usize,
    positive_bins: usize,
    filters: &[Complex64],
) -> (Vec<Complex64>, f64) {
    let mut spectrum = vec![Complex64::new(0.0, 0.0); (positive_bins - 1) * 2];
    let mut max_residual = 0.0_f64;
    for residue in 0..coefficient_frames {
        let bins = (residue..positive_bins)
            .step_by(coefficient_frames)
            .collect::<Vec<_>>();
        let size = bins.len();
        let mut frame = vec![Complex64::new(0.0, 0.0); size * size];
        for analysis_channel in 0..CHANNELS {
            let active = bins
                .iter()
                .enumerate()
                .filter_map(|(index, bin)| {
                    let value = filters[analysis_channel * positive_bins + *bin];
                    (value.norm_sqr() > 0.0).then_some((index, value))
                })
                .collect::<Vec<_>>();
            for &(row, value) in &active {
                for &(column, column_value) in &active {
                    frame[row * size + column] += value * column_value.conj();
                }
            }
        }
        let rhs = bins
            .iter()
            .map(|bin| filters[channel * positive_bins + *bin])
            .collect::<Vec<_>>();
        let (solution, residual) = conjugate_gradient(&frame, &rhs, size);
        max_residual = max_residual.max(residual);
        for (bin, value) in bins.into_iter().zip(solution) {
            spectrum[bin] = value;
        }
    }
    (spectrum, max_residual)
}

fn required_guard(atom: &[Complex64]) -> (usize, f64) {
    let peak = atom
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.norm_sqr().total_cmp(&right.1.norm_sqr()))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let total = atom.iter().map(Complex64::norm_sqr).sum::<f64>();
    if total <= f64::MIN_POSITIVE {
        return (0, 0.0);
    }
    for radius in (0..=GUARD_CAP_FRAMES - HOP).step_by(HOP) {
        let inside = circular_energy(atom, peak, radius);
        let tail_ratio = ((total - inside).max(0.0)) / total;
        if tail_ratio <= TAIL_ENERGY_LIMIT {
            return (radius + HOP, tail_ratio);
        }
    }
    let legal_inside = circular_energy(atom, peak, GUARD_CAP_FRAMES - HOP);
    (
        GUARD_CAP_FRAMES + HOP,
        ((total - legal_inside).max(0.0)) / total,
    )
}

fn circular_energy(atom: &[Complex64], peak: usize, radius: usize) -> f64 {
    (0..atom.len())
        .filter(|index| {
            let distance = index.abs_diff(peak);
            distance.min(atom.len() - distance) <= radius
        })
        .map(|index| atom[index].norm_sqr())
        .sum()
}
