use super::*;

pub(super) fn validate_request(
    channels: usize,
    signed_atoms: usize,
    positive_atoms: usize,
    coefficients: usize,
) -> Result<(), CapacityExceeded> {
    if channels > CHANNEL_CAPACITY {
        return Err(CapacityExceeded::Channels);
    }
    if signed_atoms > SIGNED_ATOM_CAPACITY {
        return Err(CapacityExceeded::SignedAtoms);
    }
    if positive_atoms > POSITIVE_ATOM_CAPACITY {
        return Err(CapacityExceeded::PositiveAtoms);
    }
    if coefficients > COEFFICIENT_CAPACITY {
        return Err(CapacityExceeded::Coefficients);
    }
    Ok(())
}

pub(super) fn peak_regions(energy: &[f64]) -> Result<Vec<Region>, CapacityExceeded> {
    if energy.len() > REGION_CAPACITY {
        return Err(CapacityExceeded::Regions);
    }
    let mut peaks = Vec::with_capacity(REGION_CAPACITY);
    for band in 0..energy.len() {
        if is_peak(energy, band) {
            peaks.push(band);
        }
    }
    if peaks.is_empty() && !energy.is_empty() {
        peaks.push(
            energy
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(right.1))
                .map_or(0, |(band, _)| band),
        );
    }
    let mut boundaries = Vec::with_capacity(peaks.len().saturating_sub(1));
    for pair in peaks.windows(2) {
        boundaries.push(
            (pair[0] + 1..pair[1])
                .min_by(|left, right| energy[*left].total_cmp(&energy[*right]))
                .unwrap_or(pair[0]),
        );
    }
    Ok(peaks
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
            owner: 0,
        })
        .collect())
}

fn is_peak(energy: &[f64], band: usize) -> bool {
    let value = energy[band];
    if value <= ENERGY_FLOOR {
        return false;
    }
    let first = band.saturating_sub(2);
    let end = (band + 3).min(energy.len());
    !(first..end).any(|other| {
        other != band && (energy[other] > value || (other < band && energy[other] == value))
    })
}
