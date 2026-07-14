use rustfft::{num_complex::Complex64, FftPlanner};

use super::HASH_OFFSET;

pub(in crate::frequency_adaptive) mod attribution;

const SAMPLE_RATE: usize = 8_000;
const RATIOS: [f64; 4] = [0.75, 1.25, 1.5, 2.0];
const BASS_FREQUENCIES: [f64; 3] = [55.0, 82.4069, 110.0];
const CHORD_FREQUENCIES: [f64; 4] = [110.0, 164.8138, 220.0, 329.6276];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::frequency_adaptive) struct MechanismCounts {
    pub(in crate::frequency_adaptive) horizontal: usize,
    pub(in crate::frequency_adaptive) short_lower: usize,
    pub(in crate::frequency_adaptive) short_upper: usize,
    pub(in crate::frequency_adaptive) long_lower: usize,
    pub(in crate::frequency_adaptive) long_upper: usize,
    pub(in crate::frequency_adaptive) corrected: usize,
    pub(in crate::frequency_adaptive) fallback: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum Direction {
    RealSourceComparison,
    PredictorResearch,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct Review {
    pub(in crate::frequency_adaptive) geometry: [usize; 3],
    pub(in crate::frequency_adaptive) structural_failures: [usize; 5],
    pub(in crate::frequency_adaptive) maximum_bass_error_hz: f64,
    pub(in crate::frequency_adaptive) octave_failures: usize,
    pub(in crate::frequency_adaptive) maximum_chord_peak_error_hz: f64,
    pub(in crate::frequency_adaptive) chord_input_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) maximum_event_error_frames: usize,
    pub(in crate::frequency_adaptive) replica_failures: usize,
    pub(in crate::frequency_adaptive) silence_peak: f64,
    pub(in crate::frequency_adaptive) mechanisms: MechanismCounts,
    pub(in crate::frequency_adaptive) output_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: Direction,
}

#[derive(Clone, Debug)]
pub(super) struct Render {
    samples: Vec<f64>,
    target_len: usize,
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    mechanisms: MechanismCounts,
    maximum_normalization_phase_delta: f64,
    significant_fallback: usize,
    hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum TraceStage {
    Analysis,
    Horizontal,
    ShortLower,
    ShortUpper,
    LongLower,
    LongUpper,
    Complete,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ChordSpectrumMetrics {
    pub maximum_peak_error_hz: f64,
    pub out_of_band_db: f64,
    pub strongest_sideband_hz: f64,
    pub strongest_sideband_offset_hz: f64,
}

pub(in crate::frequency_adaptive) fn review() -> Review {
    let first = run();
    let second = run();
    let repeated = first == second;
    let passed = repeated
        && first.structural_failures == [0; 5]
        && first.maximum_bass_error_hz <= 0.5
        && first.octave_failures == 0
        && first.maximum_chord_peak_error_hz <= 0.5
        && first.chord_input_out_of_band_db <= -60.0
        && first.chord_out_of_band_db <= -60.0
        && first.maximum_event_error_frames <= 256
        && first.replica_failures == 0
        && first.silence_peak == 0.0
        && mechanisms_exercised(first.mechanisms);
    Review {
        repeated,
        direction: if passed {
            Direction::RealSourceComparison
        } else {
            Direction::PredictorResearch
        },
        ..first
    }
}

fn run() -> Review {
    let hop = ((SAMPLE_RATE as f64 * 0.03).round() as usize).max(1);
    let length = 4 * hop;
    let mut structural_failures = [0; 5];
    let mut mechanisms = MechanismCounts::default();
    let mut output_hash = HASH_OFFSET;

    let structural = structural_control();
    for ratio in RATIOS {
        let render = render(&structural, ratio, SAMPLE_RATE);
        structural_failures[0] += usize::from(render.samples.len() != render.target_len);
        structural_failures[1] += render.non_finite;
        structural_failures[2] += render.uncovered;
        structural_failures[3] += render.boundary_failures;
        structural_failures[4] += usize::from(render.hash == HASH_OFFSET);
        add_counts(&mut mechanisms, render.mechanisms);
        mix(&mut output_hash, render.hash);
    }
    let identity = render(&structural, 1.0, SAMPLE_RATE);
    structural_failures[4] += identity
        .samples
        .iter()
        .zip(&structural)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();
    mix(&mut output_hash, identity.hash);

    let (maximum_bass_error_hz, octave_failures, bass_render) = bass_review();
    add_counts(&mut mechanisms, bass_render.mechanisms);
    mix(&mut output_hash, bass_render.hash);

    let (
        maximum_chord_peak_error_hz,
        chord_input_out_of_band_db,
        chord_out_of_band_db,
        chord_render,
    ) = chord_review();
    add_counts(&mut mechanisms, chord_render.mechanisms);
    mix(&mut output_hash, chord_render.hash);

    let (maximum_event_error_frames, replica_failures, transient_render) = transient_review();
    add_counts(&mut mechanisms, transient_render.mechanisms);
    mix(&mut output_hash, transient_render.hash);

    let silence = vec![0.0; SAMPLE_RATE];
    let silence_render = render(&silence, 1.5, SAMPLE_RATE);
    let silence_peak = silence_render
        .samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    add_counts(&mut mechanisms, silence_render.mechanisms);
    mix(&mut output_hash, silence_render.hash);

    // Two exactly opposed components exercise the weak-evidence path as a
    // cancellation control, independently of the literal silence vector.
    let cancellation = (0..SAMPLE_RATE)
        .map(|index| {
            let value = (std::f64::consts::TAU * 220.0 * index as f64 / SAMPLE_RATE as f64).sin();
            value + -value
        })
        .collect::<Vec<_>>();
    let cancellation_render = render(&cancellation, 0.75, SAMPLE_RATE);
    add_counts(&mut mechanisms, cancellation_render.mechanisms);
    mix(&mut output_hash, cancellation_render.hash);

    Review {
        geometry: [hop, length, length / hop],
        structural_failures,
        maximum_bass_error_hz,
        octave_failures,
        maximum_chord_peak_error_hz,
        chord_input_out_of_band_db,
        chord_out_of_band_db,
        maximum_event_error_frames,
        replica_failures,
        silence_peak,
        mechanisms,
        output_hash,
        repeated: false,
        direction: Direction::PredictorResearch,
    }
}

fn render(input: &[f64], ratio: f64, sample_rate: usize) -> Render {
    render_stage(input, ratio, sample_rate, TraceStage::Complete)
}

pub(super) fn render_stage(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
) -> Render {
    let target_len = (input.len() as f64 * ratio).round() as usize;
    if ratio == 1.0 && trace_stage == TraceStage::Complete {
        let samples = input.to_vec();
        return Render {
            hash: hash_samples(&samples),
            samples,
            target_len,
            uncovered: 0,
            non_finite: 0,
            boundary_failures: 0,
            mechanisms: MechanismCounts::default(),
            maximum_normalization_phase_delta: 0.0,
            significant_fallback: 0,
        };
    }
    let hop = ((sample_rate as f64 * 0.03).round() as usize).max(1);
    let length = 4 * hop;
    let bins = length / 2 + 1;
    let long_distance = ((length as f64 / hop as f64).round() as usize).max(1);
    let window = (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(length);
    let inverse = planner.plan_fft_inverse(length);
    let mut output = vec![0.0; target_len];
    let mut normalization = vec![0.0; target_len];
    let mut previous_output = vec![Complex64::new(0.0, 0.0); bins];
    let mut previous_source_center: Option<isize> = None;
    let mut mechanisms = MechanismCounts::default();
    let mut maximum_normalization_phase_delta = 0.0_f64;
    let mut significant_fallback = 0;
    let mut output_center = -(length as isize / 2);
    while output_center < target_len as isize + length as isize / 2 {
        let source_center = (output_center as f64 / ratio).round() as isize;
        let current = analyse(input, source_center, &window, &forward);
        let auxiliary = analyse(input, source_center - hop as isize, &window, &forward);
        let mut preliminary = current[..bins].to_vec();
        let mut traced = preliminary.clone();
        if let Some(previous_source_center) = previous_source_center {
            for bin in 0..bins {
                let prediction = previous_output[bin] * current[bin] * auxiliary[bin].conj();
                preliminary[bin] = normalize_or(prediction, current[bin], current[bin]);
                mechanisms.horizontal += 1;
            }
            let input_hop = (source_center - previous_source_center)
                .unsigned_abs()
                .max(1);
            let time_factor = hop as f64 / input_hop as f64;
            let significant_energy = current[..bins]
                .iter()
                .map(Complex64::norm_sqr)
                .fold(0.0, f64::max)
                * 1.0e-8;
            let mut corrected = preliminary.clone();
            for bin in 0..bins {
                let mut prediction = Complex64::new(0.0, 0.0);
                let mut selected = Complex64::new(0.0, 0.0);
                if bin >= 1 {
                    let lower_input = interpolate(&current[..bins], bin as f64 - time_factor);
                    let twist = current[bin] * lower_input.conj();
                    let candidate = corrected[bin - 1] * twist;
                    prediction += candidate;
                    if trace_stage == TraceStage::ShortLower {
                        selected = candidate;
                    }
                    mechanisms.short_lower += 1;
                }
                if bin + 1 < bins {
                    let lower_input = interpolate(&current[..bins], bin as f64 + 1.0 - time_factor);
                    let twist = current[bin + 1] * lower_input.conj();
                    let candidate = preliminary[bin + 1] * twist.conj();
                    prediction += candidate;
                    if trace_stage == TraceStage::ShortUpper {
                        selected = candidate;
                    }
                    mechanisms.short_upper += 1;
                }
                if bin >= long_distance {
                    let lower_input = interpolate(
                        &current[..bins],
                        bin as f64 - long_distance as f64 * time_factor,
                    );
                    let twist = current[bin] * lower_input.conj();
                    let candidate = corrected[bin - long_distance] * twist;
                    prediction += candidate;
                    if trace_stage == TraceStage::LongLower {
                        selected = candidate;
                    }
                    mechanisms.long_lower += 1;
                }
                if bin + long_distance < bins {
                    let lower_input = interpolate(
                        &current[..bins],
                        bin as f64 + long_distance as f64 - long_distance as f64 * time_factor,
                    );
                    let twist = current[bin + long_distance] * lower_input.conj();
                    let candidate = preliminary[bin + long_distance] * twist.conj();
                    prediction += candidate;
                    if trace_stage == TraceStage::LongUpper {
                        selected = candidate;
                    }
                    mechanisms.long_upper += 1;
                }
                let target_energy = current[bin].norm_sqr();
                let prediction_energy = prediction.norm_sqr();
                let floor = target_energy * f64::EPSILON * 64.0;
                if prediction_energy > floor {
                    corrected[bin] = prediction * (target_energy / prediction_energy).sqrt();
                    maximum_normalization_phase_delta = maximum_normalization_phase_delta
                        .max(wrap(corrected[bin].arg() - prediction.arg()).abs());
                    mechanisms.corrected += 1;
                } else {
                    corrected[bin] = current[bin];
                    mechanisms.fallback += 1;
                    significant_fallback += usize::from(target_energy > significant_energy);
                }
                traced[bin] = match trace_stage {
                    TraceStage::Analysis => current[bin],
                    TraceStage::Horizontal => preliminary[bin],
                    TraceStage::Complete => corrected[bin],
                    _ => normalize_or(selected, current[bin], current[bin]),
                };
            }
            preliminary = corrected;
        }
        preliminary[0].im = 0.0;
        traced[0].im = 0.0;
        if length % 2 == 0 {
            preliminary[bins - 1].im = 0.0;
            traced[bins - 1].im = 0.0;
        }
        let mut spectrum = vec![Complex64::new(0.0, 0.0); length];
        spectrum[..bins].copy_from_slice(&traced);
        for bin in 1..length / 2 {
            spectrum[length - bin] = spectrum[bin].conj();
        }
        inverse.process(&mut spectrum);
        for offset in 0..length {
            let output_index = output_center - length as isize / 2 + offset as isize;
            if (0..target_len as isize).contains(&output_index) {
                let output_index = output_index as usize;
                output[output_index] += spectrum[offset].re * window[offset] / length as f64;
                normalization[output_index] += window[offset] * window[offset];
            }
        }
        previous_output = preliminary;
        previous_source_center = Some(source_center);
        output_center += hop as isize;
    }
    let uncovered = normalization.iter().filter(|value| **value <= 0.0).count();
    for (sample, weight) in output.iter_mut().zip(normalization) {
        if weight > 0.0 {
            *sample /= weight;
        }
    }
    let non_finite = output.iter().filter(|sample| !sample.is_finite()).count();
    let boundary_failures = usize::from(output.first().is_none_or(|sample| !sample.is_finite()))
        + usize::from(output.last().is_none_or(|sample| !sample.is_finite()));
    Render {
        hash: hash_samples(&output),
        samples: output,
        target_len,
        uncovered,
        non_finite,
        boundary_failures,
        mechanisms,
        maximum_normalization_phase_delta,
        significant_fallback,
    }
}

fn analyse(
    input: &[f64],
    center: isize,
    window: &[f64],
    forward: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<Complex64> {
    let length = window.len();
    let mut spectrum = (0..length)
        .map(|offset| {
            let index = center - length as isize / 2 + offset as isize;
            Complex64::new(reflected(input, index) * window[offset], 0.0)
        })
        .collect::<Vec<_>>();
    forward.process(&mut spectrum);
    spectrum
}

fn reflected(input: &[f64], mut index: isize) -> f64 {
    let end = input.len() as isize - 1;
    while index < 0 || index > end {
        index = if index < 0 {
            -index - 1
        } else {
            2 * end - index + 1
        };
    }
    input[index as usize]
}

fn interpolate(spectrum: &[Complex64], position: f64) -> Complex64 {
    let position = position.clamp(0.0, (spectrum.len() - 1) as f64);
    let lower = position.floor() as usize;
    let upper = (lower + 1).min(spectrum.len() - 1);
    let fraction = position - lower as f64;
    spectrum[lower] * (1.0 - fraction) + spectrum[upper] * fraction
}

fn normalize_or(prediction: Complex64, target: Complex64, fallback: Complex64) -> Complex64 {
    let prediction_energy = prediction.norm_sqr();
    if prediction_energy > target.norm_sqr() * f64::EPSILON * 64.0 {
        prediction * (target.norm_sqr() / prediction_energy).sqrt()
    } else {
        fallback
    }
}

fn structural_control() -> Vec<f64> {
    (0..SAMPLE_RATE)
        .map(|index| {
            let time = index as f64 / SAMPLE_RATE as f64;
            let tone = 0.25 * (std::f64::consts::TAU * 110.0 * time).sin()
                + 0.15 * (std::f64::consts::TAU * 329.6276 * time).sin();
            let attack = if index == SAMPLE_RATE / 3 { 0.8 } else { 0.0 };
            tone + attack
        })
        .collect()
}

fn bass_review() -> (f64, usize, Render) {
    let note_frames = SAMPLE_RATE;
    let input = (0..note_frames * BASS_FREQUENCIES.len())
        .map(|index| {
            let frequency = BASS_FREQUENCIES[index / note_frames];
            (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin() * 0.5
        })
        .collect::<Vec<_>>();
    let ratio = 1.5;
    let render = render(&input, ratio, SAMPLE_RATE);
    let mut maximum_error = 0.0_f64;
    let mut octave_failures = 0;
    for (note, expected) in BASS_FREQUENCIES.into_iter().enumerate() {
        let start = ((note as f64 + 0.25) * note_frames as f64 * ratio).round() as usize;
        let end = ((note as f64 + 0.75) * note_frames as f64 * ratio).round() as usize;
        let measured = zero_crossing_frequency(&render.samples[start..end], SAMPLE_RATE as f64);
        maximum_error = maximum_error.max((measured - expected).abs());
        octave_failures += usize::from(
            (measured - expected * 2.0).abs() < (measured - expected).abs()
                || (measured - expected * 0.5).abs() < (measured - expected).abs(),
        );
    }
    (maximum_error, octave_failures, render)
}

fn zero_crossing_frequency(samples: &[f64], sample_rate: f64) -> f64 {
    let crossings = samples
        .windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            (pair[0] <= 0.0 && pair[1] > 0.0)
                .then(|| index as f64 + (-pair[0] / (pair[1] - pair[0])).clamp(0.0, 1.0))
        })
        .collect::<Vec<_>>();
    if crossings.len() < 2 {
        return f64::INFINITY;
    }
    (crossings.len() - 1) as f64 * sample_rate / (crossings[crossings.len() - 1] - crossings[0])
}

fn chord_review() -> (f64, f64, f64, Render) {
    let input = chord_control();
    let render = render(&input, 2.0, SAMPLE_RATE);
    let input_metrics = chord_spectrum_metrics(&input);
    let output_metrics = chord_spectrum_metrics(&render.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    (
        output_metrics.maximum_peak_error_hz,
        input_metrics.out_of_band_db,
        output_metrics.out_of_band_db,
        render,
    )
}

pub(super) fn chord_control() -> Vec<f64> {
    chord_control_frames(SAMPLE_RATE * 2)
}

pub(super) fn chord_control_frames(frames: usize) -> Vec<f64> {
    (0..frames)
        .map(|index| {
            CHORD_FREQUENCIES
                .iter()
                .enumerate()
                .map(|(tone, frequency)| {
                    (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin()
                        * (0.16 - tone as f64 * 0.015)
                })
                .sum::<f64>()
        })
        .collect()
}

pub(super) fn chord_spectrum_metrics(segment: &[f64]) -> ChordSpectrumMetrics {
    let fft_len = segment.len().next_power_of_two();
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(fft_len);
    let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_len];
    for (index, sample) in segment.iter().enumerate() {
        let weight =
            0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / segment.len() as f64).cos();
        spectrum[index] = Complex64::new(sample * weight, 0.0);
    }
    fft.process(&mut spectrum);
    let bin_hz = SAMPLE_RATE as f64 / fft_len as f64;
    let mut maximum_peak_error = 0.0_f64;
    for expected in CHORD_FREQUENCIES {
        let nominal = (expected / bin_hz).round() as usize;
        let peak = (nominal.saturating_sub(2)..=(nominal + 2).min(fft_len / 2))
            .max_by(|left, right| {
                spectrum[*left]
                    .norm_sqr()
                    .total_cmp(&spectrum[*right].norm_sqr())
            })
            .unwrap_or(nominal);
        let refined = parabolic_peak(&spectrum, peak) * bin_hz;
        maximum_peak_error = maximum_peak_error.max((refined - expected).abs());
    }
    let total = spectrum[..=fft_len / 2]
        .iter()
        .map(Complex64::norm_sqr)
        .sum::<f64>();
    let out_of_band = spectrum[..=fft_len / 2]
        .iter()
        .enumerate()
        .filter(|(bin, _)| {
            CHORD_FREQUENCIES
                .iter()
                .all(|frequency| (*bin as f64 * bin_hz - frequency).abs() > 8.0)
        })
        .map(|(_, value)| value.norm_sqr())
        .sum::<f64>();
    let strongest_sideband = spectrum[..=fft_len / 2]
        .iter()
        .enumerate()
        .filter(|(bin, _)| {
            CHORD_FREQUENCIES
                .iter()
                .all(|frequency| (*bin as f64 * bin_hz - frequency).abs() > 8.0)
        })
        .max_by(|(_, left), (_, right)| left.norm_sqr().total_cmp(&right.norm_sqr()))
        .map(|(bin, _)| bin as f64 * bin_hz)
        .unwrap_or(0.0);
    let strongest_sideband_offset_hz = CHORD_FREQUENCIES
        .iter()
        .map(|frequency| (strongest_sideband - frequency).abs())
        .fold(f64::INFINITY, f64::min);
    let out_of_band_db =
        10.0 * (out_of_band.max(f64::MIN_POSITIVE) / total.max(f64::MIN_POSITIVE)).log10();
    ChordSpectrumMetrics {
        maximum_peak_error_hz: maximum_peak_error,
        out_of_band_db,
        strongest_sideband_hz: strongest_sideband,
        strongest_sideband_offset_hz,
    }
}

fn parabolic_peak(spectrum: &[Complex64], bin: usize) -> f64 {
    if bin == 0 || bin + 1 >= spectrum.len() {
        return bin as f64;
    }
    let left = spectrum[bin - 1].norm_sqr().max(f64::MIN_POSITIVE).ln();
    let center = spectrum[bin].norm_sqr().max(f64::MIN_POSITIVE).ln();
    let right = spectrum[bin + 1].norm_sqr().max(f64::MIN_POSITIVE).ln();
    let denominator = left - 2.0 * center + right;
    bin as f64
        + if denominator.abs() > 1.0e-12 {
            0.5 * (left - right) / denominator
        } else {
            0.0
        }
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

fn transient_review() -> (usize, usize, Render) {
    let events = [SAMPLE_RATE / 3, SAMPLE_RATE, SAMPLE_RATE + SAMPLE_RATE / 6];
    let mut input = vec![0.0; SAMPLE_RATE * 2];
    for (event, amplitude) in events.into_iter().zip([1.0, 0.8, 0.7]) {
        input[event] = amplitude;
    }
    let ratio = 2.0;
    let render = render(&input, ratio, SAMPLE_RATE);
    let targets = events.map(|event| (event as f64 * ratio).round() as usize);
    let mut maximum_error = 0;
    let mut peaks = [0.0; 3];
    for (index, target) in targets.into_iter().enumerate() {
        let start = target.saturating_sub(256);
        let end = (target + 256).min(render.samples.len() - 1);
        let (peak_index, peak) = render.samples[start..=end]
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))
            .map(|(offset, value)| (start + offset, value.abs()))
            .unwrap_or((target, 0.0));
        maximum_error = maximum_error.max(peak_index.abs_diff(target));
        peaks[index] = peak;
    }
    let midpoint = (targets[1] + targets[2]) / 2;
    let midpoint_peak = render.samples[midpoint.saturating_sub(128)..=(midpoint + 128)]
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f64::max);
    let replica_failures = usize::from(midpoint_peak > peaks[1].min(peaks[2]));
    (maximum_error, replica_failures, render)
}

fn mechanisms_exercised(counts: MechanismCounts) -> bool {
    counts.horizontal > 0
        && counts.short_lower > 0
        && counts.short_upper > 0
        && counts.long_lower > 0
        && counts.long_upper > 0
        && counts.corrected > 0
        && counts.fallback > 0
}

fn add_counts(target: &mut MechanismCounts, source: MechanismCounts) {
    target.horizontal += source.horizontal;
    target.short_lower += source.short_lower;
    target.short_upper += source.short_upper;
    target.long_lower += source.long_lower;
    target.long_upper += source.long_upper;
    target.corrected += source.corrected;
    target.fallback += source.fallback;
}

fn hash_samples(samples: &[f64]) -> u64 {
    let mut state = HASH_OFFSET;
    for sample in samples {
        mix(&mut state, sample.to_bits());
    }
    state
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
