use std::sync::Arc;

use rustfft::Fft;

use super::{relation::AtomEndpoints, *};
use crate::frequency_adaptive::material_state_frequency_frame::material_phase::Material;

mod material;
use material::interpolate_material;

pub(super) struct FrameAnalysis {
    pub atoms: Vec<AtomAnalysis>,
    pub centers: Vec<bool>,
    pub center_position: f64,
}

pub(super) struct AtomAnalysis {
    pub source: AtomEndpoints,
    pub material: Material,
}

struct SourceSlice {
    start: isize,
    coefficients: [Vec<Vec<Complex64>>; 2],
}

pub(super) struct SourceCache<'a> {
    inputs: [&'a [f64]; 2],
    representation: &'a Representation,
    positive: &'a [usize],
    same_scale_neighbors: Vec<Vec<usize>>,
    window: Vec<f64>,
    forward: Arc<dyn Fft<f64>>,
    inverse_band: Arc<dyn Fft<f64>>,
    slices: VecDeque<SourceSlice>,
    maximum_live_slices: usize,
}

impl<'a> SourceCache<'a> {
    pub fn new(
        inputs: [&'a [f64]; 2],
        representation: &'a Representation,
        positive: &'a [usize],
    ) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        let same_scale_neighbors = positive
            .iter()
            .enumerate()
            .map(|(local, band)| {
                let same_scale = positive
                    .iter()
                    .enumerate()
                    .filter(|(_, candidate)| {
                        representation.bands[**candidate].scale == representation.bands[*band].scale
                    })
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                let position = same_scale
                    .iter()
                    .position(|index| *index == local)
                    .expect("owned scale band");
                same_scale[position.saturating_sub(1)..=(position + 1).min(same_scale.len() - 1)]
                    .to_vec()
            })
            .collect();
        Self {
            inputs,
            representation,
            positive,
            same_scale_neighbors,
            window: outer_window(),
            forward: planner.plan_fft_forward(FFT_FRAMES),
            inverse_band: planner.plan_fft_inverse(representation.common_coefficients),
            slices: VecDeque::new(),
            maximum_live_slices: 0,
        }
    }

    pub fn frame(&mut self, source_position: f64) -> FrameAnalysis {
        let source_time = source_position / COMMON_HOP as f64;
        let first = source_time.floor() as isize;
        let fraction = (source_time - first as f64).clamp(0.0, 1.0);
        let rounded = source_time.round() as isize;
        let maximum_radius = SUPPORT_FRAMES[0] as isize / (2 * COMMON_HOP as isize);
        let center_first = rounded - maximum_radius;
        let center_last = rounded + maximum_radius;
        let material_first = center_first - 1;
        let material_last = center_last + 1;
        let magnitude_first = material_first - maximum_radius;
        let magnitude_last = material_last + maximum_radius;
        let times = (magnitude_first..=magnitude_last).collect::<Vec<_>>();
        let magnitudes = self.magnitude_map(&times);
        let materials = self.material_map(&times, &magnitudes, material_first, material_last);
        let transientness = (material_first..=material_last)
            .map(|time| {
                let material_time = (time - material_first) as usize;
                let magnitude_time = (time - magnitude_first) as usize;
                let mut weighted = 0.0;
                let mut weight = 0.0;
                for band in 0..self.positive.len() {
                    weighted += materials[band][material_time].transientness
                        * magnitudes[band][magnitude_time];
                    weight += magnitudes[band][magnitude_time];
                }
                if weight == 0.0 {
                    0.0
                } else {
                    weighted / weight
                }
            })
            .collect::<Vec<_>>();
        let centers = (center_first..=center_last)
            .map(|time| {
                let index = (time - material_first) as usize;
                transientness[index] > transientness[index - 1]
                    && transientness[index] >= transientness[index + 1]
            })
            .collect::<Vec<_>>();
        let atoms = (0..self.positive.len())
            .map(|local| {
                let first_material = materials[local][(first - material_first) as usize];
                let second_material = materials[local][(first + 1 - material_first) as usize];
                AtomAnalysis {
                    source: self.atom_endpoints(local, first, fraction),
                    material: interpolate_material(first_material, second_material, fraction),
                }
            })
            .collect();
        FrameAnalysis {
            atoms,
            centers,
            center_position: source_time - center_first as f64,
        }
    }

    pub fn maximum_live_slices(&self) -> usize {
        self.maximum_live_slices
    }

    fn magnitude_map(&mut self, times: &[isize]) -> Vec<Vec<f64>> {
        (0..self.positive.len())
            .map(|band| {
                times
                    .iter()
                    .map(|time| {
                        let layer = self.dominant_layer(*time);
                        let values = self.layer_coefficients(band, *time, layer);
                        values[0].norm().max(values[1].norm())
                    })
                    .collect()
            })
            .collect()
    }

    fn atom_endpoints(&mut self, band: usize, first: isize, fraction: f64) -> AtomEndpoints {
        let layers = std::array::from_fn(|layer| {
            std::array::from_fn(|channel| {
                [
                    self.layer_coefficients(band, first, layer)[channel],
                    self.layer_coefficients(band, first + 1, layer)[channel],
                ]
            })
        });
        AtomEndpoints::new(layers, fraction)
    }

    fn dominant_layer(&self, time: isize) -> usize {
        let frame = time * COMMON_HOP as isize;
        let current = frame.div_euclid(OUTER_ADVANCE as isize) * OUTER_ADVANCE as isize;
        let current_local = (frame - current) as usize;
        let previous_local = current_local + OUTER_ADVANCE;
        usize::from(self.window[current_local] > self.window[previous_local])
    }

    fn layer_coefficients(&mut self, band: usize, time: isize, layer: usize) -> [Complex64; 2] {
        let frame = time * COMMON_HOP as isize;
        let current = frame.div_euclid(OUTER_ADVANCE as isize) * OUTER_ADVANCE as isize;
        let start = if layer == 0 {
            current - OUTER_ADVANCE as isize
        } else {
            current
        };
        let local = ((frame - start) / COMMON_HOP as isize) as usize;
        let band = self.positive[band];
        std::array::from_fn(|channel| self.coefficient(start, channel, band, local))
    }

    fn coefficient(
        &mut self,
        start: isize,
        channel: usize,
        band: usize,
        local: usize,
    ) -> Complex64 {
        if let Some(index) = self.slices.iter().position(|slice| slice.start == start) {
            let slice = self.slices.remove(index).expect("cached slice");
            let value = slice.coefficients[channel][band][local];
            self.slices.push_back(slice);
            return value;
        }
        let slice = self.analyse_slice(start);
        let value = slice.coefficients[channel][band][local];
        self.slices.push_back(slice);
        while self.slices.len() > 6 {
            self.slices.pop_front();
        }
        self.maximum_live_slices = self.maximum_live_slices.max(self.slices.len());
        value
    }

    fn analyse_slice(&self, start: isize) -> SourceSlice {
        let coefficients = std::array::from_fn(|channel| {
            let mut spectrum = (0..FFT_FRAMES)
                .map(|local| {
                    Complex64::new(
                        reflected_sample(self.inputs[channel], start + local as isize)
                            * self.window[local],
                        0.0,
                    )
                })
                .collect::<Vec<_>>();
            self.forward.process(&mut spectrum);
            self.representation
                .bands
                .iter()
                .map(|band| {
                    let mut values =
                        vec![Complex64::default(); self.representation.common_coefficients];
                    for &(bin, weight) in &band.taps {
                        let local = local_coefficient(bin, band.center, values.len(), FFT_FRAMES);
                        values[local] = spectrum[bin] * weight;
                    }
                    self.inverse_band.process(&mut values);
                    let scale = 1.0 / values.len() as f64;
                    values.iter_mut().for_each(|value| *value *= scale);
                    values
                })
                .collect()
        });
        SourceSlice {
            start,
            coefficients,
        }
    }
}
