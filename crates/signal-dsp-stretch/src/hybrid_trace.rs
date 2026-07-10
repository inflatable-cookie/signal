use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

use crate::{
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP, COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
    DEFAULT_WINDOW_SIZE,
};

mod render;
mod transition;

#[cfg(test)]
use transition::nearest_owner;
use transition::schedule_transitions;

pub(crate) use render::build_hybrid_render;
pub use render::{
    StretchHybridRender, StretchHybridTransitionDecision, StretchHybridTransitionRejection,
};

const TRANSIENT_FLUX_RATIO: f64 = 0.30;
const TRANSIENT_ENERGY_RATIO: f64 = 1.20;
const TONAL_STABILITY: f64 = 0.70;
const TONAL_ENTRY_FRAMES: usize = 4;
const TRANSIENT_PREROLL_FRAMES: usize = 1;
const TRANSIENT_POSTROLL_FRAMES: usize = 3;
const TRANSITION_FRAMES: usize = 256;

/// Local synthesis owner selected for one structural-hybrid analysis frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchHybridOwner {
    /// Short-window independent-bin ownership around detected attacks.
    Transient,
    /// Current-window fallback ownership for uncertain and boundary regions.
    Mixed,
    /// Long-window identity-locked ownership for stable expansion regions.
    Tonal,
}

/// Report-only structural-hybrid classification for one source analysis frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchHybridFrameTrace {
    /// Source-domain centre of the short analysis frame.
    pub source_frame: usize,
    /// Ratio-projected output-domain frame.
    pub output_frame: usize,
    /// Positive spectral flux divided by current spectral magnitude.
    pub spectral_flux_ratio: f64,
    /// Windowed energy divided by previous-frame windowed energy.
    pub energy_ratio: f64,
    /// Magnitude stability against the preceding short analysis frame.
    pub spectral_stability: f64,
    /// Frozen local owner for this frame.
    pub owner: StretchHybridOwner,
}

/// Report-only scheduled ownership transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchHybridTransitionTrace {
    /// Owner leaving the output timeline.
    pub from: StretchHybridOwner,
    /// Owner entering the output timeline.
    pub to: StretchHybridOwner,
    /// Ratio-projected transition frame before low-energy search.
    pub requested_output_frame: usize,
    /// Selected low-energy transition frame on the current output.
    pub scheduled_output_frame: usize,
    /// Signed scheduled-minus-requested output-frame offset.
    pub search_offset_frames: i64,
    /// Frozen crossfade span for the later audio candidate.
    pub crossfade_frames: usize,
}

/// Deterministic report-only trace for the first structural-hybrid candidate.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHybridTrace {
    /// Sanitized fixed output/input duration ratio.
    pub ratio: f64,
    /// Input sample-frame count.
    pub input_frames: usize,
    /// Current-path output sample-frame count.
    pub output_frames: usize,
    /// Per-frame local classification trace.
    pub frames: Vec<StretchHybridFrameTrace>,
    /// Scheduled ownership transitions. No branch audio is mixed by this trace.
    pub transitions: Vec<StretchHybridTransitionTrace>,
}

#[derive(Clone, Copy, Debug)]
struct HybridFrameFeatures {
    source_frame: usize,
    output_frame: usize,
    spectral_flux_ratio: f64,
    energy_ratio: f64,
    spectral_stability: f64,
    onset: bool,
}

pub(crate) fn build_hybrid_trace(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> StretchHybridTrace {
    let ratio = if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    };
    let features = analyze_short_frames(input, current_output.len(), ratio);
    let owners = classify_frames(&features, input.len(), ratio);
    let frames = features
        .iter()
        .zip(owners)
        .map(|(features, owner)| StretchHybridFrameTrace {
            source_frame: features.source_frame,
            output_frame: features.output_frame,
            spectral_flux_ratio: features.spectral_flux_ratio,
            energy_ratio: features.energy_ratio,
            spectral_stability: features.spectral_stability,
            owner,
        })
        .collect::<Vec<_>>();
    let transitions = schedule_transitions(&frames, current_output, ratio);

    StretchHybridTrace {
        ratio,
        input_frames: input.len(),
        output_frames: current_output.len(),
        frames,
        transitions,
    }
}

fn analyze_short_frames(
    input: &[Sample],
    output_frames: usize,
    ratio: f64,
) -> Vec<HybridFrameFeatures> {
    let window_size = COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE;
    let hop = COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP;
    if input.len() < window_size {
        return Vec::new();
    }

    let window = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect::<Vec<_>>();
    let bins = window_size / 2 + 1;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(window_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    let mut previous_magnitudes = vec![0.0f32; bins];
    let mut previous_energy = 0.0f64;
    let frame_count = (input.len() - window_size) / hop + 1;
    let mut features = Vec::with_capacity(frame_count);

    for frame_index in 0..frame_count {
        let start = frame_index * hop;
        let mut energy = 0.0f64;
        for (slot, (sample, weight)) in buffer
            .iter_mut()
            .zip(input[start..start + window_size].iter().zip(window.iter()))
        {
            let windowed = sample * weight;
            energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
        }
        energy /= window_size as f64;
        fft.process(&mut buffer);

        let mut magnitude_sum = 0.0f64;
        let mut positive_flux = 0.0f64;
        let mut stability_denominator = 0.0f64;
        let mut stability_delta = 0.0f64;
        for bin in 0..bins {
            let magnitude = buffer[bin].norm();
            let previous = previous_magnitudes[bin];
            magnitude_sum += magnitude as f64;
            positive_flux += (magnitude - previous).max(0.0) as f64;
            stability_denominator += (magnitude + previous) as f64;
            stability_delta += (magnitude - previous).abs() as f64;
            previous_magnitudes[bin] = magnitude;
        }

        let spectral_flux_ratio = if frame_index == 0 {
            0.0
        } else {
            positive_flux / (magnitude_sum + 1.0e-12)
        };
        let energy_ratio = if frame_index == 0 {
            1.0
        } else {
            energy / (previous_energy + 1.0e-12)
        };
        let spectral_stability = if frame_index == 0 {
            1.0
        } else {
            (1.0 - stability_delta / (stability_denominator + 1.0e-12)).clamp(0.0, 1.0)
        };
        let source_frame = start + window_size / 2;
        let output_frame = projected_output_frame(source_frame, output_frames, ratio);
        features.push(HybridFrameFeatures {
            source_frame,
            output_frame,
            spectral_flux_ratio,
            energy_ratio,
            spectral_stability,
            onset: frame_index > 0
                && spectral_flux_ratio >= TRANSIENT_FLUX_RATIO
                && energy_ratio >= TRANSIENT_ENERGY_RATIO,
        });
        previous_energy = energy;
    }
    features
}

fn classify_frames(
    features: &[HybridFrameFeatures],
    input_frames: usize,
    ratio: f64,
) -> Vec<StretchHybridOwner> {
    if (ratio - 1.0).abs() < 1.0e-9 {
        return vec![StretchHybridOwner::Mixed; features.len()];
    }

    let mut transient_guard = vec![false; features.len()];
    for (index, frame) in features.iter().enumerate() {
        if !frame.onset {
            continue;
        }
        let start = index.saturating_sub(TRANSIENT_PREROLL_FRAMES);
        let end = (index + TRANSIENT_POSTROLL_FRAMES + 1).min(features.len());
        transient_guard[start..end].fill(true);
    }

    let boundary_guard = DEFAULT_WINDOW_SIZE / 2;
    let mut stable_frames = 0usize;
    features
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            let in_boundary_guard = frame.source_frame < boundary_guard
                || frame.source_frame.saturating_add(boundary_guard) >= input_frames;
            if in_boundary_guard {
                stable_frames = 0;
                return StretchHybridOwner::Mixed;
            }
            if transient_guard[index] {
                stable_frames = 0;
                return StretchHybridOwner::Transient;
            }
            if ratio > 1.0 && frame.spectral_stability >= TONAL_STABILITY {
                stable_frames += 1;
                if stable_frames >= TONAL_ENTRY_FRAMES {
                    return StretchHybridOwner::Tonal;
                }
            } else {
                stable_frames = 0;
            }
            StretchHybridOwner::Mixed
        })
        .collect()
}

fn projected_output_frame(source_frame: usize, output_frames: usize, ratio: f64) -> usize {
    if output_frames == 0 {
        return 0;
    }
    ((source_frame as f64 * ratio).round() as usize).min(output_frames.saturating_sub(1))
}

#[cfg(test)]
#[path = "hybrid_trace/tests.rs"]
mod tests;
