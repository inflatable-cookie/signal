//! Offline stretch-artifact planning and materialization.

use std::collections::HashMap;
use std::sync::Arc;

use signal_dsp_stretch::{
    plan_offline_stretch_chunks, stretch_backend_plan, OfflineHighQualityPath,
    OfflineHighQualityStretcher, ResumableOfflineStretch, ResumableStretchConfig,
    StretchBackendStatus, StretchBackendTier, StretchCacheIdentity, StretchCacheIdentityError,
    StretchCacheIdentityInput, StretchOfflineChunkConfig, StretchOfflineChunkPlan,
    StretchPromotionReceipt, StretchRatioPoint, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
};
use signal_primitives::SampleRate;

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
    pub identity_input: &'a StretchCacheIdentityInput,
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
    handoffs_by_hash: HashMap<String, OfflineStretchArtifactCacheHandoff>,
}

impl OfflineStretchArtifactRenderCacheBridge {
    /// Create an empty render-cache bridge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of cache handoffs currently retained by this bridge.
    pub fn len(&self) -> usize {
        self.handoffs_by_hash.len()
    }

    /// Whether the bridge has no retained cache handoffs.
    pub fn is_empty(&self) -> bool {
        self.handoffs_by_hash.is_empty()
    }

    /// Return true when a stable cache identity hash is retained.
    pub fn contains_identity_hash(&self, cache_identity_hash: &str) -> bool {
        self.handoffs_by_hash.contains_key(cache_identity_hash)
    }

    /// Remove one retained cache handoff by stable identity hash.
    pub fn invalidate_identity_hash(
        &mut self,
        cache_identity_hash: &str,
    ) -> Option<OfflineStretchArtifactCacheHandoff> {
        self.invalidate_identity_hash_with_decision(cache_identity_hash)
            .map(|decision| decision.handoff)
    }

    /// Remove one retained cache handoff and return an invalidation decision.
    pub fn invalidate_identity_hash_with_decision(
        &mut self,
        cache_identity_hash: &str,
    ) -> Option<OfflineStretchArtifactCacheDecision> {
        self.handoffs_by_hash
            .remove(cache_identity_hash)
            .map(|handoff| OfflineStretchArtifactCacheDecision {
                kind: OfflineStretchArtifactCacheDecisionKind::Invalidated,
                handoff,
            })
    }

    /// Resolve a promotion-gated render-cache request against retained handoffs.
    ///
    /// Incomplete promotion evidence cannot write a new product-facing
    /// handoff. A miss returns
    /// [`OfflineStretchArtifactMaterializeError::NotReady`] and writes nothing.
    pub fn resolve(
        &mut self,
        request: OfflineStretchArtifactBuildRequest<'_>,
    ) -> Result<OfflineStretchArtifactCacheDecision, OfflineStretchArtifactMaterializeError> {
        let identity = request
            .identity_input
            .identity()
            .map_err(OfflineStretchArtifactPlanError::InvalidIdentity)?;
        if let Some(handoff) = self.handoffs_by_hash.get(&identity.stable_hash) {
            return Ok(OfflineStretchArtifactCacheDecision {
                kind: OfflineStretchArtifactCacheDecisionKind::Hit,
                handoff: handoff.clone(),
            });
        }

        let handoff = build_offline_stretch_artifact_cache_handoff(request)?;
        self.handoffs_by_hash
            .insert(handoff.cache_identity_hash.clone(), handoff.clone());
        Ok(OfflineStretchArtifactCacheDecision {
            kind: OfflineStretchArtifactCacheDecisionKind::Written,
            handoff,
        })
    }
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
    InvalidIdentity(StretchCacheIdentityError),
    /// Render/export artifacts must use the high-quality offline tier.
    UnsupportedTier(StretchBackendTier),
}

impl std::fmt::Display for OfflineStretchArtifactPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineStretchArtifactPlanError::InvalidIdentity(error) => {
                write!(formatter, "invalid stretch cache identity: {error:?}")
            }
            OfflineStretchArtifactPlanError::UnsupportedTier(tier) => write!(
                formatter,
                "offline stretch artifacts require OfflineHighQuality, got {tier:?}",
            ),
        }
    }
}

impl std::error::Error for OfflineStretchArtifactPlanError {}

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

impl std::fmt::Display for OfflineStretchArtifactMaterializeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineStretchArtifactMaterializeError::Plan(error) => write!(formatter, "{error}"),
            OfflineStretchArtifactMaterializeError::NotReady(readiness) => write!(
                formatter,
                "offline stretch artifact is not product-facing ready: {readiness:?}",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels } => {
                write!(
                    formatter,
                    "offline stretch artifact PCM requires stereo source, got {channels} channels",
                )
            }
            OfflineStretchArtifactMaterializeError::SourceSampleRateMismatch {
                expected_hz,
                actual_hz,
            } => write!(
                formatter,
                "offline stretch artifact source sample rate mismatch: expected {expected_hz}, got {actual_hz}",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation => write!(
                formatter,
                "offline stretch artifact materialization requires static pitch shift",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                path,
            } => write!(
                formatter,
                "offline stretch artifact path {path:?} does not support dynamic ratio materialization yet",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift { path } => {
                write!(
                    formatter,
                    "offline stretch artifact path {path:?} does not support pitch-shift materialization yet",
                )
            }
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope { scope } => {
                write!(
                    formatter,
                    "offline stretch render-cache handoff requires RenderCache scope, got {scope:?}",
                )
            }
        }
    }
}

impl std::error::Error for OfflineStretchArtifactMaterializeError {}

impl From<OfflineStretchArtifactPlanError> for OfflineStretchArtifactMaterializeError {
    fn from(error: OfflineStretchArtifactPlanError) -> Self {
        Self::Plan(error)
    }
}

/// Build a control-side artifact plan for an offline high-quality stretch
/// candidate.
///
/// This function does not render or promote anything. It gives cache/export
/// callers a deterministic identity and a typed answer for why the artifact
/// may or may not feed product-facing output yet.
pub fn plan_offline_stretch_artifact(
    scope: OfflineStretchArtifactScope,
    identity_input: &StretchCacheIdentityInput,
    promotion_receipt: StretchPromotionReceipt,
) -> Result<OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError> {
    if identity_input.tier != StretchBackendTier::OfflineHighQuality {
        return Err(OfflineStretchArtifactPlanError::UnsupportedTier(
            identity_input.tier,
        ));
    }
    let identity = identity_input
        .identity()
        .map_err(OfflineStretchArtifactPlanError::InvalidIdentity)?;
    let backend = stretch_backend_plan(identity_input.tier);
    let capability_status = offline_stretch_artifact_capability_status(identity_input);
    let promotion_accepted = promotion_receipt
        .accepts_product_facing_path(identity_input.tier, identity_input.offline_path);
    let readiness = match (backend.status, promotion_accepted, capability_status) {
        (StretchBackendStatus::Planned, _, _) => {
            OfflineStretchArtifactReadiness::AwaitingImplementation
        }
        (StretchBackendStatus::Prototype, _, _) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (StretchBackendStatus::Implemented, false, _) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (
            StretchBackendStatus::Implemented,
            true,
            OfflineStretchArtifactCapabilityStatus::Supported,
        ) => OfflineStretchArtifactReadiness::Ready,
        (StretchBackendStatus::Implemented, true, _) => {
            OfflineStretchArtifactReadiness::UnsupportedCapability
        }
    };

    Ok(OfflineStretchArtifactPlan {
        scope,
        identity,
        tier: identity_input.tier,
        offline_path: identity_input.offline_path,
        readiness,
        capability_status,
        promotion_receipt,
        product_facing_allowed: readiness == OfflineStretchArtifactReadiness::Ready,
    })
}

/// Materialize a ready OfflineHighQuality stretch artifact as interleaved
/// stereo PCM.
///
/// This is an offline control-side operation. It never runs on the realtime
/// audio thread. The result is a [`RenderSampleBuffer`] so render-cache,
/// freeze, and export callers can consume the artifact through the existing
/// sample-source render path. Product-facing output is refused unless the
/// attached promotion receipt makes the artifact plan
/// [`OfflineStretchArtifactReadiness::Ready`].
///
/// The first materialization slice supports interleaved stereo render-plane
/// media with a dynamic ratio curve and one static pitch shift.
pub fn materialize_offline_stretch_artifact_pcm(
    scope: OfflineStretchArtifactScope,
    identity_input: &StretchCacheIdentityInput,
    promotion_receipt: StretchPromotionReceipt,
    source: &RenderSampleBuffer,
) -> Result<OfflineStretchArtifactPcm, OfflineStretchArtifactMaterializeError> {
    materialize_offline_stretch_artifact_pcm_with_chunk_config(
        scope,
        identity_input,
        promotion_receipt,
        source,
        StretchOfflineChunkConfig::default(),
    )
}

/// Materialize a ready OfflineHighQuality stretch artifact with an explicit
/// chunking policy.
///
/// This is the long-media test and integration entry point. Production callers
/// normally use [`materialize_offline_stretch_artifact_pcm`], which applies the
/// default bounded chunk policy.
pub fn materialize_offline_stretch_artifact_pcm_with_chunk_config(
    scope: OfflineStretchArtifactScope,
    identity_input: &StretchCacheIdentityInput,
    promotion_receipt: StretchPromotionReceipt,
    source: &RenderSampleBuffer,
    chunk_config: StretchOfflineChunkConfig,
) -> Result<OfflineStretchArtifactPcm, OfflineStretchArtifactMaterializeError> {
    // Chunk boundaries move where segment renders restart phase, so the policy
    // this call renders with is part of what the artifact is. Key by the policy
    // actually used rather than whatever the caller left on the identity.
    let identity_input = &identity_input.clone().with_chunk_policy(chunk_config);
    let plan = plan_offline_stretch_artifact(scope, identity_input, promotion_receipt)?;
    if let Some(error) = materialization_error_for_capability(plan.capability_status) {
        return Err(error);
    }
    if plan.readiness != OfflineStretchArtifactReadiness::Ready {
        return Err(OfflineStretchArtifactMaterializeError::NotReady(
            plan.readiness,
        ));
    }
    if identity_input.channel_layout.channels != 2 {
        return Err(
            OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout {
                channels: identity_input.channel_layout.channels,
            },
        );
    }
    if source.sample_rate_hz != identity_input.channel_layout.sample_rate_hz {
        return Err(
            OfflineStretchArtifactMaterializeError::SourceSampleRateMismatch {
                expected_hz: identity_input.channel_layout.sample_rate_hz,
                actual_hz: source.sample_rate_hz,
            },
        );
    }

    let ratio = static_or_initial_ratio(&identity_input.ratio_curve);
    let pitch_shift = static_pitch_shift(identity_input)?;
    if selector_offline_path_requires_static_materialization(identity_input.offline_path) {
        if ratio_curve_has_dynamic_changes(&identity_input.ratio_curve) {
            return Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                    path: identity_input.offline_path,
                },
            );
        }
        if pitch_shift.abs() > 1.0e-9 {
            return Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift {
                    path: identity_input.offline_path,
                },
            );
        }
    }
    let mut stretcher = OfflineHighQualityStretcher::with_path(ratio, identity_input.offline_path);
    let chunk_plan = plan_offline_stretch_chunks(
        source.frame_count(),
        &identity_input.ratio_curve,
        ratio,
        chunk_config,
    );
    let frames =
        if selector_offline_path_requires_static_materialization(identity_input.offline_path) {
            stretcher
                .stretch_interleaved_stereo(&source.frames)
                .expect("render fits the offline output bound")
        } else if resumable_render_supported(identity_input.offline_path, pitch_shift) {
            // Length must not select the algorithm: a single-chunk artifact and
            // a multi-chunk artifact of the same source share a cache key, so
            // they must share a renderer.
            materialize_resumable_offline_stretch_artifact_frames(
                source,
                &identity_input.ratio_curve,
                ratio,
                pitch_shift,
                &chunk_plan,
            )
        } else {
            // Unreachable: `OfflineHighQualityPath` has three variants, the two
            // selectors take the branch above and `Default` takes the resumable
            // renderer. Stated rather than left as a fallback, because the
            // fallback this replaced switched algorithms under one cache key.
            unreachable!(
                "every offline path is either selector-materialized or resumable, saw {:?}",
                identity_input.offline_path
            )
        };

    let output_frame_count = frames.len() / 2;
    let receipt = OfflineStretchArtifactMaterializationReceipt {
        scope,
        tier: plan.tier,
        offline_path: plan.offline_path,
        cache_identity_hash: plan.identity.stable_hash.clone(),
        cache_identity_key: plan.identity.canonical_key.clone(),
        promotion_evidence_id: plan.promotion_receipt.evidence_id.clone(),
        input_frame_count: source.frame_count(),
        output_frame_count,
        channels: identity_input.channel_layout.channels,
        sample_rate_hz: source.sample_rate_hz,
        chunk_count: chunk_plan.chunks.len(),
        max_chunk_source_frames: chunk_plan.config.max_source_frames,
        chunk_overlap_frames: chunk_plan.config.overlap_frames,
        max_chunk_render_source_frames: chunk_plan.max_render_source_frames(),
        product_facing_allowed: plan.product_facing_allowed,
    };
    Ok(OfflineStretchArtifactPcm {
        plan,
        receipt,
        buffer: RenderSampleBuffer::stereo(
            source.sample_rate_hz,
            Arc::from(frames.into_boxed_slice()),
        ),
        chunk_plan,
        input_frame_count: source.frame_count(),
        output_frame_count,
    })
}

/// Build a promotion-gated OfflineHighQuality render source.
pub fn build_offline_stretch_artifact_render_source(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactRenderSource, OfflineStretchArtifactMaterializeError> {
    let artifact = materialize_offline_stretch_artifact_pcm(
        request.scope,
        request.identity_input,
        request.promotion_receipt,
        request.source,
    )?;
    Ok(OfflineStretchArtifactRenderSource {
        source: crate::RenderSource::Samples(artifact.buffer.clone()),
        artifact,
    })
}

/// Build a promotion-gated OfflineHighQuality cache handoff.
///
/// This helper is scoped to [`OfflineStretchArtifactScope::RenderCache`].
pub fn build_offline_stretch_artifact_cache_handoff(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactCacheHandoff, OfflineStretchArtifactMaterializeError> {
    if request.scope != OfflineStretchArtifactScope::RenderCache {
        return Err(
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                scope: request.scope,
            },
        );
    }
    let artifact_source = build_offline_stretch_artifact_render_source(request)?;
    let receipt = artifact_source.artifact.receipt.clone();
    Ok(OfflineStretchArtifactCacheHandoff {
        cache_identity_hash: receipt.cache_identity_hash.clone(),
        cache_identity_key: receipt.cache_identity_key.clone(),
        receipt,
        source: artifact_source.source,
    })
}

/// Whether the resumable renderer can serve this artifact.
///
/// It owns the default offline path with no pitch shift. Selector paths and
/// pitch composition still route through the legacy per-chunk path, which keeps
/// its boundary smoother because it still creates boundaries.
/// Whether the resumable renderer serves this artifact.
///
/// Pitch was admitted by listening on 2026-08-05 (`g10.042`), which removed the
/// last caller of the chunked renderer. Selector paths render whole-buffer and
/// never chunked, so they were never served by it either.
fn resumable_render_supported(offline_path: OfflineHighQualityPath, _pitch_shift: f64) -> bool {
    matches!(offline_path, OfflineHighQualityPath::Default)
}

/// Render the whole artifact through one state-carrying renderer.
///
/// The chunk plan still bounds how much source is in flight; it no longer cuts
/// the render into independent pieces, so there are no joins to patch.
fn materialize_resumable_offline_stretch_artifact_frames(
    source: &RenderSampleBuffer,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    pitch_shift: f64,
    chunk_plan: &StretchOfflineChunkPlan,
) -> Vec<f32> {
    let frame_count = source.frame_count();
    let even_source = &source.frames[..frame_count * 2];
    // Not fallible in practice: the configuration is fixed here and the only
    // rejections are an over-large window or an unsupported channel count.
    // Stated as an expectation rather than an `Option` because the previous
    // shape fell back to the legacy chunked renderer on any error, which would
    // have rendered the same cache key with a different algorithm — the exact
    // invariant the caller's comment asserts, broken by its own safety net.
    let mut renderer = ResumableOfflineStretch::new(ResumableStretchConfig {
        channels: 2,
        window_size: DEFAULT_WINDOW_SIZE,
        analysis_hop: DEFAULT_ANALYSIS_HOP,
        source_frames: frame_count,
        ratio_curve: ratio_curve.to_vec(),
        fallback_ratio,
        sample_rate: SampleRate(source.sample_rate_hz),
        pitch_shift_semitones: pitch_shift,
    })
    .expect("the fixed resumable configuration is supported");

    let mut output = Vec::with_capacity(chunk_plan.total_output_frames * 2);
    for chunk in &chunk_plan.chunks {
        let start = chunk.source_start_frame * 2;
        let end = chunk.source_end_frame * 2;
        // `render` is genuinely fallible: `g10.039` made it return an error
        // rather than discard source when a drain cannot advance. That is a
        // defect to surface, not a reason to switch renderers behind the
        // caller's back.
        renderer
            .render(&even_source[start..end], &mut output)
            .expect("resumable render accepts the planned chunk");
    }
    renderer
        .flush(&mut output)
        .expect("resumable render flushes its tail");

    // The renderer can finish a frame or two short of the planned length
    // through rounding. Padding beyond that would be the `g10.039` failure
    // again, where a silent renderer was zero-filled to its contracted length
    // and nothing downstream noticed.
    let planned = chunk_plan.total_output_frames * 2;
    let shortfall = planned.saturating_sub(output.len());
    assert!(
        shortfall <= 4,
        "resumable render produced {} samples against a planned {planned}; \
         padding that gap would hide a render failure",
        output.len(),
    );
    output.resize(planned, 0.0);
    output
}

fn static_or_initial_ratio(ratio_curve: &[StretchRatioPoint]) -> f64 {
    ratio_curve
        .iter()
        .find(|point| point.ratio.is_finite() && point.ratio > 0.0)
        .map(|point| point.ratio)
        .unwrap_or(1.0)
}

fn selector_offline_path_requires_static_materialization(path: OfflineHighQualityPath) -> bool {
    matches!(
        path,
        OfflineHighQualityPath::CompressionShortWindowSelector
            | OfflineHighQualityPath::ExpansionShortWindowSelector
    )
}

fn offline_stretch_artifact_capability_status(
    identity_input: &StretchCacheIdentityInput,
) -> OfflineStretchArtifactCapabilityStatus {
    if identity_input.channel_layout.channels != 2 {
        return OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout {
            channels: identity_input.channel_layout.channels,
        };
    }
    let pitch_shift = identity_input
        .pitch_curve
        .first()
        .map(|point| point.semitones)
        .unwrap_or(0.0);
    if identity_input
        .pitch_curve
        .iter()
        .any(|point| (point.semitones - pitch_shift).abs() > 1.0e-9)
    {
        return OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation;
    }
    if selector_offline_path_requires_static_materialization(identity_input.offline_path) {
        if ratio_curve_has_dynamic_changes(&identity_input.ratio_curve) {
            return OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathDynamicRatio {
                path: identity_input.offline_path,
            };
        }
        if pitch_shift.abs() > 1.0e-9 {
            return OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathPitchShift {
                path: identity_input.offline_path,
            };
        }
    }
    OfflineStretchArtifactCapabilityStatus::Supported
}

fn materialization_error_for_capability(
    capability_status: OfflineStretchArtifactCapabilityStatus,
) -> Option<OfflineStretchArtifactMaterializeError> {
    match capability_status {
        OfflineStretchArtifactCapabilityStatus::Supported => None,
        OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout { channels } => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels })
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathDynamicRatio { path } => {
            Some(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio { path },
            )
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathPitchShift { path } => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift { path })
        }
    }
}

fn ratio_curve_has_dynamic_changes(ratio_curve: &[StretchRatioPoint]) -> bool {
    let mut valid_ratios = ratio_curve
        .iter()
        .filter(|point| point.ratio.is_finite() && point.ratio > 0.0)
        .map(|point| point.ratio);
    let Some(first) = valid_ratios.next() else {
        return false;
    };
    valid_ratios.any(|ratio| (ratio - first).abs() > 1.0e-9)
}

fn static_pitch_shift(
    identity_input: &StretchCacheIdentityInput,
) -> Result<f64, OfflineStretchArtifactMaterializeError> {
    let first = identity_input
        .pitch_curve
        .first()
        .map(|point| point.semitones)
        .unwrap_or(0.0);
    if identity_input
        .pitch_curve
        .iter()
        .any(|point| (point.semitones - first).abs() > 1.0e-9)
    {
        return Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation);
    }
    Ok(first)
}
