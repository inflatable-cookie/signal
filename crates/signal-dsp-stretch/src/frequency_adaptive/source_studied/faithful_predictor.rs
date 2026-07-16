use rustfft::{num_complex::Complex64, FftPlanner};

use super::HASH_OFFSET;

pub(in crate::frequency_adaptive) mod analysis_grid;
pub(in crate::frequency_adaptive) mod analysis_interaction;
pub(in crate::frequency_adaptive) mod analysis_window;
pub(in crate::frequency_adaptive) mod attribution;
pub(in crate::frequency_adaptive) mod coherent_representation;
pub(in crate::frequency_adaptive) mod concealed_comparison;
pub(in crate::frequency_adaptive) mod linked_stereo;
pub(in crate::frequency_adaptive) mod pinned_source;
pub(in crate::frequency_adaptive) mod real_source_confirmation;
pub(in crate::frequency_adaptive) mod rubber_band_comparison;
pub(in crate::frequency_adaptive) mod stage_trace;

const SAMPLE_RATE: usize = 8_000;
const HORIZONTAL_ENERGY_FLOOR: f64 = 1.0e-15;
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
    PinnedSourceParity,
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
    horizontal_ratio_errors: Vec<[f64; 4]>,
    horizontal_output_phases: Vec<[f64; 4]>,
    stage_trace: Option<StageFrameTrace>,
    hash: u64,
}

#[derive(Clone, Debug)]
pub(super) struct StageFrameTrace {
    source_center: isize,
    current: Vec<Complex64>,
    preliminary: Vec<Complex64>,
    corrected: Vec<Complex64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum TraceStage {
    Analysis,
    Horizontal,
    HorizontalPhaseRecurrence,
    ShortLower,
    ShortUpper,
    LongLower,
    LongUpper,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum FrequencyBoundaryPolicy {
    Clamp,
    ZeroExtend,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum TransformGrid {
    Standard,
    ModifiedHalfBin,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ChordSpectrumMetrics {
    pub maximum_peak_error_hz: f64,
    pub out_of_band_db: f64,
    pub strongest_sideband_hz: f64,
    pub strongest_sideband_offset_hz: f64,
}

pub(in crate::frequency_adaptive) fn review() -> Review {
    review_representation(TransformGrid::Standard, None)
}

pub(super) fn review_with_grid_and_window(window: &[f64]) -> Review {
    review_representation(TransformGrid::ModifiedHalfBin, Some(window))
}

fn review_representation(grid: TransformGrid, window: Option<&[f64]>) -> Review {
    let first = run(grid, window);
    let second = run(grid, window);
    let repeated = first == second;
    let passed = repeated
        && first.structural_failures == [0; 5]
        && first.maximum_bass_error_hz <= 0.5
        && first.octave_failures == 0
        && first.maximum_chord_peak_error_hz <= 0.5
        && first.chord_input_out_of_band_db <= -60.0
        && first.maximum_event_error_frames <= 256
        && first.replica_failures == 0
        && first.silence_peak == 0.0
        && mechanisms_exercised(first.mechanisms);
    Review {
        repeated,
        direction: if passed {
            Direction::PinnedSourceParity
        } else {
            Direction::PredictorResearch
        },
        ..first
    }
}

fn run(grid: TransformGrid, window: Option<&[f64]>) -> Review {
    let hop = ((SAMPLE_RATE as f64 * 0.03).round() as usize).max(1);
    let length = 4 * hop;
    let mut structural_failures = [0; 5];
    let mut mechanisms = MechanismCounts::default();
    let mut output_hash = HASH_OFFSET;

    let structural = structural_control();
    for ratio in RATIOS {
        let render = render_with_representation(&structural, ratio, SAMPLE_RATE, grid, window);
        structural_failures[0] += usize::from(render.samples.len() != render.target_len);
        structural_failures[1] += render.non_finite;
        structural_failures[2] += render.uncovered;
        structural_failures[3] += render.boundary_failures;
        structural_failures[4] += usize::from(render.hash == HASH_OFFSET);
        add_counts(&mut mechanisms, render.mechanisms);
        mix(&mut output_hash, render.hash);
    }
    let identity = render_with_representation(&structural, 1.0, SAMPLE_RATE, grid, window);
    structural_failures[4] += identity
        .samples
        .iter()
        .zip(&structural)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();
    mix(&mut output_hash, identity.hash);

    let (maximum_bass_error_hz, octave_failures, bass_render) = bass_review(grid, window);
    add_counts(&mut mechanisms, bass_render.mechanisms);
    mix(&mut output_hash, bass_render.hash);

    let (
        maximum_chord_peak_error_hz,
        chord_input_out_of_band_db,
        chord_out_of_band_db,
        chord_render,
    ) = chord_review(grid, window);
    add_counts(&mut mechanisms, chord_render.mechanisms);
    mix(&mut output_hash, chord_render.hash);

    let (maximum_event_error_frames, replica_failures, transient_render) =
        transient_review(grid, window);
    add_counts(&mut mechanisms, transient_render.mechanisms);
    mix(&mut output_hash, transient_render.hash);

    let silence = vec![0.0; SAMPLE_RATE];
    let silence_render = render_with_representation(&silence, 1.5, SAMPLE_RATE, grid, window);
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
    let cancellation_render =
        render_with_representation(&cancellation, 0.75, SAMPLE_RATE, grid, window);
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

fn render_with_representation(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    grid: TransformGrid,
    window: Option<&[f64]>,
) -> Render {
    render_stage_with_boundary_policy_grid_and_window(
        input,
        ratio,
        sample_rate,
        TraceStage::Complete,
        FrequencyBoundaryPolicy::Clamp,
        grid,
        window,
    )
}

pub(super) fn render_stage(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
) -> Render {
    render_stage_with_boundary_policy(
        input,
        ratio,
        sample_rate,
        trace_stage,
        FrequencyBoundaryPolicy::Clamp,
    )
}

pub(super) fn render_stage_with_grid(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
    grid: TransformGrid,
) -> Render {
    render_stage_with_boundary_policy_grid_and_window(
        input,
        ratio,
        sample_rate,
        trace_stage,
        FrequencyBoundaryPolicy::Clamp,
        grid,
        None,
    )
}

pub(super) fn render_stage_with_window(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
    window: &[f64],
) -> Render {
    render_stage_with_boundary_policy_grid_and_window(
        input,
        ratio,
        sample_rate,
        trace_stage,
        FrequencyBoundaryPolicy::Clamp,
        TransformGrid::Standard,
        Some(window),
    )
}

pub(super) fn render_stage_with_grid_and_window(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
    grid: TransformGrid,
    window: &[f64],
) -> Render {
    render_stage_with_boundary_policy_grid_and_window(
        input,
        ratio,
        sample_rate,
        trace_stage,
        FrequencyBoundaryPolicy::Clamp,
        grid,
        Some(window),
    )
}

pub(super) fn render_stage_with_boundary_policy(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
    boundary_policy: FrequencyBoundaryPolicy,
) -> Render {
    render_stage_with_boundary_policy_grid_and_window(
        input,
        ratio,
        sample_rate,
        trace_stage,
        boundary_policy,
        TransformGrid::Standard,
        None,
    )
}

fn render_stage_with_boundary_policy_grid_and_window(
    input: &[f64],
    ratio: f64,
    sample_rate: usize,
    trace_stage: TraceStage,
    boundary_policy: FrequencyBoundaryPolicy,
    grid: TransformGrid,
    window_override: Option<&[f64]>,
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
            horizontal_ratio_errors: Vec::new(),
            horizontal_output_phases: Vec::new(),
            stage_trace: None,
        };
    }
    let hop = ((sample_rate as f64 * 0.03).round() as usize).max(1);
    let length = 4 * hop;
    let transform_length = match grid {
        TransformGrid::Standard => length,
        TransformGrid::ModifiedHalfBin => modified_transform_length(length),
    };
    let bins = match grid {
        TransformGrid::Standard => transform_length / 2 + 1,
        TransformGrid::ModifiedHalfBin => transform_length / 2,
    };
    let long_distance = ((transform_length as f64 / hop as f64).round() as usize).max(1);
    let window = window_override.map_or_else(
        || {
            (0..length)
                .map(|index| {
                    (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos())
                        .sqrt()
                })
                .collect::<Vec<_>>()
        },
        |window| {
            assert_eq!(window.len(), length, "frozen window support");
            window.to_vec()
        },
    );
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(transform_length);
    let inverse = planner.plan_fft_inverse(transform_length);
    let mut output = vec![0.0; target_len];
    let mut normalization = vec![0.0; target_len];
    let mut previous_output = vec![Complex64::new(0.0, 0.0); bins];
    let mut previous_input_energy = vec![0.0_f64; bins];
    let mut previous_source_center: Option<isize> = None;
    let mut mechanisms = MechanismCounts::default();
    let mut maximum_normalization_phase_delta = 0.0_f64;
    let mut significant_fallback = 0;
    let mut horizontal_ratio_errors = Vec::new();
    let mut horizontal_output_phases = Vec::new();
    let mut stage_trace = None;
    let mut output_center = -(length as isize / 2);
    while output_center < target_len as isize + length as isize / 2 {
        let source_center = (output_center as f64 / ratio).round() as isize;
        let current = analyse(
            input,
            source_center,
            &window,
            transform_length,
            grid,
            &forward,
        );
        let auxiliary = analyse(
            input,
            source_center - hop as isize,
            &window,
            transform_length,
            grid,
            &forward,
        );
        let interior = source_center >= sample_rate as isize / 2
            && source_center < input.len() as isize - sample_rate as isize / 2;
        if interior {
            horizontal_ratio_errors.push(std::array::from_fn(|tone| {
                let frequency = CHORD_FREQUENCIES[tone];
                let bin = frequency_bin(frequency, sample_rate, transform_length, grid);
                let observed = (current[bin] * auxiliary[bin].conj()).arg();
                let expected = std::f64::consts::TAU * frequency * hop as f64 / sample_rate as f64;
                wrap(observed - expected)
            }));
        }
        let mut preliminary = current.clone();
        let mut traced = preliminary.clone();
        let mut next_horizontal_state = None;
        if let Some(previous_source_center) = previous_source_center {
            for bin in 0..bins {
                let prediction = previous_output[bin] * current[bin] * auxiliary[bin].conj();
                let current_energy = current[bin].norm_sqr();
                let denominator =
                    previous_input_energy[bin].max(current_energy) + HORIZONTAL_ENERGY_FLOOR;
                preliminary[bin] = prediction / denominator;
                mechanisms.horizontal += 1;
            }
            if interior {
                horizontal_output_phases.push(std::array::from_fn(|tone| {
                    let frequency = CHORD_FREQUENCIES[tone];
                    let bin = frequency_bin(frequency, sample_rate, transform_length, grid);
                    preliminary[bin].arg()
                }));
            }
            let input_hop = (source_center - previous_source_center)
                .unsigned_abs()
                .max(1);
            let time_factor = hop as f64 / input_hop as f64;
            let significant_energy =
                current.iter().map(Complex64::norm_sqr).fold(0.0, f64::max) * 1.0e-8;
            if trace_stage == TraceStage::HorizontalPhaseRecurrence {
                let mut horizontal_state = preliminary.clone();
                constrain_real_edges(&mut horizontal_state, grid);
                next_horizontal_state = Some(horizontal_state);
            }
            let mut corrected = preliminary.clone();
            for bin in 0..bins {
                let mut prediction = Complex64::new(0.0, 0.0);
                let mut selected = Complex64::new(0.0, 0.0);
                if bin >= 1 {
                    let lower_input =
                        interpolate(&current, bin as f64 - time_factor, boundary_policy);
                    let twist = current[bin] * lower_input.conj();
                    let candidate = corrected[bin - 1] * twist;
                    prediction += candidate;
                    if trace_stage == TraceStage::ShortLower {
                        selected = candidate;
                    }
                    mechanisms.short_lower += 1;
                }
                if bin + 1 < bins {
                    let lower_input =
                        interpolate(&current, bin as f64 + 1.0 - time_factor, boundary_policy);
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
                        &current,
                        bin as f64 - long_distance as f64 * time_factor,
                        boundary_policy,
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
                        &current,
                        bin as f64 + long_distance as f64 - long_distance as f64 * time_factor,
                        boundary_policy,
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
                    TraceStage::HorizontalPhaseRecurrence => {
                        normalize_or(preliminary[bin], current[bin], current[bin])
                    }
                    TraceStage::Complete => corrected[bin],
                    _ => normalize_or(selected, current[bin], current[bin]),
                };
            }
            if source_center == 8_400 {
                stage_trace = Some(StageFrameTrace {
                    source_center,
                    current: current.clone(),
                    preliminary: preliminary.clone(),
                    corrected: corrected.clone(),
                });
            }
            preliminary = corrected;
        }
        for bin in 0..bins {
            previous_input_energy[bin] = current[bin].norm_sqr();
        }
        constrain_real_edges(&mut preliminary, grid);
        constrain_real_edges(&mut traced, grid);
        let frame = synthesise(&traced, length, transform_length, grid, &inverse);
        for offset in 0..length {
            let output_index = output_center - length as isize / 2 + offset as isize;
            if (0..target_len as isize).contains(&output_index) {
                let output_index = output_index as usize;
                output[output_index] += frame[offset] * window[offset] / transform_length as f64;
                normalization[output_index] += window[offset] * window[offset];
            }
        }
        previous_output = next_horizontal_state.unwrap_or(preliminary);
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
        horizontal_ratio_errors,
        horizontal_output_phases,
        stage_trace,
    }
}

fn modified_transform_length(block_frames: usize) -> usize {
    let real_request = (block_frames + 1) / 2;
    let complex_request = (real_request + 1) / 2;
    split_fft_fast_size_above(complex_request) * 4
}

fn split_fft_fast_size_above(size: usize) -> usize {
    let mut power = 1;
    while power < 16 && power < size {
        power *= 2;
    }
    while power * 8 < size {
        power *= 2;
    }
    let mut multiple = size.div_ceil(power);
    if multiple == 7 {
        multiple += 1;
    }
    multiple * power
}

fn analyse(
    input: &[f64],
    center: isize,
    window: &[f64],
    transform_length: usize,
    grid: TransformGrid,
    forward: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<Complex64> {
    let support_length = window.len();
    let mut spectrum = vec![Complex64::new(0.0, 0.0); transform_length];
    for offset in 0..support_length {
        let relative = offset as isize - support_length as isize / 2;
        let value = reflected(input, center + relative) * window[offset];
        match grid {
            TransformGrid::Standard => {
                spectrum[offset] = Complex64::new(value, 0.0);
            }
            TransformGrid::ModifiedHalfBin => {
                let index = relative.rem_euclid(transform_length as isize) as usize;
                let phase = -std::f64::consts::PI * relative as f64 / transform_length as f64;
                spectrum[index] = Complex64::from_polar(value, phase);
            }
        }
    }
    forward.process(&mut spectrum);
    spectrum.truncate(match grid {
        TransformGrid::Standard => transform_length / 2 + 1,
        TransformGrid::ModifiedHalfBin => transform_length / 2,
    });
    spectrum
}

fn synthesise(
    bins: &[Complex64],
    support_length: usize,
    transform_length: usize,
    grid: TransformGrid,
    inverse: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<f64> {
    let mut spectrum = vec![Complex64::new(0.0, 0.0); transform_length];
    spectrum[..bins.len()].copy_from_slice(bins);
    match grid {
        TransformGrid::Standard => {
            for bin in 1..transform_length / 2 {
                spectrum[transform_length - bin] = spectrum[bin].conj();
            }
        }
        TransformGrid::ModifiedHalfBin => {
            for bin in 0..transform_length / 2 {
                spectrum[transform_length - 1 - bin] = spectrum[bin].conj();
            }
        }
    }
    inverse.process(&mut spectrum);
    (0..support_length)
        .map(|offset| match grid {
            TransformGrid::Standard => spectrum[offset].re,
            TransformGrid::ModifiedHalfBin => {
                let relative = offset as isize - support_length as isize / 2;
                let index = relative.rem_euclid(transform_length as isize) as usize;
                let phase = std::f64::consts::PI * relative as f64 / transform_length as f64;
                (spectrum[index] * Complex64::from_polar(1.0, phase)).re
            }
        })
        .collect()
}

fn frequency_bin(
    frequency: f64,
    sample_rate: usize,
    transform_length: usize,
    grid: TransformGrid,
) -> usize {
    let position = frequency * transform_length as f64 / sample_rate as f64
        - match grid {
            TransformGrid::Standard => 0.0,
            TransformGrid::ModifiedHalfBin => 0.5,
        };
    position.round().clamp(
        0.0,
        match grid {
            TransformGrid::Standard => transform_length / 2,
            TransformGrid::ModifiedHalfBin => transform_length / 2 - 1,
        } as f64,
    ) as usize
}

fn constrain_real_edges(spectrum: &mut [Complex64], grid: TransformGrid) {
    if grid == TransformGrid::Standard {
        spectrum[0].im = 0.0;
        if spectrum.len() > 1 {
            spectrum[spectrum.len() - 1].im = 0.0;
        }
    }
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

fn interpolate(
    spectrum: &[Complex64],
    position: f64,
    boundary_policy: FrequencyBoundaryPolicy,
) -> Complex64 {
    match boundary_policy {
        FrequencyBoundaryPolicy::Clamp => {
            let position = position.clamp(0.0, (spectrum.len() - 1) as f64);
            let lower = position.floor() as usize;
            let upper = (lower + 1).min(spectrum.len() - 1);
            let fraction = position - lower as f64;
            spectrum[lower] * (1.0 - fraction) + spectrum[upper] * fraction
        }
        FrequencyBoundaryPolicy::ZeroExtend => {
            let lower = position.floor() as isize;
            let fraction = position - lower as f64;
            let get = |index: isize| {
                usize::try_from(index)
                    .ok()
                    .and_then(|index| spectrum.get(index))
                    .copied()
                    .unwrap_or_default()
            };
            get(lower) * (1.0 - fraction) + get(lower + 1) * fraction
        }
    }
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

fn bass_review(grid: TransformGrid, window: Option<&[f64]>) -> (f64, usize, Render) {
    let note_frames = SAMPLE_RATE;
    let input = (0..note_frames * BASS_FREQUENCIES.len())
        .map(|index| {
            let frequency = BASS_FREQUENCIES[index / note_frames];
            (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin() * 0.5
        })
        .collect::<Vec<_>>();
    let ratio = 1.5;
    let render = render_with_representation(&input, ratio, SAMPLE_RATE, grid, window);
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

fn chord_review(grid: TransformGrid, window: Option<&[f64]>) -> (f64, f64, f64, Render) {
    let input = chord_control();
    let render = render_with_representation(&input, 2.0, SAMPLE_RATE, grid, window);
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
    spectrum_metrics(segment, &CHORD_FREQUENCIES)
}

pub(super) fn spectrum_metrics(segment: &[f64], frequencies: &[f64]) -> ChordSpectrumMetrics {
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
    for expected in frequencies.iter().copied() {
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
            frequencies
                .iter()
                .all(|frequency| (*bin as f64 * bin_hz - frequency).abs() > 8.0)
        })
        .map(|(_, value)| value.norm_sqr())
        .sum::<f64>();
    let strongest_sideband = spectrum[..=fft_len / 2]
        .iter()
        .enumerate()
        .filter(|(bin, _)| {
            frequencies
                .iter()
                .all(|frequency| (*bin as f64 * bin_hz - frequency).abs() > 8.0)
        })
        .max_by(|(_, left), (_, right)| left.norm_sqr().total_cmp(&right.norm_sqr()))
        .map(|(bin, _)| bin as f64 * bin_hz)
        .unwrap_or(0.0);
    let strongest_sideband_offset_hz = frequencies
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

fn transient_review(grid: TransformGrid, window: Option<&[f64]>) -> (usize, usize, Render) {
    let events = [SAMPLE_RATE / 3, SAMPLE_RATE, SAMPLE_RATE + SAMPLE_RATE / 6];
    let mut input = vec![0.0; SAMPLE_RATE * 2];
    for (event, amplitude) in events.into_iter().zip([1.0, 0.8, 0.7]) {
        input[event] = amplitude;
    }
    let ratio = 2.0;
    let render = render_with_representation(&input, ratio, SAMPLE_RATE, grid, window);
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
