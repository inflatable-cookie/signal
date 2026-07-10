use signal_primitives::Sample;

use crate::{
    phase_vocoder::fixed_map_peak_transient_phase_vocoder, DEFAULT_ANALYSIS_HOP,
    DEFAULT_WINDOW_SIZE,
};

use super::{build_hybrid_trace, StretchHybridOwner, StretchHybridTrace};

/// One guarded onset event in the report-only fixed-map peak proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFixedMapPeakEventTrace {
    /// Frozen classifier onset centre in source sample frames.
    pub onset_frame: usize,
    /// First assigned default-grid analysis frame, when one exists.
    pub first_analysis_frame: Option<usize>,
    /// Last assigned default-grid analysis frame, when one exists.
    pub last_analysis_frame: Option<usize>,
    /// Analysis frame where collected bins copied analysis phase.
    pub reinitialized_analysis_frame: Option<usize>,
    /// Peak-region observations collected before the centre crossing.
    pub collected_peak_regions: usize,
    /// Number of bins reinitialized together for this event.
    pub reinitialized_bins: usize,
}

/// One peak-local group-delay candidate in a guarded event.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFixedMapPeakRegionTrace {
    /// Index into the render's event trace.
    pub event_index: usize,
    /// Default-grid analysis frame containing this candidate.
    pub analysis_frame_index: usize,
    /// Analysis-window centre in source sample frames.
    pub source_center_frame: usize,
    /// Local spectral-peak bin.
    pub peak_bin: usize,
    /// First bin in the magnitude-minimum region.
    pub first_bin: usize,
    /// Exclusive end bin in the magnitude-minimum region.
    pub end_bin: usize,
    /// Energy-weighted group delay relative to the window centre.
    pub energy_position_frames: f64,
}

/// Report-only fixed-map peak-selective transient render and evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFixedMapPeakTransientRender {
    /// Candidate mono samples. Product routing never consumes this field.
    pub samples: Vec<Sample>,
    /// Frozen classifier trace used only to guard onset events.
    pub trace: StretchHybridTrace,
    /// Window-derived centre threshold in sample frames.
    pub center_threshold_frames: f64,
    /// Guarded event decisions.
    pub events: Vec<StretchFixedMapPeakEventTrace>,
    /// Peak-local candidates collected inside guarded events.
    pub candidate_regions: Vec<StretchFixedMapPeakRegionTrace>,
    /// Number of event energy positions that crossed the centre threshold.
    pub threshold_crossings: usize,
    /// Cropped output frames without overlap-add normalization coverage.
    pub uncovered_output_frames: usize,
}

pub(crate) fn build_fixed_map_peak_transient_render(
    input: &[Sample],
    current: &[Sample],
    ratio: f64,
) -> StretchFixedMapPeakTransientRender {
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
    let engine = fixed_map_peak_transient_phase_vocoder(
        input,
        current.len(),
        ratio,
        DEFAULT_WINDOW_SIZE,
        DEFAULT_ANALYSIS_HOP,
        &onset_frames,
    );
    StretchFixedMapPeakTransientRender {
        samples: engine.samples,
        trace,
        center_threshold_frames: engine.evidence.center_threshold_frames,
        events: engine
            .evidence
            .events
            .into_iter()
            .map(|event| StretchFixedMapPeakEventTrace {
                onset_frame: event.onset_frame,
                first_analysis_frame: event.first_analysis_frame,
                last_analysis_frame: event.last_analysis_frame,
                reinitialized_analysis_frame: event.reinitialized_analysis_frame,
                collected_peak_regions: event.collected_peak_regions,
                reinitialized_bins: event.reinitialized_bins,
            })
            .collect(),
        candidate_regions: engine
            .evidence
            .candidate_regions
            .into_iter()
            .map(|region| StretchFixedMapPeakRegionTrace {
                event_index: region.event_index,
                analysis_frame_index: region.analysis_frame_index,
                source_center_frame: region.source_center_frame,
                peak_bin: region.peak_bin,
                first_bin: region.first_bin,
                end_bin: region.end_bin,
                energy_position_frames: region.energy_position_frames,
            })
            .collect(),
        threshold_crossings: engine.evidence.threshold_crossings,
        uncovered_output_frames: engine.uncovered_output_frames,
    }
}

fn unchanged_render(
    current: &[Sample],
    trace: StretchHybridTrace,
) -> StretchFixedMapPeakTransientRender {
    StretchFixedMapPeakTransientRender {
        samples: current.to_vec(),
        trace,
        center_threshold_frames: 0.0,
        events: Vec::new(),
        candidate_regions: Vec::new(),
        threshold_crossings: 0,
        uncovered_output_frames: 0,
    }
}
