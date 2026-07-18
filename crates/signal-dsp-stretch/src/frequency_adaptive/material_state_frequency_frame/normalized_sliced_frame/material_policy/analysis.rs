use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{
    local_coefficient, reflected_sample,
};

struct SourceSlice {
    start: isize,
    coefficients: [Vec<Vec<Complex64>>; CHANNEL_CAPACITY],
}

pub(super) struct FrameAnalysis {
    pub current: Frame,
    pub layers: [Frame; OUTPUT_SLICE_CAPACITY],
    pub material: Vec<Material>,
    pub transient_center: bool,
}

pub(super) struct SourceCache<'a> {
    inputs: [&'a [f64]; CHANNEL_CAPACITY],
    geometry: &'a Geometry,
    positive: Vec<usize>,
    same_scale_neighbors: Vec<Vec<usize>>,
    window: Vec<f64>,
    forward: Arc<dyn Fft<f64>>,
    inverse_band: Arc<dyn Fft<f64>>,
    slices: VecDeque<SourceSlice>,
    maximum_live_slices: usize,
}

impl<'a> SourceCache<'a> {
    pub fn new(inputs: [&'a [f64]; CHANNEL_CAPACITY], geometry: &'a Geometry) -> Self {
        let positive = geometry
            .representation
            .bands
            .iter()
            .enumerate()
            .filter(|(_, band)| band.center <= geometry.fft_frames / 2)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let same_scale_neighbors = positive
            .iter()
            .enumerate()
            .map(|(local, band)| {
                let same_scale = positive
                    .iter()
                    .enumerate()
                    .filter(|(_, candidate)| {
                        geometry.representation.bands[**candidate].scale
                            == geometry.representation.bands[*band].scale
                    })
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                let position = same_scale
                    .iter()
                    .position(|index| *index == local)
                    .expect("owned normalized scale atom");
                same_scale[position.saturating_sub(1)..=(position + 1).min(same_scale.len() - 1)]
                    .to_vec()
            })
            .collect();
        let mut planner = FftPlanner::<f64>::new();
        Self {
            inputs,
            geometry,
            positive,
            same_scale_neighbors,
            window: (0..geometry.fft_frames)
                .map(|index| {
                    (std::f64::consts::PI * (index as f64 + 0.5) / geometry.fft_frames as f64).sin()
                })
                .collect(),
            forward: planner.plan_fft_forward(geometry.fft_frames),
            inverse_band: planner.plan_fft_inverse(COEFFICIENT_CAPACITY),
            slices: VecDeque::with_capacity(SOURCE_SLICE_CAPACITY),
            maximum_live_slices: 0,
        }
    }

    pub fn frame(&mut self, source_position: f64) -> FrameAnalysis {
        let source_time = source_position / self.geometry.hop as f64;
        let first = source_time.floor() as isize;
        let fraction = (source_time - first as f64).clamp(0.0, 1.0);
        let rounded = source_time.round() as isize;
        let maximum_radius = 4_isize;
        let center_first = rounded - maximum_radius;
        let center_last = rounded + maximum_radius;
        let material_first = center_first - 1;
        let material_last = center_last + 1;
        let magnitude_first = material_first - maximum_radius;
        let magnitude_last = material_last + maximum_radius;
        let times = (magnitude_first..=magnitude_last).collect::<Vec<_>>();
        let magnitude_map = self.magnitude_map(&times);
        let materials = guidance::material_map(
            &self.geometry.representation,
            &self.positive,
            &self.same_scale_neighbors,
            &times,
            &magnitude_map,
            material_first,
            material_last,
        );
        let transient_trace = (material_first..=material_last)
            .map(|time| {
                let material_time = (time - material_first) as usize;
                let magnitude_time = (time - magnitude_first) as usize;
                let mut weighted = 0.0;
                let mut weight = 0.0;
                for band in 0..self.positive.len() {
                    weighted += materials[band][material_time].transientness
                        * magnitude_map[band][magnitude_time];
                    weight += magnitude_map[band][magnitude_time];
                }
                if weight == 0.0 {
                    0.0
                } else {
                    weighted / weight
                }
            })
            .collect::<Vec<_>>();
        let center_index = (rounded - material_first) as usize;
        let transient_center = transient_trace[center_index] > transient_trace[center_index - 1]
            && transient_trace[center_index] >= transient_trace[center_index + 1];
        let material = (0..self.positive.len())
            .map(|band| {
                guidance::interpolate_material(
                    materials[band][(first - material_first) as usize],
                    materials[band][(first + 1 - material_first) as usize],
                    fraction,
                )
            })
            .collect();
        let layers = std::array::from_fn(|layer| self.sample_layer(first, fraction, layer));
        let current: Frame = std::array::from_fn(|channel| {
            (0..self.positive.len())
                .map(|band| {
                    let first_layer = self.dominant_layer(first);
                    let second_layer = self.dominant_layer(first + 1);
                    polar_interpolate(
                        self.layer_coefficients(band, first, first_layer)[channel],
                        self.layer_coefficients(band, first + 1, second_layer)[channel],
                        fraction,
                    )
                })
                .collect()
        });
        FrameAnalysis {
            current,
            layers,
            material,
            transient_center,
        }
    }

    pub fn positive(&self) -> &[usize] {
        &self.positive
    }

    pub fn maximum_live_slices(&self) -> usize {
        self.maximum_live_slices
    }

    fn sample_layer(&mut self, first: isize, fraction: f64, layer: usize) -> Frame {
        std::array::from_fn(|channel| {
            (0..self.positive.len())
                .map(|band| {
                    polar_interpolate(
                        self.layer_coefficients(band, first, layer)[channel],
                        self.layer_coefficients(band, first + 1, layer)[channel],
                        fraction,
                    )
                })
                .collect()
        })
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

    fn dominant_layer(&self, time: isize) -> usize {
        let frame = time * self.geometry.hop as isize;
        let current = frame.div_euclid(self.geometry.outer_advance as isize)
            * self.geometry.outer_advance as isize;
        let current_local = (frame - current) as usize;
        usize::from(
            self.window[current_local] > self.window[current_local + self.geometry.outer_advance],
        )
    }

    fn layer_coefficients(&mut self, band: usize, time: isize, layer: usize) -> [Complex64; 2] {
        let frame = time * self.geometry.hop as isize;
        let current = frame.div_euclid(self.geometry.outer_advance as isize)
            * self.geometry.outer_advance as isize;
        let start = if layer == 0 {
            current - self.geometry.outer_advance as isize
        } else {
            current
        };
        let local = ((frame - start) / self.geometry.hop as isize) as usize;
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
            let slice = self
                .slices
                .remove(index)
                .expect("cached normalized source slice");
            let value = slice.coefficients[channel][band][local];
            self.slices.push_back(slice);
            return value;
        }
        let slice = self.analyse_slice(start);
        let value = slice.coefficients[channel][band][local];
        self.slices.push_back(slice);
        while self.slices.len() > SOURCE_SLICE_CAPACITY {
            self.slices.pop_front();
        }
        self.maximum_live_slices = self.maximum_live_slices.max(self.slices.len());
        value
    }

    fn analyse_slice(&self, start: isize) -> SourceSlice {
        let coefficients = std::array::from_fn(|channel| {
            let mut spectrum = (0..self.geometry.fft_frames)
                .map(|local| {
                    Complex64::new(
                        reflected_sample(self.inputs[channel], start + local as isize)
                            * self.window[local],
                        0.0,
                    )
                })
                .collect::<Vec<_>>();
            self.forward.process(&mut spectrum);
            self.geometry
                .representation
                .bands
                .iter()
                .map(|band| {
                    let mut values = vec![Complex64::default(); COEFFICIENT_CAPACITY];
                    for &(bin, weight) in &band.taps {
                        let local = local_coefficient(
                            bin,
                            band.center,
                            COEFFICIENT_CAPACITY,
                            self.geometry.fft_frames,
                        );
                        values[local] = spectrum[bin] * weight;
                    }
                    self.inverse_band.process(&mut values);
                    values
                        .iter_mut()
                        .for_each(|value| *value /= COEFFICIENT_CAPACITY as f64);
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

fn polar_interpolate(first: Complex64, second: Complex64, fraction: f64) -> Complex64 {
    let magnitude = first.norm() + (second.norm() - first.norm()) * fraction;
    let phase = first.arg()
        + crate::frequency_adaptive::material_state_frequency_frame::guided_frequency_partitioned_linked_phase::wrap(
            second.arg() - first.arg(),
        ) * fraction;
    Complex64::from_polar(magnitude, phase)
}
