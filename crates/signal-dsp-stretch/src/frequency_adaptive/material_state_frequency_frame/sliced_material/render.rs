use std::sync::Arc;

use rustfft::Fft;

use super::{analysis::SourceCache, phase::PhaseState, *};

struct OutputSlice {
    start: isize,
    coefficients: [Vec<Vec<Complex64>>; 2],
}

impl OutputSlice {
    fn new(start: isize, bands: usize, coefficients: usize) -> Self {
        Self {
            start,
            coefficients: std::array::from_fn(|_| {
                vec![vec![Complex64::default(); coefficients]; bands]
            }),
        }
    }
}

struct Synthesis<'a> {
    representation: &'a Representation,
    positive: &'a [usize],
    window: Vec<f64>,
    forward_band: Arc<dyn Fft<f64>>,
    inverse_full: Arc<dyn Fft<f64>>,
}

impl<'a> Synthesis<'a> {
    fn new(representation: &'a Representation, positive: &'a [usize]) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        Self {
            representation,
            positive,
            window: outer_window(),
            forward_band: planner.plan_fft_forward(representation.common_coefficients),
            inverse_full: planner.plan_fft_inverse(FFT_FRAMES),
        }
    }

    fn add_slice(
        &self,
        mut slice: OutputSlice,
        output: &mut [Vec<f64>; 2],
        coverage: &mut [usize],
    ) -> usize {
        mirror_coefficients(self.representation, self.positive, &mut slice.coefficients);
        let mut non_finite = 0;
        for channel in 0..2 {
            let mut spectrum = vec![Complex64::default(); FFT_FRAMES];
            for (band, mut values) in self
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
                        self.representation.common_coefficients,
                        FFT_FRAMES,
                    );
                    spectrum[bin] +=
                        values[local] * weight / self.representation.frame_operator[bin];
                }
            }
            close_conjugate_spectrum(&mut spectrum);
            self.inverse_full.process(&mut spectrum);
            let scale = 1.0 / FFT_FRAMES as f64;
            for (local, sample) in spectrum.into_iter().enumerate() {
                let logical = slice.start + local as isize;
                if (0..output[channel].len() as isize).contains(&logical) {
                    let sample = sample * scale * self.window[local];
                    output[channel][logical as usize] += sample.re;
                    non_finite +=
                        usize::from(!sample.re.is_finite()) + usize::from(!sample.im.is_finite());
                    if channel == 0 {
                        coverage[logical as usize] += 1;
                    }
                }
            }
        }
        non_finite
    }
}

pub(super) fn render_detailed(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
) -> CandidateRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "non-empty linked input");
    assert!(ratio.is_finite() && ratio > 0.0, "positive finite ratio");
    if ratio == 1.0 {
        return CandidateRender {
            render: finish(
                [inputs[0].to_vec(), inputs[1].to_vec()],
                inputs[0].len(),
                0,
                StateCounts::default(),
            ),
            relations: RelationCounts::default(),
            maximum_relation_error: 0.0,
            maximum_live_source_slices: 0,
            maximum_live_output_slices: 0,
        };
    }

    let target_length = (inputs[0].len() as f64 * ratio).round() as usize;
    let representation = build_representation_for(FFT_FRAMES, sample_rate, COMMON_HOP);
    let positive = representation
        .bands
        .iter()
        .enumerate()
        .filter(|(_, band)| band.center <= FFT_FRAMES / 2)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mut source = SourceCache::new(inputs, &representation, &positive);
    let synthesis = Synthesis::new(&representation, &positive);
    let mut phase = PhaseState::new();
    let mut output = std::array::from_fn(|_| vec![0.0; target_length]);
    let mut coverage = vec![0_usize; target_length];
    let mut active = Vec::<OutputSlice>::new();
    let mut maximum_live_output_slices = 0;
    let mut non_finite = 0;
    let first_start = -(OUTER_ADVANCE as isize);
    let last_start = ((target_length - 1) / OUTER_ADVANCE * OUTER_ADVANCE) as isize;
    let first_time = first_start / COMMON_HOP as isize;
    let last_time = (last_start + FFT_FRAMES as isize - COMMON_HOP as isize) / COMMON_HOP as isize;

    for (output_time, time) in (first_time..=last_time).enumerate() {
        let output_frame = time * COMMON_HOP as isize;
        let reflected = reflect_index(output_frame, target_length);
        let source_position = reflected as f64 / ratio;
        let frame = source.frame(source_position);
        let transported = phase.advance(
            &frame,
            &representation,
            &positive,
            source_position,
            ratio,
            output_time,
        );
        let current = output_frame.div_euclid(OUTER_ADVANCE as isize) * OUTER_ADVANCE as isize;
        for (layer, start) in [current - OUTER_ADVANCE as isize, current]
            .into_iter()
            .enumerate()
        {
            if start < first_start || start > last_start {
                continue;
            }
            if active.iter().all(|slice| slice.start != start) {
                active.push(OutputSlice::new(
                    start,
                    representation.bands.len(),
                    representation.common_coefficients,
                ));
            }
            let slice = active
                .iter_mut()
                .find(|slice| slice.start == start)
                .expect("active output slice");
            let local_time = ((output_frame - start) / COMMON_HOP as isize) as usize;
            for (local_band, band) in positive.iter().copied().enumerate() {
                for channel in 0..2 {
                    slice.coefficients[channel][band][local_time] =
                        transported[layer][channel][local_band];
                }
            }
        }
        maximum_live_output_slices = maximum_live_output_slices.max(active.len());
        let mut index = 0;
        while index < active.len() {
            let complete_time = active[index].start / COMMON_HOP as isize
                + representation.common_coefficients as isize
                - 1;
            if time == complete_time {
                let slice = active.remove(index);
                non_finite += synthesis.add_slice(slice, &mut output, &mut coverage);
            } else {
                index += 1;
            }
        }
    }
    let uncovered = coverage.iter().filter(|count| **count != 2).count();
    let mut rendered = finish(output, target_length, uncovered, phase.states);
    rendered.non_finite += non_finite;
    CandidateRender {
        render: rendered,
        relations: phase.relations,
        maximum_relation_error: phase.maximum_relation_error,
        maximum_live_source_slices: source.maximum_live_slices(),
        maximum_live_output_slices,
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
    representation: &Representation,
    positive: &[usize],
    coefficients: &mut [Vec<Vec<Complex64>>; 2],
) {
    for &band in positive {
        let center = representation.bands[band].center;
        if center == 0 || center == FFT_FRAMES / 2 {
            coefficients
                .iter_mut()
                .for_each(|channel| channel[band].iter_mut().for_each(|value| value.im = 0.0));
        } else {
            let mirror = representation
                .bands
                .binary_search_by_key(&(FFT_FRAMES - center), |candidate| candidate.center)
                .expect("conjugate band");
            for channel in coefficients.iter_mut() {
                channel[mirror] = channel[band].iter().map(Complex64::conj).collect();
            }
        }
    }
}

fn close_conjugate_spectrum(spectrum: &mut [Complex64]) {
    spectrum[0].im = 0.0;
    spectrum[FFT_FRAMES / 2].im = 0.0;
    for bin in 1..FFT_FRAMES / 2 {
        let mirror = FFT_FRAMES - bin;
        let closed = (spectrum[bin] + spectrum[mirror].conj()) * 0.5;
        spectrum[bin] = closed;
        spectrum[mirror] = closed.conj();
    }
}
