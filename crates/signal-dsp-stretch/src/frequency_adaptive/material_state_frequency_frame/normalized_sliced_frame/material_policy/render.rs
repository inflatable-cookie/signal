use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{
    guided_frequency_partitioned_linked_phase::{wrap, Workspace, ENERGY_FLOOR},
    local_coefficient,
};

struct OutputSlice {
    start: isize,
    coefficients: [Vec<Vec<Complex64>>; CHANNEL_CAPACITY],
    reference: Option<[Vec<Vec<Complex64>>; CHANNEL_CAPACITY]>,
}

impl OutputSlice {
    fn new(start: isize, bands: usize, attributed: bool) -> Self {
        let coefficients = || {
            std::array::from_fn(|_| vec![vec![Complex64::default(); COEFFICIENT_CAPACITY]; bands])
        };
        Self {
            start,
            coefficients: coefficients(),
            reference: attributed.then(coefficients),
        }
    }
}

struct Synthesis<'a> {
    geometry: &'a Geometry,
    positive: &'a [usize],
    window: Vec<f64>,
    forward_band: Arc<dyn Fft<f64>>,
    inverse_full: Arc<dyn Fft<f64>>,
}

impl<'a> Synthesis<'a> {
    fn new(geometry: &'a Geometry, positive: &'a [usize]) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        Self {
            geometry,
            positive,
            window: (0..geometry.fft_frames)
                .map(|index| {
                    (std::f64::consts::PI * (index as f64 + 0.5) / geometry.fft_frames as f64).sin()
                })
                .collect(),
            forward_band: planner.plan_fft_forward(COEFFICIENT_CAPACITY),
            inverse_full: planner.plan_fft_inverse(geometry.fft_frames),
        }
    }

    fn add_slice(
        &self,
        mut slice: OutputSlice,
        output: &mut [Vec<f64>; CHANNEL_CAPACITY],
        coverage: &mut [usize],
        trace: Option<&mut attribution::TraceCollector>,
    ) -> usize {
        mirror_coefficients(self.geometry, self.positive, &mut slice.coefficients);
        if let Some(reference) = &mut slice.reference {
            mirror_coefficients(self.geometry, self.positive, reference);
        }
        let mut actual_contribution = trace
            .as_ref()
            .map(|_| std::array::from_fn(|_| vec![0.0; self.geometry.fft_frames]));
        let mut non_finite = 0;
        for channel in 0..CHANNEL_CAPACITY {
            let mut spectrum = vec![Complex64::default(); self.geometry.fft_frames];
            for (band, mut values) in self
                .geometry
                .representation
                .bands
                .iter()
                .zip(slice.coefficients[channel].clone())
            {
                self.forward_band.process(&mut values);
                for &(bin, weight) in &band.taps {
                    let local = local_coefficient(
                        bin,
                        band.center,
                        COEFFICIENT_CAPACITY,
                        self.geometry.fft_frames,
                    );
                    spectrum[bin] +=
                        values[local] * weight / self.geometry.representation.frame_operator[bin];
                }
            }
            close_conjugate_spectrum(&mut spectrum);
            self.inverse_full.process(&mut spectrum);
            for (local, sample) in spectrum.into_iter().enumerate() {
                let logical = slice.start + local as isize;
                if (0..output[channel].len() as isize).contains(&logical) {
                    let sample = sample / self.geometry.fft_frames as f64 * self.window[local];
                    if let Some(contribution) = &mut actual_contribution {
                        contribution[channel][local] = sample.re;
                    }
                    output[channel][logical as usize] += sample.re;
                    non_finite +=
                        usize::from(!sample.re.is_finite()) + usize::from(!sample.im.is_finite());
                    if channel == 0 {
                        coverage[logical as usize] += 1;
                    }
                }
            }
        }
        if let (Some(trace), Some(reference), Some(actual)) =
            (trace, slice.reference, actual_contribution)
        {
            let reference = self.contribution(reference);
            trace.observe_slice(slice.start, &reference, &actual, output);
        }
        non_finite
    }

    fn contribution(
        &self,
        coefficients: [Vec<Vec<Complex64>>; CHANNEL_CAPACITY],
    ) -> [Vec<f64>; CHANNEL_CAPACITY] {
        std::array::from_fn(|channel| {
            let mut spectrum = vec![Complex64::default(); self.geometry.fft_frames];
            for (band, mut values) in self
                .geometry
                .representation
                .bands
                .iter()
                .zip(coefficients[channel].clone())
            {
                self.forward_band.process(&mut values);
                for &(bin, weight) in &band.taps {
                    let local = local_coefficient(
                        bin,
                        band.center,
                        COEFFICIENT_CAPACITY,
                        self.geometry.fft_frames,
                    );
                    spectrum[bin] +=
                        values[local] * weight / self.geometry.representation.frame_operator[bin];
                }
            }
            close_conjugate_spectrum(&mut spectrum);
            self.inverse_full.process(&mut spectrum);
            spectrum
                .into_iter()
                .enumerate()
                .map(|(local, sample)| {
                    sample.re / self.geometry.fft_frames as f64 * self.window[local]
                })
                .collect()
        })
    }
}

pub(super) fn render(
    inputs: [&[f64]; CHANNEL_CAPACITY],
    ratio: f64,
    sample_rate: usize,
) -> CandidateRender {
    render_inner(inputs, ratio, sample_rate, None)
}

pub(super) fn render_attributed(
    inputs: [&[f64]; CHANNEL_CAPACITY],
    ratio: f64,
    sample_rate: usize,
) -> (CandidateRender, attribution::RenderAttribution) {
    let target_length = (inputs[0].len() as f64 * ratio).round() as usize;
    let mut trace = attribution::TraceCollector::new(target_length);
    let rendered = render_inner(inputs, ratio, sample_rate, Some(&mut trace));
    (rendered, trace.finish())
}

fn render_inner(
    inputs: [&[f64]; CHANNEL_CAPACITY],
    ratio: f64,
    sample_rate: usize,
    mut trace: Option<&mut attribution::TraceCollector>,
) -> CandidateRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "non-empty linked input");
    assert!(ratio.is_finite() && ratio > 0.0, "positive finite ratio");
    if ratio == 1.0 {
        return finish(
            [inputs[0].to_vec(), inputs[1].to_vec()],
            inputs[0].len(),
            0,
            0,
            StateCounts::default(),
            0,
            0,
            0,
        );
    }

    let geometry = prepare(sample_rate).expect("Rule 31V supported geometry");
    let target_length = (inputs[0].len() as f64 * ratio).round() as usize;
    let mut source = SourceCache::new(inputs, &geometry);
    let positive = source.positive().to_vec();
    let frequencies_hz = positive
        .iter()
        .map(|band| {
            let center = geometry.representation.bands[*band].center;
            let absolute = if center <= geometry.fft_frames / 2 {
                center
            } else {
                geometry.fft_frames - center
            };
            absolute as f64 * sample_rate as f64 / geometry.fft_frames as f64
        })
        .collect::<Vec<_>>();
    let synthesis = Synthesis::new(&geometry, &positive);
    let mut workspace = Workspace::new(sample_rate, geometry.hop);
    let mut guidance = GuidanceState::new();
    let mut output = std::array::from_fn(|_| vec![0.0; target_length]);
    let mut coverage = vec![0_usize; target_length];
    let mut active = Vec::<OutputSlice>::with_capacity(OUTPUT_SLICE_CAPACITY);
    let mut maximum_live_output_slices = 0;
    let mut non_finite = 0;
    let mut previous_source = None;
    let first_start = -(geometry.outer_advance as isize);
    let last_start =
        ((target_length - 1) / geometry.outer_advance * geometry.outer_advance) as isize;
    let first_time = first_start / geometry.hop as isize;
    let last_time =
        (last_start + geometry.fft_frames as isize - geometry.hop as isize) / geometry.hop as isize;

    for time in first_time..=last_time {
        let output_frame = time * geometry.hop as isize;
        let reflected = reflect_index(output_frame, target_length);
        let source_position = reflected as f64 / ratio;
        let analysis = source.frame(source_position);
        let analysis_advance =
            previous_source.map_or(geometry.hop as f64, |previous| source_position - previous);
        let discontinuity = analysis_advance <= 0.0;
        let decisions = guidance.decisions(&analysis, &frequencies_hz, discontinuity);
        let decided = workspace
            .process_decisions_reference_unlocked(
                &analysis.current,
                &frequencies_hz,
                &decisions,
                analysis_advance,
            )
            .expect("Rule 31V state capacity");
        let projected = std::array::from_fn::<_, OUTPUT_SLICE_CAPACITY, _>(|layer| {
            project_layer(&analysis.layers[layer], &analysis.current, &decided)
        });
        if let Some(observer) = trace.as_deref_mut() {
            observer.observe_frame(
                &analysis,
                &decisions,
                &decided,
                &projected,
                &geometry,
                &positive,
                source_position,
                output_frame,
                output_frame < 0 || output_frame >= target_length as isize,
            );
        }
        previous_source = Some(source_position);

        let current = output_frame.div_euclid(geometry.outer_advance as isize)
            * geometry.outer_advance as isize;
        for (layer, start) in [current - geometry.outer_advance as isize, current]
            .into_iter()
            .enumerate()
        {
            if start < first_start || start > last_start {
                continue;
            }
            if active.iter().all(|slice| slice.start != start) {
                active.push(OutputSlice::new(
                    start,
                    geometry.representation.bands.len(),
                    trace.is_some(),
                ));
            }
            let slice = active
                .iter_mut()
                .find(|slice| slice.start == start)
                .expect("active normalized output slice");
            let local_time = ((output_frame - start) / geometry.hop as isize) as usize;
            for (local_band, band) in positive.iter().copied().enumerate() {
                for channel in 0..CHANNEL_CAPACITY {
                    slice.coefficients[channel][band][local_time] =
                        projected[layer][channel][local_band];
                    if let Some(reference) = &mut slice.reference {
                        reference[channel][band][local_time] =
                            analysis.layers[layer][channel][local_band];
                    }
                }
            }
        }
        maximum_live_output_slices = maximum_live_output_slices.max(active.len());
        let mut index = 0;
        while index < active.len() {
            let complete_time =
                active[index].start / geometry.hop as isize + COEFFICIENT_CAPACITY as isize - 1;
            if time == complete_time {
                let slice = active.remove(index);
                non_finite +=
                    synthesis.add_slice(slice, &mut output, &mut coverage, trace.as_deref_mut());
            } else {
                index += 1;
            }
        }
    }
    let uncovered = coverage.iter().filter(|count| **count != 2).count();
    finish(
        output,
        target_length,
        uncovered,
        non_finite,
        workspace.counts,
        source.maximum_live_slices(),
        maximum_live_output_slices,
        MATERIAL_HALO_FRAMES,
    )
}

fn project_layer(layer: &Frame, current: &Frame, decided: &Frame) -> Frame {
    std::array::from_fn(|channel| {
        layer[channel]
            .iter()
            .zip(&current[channel])
            .zip(&decided[channel])
            .map(|((local, analysis), shared)| {
                if local.norm_sqr() <= ENERGY_FLOOR {
                    Complex64::default()
                } else {
                    Complex64::from_polar(
                        local.norm(),
                        shared.arg() + wrap(local.arg() - analysis.arg()),
                    )
                }
            })
            .collect()
    })
}

fn finish(
    channels: [Vec<f64>; CHANNEL_CAPACITY],
    target_length: usize,
    uncovered: usize,
    extra_non_finite: usize,
    states: StateCounts,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
    maximum_guidance_frames: usize,
) -> CandidateRender {
    let non_finite = extra_non_finite
        + channels
            .iter()
            .flatten()
            .filter(|sample| !sample.is_finite())
            .count();
    let boundary_failures = channels
        .iter()
        .map(|channel| {
            usize::from(channel.first().is_none_or(|sample| !sample.is_finite()))
                + usize::from(channel.last().is_none_or(|sample| !sample.is_finite()))
        })
        .sum();
    let mut hash = HASH_OFFSET;
    for sample in channels.iter().flatten() {
        hash_u64(&mut hash, sample.to_bits());
    }
    CandidateRender {
        channels,
        target_length,
        uncovered,
        non_finite,
        boundary_failures,
        states,
        maximum_live_source_slices,
        maximum_live_output_slices,
        maximum_guidance_frames,
        hash,
    }
}

fn reflect_index(index: isize, length: usize) -> usize {
    let period = (length * 2) as isize;
    let wrapped = index.rem_euclid(period) as usize;
    if wrapped < length {
        wrapped
    } else {
        length * 2 - 1 - wrapped
    }
}

fn mirror_coefficients(
    geometry: &Geometry,
    positive: &[usize],
    coefficients: &mut [Vec<Vec<Complex64>>; CHANNEL_CAPACITY],
) {
    for &band in positive {
        let center = geometry.representation.bands[band].center;
        if center == 0 || center == geometry.fft_frames / 2 {
            for channel in coefficients.iter_mut() {
                for value in &mut channel[band] {
                    value.im = 0.0;
                }
            }
        } else {
            let mirror = geometry
                .representation
                .bands
                .binary_search_by_key(&(geometry.fft_frames - center), |candidate| {
                    candidate.center
                })
                .expect("normalized conjugate atom");
            for channel in coefficients.iter_mut() {
                channel[mirror] = channel[band].iter().map(Complex64::conj).collect();
            }
        }
    }
}

fn close_conjugate_spectrum(spectrum: &mut [Complex64]) {
    spectrum[0].im = 0.0;
    let nyquist = spectrum.len() / 2;
    spectrum[nyquist].im = 0.0;
    for bin in 1..nyquist {
        let mirror = spectrum.len() - bin;
        let closed = (spectrum[bin] + spectrum[mirror].conj()) * 0.5;
        spectrum[bin] = closed;
        spectrum[mirror] = closed.conj();
    }
}
