#[derive(Clone, Copy, Debug)]
pub(super) struct Region {
    pub(super) first: usize,
    pub(super) end: usize,
    pub(super) peak: usize,
}

#[derive(Clone, Debug)]
pub(super) struct RegionState {
    pub(super) region: Region,
    pub(super) owner: usize,
    pub(super) rotation: f64,
    pub(super) analysis_phases: [f64; 2],
    pub(super) analysis_energies: [f64; 2],
}

pub(super) fn regions(energy: &[f64]) -> Vec<Region> {
    let mut peaks = (0..energy.len())
        .filter(|bin| is_peak(energy, *bin))
        .collect::<Vec<_>>();
    if peaks.is_empty() {
        let peak = energy
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(bin, _)| bin)
            .expect("non-empty spectrum");
        peaks.push(peak);
    }
    let boundaries = peaks
        .windows(2)
        .map(|pair| {
            (pair[0] + 1..pair[1])
                .min_by(|left, right| energy[*left].total_cmp(&energy[*right]))
                .unwrap_or(pair[0])
        })
        .collect::<Vec<_>>();
    peaks
        .iter()
        .enumerate()
        .map(|(index, peak)| Region {
            first: index
                .checked_sub(1)
                .map_or(0, |prior| boundaries[prior] + 1),
            end: boundaries
                .get(index)
                .map_or(energy.len(), |boundary| boundary + 1),
            peak: *peak,
        })
        .collect()
}

fn is_peak(energy: &[f64], bin: usize) -> bool {
    let value = energy[bin];
    if value == 0.0 {
        return false;
    }
    let first = bin.saturating_sub(2);
    let end = (bin + 3).min(energy.len());
    !(first..end).any(|other| {
        other != bin && (energy[other] > value || (other < bin && energy[other] == value))
    })
}

pub(super) fn tracked_rotation(
    prior: &RegionState,
    current_peak: usize,
    owner: usize,
    current_phase: f64,
    analysis_hop: usize,
    synthesis_hop: usize,
    transform_length: usize,
) -> f64 {
    let prior_frequency =
        std::f64::consts::TAU * (prior.region.peak as f64 + 0.5) / transform_length as f64;
    let current_frequency =
        std::f64::consts::TAU * (current_peak as f64 + 0.5) / transform_length as f64;
    let expected = (prior_frequency + current_frequency) * 0.5 * analysis_hop as f64;
    let observed = expected + wrap(current_phase - prior.analysis_phases[owner] - expected);
    let synthesis_phase = prior.analysis_phases[owner]
        + prior.rotation
        + observed * synthesis_hop as f64 / analysis_hop as f64;
    wrap(synthesis_phase - current_phase)
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_ties_choose_the_lower_bin_and_regions_cover_once() {
        let energy = [0.0, 2.0, 2.0, 0.1, 0.1, 3.0, 0.0];
        let regions = regions(&energy);
        assert_eq!(
            regions.iter().map(|region| region.peak).collect::<Vec<_>>(),
            [1, 5]
        );
        assert_eq!(regions[0].first, 0);
        assert_eq!(regions[0].end, 4);
        assert_eq!(regions.last().expect("region").end, energy.len());
        assert!(regions.windows(2).all(|pair| pair[0].end == pair[1].first));
    }

    #[test]
    fn tracked_rotation_is_owner_change_safe_at_identity_hops() {
        let prior = RegionState {
            region: Region {
                first: 0,
                end: 8,
                peak: 3,
            },
            owner: 0,
            rotation: 0.4,
            analysis_phases: [0.2, -0.7],
            analysis_energies: [1.0, 1.0],
        };
        let current_phase = -0.7 + std::f64::consts::TAU * 3.5 * 4.0 / 16.0;
        let rotation = tracked_rotation(&prior, 3, 1, current_phase, 4, 4, 16);
        assert!((wrap(rotation - prior.rotation)).abs() <= 1.0e-12);
    }
}
