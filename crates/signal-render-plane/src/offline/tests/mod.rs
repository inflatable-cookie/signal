//! Offline render-plane unit tests (split by topic).

pub(super) use super::stretch_artifact::{
    materialize_offline_stretch_artifact_pcm_with_chunk_config,
    OfflineStretchArtifactCapabilityStatus,
};
pub(super) use super::*;
pub(super) use crate::{
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderLimiterSpec,
    RenderParamEnvelope, RenderPlanSpec, RenderPluginProcessor, RenderSampleBuffer, RenderSource,
    RenderStageKind, RenderStageSpec,
};
pub(super) use signal_dsp_stretch::{
    OfflineHighQualityPath, StretchBackendTier, StretchCacheIdentityInput, StretchChannelLayout,
    StretchOfflineChunkConfig, StretchPitchPoint, StretchProductQualityEvidence,
    StretchPromotionReceipt, StretchPromotionStatus, StretchRatioPoint, StretchWarpMarker,
    REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
};
pub(super) use std::sync::Arc;

mod support;

mod bounce_wav;
mod cache_bridge;
mod limiter;
mod offline_render_parity;
mod param_envelopes;
mod promotion;
mod selectors;
mod stretch_artifact;
