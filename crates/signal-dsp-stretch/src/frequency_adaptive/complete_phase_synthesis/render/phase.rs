use rustfft::num_complex::Complex64;

use super::{Frame, Mode, LAYERS};

pub(super) struct PhaseState {
    analysis: [Vec<f64>; 3],
    synthesis: [Vec<f64>; 3],
    frequency: [Vec<f64>; 3],
    source: [Option<isize>; 3],
    output: [Option<isize>; 3],
    reference: Option<(f64, f64, f64, isize, isize)>,
}

impl PhaseState {
    pub(super) fn new() -> Self {
        Self {
            analysis: std::array::from_fn(|layer| vec![0.0; LAYERS[layer] / 2 + 1]),
            synthesis: std::array::from_fn(|layer| vec![0.0; LAYERS[layer] / 2 + 1]),
            frequency: std::array::from_fn(|layer| vec![0.0; LAYERS[layer] / 2 + 1]),
            source: [None; 3],
            output: [None; 3],
            reference: None,
        }
    }
}

pub(super) fn transport(
    spectrum: &mut [Complex64],
    frame: &Frame,
    state: &mut PhaseState,
    events: &[usize],
    dominant: usize,
    mode: Mode,
) -> (usize, usize) {
    let layer = frame.layer;
    let length = LAYERS[layer];
    let first = state.source[layer].is_none();
    let source_hop = state.source[layer]
        .map(|previous| (frame.source - previous) as f64)
        .unwrap_or(0.0);
    let output_hop = state.output[layer]
        .map(|previous| (frame.output - previous) as f64)
        .unwrap_or(0.0);
    for bin in 0..=length / 2 {
        let analysis = spectrum[bin].arg();
        let (synthesis, frequency) = if first {
            (analysis, std::f64::consts::TAU * bin as f64 / length as f64)
        } else {
            let expected = std::f64::consts::TAU * bin as f64 * source_hop / length as f64;
            let residual = wrap(analysis - state.analysis[layer][bin] - expected);
            let frequency = (expected + residual) / source_hop;
            (
                state.synthesis[layer][bin] + frequency * output_hop,
                frequency,
            )
        };
        state.analysis[layer][bin] = analysis;
        state.synthesis[layer][bin] = synthesis;
        state.frequency[layer][bin] = frequency;
    }
    let event = events
        .iter()
        .any(|point| frame.source >= 0 && frame.source.abs_diff(*point as isize) <= 64);
    let event_resets = if mode.event() && layer == 0 && event {
        state.synthesis[layer].copy_from_slice(&state.analysis[layer]);
        length / 2 + 1
    } else {
        0
    };
    let mut vertical_alignments = 0;
    if mode.vertical() {
        if layer == 2 {
            state.reference = Some((
                state.analysis[layer][dominant],
                state.synthesis[layer][dominant],
                state.frequency[layer][dominant],
                frame.source,
                frame.output,
            ));
        } else if let Some((analysis, synthesis, frequency, source, output)) = state.reference {
            let projected_analysis = analysis + frequency * (frame.source - source) as f64;
            let projected_synthesis = synthesis + frequency * (frame.output - output) as f64;
            state.synthesis[layer][dominant] =
                projected_synthesis + wrap(state.analysis[layer][dominant] - projected_analysis);
            vertical_alignments = 1;
        }
    }
    for bin in 0..=length / 2 {
        let magnitude = spectrum[bin].norm();
        spectrum[bin] = Complex64::from_polar(magnitude, state.synthesis[layer][bin]);
    }
    spectrum[0].im = 0.0;
    spectrum[length / 2].im = 0.0;
    state.source[layer] = Some(frame.source);
    state.output[layer] = Some(frame.output);
    (event_resets, vertical_alignments)
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
