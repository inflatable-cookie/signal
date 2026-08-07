//! Offline stretch-artifact control-side types.

use signal_dsp_stretch::{
    OfflineHighQualityPath, StretchBackendTier, StretchCacheIdentity, StretchOfflineChunkPlan,
    StretchPromotionReceipt,
};

use crate::RenderSampleBuffer;

/// Offline destination that may consume a cacheable stretch artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineStretchArtifactScope {
    /// Final exported render output.
    Export,
    /// Frozen track or clip output.
    Freeze,
    /// Internal post-warp render cache reuse.
    RenderCache,
}

/// Promotion/readiness posture for a planned offline stretch artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineStretchArtifactReadiness {
    /// The artifact has a stable identity, but the tier is not implemented yet.
    AwaitingImplementation,
    /// The tier exists, but corpus evidence or prototype promotion has not
    /// accepted product-facing use.
    AwaitingCorpusEvidence,
    /// The plan identity is valid, but this artifact shape is not supported
    /// by the current materialization surface.
    UnsupportedCapability,
    /// The artifact may be consumed by render/export/freeze callers.
    Ready,
}

/// Product capability status for a planned offline stretch artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineStretchArtifactCapabilityStatus {
    /// The current materialization surface can build this artifact shape.
    Supported,
    /// The current artifact path only supports linked stereo PCM.
    UnsupportedChannelLayout {
        /// Channel count declared by the stretch cache identity.
        channels: u16,
    },
    /// The current artifact path supports one static pitch shift, not pitch
    /// automation.
    UnsupportedPitchAutomation,
    /// Selector paths are currently limited to static-ratio materialization.
    UnsupportedOfflinePathDynamicRatio {
        /// Requested offline high-quality renderer path.
        path: OfflineHighQualityPath,
    },
    /// Selector paths are currently limited to unshifted static-ratio
    /// materialization.
    UnsupportedOfflinePathPitchShift {
        /// Requested offline high-quality renderer path.
        path: OfflineHighQualityPath,
    },
}

/// Control-side plan for a cacheable offline stretch artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineStretchArtifactPlan {
    /// Consumer scope this artifact would serve.
    pub scope: OfflineStretchArtifactScope,
    /// Validated cache identity for this artifact candidate.
    pub identity: StretchCacheIdentity,
    /// Signal stretch tier named by the identity input.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path named by the identity input.
    pub offline_path: OfflineHighQualityPath,
    /// Current implementation/evidence readiness.
    pub readiness: OfflineStretchArtifactReadiness,
    /// Current product capability support for this artifact shape.
    pub capability_status: OfflineStretchArtifactCapabilityStatus,
    /// Promotion evidence associated with this artifact plan.
    pub promotion_receipt: StretchPromotionReceipt,
    /// Whether the artifact is allowed to feed product-facing render/export output.
    pub product_facing_allowed: bool,
}

/// Receipt-owned offline stretch artifact materialization request.
#[derive(Clone, Debug)]
pub struct OfflineStretchArtifactBuildRequest<'a> {
    /// Consumer scope this artifact would serve.
    pub scope: OfflineStretchArtifactScope,
    /// Cache identity input for the artifact candidate.
    pub identity_input: &'a signal_dsp_stretch::StretchCacheIdentityInput,
    /// Explicit promotion evidence evaluated before product-facing output.
    pub promotion_receipt: StretchPromotionReceipt,
    /// Decoded source buffer to stretch into a cacheable render source.
    pub source: &'a RenderSampleBuffer,
}

/// Materialized offline stretch artifact PCM.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineStretchArtifactPcm {
    /// Readiness and cache identity used to produce this artifact.
    pub plan: OfflineStretchArtifactPlan,
    /// Materialization receipt for cache/export/freeze bookkeeping.
    pub receipt: OfflineStretchArtifactMaterializationReceipt,
    /// Cacheable interleaved stereo PCM that render/export/freeze consumers can
    /// feed back through [`crate::RenderSource::Samples`].
    pub buffer: RenderSampleBuffer,
    /// Deterministic chunk plan used to materialize the artifact.
    pub chunk_plan: StretchOfflineChunkPlan,
    /// Source frame count consumed from `source`.
    pub input_frame_count: usize,
    /// Output frame count produced in `buffer`.
    pub output_frame_count: usize,
}

/// Policy-gated stretch artifact packaged for direct render-plan consumption.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineStretchArtifactRenderSource {
    /// Materialized artifact metadata and PCM.
    pub artifact: OfflineStretchArtifactPcm,
    /// Ready render source wrapping the artifact buffer as
    /// [`crate::RenderSource::Samples`].
    pub source: crate::RenderSource,
}

/// Cache write/read handoff for a promotion-gated render-cache stretch artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineStretchArtifactCacheHandoff {
    /// Stable cache identity hash for lookup/write decisions.
    pub cache_identity_hash: String,
    /// Canonical cache identity key for cache diagnostics and receipts.
    pub cache_identity_key: String,
    /// Materialization receipt produced with the same cache identity and source.
    pub receipt: OfflineStretchArtifactMaterializationReceipt,
    /// Ready render source wrapping the cacheable artifact PCM.
    pub source: crate::RenderSource,
}

/// Outcome kind for a render-cache bridge lookup/write decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineStretchArtifactCacheDecisionKind {
    /// A matching cache identity already existed and was reused.
    Hit,
    /// No matching cache identity existed, so a new handoff was written.
    Written,
    /// A retained cache identity was invalidated.
    Invalidated,
}

/// Render-cache bridge decision for a promotion-gated stretch artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineStretchArtifactCacheDecision {
    /// Whether the bridge reused an existing handoff or wrote a new one.
    pub kind: OfflineStretchArtifactCacheDecisionKind,
    /// Cache identity, render source, and receipt selected by this decision.
    pub handoff: OfflineStretchArtifactCacheHandoff,
}

/// Control-side render-cache bridge for promotion-gated stretch artifacts.
#[derive(Debug, Clone, Default)]
pub struct OfflineStretchArtifactRenderCacheBridge {
    pub(crate) handoffs_by_hash:
        std::collections::HashMap<String, OfflineStretchArtifactCacheHandoff>,
}

/// Receipt for one materialized offline stretch artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineStretchArtifactMaterializationReceipt {
    /// Consumer scope this artifact was materialized for.
    pub scope: OfflineStretchArtifactScope,
    /// Signal stretch tier used to produce the artifact.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used to produce the artifact.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash for the materialized artifact.
    pub cache_identity_hash: String,
    /// Canonical cache identity key for the materialized artifact.
    pub cache_identity_key: String,
    /// Accepted promotion evidence used for product-facing materialization.
    pub promotion_evidence_id: String,
    /// Source frame count consumed from the decoded source buffer.
    pub input_frame_count: usize,
    /// Output frame count produced for cache/export/freeze consumption.
    pub output_frame_count: usize,
    /// Output channel count.
    pub channels: u16,
    /// Output sample rate.
    pub sample_rate_hz: u32,
    /// Number of planned offline stretch chunks.
    pub chunk_count: usize,
    /// Maximum non-overlap source payload frames allowed per chunk.
    pub max_chunk_source_frames: usize,
    /// Source overlap context requested around each chunk payload.
    pub chunk_overlap_frames: usize,
    /// Largest source render span requested by any chunk, including context.
    pub max_chunk_render_source_frames: usize,
    /// Whether this materialized artifact may feed product-facing output.
    pub product_facing_allowed: bool,
}

/// Control-side failure while planning a stretch artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineStretchArtifactPlanError {
    /// The cache identity input was invalid.
    InvalidIdentity(signal_dsp_stretch::StretchCacheIdentityError),
    /// Render/export artifacts must use the high-quality offline tier.
    UnsupportedTier(StretchBackendTier),
}

/// Control-side failure while materializing an offline stretch artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineStretchArtifactMaterializeError {
    /// Artifact planning failed before rendering could begin.
    Plan(OfflineStretchArtifactPlanError),
    /// The plan did not satisfy the accepted promotion gate.
    NotReady(OfflineStretchArtifactReadiness),
    /// The current render-plane PCM artifact path only accepts stereo media.
    UnsupportedChannelLayout {
        /// Channel count declared by the stretch cache identity.
        channels: u16,
    },
    /// The source buffer's sample rate did not match the cache identity.
    SourceSampleRateMismatch {
        /// Sample rate declared in the cache identity.
        expected_hz: u32,
        /// Sample rate on the source buffer.
        actual_hz: u32,
    },
    /// Non-static pitch automation is not materialized by this artifact path.
    UnsupportedPitchAutomation,
    /// The requested offline path is not implemented for dynamic ratio curves
    /// in artifact materialization.
    UnsupportedOfflinePathDynamicRatio {
        /// Requested offline high-quality renderer path.
        path: OfflineHighQualityPath,
    },
    /// The requested offline path is not implemented for static pitch shift in
    /// artifact materialization.
    UnsupportedOfflinePathPitchShift {
        /// Requested offline high-quality renderer path.
        path: OfflineHighQualityPath,
    },
    /// Render-cache handoff helpers only accept render-cache scoped artifacts.
    UnsupportedCacheHandoffScope {
        /// Scope supplied to the render-cache handoff helper.
        scope: OfflineStretchArtifactScope,
    },
}
