//! Offline bounce driver: drive the realtime executor faster-than-realtime.

use signal_primitives::SampleRate;

use crate::{
    render_plane, RenderLimiterSpec, RenderParamEnvelope, RenderPlanSpec, RenderPlaneError,
    RenderPlaneExecutor, RenderPluginProcessor, MAX_BLOCK_FRAMES,
};

use super::{OfflineRenderOptions, OfflineRenderOutput};

/// Holds every stage processor in offline waiting for the length of a render,
/// restoring each one's previous setting on drop — including on the `?` early
/// returns inside the render.
///
/// Without this a plugin insert is silently dropped for any block its backend
/// misses. A realtime callback misses because it must return before its buffer
/// drains; an offline render has no such deadline, so the same miss is not a
/// late block but a wrong one, and it happens at block boundaries that depend
/// on machine load rather than on the plan. See
/// [`crate::PluginBlockProcessor::set_offline_waiting`].
struct OfflineWaitingGuard {
    restore: Vec<(RenderPluginProcessor, bool)>,
}

impl OfflineWaitingGuard {
    fn install(spec: &RenderPlanSpec) -> Self {
        Self {
            restore: spec
                .stages
                .iter()
                .filter_map(|stage| stage.processor.as_ref())
                .map(|processor| (processor.clone(), processor.set_offline_waiting(true)))
                .collect(),
        }
    }
}

impl Drop for OfflineWaitingGuard {
    fn drop(&mut self) {
        // Reverse order so one processor carried by several stages restores
        // the setting it had before the first flip, not after it.
        for (processor, previous) in self.restore.iter().rev() {
            processor.set_offline_waiting(*previous);
        }
    }
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
    let _offline_waiting = OfflineWaitingGuard::install(spec);
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

    // Offline param bake (g13.029): stages carrying parameter envelopes get
    // them applied at every block boundary through the processor
    // set-parameter seam — the offline mirror of the host's live playback-
    // poll forwarding. Envelope-less plans build an empty list and the
    // render loop below is byte-identical to the pre-envelope driver.
    let mut envelope_appliers: Vec<ParamEnvelopeApplier> = spec
        .stages
        .iter()
        .filter_map(|stage| {
            let processor = stage.processor.as_ref()?;
            if stage.parameter_envelopes.is_empty() {
                return None;
            }
            Some(
                stage
                    .parameter_envelopes
                    .iter()
                    .map(|envelope| ParamEnvelopeApplier {
                        processor: processor.clone(),
                        envelope: envelope.clone(),
                        last_applied: None,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect();

    let block_frames = options.block_frames.clamp(1, MAX_BLOCK_FRAMES);
    let mut block = vec![0.0f32; block_frames * channels as usize];
    let mut master = Vec::with_capacity(options.frame_count as usize * channels as usize);
    let mut remaining = options.frame_count;
    while remaining > 0 {
        let block_start_frame = options.start_frame + (options.frame_count - remaining);
        for applier in &mut envelope_appliers {
            applier.apply_at(block_start_frame);
        }
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

/// One (processor, envelope) pair being applied across the offline render:
/// the envelope is sampled at each block-start frame (linear interpolation,
/// end values held) and pushed through the processor set-parameter seam
/// when the sampled value moves. Backends without parameter transport
/// reject the write (`set_parameter_normalized` returns `false`) and the
/// audio path stays untouched — honest bypass, no partial application.
struct ParamEnvelopeApplier {
    processor: RenderPluginProcessor,
    envelope: RenderParamEnvelope,
    last_applied: Option<f32>,
}

impl ParamEnvelopeApplier {
    fn apply_at(&mut self, frame: u64) {
        let Some(value) = self.envelope.value_at(frame) else {
            return;
        };
        let value = value.clamp(0.0, 1.0);
        if self.last_applied == Some(value) {
            return;
        }
        // The applied cache tracks delivery attempts, not acceptance: a
        // backend that rejects the id would reject it every block, so
        // retrying per block is pure overhead.
        self.processor
            .set_parameter_normalized(self.envelope.parameter_id, value);
        self.last_applied = Some(value);
    }
}

/// Apply the engine's linked soft master limiter to interleaved PCM,
/// offline (g13.029 delivery post-chain seam).
///
/// This is the SAME `LimiterState` the realtime executor runs for
/// [`RenderPlanSpec::master_limiter`] (instant attack, one-pole release,
/// linked max-abs detection across channels), driven control-side over an
/// already-rendered buffer. The limiter's output ceiling is FIXED at
/// 0 dBFS (linear 1.0, approached asymptotically); `spec.threshold` centers
/// the knee below it. Consumers needing an arbitrary ceiling scale the
/// buffer up by `1 / ceiling` before this call and back down after —
/// sample peaks then stay below `ceiling` by construction (true peak is a
/// measurement, not a guarantee: the limiter bounds sample peaks only).
pub fn apply_soft_limiter_to_pcm(
    samples: &mut [f32],
    channels: u16,
    sample_rate_hz: u32,
    spec: &RenderLimiterSpec,
) {
    let channels = usize::from(channels.max(1));
    let mut limiter = signal_dsp::LimiterState::new(
        SampleRate(sample_rate_hz.max(1)),
        spec.threshold,
        spec.knee_width,
        signal_primitives::Seconds(spec.release_seconds),
    );
    for frame in samples.chunks_exact_mut(channels) {
        limiter.process_frame(frame);
    }
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
