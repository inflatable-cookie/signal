mod entry;
mod overlap;
mod recurrence;
mod report;
mod synthesis_trace;
mod trace;

use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{
    analyse, coherent_representation, constrain_real_edges, synthesise, TransformGrid,
    HORIZONTAL_ENERGY_FLOOR,
};
pub(super) use entry::{
    linked, linked_analytic, linked_analytic_with_relation_oracle, linked_with_relation_oracle,
    linked_with_synthesis_trace,
};
use overlap::{Overlap, SynthesisMode};
use recurrence::{reference_relative_bin, reference_relative_bin_with_oracle};
use report::finish;
pub(super) use report::StereoRender;
pub(super) use synthesis_trace::SynthesisRelationTrace;
use synthesis_trace::{SynthesisTraceSpec, SynthesisTraceState};

fn linked_inner(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    channel_one_phase_offset: Option<f64>,
    synthesis_trace_spec: Option<SynthesisTraceSpec>,
    synthesis_mode: SynthesisMode,
) -> StereoRender {
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
            0.0,
            0.0,
            None,
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
    let mut overlap = Overlap::new(target_len, synthesis_mode);
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
    let mut maximum_projected_relation_error = 0.0_f64;
    let mut maximum_constrained_relation_error = 0.0_f64;
    let mut synthesis_trace =
        synthesis_trace_spec.map(|spec| SynthesisTraceState::new(spec, length));
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
                let result = if let Some(offset) = channel_one_phase_offset {
                    reference_relative_bin_with_oracle(
                        bin,
                        bins,
                        long_distance,
                        time_factor,
                        &current,
                        &preliminary,
                        &corrected,
                        significant_energy,
                        offset,
                    )
                } else {
                    reference_relative_bin(
                        bin,
                        bins,
                        long_distance,
                        time_factor,
                        &current,
                        &preliminary,
                        &corrected,
                        significant_energy,
                    )
                };
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
            let relation_errors =
                trace::relation_errors(&current, &corrected, significant_energy, grid);
            maximum_projected_relation_error =
                maximum_projected_relation_error.max(relation_errors[0]);
            maximum_constrained_relation_error =
                maximum_constrained_relation_error.max(relation_errors[1]);
            next = corrected;
        }

        for channel in 0..2 {
            for bin in 0..bins {
                previous_input_energy[channel][bin] = current[channel][bin].norm_sqr();
            }
            constrain_real_edges(&mut next[channel], grid);
            match synthesis_mode {
                SynthesisMode::Real => {
                    let frame =
                        synthesise(&next[channel], length, transform_length, grid, &inverse);
                    if let Some(trace) = &mut synthesis_trace {
                        trace.record_frame_channel(channel, &frame);
                    }
                    overlap.add_real(channel, output_center, &frame, &window, transform_length);
                }
                SynthesisMode::Analytic => {
                    let frame = overlap::synthesise_analytic(
                        &next[channel],
                        length,
                        transform_length,
                        grid,
                        &inverse,
                    );
                    overlap.add_analytic(channel, output_center, &frame, &window, transform_length);
                }
            }
            previous_output[channel] = next[channel].clone();
        }
        if let Some(trace) = &mut synthesis_trace {
            trace.complete_frame(output_center, target_len, length);
        }
        previous_source_center = Some(source_center);
        output_center += hop as isize;
    }

    if let Some(trace) = &mut synthesis_trace {
        trace.record_accumulated(overlap.real_output());
    }
    let uncovered = overlap.uncovered();
    let output = overlap.finish();
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
    let synthesis_relation_trace = synthesis_trace.map(|mut trace| {
        trace.record_normalized(&output);
        trace.finish()
    });
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
        maximum_projected_relation_error,
        maximum_constrained_relation_error,
        synthesis_relation_trace,
    )
}
