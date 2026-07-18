use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{Representation, Scale};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Material {
    pub tonalness: f64,
    pub noisiness: f64,
    pub transientness: f64,
}

pub(super) struct GuidanceState {
    previous_silent: bool,
    first: bool,
}

impl GuidanceState {
    pub fn new() -> Self {
        Self {
            previous_silent: true,
            first: true,
        }
    }

    pub fn decisions(
        &mut self,
        analysis: &analysis::FrameAnalysis,
        frequencies_hz: &[f64],
        discontinuity: bool,
    ) -> Vec<Decision> {
        let silent = analysis
            .current
            .iter()
            .flatten()
            .all(|value| value.norm_sqr() <= crate::frequency_adaptive::material_state_frequency_frame::guided_frequency_partitioned_linked_phase::ENERGY_FLOOR);
        let reset = self.first || discontinuity || (!silent && self.previous_silent);
        let decisions = analysis
            .material
            .iter()
            .zip(frequencies_hz)
            .map(|(material, frequency)| {
                if reset || silent {
                    Decision::Reset
                } else if material.transientness > material.tonalness
                    && analysis.transient_center
                    && *frequency < 6_000.0
                {
                    Decision::Attack
                } else if material.noisiness > material.tonalness {
                    Decision::Unlocked
                } else {
                    Decision::Locked
                }
            })
            .collect();
        self.first = false;
        self.previous_silent = silent;
        decisions
    }
}

pub(super) fn material_map(
    representation: &Representation,
    positive: &[usize],
    same_scale_neighbors: &[Vec<usize>],
    times: &[isize],
    magnitudes: &[Vec<f64>],
    first: isize,
    last: isize,
) -> Vec<Vec<Material>> {
    (0..positive.len())
        .map(|band| {
            (first..=last)
                .map(|time| {
                    let radius = match representation.bands[positive[band]].scale {
                        Scale::Long => 4,
                        Scale::Middle => 2,
                        Scale::Short => 1,
                    };
                    let tonal =
                        median((time - radius..=time + radius).map(|sample_time| {
                            magnitudes[band][(sample_time - times[0]) as usize]
                        }));
                    let transient = median(
                        same_scale_neighbors[band]
                            .iter()
                            .map(|neighbor| magnitudes[*neighbor][(time - times[0]) as usize]),
                    );
                    normalize(tonal, transient)
                })
                .collect()
        })
        .collect()
}

pub(super) fn interpolate_material(first: Material, second: Material, fraction: f64) -> Material {
    let lerp = |left: f64, right: f64| left + (right - left) * fraction;
    Material {
        tonalness: lerp(first.tonalness, second.tonalness),
        noisiness: lerp(first.noisiness, second.noisiness),
        transientness: lerp(first.transientness, second.transientness),
    }
}

fn normalize(tonal: f64, transient: f64) -> Material {
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

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
