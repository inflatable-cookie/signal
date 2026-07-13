use rustfft::num_complex::Complex64;

use super::{Frame, Mode, FFT_FRAMES};

pub(super) struct PhaseState {
    analysis: Vec<f64>,
    synthesis: Vec<f64>,
    source: Option<isize>,
    output: Option<isize>,
    dominant: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct Trace {
    pub(in crate::frequency_adaptive) source_hop: f64,
    pub(in crate::frequency_adaptive) output_hop: f64,
    pub(in crate::frequency_adaptive) bin: usize,
    pub(in crate::frequency_adaptive) prior_bin: usize,
    pub(in crate::frequency_adaptive) peak_owner: usize,
    pub(in crate::frequency_adaptive) analysis_advance: f64,
    pub(in crate::frequency_adaptive) estimated_frequency: f64,
    pub(in crate::frequency_adaptive) transported_advance: f64,
    pub(in crate::frequency_adaptive) final_advance: f64,
    pub(in crate::frequency_adaptive) event_assignment: bool,
    pub(in crate::frequency_adaptive) vertical_assignment: bool,
}

pub(super) struct Result {
    pub(super) event_changes: usize,
    pub(super) vertical_changes: usize,
    pub(super) initialization: usize,
    pub(super) trace: Trace,
}

impl PhaseState {
    pub(super) fn new() -> Self {
        Self {
            analysis: vec![0.0; FFT_FRAMES / 2 + 1],
            synthesis: vec![0.0; FFT_FRAMES / 2 + 1],
            source: None,
            output: None,
            dominant: None,
        }
    }
}

pub(super) fn transport(
    spectrum: &mut [Complex64],
    frame: &Frame,
    state: &mut PhaseState,
    events: &[usize],
    peaks: &[usize],
    mode: Mode,
    trace_bin: usize,
) -> Result {
    let first = state.source.is_none();
    let source_hop = state
        .source
        .map(|previous| (frame.source - previous) as f64)
        .unwrap_or(0.0);
    let output_hop = state
        .output
        .map(|previous| (frame.output - previous) as f64)
        .unwrap_or(0.0);
    let prior_analysis = state.analysis[trace_bin];
    let prior_synthesis = state.synthesis[trace_bin];
    let prior_bin = state.dominant.unwrap_or(trace_bin);
    let mut trace_frequency = 0.0;
    let mut transported_phase = 0.0;
    for bin in 0..=FFT_FRAMES / 2 {
        let analysis = spectrum[bin].arg();
        let (synthesis, frequency) = if first {
            (
                analysis,
                std::f64::consts::TAU * bin as f64 / FFT_FRAMES as f64,
            )
        } else {
            let omega = std::f64::consts::TAU * bin as f64 / FFT_FRAMES as f64;
            let residual = wrap(analysis - state.analysis[bin] - omega * source_hop);
            let frequency = omega + residual / source_hop;
            (state.synthesis[bin] + frequency * output_hop, frequency)
        };
        if bin == trace_bin {
            trace_frequency = frequency;
            transported_phase = synthesis;
        }
        state.analysis[bin] = analysis;
        state.synthesis[bin] = synthesis;
    }
    let event = frame.source >= 0 && events.contains(&(frame.source as usize));
    let event_changes = if mode.event() && event {
        let mut changes = 0;
        for bin in 0..=FFT_FRAMES / 2 {
            changes +=
                usize::from(wrap(state.synthesis[bin] - state.analysis[bin]).abs() > 1.0e-12);
            state.synthesis[bin] = state.analysis[bin];
        }
        changes
    } else {
        0
    };
    let vertical_changes = if mode.vertical() {
        lock_to_peaks(&state.analysis, &mut state.synthesis, peaks)
    } else {
        0
    };
    let final_phase = state.synthesis[trace_bin];
    for bin in 0..=FFT_FRAMES / 2 {
        spectrum[bin] = Complex64::from_polar(spectrum[bin].norm(), state.synthesis[bin]);
    }
    state.source = Some(frame.source);
    state.output = Some(frame.output);
    state.dominant = Some(trace_bin);
    Result {
        event_changes,
        vertical_changes,
        initialization: usize::from(first),
        trace: Trace {
            source_hop,
            output_hop,
            bin: trace_bin,
            prior_bin,
            peak_owner: peak_owner(trace_bin, peaks),
            analysis_advance: if first {
                0.0
            } else {
                wrap(state.analysis[trace_bin] - prior_analysis)
            },
            estimated_frequency: trace_frequency,
            transported_advance: if first {
                0.0
            } else {
                wrap(transported_phase - prior_synthesis)
            },
            final_advance: if first {
                0.0
            } else {
                wrap(final_phase - prior_synthesis)
            },
            event_assignment: mode.event() && event,
            vertical_assignment: mode.vertical()
                && wrap(final_phase - transported_phase).abs() > 1.0e-12,
        },
    }
}

fn peak_owner(bin: usize, peaks: &[usize]) -> usize {
    peaks
        .iter()
        .copied()
        .min_by_key(|peak| peak.abs_diff(bin))
        .unwrap_or(bin)
}

fn lock_to_peaks(analysis: &[f64], synthesis: &mut [f64], peaks: &[usize]) -> usize {
    let mut changes = 0;
    for (index, peak) in peaks.iter().copied().enumerate() {
        let left = index
            .checked_sub(1)
            .map(|prior| (peaks[prior] + peak) / 2 + 1)
            .unwrap_or(0);
        let right = peaks
            .get(index + 1)
            .map(|next| (peak + *next) / 2 + 1)
            .unwrap_or(synthesis.len());
        let peak_phase = synthesis[peak];
        for bin in left..right {
            let locked = peak_phase + wrap(analysis[bin] - analysis[peak]);
            changes += usize::from(wrap(locked - synthesis[bin]).abs() > 1.0e-12);
            synthesis[bin] = locked;
        }
    }
    changes
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
