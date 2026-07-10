use signal_primitives::Sample;

/// Mechanism evidence from the report-only full phase-gradient proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchPhaseGradientEvidence {
    /// Frozen Hann-window length.
    pub window_frames: usize,
    /// Frozen transform length.
    pub fft_frames: usize,
    /// Ratio-derived analysis hop.
    pub analysis_hop_frames: usize,
    /// Adjacent analysis intervals equal to the ideal interval floor.
    pub analysis_interval_floor_count: usize,
    /// Adjacent analysis intervals equal to the ideal interval ceiling.
    pub analysis_interval_ceiling_count: usize,
    /// Largest absolute analysis-centre error from the ideal source map.
    pub max_analysis_mapping_error_frames: f64,
    /// Final analysis-centre error from the ideal source map.
    pub final_analysis_mapping_error_frames: f64,
    /// Whether absolute analysis centres increase strictly.
    pub analysis_positions_monotonic: bool,
    /// Frozen synthesis hop.
    pub synthesis_hop_frames: usize,
    /// Number of synthesized STFT frames.
    pub synthesis_frames: usize,
    /// Significant current-frame bins processed by the heap.
    pub significant_bins: usize,
    /// Current-frame bins at or below the relative tolerance.
    pub insignificant_bins: usize,
    /// Phase assignments propagated from the preceding frame.
    pub horizontal_assignments: usize,
    /// Phase assignments propagated between adjacent frequency bins.
    pub vertical_assignments: usize,
    /// Significant bins assigned more than once.
    pub duplicate_assignments: usize,
    /// Significant bins left without an assignment.
    pub missing_assignments: usize,
    /// Largest number of entries held by the integration heap.
    pub heap_high_water: usize,
    /// Declared maximum integration-heap entries.
    pub heap_capacity_bound: usize,
    /// Largest mirrored-spectrum conjugate-symmetry error.
    pub max_conjugate_symmetry_error: f64,
    /// Cropped output samples without overlap-add normalization coverage.
    pub uncovered_output_samples: usize,
    /// Whether every estimated phase derivative was finite.
    pub derivatives_finite: bool,
    /// Whether every rendered output sample was finite.
    pub all_samples_finite: bool,
    /// Whether synthesis frame positions were monotonic.
    pub synthesis_positions_monotonic: bool,
    /// Stable FNV-1a hash of output sample bits.
    pub sample_hash: u64,
    /// Stable FNV-1a hash of phase-assignment decisions.
    pub trace_hash: u64,
}

/// Report-only fixed-ratio mono full phase-gradient render.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchPhaseGradientRender {
    /// Exact-length rendered mono samples.
    pub samples: Vec<Sample>,
    /// Phase-gradient integration and reconstruction evidence.
    pub evidence: StretchPhaseGradientEvidence,
}
