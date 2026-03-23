use signal_analysis::Confidence;

#[derive(Clone, Debug, PartialEq)]
pub struct TempoCandidate {
    pub bpm: f32,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalTempoPoint {
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub bpm: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempoDiagnostics {
    pub interval_tempi: Vec<LocalTempoPoint>,
    pub windowed_tempi: Vec<LocalTempoPoint>,
    pub median_bpm: f32,
    pub drift_span_bpm: f32,
    pub mean_abs_deviation_bpm: f32,
    pub windowed_median_bpm: f32,
    pub windowed_drift_span_bpm: f32,
    pub windowed_mean_abs_deviation_bpm: f32,
    pub core_windowed_median_bpm: f32,
    pub core_windowed_drift_span_bpm: f32,
    pub core_windowed_mean_abs_deviation_bpm: f32,
    pub boundary_bias_bpm: f32,
    pub trend: TempoTrendDiagnostics,
    pub beat_grid_error: BeatGridErrorDiagnostics,
    pub beat_interval_outliers: BeatIntervalOutlierDiagnostics,
    pub stability_scope: TempoStabilityScopeSummary,
    pub edge_trimmed_stable_span: Option<BeatGridCoreSpanDiagnostics>,
    pub stable_core_span: Option<BeatGridCoreSpanDiagnostics>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStabilityScope {
    WholeTrackStable,
    StableWithLocalizedEdgeDamage,
    CoreStableOnly,
    MidTrackUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStabilityScopeSupport {
    pub edge_trimmed_coverage: Confidence,
    pub contiguous_core_coverage: Confidence,
    pub interior_stability: Confidence,
    pub edge_locality: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStabilityScopeSummary {
    pub scope: TempoStabilityScope,
    pub support: TempoStabilityScopeSupport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrendDirection {
    Stable,
    Accelerating,
    Decelerating,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoTrendDiagnostics {
    pub direction: TempoTrendDirection,
    pub start_bpm: f32,
    pub end_bpm: f32,
    pub total_drift_bpm: f32,
    pub slope_bpm_per_beat: f32,
    pub fit_mean_abs_deviation_bpm: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGridResidualPoint {
    pub beat_index: usize,
    pub seconds: f32,
    pub fitted_residual_ms: f32,
    pub anchored_drift_ms: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeatGridErrorDiagnostics {
    pub residuals: Vec<BeatGridResidualPoint>,
    pub mean_abs_residual_ms: f32,
    pub max_abs_residual_ms: f32,
    pub edge_mean_abs_residual_ms: f32,
    pub core_mean_abs_residual_ms: f32,
    pub end_anchored_drift_ms: f32,
    pub mean_abs_anchored_drift_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatIntervalOutlierDiagnostics {
    pub total_intervals: usize,
    pub retained_intervals: usize,
    pub rejected_intervals: usize,
    pub leading_rejected_intervals: usize,
    pub trailing_rejected_intervals: usize,
    pub median_interval: f32,
    pub median_abs_deviation: f32,
    pub max_rejected_deviation_ratio: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGridCoreSpanDiagnostics {
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub coverage: Confidence,
    pub retained_windows: usize,
    pub total_windows: usize,
    pub trimmed_leading_windows: usize,
    pub trimmed_trailing_windows: usize,
    pub interior_rejected_windows: usize,
}
