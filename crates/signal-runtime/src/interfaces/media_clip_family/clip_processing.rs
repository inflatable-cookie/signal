use super::*;

/// Readiness of a clip for audio processing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingReadiness {
    /// Clip is ready to render.
    Ready,
    /// Waiting for the backing media asset to become available.
    PendingMedia,
    /// Waiting for the warp pipeline to become ready.
    PendingWarp,
    /// Clip is in an unrecoverable error state.
    Invalid,
}

/// Curve shape for a clip fade envelope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipFadeShape {
    #[default]
    /// Straight-line fade.
    Linear,
    /// Equal-power fade suited for crossfades.
    EqualPower,
    /// S-curve fade with smooth acceleration and deceleration.
    SmoothStep,
}

/// Interpolation shape for a clip gain envelope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipGainShape {
    #[default]
    /// Gain holds constant at `start_linear` for the full clip duration.
    Hold,
    /// Gain ramps linearly from `start_linear` to `end_linear`.
    Linear,
}

/// Processing treatment stage applied to a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClipProcessingStage {
    /// Time-stretch/warp processing stage.
    Warp,
    /// Fade-in envelope stage.
    FadeIn,
    /// Gain envelope shaping stage.
    GainShape,
    /// Fade-out envelope stage.
    FadeOut,
}

/// Duration and shape of a clip fade-in or fade-out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipFadeEnvelope {
    /// Duration of the fade in samples.
    pub duration_samples: u32,
    /// Curve shape of the fade.
    pub shape: RuntimeClipFadeShape,
}

/// Gain curve from start to end of a clip.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipGainEnvelope {
    /// Gain at the start of the clip in linear scale.
    pub start_linear: f32,
    /// Gain at the end of the clip in linear scale.
    pub end_linear: f32,
    /// Interpolation shape between start and end gain.
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

/// Registration parameters for a clip in the clip processing pipeline.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingRegistration {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Warp mode to apply.
    pub warp_mode: RuntimeWarpMode,
    /// Timeline start position of the clip in samples.
    pub start_samples: i64,
    /// Duration of the clip in samples.
    pub duration_samples: u32,
    /// Fade-in envelope parameters.
    pub fade_in: RuntimeClipFadeEnvelope,
    /// Fade-out envelope parameters.
    pub fade_out: RuntimeClipFadeEnvelope,
    /// Gain envelope parameters.
    pub clip_gain: RuntimeClipGainEnvelope,
}

/// Full processing snapshot for one clip: warp ratio, fade/gain envelopes,
/// treatment stages, and readiness.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipProcessingSnapshot {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Warp mode applied to this clip.
    pub warp_mode: RuntimeWarpMode,
    /// Timeline start position of the clip in samples.
    pub start_samples: i64,
    /// Duration of the clip in samples.
    pub duration_samples: u32,
    /// Fade-in envelope parameters.
    pub fade_in: RuntimeClipFadeEnvelope,
    /// Fade-out envelope parameters.
    pub fade_out: RuntimeClipFadeEnvelope,
    /// Sample position where the fade-in envelope ends.
    pub fade_in_end_samples: i64,
    /// Sample position where the fade-out envelope begins.
    pub fade_out_start_samples: i64,
    /// Gain envelope parameters.
    pub clip_gain: RuntimeClipGainEnvelope,
    /// Active treatment stages applied to this clip.
    pub treatment_stages: Vec<RuntimeClipProcessingStage>,
    /// The realized warp ratio, if warp is active.
    pub realized_warp_ratio: Option<f64>,
    /// Source from which the project tempo was derived for warp, if applicable.
    pub project_tempo_source: Option<RuntimeTempoSource>,
    /// ID of the tempo map segment used for warp, if applicable.
    pub project_tempo_segment_id: Option<String>,
    /// Current processing readiness of this clip.
    pub readiness: RuntimeClipProcessingReadiness,
    /// Last error message if the clip processing pipeline encountered a problem, if any.
    pub last_error: Option<String>,
}

/// Aggregate snapshot of the clip processing pipeline: counts by readiness
/// and treatment stage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeClipProcessingPipelineSnapshot {
    /// Total number of clips registered.
    pub clip_count: usize,
    /// Number of clips in the ready state.
    pub ready_clip_count: usize,
    /// Number of clips waiting for media.
    pub pending_media_clip_count: usize,
    /// Number of clips waiting for warp.
    pub pending_warp_clip_count: usize,
    /// Number of clips in the invalid state.
    pub invalid_clip_count: usize,
    /// Number of clips with an active fade treatment stage.
    pub faded_clip_count: usize,
    /// Number of clips with an active gain-shape treatment stage.
    pub gain_shaped_clip_count: usize,
    /// Number of clips with an active warp treatment stage.
    pub warped_clip_count: usize,
    /// Total number of active treatment stages across all clips.
    pub treatment_stage_count: usize,
    /// Per-clip processing snapshots.
    pub clips: Vec<RuntimeClipProcessingSnapshot>,
}
