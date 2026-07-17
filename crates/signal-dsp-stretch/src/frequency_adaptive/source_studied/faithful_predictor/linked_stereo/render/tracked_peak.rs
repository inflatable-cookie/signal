use rustfft::num_complex::Complex64;

use super::super::super::HORIZONTAL_ENERGY_FLOOR;
use super::tracked_peak_trace::{record_phase_field, TrackedPeakPhaseTrace};
use super::PeakMap;

pub(super) struct FrameResult {
    pub(super) output: [Vec<Complex64>; 2],
    pub(super) counts: [usize; 4],
    pub(super) trace: TrackedPeakPhaseTrace,
}

pub(super) fn advance(
    peak_maps: &[PeakMap; 2],
    previous_peak_maps: Option<&[PeakMap; 2]>,
    current: &[Vec<Complex64>; 2],
    previous_analysis: &[Vec<Complex64>; 2],
    previous_output: &[Vec<Complex64>; 2],
    previous_input_energy: &[Vec<f64>; 2],
    relational: &[Vec<Complex64>; 2],
    input_hop: usize,
    synthesis_hop: usize,
    transform_length: usize,
) -> FrameResult {
    let mut output = relational.clone();
    let Some(previous_peak_maps) = previous_peak_maps else {
        return FrameResult {
            counts: [regions(peak_maps), 0, 0, current[0].len()],
            output,
            trace: TrackedPeakPhaseTrace::default(),
        };
    };
    let region_states: Vec<_> = (0..current[0].len())
        .map(|bin| {
            let pair = [peak_maps[0].owner(bin), peak_maps[1].owner(bin)];
            let predecessors = [
                pair[0].and_then(|peak| previous_peak_maps[0].owner(peak)),
                pair[1].and_then(|peak| previous_peak_maps[1].owner(peak)),
            ];
            (predecessors[0].is_some() && predecessors[0] == predecessors[1]).then_some(pair)
        })
        .collect();
    let mut eligible_regions = 0;
    let mut overlaid_bins = 0;
    let mut previous_eligible_pair = None;
    let mut trace = TrackedPeakPhaseTrace::default();
    for bin in 0..current[0].len() {
        let pair = [peak_maps[0].owner(bin), peak_maps[1].owner(bin)];
        let predecessors = [
            pair[0].and_then(|peak| previous_peak_maps[0].owner(peak)),
            pair[1].and_then(|peak| previous_peak_maps[1].owner(peak)),
        ];
        let eligible = predecessors[0].is_some() && predecessors[0] == predecessors[1];
        if eligible && previous_eligible_pair != Some(pair) {
            eligible_regions += 1;
        }
        previous_eligible_pair = eligible.then_some(pair);
        if !eligible {
            continue;
        }
        let predecessor = predecessors[0].expect("eligible predecessor");
        let mut changed = false;
        let mut applied = [false; 2];
        for channel in 0..2 {
            let Some(peak) = pair[channel] else {
                continue;
            };
            let reference = usize::from(current[1][peak].norm_sqr() > current[0][peak].norm_sqr());
            let Some(anchor) = tracked_anchor(
                reference,
                peak,
                predecessor,
                current,
                previous_analysis,
                previous_output,
                previous_input_energy,
                input_hop,
                synthesis_hop,
                transform_length,
            ) else {
                continue;
            };
            let peak_input = current[reference][peak];
            let target_energy = current[channel][bin].norm_sqr();
            if peak_input.norm_sqr() == 0.0 || target_energy == 0.0 {
                continue;
            }
            let projected = anchor * current[channel][bin] * peak_input.conj();
            let projected_energy = projected.norm_sqr();
            if projected_energy > target_energy * f64::EPSILON * 64.0 {
                output[channel][bin] = projected * (target_energy / projected_energy).sqrt();
                changed = true;
                applied[channel] = true;
            }
        }
        record_phase_field(
            &mut trace,
            bin,
            pair,
            &region_states,
            applied,
            current,
            relational,
            &output,
        );
        overlaid_bins += usize::from(changed);
    }
    let bins = current[0].len();
    FrameResult {
        output,
        counts: [
            regions(peak_maps),
            eligible_regions,
            overlaid_bins,
            bins.saturating_sub(overlaid_bins),
        ],
        trace,
    }
}

pub(super) fn tracked_anchor(
    reference: usize,
    current_peak: usize,
    predecessor: usize,
    current: &[Vec<Complex64>; 2],
    previous_analysis: &[Vec<Complex64>; 2],
    previous_output: &[Vec<Complex64>; 2],
    previous_input_energy: &[Vec<f64>; 2],
    input_hop: usize,
    synthesis_hop: usize,
    transform_length: usize,
) -> Option<Complex64> {
    let target_energy = current[reference][current_peak].norm_sqr();
    let previous_energy = previous_input_energy[reference][predecessor];
    let output_energy = previous_output[reference][predecessor].norm_sqr();
    if target_energy == 0.0 || previous_energy == 0.0 || output_energy <= HORIZONTAL_ENERGY_FLOOR {
        return None;
    }
    let analysis_increment =
        (current[reference][current_peak] * previous_analysis[reference][predecessor].conj()).arg();
    let angular_frequency =
        std::f64::consts::TAU * (current_peak as f64 + 0.5) / transform_length as f64;
    let expected = angular_frequency * input_hop as f64;
    let deviation = wrap(analysis_increment - expected);
    let synthesis_increment = angular_frequency * synthesis_hop as f64
        + deviation * synthesis_hop as f64 / input_hop as f64;
    Some(
        previous_output[reference][predecessor]
            * Complex64::from_polar((target_energy / output_energy).sqrt(), synthesis_increment),
    )
}

fn wrap(phase: f64) -> f64 {
    (phase + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

pub(super) fn regions(peak_maps: &[PeakMap; 2]) -> usize {
    let mut count = 0;
    let mut previous = None;
    for bin in 0..peak_maps[0].owners.len() {
        let pair = [peak_maps[0].owner(bin), peak_maps[1].owner(bin)];
        if previous != Some(pair) {
            count += 1;
            previous = Some(pair);
        }
    }
    count
}
