mod contribution;
mod entry;
mod overlap;
mod peak_region;
mod recurrence;
mod report;
mod synthesis_trace;
mod trace;
mod tracked_peak;

use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::{
    analyse, coherent_representation, constrain_real_edges, synthesise, TransformGrid,
    HORIZONTAL_ENERGY_FLOOR,
};
pub(super) use contribution::{
    CoefficientAblation, CoefficientClassEvidence, CoefficientContributionTrace,
};
use contribution::{CoefficientTraceSpec, CoefficientTraceState, ContributionFrame};
pub(super) use entry::{
    linked, linked_analytic, linked_analytic_with_relation_oracle, linked_independent,
    linked_peak_regions, linked_tracked_peaks, linked_with_coefficient_trace,
    linked_with_relation_oracle, linked_with_synthesis_trace,
};
use overlap::{Overlap, SynthesisMode};
use peak_region::PeakMap;
use recurrence::{reference_relative_bin, reference_relative_bin_with_oracle};
use report::finish;
pub(super) use report::StereoRender;
pub(super) use synthesis_trace::SynthesisRelationTrace;
use synthesis_trace::{SynthesisTraceSpec, SynthesisTraceState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecurrenceMode {
    ReferenceRelative,
    Independent,
    PeakRegion,
    TrackedPeak,
}

fn linked_inner(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    channel_one_phase_offset: Option<f64>,
    synthesis_trace_spec: Option<SynthesisTraceSpec>,
    coefficient_trace_spec: Option<CoefficientTraceSpec>,
    synthesis_mode: SynthesisMode,
    recurrence_mode: RecurrenceMode,
) -> StereoRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    if ratio == 1.0 {
        return finish(
            [inputs[0].to_vec(), inputs[1].to_vec()],
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
            None,
            [0; 4],
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
    let mut previous_input = (recurrence_mode == RecurrenceMode::TrackedPeak).then(|| {
        [
            vec![Complex64::new(0.0, 0.0); bins],
            vec![Complex64::new(0.0, 0.0); bins],
        ]
    });
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
    let mut coefficient_trace = coefficient_trace_spec.map(CoefficientTraceState::new);
    let mut previous_reference = vec![None; bins];
    let mut previous_peak_maps: Option<[PeakMap; 2]> = None;
    let mut peak_region_counts = [0; 4];
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
        let significant_energy = current
            .iter()
            .flat_map(|spectrum| spectrum.iter())
            .map(Complex64::norm_sqr)
            .fold(0.0, f64::max)
            * 1.0e-8;
        let mut contribution_frame = ContributionFrame::new(coefficient_trace.is_some(), bins);
        let peak_maps = matches!(
            recurrence_mode,
            RecurrenceMode::Independent | RecurrenceMode::PeakRegion | RecurrenceMode::TrackedPeak
        )
        .then(|| std::array::from_fn(|channel| PeakMap::new(&current[channel])));

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
            let mut corrected = preliminary.clone();

            match recurrence_mode {
                RecurrenceMode::ReferenceRelative | RecurrenceMode::TrackedPeak => {
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
                            previous_reference[bin]
                                .is_some_and(|before| before != result.reference),
                        );
                        previous_reference[bin] = Some(result.reference);
                        if result.corrected {
                            shared_corrected += 1;
                        } else {
                            shared_fallback += 1;
                        }
                        contribution_frame.record_recurrence(bin, result.corrected);
                        unilateral_non_silent_completions +=
                            usize::from(result.unilateral_non_silent_completion);
                        for channel in 0..2 {
                            corrected[channel][bin] = result.output[channel];
                        }
                    }
                    if recurrence_mode == RecurrenceMode::TrackedPeak {
                        let frame = tracked_peak::advance(
                            peak_maps.as_ref().expect("peak maps"),
                            previous_peak_maps.as_ref(),
                            &current,
                            previous_input.as_ref().expect("previous input"),
                            &previous_output,
                            &previous_input_energy,
                            &corrected,
                            input_hop,
                            hop,
                            transform_length,
                        );
                        corrected = frame.output;
                        for (total, value) in peak_region_counts.iter_mut().zip(frame.counts) {
                            *total += value;
                        }
                    }
                }
                RecurrenceMode::Independent | RecurrenceMode::PeakRegion => {
                    let frame = peak_region::advance(
                        peak_maps.as_ref().expect("peak maps"),
                        previous_peak_maps.as_ref(),
                        bins,
                        long_distance,
                        time_factor,
                        &current,
                        &preliminary,
                        recurrence_mode == RecurrenceMode::PeakRegion,
                    );
                    corrected = frame.output;
                    shared_corrected += frame.corrected;
                    shared_fallback += frame.fallback;
                    reference_bins[0] += frame.reference_bins[0];
                    reference_bins[1] += frame.reference_bins[1];
                    active_reference_ties += frame.active_ties;
                    for (bin, reference) in frame.references.iter().copied().enumerate() {
                        reference_switches += usize::from(
                            reference.is_some()
                                && previous_reference[bin].is_some()
                                && previous_reference[bin] != reference,
                        );
                        previous_reference[bin] = reference;
                        contribution_frame.record_recurrence(bin, frame.bin_corrected[bin]);
                    }
                    for (total, value) in peak_region_counts.iter_mut().zip(frame.counts) {
                        *total += value;
                    }
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
        contribution_frame.finish(
            &mut coefficient_trace,
            &current,
            &mut next,
            significant_energy,
        );

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
        if let Some(previous_input) = &mut previous_input {
            *previous_input = current.clone();
        }
        if let Some(trace) = &mut synthesis_trace {
            trace.complete_frame(output_center, target_len, length);
        }
        previous_source_center = Some(source_center);
        if let Some(peak_maps) = peak_maps {
            previous_peak_maps = Some(peak_maps);
        }
        output_center += hop as isize;
    }

    if let Some(trace) = &mut synthesis_trace {
        trace.record_accumulated(overlap.real_output());
    }
    let uncovered = overlap.uncovered();
    let output = overlap.finish();
    let synthesis_relation_trace = synthesis_trace.map(|mut trace| {
        trace.record_normalized(&output);
        trace.finish()
    });
    let coefficient_contribution_trace = coefficient_trace.map(CoefficientTraceState::finish);
    finish(
        output,
        uncovered,
        shared_corrected,
        shared_fallback,
        unilateral_non_silent_completions,
        reference_bins,
        active_reference_ties,
        reference_switches,
        maximum_projected_relation_error,
        maximum_constrained_relation_error,
        synthesis_relation_trace,
        coefficient_contribution_trace,
        peak_region_counts,
    )
}
