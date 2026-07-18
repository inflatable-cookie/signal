use super::{geometry::Prepared, *};

impl Prepared {
    pub(super) fn guidance_at(&self, tick: isize, output: &mut [MaterialGuidance]) -> bool {
        let atoms = self.owned_bins.iter().sum::<usize>();
        assert_eq!(output.len(), atoms);
        for (atom, output) in output.iter_mut().enumerate() {
            *output = self.material_at(tick, atom);
        }
        let trace = [-1, 0, 1].map(|offset| self.transient_trace(tick + offset));
        trace[1] > trace[0] && trace[1] >= trace[2]
    }

    fn material_at(&self, tick: isize, atom: usize) -> MaterialGuidance {
        let (scale, first, end) = self.atom_scale(atom);
        let radius = match scale {
            Scale::Long => 4,
            Scale::Middle => 2,
            Scale::Short => 1,
        };
        let mut temporal = [0.0; 9];
        let mut count = 0;
        for offset in -radius..=radius {
            temporal[count] = self.magnitude(tick + offset, atom);
            count += 1;
        }
        let tonal = median(&mut temporal[..count]);

        let neighbor_first = atom.saturating_sub(1).max(first);
        let neighbor_end = (atom + 2).min(end);
        let mut frequency = [0.0; 3];
        let count = neighbor_end - neighbor_first;
        for (slot, neighbor) in (neighbor_first..neighbor_end).enumerate() {
            frequency[slot] = self.magnitude(tick, neighbor);
        }
        let transient = median(&mut frequency[..count]);
        let sum = tonal + transient;
        if sum == 0.0 {
            MaterialGuidance::default()
        } else {
            let tonalness = tonal / sum;
            let transientness = transient / sum;
            MaterialGuidance {
                tonalness,
                noisiness: 1.0 - (tonalness - transientness).abs(),
                transientness,
            }
        }
    }

    fn transient_trace(&self, tick: isize) -> f64 {
        let atoms = self.owned_bins.iter().sum::<usize>();
        let mut weighted = 0.0;
        let mut weight = 0.0;
        for atom in 0..atoms {
            let magnitude = self.magnitude(tick, atom);
            weighted += self.material_at(tick, atom).transientness * magnitude;
            weight += magnitude;
        }
        if weight == 0.0 {
            0.0
        } else {
            weighted / weight
        }
    }

    fn magnitude(&self, tick: isize, atom: usize) -> f64 {
        let atoms = self.owned_bins.iter().sum::<usize>();
        let slot = tick.rem_euclid(GUIDANCE_TICKS as isize) as usize;
        self.guidance[slot * atoms + atom]
    }

    fn atom_scale(&self, atom: usize) -> (Scale, usize, usize) {
        let mut first = 0;
        for scale in Scale::ALL {
            let end = first + self.owned_bins[scale.index()];
            if atom < end {
                return (scale, first, end);
            }
            first = end;
        }
        panic!("direct timeline atom outside prepared geometry");
    }
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
