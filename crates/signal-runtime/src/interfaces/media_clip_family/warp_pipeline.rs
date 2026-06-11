/// Time-stretch/warp mode for a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpMode {
    /// Warp disabled; audio plays at native rate.
    Off,
    /// Pitch is shifted without time-stretching.
    Repitch,
    /// Elastique Draft time-stretch algorithm.
    ElastiqueDraft,
}

/// Interpolation mode between tempo map segments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoMapInterpolation {
    #[default]
    /// Tempo holds constant until the next segment.
    Hold,
    /// Tempo ramps linearly to the next segment's start tempo.
    Linear,
}

/// A single segment of the tempo map projection with optional end tempo for ramps.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentProjection {
    /// Unique identifier for this segment.
    pub segment_id: String,
    /// Timeline start position in samples.
    pub start_samples: i64,
    /// Timeline end position in samples, or `None` if the segment extends indefinitely.
    pub end_samples: Option<i64>,
    /// Tempo at the start of this segment in BPM.
    pub start_tempo_bpm: f64,
    /// Tempo at the end of this segment in BPM, used for ramp interpolation.
    pub end_tempo_bpm: Option<f64>,
    /// Interpolation mode between this segment and the next.
    pub interpolation: RuntimeTempoMapInterpolation,
}

/// Full tempo map projection: ordered segments covering the session timeline.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapProjection {
    /// Number of segments in the projection.
    pub segment_count: usize,
    /// Ordered tempo map segments.
    pub segments: Vec<RuntimeTempoMapSegmentProjection>,
}

/// Source from which the resolved project tempo was derived.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoSource {
    #[default]
    /// No explicit tempo; using the built-in fallback value.
    DefaultFallback,
    /// Tempo derived from the transport projection.
    TransportProjection,
    /// Tempo derived from a specific tempo map segment.
    TempoMapSegment,
}

/// Snapshot of a single tempo map segment including current-position coverage flag.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentSnapshot {
    /// Unique identifier for this segment.
    pub segment_id: String,
    /// Timeline start position in samples.
    pub start_samples: i64,
    /// Timeline end position in samples, or `None` if the segment extends indefinitely.
    pub end_samples: Option<i64>,
    /// Tempo at the start of this segment in BPM.
    pub start_tempo_bpm: f64,
    /// Tempo at the end of this segment in BPM, used for ramp interpolation.
    pub end_tempo_bpm: Option<f64>,
    /// Interpolation mode between this segment and the next.
    pub interpolation: RuntimeTempoMapInterpolation,
    /// Whether this segment covers the current timeline playback position.
    pub covers_timeline_position: bool,
}

/// Runtime snapshot of the full tempo map with the active segment and
/// resolved project tempo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapSnapshot {
    /// Number of segments in the tempo map.
    pub segment_count: usize,
    /// ID of the segment that covers the current timeline position, if any.
    pub active_segment_id: Option<String>,
    /// Index of the active segment within the ordered segment list, if any.
    pub active_segment_index: Option<usize>,
    /// Timeline start position of the next segment in samples, if any.
    pub next_segment_start_samples: Option<i64>,
    /// Resolved project tempo in BPM used by the engine this tick.
    pub resolved_tempo_bpm: f64,
    /// Source from which `resolved_tempo_bpm` was derived.
    pub tempo_source: RuntimeTempoSource,
    /// Current timeline playback position in samples, if known.
    pub timeline_position_samples: Option<i64>,
    /// All tempo map segment snapshots in order.
    pub segments: Vec<RuntimeTempoMapSegmentSnapshot>,
}

/// Warp readiness state for a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpReadiness {
    /// Warp is disabled for this clip.
    Bypassed,
    /// Warp pipeline is ready to process this clip.
    Ready,
    /// Warp pipeline is in a degraded state for this clip.
    Degraded,
}

/// Registration parameters for a warp clip in the warp pipeline.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipRegistration {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Warp mode to apply to this clip.
    pub mode: RuntimeWarpMode,
    /// Native source tempo of the clip in BPM, if known.
    pub source_tempo_bpm: Option<f64>,
    /// Timeline sample position used as the warp anchor.
    pub anchor_timeline_samples: i64,
    /// Start position of the clip in timeline samples.
    pub start_samples: i64,
    /// Duration of the clip in samples.
    pub duration_samples: u32,
}

/// Full warp snapshot for one clip: mode, ratios, tempo source, and readiness.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipSnapshot {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Warp mode applied to this clip.
    pub mode: RuntimeWarpMode,
    /// Native source tempo of the clip in BPM, if known.
    pub source_tempo_bpm: Option<f64>,
    /// Resolved project tempo used to compute the warp ratio, in BPM.
    pub project_tempo_bpm: f64,
    /// Source from which `project_tempo_bpm` was derived.
    pub project_tempo_source: RuntimeTempoSource,
    /// ID of the tempo map segment used to resolve `project_tempo_bpm`, if any.
    pub project_tempo_segment_id: Option<String>,
    /// The realized stretch ratio applied to this clip.
    pub realized_ratio: f64,
    /// Timeline sample position used as the warp anchor.
    pub anchor_timeline_samples: i64,
    /// Start position of the clip in timeline samples.
    pub start_samples: i64,
    /// Duration of the clip in samples.
    pub duration_samples: u32,
    /// Current warp readiness state.
    pub readiness: RuntimeWarpReadiness,
    /// Last error message if the warp pipeline encountered a problem, if any.
    pub last_error: Option<String>,
}

/// Aggregate warp pipeline snapshot: counts by readiness and resolved
/// project tempo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeWarpPipelineSnapshot {
    /// Total number of clips registered in the warp pipeline.
    pub clip_count: usize,
    /// Number of clips with warp ready.
    pub ready_clip_count: usize,
    /// Number of clips in a degraded warp state.
    pub degraded_clip_count: usize,
    /// Number of clips with warp bypassed.
    pub bypassed_clip_count: usize,
    /// Number of clips with active (non-bypass, non-off) warp.
    pub active_warp_count: usize,
    /// Resolved project tempo in BPM used across the pipeline this tick.
    pub resolved_project_tempo_bpm: f64,
    /// Source from which `resolved_project_tempo_bpm` was derived.
    pub resolved_project_tempo_source: RuntimeTempoSource,
    /// ID of the tempo map segment used to resolve project tempo, if any.
    pub resolved_project_tempo_segment_id: Option<String>,
    /// Per-clip warp snapshots.
    pub clips: Vec<RuntimeWarpClipSnapshot>,
}
