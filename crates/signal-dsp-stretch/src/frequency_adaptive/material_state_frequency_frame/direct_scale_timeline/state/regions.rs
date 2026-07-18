use super::*;

pub(super) fn build_regions(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    owned: [usize; 3],
    records: &mut [RegionRecord],
) {
    let mut first = 0;
    for count in owned {
        let end = first + count;
        if first == end {
            continue;
        }
        let mut prior_peak = None;
        let mut region_first = first;
        for atom in first..end {
            if is_peak(current, channels, atoms, first, end, atom) {
                if let Some(peak) = prior_peak {
                    let boundary = valley(current, channels, atoms, peak, atom);
                    assign_region(
                        current,
                        channels,
                        atoms,
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
                if joint_energy(current, channels, atoms, atom)
                    > joint_energy(current, channels, atoms, best)
                {
                    atom
                } else {
                    best
                }
            })
        });
        assign_region(current, channels, atoms, records, region_first, end, peak);
        first = end;
    }
}

fn assign_region(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    records: &mut [RegionRecord],
    first: usize,
    end: usize,
    peak: usize,
) {
    let owner =
        usize::from(channels == 2 && current[atoms + peak].norm_sqr() > current[peak].norm_sqr());
    for channel in 0..channels {
        for atom in first..end {
            records[channel * atoms + atom] = RegionRecord {
                peak,
                owner,
                supported: current[channel * atoms + atom].norm_sqr() > SUPPORT_FLOOR,
            };
        }
    }
}

fn is_peak(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    first: usize,
    end: usize,
    atom: usize,
) -> bool {
    let energy = joint_energy(current, channels, atoms, atom);
    energy > SUPPORT_FLOOR
        && !(atom.saturating_sub(2).max(first)..(atom + 3).min(end)).any(|other| {
            other != atom
                && (joint_energy(current, channels, atoms, other) > energy
                    || (other < atom && joint_energy(current, channels, atoms, other) == energy))
        })
}

fn valley(
    current: &[Complex64],
    channels: usize,
    atoms: usize,
    left: usize,
    right: usize,
) -> usize {
    (left + 1..right).fold(left, |best, atom| {
        if best == left
            || joint_energy(current, channels, atoms, atom)
                < joint_energy(current, channels, atoms, best)
        {
            atom
        } else {
            best
        }
    })
}

fn joint_energy(current: &[Complex64], channels: usize, atoms: usize, atom: usize) -> f64 {
    (0..channels)
        .map(|channel| current[channel * atoms + atom].norm_sqr())
        .fold(0.0_f64, f64::max)
}
