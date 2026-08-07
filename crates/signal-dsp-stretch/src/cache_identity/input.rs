use crate::{OfflineHighQualityPath, StretchBackendTier, StretchOfflineChunkConfig};

use super::identity::{StretchCacheIdentity, StretchCacheIdentityError};
use super::types::{
    StretchChannelLayout, StretchPitchPoint, StretchRatioPoint, StretchRenderGeometry,
    StretchWarpMarker, SIGNAL_STRETCH_ENGINE_VERSION,
};

/// Inputs that define one cacheable Signal stretch artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCacheIdentityInput {
    /// Signal stretch engine version.
    pub engine_version: String,
    /// Stretch tier that produced the artifact.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used for the artifact.
    pub offline_path: OfflineHighQualityPath,
    /// Content hash of the decoded source media identity.
    pub source_content_hash: String,
    /// Source channel layout.
    pub channel_layout: StretchChannelLayout,
    /// Ratio curve sampled from the canonical tick/sample projection.
    pub ratio_curve: Vec<StretchRatioPoint>,
    /// Independent pitch curve sampled from the canonical tick/sample projection.
    pub pitch_curve: Vec<StretchPitchPoint>,
    /// Warp markers included in this artifact.
    pub warp_markers: Vec<StretchWarpMarker>,
    /// Projection epoch for the ADR-001 tick/sample mapping used by this render.
    pub projection_epoch: String,
    /// STFT geometry the render used.
    pub render_geometry: StretchRenderGeometry,
    /// Bounded-memory chunk policy the render used.
    ///
    /// Chunk boundaries move where segment renders restart phase, so two
    /// chunk policies produce different audio from one source. Measured at
    /// correlation `-0.296620` between a single-chunk and an eight-chunk render
    /// of the same identity.
    pub chunk_policy: StretchOfflineChunkConfig,
}

impl StretchCacheIdentityInput {
    /// Construct an input using the current Signal stretch engine version.
    pub fn signal_native(
        tier: StretchBackendTier,
        source_content_hash: impl Into<String>,
        channel_layout: StretchChannelLayout,
        projection_epoch: impl Into<String>,
    ) -> Self {
        Self {
            engine_version: SIGNAL_STRETCH_ENGINE_VERSION.to_string(),
            tier,
            offline_path: OfflineHighQualityPath::Default,
            source_content_hash: source_content_hash.into(),
            channel_layout,
            ratio_curve: Vec::new(),
            pitch_curve: Vec::new(),
            warp_markers: Vec::new(),
            projection_epoch: projection_epoch.into(),
            render_geometry: StretchRenderGeometry::default(),
            chunk_policy: StretchOfflineChunkConfig::default(),
        }
    }

    /// Set the offline high-quality renderer path.
    pub fn with_offline_path(mut self, offline_path: OfflineHighQualityPath) -> Self {
        self.offline_path = offline_path;
        self
    }

    /// Set the STFT geometry the render used.
    pub fn with_render_geometry(mut self, render_geometry: StretchRenderGeometry) -> Self {
        self.render_geometry = render_geometry;
        self
    }

    /// Set the bounded-memory chunk policy the render used.
    pub fn with_chunk_policy(mut self, chunk_policy: StretchOfflineChunkConfig) -> Self {
        self.chunk_policy = chunk_policy;
        self
    }

    /// Set the ratio curve.
    pub fn with_ratio_curve(mut self, ratio_curve: Vec<StretchRatioPoint>) -> Self {
        self.ratio_curve = ratio_curve;
        self
    }

    /// Set the pitch curve.
    pub fn with_pitch_curve(mut self, pitch_curve: Vec<StretchPitchPoint>) -> Self {
        self.pitch_curve = pitch_curve;
        self
    }

    /// Set the warp markers.
    pub fn with_warp_markers(mut self, warp_markers: Vec<StretchWarpMarker>) -> Self {
        self.warp_markers = warp_markers;
        self
    }

    /// Validate and materialize a stable cache identity.
    pub fn identity(&self) -> Result<StretchCacheIdentity, StretchCacheIdentityError> {
        StretchCacheIdentity::from_input(self)
    }
}
