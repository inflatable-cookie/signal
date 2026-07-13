use rustfft::num_complex::Complex64;

use super::{Frame, Mode, FFT_FRAMES};

pub(super) struct PhaseState {
    analysis: Vec<f64>,
    synthesis: Vec<f64>,
    source: Option<isize>,
    output: Option<isize>,
}

impl PhaseState {
    pub(super) fn new() -> Self {
        Self {
            analysis: vec![0.0; FFT_FRAMES / 2 + 1],
            synthesis: vec![0.0; FFT_FRAMES / 2 + 1],
            source: None,
            output: None,
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
) -> (usize, usize, usize) {
    let first = state.source.is_none();
    let source_hop = state
        .source
        .map(|previous| (frame.source - previous) as f64)
        .unwrap_or(0.0);
    let output_hop = state
        .output
        .map(|previous| (frame.output - previous) as f64)
        .unwrap_or(0.0);
    for bin in 0..=FFT_FRAMES / 2 {
        let analysis = spectrum[bin].arg();
        let synthesis = if first {
            analysis
        } else {
            let omega = std::f64::consts::TAU * bin as f64 / FFT_FRAMES as f64;
            let residual = wrap(analysis - state.analysis[bin] - omega * source_hop);
            let frequency = omega + residual / source_hop;
            state.synthesis[bin] + frequency * output_hop
        };
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
    for bin in 0..=FFT_FRAMES / 2 {
        spectrum[bin] = Complex64::from_polar(spectrum[bin].norm(), state.synthesis[bin]);
    }
    state.source = Some(frame.source);
    state.output = Some(frame.output);
    (event_changes, vertical_changes, usize::from(first))
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
