use rustfft::num_complex::Complex64;

use super::{tracked_peak, PeakMap};

pub(super) struct FrameResult {
    pub(super) output: [Vec<Complex64>; 2],
    pub(super) counts: [usize; 4],
}

#[allow(clippy::too_many_arguments)]
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
            counts: [tracked_peak::regions(peak_maps), 0, 0, current[0].len()],
            output,
        };
    };
    let mut eligible_regions = 0;
    let mut owned_bins = 0;
    let mut previous_eligible_pair = None;
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
        let peaks = [
            pair[0].expect("eligible left peak"),
            pair[1].expect("eligible right peak"),
        ];
        let owner = usize::from(current[1][peaks[1]].norm_sqr() > current[0][peaks[0]].norm_sqr());
        let peer = 1 - owner;
        let owner_peak = peaks[owner];
        let predecessor = predecessors[owner].expect("eligible predecessor");
        let Some(anchor) = tracked_peak::tracked_anchor(
            owner,
            owner_peak,
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
        let peak_input = current[owner][owner_peak];
        let owner_input = current[owner][bin];
        let Some(owner_output) = project_energy(
            anchor * owner_input * peak_input.conj(),
            owner_input.norm_sqr(),
        ) else {
            continue;
        };
        output[owner][bin] = owner_output;
        let peer_input = current[peer][bin];
        output[peer][bin] = project_energy(
            owner_output * peer_input * owner_input.conj(),
            peer_input.norm_sqr(),
        )
        .unwrap_or(Complex64::new(0.0, 0.0));
        owned_bins += 1;
    }
    let bins = current[0].len();
    FrameResult {
        output,
        counts: [
            tracked_peak::regions(peak_maps),
            eligible_regions,
            owned_bins,
            bins.saturating_sub(owned_bins),
        ],
    }
}

fn project_energy(value: Complex64, target_energy: f64) -> Option<Complex64> {
    if target_energy == 0.0 {
        return Some(Complex64::new(0.0, 0.0));
    }
    let energy = value.norm_sqr();
    (energy > target_energy * f64::EPSILON * 64.0).then(|| value * (target_energy / energy).sqrt())
}
