use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    super::{analyse, coherent_representation, constrain_real_edges, synthesise},
    render::StereoRender,
    shared_rotation_region_locked::{
        output::{add_overlap, finish},
        phase::{regions, tracked_rotation, RegionState},
        SharedRotationRender, StateCounts,
    },
};
use crate::frequency_adaptive::source_studied::faithful_predictor::TransformGrid;

pub(in crate::frequency_adaptive) fn render(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
) -> SharedRotationRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "non-empty linked input");
    assert!(ratio.is_finite() && ratio > 0.0, "positive finite ratio");
    if ratio == 1.0 {
        return finish(
            [inputs[0].to_vec(), inputs[1].to_vec()],
            inputs[0].len(),
            0,
            StateCounts::default(),
        );
    }

    let target_length = (inputs[0].len() as f64 * ratio).round() as usize;
    let [support_length, synthesis_hop, transform_length, bins] =
        coherent_representation::source_geometry(sample_rate);
    let window = coherent_representation::source_kaiser_window(support_length, synthesis_hop);
    let grid = TransformGrid::ModifiedHalfBin;
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(transform_length);
    let inverse = planner.plan_fft_inverse(transform_length);
    let mut output = std::array::from_fn(|_| vec![0.0; target_length]);
    let mut normalization: [Vec<f64>; 2] = std::array::from_fn(|_| vec![0.0; target_length]);
    let mut previous_regions = Vec::<RegionState>::new();
    let mut previous_source_center = None;
    let mut states = StateCounts::default();
    let mut output_center = -(support_length as isize / 2);

    while output_center < target_length as isize + support_length as isize / 2 {
        let source_center = (output_center as f64 / ratio).round() as isize;
        let finite_support_reset =
            support_crosses_input(source_center, support_length, inputs[0].len());
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
        let energy = (0..bins)
            .map(|bin| current[0][bin].norm_sqr().max(current[1][bin].norm_sqr()))
            .collect::<Vec<_>>();
        let mut next = current.clone();
        let mut next_regions = Vec::new();

        if energy.iter().all(|value| *value == 0.0) {
            next.iter_mut()
                .flatten()
                .for_each(|value| *value = Complex64::default());
            states.silent += 1;
        } else {
            let regions = regions(&energy);
            states.regions += regions.len();
            if finite_support_reset {
                states.reset += regions.len();
            } else {
                let continuous =
                    previous_source_center.is_some_and(|previous| source_center > previous);
                let analysis_hop = previous_source_center
                    .map(|previous| (source_center - previous).unsigned_abs())
                    .unwrap_or(0);

                for region in regions {
                    let owner = usize::from(
                        current[1][region.peak].norm_sqr() > current[0][region.peak].norm_sqr(),
                    );
                    let current_phases =
                        std::array::from_fn(|channel| current[channel][region.peak].arg());
                    let current_energies =
                        std::array::from_fn(|channel| current[channel][region.peak].norm_sqr());
                    let predecessor = continuous
                        .then(|| {
                            previous_regions.iter().find(|prior| {
                                (prior.region.first..prior.region.end).contains(&region.peak)
                            })
                        })
                        .flatten();
                    let rotation = predecessor
                        .filter(|prior| analysis_hop > 0 && prior.analysis_energies[owner] > 0.0)
                        .map(|prior| {
                            tracked_rotation(
                                prior,
                                region.peak,
                                owner,
                                current_phases[owner],
                                analysis_hop,
                                synthesis_hop,
                                transform_length,
                            )
                        })
                        .unwrap_or(0.0);
                    if predecessor.is_some_and(|prior| {
                        analysis_hop > 0 && prior.analysis_energies[owner] > 0.0
                    }) {
                        states.tracked += 1;
                        states.owner_switches +=
                            usize::from(predecessor.is_some_and(|prior| prior.owner != owner));
                    } else {
                        states.reset += 1;
                    }
                    let operator = Complex64::from_polar(1.0, rotation);
                    for channel in 0..2 {
                        for bin in region.first..region.end {
                            next[channel][bin] = current[channel][bin] * operator;
                        }
                    }
                    next_regions.push(RegionState {
                        region,
                        owner,
                        rotation,
                        analysis_phases: current_phases,
                        analysis_energies: current_energies,
                    });
                }
            }
        }

        for channel in 0..2 {
            constrain_real_edges(&mut next[channel], grid);
            let frame = synthesise(
                &next[channel],
                support_length,
                transform_length,
                grid,
                &inverse,
            );
            add_overlap(
                &mut output[channel],
                &mut normalization[channel],
                output_center,
                &frame,
                &window,
                transform_length,
            );
        }
        previous_regions = next_regions;
        previous_source_center = Some(source_center);
        output_center += synthesis_hop as isize;
    }

    let uncovered = normalization
        .iter()
        .flatten()
        .filter(|weight| **weight <= 0.0)
        .count();
    for channel in 0..2 {
        for (sample, weight) in output[channel].iter_mut().zip(&normalization[channel]) {
            if *weight > 0.0 {
                *sample /= *weight;
            }
        }
    }
    finish(output, target_length, uncovered, states)
}

fn support_crosses_input(center: isize, support_length: usize, input_length: usize) -> bool {
    let first = center - support_length as isize / 2;
    let end = first + support_length as isize;
    first < 0 || end > input_length as isize
}

pub(super) fn stereo_adapter(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    let rendered = render(inputs, ratio, sample_rate);
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: 0,
        shared_fallback: 0,
        unilateral_non_silent_completions: 0,
        reference_bins: [0; 2],
        active_reference_ties: 0,
        reference_switches: 0,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.regions,
            rendered.states.tracked,
            rendered.states.reset,
            rendered.states.silent,
        ],
        tracked_peak_phase_trace: Default::default(),
        hash: rendered.hash,
    }
}

#[cfg(test)]
mod tests {
    use super::support_crosses_input;

    #[test]
    fn finite_support_condition_matches_exact_analysis_extent() {
        assert!(support_crosses_input(3, 8, 16));
        assert!(!support_crosses_input(4, 8, 16));
        assert!(!support_crosses_input(12, 8, 16));
        assert!(support_crosses_input(13, 8, 16));
    }
}
