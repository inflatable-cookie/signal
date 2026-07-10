#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FixedMapPeakRegionEvidence {
    pub(crate) event_index: usize,
    pub(crate) analysis_frame_index: usize,
    pub(crate) source_center_frame: usize,
    pub(crate) peak_bin: usize,
    pub(crate) first_bin: usize,
    pub(crate) end_bin: usize,
    pub(crate) energy_position_frames: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FixedMapPeakEventEvidence {
    pub(crate) onset_frame: usize,
    pub(crate) first_analysis_frame: Option<usize>,
    pub(crate) last_analysis_frame: Option<usize>,
    pub(crate) reinitialized_analysis_frame: Option<usize>,
    pub(crate) collected_peak_regions: usize,
    pub(crate) reinitialized_bins: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FixedMapPeakEvidence {
    pub(crate) center_threshold_frames: f64,
    pub(crate) events: Vec<FixedMapPeakEventEvidence>,
    pub(crate) candidate_regions: Vec<FixedMapPeakRegionEvidence>,
    pub(crate) threshold_crossings: usize,
}
