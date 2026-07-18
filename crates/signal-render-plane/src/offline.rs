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

use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    render_plane, RenderPlanSpec, RenderPlaneError, RenderPlaneExecutor, RenderSampleBuffer,
    MAX_BLOCK_FRAMES,
};
use signal_dsp_stretch::{
    compare_synthetic_stretch_backends, plan_offline_stretch_chunks, stretch_backend_plan,
    OfflineHighQualityPath, OfflineHighQualityStretcher, StretchBackendStatus, StretchBackendTier,
    StretchCacheIdentity, StretchCacheIdentityError, StretchCacheIdentityInput,
    StretchOfflineChunk, StretchOfflineChunkConfig, StretchOfflineChunkPlan,
    StretchPromotionReceipt, StretchRatioPoint, StretchSyntheticPromotionPolicy,
};
use signal_primitives::SampleRate;

/// Default block quantum for offline rendering.
const DEFAULT_BLOCK_FRAMES: usize = 1024;
const OFFLINE_STRETCH_ARTIFACT_CHUNK_CROSSFADE_FRAMES: usize = 256;

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

/// Policy-derived offline stretch artifact planning request.
#[derive(Clone, Copy, Debug)]
pub struct OfflineStretchArtifactPolicyRequest<'a> {
    /// Consumer scope this artifact would serve.
    pub scope: OfflineStretchArtifactScope,
    /// Cache identity input for the artifact candidate.
    pub identity_input: &'a StretchCacheIdentityInput,
    /// Stable evidence id to attach to the derived promotion receipt.
    pub evidence_id: &'a str,
    /// Regression policy applied to Signal's synthetic comparison report.
    pub promotion_policy: StretchSyntheticPromotionPolicy,
}

/// Policy-derived offline stretch artifact materialization request.
#[derive(Clone, Copy, Debug)]
pub struct OfflineStretchArtifactBuildRequest<'a> {
    /// Planning and promotion policy inputs for this artifact.
    pub policy: OfflineStretchArtifactPolicyRequest<'a>,
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

/// Cache write/read handoff for a policy-gated render-cache stretch artifact.
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

/// Render-cache bridge decision for a policy-gated stretch artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineStretchArtifactCacheDecision {
    /// Whether the bridge reused an existing handoff or wrote a new one.
    pub kind: OfflineStretchArtifactCacheDecisionKind,
    /// Cache identity, render source, and receipt selected by this decision.
    pub handoff: OfflineStretchArtifactCacheHandoff,
}

/// Control-side render-cache bridge for policy-gated stretch artifacts.
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

    /// Resolve a policy-gated render-cache request against retained handoffs.
    ///
    /// Synthetic evidence alone cannot write a new product-facing handoff. A
    /// miss returns [`OfflineStretchArtifactMaterializeError::NotReady`] and
    /// writes nothing while the composite quality gate is incomplete.
    pub fn resolve_with_synthetic_policy(
        &mut self,
        request: OfflineStretchArtifactBuildRequest<'_>,
    ) -> Result<OfflineStretchArtifactCacheDecision, OfflineStretchArtifactMaterializeError> {
        let identity = request
            .policy
            .identity_input
            .identity()
            .map_err(OfflineStretchArtifactPlanError::InvalidIdentity)?;
        if let Some(handoff) = self.handoffs_by_hash.get(&identity.stable_hash) {
            return Ok(OfflineStretchArtifactCacheDecision {
                kind: OfflineStretchArtifactCacheDecisionKind::Hit,
                handoff: handoff.clone(),
            });
        }

        let handoff = build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(request)?;
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

/// Build an offline stretch artifact plan from Signal's synthetic comparison
/// report and an explicit regression policy.
///
/// Synthetic evidence is regression evidence only. This helper returns an
/// observable non-ready plan until absolute integrity, external comparator,
/// and completed real-source listening evidence are supplied through a
/// composite [`StretchPromotionReceipt`].
pub fn plan_offline_stretch_artifact_with_synthetic_policy(
    request: OfflineStretchArtifactPolicyRequest<'_>,
) -> Result<OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError> {
    let promotion_receipt = synthetic_policy_promotion_receipt(&request);
    plan_offline_stretch_artifact(request.scope, request.identity_input, promotion_receipt)
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
            stretcher.stretch_interleaved_stereo(&source.frames)
        } else if chunk_plan.is_single_chunk() {
            if pitch_shift.abs() > 1.0e-9 {
                stretcher.stretch_dynamic_ratio_pitch_interleaved_stereo(
                    &source.frames,
                    &identity_input.ratio_curve,
                    SampleRate(source.sample_rate_hz),
                    pitch_shift,
                )
            } else if identity_input.ratio_curve.is_empty() {
                stretcher.stretch_interleaved_stereo(&source.frames)
            } else {
                stretcher.stretch_dynamic_ratio_interleaved_stereo(
                    &source.frames,
                    &identity_input.ratio_curve,
                )
            }
        } else {
            materialize_chunked_offline_stretch_artifact_frames(
                source,
                identity_input.offline_path,
                pitch_shift,
                &chunk_plan,
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

/// Attempt materialization from Signal's synthetic regression report.
///
/// Synthetic-only evidence cannot satisfy the composite product-quality gate,
/// so this call returns [`OfflineStretchArtifactMaterializeError::NotReady`]
/// and produces no PCM buffer.
pub fn build_offline_stretch_artifact_pcm_with_synthetic_policy(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactPcm, OfflineStretchArtifactMaterializeError> {
    let promotion_receipt = synthetic_policy_promotion_receipt(&request.policy);
    materialize_offline_stretch_artifact_pcm(
        request.policy.scope,
        request.policy.identity_input,
        promotion_receipt,
        request.source,
    )
}

/// Attempt to build a synthetic-policy OfflineHighQuality render source.
///
/// Synthetic-only evidence returns
/// [`OfflineStretchArtifactMaterializeError::NotReady`] and does not produce a
/// product-facing source.
pub fn build_offline_stretch_artifact_render_source_with_synthetic_policy(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactRenderSource, OfflineStretchArtifactMaterializeError> {
    let artifact = build_offline_stretch_artifact_pcm_with_synthetic_policy(request)?;
    Ok(OfflineStretchArtifactRenderSource {
        source: crate::RenderSource::Samples(artifact.buffer.clone()),
        artifact,
    })
}

/// Attempt to build a synthetic-policy OfflineHighQuality cache handoff.
///
/// This helper is intentionally scoped to [`OfflineStretchArtifactScope::RenderCache`].
/// Synthetic-only evidence returns
/// [`OfflineStretchArtifactMaterializeError::NotReady`] and produces no
/// cacheable source.
pub fn build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactCacheHandoff, OfflineStretchArtifactMaterializeError> {
    if request.policy.scope != OfflineStretchArtifactScope::RenderCache {
        return Err(
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                scope: request.policy.scope,
            },
        );
    }
    let artifact_source =
        build_offline_stretch_artifact_render_source_with_synthetic_policy(request)?;
    let receipt = artifact_source.artifact.receipt.clone();
    Ok(OfflineStretchArtifactCacheHandoff {
        cache_identity_hash: receipt.cache_identity_hash.clone(),
        cache_identity_key: receipt.cache_identity_key.clone(),
        receipt,
        source: artifact_source.source,
    })
}

fn materialize_chunked_offline_stretch_artifact_frames(
    source: &RenderSampleBuffer,
    offline_path: OfflineHighQualityPath,
    pitch_shift: f64,
    chunk_plan: &StretchOfflineChunkPlan,
) -> Vec<f32> {
    let frame_count = source.frame_count();
    let even_source = &source.frames[..frame_count * 2];
    let mut output = Vec::with_capacity(chunk_plan.total_output_frames * 2);
    let mut boundaries = Vec::with_capacity(chunk_plan.chunks.len().saturating_sub(1));

    for (index, chunk) in chunk_plan.chunks.iter().enumerate() {
        let chunk_payload = materialize_stretch_chunk_payload(
            even_source,
            source.sample_rate_hz,
            offline_path,
            pitch_shift,
            chunk,
        );
        output.extend(chunk_payload);
        if index + 1 < chunk_plan.chunks.len() {
            boundaries.push(output.len() / 2);
        }
    }

    output.resize(chunk_plan.total_output_frames * 2, 0.0);
    smooth_artifact_chunk_boundaries_interleaved(
        &mut output,
        &boundaries,
        OFFLINE_STRETCH_ARTIFACT_CHUNK_CROSSFADE_FRAMES,
    );
    output
}

fn materialize_stretch_chunk_payload(
    even_source: &[f32],
    sample_rate_hz: u32,
    offline_path: OfflineHighQualityPath,
    pitch_shift: f64,
    chunk: &StretchOfflineChunk,
) -> Vec<f32> {
    let render_start = chunk.render_start_frame * 2;
    let render_end = chunk.render_end_frame * 2;
    let render_source = &even_source[render_start..render_end];
    let mut stretcher = OfflineHighQualityStretcher::with_path(chunk.ratio, offline_path);
    let rendered = if pitch_shift.abs() > 1.0e-9 {
        stretcher.stretch_pitch_interleaved_stereo(
            render_source,
            SampleRate(sample_rate_hz),
            pitch_shift,
        )
    } else {
        stretcher.stretch_interleaved_stereo(render_source)
    };

    let prefix_source_frames = chunk.source_start_frame - chunk.render_start_frame;
    let prefix_output_frames = (prefix_source_frames as f64 * chunk.ratio).round() as usize;
    let payload_output_frames = chunk.output_frames();
    let rendered_frames = rendered.len() / 2;
    let start_frame = prefix_output_frames.min(rendered_frames);
    let end_frame = (start_frame + payload_output_frames).min(rendered_frames);
    let mut payload = rendered[start_frame * 2..end_frame * 2].to_vec();
    payload.resize(payload_output_frames * 2, 0.0);
    payload
}

fn smooth_artifact_chunk_boundaries_interleaved(
    interleaved_samples: &mut [f32],
    boundary_frames: &[usize],
    fade_frames: usize,
) {
    if fade_frames == 0 {
        return;
    }
    let frames = interleaved_samples.len() / 2;
    if frames < 2 {
        return;
    }

    for boundary in boundary_frames {
        if *boundary == 0 || *boundary >= frames {
            continue;
        }
        let fade_frames = fade_frames.min(*boundary).min(frames - *boundary).max(1);
        for channel in 0..2 {
            let before_edge_index = (*boundary - 1) * 2 + channel;
            let after_edge_index = *boundary * 2 + channel;
            let before_edge = interleaved_samples[before_edge_index];
            let after_edge = interleaved_samples[after_edge_index];
            let midpoint = (before_edge + after_edge) * 0.5;
            for offset in 0..fade_frames {
                let weight = (fade_frames - offset) as f32 / fade_frames as f32;
                let before_frame = *boundary - 1 - offset;
                let after_frame = *boundary + offset;
                let before_index = before_frame * 2 + channel;
                let after_index = after_frame * 2 + channel;
                interleaved_samples[before_index] += (midpoint - before_edge) * weight;
                interleaved_samples[after_index] += (midpoint - after_edge) * weight;
            }
        }
    }
}

fn synthetic_policy_promotion_receipt(
    request: &OfflineStretchArtifactPolicyRequest<'_>,
) -> StretchPromotionReceipt {
    let report = compare_synthetic_stretch_backends();
    StretchPromotionReceipt::from_synthetic_offline_high_quality_report(
        request.evidence_id,
        &report,
        request.promotion_policy,
    )
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

/// Render `spec` offline: install it on a fresh controller/executor pair and
/// loop [`RenderPlaneExecutor::render_block`] as fast as the CPU allows.
///
/// The stream is set to the plan's master channel count, so the hardware
/// boundary is an identity copy and the export carries the full creative
/// mix format (no device-shaped downmix).
///
/// Edge-envelope bypass: all transport commands (install, seek, play) are
/// drained while the executor is still inaudible — the seek therefore lands
/// immediately, not through the audible ramp-out path — and then the edge
/// envelope is snapped open before the first rendered block. Realtime
/// behavior is untouched; the snap is a crate-private offline-only hook.
pub fn render_plan_to_pcm(
    spec: &RenderPlanSpec,
    options: &OfflineRenderOptions,
) -> Result<OfflineRenderOutput, RenderPlaneError> {
    let channels = spec.output_channels();
    let (mut controller, mut executor) = render_plane();
    controller.set_stream_channels(channels)?;
    controller.install_plan(spec)?;
    controller.seek(options.start_frame)?;
    controller.set_playing(true)?;
    // Apply everything queued above while the executor is inaudible
    // (edge_gain == 0), then snap the transport envelope open so frame one
    // of the bounce is at full level instead of 5 ms into a fade-in.
    executor.drain_commands();
    executor.set_edge_gain_immediate(1.0);

    // Resolve captured stage ids against the installed plan's topology once.
    let stem_indices: Vec<Option<(usize, usize)>> = {
        let plan = executor.plan.as_ref().expect("plan installed above");
        options
            .capture_stage_ids
            .iter()
            .map(|stage_id| {
                plan.stages
                    .iter()
                    .position(|stage| stage.stage_id == *stage_id)
                    .map(|index| (index, plan.stages[index].channels))
            })
            .collect()
    };
    let mut stems: Vec<(u64, Vec<f32>)> = options
        .capture_stage_ids
        .iter()
        .map(|stage_id| (*stage_id, Vec::new()))
        .collect();

    let block_frames = options.block_frames.clamp(1, MAX_BLOCK_FRAMES);
    let mut block = vec![0.0f32; block_frames * channels as usize];
    let mut master = Vec::with_capacity(options.frame_count as usize * channels as usize);
    let mut remaining = options.frame_count;
    while remaining > 0 {
        let frames_this_block = (remaining as usize).min(block_frames);
        let slice = &mut block[..frames_this_block * channels as usize];
        executor.render_block(slice);
        master.extend_from_slice(slice);
        capture_stems(&executor, &stem_indices, &mut stems, frames_this_block);
        remaining -= frames_this_block as u64;
    }

    // Free the retired-plan slot control-side (nothing retired in a single
    // install, but keep the contract symmetric).
    controller.collect_retired();

    Ok(OfflineRenderOutput {
        master,
        channels,
        sample_rate_hz: spec.sample_rate_hz.max(1),
        stems,
    })
}

/// Copy each captured stage's scratch for the block just rendered into its
/// stem buffer, scaled by the stage's per-frame block gain ramp — the same
/// post-fader level its consumers (edges, boundary) read, so unity-gain
/// stems sum to the master exactly.
fn capture_stems(
    executor: &RenderPlaneExecutor,
    stem_indices: &[Option<(usize, usize)>],
    stems: &mut [(u64, Vec<f32>)],
    frame_count: usize,
) {
    let Some(plan) = executor.plan.as_ref() else {
        return;
    };
    for (resolved, (_, stem)) in stem_indices.iter().zip(stems.iter_mut()) {
        let Some((stage_index, stage_channels)) = *resolved else {
            continue;
        };
        let stage = &plan.stages[stage_index];
        let scratch = &stage.scratch[..frame_count * stage_channels];
        stem.reserve(scratch.len());
        for frame_index in 0..frame_count {
            let gain = stage.block_gain_begin + stage.block_gain_slope * frame_index as f32;
            let base = frame_index * stage_channels;
            for channel in 0..stage_channels {
                stem.push(scratch[base + channel] * gain);
            }
        }
    }
}

// ── WAV writing with TPDF dither ────────────────────────────────────────────

/// Output bit depth for [`write_wav`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavBitDepth {
    /// 32-bit IEEE float: bit-transparent, no dither needed.
    Float32,
    /// 24-bit integer with TPDF dither.
    Int24,
    /// 16-bit integer with TPDF dither.
    Int16,
}

/// Minimal LCG for dither noise: deterministic, dependency-free, never used
/// for anything security- or statistics-critical beyond decorrelating
/// quantization error.
struct DitherLcg(u64);

impl DitherLcg {
    /// Uniform sample in `[0, 1)`.
    fn next_unit(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }

    /// TPDF sample in `(-1, 1)` LSB: sum of two independent uniforms,
    /// triangular probability density centered on zero.
    fn next_tpdf(&mut self) -> f32 {
        self.next_unit() + self.next_unit() - 1.0
    }
}

/// Write interleaved f32 PCM to a WAV file at `bit_depth`.
///
/// Integer depths apply TPDF dither (±1 LSB triangular, two independent
/// uniform randoms per sample from a constant-seeded LCG — no `rand`
/// dependency) before quantization, then clamp to the integer range.
/// `Float32` writes samples bit-exactly.
pub fn write_wav(
    path: &Path,
    samples: &[f32],
    channels: u16,
    sample_rate_hz: u32,
    bit_depth: WavBitDepth,
) -> Result<(), RenderPlaneError> {
    let io_error = |error: hound::Error| RenderPlaneError {
        message: format!("wav write failed: {error}"),
    };
    let spec = hound::WavSpec {
        channels: channels.max(1),
        sample_rate: sample_rate_hz.max(1),
        bits_per_sample: match bit_depth {
            WavBitDepth::Float32 => 32,
            WavBitDepth::Int24 => 24,
            WavBitDepth::Int16 => 16,
        },
        sample_format: match bit_depth {
            WavBitDepth::Float32 => hound::SampleFormat::Float,
            WavBitDepth::Int24 | WavBitDepth::Int16 => hound::SampleFormat::Int,
        },
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(io_error)?;
    match bit_depth {
        WavBitDepth::Float32 => {
            for sample in samples {
                writer.write_sample(*sample).map_err(io_error)?;
            }
        }
        WavBitDepth::Int24 => {
            let mut lcg = DitherLcg(0x0FF1_CED1_74E2_u64);
            const SCALE: f32 = 8_388_608.0; // 2^23
            for sample in samples {
                let dithered = sample * SCALE + lcg.next_tpdf();
                let quantized = dithered.round().clamp(-SCALE, SCALE - 1.0) as i32;
                writer.write_sample(quantized).map_err(io_error)?;
            }
        }
        WavBitDepth::Int16 => {
            let mut lcg = DitherLcg(0x0FF1_CED1_74E2_u64);
            const SCALE: f32 = 32_768.0; // 2^15
            for sample in samples {
                let dithered = sample * SCALE + lcg.next_tpdf();
                let quantized = dithered.round().clamp(-SCALE, SCALE - 1.0) as i16;
                writer.write_sample(quantized).map_err(io_error)?;
            }
        }
    }
    writer.finalize().map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderLimiterSpec, RenderSampleBuffer,
        RenderSource, RenderStageKind, RenderStageSpec,
    };
    use signal_dsp_stretch::{
        measure_dynamic_segment_seam_click, OfflineHighQualityPath, StretchChannelLayout,
        StretchPitchPoint, StretchProductQualityEvidence, StretchPromotionReceipt,
        StretchPromotionStatus, StretchRatioPoint, StretchSyntheticPromotionPolicy,
        StretchWarpMarker, REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
    };
    use std::sync::Arc;

    const MASTER_ID: u64 = 9_000;

    fn lane(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
        RenderStageSpec {
            accepts_live_events: false,
            processor: None,
            events: None,
            stage_id,
            format: ChannelFormat::stereo(),
            gain,
            gain_automation: None,
            kind: RenderStageKind::Source { clips },
            inputs: Vec::new(),
        }
    }

    fn master(inputs: Vec<u64>) -> RenderStageSpec {
        RenderStageSpec {
            accepts_live_events: false,
            processor: None,
            events: None,
            stage_id: MASTER_ID,
            format: ChannelFormat::stereo(),
            gain: 1.0,
            gain_automation: None,
            kind: RenderStageKind::Output,
            inputs: inputs
                .into_iter()
                .map(|source_stage_id| RenderEdgeSpec {
                    source_stage_id,
                    gain: 1.0,
                    matrix: None,
                })
                .collect(),
        }
    }

    fn tone_clip(clip_id: u64, frequency_hz: f32) -> RenderClipSpec {
        RenderClipSpec {
            clip_id,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone { frequency_hz },
            loop_source: false,
        }
    }

    /// Constant-value looped stereo sample clip: a DC plateau that reads at
    /// exactly `value` once past the clip edge fade.
    fn constant_clip(clip_id: u64, value: f32) -> RenderClipSpec {
        let mut data = Vec::new();
        for _ in 0..480 {
            data.push(value);
            data.push(value);
        }
        RenderClipSpec {
            clip_id,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Samples(RenderSampleBuffer::stereo(
                48_000,
                Arc::from(data.into_boxed_slice()),
            )),
            loop_source: true,
        }
    }

    fn reference_spec() -> RenderPlanSpec {
        let mut automated = lane(2, 0.4, vec![tone_clip(21, 661.0)]);
        automated.gain_automation = Some(vec![(0, 0.1), (24_000, 0.7), (48_000, 0.2)]);
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 0.9,
            master_limiter: Some(RenderLimiterSpec {
                threshold: 0.8,
                knee_width: 0.2,
                release_seconds: 0.05,
            }),
            stages: vec![
                lane(1, 0.5, vec![tone_clip(11, 440.0)]),
                automated,
                master(vec![1, 2]),
            ],
        }
    }

    fn stretch_identity_input() -> StretchCacheIdentityInput {
        StretchCacheIdentityInput::signal_native(
            StretchBackendTier::OfflineHighQuality,
            "sha256:render-source",
            StretchChannelLayout::new(2, 48_000),
            "projection-42",
        )
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(48_000, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
        .with_warp_markers(vec![StretchWarpMarker::new(0, 0)])
    }

    fn stretch_artifact_source(frame_count: usize) -> RenderSampleBuffer {
        let mut frames = Vec::with_capacity(frame_count * 2);
        for frame in 0..frame_count {
            let sample = (frame as f32 / 17.0).sin() * 0.25;
            frames.push(sample);
            frames.push(sample * 0.75);
        }
        RenderSampleBuffer::stereo(48_000, Arc::from(frames.into_boxed_slice()))
    }

    fn cache_bridge_request<'a>(
        identity_input: &'a StretchCacheIdentityInput,
        source: &'a RenderSampleBuffer,
        promotion_policy: StretchSyntheticPromotionPolicy,
    ) -> OfflineStretchArtifactBuildRequest<'a> {
        OfflineStretchArtifactBuildRequest {
            policy: OfflineStretchArtifactPolicyRequest {
                scope: OfflineStretchArtifactScope::RenderCache,
                identity_input,
                evidence_id: "synthetic:cache-bridge-current",
                promotion_policy,
            },
            source,
        }
    }

    fn current_corpus_policy_request<'a>(
        scope: OfflineStretchArtifactScope,
        identity_input: &'a StretchCacheIdentityInput,
        evidence_id: &'a str,
    ) -> OfflineStretchArtifactPolicyRequest<'a> {
        OfflineStretchArtifactPolicyRequest {
            scope,
            identity_input,
            evidence_id,
            promotion_policy: StretchSyntheticPromotionPolicy::default(),
        }
    }

    fn current_corpus_build_request<'a>(
        scope: OfflineStretchArtifactScope,
        identity_input: &'a StretchCacheIdentityInput,
        evidence_id: &'a str,
        source: &'a RenderSampleBuffer,
    ) -> OfflineStretchArtifactBuildRequest<'a> {
        OfflineStretchArtifactBuildRequest {
            policy: current_corpus_policy_request(scope, identity_input, evidence_id),
            source,
        }
    }

    fn complete_product_quality_evidence(
        passed_case_count: u32,
        required_case_count: u32,
    ) -> StretchProductQualityEvidence {
        StretchProductQualityEvidence {
            compared_to_draft_baseline: true,
            absolute_integrity_passed: true,
            comparator_row_count: 18,
            required_comparator_row_count: 18,
            passed_case_count,
            required_case_count,
            completed_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
            required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
        }
    }

    fn accepted_product_quality_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
        let required_case_count =
            StretchSyntheticPromotionPolicy::default().min_comparison_count as u32;
        let receipt = StretchPromotionReceipt::from_product_quality_evidence(
            evidence_id,
            OfflineHighQualityPath::Default,
            complete_product_quality_evidence(required_case_count, required_case_count),
        );
        assert_product_quality_promotion_receipt(&receipt, evidence_id);
        receipt
    }

    fn accepted_selector_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
        let receipt = StretchPromotionReceipt::from_product_quality_evidence(
            evidence_id,
            OfflineHighQualityPath::CompressionShortWindowSelector,
            complete_product_quality_evidence(20, 20),
        );
        assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
        assert_eq!(receipt.evidence_id, evidence_id);
        assert!(receipt.accepts_product_facing_path(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::CompressionShortWindowSelector
        ));
        receipt
    }

    fn accepted_expansion_selector_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
        let receipt = StretchPromotionReceipt::from_product_quality_evidence(
            evidence_id,
            OfflineHighQualityPath::ExpansionShortWindowSelector,
            complete_product_quality_evidence(40, 40),
        );
        assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
        assert_eq!(receipt.evidence_id, evidence_id);
        assert!(receipt.accepts_product_facing_path(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::ExpansionShortWindowSelector
        ));
        receipt
    }

    fn assert_product_quality_promotion_receipt(
        receipt: &StretchPromotionReceipt,
        evidence_id: &str,
    ) {
        assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
        assert_eq!(receipt.evidence_id, evidence_id);
        assert!(receipt.compared_to_draft_baseline);
        assert!(receipt.absolute_integrity_passed);
        assert!(receipt.comparator_row_count >= receipt.required_comparator_row_count);
        assert_eq!(
            receipt.required_case_count,
            StretchSyntheticPromotionPolicy::default().min_comparison_count as u32
        );
        assert_eq!(receipt.offline_path, OfflineHighQualityPath::Default);
        assert!(receipt.passed_case_count >= receipt.required_case_count);
        assert_eq!(
            receipt.completed_listening_family_count,
            REQUIRED_STRETCH_LISTENING_FAMILY_COUNT
        );
        assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    }

    fn max_abs_delta(left: &[f32], right: &[f32]) -> f32 {
        left.iter()
            .zip(right.iter())
            .map(|(left, right)| (left - right).abs())
            .fold(0.0f32, f32::max)
    }

    #[test]
    fn synthetic_only_policy_keeps_export_plan_awaiting_quality_evidence() {
        let input = stretch_identity_input();
        let plan =
            plan_offline_stretch_artifact_with_synthetic_policy(current_corpus_policy_request(
                OfflineStretchArtifactScope::Export,
                &input,
                "synthetic:render-plane-current",
            ))
            .expect("current corpus policy should produce artifact plan");

        assert_eq!(plan.scope, OfflineStretchArtifactScope::Export);
        assert_eq!(plan.tier, StretchBackendTier::OfflineHighQuality);
        assert_eq!(plan.offline_path, OfflineHighQualityPath::Default);
        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        );
        assert_eq!(
            plan.promotion_receipt.status,
            StretchPromotionStatus::Accepted
        );
        assert_eq!(
            plan.promotion_receipt
                .product_facing_blocker(StretchBackendTier::OfflineHighQuality),
            Some("promotion evidence did not pass absolute render integrity")
        );
        assert!(!plan.product_facing_allowed);
        assert!(plan
            .identity
            .canonical_key
            .contains("tier=OfflineHighQuality"));
        assert!(plan.identity.canonical_key.contains("offline_path=Default"));
        assert_eq!(plan.identity, input.identity().expect("same identity"));
    }

    #[test]
    fn ready_stretch_artifact_materializes_cacheable_pcm_for_render_consumers() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);
        let artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::Freeze,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:materialize-current"),
            &source,
        )
        .expect("composite product-quality evidence should materialize");
        assert_product_quality_promotion_receipt(
            &artifact.plan.promotion_receipt,
            "product-quality:materialize-current",
        );

        assert_eq!(artifact.plan.scope, OfflineStretchArtifactScope::Freeze);
        assert_eq!(
            artifact.plan.readiness,
            OfflineStretchArtifactReadiness::Ready
        );
        assert!(artifact.plan.product_facing_allowed);
        assert_eq!(artifact.input_frame_count, 480);
        assert_eq!(artifact.output_frame_count, 600);
        assert_eq!(
            artifact.receipt.cache_identity_hash,
            artifact.plan.identity.stable_hash
        );
        assert_eq!(
            artifact.receipt.offline_path,
            OfflineHighQualityPath::Default
        );
        assert_eq!(
            artifact.receipt.promotion_evidence_id,
            "product-quality:materialize-current"
        );
        assert_eq!(
            artifact.receipt.input_frame_count,
            artifact.input_frame_count
        );
        assert_eq!(
            artifact.receipt.output_frame_count,
            artifact.output_frame_count
        );
        assert!(artifact.receipt.product_facing_allowed);
        assert_eq!(artifact.buffer.sample_rate_hz, source.sample_rate_hz);
        assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
        assert_eq!(
            artifact.plan.identity,
            input.identity().expect("same identity")
        );

        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane(
                    44,
                    1.0,
                    vec![RenderClipSpec {
                        clip_id: 440,
                        start_frames: 0,
                        end_frames: artifact.output_frame_count as u64,
                        source: RenderSource::Samples(artifact.buffer.clone()),
                        loop_source: false,
                    }],
                ),
                master(vec![44]),
            ],
        };
        let rendered = render_plan_to_pcm(
            &spec,
            &OfflineRenderOptions {
                frame_count: artifact.output_frame_count as u64,
                ..OfflineRenderOptions::default()
            },
        )
        .expect("materialized artifact should render as a sample source");

        assert_eq!(rendered.master.len(), artifact.output_frame_count * 2);
    }

    #[test]
    fn direct_receipt_materialization_keeps_lower_level_plan_gate() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);
        let artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt(
                "product-quality:direct-materialize-current",
            ),
            &source,
        )
        .expect("accepted direct receipt should materialize through the lower-level gate");

        assert_eq!(
            artifact.plan.scope,
            OfflineStretchArtifactScope::RenderCache
        );
        assert_eq!(
            artifact.plan.readiness,
            OfflineStretchArtifactReadiness::Ready
        );
        assert!(artifact.plan.product_facing_allowed);
        assert_eq!(artifact.input_frame_count, 480);
        assert_eq!(artifact.output_frame_count, 600);
        assert_eq!(
            artifact.receipt.cache_identity_hash,
            artifact.plan.identity.stable_hash
        );
        assert_eq!(
            artifact.receipt.promotion_evidence_id,
            "product-quality:direct-materialize-current"
        );
        assert_eq!(
            artifact.receipt.input_frame_count,
            artifact.input_frame_count
        );
        assert_eq!(
            artifact.receipt.output_frame_count,
            artifact.output_frame_count
        );
        assert!(artifact.receipt.product_facing_allowed);
        assert_eq!(artifact.buffer.sample_rate_hz, source.sample_rate_hz);
        assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
        assert_eq!(
            artifact.plan.identity,
            input.identity().expect("same identity")
        );

        assert_product_quality_promotion_receipt(
            &artifact.plan.promotion_receipt,
            "product-quality:direct-materialize-current",
        );
    }

    #[test]
    fn chunked_artifact_materialization_records_bounded_chunk_plan() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let source = stretch_artifact_source(1_024);
        let artifact = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt(
                "product-quality:chunked-materialize-current",
            ),
            &source,
            StretchOfflineChunkConfig::new(256, 64),
        )
        .expect("accepted direct receipt should materialize with bounded chunks");

        assert_eq!(artifact.chunk_plan.chunks.len(), 4);
        assert_eq!(
            artifact.chunk_plan.total_source_frames,
            source.frame_count()
        );
        assert_eq!(artifact.chunk_plan.total_output_frames, 1_280);
        assert_eq!(artifact.output_frame_count, 1_280);
        assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
        assert_eq!(artifact.receipt.chunk_count, 4);
        assert_eq!(artifact.receipt.max_chunk_source_frames, 256);
        assert_eq!(artifact.receipt.chunk_overlap_frames, 64);
        assert!(artifact.receipt.max_chunk_render_source_frames <= 256 + 64 * 2);
        assert_eq!(
            artifact.receipt.max_chunk_render_source_frames,
            artifact.chunk_plan.max_render_source_frames()
        );
        assert!(artifact.receipt.product_facing_allowed);
    }

    #[test]
    fn chunked_artifact_materialization_is_deterministic_for_dynamic_ratio() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(512, 1.25),
            ])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let source = stretch_artifact_source(2_048);
        let receipt =
            accepted_product_quality_promotion_receipt("product-quality:chunked-dynamic-ratio");

        let first = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt.clone(),
            &source,
            StretchOfflineChunkConfig::new(512, 128),
        )
        .expect("first chunked materialization should succeed");
        let repeated = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt,
            &source,
            StretchOfflineChunkConfig::new(512, 128),
        )
        .expect("repeated chunked materialization should succeed");

        assert_eq!(first.output_frame_count, 2_432);
        assert_eq!(
            first.chunk_plan.total_output_frames,
            first.output_frame_count
        );
        assert_eq!(first.receipt.chunk_count, 4);
        assert_eq!(first.buffer.sample_rate_hz, repeated.buffer.sample_rate_hz);
        assert_eq!(first.buffer.frame_count(), repeated.buffer.frame_count());
        assert!(
            max_abs_delta(&first.buffer.frames, &repeated.buffer.frames) < 1.0e-6,
            "chunked materialization should be sample-stable"
        );
        assert_eq!(first.chunk_plan, repeated.chunk_plan);
        assert_eq!(
            first.receipt.cache_identity_hash,
            repeated.receipt.cache_identity_hash
        );
    }

    #[test]
    fn chunked_artifact_boundary_smoothing_reduces_rendered_seam_clicks() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(512, 1.25),
            ])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let source = stretch_artifact_source(2_048);
        let chunk_plan = plan_offline_stretch_chunks(
            source.frame_count(),
            &input.ratio_curve,
            static_or_initial_ratio(&input.ratio_curve),
            StretchOfflineChunkConfig::new(512, 128),
        );
        let even_source = &source.frames[..source.frame_count() * 2];
        let mut raw = Vec::with_capacity(chunk_plan.total_output_frames * 2);
        let mut boundaries = Vec::with_capacity(chunk_plan.chunks.len().saturating_sub(1));
        for (index, chunk) in chunk_plan.chunks.iter().enumerate() {
            raw.extend(materialize_stretch_chunk_payload(
                even_source,
                source.sample_rate_hz,
                input.offline_path,
                0.0,
                chunk,
            ));
            if index + 1 < chunk_plan.chunks.len() {
                boundaries.push(raw.len() / 2);
            }
        }

        let before = measure_dynamic_segment_seam_click(&raw, 2, &boundaries, 1.0);
        let mut smoothed = raw.clone();
        smooth_artifact_chunk_boundaries_interleaved(
            &mut smoothed,
            &boundaries,
            OFFLINE_STRETCH_ARTIFACT_CHUNK_CROSSFADE_FRAMES,
        );
        let after = measure_dynamic_segment_seam_click(&smoothed, 2, &boundaries, 1.0);

        assert_eq!(before.seam_frames, boundaries);
        assert_eq!(after.seam_frames, boundaries);
        assert!(
            after.peak_seam_delta < before.peak_seam_delta,
            "boundary smoothing should reduce rendered chunk seam clicks"
        );
    }

    #[test]
    fn synthetic_only_policy_blocks_pcm_artifact_builder() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);

        let result = build_offline_stretch_artifact_pcm_with_synthetic_policy(
            OfflineStretchArtifactBuildRequest {
                policy: OfflineStretchArtifactPolicyRequest {
                    scope: OfflineStretchArtifactScope::Export,
                    identity_input: &input,
                    evidence_id: "synthetic:builder-policy-current",
                    promotion_policy: StretchSyntheticPromotionPolicy::default(),
                },
                source: &source,
            },
        );

        assert_eq!(
            result,
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn synthetic_only_policy_blocks_render_source_builder() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);

        let result = build_offline_stretch_artifact_render_source_with_synthetic_policy(
            OfflineStretchArtifactBuildRequest {
                policy: OfflineStretchArtifactPolicyRequest {
                    scope: OfflineStretchArtifactScope::RenderCache,
                    identity_input: &input,
                    evidence_id: "synthetic:builder-render-source-current",
                    promotion_policy: StretchSyntheticPromotionPolicy::default(),
                },
                source: &source,
            },
        );

        assert_eq!(
            result,
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn synthetic_only_policy_blocks_freeze_render_source() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);
        let result = build_offline_stretch_artifact_render_source_with_synthetic_policy(
            OfflineStretchArtifactBuildRequest {
                policy: OfflineStretchArtifactPolicyRequest {
                    scope: OfflineStretchArtifactScope::Freeze,
                    identity_input: &input,
                    evidence_id: "synthetic:builder-export-freeze-current",
                    promotion_policy: StretchSyntheticPromotionPolicy::default(),
                },
                source: &source,
            },
        );

        assert_eq!(
            result,
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn composite_quality_artifacts_preserve_cache_identity_and_change_on_inputs() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let changed_projection = StretchCacheIdentityInput {
            projection_epoch: "projection-43".to_string(),
            ..input.clone()
        };
        let changed_curve = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.5)])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let changed_path = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
        let source = stretch_artifact_source(480);
        let build = |identity_input: &StretchCacheIdentityInput| {
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                identity_input,
                accepted_product_quality_promotion_receipt(
                    "product-quality:builder-cache-identity",
                ),
                &source,
            )
            .expect("composite evidence should produce cacheable PCM")
        };

        let base = build(&input);
        let repeated = build(&input);
        let projection_changed = build(&changed_projection);
        let curve_changed = build(&changed_curve);
        let path_changed = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &changed_path,
            accepted_selector_promotion_receipt("fma-rubberband:builder-cache-identity"),
            &source,
        )
        .expect("selector-specific evidence should produce selector cache identity");

        assert_eq!(
            base.receipt.cache_identity_hash,
            repeated.receipt.cache_identity_hash
        );
        assert_eq!(
            base.receipt.cache_identity_key,
            repeated.receipt.cache_identity_key
        );
        assert_ne!(
            base.receipt.cache_identity_hash,
            projection_changed.receipt.cache_identity_hash
        );
        assert_ne!(
            base.receipt.cache_identity_hash,
            curve_changed.receipt.cache_identity_hash
        );
        assert_ne!(
            base.receipt.cache_identity_hash,
            path_changed.receipt.cache_identity_hash
        );
        assert_eq!(
            path_changed.receipt.offline_path,
            OfflineHighQualityPath::CompressionShortWindowSelector
        );
        assert_ne!(base.buffer, projection_changed.buffer);
        assert_ne!(base.buffer, curve_changed.buffer);
        assert_ne!(base.output_frame_count, curve_changed.output_frame_count);
    }

    #[test]
    fn synthetic_only_policy_blocks_render_cache_handoff() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);

        let result = build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(
            OfflineStretchArtifactBuildRequest {
                policy: OfflineStretchArtifactPolicyRequest {
                    scope: OfflineStretchArtifactScope::RenderCache,
                    identity_input: &input,
                    evidence_id: "synthetic:cache-handoff-current",
                    promotion_policy: StretchSyntheticPromotionPolicy::default(),
                },
                source: &source,
            },
        );

        assert_eq!(
            result,
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn render_cache_handoff_rejects_non_cache_scope_and_rejected_policy_without_source() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);
        let rejected_policy = StretchSyntheticPromotionPolicy {
            min_comparison_count: usize::MAX,
            ..StretchSyntheticPromotionPolicy::default()
        };

        assert_eq!(
            build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(
                OfflineStretchArtifactBuildRequest {
                    policy: OfflineStretchArtifactPolicyRequest {
                        scope: OfflineStretchArtifactScope::Freeze,
                        identity_input: &input,
                        evidence_id: "synthetic:cache-handoff-wrong-scope",
                        promotion_policy: StretchSyntheticPromotionPolicy::default(),
                    },
                    source: &source,
                },
            ),
            Err(
                OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                    scope: OfflineStretchArtifactScope::Freeze
                }
            )
        );
        assert_eq!(
            build_offline_stretch_artifact_cache_handoff_with_synthetic_policy(
                OfflineStretchArtifactBuildRequest {
                    policy: OfflineStretchArtifactPolicyRequest {
                        scope: OfflineStretchArtifactScope::RenderCache,
                        identity_input: &input,
                        evidence_id: "synthetic:cache-handoff-rejected",
                        promotion_policy: rejected_policy,
                    },
                    source: &source,
                },
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn synthetic_only_cache_bridge_rejects_without_writing_handoffs() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let changed_projection = StretchCacheIdentityInput {
            projection_epoch: "projection-44".to_string(),
            ..input.clone()
        };
        let source = stretch_artifact_source(480);
        let mut bridge = OfflineStretchArtifactRenderCacheBridge::new();

        assert!(bridge.is_empty());
        assert_eq!(
            bridge.resolve_with_synthetic_policy(cache_bridge_request(
                &input,
                &source,
                StretchSyntheticPromotionPolicy::default(),
            )),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
        assert_eq!(
            bridge.resolve_with_synthetic_policy(cache_bridge_request(
                &changed_projection,
                &source,
                StretchSyntheticPromotionPolicy::default(),
            )),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
        assert!(bridge.is_empty());
        assert!(!bridge.contains_identity_hash(
            &changed_projection
                .identity()
                .expect("changed identity should validate")
                .stable_hash
        ));
    }

    #[test]
    fn materialization_receipts_audit_cache_identity_inputs() {
        let base = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
            .with_warp_markers(vec![StretchWarpMarker::new(0, 0)]);
        let changed_engine = StretchCacheIdentityInput {
            engine_version: "signal-native-stretch-v3-test".to_string(),
            ..base.clone()
        };
        let changed_media = StretchCacheIdentityInput {
            source_content_hash: "sha256:render-source-b".to_string(),
            ..base.clone()
        };
        let changed_projection = StretchCacheIdentityInput {
            projection_epoch: "projection-43".to_string(),
            ..base.clone()
        };
        let changed_ratio = base
            .clone()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.5)]);
        let changed_pitch = base
            .clone()
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 1.0)]);
        let changed_marker = base
            .clone()
            .with_warp_markers(vec![StretchWarpMarker::new(96, 128)]);
        let source = stretch_artifact_source(96);
        let receipt = accepted_product_quality_promotion_receipt("product-quality:identity-audit");

        let mut observed = Vec::new();
        for input in [
            &base,
            &changed_engine,
            &changed_media,
            &changed_projection,
            &changed_ratio,
            &changed_pitch,
            &changed_marker,
        ] {
            let artifact = materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                input,
                receipt.clone(),
                &source,
            )
            .expect("identity variant should materialize");
            observed.push((
                artifact.receipt.cache_identity_hash,
                artifact.receipt.cache_identity_key,
            ));
        }

        for (index, (hash, _)) in observed.iter().enumerate() {
            assert!(
                observed
                    .iter()
                    .enumerate()
                    .all(
                        |(other_index, (other_hash, _))| index == other_index || hash != other_hash
                    ),
                "identity hash {hash} should be unique"
            );
        }
        assert!(observed[1]
            .1
            .contains("engine=signal-native-stretch-v3-test"));
        assert!(observed[2]
            .1
            .contains("source_content_hash=sha256:render-source-b"));
        assert!(observed[3].1.contains("projection_epoch=projection-43"));
        assert!(observed[4].1.contains("ratio_curve="));
        assert!(observed[5].1.contains("pitch_curve="));
        assert!(observed[6].1.contains("warp_markers=96:128"));
    }

    #[test]
    fn materialization_receipts_make_chunk_policy_auditable() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(1_024);
        let receipt =
            accepted_product_quality_promotion_receipt("product-quality:chunk-policy-audit");

        let fine = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt.clone(),
            &source,
            StretchOfflineChunkConfig::new(256, 64),
        )
        .expect("fine chunk policy should materialize");
        let coarse = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt,
            &source,
            StretchOfflineChunkConfig::new(512, 128),
        )
        .expect("coarse chunk policy should materialize");

        assert_eq!(fine.output_frame_count, coarse.output_frame_count);
        assert_eq!(fine.receipt.chunk_count, 4);
        assert_eq!(coarse.receipt.chunk_count, 2);
        assert_eq!(fine.receipt.max_chunk_source_frames, 256);
        assert_eq!(coarse.receipt.max_chunk_source_frames, 512);
        assert_eq!(fine.receipt.chunk_overlap_frames, 64);
        assert_eq!(coarse.receipt.chunk_overlap_frames, 128);
        assert_ne!(
            fine.receipt.max_chunk_render_source_frames,
            coarse.receipt.max_chunk_render_source_frames
        );
    }

    #[test]
    fn chunked_artifact_renders_realistic_duration_through_export_fixture() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(24_000, 1.25),
            ])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let source = stretch_artifact_source(48_000);
        let artifact = materialize_offline_stretch_artifact_pcm_with_chunk_config(
            OfflineStretchArtifactScope::Export,
            &input,
            accepted_product_quality_promotion_receipt(
                "product-quality:realistic-duration-chunked",
            ),
            &source,
            StretchOfflineChunkConfig::new(8_000, 512),
        )
        .expect("realistic-duration chunked artifact should materialize");

        assert_eq!(artifact.input_frame_count, 48_000);
        assert_eq!(artifact.output_frame_count, 54_000);
        assert_eq!(artifact.receipt.chunk_count, 6);
        assert!(artifact.receipt.max_chunk_render_source_frames <= 9_024);

        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane(
                    48,
                    1.0,
                    vec![RenderClipSpec {
                        clip_id: 480,
                        start_frames: 0,
                        end_frames: artifact.output_frame_count as u64,
                        source: RenderSource::Samples(artifact.buffer.clone()),
                        loop_source: false,
                    }],
                ),
                master(vec![48]),
            ],
        };
        let rendered = render_plan_to_pcm(
            &spec,
            &OfflineRenderOptions {
                frame_count: artifact.output_frame_count as u64,
                block_frames: 512,
                ..OfflineRenderOptions::default()
            },
        )
        .expect("chunked artifact should render through export fixture");

        assert_eq!(rendered.master.len(), artifact.output_frame_count * 2);
        assert_eq!(rendered.sample_rate_hz, 48_000);
    }

    #[test]
    fn artifact_builder_gate_blocks_rejected_policy_without_product_buffer() {
        let input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let source = stretch_artifact_source(480);
        let rejecting_policy = StretchSyntheticPromotionPolicy {
            min_comparison_count: usize::MAX,
            ..StretchSyntheticPromotionPolicy::default()
        };
        let policy_request = OfflineStretchArtifactPolicyRequest {
            scope: OfflineStretchArtifactScope::Freeze,
            identity_input: &input,
            evidence_id: "synthetic:builder-policy-rejected",
            promotion_policy: rejecting_policy,
        };
        let plan = plan_offline_stretch_artifact_with_synthetic_policy(policy_request)
            .expect("rejected policy still produces a non-ready plan");

        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        );
        assert!(!plan.product_facing_allowed);
        assert_eq!(
            build_offline_stretch_artifact_pcm_with_synthetic_policy(
                OfflineStretchArtifactBuildRequest {
                    policy: policy_request,
                    source: &source,
                },
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
        assert_eq!(
            build_offline_stretch_artifact_render_source_with_synthetic_policy(
                OfflineStretchArtifactBuildRequest {
                    policy: policy_request,
                    source: &source,
                },
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn stretch_artifact_plan_blocks_export_without_accepted_promotion() {
        let input = stretch_identity_input();
        let plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::Export,
            &input,
            StretchPromotionReceipt::default(),
        )
        .expect("artifact plan");

        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        );
        assert!(!plan.product_facing_allowed);
    }

    #[test]
    fn stretch_artifact_materialization_blocks_without_accepted_promotion() {
        let input = stretch_identity_input();
        let source = stretch_artifact_source(480);

        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::Export,
                &input,
                StretchPromotionReceipt::default(),
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn stretch_artifact_plan_marks_unsupported_channel_layout_as_capability_blocker() {
        let input = StretchCacheIdentityInput::signal_native(
            StretchBackendTier::OfflineHighQuality,
            "sha256:mono-render-source",
            StretchChannelLayout::new(1, 48_000),
            "projection-mono",
        )
        .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)]);
        let source =
            RenderSampleBuffer::stereo(48_000, Arc::from(vec![0.0f32; 480].into_boxed_slice()));
        let receipt = accepted_product_quality_promotion_receipt("product-quality:mono-capability");
        let plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt.clone(),
        )
        .expect("mono identity should still produce an observable plan");

        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::UnsupportedCapability
        );
        assert_eq!(
            plan.capability_status,
            OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout { channels: 1 }
        );
        assert!(!plan.product_facing_allowed);
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                receipt,
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels: 1 })
        );
    }

    #[test]
    fn stretch_artifact_plan_marks_pitch_automation_as_capability_blocker() {
        let input = stretch_identity_input().with_pitch_curve(vec![
            StretchPitchPoint::new(0, 0.0),
            StretchPitchPoint::new(240, 2.0),
        ]);
        let source = stretch_artifact_source(480);
        let receipt =
            accepted_product_quality_promotion_receipt("product-quality:pitch-automation-plan");
        let plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            receipt.clone(),
        )
        .expect("pitch automation identity should still produce an observable plan");

        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::UnsupportedCapability
        );
        assert_eq!(
            plan.capability_status,
            OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation
        );
        assert!(!plan.product_facing_allowed);
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                receipt,
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
        );
    }

    #[test]
    fn synthetic_only_policy_blocks_static_pitch_with_dynamic_ratio_curve() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(240, 1.25),
            ])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)]);
        let source = stretch_artifact_source(480);

        let result =
            build_offline_stretch_artifact_pcm_with_synthetic_policy(current_corpus_build_request(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                "synthetic:static-pitch-dynamic-ratio",
                &source,
            ));

        assert_eq!(
            result,
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn direct_receipt_materializes_static_pitch_with_dynamic_ratio_curve() {
        let input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(240, 1.25),
            ])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)]);
        let source = stretch_artifact_source(480);

        let artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt(
                "product-quality:direct-static-pitch-dynamic-ratio",
            ),
            &source,
        )
        .expect("accepted direct receipt should materialize static pitch plus dynamic ratio");

        assert_eq!(artifact.output_frame_count, 540);
        assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
        assert_eq!(
            artifact.receipt.promotion_evidence_id,
            "product-quality:direct-static-pitch-dynamic-ratio"
        );
    }

    #[test]
    fn stretch_artifact_materialization_rejects_pitch_automation() {
        let input = stretch_identity_input().with_pitch_curve(vec![
            StretchPitchPoint::new(0, 0.0),
            StretchPitchPoint::new(240, 2.0),
        ]);
        let source = stretch_artifact_source(480);

        assert_eq!(
            build_offline_stretch_artifact_pcm_with_synthetic_policy(current_corpus_build_request(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                "synthetic:pitch-automation",
                &source,
            ),),
            Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
        );
    }

    #[test]
    fn direct_receipt_materialization_rejects_pitch_automation() {
        let input = stretch_identity_input().with_pitch_curve(vec![
            StretchPitchPoint::new(0, 0.0),
            StretchPitchPoint::new(240, 2.0),
        ]);
        let source = stretch_artifact_source(480);

        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                accepted_product_quality_promotion_receipt(
                    "product-quality:direct-pitch-automation",
                ),
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
        );
    }

    #[test]
    fn compression_short_window_selector_materializes_static_stereo_and_changes_identity() {
        let default_input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)]);
        let selector_input = default_input
            .clone()
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
        let source = stretch_artifact_source(2_048);

        let default_artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &default_input,
            accepted_product_quality_promotion_receipt("product-quality:default-path-static"),
            &source,
        )
        .expect("default path should materialize");
        let selector_artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            accepted_selector_promotion_receipt("fma-rubberband:selector-path-static"),
            &source,
        )
        .expect("selector path should materialize static stereo");

        assert_eq!(
            selector_artifact.plan.offline_path,
            OfflineHighQualityPath::CompressionShortWindowSelector
        );
        assert_eq!(
            selector_artifact.receipt.offline_path,
            OfflineHighQualityPath::CompressionShortWindowSelector
        );
        assert_eq!(selector_artifact.output_frame_count, 1_536);
        assert_ne!(
            default_artifact.receipt.cache_identity_hash,
            selector_artifact.receipt.cache_identity_hash
        );
        assert!(selector_artifact
            .receipt
            .cache_identity_key
            .contains("offline_path=CompressionShortWindowSelector"));
    }

    #[test]
    fn expansion_short_window_selector_materializes_static_stereo_and_changes_identity() {
        let default_input =
            stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
        let selector_input = default_input
            .clone()
            .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
        let source = stretch_artifact_source(2_048);

        let default_artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &default_input,
            accepted_product_quality_promotion_receipt("product-quality:default-expansion-static"),
            &source,
        )
        .expect("default path should materialize");
        let selector_artifact = materialize_offline_stretch_artifact_pcm(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            accepted_expansion_selector_promotion_receipt(
                "fma-rubberband:expansion-selector-path-static",
            ),
            &source,
        )
        .expect("expansion selector path should materialize static stereo");

        assert_eq!(
            selector_artifact.plan.offline_path,
            OfflineHighQualityPath::ExpansionShortWindowSelector
        );
        assert_eq!(
            selector_artifact.receipt.offline_path,
            OfflineHighQualityPath::ExpansionShortWindowSelector
        );
        assert_eq!(selector_artifact.output_frame_count, 2_560);
        assert_ne!(
            default_artifact.receipt.cache_identity_hash,
            selector_artifact.receipt.cache_identity_hash
        );
        assert!(selector_artifact
            .receipt
            .cache_identity_key
            .contains("offline_path=ExpansionShortWindowSelector"));
    }

    #[test]
    fn compression_short_window_selector_rejects_default_promotion_receipt() {
        let selector_input = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
        let source = stretch_artifact_source(2_048);
        let default_receipt =
            accepted_product_quality_promotion_receipt("product-quality:default-path-not-selector");

        let plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            default_receipt.clone(),
        )
        .expect("selector plan should still validate identity");
        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        );
        assert!(!plan.product_facing_allowed);
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &selector_input,
                default_receipt,
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
        assert_eq!(
            build_offline_stretch_artifact_pcm_with_synthetic_policy(current_corpus_build_request(
                OfflineStretchArtifactScope::RenderCache,
                &selector_input,
                "synthetic:selector-default-policy",
                &source,
            )),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn expansion_short_window_selector_rejects_default_promotion_receipt() {
        let selector_input = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
            .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
        let source = stretch_artifact_source(2_048);
        let default_receipt = accepted_product_quality_promotion_receipt(
            "product-quality:default-path-not-expansion-selector",
        );

        let plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &selector_input,
            default_receipt.clone(),
        )
        .expect("expansion selector plan should still validate identity");
        assert_eq!(
            plan.readiness,
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        );
        assert!(!plan.product_facing_allowed);
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &selector_input,
                default_receipt,
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::NotReady(
                OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            ))
        );
    }

    #[test]
    fn compression_short_window_selector_rejects_unproven_artifact_combinations() {
        let dynamic_input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 0.75),
                StretchRatioPoint::new(240, 1.25),
            ])
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
        let pitch_input = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 0.75)])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)])
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector);
        let source = stretch_artifact_source(480);

        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &dynamic_input,
                accepted_selector_promotion_receipt("fma-rubberband:selector-dynamic"),
                &source,
            ),
            Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                    path: OfflineHighQualityPath::CompressionShortWindowSelector
                }
            )
        );
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &pitch_input,
                accepted_selector_promotion_receipt("fma-rubberband:selector-pitch"),
                &source,
            ),
            Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift {
                    path: OfflineHighQualityPath::CompressionShortWindowSelector
                }
            )
        );
    }

    #[test]
    fn expansion_short_window_selector_rejects_unproven_artifact_combinations() {
        let dynamic_input = stretch_identity_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.25),
                StretchRatioPoint::new(240, 1.5),
            ])
            .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
        let pitch_input = stretch_identity_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)])
            .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector);
        let source = stretch_artifact_source(480);

        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &dynamic_input,
                accepted_expansion_selector_promotion_receipt(
                    "fma-rubberband:expansion-selector-dynamic"
                ),
                &source,
            ),
            Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                    path: OfflineHighQualityPath::ExpansionShortWindowSelector
                }
            )
        );
        assert_eq!(
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &pitch_input,
                accepted_expansion_selector_promotion_receipt(
                    "fma-rubberband:expansion-selector-pitch"
                ),
                &source,
            ),
            Err(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift {
                    path: OfflineHighQualityPath::ExpansionShortWindowSelector
                }
            )
        );
    }

    #[test]
    fn stretch_artifact_plan_changes_identity_when_projection_changes() {
        let input = stretch_identity_input();
        let changed = StretchCacheIdentityInput {
            projection_epoch: "projection-43".to_string(),
            ..stretch_identity_input()
        };
        let base_plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            StretchPromotionReceipt::default(),
        )
        .expect("artifact plan");
        let changed_plan = plan_offline_stretch_artifact(
            OfflineStretchArtifactScope::RenderCache,
            &changed,
            StretchPromotionReceipt::default(),
        )
        .expect("artifact plan");

        assert_ne!(
            base_plan.identity.stable_hash,
            changed_plan.identity.stable_hash
        );
        assert!(!base_plan.product_facing_allowed);
        assert!(!changed_plan.product_facing_allowed);
    }

    #[test]
    fn stretch_artifact_plan_rejects_preview_or_repitch_tiers() {
        let preview = StretchCacheIdentityInput::signal_native(
            StretchBackendTier::RealtimePreview,
            "sha256:render-source",
            StretchChannelLayout::new(2, 48_000),
            "projection-42",
        );
        let repitch = StretchCacheIdentityInput::signal_native(
            StretchBackendTier::Repitch,
            "sha256:render-source",
            StretchChannelLayout::new(2, 48_000),
            "projection-42",
        );

        assert_eq!(
            plan_offline_stretch_artifact(
                OfflineStretchArtifactScope::Export,
                &preview,
                accepted_product_quality_promotion_receipt("product-quality:ok"),
            ),
            Err(OfflineStretchArtifactPlanError::UnsupportedTier(
                StretchBackendTier::RealtimePreview
            ))
        );
        assert_eq!(
            plan_offline_stretch_artifact(
                OfflineStretchArtifactScope::Freeze,
                &repitch,
                accepted_product_quality_promotion_receipt("product-quality:ok"),
            ),
            Err(OfflineStretchArtifactPlanError::UnsupportedTier(
                StretchBackendTier::Repitch
            ))
        );
    }

    #[test]
    fn offline_render_is_sample_identical_to_a_manual_executor_loop() {
        // Identity gate: render_plan_to_pcm and a hand-rolled
        // controller/executor loop over the same spec and block size must
        // produce byte-identical PCM. Same code path today (this is the
        // point of WYSIWYG bounce); the test exists to catch any future
        // offline-only divergence.
        let spec = reference_spec();
        let options = OfflineRenderOptions {
            start_frame: 960,
            frame_count: 48_000,
            block_frames: 512,
            capture_stage_ids: Vec::new(),
        };
        let output = render_plan_to_pcm(&spec, &options).unwrap();

        let (mut controller, mut executor) = render_plane();
        controller.set_stream_channels(2).unwrap();
        controller.install_plan(&spec).unwrap();
        controller.seek(options.start_frame).unwrap();
        controller.set_playing(true).unwrap();
        executor.drain_commands();
        executor.set_edge_gain_immediate(1.0);
        let mut manual = Vec::new();
        let mut block = vec![0.0f32; 512 * 2];
        let mut remaining = options.frame_count as usize;
        while remaining > 0 {
            let frames_this_block = remaining.min(512);
            let slice = &mut block[..frames_this_block * 2];
            executor.render_block(slice);
            manual.extend_from_slice(slice);
            remaining -= frames_this_block;
        }

        assert_eq!(output.master.len(), manual.len());
        assert!(
            output
                .master
                .iter()
                .zip(manual.iter())
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "offline driver diverged from the manual executor loop",
        );
        assert_eq!(output.channels, 2);
        assert_eq!(output.sample_rate_hz, 48_000);
    }

    #[test]
    fn offline_render_completes_ten_seconds_faster_than_realtime() {
        let spec = reference_spec();
        let options = OfflineRenderOptions {
            frame_count: 480_000, // 10 s at 48 kHz.
            ..OfflineRenderOptions::default()
        };
        let started = std::time::Instant::now();
        let output = render_plan_to_pcm(&spec, &options).unwrap();
        let elapsed = started.elapsed();
        assert_eq!(output.master.len(), 480_000 * 2);
        // Generous bound (debug builds, loaded CI): still far inside the
        // 10 s of audio rendered, proving faster-than-realtime.
        assert!(
            elapsed.as_secs_f64() < 8.0,
            "10 s bounce took {elapsed:?} — not faster than realtime",
        );
    }

    #[test]
    fn unity_stems_sum_to_the_master() {
        // Two lanes at unity through identity edges into a unity master:
        // the captured post-fader stems must sum to the master output.
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane(1, 1.0, vec![tone_clip(11, 440.0)]),
                lane(2, 1.0, vec![tone_clip(21, 553.0)]),
                master(vec![1, 2]),
            ],
        };
        let options = OfflineRenderOptions {
            frame_count: 24_000,
            capture_stage_ids: vec![1, 2],
            ..OfflineRenderOptions::default()
        };
        let output = render_plan_to_pcm(&spec, &options).unwrap();
        assert_eq!(output.stems.len(), 2);
        let (stem_a_id, stem_a) = &output.stems[0];
        let (stem_b_id, stem_b) = &output.stems[1];
        assert_eq!((*stem_a_id, *stem_b_id), (1, 2));
        assert_eq!(stem_a.len(), output.master.len());
        assert_eq!(stem_b.len(), output.master.len());
        for index in 0..output.master.len() {
            let sum = stem_a[index] + stem_b[index];
            assert!(
                (sum - output.master[index]).abs() < 1e-6,
                "stem sum diverged from master at sample {index}: {sum} vs {}",
                output.master[index],
            );
        }
    }

    #[test]
    fn int16_dither_round_trip_stays_within_a_lsb_and_decorrelates() {
        let dir = std::env::temp_dir().join(format!(
            "render-plane-offline-dither-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dither.wav");

        // A slow ramp plus a long constant plateau: the plateau exposes
        // dither decorrelation, the ramp exercises quantization accuracy.
        let mut samples = Vec::new();
        for index in 0..4_000 {
            samples.push((index as f32 / 4_000.0) * 0.5 - 0.25);
        }
        samples.extend(std::iter::repeat_n(0.000_02f32, 4_000));
        write_wav(&path, &samples, 1, 48_000, WavBitDepth::Int16).unwrap();

        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().bits_per_sample, 16);
        let decoded: Vec<i16> = reader.samples::<i16>().map(Result::unwrap).collect();
        assert_eq!(decoded.len(), samples.len());
        let lsb = 1.0 / 32_768.0;
        for (index, (source, quantized)) in samples.iter().zip(decoded.iter()).enumerate() {
            let restored = *quantized as f32 / 32_768.0;
            assert!(
                (restored - source).abs() <= 1.5 * lsb,
                "sample {index} drifted past 1.5 LSB: {source} -> {restored}",
            );
        }
        // The constant plateau sits between integer codes; TPDF dither must
        // toggle adjacent codes rather than collapsing to one value.
        let plateau = &decoded[4_000..];
        assert!(
            plateau.iter().any(|value| *value != plateau[0]),
            "dithered constant plateau quantized to a single code",
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn float32_wav_round_trips_bit_exactly() {
        let dir =
            std::env::temp_dir().join(format!("render-plane-offline-f32-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("float.wav");
        let samples: Vec<f32> = (0..512).map(|index| (index as f32 * 0.01).sin()).collect();
        write_wav(&path, &samples, 2, 44_100, WavBitDepth::Float32).unwrap();
        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, 44_100);
        assert_eq!(reader.spec().channels, 2);
        let decoded: Vec<f32> = reader.samples::<f32>().map(Result::unwrap).collect();
        assert_eq!(decoded.len(), samples.len());
        assert!(samples
            .iter()
            .zip(decoded.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bounce_starts_at_full_level_with_no_transport_fade_in() {
        // A constant-amplitude source mid-clip: the first exported sample
        // must already be at full level. With the realtime edge envelope a
        // 5 ms fade-in would zero the first sample.
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![lane(1, 1.0, vec![constant_clip(11, 0.5)]), master(vec![1])],
        };
        let options = OfflineRenderOptions {
            start_frame: 4_800, // Mid-clip: past the clip edge declick fade.
            frame_count: 256,
            ..OfflineRenderOptions::default()
        };
        let output = render_plan_to_pcm(&spec, &options).unwrap();
        assert!(
            (output.master[0] - 0.5).abs() < 1e-6,
            "first bounce sample read {} — transport fade-in leaked into the export",
            output.master[0],
        );
        assert!(output
            .master
            .iter()
            .all(|sample| (sample - 0.5).abs() < 1e-6));
    }
}
