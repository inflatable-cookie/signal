use super::*;

pub(super) const MASTER_ID: u64 = 9_000;
pub(super) const REQUIRED_SYNTHETIC_CASE_COUNT: u32 = 27;

pub(super) fn lane(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
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

pub(super) fn master(inputs: Vec<u64>) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
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

/// Halves the block, but only while it is in offline waiting — the stand-in
/// for a backend whose realtime budget the child misses under load.
#[derive(Default)]
pub(super) struct OfflineOnlyGainProcessor {
    pub(super) offline: std::sync::atomic::AtomicBool,
    pub(super) bypassed_blocks: std::sync::atomic::AtomicU64,
}

impl crate::PluginBlockProcessor for OfflineOnlyGainProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        if !self.offline.load(std::sync::atomic::Ordering::Relaxed) {
            self.bypassed_blocks
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return false;
        }
        for sample in scratch[..frame_count * channels].iter_mut() {
            *sample *= 0.5;
        }
        true
    }

    fn set_offline_waiting(&self, enabled: bool) -> bool {
        self.offline
            .swap(enabled, std::sync::atomic::Ordering::Relaxed)
    }
}

/// A bypassed block in an offline render is not a late block, it is a
/// wrong render: the insert silently vanishes for that block. The driver
/// must therefore put every stage processor into offline waiting, and put
/// it back afterwards so a handle shared with live playback keeps its
/// realtime bound.
pub(super) fn tone_clip(clip_id: u64, frequency_hz: f32) -> RenderClipSpec {
    RenderClipSpec {
        clip_id,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::TestTone { frequency_hz },
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    }
}

/// Constant-value looped stereo sample clip: a DC plateau that reads at
/// exactly `value` once past the clip edge fade.
pub(super) fn constant_clip(clip_id: u64, value: f32) -> RenderClipSpec {
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
        fade_in_frames: 0,
        fade_out_frames: 0,
    }
}

pub(super) fn reference_spec() -> RenderPlanSpec {
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

pub(super) fn stretch_identity_input() -> StretchCacheIdentityInput {
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

pub(super) fn stretch_artifact_source(frame_count: usize) -> RenderSampleBuffer {
    let mut frames = Vec::with_capacity(frame_count * 2);
    for frame in 0..frame_count {
        let sample = (frame as f32 / 17.0).sin() * 0.25;
        frames.push(sample);
        frames.push(sample * 0.75);
    }
    RenderSampleBuffer::stereo(48_000, Arc::from(frames.into_boxed_slice()))
}

pub(super) fn cache_bridge_request<'a>(
    identity_input: &'a StretchCacheIdentityInput,
    source: &'a RenderSampleBuffer,
) -> OfflineStretchArtifactBuildRequest<'a> {
    OfflineStretchArtifactBuildRequest {
        scope: OfflineStretchArtifactScope::RenderCache,
        identity_input,
        promotion_receipt: rejected_promotion_receipt("evidence:cache-bridge-incomplete"),
        source,
    }
}

pub(super) fn rejected_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
    StretchPromotionReceipt::rejected_offline_high_quality(
        evidence_id,
        0,
        REQUIRED_SYNTHETIC_CASE_COUNT,
        "composite product-quality evidence is incomplete",
    )
}

pub(super) fn artifact_build_request<'a>(
    scope: OfflineStretchArtifactScope,
    identity_input: &'a StretchCacheIdentityInput,
    promotion_receipt: StretchPromotionReceipt,
    source: &'a RenderSampleBuffer,
) -> OfflineStretchArtifactBuildRequest<'a> {
    OfflineStretchArtifactBuildRequest {
        scope,
        identity_input,
        promotion_receipt,
        source,
    }
}

pub(super) fn incomplete_receipt_build_request<'a>(
    scope: OfflineStretchArtifactScope,
    identity_input: &'a StretchCacheIdentityInput,
    evidence_id: &'a str,
    source: &'a RenderSampleBuffer,
) -> OfflineStretchArtifactBuildRequest<'a> {
    artifact_build_request(
        scope,
        identity_input,
        rejected_promotion_receipt(evidence_id),
        source,
    )
}

pub(super) fn build_offline_stretch_artifact_pcm(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactPcm, OfflineStretchArtifactMaterializeError> {
    materialize_offline_stretch_artifact_pcm(
        request.scope,
        request.identity_input,
        request.promotion_receipt,
        request.source,
    )
}

pub(super) fn complete_product_quality_evidence(
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

pub(super) fn accepted_product_quality_promotion_receipt(
    evidence_id: &str,
) -> StretchPromotionReceipt {
    let required_case_count = REQUIRED_SYNTHETIC_CASE_COUNT;
    let receipt = StretchPromotionReceipt::from_product_quality_evidence(
        evidence_id,
        OfflineHighQualityPath::Default,
        complete_product_quality_evidence(required_case_count, required_case_count),
    );
    assert_product_quality_promotion_receipt(&receipt, evidence_id);
    receipt
}

pub(super) fn accepted_selector_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
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

pub(super) fn accepted_expansion_selector_promotion_receipt(
    evidence_id: &str,
) -> StretchPromotionReceipt {
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

pub(super) fn assert_product_quality_promotion_receipt(
    receipt: &StretchPromotionReceipt,
    evidence_id: &str,
) {
    assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
    assert_eq!(receipt.evidence_id, evidence_id);
    assert!(receipt.compared_to_draft_baseline);
    assert!(receipt.absolute_integrity_passed);
    assert!(receipt.comparator_row_count >= receipt.required_comparator_row_count);
    assert_eq!(receipt.required_case_count, REQUIRED_SYNTHETIC_CASE_COUNT);
    assert_eq!(receipt.offline_path, OfflineHighQualityPath::Default);
    assert!(receipt.passed_case_count >= receipt.required_case_count);
    assert_eq!(
        receipt.completed_listening_family_count,
        REQUIRED_STRETCH_LISTENING_FAMILY_COUNT
    );
    assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
}

pub(super) fn max_abs_delta(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0f32, f32::max)
}
