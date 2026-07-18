use super::*;

impl SourceCache<'_> {
    pub(super) fn material_map(
        &self,
        times: &[isize],
        magnitudes: &[Vec<f64>],
        first: isize,
        last: isize,
    ) -> Vec<Vec<Material>> {
        (0..self.positive.len())
            .map(|band| {
                (first..=last)
                    .map(|time| {
                        let radius = SUPPORT_FRAMES
                            [self.representation.bands[self.positive[band]].scale.index()]
                            as isize
                            / (2 * COMMON_HOP as isize);
                        let tonal =
                            median((time - radius..=time + radius).map(|sample_time| {
                                magnitudes[band][time_offset(times, sample_time)]
                            }));
                        let transient = median(
                            self.same_scale_neighbors[band]
                                .iter()
                                .map(|neighbor| magnitudes[*neighbor][time_offset(times, time)]),
                        );
                        material(tonal, transient)
                    })
                    .collect()
            })
            .collect()
    }
}

fn material(tonal: f64, transient: f64) -> Material {
    let sum = tonal + transient;
    if sum == 0.0 {
        Material::default()
    } else {
        let tonalness = tonal / sum;
        let transientness = transient / sum;
        Material {
            tonalness,
            noisiness: 1.0 - (tonalness - transientness).abs(),
            transientness,
        }
    }
}

pub(super) fn interpolate_material(first: Material, second: Material, fraction: f64) -> Material {
    let lerp = |left: f64, right: f64| left + (right - left) * fraction;
    Material {
        tonalness: lerp(first.tonalness, second.tonalness),
        noisiness: lerp(first.noisiness, second.noisiness),
        transientness: lerp(first.transientness, second.transientness),
    }
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn time_offset(times: &[isize], time: isize) -> usize {
    (time - times[0]) as usize
}
