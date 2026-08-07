//! Offline (faster-than-realtime) bounce driver.
//!
//! WYSIWYG export: drives the SAME [`RenderPlaneExecutor`] over the SAME
//! compiled [`RenderPlanSpec`] that realtime playback uses — no parallel
//! render path, no resampling shortcut. The only intentional divergence is
//! the transport edge envelope: realtime ramps in over ~5 ms at play so the
//! speaker never steps, but a bounce must start at full level, so the driver
//! snaps the envelope open before the first block (see
//! [`RenderPlaneExecutor::set_edge_gain_immediate`]). Everything else —
//! stage scheduling, matrices, gain smoothing, automation envelopes, declick
//! fades, the master limiter, the hardware-boundary write — is the realtime
//! code, byte for byte.

mod bounce;
mod stretch_artifact;
mod wav;

pub use bounce::{apply_soft_limiter_to_pcm, render_plan_to_pcm};
pub use stretch_artifact::{
    build_offline_stretch_artifact_cache_handoff, build_offline_stretch_artifact_render_source,
    materialize_offline_stretch_artifact_pcm, plan_offline_stretch_artifact,
    OfflineStretchArtifactBuildRequest, OfflineStretchArtifactCacheDecision,
    OfflineStretchArtifactCacheDecisionKind, OfflineStretchArtifactCacheHandoff,
    OfflineStretchArtifactMaterializationReceipt, OfflineStretchArtifactMaterializeError,
    OfflineStretchArtifactPcm, OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError,
    OfflineStretchArtifactReadiness, OfflineStretchArtifactRenderCacheBridge,
    OfflineStretchArtifactRenderSource, OfflineStretchArtifactScope,
};
pub use wav::{write_wav, WavBitDepth};

/// Default block quantum for offline rendering.
const DEFAULT_BLOCK_FRAMES: usize = 1024;

/// Options for one offline render pass.
#[derive(Debug, Clone)]
pub struct OfflineRenderOptions {
    /// First stream-clock frame to render (the bounce range start).
    pub start_frame: u64,
    /// Number of frames to render.
    pub frame_count: u64,
    /// Block quantum the executor is driven at; clamped to
    /// `1..=`[`MAX_BLOCK_FRAMES`]. Smaller blocks tighten automation/gain
    /// ramp granularity exactly as they would in realtime.
    pub block_frames: usize,
    /// Stage ids whose post-fader output is captured as stems alongside the
    /// master. Each stem is interleaved at that stage's own channel format.
    pub capture_stage_ids: Vec<u64>,
}

impl Default for OfflineRenderOptions {
    fn default() -> Self {
        OfflineRenderOptions {
            start_frame: 0,
            frame_count: 0,
            block_frames: DEFAULT_BLOCK_FRAMES,
            capture_stage_ids: Vec::new(),
        }
    }
}

/// Result of an offline render: interleaved f32 PCM plus optional stems.
#[derive(Debug, Clone)]
pub struct OfflineRenderOutput {
    /// Interleaved master PCM at the plan's master channel count.
    pub master: Vec<f32>,
    /// Channel count of `master` (the plan's master stage format).
    pub channels: u16,
    /// Sample rate the plan rendered at.
    pub sample_rate_hz: u32,
    /// Captured stems: `(stage_id, interleaved post-fader PCM)`, in the
    /// order of [`OfflineRenderOptions::capture_stage_ids`]. Each stem is
    /// interleaved at its stage's own channel count.
    pub stems: Vec<(u64, Vec<f32>)>,
}

#[cfg(test)]
mod tests;
