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

use std::{path::Path, sync::Arc};

use crate::{
    render_plane, RenderPlanSpec, RenderPlaneError, RenderPlaneExecutor, RenderSampleBuffer,
    MAX_BLOCK_FRAMES,
};
use signal_dsp_stretch::{
    stretch_backend_plan, OfflineHighQualityStretcher, StretchBackendStatus, StretchBackendTier,
    StretchCacheIdentity, StretchCacheIdentityError, StretchCacheIdentityInput,
    StretchPromotionReceipt, StretchRatioPoint,
};
use signal_primitives::SampleRate;

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
    /// The artifact may be consumed by render/export/freeze callers.
    Ready,
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
    /// Current implementation/evidence readiness.
    pub readiness: OfflineStretchArtifactReadiness,
    /// Promotion evidence associated with this artifact plan.
    pub promotion_receipt: StretchPromotionReceipt,
    /// Whether the artifact is allowed to feed product-facing render/export output.
    pub product_facing_allowed: bool,
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
    /// Source frame count consumed from `source`.
    pub input_frame_count: usize,
    /// Output frame count produced in `buffer`.
    pub output_frame_count: usize,
}

/// Receipt for one materialized offline stretch artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineStretchArtifactMaterializationReceipt {
    /// Consumer scope this artifact was materialized for.
    pub scope: OfflineStretchArtifactScope,
    /// Signal stretch tier used to produce the artifact.
    pub tier: StretchBackendTier,
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
    let promotion_accepted = promotion_receipt.accepts_product_facing_use(identity_input.tier);
    let readiness = match (backend.status, promotion_accepted) {
        (StretchBackendStatus::Planned, _) => {
            OfflineStretchArtifactReadiness::AwaitingImplementation
        }
        (StretchBackendStatus::Prototype, _) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (StretchBackendStatus::Implemented, false) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (StretchBackendStatus::Implemented, true) => OfflineStretchArtifactReadiness::Ready,
    };

    Ok(OfflineStretchArtifactPlan {
        scope,
        identity,
        tier: identity_input.tier,
        readiness,
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
    let plan = plan_offline_stretch_artifact(scope, identity_input, promotion_receipt)?;
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
    let mut stretcher = OfflineHighQualityStretcher::new(ratio);
    let frames = if pitch_shift.abs() > 1.0e-9 {
        stretcher.stretch_dynamic_ratio_pitch_interleaved_stereo(
            &source.frames,
            &identity_input.ratio_curve,
            SampleRate(source.sample_rate_hz),
            pitch_shift,
        )
    } else if identity_input.ratio_curve.is_empty() {
        stretcher.stretch_interleaved_stereo(&source.frames)
    } else {
        stretcher
            .stretch_dynamic_ratio_interleaved_stereo(&source.frames, &identity_input.ratio_curve)
    };

    let output_frame_count = frames.len() / 2;
    let receipt = OfflineStretchArtifactMaterializationReceipt {
        scope,
        tier: plan.tier,
        cache_identity_hash: plan.identity.stable_hash.clone(),
        cache_identity_key: plan.identity.canonical_key.clone(),
        promotion_evidence_id: plan.promotion_receipt.evidence_id.clone(),
        input_frame_count: source.frame_count(),
        output_frame_count,
        channels: identity_input.channel_layout.channels,
        sample_rate_hz: source.sample_rate_hz,
        product_facing_allowed: plan.product_facing_allowed,
    };
    Ok(OfflineStretchArtifactPcm {
        plan,
        receipt,
        buffer: RenderSampleBuffer {
            sample_rate_hz: source.sample_rate_hz,
            frames: Arc::from(frames.into_boxed_slice()),
        },
        input_frame_count: source.frame_count(),
        output_frame_count,
    })
}

fn static_or_initial_ratio(ratio_curve: &[StretchRatioPoint]) -> f64 {
    ratio_curve
        .iter()
        .find(|point| point.ratio.is_finite() && point.ratio > 0.0)
        .map(|point| point.ratio)
        .unwrap_or(1.0)
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
        current_synthetic_offline_high_quality_promotion_receipt, StretchChannelLayout,
        StretchPitchPoint, StretchPromotionReceipt, StretchRatioPoint, StretchWarpMarker,
    };
    use std::sync::Arc;

    const MASTER_ID: u64 = 9_000;

    fn lane(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
        RenderStageSpec {
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
            source: RenderSource::Samples(RenderSampleBuffer {
                sample_rate_hz: 48_000,
                frames: Arc::from(data.into_boxed_slice()),
            }),
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
        RenderSampleBuffer {
            sample_rate_hz: 48_000,
            frames: Arc::from(frames.into_boxed_slice()),
        }
    }

    fn accepted_synthetic_promotion_receipt(evidence_id: &str) -> StretchPromotionReceipt {
        current_synthetic_offline_high_quality_promotion_receipt(evidence_id)
    }

    #[test]
    fn stretch_artifact_plan_allows_export_with_accepted_synthetic_promotion() {
        let input = stretch_identity_input();
        let promotion = accepted_synthetic_promotion_receipt("synthetic:render-plane-current");
        let plan =
            plan_offline_stretch_artifact(OfflineStretchArtifactScope::Export, &input, promotion)
                .expect("artifact plan");

        assert_eq!(plan.scope, OfflineStretchArtifactScope::Export);
        assert_eq!(plan.tier, StretchBackendTier::OfflineHighQuality);
        assert_eq!(plan.readiness, OfflineStretchArtifactReadiness::Ready);
        assert!(plan
            .promotion_receipt
            .accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
        assert!(plan.product_facing_allowed);
        assert!(plan
            .identity
            .canonical_key
            .contains("tier=OfflineHighQuality"));
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
            accepted_synthetic_promotion_receipt("synthetic:materialize-current"),
            &source,
        )
        .expect("ready artifact should materialize");

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
            artifact.receipt.promotion_evidence_id,
            "synthetic:materialize-current"
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
    fn stretch_artifact_materializes_static_pitch_with_dynamic_ratio_curve() {
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
            accepted_synthetic_promotion_receipt("synthetic:static-pitch-dynamic-ratio"),
            &source,
        )
        .expect("static pitch plus dynamic ratio should materialize");

        assert_eq!(artifact.output_frame_count, 540);
        assert_eq!(artifact.buffer.frame_count(), artifact.output_frame_count);
        assert_eq!(
            artifact.receipt.promotion_evidence_id,
            "synthetic:static-pitch-dynamic-ratio"
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
            materialize_offline_stretch_artifact_pcm(
                OfflineStretchArtifactScope::RenderCache,
                &input,
                accepted_synthetic_promotion_receipt("synthetic:pitch-automation"),
                &source,
            ),
            Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
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
                accepted_synthetic_promotion_receipt("stretch-corpus:ok"),
            ),
            Err(OfflineStretchArtifactPlanError::UnsupportedTier(
                StretchBackendTier::RealtimePreview
            ))
        );
        assert_eq!(
            plan_offline_stretch_artifact(
                OfflineStretchArtifactScope::Freeze,
                &repitch,
                accepted_synthetic_promotion_receipt("stretch-corpus:ok"),
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
