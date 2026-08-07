//! Offline render-plane unit tests.

use super::stretch_artifact::{
    materialize_offline_stretch_artifact_pcm_with_chunk_config,
    OfflineStretchArtifactCapabilityStatus,
};
use super::*;
use crate::{
    render_plane, ChannelFormat, RenderClipSpec, RenderEdgeSpec, RenderLimiterSpec,
    RenderParamEnvelope, RenderPlanSpec, RenderPluginProcessor, RenderSampleBuffer, RenderSource,
    RenderStageKind, RenderStageSpec,
};
use signal_dsp_stretch::{
    OfflineHighQualityPath, StretchBackendTier, StretchCacheIdentityInput, StretchChannelLayout,
    StretchOfflineChunkConfig, StretchPitchPoint, StretchProductQualityEvidence,
    StretchPromotionReceipt, StretchPromotionStatus, StretchRatioPoint, StretchWarpMarker,
    REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
};
use std::sync::Arc;

const MASTER_ID: u64 = 9_000;
const REQUIRED_SYNTHETIC_CASE_COUNT: u32 = 27;

fn lane(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
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

fn master(inputs: Vec<u64>) -> RenderStageSpec {
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
struct OfflineOnlyGainProcessor {
    offline: std::sync::atomic::AtomicBool,
    bypassed_blocks: std::sync::atomic::AtomicU64,
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
#[test]
fn offline_render_drives_stage_processors_in_offline_waiting() {
    let backend = Arc::new(OfflineOnlyGainProcessor::default());
    let processor = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut sum = master(vec![1]);
    sum.stage_id = 2;
    sum.kind = RenderStageKind::Sum;
    sum.processor = Some(processor.clone());
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 1.0)]),
            sum,
            master(vec![2]),
        ],
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 2_048,
        block_frames: 128,
        capture_stage_ids: Vec::new(),
    };

    let rendered = render_plan_to_pcm(&spec, &options).expect("offline render");

    assert_eq!(
        backend
            .bypassed_blocks
            .load(std::sync::atomic::Ordering::Relaxed),
        0,
        "no block may bypass the insert during an offline render",
    );
    // Past the clip edge fade the source is a 1.0 plateau, so the insert
    // is audible as an exact halving on every remaining sample.
    let guard = 256 * 2;
    assert!(rendered.master.len() > guard);
    for (index, sample) in rendered.master.iter().enumerate().skip(guard) {
        assert!(
            (sample - 0.5).abs() < 1e-6,
            "sample {index}: {sample} (insert dropped for this block)",
        );
    }

    // Restored, not left latched: the same handle may be live on the
    // audio thread after the bounce, where the realtime bound is correct.
    assert!(
        !backend.offline.load(std::sync::atomic::Ordering::Relaxed),
        "offline waiting must be restored when the render ends",
    );
}

fn tone_clip(clip_id: u64, frequency_hz: f32) -> RenderClipSpec {
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
        fade_in_frames: 0,
        fade_out_frames: 0,
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
) -> OfflineStretchArtifactBuildRequest<'a> {
    OfflineStretchArtifactBuildRequest {
        scope: OfflineStretchArtifactScope::RenderCache,
        identity_input,
        promotion_receipt: rejected_promotion_receipt("evidence:cache-bridge-incomplete"),
        source,
    }
}

fn rejected_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
    StretchPromotionReceipt::rejected_offline_high_quality(
        evidence_id,
        0,
        REQUIRED_SYNTHETIC_CASE_COUNT,
        "composite product-quality evidence is incomplete",
    )
}

fn artifact_build_request<'a>(
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

fn incomplete_receipt_build_request<'a>(
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

fn build_offline_stretch_artifact_pcm(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactPcm, OfflineStretchArtifactMaterializeError> {
    materialize_offline_stretch_artifact_pcm(
        request.scope,
        request.identity_input,
        request.promotion_receipt,
        request.source,
    )
}

/// The `g10.039` listening round found the adopted artifact path emitting
/// under four seconds of audio and then silence, with the length made up by
/// zero padding. Every artifact test at the time asserted lengths, identity,
/// and receipts, and none asserted that the audio was there.
///
/// This owner covers a source long enough to cross chunk boundaries and
/// requires signal in every decile of the output.
#[test]
fn multi_chunk_artifact_carries_audio_across_the_whole_output() {
    let identity = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let frames = 48_000 * 90;
    let mut pcm = Vec::with_capacity(frames * 2);
    for f in 0..frames {
        let t = f as f32 / 48_000.0;
        let chord = 0.22 * (std::f32::consts::TAU * 220.0 * t).sin()
            + 0.18 * (std::f32::consts::TAU * 277.18 * t).sin();
        pcm.push(chord);
        pcm.push(chord * 0.85);
    }
    let source = RenderSampleBuffer::stereo(48_000, Arc::from(pcm.into_boxed_slice()));
    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &identity,
        accepted_product_quality_promotion_receipt("evidence-artifact-content"),
        &source,
    )
    .expect("artifact materializes");

    assert!(
        artifact.chunk_plan.chunks.len() > 1,
        "case must cross a chunk boundary"
    );
    let out = &artifact.buffer.frames;
    let total = out.len() / 2;
    let slice = total / 10;
    for part in 0..10 {
        let start = part * slice;
        let seg = &out[start * 2..(start + slice) * 2];
        let rms = (seg.iter().map(|s| (s * s) as f64).sum::<f64>() / seg.len() as f64).sqrt();
        assert!(
            rms > 1.0e-4,
            "artifact decile {part} is silent: rms {rms:.9} at {:.1}s",
            start as f32 / 48_000.0
        );
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
    let required_case_count = REQUIRED_SYNTHETIC_CASE_COUNT;
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

fn assert_product_quality_promotion_receipt(receipt: &StretchPromotionReceipt, evidence_id: &str) {
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

fn max_abs_delta(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0f32, f32::max)
}

#[test]
fn ready_stretch_artifact_materializes_cacheable_pcm_for_render_consumers() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
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
                    fade_in_frames: 0,
                    fade_out_frames: 0,
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
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);
    let artifact = materialize_offline_stretch_artifact_pcm(
        OfflineStretchArtifactScope::RenderCache,
        &input,
        accepted_product_quality_promotion_receipt("product-quality:direct-materialize-current"),
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
        accepted_product_quality_promotion_receipt("product-quality:chunked-materialize-current"),
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
fn composite_quality_artifacts_preserve_cache_identity_and_change_on_inputs() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
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
            accepted_product_quality_promotion_receipt("product-quality:builder-cache-identity"),
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
fn render_cache_handoff_rejects_non_cache_scope_and_incomplete_receipt() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);

    assert_eq!(
        build_offline_stretch_artifact_cache_handoff(artifact_build_request(
            OfflineStretchArtifactScope::Freeze,
            &input,
            rejected_promotion_receipt("evidence:cache-handoff-wrong-scope"),
            &source,
        )),
        Err(
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                scope: OfflineStretchArtifactScope::Freeze
            }
        )
    );
    assert_eq!(
        build_offline_stretch_artifact_cache_handoff(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            rejected_promotion_receipt("evidence:cache-handoff-incomplete"),
            &source,
        )),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
}

#[test]
fn incomplete_receipt_cache_bridge_rejects_without_writing_handoffs() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let changed_projection = StretchCacheIdentityInput {
        projection_epoch: "projection-44".to_string(),
        ..input.clone()
    };
    let source = stretch_artifact_source(480);
    let mut bridge = OfflineStretchArtifactRenderCacheBridge::new();

    assert!(bridge.is_empty());
    assert_eq!(
        bridge.resolve(cache_bridge_request(&input, &source)),
        Err(OfflineStretchArtifactMaterializeError::NotReady(
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        ))
    );
    assert_eq!(
        bridge.resolve(cache_bridge_request(&changed_projection, &source)),
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
fn receipt_owned_cache_bridge_writes_then_hits() {
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(480);
    let mut bridge = OfflineStretchArtifactRenderCacheBridge::new();

    let first = bridge
        .resolve(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:cache-bridge-write"),
            &source,
        ))
        .expect("accepted receipt should write cache handoff");
    assert_eq!(first.kind, OfflineStretchArtifactCacheDecisionKind::Written);
    assert_eq!(bridge.len(), 1);

    let second = bridge
        .resolve(artifact_build_request(
            OfflineStretchArtifactScope::RenderCache,
            &input,
            accepted_product_quality_promotion_receipt("product-quality:cache-bridge-hit"),
            &source,
        ))
        .expect("matching identity should reuse cache handoff");
    assert_eq!(second.kind, OfflineStretchArtifactCacheDecisionKind::Hit);
    assert_eq!(
        second.handoff.cache_identity_hash,
        first.handoff.cache_identity_hash
    );
    assert_eq!(bridge.len(), 1);
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
                .all(|(other_index, (other_hash, _))| index == other_index || hash != other_hash),
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
    let input = stretch_identity_input().with_ratio_curve(vec![StretchRatioPoint::new(0, 1.25)]);
    let source = stretch_artifact_source(1_024);
    let receipt = accepted_product_quality_promotion_receipt("product-quality:chunk-policy-audit");

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
        accepted_product_quality_promotion_receipt("product-quality:realistic-duration-chunked"),
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
                    fade_in_frames: 0,
                    fade_out_frames: 0,
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
fn incomplete_receipt_blocks_static_pitch_with_dynamic_ratio_curve() {
    let input = stretch_identity_input()
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(240, 1.25),
        ])
        .with_pitch_curve(vec![StretchPitchPoint::new(0, 2.0)]);
    let source = stretch_artifact_source(480);

    let result = build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
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
        build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
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
            accepted_product_quality_promotion_receipt("product-quality:direct-pitch-automation",),
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
        .contains("offline_path=compression-short-window-selector"));
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
        .contains("offline_path=expansion-short-window-selector"));
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
        build_offline_stretch_artifact_pcm(incomplete_receipt_build_request(
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
    let dir = std::env::temp_dir().join(format!("render-plane-offline-f32-{}", std::process::id()));
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
/// Fixture backend for the offline param bake: parameter 7 is a linear
/// output gain, settable through the set-parameter seam. Any other id is
/// rejected (`false`), like a real backend refusing an unknown param.
struct ParamGainProcessor {
    gain_bits: std::sync::atomic::AtomicU32,
}

impl ParamGainProcessor {
    const GAIN_PARAM_ID: u32 = 7;

    fn with_gain(gain: f32) -> Self {
        ParamGainProcessor {
            gain_bits: std::sync::atomic::AtomicU32::new(gain.to_bits()),
        }
    }
}

impl crate::PluginBlockProcessor for ParamGainProcessor {
    fn process(&self, scratch: &mut [f32], _frame_count: usize, _channels: usize) -> bool {
        let gain = f32::from_bits(self.gain_bits.load(std::sync::atomic::Ordering::Relaxed));
        for sample in scratch.iter_mut() {
            *sample *= gain;
        }
        true
    }

    fn set_parameter_normalized(&self, parameter_id: u32, normalized: f32) -> bool {
        if parameter_id != Self::GAIN_PARAM_ID {
            return false;
        }
        self.gain_bits
            .store(normalized.to_bits(), std::sync::atomic::Ordering::Relaxed);
        true
    }
}

/// Fixed-gain backend WITHOUT parameter transport (trait default
/// rejects the write): envelopes aimed at it must leave audio untouched.
struct FixedGainProcessor {
    gain: f32,
}

impl crate::PluginBlockProcessor for FixedGainProcessor {
    fn process(&self, scratch: &mut [f32], _frame_count: usize, _channels: usize) -> bool {
        for sample in scratch.iter_mut() {
            *sample *= self.gain;
        }
        true
    }
}

fn processor_sum_stage(
    stage_id: u64,
    input_stage_id: u64,
    processor: crate::RenderPluginProcessor,
    parameter_envelopes: Vec<RenderParamEnvelope>,
) -> RenderStageSpec {
    RenderStageSpec {
        accepts_live_events: false,
        processor: Some(processor),
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain: 1.0,
        gain_automation: None,
        kind: RenderStageKind::Sum,
        inputs: vec![RenderEdgeSpec {
            source_stage_id: input_stage_id,
            gain: 1.0,
            matrix: None,
        }],
        parameter_envelopes,
    }
}

#[test]
fn offline_param_envelope_applies_at_block_boundaries() {
    // DC 0.5 through a param-gain processor swept 0 -> 1 over 1024
    // frames, rendered at 256-frame blocks: the output steps once per
    // block, holding the envelope value sampled at each block START —
    // the recorded block-boundary fidelity bound.
    let processor = crate::RenderPluginProcessor::new(Arc::new(ParamGainProcessor::with_gain(1.0)));
    let envelope = RenderParamEnvelope {
        parameter_id: ParamGainProcessor::GAIN_PARAM_ID,
        points: vec![(0, 0.0), (1_024, 1.0)],
    };
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 0.5)]),
            processor_sum_stage(5, 1, processor, vec![envelope]),
            master(vec![5]),
        ],
    };
    let output = render_plan_to_pcm(
        &spec,
        &OfflineRenderOptions {
            start_frame: 0,
            frame_count: 2_048,
            block_frames: 256,
            ..OfflineRenderOptions::default()
        },
    )
    .unwrap();

    // Sample the left channel mid-block (clear of the 32-frame clip
    // edge declick in block zero).
    let left = |frame: usize| output.master[frame * 2];
    let expectations = [
        (128, 0.0),   // block 0: envelope(0) = 0.0
        (384, 0.125), // block 1: envelope(256) = 0.25 -> 0.5 * 0.25
        (640, 0.25),  // block 2: envelope(512) = 0.5
        (896, 0.375), // block 3: envelope(768) = 0.75
        (1_152, 0.5), // block 4: envelope(1024) = 1.0
        (1_920, 0.5), // past the last point: end value held
    ];
    for (frame, expected) in expectations {
        assert!(
            (left(frame) - expected).abs() < 1e-4,
            "frame {frame}: read {} expected {expected}",
            left(frame),
        );
    }
    // The steps land AT block boundaries: constant within a block.
    assert!((left(300) - left(500)).abs() < 1e-6);
    assert!((left(260) - left(510)).abs() < 1e-6);
    // Non-static overall: the sweep is audible in the bounce.
    assert!((left(1_152) - left(384)).abs() > 0.3);
}

#[test]
fn param_envelope_on_transportless_backend_leaves_render_byte_identical() {
    // A backend without parameter transport rejects the set-parameter
    // write (trait default): the envelope must change NOTHING.
    let build = |parameter_envelopes: Vec<RenderParamEnvelope>| {
        let processor =
            crate::RenderPluginProcessor::new(Arc::new(FixedGainProcessor { gain: 0.7 }));
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane(1, 1.0, vec![constant_clip(11, 0.5)]),
                processor_sum_stage(5, 1, processor, parameter_envelopes),
                master(vec![5]),
            ],
        }
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 1_024,
        block_frames: 128,
        ..OfflineRenderOptions::default()
    };
    let with_envelope = render_plan_to_pcm(
        &build(vec![RenderParamEnvelope {
            parameter_id: 3,
            points: vec![(0, 0.0), (512, 1.0)],
        }]),
        &options,
    )
    .unwrap();
    let without_envelope = render_plan_to_pcm(&build(Vec::new()), &options).unwrap();
    assert_eq!(with_envelope.master.len(), without_envelope.master.len());
    assert!(with_envelope
        .master
        .iter()
        .zip(without_envelope.master.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits()));
}

#[test]
fn param_envelopes_reject_processorless_and_unsorted_stages() {
    let processorless = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            {
                let mut stage = lane(1, 1.0, vec![constant_clip(11, 0.5)]);
                stage.parameter_envelopes = vec![RenderParamEnvelope {
                    parameter_id: 7,
                    points: vec![(0, 0.5)],
                }];
                stage
            },
            master(vec![1]),
        ],
    };
    let options = OfflineRenderOptions {
        frame_count: 64,
        ..OfflineRenderOptions::default()
    };
    assert!(render_plan_to_pcm(&processorless, &options).is_err());

    let processor = crate::RenderPluginProcessor::new(Arc::new(ParamGainProcessor::with_gain(1.0)));
    let unsorted = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 0.5)]),
            processor_sum_stage(
                5,
                1,
                processor,
                vec![RenderParamEnvelope {
                    parameter_id: 7,
                    points: vec![(512, 1.0), (0, 0.0)],
                }],
            ),
            master(vec![5]),
        ],
    };
    assert!(render_plan_to_pcm(&unsorted, &options).is_err());
}

#[test]
fn envelope_value_sampling_interpolates_and_holds_ends() {
    let envelope = RenderParamEnvelope {
        parameter_id: 1,
        points: vec![(100, 0.2), (300, 0.8)],
    };
    assert_eq!(envelope.value_at(0), Some(0.2)); // held before first
    assert_eq!(envelope.value_at(100), Some(0.2));
    assert!((envelope.value_at(200).unwrap() - 0.5).abs() < 1e-6);
    assert_eq!(envelope.value_at(300), Some(0.8));
    assert_eq!(envelope.value_at(9_999), Some(0.8)); // held past last
    let empty = RenderParamEnvelope {
        parameter_id: 1,
        points: Vec::new(),
    };
    assert_eq!(empty.value_at(0), None);
}

#[test]
fn offline_soft_limiter_matches_the_known_transfer_curve() {
    // Constant 0.9 through threshold 0.5 / knee 0.2: knee_start 0.4,
    // knee_end 0.6, knee_end_output 0.55, saturation_range 0.45 ->
    // transfer(0.9) = 1 - 0.45^2 / (0.5 * 0.3 + 0.45) = 0.6625. Attack
    // is instant, so EVERY frame of a constant over-threshold signal
    // lands exactly on the curve.
    let spec = RenderLimiterSpec {
        threshold: 0.5,
        knee_width: 0.2,
        release_seconds: 0.05,
    };
    let mut samples = vec![0.9f32; 4_800 * 2];
    apply_soft_limiter_to_pcm(&mut samples, 2, 48_000, &spec);
    let expected = 0.6625f32;
    assert!(samples
        .iter()
        .all(|sample| (sample - expected).abs() < 1e-4));
    assert!(samples.iter().all(|sample| *sample <= 1.0));

    // Below the knee start the limiter is bit-transparent (gain 1.0).
    let mut quiet = vec![0.3f32; 480 * 2];
    apply_soft_limiter_to_pcm(&mut quiet, 2, 48_000, &spec);
    assert!(quiet
        .iter()
        .all(|sample| sample.to_bits() == 0.3f32.to_bits()));
}
