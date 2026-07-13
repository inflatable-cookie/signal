mod overlap;
mod phase;
mod schedule;
mod successor;
mod trace;
mod tracking;

use rustfft::{num_complex::Complex64, FftPlanner};

use phase::{transport, PhaseState};
pub(super) use successor::{
    render_successor, render_successor_owned, render_successor_owned_traced,
    render_successor_traced,
};
use trace::hash_phase_trace;
pub(super) use trace::{PhaseFrameTrace, SampleTrace, SynthesisFrameTrace};

use super::super::study_local_schedule::{schedule::Schedule, BASE_HOP, SOURCE_FRAMES};
use super::super::HASH_OFFSET;

const FFT_FRAMES: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    Ordinary,
    Event,
    Vertical,
    Both,
    Successor,
    SuccessorOwned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LatticeFactor {
    EventWarped,
    GlobalLinear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PhaseFactor {
    Transport,
    AnalysisPassthrough,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OverlapFactor {
    DiagonalDual,
    AnalysisPartition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WindowFactor {
    RootHann,
    Hann,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FftGridFactor {
    Shared4096,
    Native,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FrameGeometryFactor {
    CenteredReflected,
    StartAlignedPadded,
}

impl OverlapFactor {
    fn operator_weight(self, analysis: f64, synthesis: f64) -> f64 {
        match self {
            Self::DiagonalDual => analysis * synthesis,
            Self::AnalysisPartition => analysis,
        }
    }

    fn synthesis_weight(self, synthesis: f64) -> f64 {
        match self {
            Self::DiagonalDual => synthesis,
            Self::AnalysisPartition => 1.0,
        }
    }
}

impl Mode {
    pub(super) fn event(self) -> bool {
        matches!(
            self,
            Self::Event | Self::Both | Self::Successor | Self::SuccessorOwned
        )
    }

    pub(super) fn vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }

    fn successor(self) -> bool {
        matches!(self, Self::Successor | Self::SuccessorOwned)
    }

    fn event_owned(self) -> bool {
        self == Self::SuccessorOwned
    }
}

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(super) source: isize,
    pub(super) output: isize,
    pub(super) length: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Render {
    pub(super) samples: Vec<Vec<f64>>,
    pub(super) target_len: usize,
    pub(super) frame_count: usize,
    pub(super) resolution_changes: usize,
    pub(super) phase_initializations: usize,
    pub(super) uncovered: usize,
    pub(super) covered: usize,
    pub(super) frame_values: [f64; 3],
    pub(super) boundary_failures: usize,
    pub(super) event_order_failures: usize,
    pub(super) symmetry_error: f64,
    pub(super) imaginary_residue: f64,
    pub(super) non_finite: usize,
    pub(super) event_phase_changes: usize,
    pub(super) vertical_phase_changes: usize,
    pub(super) event_owned_samples: usize,
    pub(super) event_ownership_hash: u64,
    pub(super) schedule_hash: u64,
    pub(super) frame_hash: u64,
    pub(super) dual_hash: u64,
    pub(super) coefficient_hash: u64,
    pub(super) magnitude_hash: u64,
    pub(super) phase_hash: u64,
    pub(super) decision_hash: u64,
    pub(super) output_hash: u64,
    pub(super) phase_trace: Vec<PhaseFrameTrace>,
    pub(super) synthesis_trace: Vec<SynthesisFrameTrace>,
    pub(super) trace_hashes: [u64; 2],
}

pub(super) fn render(
    channels: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &Schedule,
    mode: Mode,
) -> Render {
    render_frames(
        channels,
        ratio,
        points,
        points,
        &[],
        schedule,
        mode,
        schedule::legacy(ratio, points, schedule),
    )
}

pub(super) fn render_ordinary_traced(
    channels: &[Vec<f64>],
    ratio: f64,
    resolution_points: &[usize],
    trace_events: &[usize],
    schedule: &Schedule,
) -> Render {
    render_frames(
        channels,
        ratio,
        resolution_points,
        trace_events,
        &[],
        schedule,
        Mode::Ordinary,
        schedule::legacy(ratio, resolution_points, schedule),
    )
}

pub(super) fn render_ordinary_fixed(
    channels: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &Schedule,
    length: usize,
) -> Render {
    render_frames(
        channels,
        ratio,
        points,
        points,
        &[],
        schedule,
        Mode::Ordinary,
        schedule::fixed(ratio, length, schedule),
    )
}

pub(super) fn render_ordinary_factor(
    channels: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &Schedule,
    lattice: LatticeFactor,
    phase: PhaseFactor,
    overlap: OverlapFactor,
) -> Render {
    let frames = match lattice {
        LatticeFactor::EventWarped => schedule::fixed(ratio, FFT_FRAMES, schedule),
        LatticeFactor::GlobalLinear => schedule::fixed_linear(ratio, FFT_FRAMES),
    };
    render_frames_factored(
        channels,
        ratio,
        points,
        points,
        &[],
        schedule,
        Mode::Ordinary,
        frames,
        phase,
        overlap,
        WindowFactor::RootHann,
        WindowFactor::RootHann,
        FftGridFactor::Shared4096,
        FrameGeometryFactor::CenteredReflected,
    )
}

pub(super) fn render_ordinary_window_factor(
    channels: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &Schedule,
    analysis: WindowFactor,
    synthesis: WindowFactor,
) -> Render {
    render_frames_factored(
        channels,
        ratio,
        points,
        points,
        &[],
        schedule,
        Mode::Ordinary,
        schedule::fixed(ratio, FFT_FRAMES, schedule),
        PhaseFactor::Transport,
        OverlapFactor::DiagonalDual,
        analysis,
        synthesis,
        FftGridFactor::Shared4096,
        FrameGeometryFactor::CenteredReflected,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_ordinary_geometry_factor(
    channels: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &Schedule,
    length: usize,
    fft_grid: FftGridFactor,
    frame_geometry: FrameGeometryFactor,
) -> Render {
    render_frames_factored(
        channels,
        ratio,
        points,
        points,
        &[],
        schedule,
        Mode::Ordinary,
        schedule::fixed(ratio, length, schedule),
        PhaseFactor::Transport,
        OverlapFactor::DiagonalDual,
        WindowFactor::Hann,
        WindowFactor::Hann,
        fft_grid,
        frame_geometry,
    )
}

fn render_frames(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    trace_events: &[usize],
    trace_outputs: &[isize],
    schedule: &Schedule,
    mode: Mode,
    frames: Vec<Frame>,
) -> Render {
    render_frames_factored(
        channels,
        ratio,
        events,
        trace_events,
        trace_outputs,
        schedule,
        mode,
        frames,
        PhaseFactor::Transport,
        OverlapFactor::DiagonalDual,
        WindowFactor::RootHann,
        WindowFactor::RootHann,
        FftGridFactor::Shared4096,
        FrameGeometryFactor::CenteredReflected,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_frames_factored(
    channels: &[Vec<f64>],
    ratio: f64,
    events: &[usize],
    trace_events: &[usize],
    trace_outputs: &[isize],
    schedule: &Schedule,
    mode: Mode,
    frames: Vec<Frame>,
    phase: PhaseFactor,
    overlap: OverlapFactor,
    analysis_window: WindowFactor,
    synthesis_window: WindowFactor,
    fft_grid: FftGridFactor,
    frame_geometry: FrameGeometryFactor,
) -> Render {
    let frame_length = frames
        .iter()
        .map(|frame| frame.length)
        .max()
        .expect("adaptive frames");
    let fft_frames = match fft_grid {
        FftGridFactor::Shared4096 => FFT_FRAMES,
        FftGridFactor::Native => frame_length,
    };
    assert!(frame_length <= fft_frames, "frame fits FFT grid");
    assert!(
        frame_geometry == FrameGeometryFactor::CenteredReflected
            || fft_grid == FftGridFactor::Native,
        "start-aligned padding requires a native FFT grid"
    );
    let target_len = (ratio * SOURCE_FRAMES as f64).round() as usize;
    let output_start = frames
        .iter()
        .map(|frame| frame.output - frame.length as isize / 2)
        .min()
        .expect("adaptive frames");
    let output_end = frames
        .iter()
        .map(|frame| frame.output + frame.length as isize / 2)
        .max()
        .expect("adaptive frames");
    let domain_len = (output_end - output_start) as usize;
    let mut operator = vec![0.0_f64; domain_len];
    for frame in &frames {
        let analysis = window(frame.length, analysis_window);
        let synthesis = window(frame.length, synthesis_window);
        for (offset, (analysis, synthesis)) in analysis.into_iter().zip(synthesis).enumerate() {
            let logical = frame.output - frame.length as isize / 2 + offset as isize;
            operator[(logical - output_start) as usize] +=
                overlap.operator_weight(analysis, synthesis);
        }
    }
    let crop = (-output_start) as usize;
    let crop_operator = &operator[crop..crop + target_len];
    let uncovered = crop_operator.iter().filter(|value| **value <= 0.0).count();
    let covered = crop_operator.len() - uncovered;
    let frame_min = crop_operator.iter().copied().fold(f64::INFINITY, f64::min);
    let frame_max = crop_operator.iter().copied().fold(0.0_f64, f64::max);
    let frame_values = [frame_min, frame_max, frame_max / frame_min];
    let dual_hash = schedule::dual_hash(&frames, &operator, output_start);
    let mut outputs = vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()];
    let mut states = channels
        .iter()
        .map(|_| PhaseState::new(fft_frames))
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(fft_frames);
    let inverse = planner.plan_fft_inverse(fft_frames);
    let tracking_channels = mode
        .successor()
        .then(|| tracking::analytic_channels(channels, &mut planner));
    let events = events
        .iter()
        .copied()
        .filter(|point| *point > 0 && *point < SOURCE_FRAMES)
        .collect::<Vec<_>>();
    let mut sample_targets = trace_events
        .iter()
        .map(|source| super::anchors::projected(schedule, *source))
        .collect::<Vec<_>>();
    sample_targets.extend_from_slice(trace_outputs);
    sample_targets.sort_unstable();
    sample_targets.dedup();
    let mut coefficient_hash = HASH_OFFSET;
    let mut magnitude_hash = HASH_OFFSET;
    let mut phase_hash = HASH_OFFSET;
    let mut decision_hash = HASH_OFFSET;
    let mut symmetry_error = 0.0_f64;
    let mut imaginary_residue = 0.0_f64;
    let mut non_finite = 0;
    let mut event_phase_changes = 0;
    let mut vertical_phase_changes = 0;
    let mut event_owned_samples = 0;
    let mut event_ownership_hash = HASH_OFFSET;
    let mut phase_initializations = 0;
    let mut phase_trace = Vec::with_capacity(frames.len());
    let mut synthesis_trace = Vec::with_capacity(frames.len());
    let mut phase_trace_hash = HASH_OFFSET;
    let mut synthesis_trace_hash = HASH_OFFSET;
    for frame in &frames {
        let analysis_weights = window(frame.length, analysis_window);
        let synthesis_weights = window(frame.length, synthesis_window);
        let buffer_offset = (fft_frames - frame.length) / 2;
        let overlap_ownership = (mode.event_owned() && ratio != 1.0)
            .then(|| overlap::Ownership::for_frame(frame, &events, schedule))
            .flatten();
        let mut spectra = Vec::with_capacity(channels.len());
        let mut linked = vec![0.0_f64; fft_frames / 2 + 1];
        let mut tracking_spectra = Vec::with_capacity(channels.len());
        for (channel_index, channel) in channels.iter().enumerate() {
            let mut spectrum = vec![Complex64::new(0.0, 0.0); fft_frames];
            for (offset, weight) in analysis_weights.iter().copied().enumerate() {
                let logical = frame.source - frame.length as isize / 2 + offset as isize;
                let original = match frame_geometry {
                    FrameGeometryFactor::CenteredReflected => reflected(channel, logical),
                    FrameGeometryFactor::StartAlignedPadded => channel
                        .get(usize::try_from(logical).unwrap_or(usize::MAX))
                        .copied()
                        .unwrap_or(0.0),
                };
                let sample = overlap_ownership
                    .as_ref()
                    .and_then(|ownership| ownership.sample(channel, logical))
                    .unwrap_or(original);
                if channel_index == 0 && sample.to_bits() != original.to_bits() {
                    event_owned_samples += 1;
                    hash(&mut event_ownership_hash, frame.source as i64 as u64);
                    hash(&mut event_ownership_hash, logical as i64 as u64);
                    hash(&mut event_ownership_hash, original.to_bits());
                    hash(&mut event_ownership_hash, sample.to_bits());
                }
                spectrum[buffer_offset + offset].re = sample * weight;
            }
            forward.process(&mut spectrum);
            for (bin, value) in spectrum.iter().take(fft_frames / 2 + 1).enumerate() {
                linked[bin] += value.norm_sqr();
                hash(&mut magnitude_hash, value.norm().to_bits());
            }
            for value in &spectrum {
                hash(&mut coefficient_hash, value.re.to_bits());
                hash(&mut coefficient_hash, value.im.to_bits());
            }
            spectra.push(spectrum);
            if mode.successor() {
                let tracking_weights = window(fft_frames, analysis_window);
                tracking_spectra.push(tracking::spectrum(
                    &tracking_channels.as_ref().expect("analytic channels")[channel_index],
                    frame.source,
                    &tracking_weights,
                    &forward,
                ));
            }
        }
        let tracking_linked = if mode.successor() {
            tracking::linked(&tracking_spectra)
        } else {
            linked.clone()
        };
        let peaks = if mode.successor() {
            tracking::active_peaks(&tracking_linked)
        } else {
            tracking::legacy_peaks(&linked)
        };
        hash(&mut decision_hash, frame.source as i64 as u64);
        hash(&mut decision_hash, frame.length as u64);
        for peak in &peaks {
            hash(&mut decision_hash, *peak as u64);
        }
        let trace_bin = tracking_linked
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(bin, _)| bin)
            .unwrap_or(0);
        for (channel_index, spectrum) in spectra.iter_mut().enumerate() {
            let result = transport(
                spectrum,
                frame,
                &mut states[channel_index],
                &events,
                &peaks,
                mode,
                trace_bin,
                tracking_spectra.get(channel_index).map(Vec::as_slice),
                phase == PhaseFactor::AnalysisPassthrough,
            );
            event_phase_changes += result.event_changes;
            vertical_phase_changes += result.vertical_changes;
            phase_initializations += result.initialization;
            if channel_index == 0 {
                let trace = PhaseFrameTrace {
                    source: frame.source,
                    output: frame.output,
                    length: frame.length,
                    phase: result.trace,
                };
                hash_phase_trace(&mut phase_trace_hash, &trace);
                phase_trace.push(trace);
            }
            mirror(spectrum);
            for bin in 0..fft_frames {
                let mirror_bin = if bin == 0 { 0 } else { fft_frames - bin };
                symmetry_error =
                    symmetry_error.max((spectrum[bin] - spectrum[mirror_bin].conj()).norm());
                non_finite +=
                    usize::from(!spectrum[bin].re.is_finite() || !spectrum[bin].im.is_finite());
                hash(&mut phase_hash, spectrum[bin].arg().to_bits());
            }
            inverse.process(spectrum);
            let mut contribution_energy = 0.0_f64;
            let mut contribution_moment = 0.0_f64;
            let mut contribution_peak = 0.0_f64;
            let mut contribution_peak_output = frame.output;
            let mut traced_samples = Vec::new();
            let mut contribution_hash = HASH_OFFSET;
            for (offset, weight) in synthesis_weights.iter().copied().enumerate() {
                if weight == 0.0 {
                    continue;
                }
                let logical = frame.output - frame.length as isize / 2 + offset as isize;
                let domain = (logical - output_start) as usize;
                let value = spectrum[buffer_offset + offset]
                    * (overlap.synthesis_weight(weight) / (fft_frames as f64 * operator[domain]));
                imaginary_residue = imaginary_residue.max(value.im.abs());
                outputs[channel_index][domain] += value;
                if channel_index == 0 {
                    for target in &sample_targets {
                        if logical == *target {
                            traced_samples.push(SampleTrace {
                                output: *target,
                                dual_weight: weight / (fft_frames as f64 * operator[domain]),
                                value: [value.re, value.im],
                            });
                        }
                    }
                    let square = value.re * value.re;
                    contribution_energy += square;
                    contribution_moment += logical as f64 * square;
                    if value.re.abs() > contribution_peak {
                        contribution_peak = value.re.abs();
                        contribution_peak_output = logical;
                    }
                    hash(&mut contribution_hash, logical as i64 as u64);
                    hash(&mut contribution_hash, value.re.to_bits());
                }
            }
            if channel_index == 0 {
                let trace = SynthesisFrameTrace {
                    source: frame.source,
                    output: frame.output,
                    length: frame.length,
                    energy: contribution_energy,
                    energy_center: if contribution_energy > 0.0 {
                        contribution_moment / contribution_energy
                    } else {
                        frame.output as f64
                    },
                    peak_output: contribution_peak_output,
                    peak_magnitude: contribution_peak,
                    traced_samples,
                    hash: contribution_hash,
                };
                hash(&mut synthesis_trace_hash, trace.hash);
                synthesis_trace.push(trace);
            }
        }
    }
    let samples = outputs
        .iter()
        .map(|channel| {
            channel[crop..crop + target_len]
                .iter()
                .map(|value| value.re)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
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
        .map(|event| schedule.positions[*event / BASE_HOP])
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
        target_len,
        frame_count: frames.len(),
        resolution_changes: frames
            .windows(2)
            .filter(|pair| pair[0].length != pair[1].length)
            .count(),
        phase_initializations,
        uncovered,
        covered,
        frame_values,
        boundary_failures,
        event_order_failures,
        symmetry_error,
        imaginary_residue,
        non_finite,
        event_phase_changes,
        vertical_phase_changes,
        event_owned_samples,
        event_ownership_hash,
        schedule_hash: schedule.hash,
        frame_hash: schedule::frame_hash(&frames),
        dual_hash,
        coefficient_hash,
        magnitude_hash,
        phase_hash,
        decision_hash,
        output_hash,
        phase_trace,
        synthesis_trace,
        trace_hashes: [phase_trace_hash, synthesis_trace_hash],
    }
}

fn window(length: usize, factor: WindowFactor) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let hann = 0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos();
            match factor {
                WindowFactor::RootHann => hann.sqrt(),
                WindowFactor::Hann => hann,
            }
        })
        .collect()
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

fn mirror(spectrum: &mut [Complex64]) {
    let fft_frames = spectrum.len();
    spectrum[0].im = 0.0;
    spectrum[fft_frames / 2].im = 0.0;
    for bin in 1..fft_frames / 2 {
        spectrum[fft_frames - bin] = spectrum[bin].conj();
    }
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
