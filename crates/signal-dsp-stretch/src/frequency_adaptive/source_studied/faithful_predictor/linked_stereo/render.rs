mod recurrence;

use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{
    analyse, coherent_representation, constrain_real_edges, hash_samples, synthesise,
    TransformGrid, HORIZONTAL_ENERGY_FLOOR,
};
use recurrence::reference_relative_bin;

#[derive(Clone, Debug)]
pub(super) struct StereoRender {
    pub(super) channels: [Vec<f64>; 2],
    pub(super) uncovered: usize,
    pub(super) non_finite: usize,
    pub(super) boundary_failures: usize,
    pub(super) shared_corrected: usize,
    pub(super) shared_fallback: usize,
    pub(super) unilateral_non_silent_completions: usize,
    pub(super) reference_bins: [usize; 2],
    pub(super) active_reference_ties: usize,
    pub(super) reference_switches: usize,
    pub(super) hash: u64,
}

pub(super) fn linked(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    if ratio == 1.0 {
        return finish(
            [inputs[0].to_vec(), inputs[1].to_vec()],
            0,
            0,
            0,
            0,
            0,
            0,
            [0; 2],
            0,
            0,
        );
    }

    let target_len = (inputs[0].len() as f64 * ratio).round() as usize;
    let [length, hop, transform_length, bins] =
        coherent_representation::source_geometry(sample_rate);
    let window = coherent_representation::source_kaiser_window(length, hop);
    let grid = TransformGrid::ModifiedHalfBin;
    let long_distance = ((transform_length as f64 / hop as f64).round() as usize).max(1);
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(transform_length);
    let inverse = planner.plan_fft_inverse(transform_length);
    let mut output = [vec![0.0; target_len], vec![0.0; target_len]];
    let mut normalization = [vec![0.0; target_len], vec![0.0; target_len]];
    let mut previous_output = [
        vec![Complex64::new(0.0, 0.0); bins],
        vec![Complex64::new(0.0, 0.0); bins],
    ];
    let mut previous_input_energy = [vec![0.0_f64; bins], vec![0.0_f64; bins]];
    let mut previous_source_center: Option<isize> = None;
    let mut shared_corrected = 0;
    let mut shared_fallback = 0;
    let mut unilateral_non_silent_completions = 0;
    let mut reference_bins = [0; 2];
    let mut active_reference_ties = 0;
    let mut reference_switches = 0;
    let mut previous_reference = vec![None; bins];
    let mut output_center = -(length as isize / 2);

    while output_center < target_len as isize + length as isize / 2 {
        let source_center = (output_center as f64 / ratio).round() as isize;
        let current: [Vec<Complex64>; 2] = std::array::from_fn(|channel| {
            analyse(
                inputs[channel],
                source_center,
                &window,
                transform_length,
                grid,
                &forward,
            )
        });
        let auxiliary: [Vec<Complex64>; 2] = std::array::from_fn(|channel| {
            analyse(
                inputs[channel],
                source_center - hop as isize,
                &window,
                transform_length,
                grid,
                &forward,
            )
        });
        let mut next = current.clone();

        if let Some(previous_center) = previous_source_center {
            let mut preliminary = current.clone();
            for channel in 0..2 {
                for bin in 0..bins {
                    let prediction = previous_output[channel][bin]
                        * current[channel][bin]
                        * auxiliary[channel][bin].conj();
                    let current_energy = current[channel][bin].norm_sqr();
                    let denominator = previous_input_energy[channel][bin].max(current_energy)
                        + HORIZONTAL_ENERGY_FLOOR;
                    preliminary[channel][bin] = prediction / denominator;
                }
            }
            let input_hop = (source_center - previous_center).unsigned_abs().max(1);
            let time_factor = hop as f64 / input_hop as f64;
            let significant_energy = current
                .iter()
                .flat_map(|spectrum| spectrum.iter())
                .map(Complex64::norm_sqr)
                .fold(0.0, f64::max)
                * 1.0e-8;
            let mut corrected = preliminary.clone();

            for bin in 0..bins {
                let result = reference_relative_bin(
                    bin,
                    bins,
                    long_distance,
                    time_factor,
                    &current,
                    &preliminary,
                    &corrected,
                    significant_energy,
                );
                reference_bins[result.reference] += 1;
                active_reference_ties += usize::from(result.active_tie);
                reference_switches += usize::from(
                    previous_reference[bin].is_some_and(|before| before != result.reference),
                );
                previous_reference[bin] = Some(result.reference);
                if result.corrected {
                    shared_corrected += 1;
                } else {
                    shared_fallback += 1;
                }
                unilateral_non_silent_completions +=
                    usize::from(result.unilateral_non_silent_completion);
                for channel in 0..2 {
                    corrected[channel][bin] = result.output[channel];
                }
            }
            next = corrected;
        }

        for channel in 0..2 {
            for bin in 0..bins {
                previous_input_energy[channel][bin] = current[channel][bin].norm_sqr();
            }
            constrain_real_edges(&mut next[channel], grid);
            let frame = synthesise(&next[channel], length, transform_length, grid, &inverse);
            for offset in 0..length {
                let output_index = output_center - length as isize / 2 + offset as isize;
                if (0..target_len as isize).contains(&output_index) {
                    let output_index = output_index as usize;
                    output[channel][output_index] +=
                        frame[offset] * window[offset] / transform_length as f64;
                    normalization[channel][output_index] += window[offset] * window[offset];
                }
            }
            previous_output[channel] = next[channel].clone();
        }
        previous_source_center = Some(source_center);
        output_center += hop as isize;
    }

    let uncovered = normalization
        .iter()
        .flat_map(|channel| channel.iter())
        .filter(|weight| **weight <= 0.0)
        .count();
    for channel in 0..2 {
        for (sample, weight) in output[channel].iter_mut().zip(&normalization[channel]) {
            if *weight > 0.0 {
                *sample /= *weight;
            }
        }
    }
    let non_finite = output
        .iter()
        .flat_map(|channel| channel.iter())
        .filter(|sample| !sample.is_finite())
        .count();
    let boundary_failures = output
        .iter()
        .map(|channel| {
            usize::from(channel.first().is_none_or(|sample| !sample.is_finite()))
                + usize::from(channel.last().is_none_or(|sample| !sample.is_finite()))
        })
        .sum();
    finish(
        output,
        uncovered,
        non_finite,
        boundary_failures,
        shared_corrected,
        shared_fallback,
        unilateral_non_silent_completions,
        reference_bins,
        active_reference_ties,
        reference_switches,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish(
    channels: [Vec<f64>; 2],
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    shared_corrected: usize,
    shared_fallback: usize,
    unilateral_non_silent_completions: usize,
    reference_bins: [usize; 2],
    active_reference_ties: usize,
    reference_switches: usize,
) -> StereoRender {
    let mut hash = hash_samples(&channels[0]);
    super::hash_values(&mut hash, &[hash_samples(&channels[1])]);
    StereoRender {
        channels,
        uncovered,
        non_finite,
        boundary_failures,
        shared_corrected,
        shared_fallback,
        unilateral_non_silent_completions,
        reference_bins,
        active_reference_ties,
        reference_switches,
        hash,
    }
}
