use signal_analysis::Confidence;

/// One ranked tempo hypothesis from the autocorrelation search.
#[derive(Clone, Debug, PartialEq)]
pub struct TempoCandidate {
    /// Tempo in beats per minute.
    pub bpm: f32,
    /// Confidence assigned to this candidate.
    pub confidence: Confidence,
}

/// Tempo measurement over a local window of beats.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalTempoPoint {
    /// Index of the first beat in this window.
    pub start_beat_index: usize,
    /// Index of the last beat in this window (exclusive).
    pub end_beat_index: usize,
    /// Audio time at the start of this window, in seconds.
    pub start_seconds: f32,
    /// Audio time at the end of this window, in seconds.
    pub end_seconds: f32,
    /// Local tempo estimate for this window, in BPM.
    pub bpm: f32,
}

/// Detailed tempo stability diagnostics for a single analysis pass.
#[derive(Clone, Debug, PartialEq)]
pub struct TempoDiagnostics {
    /// Per-beat-interval tempo measurements.
    pub interval_tempi: Vec<LocalTempoPoint>,
    /// Windowed tempo measurements (multi-beat windows).
    pub windowed_tempi: Vec<LocalTempoPoint>,
    /// Median of all per-interval tempo estimates, in BPM.
    pub median_bpm: f32,
    /// Peak-to-peak range of per-interval tempo estimates, in BPM.
    pub drift_span_bpm: f32,
    /// Mean absolute deviation of per-interval estimates from the median, in BPM.
    pub mean_abs_deviation_bpm: f32,
    /// Median of windowed tempo estimates, in BPM.
    pub windowed_median_bpm: f32,
    /// Peak-to-peak range of windowed estimates, in BPM.
    pub windowed_drift_span_bpm: f32,
    /// Mean absolute deviation of windowed estimates from their median, in BPM.
    pub windowed_mean_abs_deviation_bpm: f32,
    /// Median of core-window tempo estimates (edge-trimmed), in BPM.
    pub core_windowed_median_bpm: f32,
    /// Peak-to-peak range of core-window estimates, in BPM.
    pub core_windowed_drift_span_bpm: f32,
    /// Mean absolute deviation of core-window estimates from their median, in BPM.
    pub core_windowed_mean_abs_deviation_bpm: f32,
    /// Systematic tempo difference between the track boundary and core regions, in BPM.
    pub boundary_bias_bpm: f32,
    /// Direction and magnitude of any tempo trend across the track.
    pub trend: TempoTrendDiagnostics,
    /// Beat-grid residual and drift error statistics.
    pub beat_grid_error: BeatGridErrorDiagnostics,
    /// Statistics about beat-interval outliers filtered during refinement.
    pub beat_interval_outliers: BeatIntervalOutlierDiagnostics,
    /// Stability scope classification and its supporting evidence.
    pub stability_scope: TempoStabilityScopeSummary,
    /// Edge-trimmed stable span, if a stable core region was found after trimming edges.
    pub edge_trimmed_stable_span: Option<BeatGridCoreSpanDiagnostics>,
    /// Stable core span (more aggressively trimmed than edge-trimmed), if present.
    pub stable_core_span: Option<BeatGridCoreSpanDiagnostics>,
}

/// Classification of where tempo stability holds across the track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStabilityScope {
    /// Tempo is stable across the entire track.
    WholeTrackStable,
    /// Tempo is stable in the core but has localised instability near one or both edges.
    StableWithLocalizedEdgeDamage,
    /// Only a central core region is stable; edges are significantly unstable.
    CoreStableOnly,
    /// Instability occurs in the mid-track, not just at the edges.
    MidTrackUnstable,
}

/// Evidence scores that support the stability scope classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStabilityScopeSupport {
    /// Fraction of the track covered by the edge-trimmed stable span.
    pub edge_trimmed_coverage: Confidence,
    /// Fraction of the track covered by a contiguous stable core.
    pub contiguous_core_coverage: Confidence,
    /// Stability score for the interior of the track (excluding edges).
    pub interior_stability: Confidence,
    /// Degree to which instability is localised near the track edges.
    pub edge_locality: Confidence,
}

/// Stability scope classification and its supporting evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStabilityScopeSummary {
    /// The inferred stability scope.
    pub scope: TempoStabilityScope,
    /// Evidence scores that support the scope classification.
    pub support: TempoStabilityScopeSupport,
}

/// Overall tempo trend direction across the track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrendDirection {
    /// No significant drift detected.
    Stable,
    /// Tempo increases over the course of the track.
    Accelerating,
    /// Tempo decreases over the course of the track.
    Decelerating,
}

/// Tempo trend diagnostics derived from a linear fit over beat intervals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoTrendDiagnostics {
    /// Direction of the overall tempo trend.
    pub direction: TempoTrendDirection,
    /// Fitted tempo at the start of the track, in BPM.
    pub start_bpm: f32,
    /// Fitted tempo at the end of the track, in BPM.
    pub end_bpm: f32,
    /// Total drift from start to end (`end_bpm - start_bpm`), in BPM.
    pub total_drift_bpm: f32,
    /// Slope of the linear fit in BPM per beat.
    pub slope_bpm_per_beat: f32,
    /// Mean absolute deviation of beat intervals from the linear fit, in BPM.
    pub fit_mean_abs_deviation_bpm: f32,
}

/// Residual error for one beat against the fitted beat grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGridResidualPoint {
    /// Zero-based index of this beat.
    pub beat_index: usize,
    /// Detected beat time, in seconds.
    pub seconds: f32,
    /// Signed error relative to the ideal grid (positive = late), in milliseconds.
    pub fitted_residual_ms: f32,
    /// Cumulative drift from the first beat to this beat, in milliseconds.
    pub anchored_drift_ms: f32,
}

/// Beat-grid error statistics for a single analysis pass.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatGridErrorDiagnostics {
    /// Per-beat residuals against the fitted grid.
    pub residuals: Vec<BeatGridResidualPoint>,
    /// Mean absolute residual across all beats, in milliseconds.
    pub mean_abs_residual_ms: f32,
    /// Largest single-beat absolute residual, in milliseconds.
    pub max_abs_residual_ms: f32,
    /// Mean absolute residual for beats near the track edges, in milliseconds.
    pub edge_mean_abs_residual_ms: f32,
    /// Mean absolute residual for beats in the track core, in milliseconds.
    pub core_mean_abs_residual_ms: f32,
    /// Cumulative drift from the first beat to the last beat, in milliseconds.
    pub end_anchored_drift_ms: f32,
    /// Mean absolute anchored drift across all beats, in milliseconds.
    pub mean_abs_anchored_drift_ms: f32,
}

/// Statistics about beat-interval outliers removed during BPM refinement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatIntervalOutlierDiagnostics {
    /// Total number of beat intervals before filtering.
    pub total_intervals: usize,
    /// Number of intervals retained after outlier removal.
    pub retained_intervals: usize,
    /// Number of intervals rejected as outliers.
    pub rejected_intervals: usize,
    /// Rejected intervals at the start of the beat sequence.
    pub leading_rejected_intervals: usize,
    /// Rejected intervals at the end of the beat sequence.
    pub trailing_rejected_intervals: usize,
    /// Median beat interval of the retained set, in onset frames.
    pub median_interval: f32,
    /// Median absolute deviation of retained intervals.
    pub median_abs_deviation: f32,
    /// Largest deviation ratio among all rejected intervals.
    pub max_rejected_deviation_ratio: f32,
}

/// Diagnostics for a stable core span of the beat grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGridCoreSpanDiagnostics {
    /// Beat index of the first beat in this span.
    pub start_beat_index: usize,
    /// Beat index of the last beat in this span.
    pub end_beat_index: usize,
    /// Audio time at the start of this span, in seconds.
    pub start_seconds: f32,
    /// Audio time at the end of this span, in seconds.
    pub end_seconds: f32,
    /// Fraction of the full track covered by this span.
    pub coverage: Confidence,
    /// Number of analysis windows retained within this span.
    pub retained_windows: usize,
    /// Total number of analysis windows considered.
    pub total_windows: usize,
    /// Windows trimmed from the leading edge.
    pub trimmed_leading_windows: usize,
    /// Windows trimmed from the trailing edge.
    pub trimmed_trailing_windows: usize,
    /// Windows rejected from the interior of the span.
    pub interior_rejected_windows: usize,
}
