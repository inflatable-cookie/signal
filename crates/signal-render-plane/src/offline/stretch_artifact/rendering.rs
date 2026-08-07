//! Offline stretch-artifact PCM materialization and resumable rendering.

use std::sync::Arc;

use signal_dsp_stretch::{
    plan_offline_stretch_chunks, OfflineHighQualityPath, OfflineHighQualityStretcher,
    ResumableOfflineStretch, ResumableStretchConfig, StretchCacheIdentityInput,
    StretchOfflineChunkConfig, StretchOfflineChunkPlan, StretchPromotionReceipt, StretchRatioPoint,
    DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
};
use signal_primitives::SampleRate;

use crate::RenderSampleBuffer;

use super::planning::{
    materialization_error_for_capability, plan_offline_stretch_artifact,
    ratio_curve_has_dynamic_changes, selector_offline_path_requires_static_materialization,
    static_or_initial_ratio, static_pitch_shift,
};
use super::types::{
    OfflineStretchArtifactMaterializationReceipt, OfflineStretchArtifactMaterializeError,
    OfflineStretchArtifactPcm, OfflineStretchArtifactReadiness, OfflineStretchArtifactScope,
};

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
