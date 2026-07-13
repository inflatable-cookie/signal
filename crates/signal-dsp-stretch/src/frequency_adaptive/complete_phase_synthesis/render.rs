use rustfft::{num_complex::Complex64, FftPlanner};

use super::super::complete_system_tuning::{Configuration, ResetScope, Sensitivity};
use super::super::study_local_schedule::schedule::Schedule;
use super::super::HASH_OFFSET;
use output::{allocate_layer_outputs, crop_outputs};
use phase::{transport, PhaseState};
use support::{frames, hash, mirror, reflected, window};

mod output;
mod phase;
mod support;

const BASELINE: Configuration = Configuration {
    geometry: [512, 2_048, 8_192],
    sensitivity: Sensitivity::Responsive,
    unity_strength_index: 2,
    reset_scope: ResetScope::ShortOnly,
    vertical_alignment: true,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Mode {
    Ordinary,
    Event,
    Vertical,
    Both,
    Shared,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Render {
    pub samples: Vec<Vec<f64>>,
    pub layer_samples: Option<[Vec<Vec<f64>>; 3]>,
    pub target_len: usize,
    pub uncovered: usize,
    pub boundary_failures: usize,
    pub event_order_failures: usize,
    pub symmetry_error: f64,
    pub imaginary_residue: f64,
    pub non_finite: usize,
    pub event_resets: usize,
    pub vertical_alignments: usize,
    pub schedule_hash: u64,
    pub magnitude_hash: u64,
    pub phase_hash: u64,
    pub output_hash: u64,
    pub channel_decision_hash: u64,
}

#[derive(Clone, Copy)]
struct Frame {
    layer: usize,
    source: isize,
    output: isize,
}

pub(super) fn render(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    schedule: &Schedule,
    mode: Mode,
) -> Render {
    render_configured(channels, ratio, events, schedule, mode, BASELINE)
}

pub(crate) fn render_configured(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    schedule: &Schedule,
    mode: Mode,
    configuration: Configuration,
) -> Render {
    render_configured_internal(
        channels,
        ratio,
        events,
        schedule,
        mode,
        configuration,
        false,
    )
}

pub(crate) fn render_configured_with_layers(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    schedule: &Schedule,
    mode: Mode,
    configuration: Configuration,
) -> Render {
    render_configured_internal(channels, ratio, events, schedule, mode, configuration, true)
}

fn render_configured_internal(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    schedule: &Schedule,
    mode: Mode,
    configuration: Configuration,
    capture_layers: bool,
) -> Render {
    let layers = configuration.geometry;
    let target_len = (ratio * channels[0].len() as f64).round() as usize;
    let frames = frames(channels[0].len(), ratio, schedule, layers);
    let output_start = frames
        .iter()
        .map(|frame| frame.output - layers[frame.layer] as isize / 2)
        .min()
        .unwrap();
    let output_end = frames
        .iter()
        .map(|frame| frame.output + layers[frame.layer] as isize / 2)
        .max()
        .unwrap();
    let domain_len = (output_end - output_start) as usize;
    let mut operator = vec![0.0; domain_len];
    for frame in &frames {
        for (offset, weight) in window(layers[frame.layer]).into_iter().enumerate() {
            let output = frame.output - layers[frame.layer] as isize / 2 + offset as isize;
            operator[(output - output_start) as usize] += weight * weight;
        }
    }
    let mut outputs = vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()];
    let mut layer_outputs = allocate_layer_outputs(capture_layers, channels.len(), domain_len);
    let mut states = channels
        .iter()
        .map(|_| PhaseState::new(layers))
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::<f64>::new();
    let mut magnitude_hash = HASH_OFFSET;
    let mut phase_hash = HASH_OFFSET;
    let mut decision_hash = HASH_OFFSET;
    let mut symmetry_error = 0.0_f64;
    let mut imaginary_residue = 0.0_f64;
    let mut non_finite = 0;
    let mut event_resets = 0;
    let mut vertical_alignments = 0;
    for frame in &frames {
        let length = layers[frame.layer];
        let window = window(length);
        let forward = planner.plan_fft_forward(length);
        let inverse = planner.plan_fft_inverse(length);
        let mut spectra = Vec::with_capacity(channels.len());
        let mut linked = vec![0.0; length / 2 + 1];
        for channel in channels {
            let mut spectrum = (0..length)
                .map(|offset| {
                    let source = frame.source - length as isize / 2 + offset as isize;
                    Complex64::new(reflected(channel, source) * window[offset], 0.0)
                })
                .collect::<Vec<_>>();
            forward.process(&mut spectrum);
            for (sum, value) in linked.iter_mut().zip(&spectrum) {
                *sum += value.norm_sqr();
            }
            spectra.push(spectrum);
        }
        let dominant = (1..linked.len() - 1)
            .max_by(|left, right| linked[*left].total_cmp(&linked[*right]))
            .unwrap_or(1);
        hash(&mut decision_hash, frame.layer as u64);
        hash(&mut decision_hash, dominant as u64);
        for (channel_index, spectrum) in spectra.iter_mut().enumerate() {
            for value in spectrum.iter().take(length / 2 + 1) {
                hash(&mut magnitude_hash, value.norm().to_bits());
            }
            let changes = transport(
                spectrum,
                frame,
                &mut states[channel_index],
                events,
                dominant,
                mode,
                configuration,
                ratio == 1.0,
            );
            event_resets += changes.0;
            vertical_alignments += changes.1;
            mirror(spectrum);
            for bin in 0..length {
                let mirror_bin = if bin == 0 { 0 } else { length - bin };
                symmetry_error =
                    symmetry_error.max((spectrum[bin] - spectrum[mirror_bin].conj()).norm());
                non_finite +=
                    usize::from(!spectrum[bin].re.is_finite() || !spectrum[bin].im.is_finite());
                hash(&mut phase_hash, spectrum[bin].arg().to_bits());
            }
            inverse.process(spectrum);
            for (offset, (sample, weight)) in spectrum.iter().zip(&window).enumerate() {
                let logical = frame.output - length as isize / 2 + offset as isize;
                let domain = (logical - output_start) as usize;
                let value = *sample * (*weight / (length as f64 * operator[domain]));
                imaginary_residue = imaginary_residue.max(value.im.abs());
                outputs[channel_index][domain] += value;
                if let Some(layer_outputs) = &mut layer_outputs {
                    layer_outputs[frame.layer][channel_index][domain] += value;
                }
            }
        }
    }
    let crop = (-output_start) as usize;
    let (samples, layer_samples) = crop_outputs(&outputs, layer_outputs, crop, target_len);
    let uncovered = operator[crop..crop + target_len]
        .iter()
        .filter(|value| **value <= 0.0)
        .count();
    non_finite += samples
        .iter()
        .flatten()
        .filter(|sample| !sample.is_finite())
        .count();
    let boundary_failures = usize::from(samples.iter().any(|channel| {
        channel.first().is_none_or(|value| !value.is_finite())
            || channel.last().is_none_or(|value| !value.is_finite())
    }));
    let projected = events
        .iter()
        .map(|event| (ratio * *event as f64).round() as usize)
        .collect::<Vec<_>>();
    let event_order_failures = projected
        .windows(2)
        .filter(|pair| pair[1] <= pair[0])
        .count();
    let mut output_hash = HASH_OFFSET;
    for sample in samples.iter().flatten() {
        hash(&mut output_hash, sample.to_bits());
    }
    Render {
        samples,
        layer_samples,
        target_len,
        uncovered,
        boundary_failures,
        event_order_failures,
        symmetry_error,
        imaginary_residue,
        non_finite,
        event_resets,
        vertical_alignments,
        schedule_hash: schedule.hash,
        magnitude_hash,
        phase_hash,
        output_hash,
        channel_decision_hash: decision_hash,
    }
}
