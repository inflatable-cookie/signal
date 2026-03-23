use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingReadiness {
    Ready,
    PendingMedia,
    PendingWarp,
    Invalid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipFadeShape {
    #[default]
    Linear,
    EqualPower,
    SmoothStep,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipGainShape {
    #[default]
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingStage {
    Warp,
    FadeIn,
    GainShape,
    FadeOut,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipFadeEnvelope {
    pub duration_samples: u32,
    pub shape: RuntimeClipFadeShape,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipGainEnvelope {
    pub start_linear: f32,
    pub end_linear: f32,
    pub shape: RuntimeClipGainShape,
}

impl Default for RuntimeClipGainEnvelope {
    fn default() -> Self {
        Self {
            start_linear: 1.0,
            end_linear: 1.0,
            shape: RuntimeClipGainShape::Hold,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingRegistration {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub warp_mode: RuntimeWarpMode,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub fade_in: RuntimeClipFadeEnvelope,
    pub fade_out: RuntimeClipFadeEnvelope,
    pub clip_gain: RuntimeClipGainEnvelope,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub warp_mode: RuntimeWarpMode,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub fade_in: RuntimeClipFadeEnvelope,
    pub fade_out: RuntimeClipFadeEnvelope,
    pub fade_in_end_samples: i64,
    pub fade_out_start_samples: i64,
    pub clip_gain: RuntimeClipGainEnvelope,
    pub treatment_stages: Vec<RuntimeClipProcessingStage>,
    pub realized_warp_ratio: Option<f64>,
    pub project_tempo_source: Option<RuntimeTempoSource>,
    pub project_tempo_segment_id: Option<String>,
    pub readiness: RuntimeClipProcessingReadiness,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipProcessingPipelineSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_media_clip_count: usize,
    pub pending_warp_clip_count: usize,
    pub invalid_clip_count: usize,
    pub faded_clip_count: usize,
    pub gain_shaped_clip_count: usize,
    pub warped_clip_count: usize,
    pub treatment_stage_count: usize,
    pub clips: Vec<RuntimeClipProcessingSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchEngineClass {
    Disabled,
    RatioOnly,
    SampleDomain,
    Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchReadiness {
    Disabled,
    Ready,
    PendingMedia,
    PendingWarp,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchFallbackKind {
    None,
    Bypass,
    RatioOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStretchClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub engine_class: RuntimeStretchEngineClass,
    pub readiness: RuntimeStretchReadiness,
    pub fallback_kind: RuntimeStretchFallbackKind,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeStretchEngineSnapshot {
    pub clip_count: usize,
    pub disabled_clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_media_clip_count: usize,
    pub pending_warp_clip_count: usize,
    pub degraded_clip_count: usize,
    pub sample_domain_clip_count: usize,
    pub ratio_only_clip_count: usize,
    pub fallback_clip_count: usize,
    pub clips: Vec<RuntimeStretchClipSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMarkerAnalysisReadiness {
    #[default]
    Empty,
    PendingMedia,
    Ready,
    Degraded,
    Invalidated,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMarkerAnalysisInvalidationState {
    #[default]
    None,
    MediaInvalidated,
    StretchInvalidated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoAssistPosture {
    #[default]
    Unavailable,
    Guarded,
    Ready,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoAssistHintSource {
    #[default]
    None,
    SourceTempo,
    ProjectTempo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMarkerAnalysisClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub readiness: RuntimeMarkerAnalysisReadiness,
    pub invalidation_state: RuntimeMarkerAnalysisInvalidationState,
    pub warp_marker_count: usize,
    pub transient_anchor_count: usize,
    pub tempo_assist_posture: RuntimeTempoAssistPosture,
    pub tempo_assist_hint_bpm: Option<f64>,
    pub tempo_assist_hint_source: RuntimeTempoAssistHintSource,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMarkerAnalysisSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_media_clip_count: usize,
    pub degraded_clip_count: usize,
    pub invalidated_clip_count: usize,
    pub unsupported_clip_count: usize,
    pub tempo_assist_ready_clip_count: usize,
    pub warp_marker_count: usize,
    pub transient_anchor_count: usize,
    pub clips: Vec<RuntimeMarkerAnalysisClipSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipRenderInputStage {
    RawClip,
    #[default]
    PostWarp,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderRequest {
    pub clip_id: String,
    pub timeline_start_samples: i64,
    pub input_stage: RuntimeClipRenderInputStage,
    pub buffer: AudioBuffer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderResult {
    pub clip_id: String,
    pub timeline_start_samples: i64,
    pub timeline_end_samples: i64,
    pub input_stage: RuntimeClipRenderInputStage,
    pub clip_processing_snapshot: RuntimeClipProcessingSnapshot,
    pub stretch_engine_snapshot: RuntimeStretchClipSnapshot,
    pub transform_artifact_snapshot: RuntimeTransformArtifactClipSnapshot,
    pub preview_transform_snapshot: RuntimePreviewTransformClipSnapshot,
    pub first_frame_gain: Option<f32>,
    pub last_frame_gain: Option<f32>,
    pub peak_applied_gain: Option<f32>,
    pub output: AudioBuffer,
    pub summary: String,
}
