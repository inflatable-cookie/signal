use rustfft::{num_complex::Complex64, Fft, FftPlanner};
use std::sync::Arc;

mod support;
use support::{assert_identity, assert_mechanism, controls, empty_evidence, peak_index};
mod kernel;
use kernel::{hop_extrema, level, propagate_phases, reflected_sample, schedule, window};

const FFT_FRAMES: usize = 4_096;
const GUARD_FRAMES: isize = 4_096;
const WINDOW_LENGTHS: [usize; 4] = [512, 1_024, 2_048, 4_096];
const HASH_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Frame {
    source_center: isize,
    output_center: isize,
    length: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct Evidence {
    window_counts: [usize; 4],
    source_hop_extrema: [usize; 2],
    output_hop_extrema: [usize; 2],
    maximum_mapping_error: f64,
    frame_operator_bounds: [f64; 2],
    uncovered_output_frames: usize,
    illegal_transitions: usize,
    reflected_reads: usize,
    coefficient_count: usize,
    phase_count: usize,
    conjugate_symmetry_error: f64,
    imaginary_residue: f64,
    non_finite_values: usize,
    hashes: [u64; 3],
}

#[derive(Clone, Debug, PartialEq)]
struct Render {
    samples: Vec<f64>,
    evidence: Evidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    TargetedMonoGate,
    RetireTimeAdaptiveSynthesis,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub(super) impulse_errors: [isize; 4],
    pub(super) direction: Direction,
}

pub(super) fn oracle_adaptive_synthesis_review() -> Review {
    let controls = controls();
    let mut impulse_errors = [0; 4];
    for (name, input, events) in &controls {
        for (ratio_index, ratio) in [1.0, 0.75, 1.25, 1.5].into_iter().enumerate() {
            let first = render(input, ratio, events);
            let repeated = render(input, ratio, events);
            assert_eq!(first, repeated, "{name} {ratio} repeat");
            assert_mechanism(name, ratio, &first);
            if ratio == 1.0 {
                assert_identity(name, input, &first.samples);
            }
            if *name == "impulse" {
                let expected = (events[0] as f64 * ratio).round() as usize;
                impulse_errors[ratio_index] =
                    peak_index(&first.samples) as isize - expected as isize;
            }
        }
    }
    Review {
        impulse_errors,
        direction: if impulse_errors.iter().all(|error| error.abs() <= 1) {
            Direction::TargetedMonoGate
        } else {
            Direction::RetireTimeAdaptiveSynthesis
        },
    }
}

fn render(input: &[f64], ratio: f64, events: &[usize]) -> Render {
    if input.is_empty() {
        return Render {
            samples: Vec::new(),
            evidence: empty_evidence(),
        };
    }
    let frames = schedule(input.len(), ratio, events);
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let output_start = frames
        .iter()
        .map(|frame| frame.output_center - frame.length as isize / 2)
        .min()
        .unwrap();
    let output_end = frames
        .iter()
        .map(|frame| frame.output_center + frame.length as isize / 2)
        .max()
        .unwrap();
    let mut frame_operator = vec![0.0; (output_end - output_start) as usize];
    let mut window_counts = [0; 4];
    let mut reflected_reads = 0;
    let mut illegal_transitions = 0;
    let mut schedule_hash = HASH_OFFSET;
    for (index, frame) in frames.iter().enumerate() {
        window_counts[level(frame.length)] += 1;
        hash_i64(&mut schedule_hash, frame.source_center as i64);
        hash_i64(&mut schedule_hash, frame.output_center as i64);
        hash_u64(&mut schedule_hash, frame.length as u64);
        let window = window(frame.length);
        for (offset, weight) in window.iter().copied().enumerate() {
            let source = frame.source_center - frame.length as isize / 2 + offset as isize;
            reflected_reads += usize::from(source < 0 || source >= input.len() as isize);
            let output = frame.output_center - frame.length as isize / 2 + offset as isize;
            frame_operator[(output - output_start) as usize] += weight * weight;
        }
        if let Some(next) = frames.get(index + 1) {
            let hop = (next.source_center - frame.source_center) as usize;
            illegal_transitions += usize::from(
                level(frame.length).abs_diff(level(next.length)) > 1
                    || hop != frame.length.min(next.length) / 4,
            );
        }
    }

    let mut engine = Engine::new();
    let mut output = vec![Complex64::new(0.0, 0.0); frame_operator.len()];
    let mut coefficient_hash = HASH_OFFSET;
    let mut phase_count = 0;
    let mut symmetry_error = 0.0_f64;
    let mut imaginary_residue = 0.0_f64;
    let mut non_finite = 0;
    let mut previous_analysis_phase = vec![0.0; FFT_FRAMES / 2 + 1];
    let mut synthesis_phase = vec![0.0; FFT_FRAMES / 2 + 1];
    for (frame_index, frame) in frames.iter().enumerate() {
        let window = window(frame.length);
        let offset = (FFT_FRAMES - frame.length) / 2;
        engine.buffer.fill(Complex64::new(0.0, 0.0));
        for (local, weight) in window.iter().copied().enumerate() {
            let source = frame.source_center - frame.length as isize / 2 + local as isize;
            engine.buffer[offset + local].re = reflected_sample(input, source) * weight;
        }
        engine.forward.process(&mut engine.buffer);
        if ratio != 1.0 {
            propagate_phases(
                frame_index,
                &frames,
                &mut engine.buffer,
                &mut previous_analysis_phase,
                &mut synthesis_phase,
                &mut phase_count,
            );
        }
        for value in &engine.buffer {
            hash_f64(&mut coefficient_hash, value.re);
            hash_f64(&mut coefficient_hash, value.im);
            non_finite += usize::from(!value.re.is_finite() || !value.im.is_finite());
        }
        for bin in 0..FFT_FRAMES {
            let mirror = if bin == 0 { 0 } else { FFT_FRAMES - bin };
            symmetry_error =
                symmetry_error.max((engine.buffer[bin] - engine.buffer[mirror].conj()).norm());
        }
        engine.inverse.process(&mut engine.buffer);
        for (local, weight) in window.iter().copied().enumerate() {
            let logical = frame.output_center - frame.length as isize / 2 + local as isize;
            let domain = (logical - output_start) as usize;
            let dual = weight / frame_operator[domain];
            let value = engine.buffer[offset + local] * (dual / FFT_FRAMES as f64);
            imaginary_residue = imaginary_residue.max(value.im.abs());
            output[domain] += value;
        }
    }
    let crop_start = (-output_start) as usize;
    let samples = output[crop_start..crop_start + target_len]
        .iter()
        .map(|value| value.re)
        .collect::<Vec<_>>();
    non_finite += samples.iter().filter(|sample| !sample.is_finite()).count();
    let crop_operator = &frame_operator[crop_start..crop_start + target_len];
    let frame_min = crop_operator.iter().copied().fold(f64::INFINITY, f64::min);
    let frame_max = crop_operator.iter().copied().fold(0.0_f64, f64::max);
    let source_hops = hop_extrema(&frames, |frame| frame.source_center);
    let output_hops = hop_extrema(&frames, |frame| frame.output_center);
    let mut sample_hash = HASH_OFFSET;
    samples
        .iter()
        .for_each(|sample| hash_f64(&mut sample_hash, *sample));
    Render {
        samples,
        evidence: Evidence {
            window_counts,
            source_hop_extrema: source_hops,
            output_hop_extrema: output_hops,
            maximum_mapping_error: frames
                .iter()
                .map(|frame| {
                    (frame.output_center as f64 - ratio * frame.source_center as f64).abs()
                })
                .fold(0.0, f64::max),
            frame_operator_bounds: [frame_min, frame_max],
            uncovered_output_frames: crop_operator.iter().filter(|value| **value <= 0.0).count(),
            illegal_transitions,
            reflected_reads,
            coefficient_count: frames.len() * FFT_FRAMES,
            phase_count,
            conjugate_symmetry_error: symmetry_error,
            imaginary_residue,
            non_finite_values: non_finite,
            hashes: [schedule_hash, coefficient_hash, sample_hash],
        },
    }
}

struct Engine {
    forward: Arc<dyn Fft<f64>>,
    inverse: Arc<dyn Fft<f64>>,
    buffer: Vec<Complex64>,
}

impl Engine {
    fn new() -> Self {
        let mut planner = FftPlanner::new();
        Self {
            forward: planner.plan_fft_forward(FFT_FRAMES),
            inverse: planner.plan_fft_inverse(FFT_FRAMES),
            buffer: vec![Complex64::new(0.0, 0.0); FFT_FRAMES],
        }
    }
}

fn hash_i64(hash: &mut u64, value: i64) {
    hash_u64(hash, value as u64);
}
fn hash_f64(hash: &mut u64, value: f64) {
    hash_u64(hash, value.to_bits());
}
fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
