use rustfft::num_complex::Complex64;

use super::super::super::super::complete_system_tuning::{Configuration, ResetScope};
use super::super::Frame;
use super::{wrap, PhaseState};

pub(super) struct SharedState {
    synthesis: Vec<f64>,
    frequency: Vec<f64>,
    evidence: Vec<f64>,
    initialized: Vec<bool>,
    source: Option<isize>,
    output: Option<isize>,
}

impl SharedState {
    pub(super) fn new(length: usize) -> Self {
        let bins = length / 2 + 1;
        Self {
            synthesis: vec![0.0; bins],
            frequency: vec![0.0; bins],
            evidence: vec![0.0; bins],
            initialized: vec![false; bins],
            source: None,
            output: None,
        }
    }

    fn advance(&mut self, frame: &Frame) {
        if self.source == Some(frame.source) {
            return;
        }
        let output_hop = self
            .output
            .map(|previous| (frame.output - previous) as f64)
            .unwrap_or(0.0);
        for bin in 0..self.synthesis.len() {
            if self.initialized[bin] {
                self.synthesis[bin] += self.frequency[bin] * output_hop;
            }
            self.evidence[bin] = 0.0;
        }
        self.source = Some(frame.source);
        self.output = Some(frame.output);
    }
}

pub(super) fn transport(
    spectrum: &mut [Complex64],
    frame: &Frame,
    state: &mut PhaseState,
    events: &[usize],
    dominant: usize,
    configuration: Configuration,
    identity: bool,
) -> (usize, usize) {
    let layer = frame.layer;
    let length = configuration.geometry[layer];
    let first = state.source[layer].is_none();
    let source_hop = state.source[layer]
        .map(|previous| (frame.source - previous) as f64)
        .unwrap_or(0.0);
    for bin in 0..=length / 2 {
        let analysis = spectrum[bin].arg();
        let frequency = if first {
            std::f64::consts::TAU * bin as f64 / length as f64
        } else {
            let expected = std::f64::consts::TAU * bin as f64 * source_hop / length as f64;
            let residual = wrap(analysis - state.analysis[layer][bin] - expected);
            (expected + residual) / source_hop
        };
        state.analysis[layer][bin] = analysis;
        state.frequency[layer][bin] = frequency;
    }

    if identity {
        state.source[layer] = Some(frame.source);
        state.output[layer] = Some(frame.output);
        return (0, 0);
    }

    state.shared.advance(frame);
    let common_length = (state.shared.synthesis.len() - 1) * 2;
    for bin in 0..=length / 2 {
        let common = bin * common_length / length;
        let weight = spectrum[bin].norm_sqr();
        if !state.shared.initialized[common] {
            state.shared.synthesis[common] = state.analysis[layer][bin];
            state.shared.frequency[common] = state.frequency[layer][bin];
            state.shared.initialized[common] = true;
        } else if weight > 0.0 {
            let previous = state.shared.evidence[common];
            state.shared.frequency[common] = (state.shared.frequency[common] * previous
                + state.frequency[layer][bin] * weight)
                / (previous + weight);
        }
        state.shared.evidence[common] += weight;
    }

    let event = events.iter().any(|point| {
        frame.source >= 0 && frame.source.abs_diff(*point as isize) <= configuration.geometry[0] / 8
    });
    let resets = if event {
        reset(state, layer, length, dominant, configuration.reset_scope)
    } else {
        0
    };
    for bin in 0..=length / 2 {
        let common = bin * common_length / length;
        let magnitude = spectrum[bin].norm();
        spectrum[bin] = Complex64::from_polar(magnitude, state.shared.synthesis[common]);
    }
    spectrum[0].im = 0.0;
    spectrum[length / 2].im = 0.0;
    state.source[layer] = Some(frame.source);
    state.output[layer] = Some(frame.output);
    (resets, length / 2 + 1)
}

fn reset(
    state: &mut PhaseState,
    layer: usize,
    length: usize,
    dominant: usize,
    scope: ResetScope,
) -> usize {
    let common_length = (state.shared.synthesis.len() - 1) * 2;
    match scope {
        ResetScope::ShortOnly if layer == 0 => {
            for bin in 0..=length / 2 {
                let common = bin * common_length / length;
                state.shared.synthesis[common] = state.analysis[layer][bin];
            }
            length / 2 + 1
        }
        ResetScope::ConfidenceOwned if layer == 2 => {
            let common = dominant * common_length / length;
            state.shared.synthesis[common] = state.analysis[layer][dominant];
            1
        }
        ResetScope::FrequencyLimited if layer == 2 => {
            let frequency = dominant as f64 * 48_000.0 / length as f64;
            if !(80.0..=2_000.0).contains(&frequency) {
                let common = dominant * common_length / length;
                state.shared.synthesis[common] = state.analysis[layer][dominant];
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}
