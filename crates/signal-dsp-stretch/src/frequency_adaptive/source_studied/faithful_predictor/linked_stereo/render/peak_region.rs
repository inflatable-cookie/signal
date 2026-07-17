use rustfft::num_complex::Complex64;

use super::recurrence::vertical_prediction;

#[derive(Clone)]
pub(super) struct PeakMap {
    pub(super) owners: Vec<Option<usize>>,
}

impl PeakMap {
    pub(super) fn new(spectrum: &[Complex64]) -> Self {
        let energy = spectrum.iter().map(Complex64::norm_sqr).collect::<Vec<_>>();
        let mut peaks = (0..energy.len())
            .filter(|bin| {
                let left = bin.checked_sub(1).map_or(0.0, |left| energy[left]);
                let right = energy.get(bin + 1).copied().unwrap_or(0.0);
                energy[*bin] > 0.0 && energy[*bin] > left && energy[*bin] >= right
            })
            .collect::<Vec<_>>();
        if peaks.is_empty() {
            if let Some((peak, value)) = energy
                .iter()
                .copied()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(&right.1))
            {
                if value > 0.0 {
                    peaks.push(peak);
                }
            }
        }
        let owners = (0..energy.len())
            .map(|bin| {
                peaks
                    .iter()
                    .copied()
                    .min_by_key(|peak| (peak.abs_diff(bin), *peak))
            })
            .collect();
        Self { owners }
    }

    pub(super) fn owner(&self, bin: usize) -> Option<usize> {
        self.owners.get(bin).copied().flatten()
    }
}

pub(super) struct FrameResult {
    pub(super) output: [Vec<Complex64>; 2],
    pub(super) corrected: usize,
    pub(super) fallback: usize,
    pub(super) reference_bins: [usize; 2],
    pub(super) active_ties: usize,
    pub(super) references: Vec<Option<usize>>,
    pub(super) bin_corrected: Vec<bool>,
    pub(super) counts: [usize; 4],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn advance(
    peak_maps: &[PeakMap; 2],
    previous_peak_maps: Option<&[PeakMap; 2]>,
    bins: usize,
    long_distance: usize,
    time_factor: f64,
    current: &[Vec<Complex64>; 2],
    preliminary: &[Vec<Complex64>; 2],
    share_regions: bool,
) -> FrameResult {
    let mut output = preliminary.clone();
    let mut corrected = 0;
    let mut fallback = 0;
    let mut bin_corrected = vec![false; bins];
    for channel in 0..2 {
        for bin in 0..bins {
            let prediction = vertical_prediction(
                bin,
                bins,
                long_distance,
                time_factor,
                &current[channel],
                &preliminary[channel],
                &output[channel],
            );
            let target_energy = current[channel][bin].norm_sqr();
            let prediction_energy = prediction.norm_sqr();
            if prediction_energy > target_energy * f64::EPSILON * 64.0 {
                output[channel][bin] = prediction * (target_energy / prediction_energy).sqrt();
                corrected += 1;
                bin_corrected[bin] = true;
            } else {
                output[channel][bin] = current[channel][bin];
                fallback += 1;
            }
        }
    }

    let mut references = vec![None; bins];
    let mut reference_bins = [0; 2];
    let mut active_ties = 0;
    let mut region_count = 0;
    let mut eligible_region_count = 0;
    let mut shared_bins = 0;
    let mut previous_pair = None;
    let mut previous_eligible_pair = None;
    if !share_regions {
        return FrameResult {
            output,
            corrected,
            fallback,
            reference_bins,
            active_ties,
            references,
            bin_corrected,
            counts: [0, 0, 0, bins],
        };
    }
    for bin in 0..bins {
        let pair = [peak_maps[0].owner(bin), peak_maps[1].owner(bin)];
        if previous_pair != Some(pair) {
            region_count += 1;
            previous_pair = Some(pair);
        }
        let Some(history) = previous_peak_maps else {
            previous_eligible_pair = None;
            continue;
        };
        let prior = [
            pair[0].and_then(|peak| history[0].owner(peak)),
            pair[1].and_then(|peak| history[1].owner(peak)),
        ];
        if prior[0].is_none() || prior[0] != prior[1] {
            previous_eligible_pair = None;
            continue;
        }
        if previous_eligible_pair != Some(pair) {
            eligible_region_count += 1;
            previous_eligible_pair = Some(pair);
        }
        let target_energy = [current[0][bin].norm_sqr(), current[1][bin].norm_sqr()];
        let reference = usize::from(target_energy[1] > target_energy[0]);
        let Some(anchor_bin) = pair[reference] else {
            previous_eligible_pair = None;
            continue;
        };
        let anchor_input = current[reference][anchor_bin];
        let anchor_output = output[reference][anchor_bin];
        if anchor_input.norm_sqr() == 0.0 || anchor_output.norm_sqr() == 0.0 {
            previous_eligible_pair = None;
            continue;
        }
        references[bin] = Some(reference);
        reference_bins[reference] += 1;
        active_ties += usize::from(target_energy[0] == target_energy[1] && target_energy[0] > 0.0);
        shared_bins += 1;
        for channel in 0..2 {
            if target_energy[channel] == 0.0 {
                output[channel][bin] = Complex64::new(0.0, 0.0);
                continue;
            }
            let projected = anchor_output * current[channel][bin] * anchor_input.conj();
            let projected_energy = projected.norm_sqr();
            if projected_energy > target_energy[channel] * f64::EPSILON * 64.0 {
                output[channel][bin] =
                    projected * (target_energy[channel] / projected_energy).sqrt();
                bin_corrected[bin] = true;
            }
        }
    }
    FrameResult {
        output,
        corrected,
        fallback,
        reference_bins,
        active_ties,
        references,
        bin_corrected,
        counts: [
            region_count,
            eligible_region_count,
            shared_bins,
            bins.saturating_sub(shared_bins),
        ],
    }
}
