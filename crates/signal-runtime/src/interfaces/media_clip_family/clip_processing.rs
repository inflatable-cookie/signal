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
    /// Human-readable summary of this clip's processing state.
    pub summary: String,
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
    /// Human-readable summary of the pipeline state.
    pub summary: String,
}

/// Class of time-stretch engine active for a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchEngineClass {
    /// Stretch engine is not in use for this clip.
    Disabled,
    /// Only a ratio multiplier is applied; no sample-domain stretching.
    RatioOnly,
    /// Full sample-domain time-stretch processing.
    SampleDomain,
    /// Fallback processing path is active.
    Fallback,
}

/// Readiness of the stretch engine for a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchReadiness {
    /// Stretch engine is disabled for this clip.
    Disabled,
    /// Stretch engine is ready.
    Ready,
    /// Waiting for the backing media asset.
    PendingMedia,
    /// Waiting for the warp pipeline.
    PendingWarp,
    /// Stretch engine is in a degraded state.
    Degraded,
}

/// Fallback mode used when the preferred stretch engine is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStretchFallbackKind {
    /// No fallback; engine is operating normally.
    None,
    /// Stretch is bypassed entirely.
    Bypass,
    /// Falling back to ratio-only mode.
    RatioOnly,
}

/// Stretch engine snapshot for a single clip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStretchClipSnapshot {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Class of stretch engine active for this clip.
    pub engine_class: RuntimeStretchEngineClass,
    /// Current readiness of the stretch engine for this clip.
    pub readiness: RuntimeStretchReadiness,
    /// Fallback mode in use if the preferred engine is unavailable.
    pub fallback_kind: RuntimeStretchFallbackKind,
    /// Human-readable summary of this clip's stretch state.
    pub summary: String,
}

/// Aggregate stretch engine snapshot across all clips.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeStretchEngineSnapshot {
    /// Total number of clips tracked by the stretch engine.
    pub clip_count: usize,
    /// Number of clips with stretch disabled.
    pub disabled_clip_count: usize,
    /// Number of clips with stretch ready.
    pub ready_clip_count: usize,
    /// Number of clips waiting for media.
    pub pending_media_clip_count: usize,
    /// Number of clips waiting for warp.
    pub pending_warp_clip_count: usize,
    /// Number of clips in a degraded stretch state.
    pub degraded_clip_count: usize,
    /// Number of clips using sample-domain stretching.
    pub sample_domain_clip_count: usize,
    /// Number of clips using ratio-only mode.
    pub ratio_only_clip_count: usize,
    /// Number of clips using the fallback path.
    pub fallback_clip_count: usize,
    /// Per-clip stretch snapshots.
    pub clips: Vec<RuntimeStretchClipSnapshot>,
    /// Human-readable summary of the stretch engine state.
    pub summary: String,
}

/// Readiness of warp marker / transient analysis for a clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMarkerAnalysisReadiness {
    #[default]
    /// No analysis data; clip has not been submitted.
    Empty,
    /// Waiting for the backing media asset.
    PendingMedia,
    /// Analysis is complete and ready.
    Ready,
    /// Analysis completed in a degraded state.
    Degraded,
    /// Previously valid analysis has been invalidated and needs refresh.
    Invalidated,
    /// This clip type does not support marker analysis.
    Unsupported,
}

/// Reason a clip's marker analysis was invalidated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMarkerAnalysisInvalidationState {
    #[default]
    /// No invalidation; analysis is current.
    None,
    /// The backing media asset changed.
    MediaInvalidated,
    /// The stretch parameters changed.
    StretchInvalidated,
}

/// Availability of tempo-assist hints derived from marker analysis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoAssistPosture {
    #[default]
    /// Tempo-assist hints are not available for this clip.
    Unavailable,
    /// Hints are available but may be unreliable.
    Guarded,
    /// Hints are available and reliable.
    Ready,
}

/// Source of the tempo hint used by tempo assist.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTempoAssistHintSource {
    #[default]
    /// No hint source available.
    None,
    /// Hint derived from the clip's embedded source tempo.
    SourceTempo,
    /// Hint derived from the project tempo map.
    ProjectTempo,
}

/// Marker and transient analysis snapshot for one clip.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMarkerAnalysisClipSnapshot {
    /// Unique identifier for the clip.
    pub clip_id: String,
    /// ID of the media asset backing this clip, if any.
    pub media_asset_id: Option<String>,
    /// Current readiness of marker analysis for this clip.
    pub readiness: RuntimeMarkerAnalysisReadiness,
    /// Invalidation state of the marker analysis, if any.
    pub invalidation_state: RuntimeMarkerAnalysisInvalidationState,
    /// Number of warp markers present in the analysis.
    pub warp_marker_count: usize,
    /// Number of transient anchors present in the analysis.
    pub transient_anchor_count: usize,
    /// Tempo-assist hint availability for this clip.
    pub tempo_assist_posture: RuntimeTempoAssistPosture,
    /// Tempo-assist hint value in BPM, if available.
    pub tempo_assist_hint_bpm: Option<f64>,
    /// Source from which the tempo-assist hint was derived.
    pub tempo_assist_hint_source: RuntimeTempoAssistHintSource,
    /// Last error message if marker analysis encountered a problem, if any.
    pub last_error: Option<String>,
    /// Human-readable summary of this clip's marker analysis state.
    pub summary: String,
}

/// Aggregate marker analysis snapshot across all clips.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMarkerAnalysisSnapshot {
    /// Total number of clips tracked by marker analysis.
    pub clip_count: usize,
    /// Number of clips with analysis ready.
    pub ready_clip_count: usize,
    /// Number of clips waiting for media.
    pub pending_media_clip_count: usize,
    /// Number of clips with degraded analysis.
    pub degraded_clip_count: usize,
    /// Number of clips with invalidated analysis.
    pub invalidated_clip_count: usize,
    /// Number of clips for which analysis is unsupported.
    pub unsupported_clip_count: usize,
    /// Number of clips with tempo-assist hints ready.
    pub tempo_assist_ready_clip_count: usize,
    /// Total warp marker count across all clips.
    pub warp_marker_count: usize,
    /// Total transient anchor count across all clips.
    pub transient_anchor_count: usize,
    /// Per-clip marker analysis snapshots.
    pub clips: Vec<RuntimeMarkerAnalysisClipSnapshot>,
    /// Human-readable summary of the marker analysis state.
    pub summary: String,
}

/// Input stage from which a clip render is sourced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeClipRenderInputStage {
    /// Source audio before any warp processing.
    RawClip,
    #[default]
    /// Source audio after warp processing.
    PostWarp,
}

/// Request to render one clip into a provided audio buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderRequest {
    /// Unique identifier for the clip to render.
    pub clip_id: String,
    /// Timeline start position for the render in samples.
    pub timeline_start_samples: i64,
    /// Stage of the clip processing pipeline to use as the render source.
    pub input_stage: RuntimeClipRenderInputStage,
    /// Audio buffer to fill with the rendered output.
    pub buffer: AudioBuffer,
}

/// Result of a clip render: processed audio buffer with gain/peak metadata
/// and full clip/stretch/transform snapshots at the time of render.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeClipRenderResult {
    /// Unique identifier for the rendered clip.
    pub clip_id: String,
    /// Timeline start position of the rendered range in samples.
    pub timeline_start_samples: i64,
    /// Timeline end position of the rendered range in samples.
    pub timeline_end_samples: i64,
    /// Input stage used as the render source.
    pub input_stage: RuntimeClipRenderInputStage,
    /// Processing snapshot for this clip at render time.
    pub clip_processing_snapshot: RuntimeClipProcessingSnapshot,
    /// Stretch engine snapshot for this clip at render time.
    pub stretch_engine_snapshot: RuntimeStretchClipSnapshot,
    /// Transform artifact snapshot for this clip at render time.
    pub transform_artifact_snapshot: RuntimeTransformArtifactClipSnapshot,
    /// Preview transform snapshot for this clip at render time.
    pub preview_transform_snapshot: RuntimePreviewTransformClipSnapshot,
    /// Gain applied at the first rendered frame, if applicable.
    pub first_frame_gain: Option<f32>,
    /// Gain applied at the last rendered frame, if applicable.
    pub last_frame_gain: Option<f32>,
    /// Peak gain applied across all rendered frames, if applicable.
    pub peak_applied_gain: Option<f32>,
    /// Rendered audio output buffer.
    pub output: AudioBuffer,
    /// Human-readable summary of the render result.
    pub summary: String,
}
