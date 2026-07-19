use super::*;

pub(super) fn build_regions(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    owned: [usize; 3],
    records: &mut [RegionRecord],
) {
    for channel in 0..channels {
        let mut first = 0;
        for count in owned {
            let end = first + count;
            if first != end {
                let mut prior_peak = None;
                let mut region_first = first;
                for atom in first..end {
                    if is_peak(current, atoms, channel, first, end, atom) {
                        if let Some(peak) = prior_peak {
                            let boundary = valley(current, atoms, channel, peak, atom);
                            assign_region(
                                current,
                                atoms,
                                channel,
                                records,
                                region_first,
                                boundary + 1,
                                peak,
                            );
                            region_first = boundary + 1;
                        }
                        prior_peak = Some(atom);
                    }
                }
                let peak = prior_peak.unwrap_or_else(|| {
                    (first..end).fold(first, |best, atom| {
                        if energy(current, atoms, channel, atom)
                            > energy(current, atoms, channel, best)
                        {
                            atom
                        } else {
                            best
                        }
                    })
                });
                assign_region(current, atoms, channel, records, region_first, end, peak);
            }
            first = end;
        }
    }
}

fn assign_region(
    current: &[Complex64],
    atoms: usize,
    channel: usize,
    records: &mut [RegionRecord],
    first: usize,
    end: usize,
    peak: usize,
) {
    for atom in first..end {
        records[channel * atoms + atom] = RegionRecord {
            peak,
            trajectory_channel: channel,
            supported: current[channel * atoms + atom].norm_sqr() > SUPPORT_FLOOR,
        };
    }
}

fn is_peak(
    current: &[Complex64],
    atoms: usize,
    channel: usize,
    first: usize,
    end: usize,
    atom: usize,
) -> bool {
    let candidate = energy(current, atoms, channel, atom);
    candidate > SUPPORT_FLOOR
        && !(atom.saturating_sub(2).max(first)..(atom + 3).min(end)).any(|other| {
            other != atom
                && (energy(current, atoms, channel, other) > candidate
                    || (other < atom && energy(current, atoms, channel, other) == candidate))
        })
}

fn valley(current: &[Complex64], atoms: usize, channel: usize, left: usize, right: usize) -> usize {
    (left + 1..right).fold(left, |best, atom| {
        if best == left
            || energy(current, atoms, channel, atom) < energy(current, atoms, channel, best)
        {
            atom
        } else {
            best
        }
    })
}

fn energy(current: &[Complex64], atoms: usize, channel: usize, atom: usize) -> f64 {
    current[channel * atoms + atom].norm_sqr()
}

pub(super) fn dominant_channel(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    atom: usize,
) -> usize {
    (1..channels).fold(0, |best, channel| {
        if energy(current, atoms, channel, atom) > energy(current, atoms, best, atom) {
            channel
        } else {
            best
        }
    })
}
