#![allow(dead_code)]

use std::sync::Arc;

use signal_render_plane::{
    ChannelFormat, OfflineRenderOptions, OfflineStretchArtifactBuildRequest,
    OfflineStretchArtifactPolicyRequest, OfflineStretchArtifactScope, RenderClipSpec,
    RenderEdgeSpec, RenderPlanSpec, RenderSampleBuffer, RenderSource, RenderStageKind,
    RenderStageSpec,
};
use signal_runtime::{
    OfflineHighQualityPath, StretchBackendTier, StretchCacheIdentityInput, StretchChannelLayout,
    StretchPitchPoint, StretchProductQualityEvidence, StretchPromotionReceipt, StretchRatioPoint,
    StretchSyntheticPromotionPolicy, StretchWarpMarker, REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
};

pub const CACHE_CONSUMPTION_STAGE_ID: u64 = 61;

pub fn host_stretch_identity_input(
    content_hash: &str,
    projection_epoch: &str,
) -> StretchCacheIdentityInput {
    StretchCacheIdentityInput::signal_native(
        StretchBackendTier::OfflineHighQuality,
        content_hash,
        StretchChannelLayout::new(2, 48_000),
        projection_epoch,
    )
    .with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)])
    .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
    .with_warp_markers(vec![StretchWarpMarker::new(0, 0)])
}

pub fn host_stretch_source(value: f32, frame_count: usize) -> RenderSampleBuffer {
    RenderSampleBuffer::stereo(
        48_000,
        Arc::from(vec![value; frame_count * 2].into_boxed_slice()),
    )
}

pub fn accepted_stretch_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
    StretchPromotionReceipt::from_product_quality_evidence(
        evidence_id,
        OfflineHighQualityPath::Default,
        StretchProductQualityEvidence {
            compared_to_draft_baseline: true,
            absolute_integrity_passed: true,
            comparator_row_count: 18,
            required_comparator_row_count: 18,
            passed_case_count: 27,
            required_case_count: 27,
            completed_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
            required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
        },
    )
}

pub fn rejected_stretch_policy_request<'a>(
    scope: OfflineStretchArtifactScope,
    identity_input: &'a StretchCacheIdentityInput,
    evidence_id: &'a str,
) -> OfflineStretchArtifactPolicyRequest<'a> {
    stretch_policy_request(
        scope,
        identity_input,
        evidence_id,
        rejected_stretch_policy(),
    )
}

pub fn stretch_policy_request<'a>(
    scope: OfflineStretchArtifactScope,
    identity_input: &'a StretchCacheIdentityInput,
    evidence_id: &'a str,
    promotion_policy: StretchSyntheticPromotionPolicy,
) -> OfflineStretchArtifactPolicyRequest<'a> {
    OfflineStretchArtifactPolicyRequest {
        scope,
        identity_input,
        evidence_id,
        promotion_policy,
    }
}

pub fn stretch_build_request<'a>(
    policy: OfflineStretchArtifactPolicyRequest<'a>,
    source: &'a RenderSampleBuffer,
) -> OfflineStretchArtifactBuildRequest<'a> {
    OfflineStretchArtifactBuildRequest { policy, source }
}

pub fn rejected_stretch_policy() -> StretchSyntheticPromotionPolicy {
    StretchSyntheticPromotionPolicy {
        min_comparison_count: usize::MAX,
        ..StretchSyntheticPromotionPolicy::default()
    }
}

pub fn cache_consumption_options(output_frames: u64) -> OfflineRenderOptions {
    OfflineRenderOptions {
        frame_count: output_frames,
        capture_stage_ids: vec![CACHE_CONSUMPTION_STAGE_ID],
        ..OfflineRenderOptions::default()
    }
}

pub fn cache_consumption_spec(source: RenderSource, output_frames: u64) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                stage_id: CACHE_CONSUMPTION_STAGE_ID,
                kind: RenderStageKind::Source {
                    clips: vec![RenderClipSpec {
                        clip_id: 610,
                        start_frames: 0,
                        end_frames: output_frames,
                        source,
                        loop_source: false,
                    }],
                },
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                processor: None,
                events: None,
                inputs: Vec::new(),
            },
            RenderStageSpec {
                stage_id: 62,
                kind: RenderStageKind::Output,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                processor: None,
                events: None,
                inputs: vec![RenderEdgeSpec {
                    source_stage_id: CACHE_CONSUMPTION_STAGE_ID,
                    gain: 1.0,
                    matrix: None,
                }],
            },
        ],
    }
}
