use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    super::{analyse, coherent_representation, constrain_real_edges, synthesise},
    render::StereoRender,
    shared_rotation_region_locked::{
        output::{add_overlap, finish},
        phase::{regions, tracked_rotation, Region, RegionState},
        SharedRotationRender, StateCounts,
    },
};
use crate::frequency_adaptive::source_studied::faithful_predictor::TransformGrid;

const ENERGY_FLOOR: f64 = 1.0e-24;

/// The six policy values permitted by Rule 31M.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct Policy {
    /// Peak energy divided by mean region energy.
    pub(in crate::frequency_adaptive) peak_prominence: f64,
    /// Maximum peak displacement as a multiple of the wider region width.
    pub(in crate::frequency_adaptive) predecessor_tolerance_region_widths: f64,
    /// Region energy rise which starts a reset, measured in decibels.
    pub(in crate::frequency_adaptive) transient_rise_db: f64,
    /// Additional frames kept in reset after a detected energy rise.
    pub(in crate::frequency_adaptive) reset_support_frames: usize,
    /// Minimum normalized cross-channel region coherence for locking.
    pub(in crate::frequency_adaptive) unlock_coherence: f64,
    /// Maximum predecessor-to-current linked phase change, in radians.
    pub(in crate::frequency_adaptive) history_tolerance_radians: f64,
}

/// Frozen physical bounds and binary quantization for the bounded search.
pub(in crate::frequency_adaptive) const POLICY_LEVELS: [[f64; 2]; 6] = [
    [1.0, 2.0],
    [1.0, 0.25],
    [24.0, 6.0],
    [0.0, 1.0],
    [0.0, 0.50],
    [std::f64::consts::PI, std::f64::consts::PI / 2.0],
];

pub(in crate::frequency_adaptive) fn candidates() -> [Policy; 64] {
    std::array::from_fn(|index| Policy {
        peak_prominence: POLICY_LEVELS[0][(index >> 5) & 1],
        predecessor_tolerance_region_widths: POLICY_LEVELS[1][(index >> 4) & 1],
        transient_rise_db: POLICY_LEVELS[2][(index >> 3) & 1],
        reset_support_frames: POLICY_LEVELS[3][(index >> 2) & 1] as usize,
        unlock_coherence: POLICY_LEVELS[4][(index >> 1) & 1],
        history_tolerance_radians: POLICY_LEVELS[5][index & 1],
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Ownership {
    Reset,
    Locked,
    Unlocked,
}

#[derive(Clone, Debug)]
struct Memory {
    state: RegionState,
    energy: f64,
    linked_phase: Option<f64>,
    reset_remaining: usize,
}

pub(in crate::frequency_adaptive) fn render(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    policy: Policy,
) -> SharedRotationRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "non-empty linked input");
    assert!(ratio.is_finite() && ratio > 0.0, "positive finite ratio");
    validate(policy);
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
    let mut previous_regions = Vec::<Memory>::new();
    let mut previous_spectra: Option<[Vec<Complex64>; 2]> = None;
    let mut previous_rotations: Option<[Vec<f64>; 2]> = None;
    let mut previous_source_center = None;
    let mut states = StateCounts::default();
    let mut output_center = -(support_length as isize / 2);

    while output_center < target_length as isize + support_length as isize / 2 {
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
        let joint_energy = (0..bins)
            .map(|bin| current[0][bin].norm_sqr().max(current[1][bin].norm_sqr()))
            .collect::<Vec<_>>();
        let mut next = current.clone();
        let mut rotations = [vec![0.0; bins], vec![0.0; bins]];
        let mut next_regions = Vec::new();

        if joint_energy.iter().all(|value| *value == 0.0) {
            next.iter_mut()
                .flatten()
                .for_each(|value| *value = Complex64::default());
            states.silent += 1;
        } else {
            let frame_regions = regions(&joint_energy);
            states.regions += frame_regions.len();
            let analysis_hop = previous_source_center
                .filter(|previous| source_center > *previous)
                .map(|previous| (source_center - previous).unsigned_abs());
            let ordinary = ordinary_rotations(
                &current,
                previous_spectra.as_ref(),
                previous_rotations.as_ref(),
                analysis_hop,
                synthesis_hop,
                transform_length,
            );

            for region in frame_regions {
                let owner = usize::from(
                    current[1][region.peak].norm_sqr() > current[0][region.peak].norm_sqr(),
                );
                let phase = std::array::from_fn(|channel| current[channel][region.peak].arg());
                let energy =
                    std::array::from_fn(|channel| current[channel][region.peak].norm_sqr());
                let total_energy = joint_energy[region.first..region.end].iter().sum::<f64>();
                let linked_phase = linked_phase(current[0][region.peak], current[1][region.peak]);
                let predecessor = nearest_predecessor(
                    &previous_regions,
                    &region,
                    policy.predecessor_tolerance_region_widths,
                );
                let ownership = classify(
                    &region,
                    &joint_energy,
                    &current,
                    predecessor,
                    total_energy,
                    linked_phase,
                    policy,
                    analysis_hop.is_some(),
                );

                match ownership {
                    Ownership::Reset => {
                        states.reset += 1;
                    }
                    Ownership::Locked => {
                        states.tracked += 1;
                        states.locked += 1;
                        let predecessor = predecessor.expect("locked predecessor");
                        let rotation = tracked_rotation(
                            &predecessor.state,
                            region.peak,
                            owner,
                            phase[owner],
                            analysis_hop.expect("locked hop"),
                            synthesis_hop,
                            transform_length,
                        );
                        let operator = Complex64::from_polar(1.0, rotation);
                        for channel in 0..2 {
                            for bin in region.first..region.end {
                                next[channel][bin] = current[channel][bin] * operator;
                                rotations[channel][bin] = rotation;
                            }
                        }
                        states.owner_switches += usize::from(predecessor.state.owner != owner);
                    }
                    Ownership::Unlocked => {
                        states.diffuse += 1;
                        for channel in 0..2 {
                            for bin in region.first..region.end {
                                next[channel][bin] = current[channel][bin]
                                    * Complex64::from_polar(1.0, ordinary[channel][bin]);
                                rotations[channel][bin] = ordinary[channel][bin];
                            }
                        }
                    }
                }
                let reset_remaining = next_reset_remaining(
                    predecessor,
                    total_energy,
                    policy.transient_rise_db,
                    policy.reset_support_frames,
                );
                next_regions.push(Memory {
                    state: RegionState {
                        region,
                        owner,
                        rotation: rotations[owner][region.peak],
                        analysis_phases: phase,
                        analysis_energies: energy,
                    },
                    energy: total_energy,
                    linked_phase,
                    reset_remaining,
                });
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
        previous_spectra = Some(current);
        previous_rotations = Some(rotations);
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

pub(in crate::frequency_adaptive) fn stereo_adapter(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    policy: Policy,
) -> StereoRender {
    let rendered = render(inputs, ratio, sample_rate, policy);
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
        reference_switches: rendered.states.owner_switches,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.regions,
            rendered.states.locked,
            rendered.states.reset,
            rendered.states.diffuse,
        ],
        tracked_peak_phase_trace: Default::default(),
        hash: rendered.hash,
    }
}

fn classify(
    region: &Region,
    joint_energy: &[f64],
    spectra: &[Vec<Complex64>; 2],
    predecessor: Option<&Memory>,
    total_energy: f64,
    current_linked_phase: Option<f64>,
    policy: Policy,
    continuous: bool,
) -> Ownership {
    let Some(predecessor) = predecessor.filter(|_| continuous) else {
        return Ownership::Reset;
    };
    let rise_db =
        10.0 * ((total_energy + ENERGY_FLOOR) / (predecessor.energy + ENERGY_FLOOR)).log10();
    if predecessor.reset_remaining > 0 || rise_db >= policy.transient_rise_db {
        return Ownership::Reset;
    }
    let mean = total_energy / region.end.saturating_sub(region.first).max(1) as f64;
    let prominence = joint_energy[region.peak] / (mean + ENERGY_FLOOR);
    let history_compatible = match (predecessor.linked_phase, current_linked_phase) {
        (Some(previous), Some(current)) => {
            wrap(current - previous).abs() <= policy.history_tolerance_radians
        }
        _ => true,
    };
    if prominence >= policy.peak_prominence
        && region_coherence(spectra, region) >= policy.unlock_coherence
        && history_compatible
    {
        Ownership::Locked
    } else {
        Ownership::Unlocked
    }
}

fn nearest_predecessor<'a>(
    previous: &'a [Memory],
    current: &Region,
    tolerance_region_widths: f64,
) -> Option<&'a Memory> {
    previous
        .iter()
        .filter(|memory| {
            let prior = &memory.state.region;
            let width = (current.end - current.first).max(prior.end - prior.first) as f64;
            current.peak.abs_diff(prior.peak) as f64 <= width * tolerance_region_widths
        })
        .min_by_key(|memory| memory.state.region.peak.abs_diff(current.peak))
}

fn next_reset_remaining(
    predecessor: Option<&Memory>,
    energy: f64,
    rise_threshold_db: f64,
    support_frames: usize,
) -> usize {
    let Some(predecessor) = predecessor else {
        return 0;
    };
    if predecessor.reset_remaining > 0 {
        return predecessor.reset_remaining - 1;
    }
    let rise_db = 10.0 * ((energy + ENERGY_FLOOR) / (predecessor.energy + ENERGY_FLOOR)).log10();
    usize::from(rise_db >= rise_threshold_db) * support_frames
}

fn ordinary_rotations(
    current: &[Vec<Complex64>; 2],
    previous: Option<&[Vec<Complex64>; 2]>,
    previous_rotations: Option<&[Vec<f64>; 2]>,
    analysis_hop: Option<usize>,
    synthesis_hop: usize,
    transform_length: usize,
) -> [Vec<f64>; 2] {
    let bins = current[0].len();
    let mut rotations = [vec![0.0; bins], vec![0.0; bins]];
    let (Some(previous), Some(previous_rotations), Some(analysis_hop)) =
        (previous, previous_rotations, analysis_hop)
    else {
        return rotations;
    };
    for bin in 0..bins {
        let preferred = usize::from(current[1][bin].norm_sqr() > current[0][bin].norm_sqr());
        let owner = [preferred, 1 - preferred].into_iter().find(|channel| {
            current[*channel][bin].norm_sqr() > ENERGY_FLOOR
                && previous[*channel][bin].norm_sqr() > ENERGY_FLOOR
        });
        if let Some(owner) = owner {
            let frequency = std::f64::consts::TAU * (bin as f64 + 0.5) / transform_length as f64;
            let expected = frequency * analysis_hop as f64;
            let observed =
                expected + wrap(current[owner][bin].arg() - previous[owner][bin].arg() - expected);
            let synthesis_phase = previous[owner][bin].arg()
                + previous_rotations[owner][bin]
                + observed * synthesis_hop as f64 / analysis_hop as f64;
            let rotation = wrap(synthesis_phase - current[owner][bin].arg());
            rotations[0][bin] = rotation;
            rotations[1][bin] = rotation;
        }
    }
    rotations
}

fn linked_phase(left: Complex64, right: Complex64) -> Option<f64> {
    (left.norm_sqr() > ENERGY_FLOOR && right.norm_sqr() > ENERGY_FLOOR)
        .then(|| wrap(right.arg() - left.arg()))
}

fn region_coherence(spectra: &[Vec<Complex64>; 2], region: &Region) -> f64 {
    let mut cross = Complex64::default();
    let mut energies = [0.0; 2];
    for bin in region.first..region.end {
        cross += spectra[0][bin] * spectra[1][bin].conj();
        energies[0] += spectra[0][bin].norm_sqr();
        energies[1] += spectra[1][bin].norm_sqr();
    }
    if energies[0] <= ENERGY_FLOOR || energies[1] <= ENERGY_FLOOR {
        1.0
    } else {
        (cross.norm() / (energies[0] * energies[1]).sqrt()).clamp(0.0, 1.0)
    }
}

fn validate(policy: Policy) {
    assert!(policy.peak_prominence >= 1.0);
    assert!(policy.predecessor_tolerance_region_widths > 0.0);
    assert!(policy.transient_rise_db >= 0.0);
    assert!((0.0..=1.0).contains(&policy.unlock_coherence));
    assert!((0.0..=std::f64::consts::PI).contains(&policy.history_tolerance_radians));
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_complete_candidates_freeze_six_binary_controls_before_rendering() {
        let candidates = candidates();
        assert_eq!(candidates.len(), 64);
        assert_eq!(candidates[0].peak_prominence, POLICY_LEVELS[0][0]);
        assert_eq!(candidates[63].peak_prominence, POLICY_LEVELS[0][1]);
        for pair in candidates.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn state_complete_classifier_covers_reset_locked_and_unlocked() {
        let policy = candidates()[0];
        let spectrum = [
            vec![Complex64::new(1.0, 0.0), Complex64::new(4.0, 0.0)],
            vec![Complex64::new(1.0, 0.0), Complex64::new(4.0, 0.0)],
        ];
        let region = Region {
            first: 0,
            end: 2,
            peak: 1,
        };
        let state = RegionState {
            region,
            owner: 0,
            rotation: 0.0,
            analysis_phases: [0.0; 2],
            analysis_energies: [16.0; 2],
        };
        let memory = Memory {
            state,
            energy: 17.0,
            linked_phase: Some(0.0),
            reset_remaining: 0,
        };
        assert_eq!(
            classify(
                &region,
                &[1.0, 16.0],
                &spectrum,
                None,
                17.0,
                Some(0.0),
                policy,
                true
            ),
            Ownership::Reset
        );
        assert_eq!(
            classify(
                &region,
                &[1.0, 16.0],
                &spectrum,
                Some(&memory),
                17.0,
                Some(0.0),
                policy,
                true
            ),
            Ownership::Locked
        );
        assert_eq!(
            classify(
                &region,
                &[8.0, 9.0],
                &spectrum,
                Some(&memory),
                17.0,
                Some(1.0),
                policy,
                true
            ),
            Ownership::Unlocked
        );
    }
}
