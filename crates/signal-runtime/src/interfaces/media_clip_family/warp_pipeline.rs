#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpMode {
    Off,
    Repitch,
    ElastiqueDraft,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoMapInterpolation {
    #[default]
    Hold,
    Linear,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentProjection {
    pub segment_id: String,
    pub start_samples: i64,
    pub end_samples: Option<i64>,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: Option<f64>,
    pub interpolation: RuntimeTempoMapInterpolation,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapProjection {
    pub segment_count: usize,
    pub segments: Vec<RuntimeTempoMapSegmentProjection>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoSource {
    #[default]
    DefaultFallback,
    TransportProjection,
    TempoMapSegment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeTempoMapSegmentSnapshot {
    pub segment_id: String,
    pub start_samples: i64,
    pub end_samples: Option<i64>,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: Option<f64>,
    pub interpolation: RuntimeTempoMapInterpolation,
    pub covers_timeline_position: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTempoMapSnapshot {
    pub segment_count: usize,
    pub active_segment_id: Option<String>,
    pub active_segment_index: Option<usize>,
    pub next_segment_start_samples: Option<i64>,
    pub resolved_tempo_bpm: f64,
    pub tempo_source: RuntimeTempoSource,
    pub timeline_position_samples: Option<i64>,
    pub segments: Vec<RuntimeTempoMapSegmentSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWarpReadiness {
    Bypassed,
    Ready,
    Degraded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipRegistration {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub mode: RuntimeWarpMode,
    pub source_tempo_bpm: Option<f64>,
    pub anchor_timeline_samples: i64,
    pub start_samples: i64,
    pub duration_samples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeWarpClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub mode: RuntimeWarpMode,
    pub source_tempo_bpm: Option<f64>,
    pub project_tempo_bpm: f64,
    pub project_tempo_source: RuntimeTempoSource,
    pub project_tempo_segment_id: Option<String>,
    pub realized_ratio: f64,
    pub anchor_timeline_samples: i64,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub readiness: RuntimeWarpReadiness,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeWarpPipelineSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub degraded_clip_count: usize,
    pub bypassed_clip_count: usize,
    pub active_warp_count: usize,
    pub resolved_project_tempo_bpm: f64,
    pub resolved_project_tempo_source: RuntimeTempoSource,
    pub resolved_project_tempo_segment_id: Option<String>,
    pub clips: Vec<RuntimeWarpClipSnapshot>,
    pub summary: String,
}
