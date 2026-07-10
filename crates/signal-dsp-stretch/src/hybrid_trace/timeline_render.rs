use signal_primitives::Sample;

use crate::{
    phase_vocoder::adaptive_transient_timeline_phase_vocoder, DEFAULT_ANALYSIS_HOP,
    DEFAULT_WINDOW_SIZE,
};

use super::{build_hybrid_trace, StretchHybridOwner, StretchHybridTrace};

/// Report-only current-grid adaptive transient-timeline render and evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchAdaptiveTimelineRender {
    /// Candidate mono samples. Product routing never consumes this field.
    pub samples: Vec<Sample>,
    /// Frozen classifier trace used to select protected onset centres.
    pub trace: StretchHybridTrace,
    /// Internal synthesis-window start positions in output sample frames.
    pub synthesis_positions: Vec<usize>,
    /// Default-grid analysis frame indices reinitialized from analysis phase.
    pub reinitialized_frames: Vec<usize>,
    /// Number of non-conflicting onset islands protected by the time map.
    pub protected_onset_count: usize,
    /// Number of detected onsets rejected as dense conflicts.
    pub dense_conflict_count: usize,
    /// Largest absolute projected-anchor error in sample frames.
    pub max_anchor_error_frames: f64,
    /// Smallest adjacent synthesis hop in sample frames.
    pub min_synthesis_hop_frames: usize,
    /// Largest adjacent synthesis hop in sample frames.
    pub max_synthesis_hop_frames: usize,
    /// Cropped output frames without overlap-add normalization coverage.
    pub uncovered_output_frames: usize,
    /// Whether schedule construction fell back to the uniform current map.
    pub schedule_fallback: bool,
}

pub(crate) fn build_adaptive_timeline_render(
    input: &[Sample],
    current: &[Sample],
    ratio: f64,
) -> StretchAdaptiveTimelineRender {
    let trace = build_hybrid_trace(input, current, ratio);
    if current.is_empty() || (ratio - 1.0).abs() < 1.0e-9 {
        return unchanged_render(current, trace);
    }
    let onset_frames = trace
        .frames
        .iter()
        .filter(|frame| frame.detected_onset && frame.owner == StretchHybridOwner::Transient)
        .map(|frame| frame.source_frame)
        .collect::<Vec<_>>();
    let engine = adaptive_transient_timeline_phase_vocoder(
        input,
        current.len(),
        ratio,
        DEFAULT_WINDOW_SIZE,
        DEFAULT_ANALYSIS_HOP,
        &onset_frames,
    );
    StretchAdaptiveTimelineRender {
        samples: engine.samples,
        trace,
        synthesis_positions: engine.synthesis_positions,
        reinitialized_frames: engine.reinitialized_frames,
        protected_onset_count: engine.protected_onset_count,
        dense_conflict_count: engine.dense_conflict_count,
        max_anchor_error_frames: engine.max_anchor_error_frames,
        min_synthesis_hop_frames: engine.min_synthesis_hop_frames,
        max_synthesis_hop_frames: engine.max_synthesis_hop_frames,
        uncovered_output_frames: engine.uncovered_output_frames,
        schedule_fallback: engine.schedule_fallback,
    }
}

fn unchanged_render(
    current: &[Sample],
    trace: StretchHybridTrace,
) -> StretchAdaptiveTimelineRender {
    StretchAdaptiveTimelineRender {
        samples: current.to_vec(),
        trace,
        synthesis_positions: Vec::new(),
        reinitialized_frames: Vec::new(),
        protected_onset_count: 0,
        dense_conflict_count: 0,
        max_anchor_error_frames: 0.0,
        min_synthesis_hop_frames: 0,
        max_synthesis_hop_frames: 0,
        uncovered_output_frames: 0,
        schedule_fallback: false,
    }
}
