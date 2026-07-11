use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    common_grid::{
        build_filters, conjugate_gradient, hash_u64, tighten_frequency_response, CHANNELS,
        HASH_OFFSET, HOP,
    },
    types::{
        StretchCommonGridDualGuardEvidence, StretchCommonGridTailAtomEvidence,
        StretchCommonGridTailAttributionEvidence, StretchCommonGridTailForm,
        StretchCommonGridTailStage,
    },
};

const GUARD_CAP_FRAMES: usize = 16_384;
const TAIL_ENERGY_LIMIT: f64 = 1.0e-12;
const ATTRIBUTION_FFT_FRAMES: usize = 34_176;
const ATTRIBUTION_CHANNELS: [usize; 5] = [0, 15, 16, 768, 1_535];
const ATTRIBUTION_RADII: [usize; 6] = [384, 1_536, 4_096, 8_192, 12_288, 16_000];
const ATTRIBUTION_THRESHOLDS: [f64; 4] = [1.0e-6, 1.0e-8, 1.0e-10, 1.0e-12];

pub(crate) fn common_grid_tail_attribution_review() -> StretchCommonGridTailAttributionEvidence {
    let positive_bins = ATTRIBUTION_FFT_FRAMES / 2 + 1;
    let coefficient_frames = ATTRIBUTION_FFT_FRAMES / HOP;
    let raw_filters = build_filters(ATTRIBUTION_FFT_FRAMES);
    let mut tightened_filters = raw_filters.clone();
    tighten_frequency_response(&mut tightened_filters, positive_bins);
    let (dual_spectra, dual_residuals) = dual_atom_spectra(
        &ATTRIBUTION_CHANNELS,
        coefficient_frames,
        positive_bins,
        &tightened_filters,
    );
    let mut planner = FftPlanner::<f64>::new();
    let inverse = planner.plan_fft_inverse(ATTRIBUTION_FFT_FRAMES);
    let mut atoms = Vec::with_capacity(30);
    for (channel_position, channel) in ATTRIBUTION_CHANNELS.into_iter().enumerate() {
        for (stage, positive) in [
            (
                StretchCommonGridTailStage::RawAnalysis,
                filter_spectrum(channel, positive_bins, &raw_filters),
            ),
            (
                StretchCommonGridTailStage::TightenedAnalysis,
                filter_spectrum(channel, positive_bins, &tightened_filters),
            ),
            (
                StretchCommonGridTailStage::CanonicalDual,
                dual_spectra[channel_position].clone(),
            ),
        ] {
            for form in [
                StretchCommonGridTailForm::Analytic,
                StretchCommonGridTailForm::RealMirrored,
            ] {
                atoms.push(measure_atom(
                    channel,
                    stage,
                    form,
                    &positive,
                    if stage == StretchCommonGridTailStage::CanonicalDual {
                        dual_residuals[channel_position]
                    } else {
                        0.0
                    },
                    &inverse,
                ));
            }
        }
    }
    let tightening_ratios = ATTRIBUTION_CHANNELS
        .iter()
        .map(|channel| {
            ratio(
                tail(
                    &atoms,
                    *channel,
                    StretchCommonGridTailStage::TightenedAnalysis,
                    StretchCommonGridTailForm::RealMirrored,
                ),
                tail(
                    &atoms,
                    *channel,
                    StretchCommonGridTailStage::RawAnalysis,
                    StretchCommonGridTailForm::RealMirrored,
                ),
            )
        })
        .collect();
    let dualization_ratios = ATTRIBUTION_CHANNELS
        .iter()
        .map(|channel| {
            ratio(
                tail(
                    &atoms,
                    *channel,
                    StretchCommonGridTailStage::CanonicalDual,
                    StretchCommonGridTailForm::RealMirrored,
                ),
                tail(
                    &atoms,
                    *channel,
                    StretchCommonGridTailStage::TightenedAnalysis,
                    StretchCommonGridTailForm::RealMirrored,
                ),
            )
        })
        .collect();
    let mirroring_ratios = ATTRIBUTION_CHANNELS
        .iter()
        .flat_map(|channel| {
            [
                StretchCommonGridTailStage::RawAnalysis,
                StretchCommonGridTailStage::TightenedAnalysis,
                StretchCommonGridTailStage::CanonicalDual,
            ]
            .map(|stage| {
                ratio(
                    tail(
                        &atoms,
                        *channel,
                        stage,
                        StretchCommonGridTailForm::RealMirrored,
                    ),
                    tail(&atoms, *channel, stage, StretchCommonGridTailForm::Analytic),
                )
            })
        })
        .collect();
    let lowpass_tail = tail(
        &atoms,
        0,
        StretchCommonGridTailStage::CanonicalDual,
        StretchCommonGridTailForm::RealMirrored,
    );
    let lowpass_to_first_wavelet_ratio = ratio(
        lowpass_tail,
        tail(
            &atoms,
            16,
            StretchCommonGridTailStage::CanonicalDual,
            StretchCommonGridTailForm::RealMirrored,
        ),
    );
    let lowpass_to_interior_ratio = ratio(
        lowpass_tail,
        tail(
            &atoms,
            768,
            StretchCommonGridTailStage::CanonicalDual,
            StretchCommonGridTailForm::RealMirrored,
        ),
    );
    let max_dual_residual = dual_residuals.iter().copied().fold(0.0_f64, f64::max);
    let non_finite_values = atoms.iter().map(|atom| atom.non_finite_values).sum();
    let mut report_hash = HASH_OFFSET;
    for atom in &atoms {
        hash_u64(&mut report_hash, atom.atom_hash);
    }
    StretchCommonGridTailAttributionEvidence {
        probe_fft_frames: ATTRIBUTION_FFT_FRAMES,
        radii_frames: ATTRIBUTION_RADII.to_vec(),
        thresholds: ATTRIBUTION_THRESHOLDS.to_vec(),
        atoms,
        tightening_ratios,
        dualization_ratios,
        mirroring_ratios,
        lowpass_to_first_wavelet_ratio,
        lowpass_to_interior_ratio,
        max_dual_residual,
        non_finite_values,
        report_hash,
    }
}

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
    let (mut spectra, residuals) =
        dual_atom_spectra(&[channel], coefficient_frames, positive_bins, filters);
    (spectra.remove(0), residuals[0])
}

fn dual_atom_spectra(
    channels: &[usize],
    coefficient_frames: usize,
    positive_bins: usize,
    filters: &[Complex64],
) -> (Vec<Vec<Complex64>>, Vec<f64>) {
    let mut spectra = vec![vec![Complex64::new(0.0, 0.0); (positive_bins - 1) * 2]; channels.len()];
    let mut max_residuals = vec![0.0_f64; channels.len()];
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
        for ((spectrum, channel), max_residual) in
            spectra.iter_mut().zip(channels).zip(&mut max_residuals)
        {
            let rhs = bins
                .iter()
                .map(|bin| filters[*channel * positive_bins + *bin])
                .collect::<Vec<_>>();
            let (solution, residual) = conjugate_gradient(&frame, &rhs, size);
            *max_residual = max_residual.max(residual);
            for (bin, value) in bins.iter().copied().zip(solution) {
                spectrum[bin] = value;
            }
        }
    }
    (spectra, max_residuals)
}

fn filter_spectrum(channel: usize, positive_bins: usize, filters: &[Complex64]) -> Vec<Complex64> {
    let mut spectrum = vec![Complex64::new(0.0, 0.0); (positive_bins - 1) * 2];
    spectrum[..positive_bins]
        .copy_from_slice(&filters[channel * positive_bins..(channel + 1) * positive_bins]);
    spectrum
}

fn measure_atom(
    channel: usize,
    stage: StretchCommonGridTailStage,
    form: StretchCommonGridTailForm,
    positive: &[Complex64],
    dual_residual: f64,
    inverse: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> StretchCommonGridTailAtomEvidence {
    let mut spectrum = positive.to_vec();
    if form == StretchCommonGridTailForm::RealMirrored {
        for bin in 1..ATTRIBUTION_FFT_FRAMES / 2 {
            spectrum[ATTRIBUTION_FFT_FRAMES - bin] = spectrum[bin].conj();
        }
        spectrum[0].im = 0.0;
        spectrum[ATTRIBUTION_FFT_FRAMES / 2].im = 0.0;
    }
    let mut non_finite_values = spectrum
        .iter()
        .filter(|value| !value.re.is_finite() || !value.im.is_finite())
        .count();
    inverse.process(&mut spectrum);
    let mut atom_hash = HASH_OFFSET;
    for value in &mut spectrum {
        *value /= ATTRIBUTION_FFT_FRAMES as f64;
        hash_u64(&mut atom_hash, value.re.to_bits());
        hash_u64(&mut atom_hash, value.im.to_bits());
    }
    non_finite_values += spectrum
        .iter()
        .filter(|value| !value.re.is_finite() || !value.im.is_finite())
        .count();
    let peak_frame = spectrum
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.norm_sqr().total_cmp(&right.1.norm_sqr()))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let total_energy = spectrum.iter().map(Complex64::norm_sqr).sum::<f64>();
    let tail_energy_ratios = ATTRIBUTION_RADII
        .iter()
        .map(|radius| tail_energy_ratio(&spectrum, peak_frame, *radius, total_energy))
        .collect();
    let guard_lower_bounds = ATTRIBUTION_THRESHOLDS
        .iter()
        .map(|threshold| guard_lower_bound(&spectrum, peak_frame, total_energy, *threshold))
        .collect();
    StretchCommonGridTailAtomEvidence {
        channel,
        stage,
        form,
        peak_frame,
        total_energy,
        tail_energy_ratios,
        guard_lower_bounds,
        dual_residual,
        non_finite_values,
        atom_hash,
    }
}

fn tail_energy_ratio(atom: &[Complex64], peak: usize, radius: usize, total: f64) -> f64 {
    if total <= f64::MIN_POSITIVE {
        return 0.0;
    }
    ((total - circular_energy(atom, peak, radius)).max(0.0)) / total
}

fn guard_lower_bound(atom: &[Complex64], peak: usize, total: f64, threshold: f64) -> usize {
    for radius in (0..=16_000).step_by(HOP) {
        if tail_energy_ratio(atom, peak, radius, total) <= threshold {
            return radius + HOP;
        }
    }
    GUARD_CAP_FRAMES + HOP
}

fn tail(
    atoms: &[StretchCommonGridTailAtomEvidence],
    channel: usize,
    stage: StretchCommonGridTailStage,
    form: StretchCommonGridTailForm,
) -> f64 {
    atoms
        .iter()
        .find(|atom| atom.channel == channel && atom.stage == stage && atom.form == form)
        .and_then(|atom| atom.tail_energy_ratios.last().copied())
        .unwrap_or(f64::NAN)
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        f64::INFINITY
    } else {
        numerator / denominator
    }
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
