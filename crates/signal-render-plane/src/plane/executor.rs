//! Audio-thread render plane executor.

use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;
use std::time::Instant;

use signal_dsp::DenormalGuard;

use crate::plan::{CompiledNode, RenderPlan};
use crate::plan_render::{
    insertion_sort_events_by_offset, render_clips_into_scratch, sample_envelope,
};
use crate::plugin_events::append_plugin_state_chase;
use crate::sample_buffer::XRUN_INTERVAL_FACTOR;
use crate::{
    RenderBlockPluginEvent, LIVE_EVENT_PUSH_CAPACITY, MAX_BLOCK_FRAMES, METER_SLOT_CAPACITY,
    PLUGIN_EVENTS_PER_BLOCK_CAPACITY,
};

use super::command::{RenderCommand, SharedState};
use super::{EDGE_RAMP_SECONDS, GAIN_SMOOTHING_SECONDS, LOOP_WRAP_FADE_FRAMES};

/// Audio-thread executor: drains the command mailbox between blocks and
/// runs [`RenderPlaneExecutor::render_block`] with a hard no-alloc contract.
pub struct RenderPlaneExecutor {
    pub(crate) commands: Receiver<RenderCommand>,
    pub(crate) retired: SyncSender<Box<RenderPlan>>,
    pub(crate) shared: Arc<SharedState>,
    pub(crate) plan: Option<Box<RenderPlan>>,
    pub(crate) parked_retired: Option<Box<RenderPlan>>,
    /// Stream channel count as told by the controller; the active plan's
    /// expectation (its master format when no stream was known at install)
    /// applies until the first [`RenderCommand::SetStreamChannels`].
    pub(crate) stream_channels: Option<u16>,
    /// Transport gate target. Audio follows through `edge_gain`.
    pub(crate) playing: bool,
    /// Live render posture (g13.018): while set, stages render even when the
    /// transport is stopped (live monitoring and live events stay audible);
    /// compiled timeline content stays `playing`-gated and the position does
    /// not advance while stopped.
    pub(crate) live_render: bool,
    /// True while timeline content is still winding down after a stop: set
    /// while playing and held through the stop edge ramp-out, cleared once
    /// the edge envelope closes (or immediately under the live render
    /// posture, whose stop cuts timeline content at the block boundary).
    /// Distinguishes "ramping out after playback" from "edge held open by
    /// the live render posture" — only the former renders clips and
    /// advances the position while `playing` is false.
    pub(crate) timeline_tail: bool,
    /// Transport edge envelope: ramps toward 1 when playing, 0 when stopped
    /// or before an in-flight seek, so transport actions never step audio.
    pub(crate) edge_gain: f32,
    /// Seek requested while audible: applied once `edge_gain` reaches zero.
    pub(crate) pending_seek: Option<u64>,
    /// Timeline position abandoned by the most recently applied seek. The
    /// next event-bearing block uses it to release old notes and chase the
    /// destination state, then clears it.
    pub(crate) event_discontinuity_from: Option<u64>,
    /// Transport loop region `[start, end)`; playback wraps to `start` when
    /// a rendered block crosses `end` (see `render_block_inner`).
    pub(crate) loop_region: Option<(u64, u64)>,
    pub(crate) position_frames: u64,
    /// Start instant of the previous callback, for xrun inference.
    pub(crate) last_callback_instant: Option<Instant>,
}

impl RenderPlaneExecutor {
    /// Offline-bounce hook: snap the transport edge envelope to `gain`
    /// without ramping. Realtime transport must NEVER use this — the 5 ms
    /// edge ramp is what keeps play/stop/seek click-free on speakers — but
    /// an offline export has no speaker and must start at full level, so
    /// the offline driver (`offline::render_plan_to_pcm`) snaps the
    /// envelope open after draining transport commands and before the first
    /// rendered block. Crate-private on purpose: hosts cannot reach it.
    pub(crate) fn set_edge_gain_immediate(&mut self, gain: f32) {
        self.edge_gain = gain.clamp(0.0, 1.0);
    }

    fn retire(&mut self, plan: Box<RenderPlan>) {
        // Re-offer any parked plan first to preserve retirement order.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }
        if let Err(TrySendError::Full(plan)) | Err(TrySendError::Disconnected(plan)) =
            self.retired.try_send(plan)
        {
            self.shared
                .retired_parked_blocks
                .fetch_add(1, Ordering::Relaxed);
            debug_assert!(
                self.parked_retired.is_none(),
                "retired mailbox capacity invariant violated",
            );
            // Never deallocate on the audio thread: hold the plan and
            // re-offer it next block. The mailbox capacity invariant makes
            // double saturation unreachable in correct use.
            if self.parked_retired.is_none() {
                self.parked_retired = Some(plan);
            }
        }
    }

    fn apply_seek(&mut self, position_frames: u64) {
        self.event_discontinuity_from = Some(self.position_frames);
        self.position_frames = position_frames;
        self.shared
            .position_frames
            .store(position_frames, Ordering::Relaxed);
    }

    pub(crate) fn drain_commands(&mut self) {
        loop {
            match self.commands.try_recv() {
                Ok(RenderCommand::InstallPlan(mut next_plan)) => {
                    if let Some(mut previous) = self.plan.take() {
                        next_plan.inherit_state(&mut previous);
                        self.retire(previous);
                    }
                    self.plan = Some(next_plan);
                }
                Ok(RenderCommand::SetPlaying(playing)) => {
                    self.playing = playing;
                    self.shared.playing.store(playing, Ordering::Relaxed);
                }
                Ok(RenderCommand::Seek(position_frames)) => {
                    if self.edge_gain <= 0.0 {
                        // Inaudible: jump immediately.
                        self.pending_seek = None;
                        self.apply_seek(position_frames);
                    } else {
                        // Audible: ramp out first, jump at the zero crossing,
                        // ramp back in (handled in render_block).
                        self.pending_seek = Some(position_frames);
                    }
                }
                Ok(RenderCommand::SetStageGain {
                    stage_index,
                    target,
                }) => {
                    if let Some(plan) = self.plan.as_mut() {
                        if let Some(stage) = plan.stages.get_mut(stage_index) {
                            stage.gain_target = target;
                        }
                    }
                }
                Ok(RenderCommand::SetLoopRegion(region)) => {
                    self.loop_region = region;
                }
                Ok(RenderCommand::SetLiveRender { active }) => {
                    self.live_render = active;
                    self.shared.live_render.store(active, Ordering::Relaxed);
                }
                Ok(RenderCommand::PushLiveEvents {
                    stage_index,
                    events,
                    len,
                }) => {
                    // Append into the stage's preallocated ring; anything
                    // that cannot land (no plan, stage gone or no longer
                    // accepting after a reinstall raced the push, ring full)
                    // drops and counts — never blocks, never allocates.
                    let len = len.min(LIVE_EVENT_PUSH_CAPACITY);
                    let mut accepted = 0usize;
                    if let Some(plan) = self.plan.as_mut() {
                        if let Some(stage) = plan.stages.get_mut(stage_index) {
                            if stage.accepts_live_events {
                                for event in events.iter().take(len) {
                                    if stage.live_events.len() < stage.live_events.capacity() {
                                        stage.live_events.push(*event);
                                        accepted += 1;
                                    }
                                }
                            }
                        }
                    }
                    if accepted < len {
                        self.shared
                            .live_event_drop_count
                            .fetch_add((len - accepted) as u64, Ordering::Relaxed);
                    }
                }
                Ok(RenderCommand::SetStreamChannels(channels)) => {
                    self.stream_channels = Some(channels.max(1));
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Render one callback quantum into `frames` (interleaved f32 at the
    /// stream's channel count).
    ///
    /// Safe on the audio thread: no allocation, no locks, no I/O.
    /// (`Instant::now()` is the one syscall-shaped exception, used for
    /// callback-health timing: it is `mach_absolute_time` on macOS and
    /// `clock_gettime(CLOCK_MONOTONIC)` elsewhere — no allocation, no lock,
    /// vDSO/commpage fast paths — and is the accepted way to observe
    /// callback cadence.)
    pub fn render_block(&mut self, frames: &mut [f32]) {
        let callback_start = Instant::now();
        // Flush-to-zero for the whole callback: feedback DSP (limiter
        // release, future filters) cannot decay into denormal range and burn
        // CPU. RAII restores the FP control register on exit.
        let _denormals = DenormalGuard::new();
        self.render_block_inner(frames);
        self.publish_callback_health(callback_start, frames.len());
    }

    /// Publish callback-health counters: count, duration (last/max), and
    /// inferred xruns. An xrun is an interval since the previous callback
    /// longer than [`XRUN_INTERVAL_FACTOR`] × the block duration at the
    /// active plan's rate; without a plan no xrun can be inferred (the
    /// expected cadence is unknown) but count and duration still publish.
    fn publish_callback_health(&mut self, callback_start: Instant, samples_len: usize) {
        let shared = &self.shared;
        shared.callback_count.fetch_add(1, Ordering::Relaxed);
        let duration_micros = callback_start.elapsed().as_micros() as u64;
        shared
            .last_callback_duration_micros
            .store(duration_micros, Ordering::Relaxed);
        shared
            .max_callback_duration_micros
            .fetch_max(duration_micros, Ordering::Relaxed);
        if let (Some(previous), Some(plan)) = (self.last_callback_instant, self.plan.as_ref()) {
            let stream_channels = self
                .stream_channels
                .map(|channels| channels as usize)
                .unwrap_or(plan.stream_channels)
                .max(1);
            let frame_count = samples_len / stream_channels;
            let block_seconds = frame_count as f64 / plan.sample_rate_hz.max(1) as f64;
            let interval = callback_start.duration_since(previous).as_secs_f64();
            if frame_count > 0 && interval > block_seconds * XRUN_INTERVAL_FACTOR {
                shared.xrun_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.last_callback_instant = Some(callback_start);
    }

    /// Write per-stage peak/RMS for this block into the shared meter table
    /// and stamp the plan generation. Levels are taken from each stage's
    /// scratch (pre-consumption) and scaled by the stage's end-of-block
    /// smoothed gain so fader moves read on the meters — a per-block
    /// approximation of the post-fader level (the transport edge ramp is
    /// not included). Cheap loops over already-rendered scratch: no
    /// allocation. Stages past [`METER_SLOT_CAPACITY`] are unmetered.
    fn publish_meters(shared: &SharedState, plan: &RenderPlan, frame_count: usize) {
        for (index, stage) in plan.stages.iter().take(METER_SLOT_CAPACITY).enumerate() {
            let samples = &stage.scratch[..frame_count * stage.channels];
            let mut peak = 0.0f32;
            let mut sum_squares = 0.0f32;
            for sample in samples {
                let magnitude = sample.abs();
                if magnitude > peak {
                    peak = magnitude;
                }
                sum_squares += sample * sample;
            }
            let gain = stage.gain_current.abs();
            let rms = (sum_squares / samples.len().max(1) as f32).sqrt() * gain;
            let slot = &shared.meter_slots[index];
            slot.peak_bits
                .store((peak * gain).to_bits(), Ordering::Relaxed);
            slot.rms_bits.store(rms.to_bits(), Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }

    /// Zero the active plan's meter slots (silence: stopped or fully ramped
    /// out) and stamp the generation so readers see live zeros, not stale
    /// levels from the last audible block.
    fn publish_silent_meters(shared: &SharedState, plan: &RenderPlan) {
        for slot in shared
            .meter_slots
            .iter()
            .take(plan.stages.len().min(METER_SLOT_CAPACITY))
        {
            slot.peak_bits.store(0, Ordering::Relaxed);
            slot.rms_bits.store(0, Ordering::Relaxed);
        }
        shared
            .meter_generation
            .store(plan.generation, Ordering::Relaxed);
    }

    fn render_block_inner(&mut self, frames: &mut [f32]) {
        self.drain_commands();

        // Re-offer a parked retired plan without rendering work attached.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }

        frames.fill(0.0);

        let Some(plan) = self.plan.as_mut() else {
            return;
        };
        // Audible while the gate is open, while ramping out after a stop,
        // while ramping around an in-flight seek, or whenever the live
        // render posture is active (g13.018: live monitoring and live
        // events do not require the transport to roll).
        if !self.playing && self.edge_gain <= 0.0 && !self.live_render {
            // Silent block: meters read zero rather than holding the last
            // audible level.
            Self::publish_silent_meters(&self.shared, plan);
            return;
        }

        let stream_channels = self
            .stream_channels
            .map(|channels| channels as usize)
            .unwrap_or(plan.stream_channels)
            .max(1);
        let frame_count = frames.len() / stream_channels;
        debug_assert!(
            frame_count <= MAX_BLOCK_FRAMES,
            "render_block called with {frame_count} frames; scratch holds {MAX_BLOCK_FRAMES}",
        );
        let frame_count = frame_count.min(MAX_BLOCK_FRAMES);
        let block_start_frame = self.position_frames;
        let playing = self.playing;
        let live_render = self.live_render;
        // Timeline content (clips, position advance, discontinuity
        // consumption) renders while playing, and — exactly as before this
        // flag existed — through the stop edge ramp-out (`timeline_tail`).
        // Under the live render posture a stopped transport renders the
        // stage GRAPH (live input drains, live events deliver, processors
        // run, meters publish) but timeline content is silent and the
        // position holds.
        let timeline_active =
            playing || (self.timeline_tail && !live_render && self.edge_gain > 0.0);

        // Loop-region segmentation: while playing with a region set, a block
        // whose span crosses `loop_end` renders as (up to) TWO timeline
        // segments into one output buffer — `[pos, loop_end)` then the
        // remainder from `loop_start`. Each segment is
        // `(timeline_start, buffer_frame_offset, frame_count)`. Segments only
        // change the timeline positions clips see; gain smoothing, automation
        // sampling, meters, and the boundary/limiter/edge paths stay
        // per-BLOCK. Seeks outside the region are allowed — the wrap only
        // triggers when the block actually crosses `loop_end`.
        let mut segments: [(u64, usize, usize); 2] =
            [(block_start_frame, 0, frame_count), (0, 0, 0)];
        let mut segment_count = 1usize;
        // Buffer frame index where the wrap lands (first frame rendered from
        // `loop_start`); drives the wrap micro-fade below.
        let mut wrap_offset: Option<usize> = None;
        let mut end_position = block_start_frame + frame_count as u64;
        if let Some((loop_start, loop_end)) = self.loop_region {
            let block_end_frame = block_start_frame + frame_count as u64;
            if self.playing && block_start_frame < loop_end && block_end_frame >= loop_end {
                let first = (loop_end - block_start_frame) as usize;
                let remainder = frame_count - first;
                segments[0] = (block_start_frame, 0, first);
                segments[1] = (loop_start, first, remainder);
                segment_count = 2;
                wrap_offset = Some(first);
                end_position = loop_start + remainder as u64;
            }
        }
        // Stream chunk-retention cap (see `render_clips_into_scratch`):
        // active only while the playhead is inside the loop region, so
        // seeking past it never churns linear-playback prefetch.
        let loop_end_frame = match self.loop_region {
            Some((_, loop_end)) if self.playing && block_start_frame < loop_end => Some(loop_end),
            _ => None,
        };

        // Smoothed gains: move toward targets at a fixed full-swing rate and
        // interpolate across the block, so edits never step audio.
        let gain_step =
            frame_count as f32 / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0);

        // Walk the schedule: stages are in topological order, so every edge's
        // source sits strictly earlier and split-borrowing is safe.
        for node_index in 0..plan.stages.len() {
            let (earlier, rest) = plan.stages.split_at_mut(node_index);
            let stage = &mut rest[0];

            let gain_begin = stage.gain_current;
            let gain_end = if stage.gain_envelope.is_empty() {
                gain_begin + (stage.gain_target - gain_begin).clamp(-gain_step, gain_step)
            } else {
                // Automation: the target is the envelope's value at the
                // block end. The ramp starts from the inherited smoothed
                // gain, so plan swaps and seeks stay continuous while normal
                // playback tracks the curve sample-accurately block-wise.
                sample_envelope(&stage.gain_envelope, block_start_frame + frame_count as u64)
            };
            stage.block_gain_begin = gain_begin;
            stage.block_gain_slope = (gain_end - gain_begin) / frame_count.max(1) as f32;
            stage.gain_current = gain_end;

            let CompiledNode {
                channels,
                clips,
                inputs,
                processor,
                events,
                event_scratch,
                live_events,
                delay_ring,
                delay_cursor,
                scratch,
                ..
            } = stage;
            let channels = *channels;
            let scratch = &mut scratch[..frame_count * channels];
            scratch.fill(0.0);

            // Lane content first (at unity), then summed inputs. Rendered
            // per loop segment: each segment writes its own buffer span with
            // its own timeline start, so clips see wrapped positions while
            // tone phases carry across the wrap (segments render in order).
            for (segment_start, segment_offset, segment_frames) in
                segments.iter().take(segment_count)
            {
                if *segment_frames == 0 {
                    continue;
                }
                render_clips_into_scratch(
                    clips,
                    &mut scratch
                        [segment_offset * channels..(segment_offset + segment_frames) * channels],
                    channels,
                    *segment_start,
                    *segment_frames,
                    loop_end_frame,
                    timeline_active,
                );
            }

            for input in inputs.iter() {
                let source = &earlier[input.source_index];
                let source_channels = input.source_channels;
                let matrix = &input.matrix;
                // Per-frame: dest[c_out] += src[c_in] * m[c_in][c_out] *
                // source stage's smoothed gain (edge gain is folded into the
                // matrix at compile). Plain loops, alloc-free.
                for frame_index in 0..frame_count {
                    let source_gain =
                        source.block_gain_begin + source.block_gain_slope * frame_index as f32;
                    let source_base = frame_index * source_channels;
                    let dest_base = frame_index * channels;
                    for source_channel in 0..source_channels {
                        let sample = source.scratch[source_base + source_channel] * source_gain;
                        let row = &matrix[source_channel * channels..][..channels];
                        for (dest_channel, coefficient) in row.iter().enumerate() {
                            scratch[dest_base + dest_channel] += sample * coefficient;
                        }
                    }
                }
            }

            // Explicit graph delay: swap the summed block through the
            // preallocated interleaved ring. Cursor state spans callbacks
            // and compatible plan swaps, so the delay is sample-exact even
            // when its length exceeds the current block.
            if !delay_ring.is_empty() {
                for sample in scratch.iter_mut() {
                    std::mem::swap(sample, &mut delay_ring[*delay_cursor]);
                    *delay_cursor += 1;
                    if *delay_cursor == delay_ring.len() {
                        *delay_cursor = 0;
                    }
                }
            }

            // Plugin processor (g11.012): transforms the stage's summed
            // scratch in place before consumers read it. Bypass (`false` —
            // deadline miss, dead sandbox, unsupported layout) leaves the
            // scratch untouched by the backend contract, so the dry signal
            // flows on and plans without processors render bit-identically.
            //
            // Event delivery (g12.034 follow-up): stages carrying a compiled
            // event stream slice it per loop segment — binary search to the
            // segment start, then push events inside `[start, start+frames)`
            // with buffer-relative sample offsets into the preallocated
            // scratch (alloc-free; capacity overflow drops, earliest wins).
            // Delivery is playback-gated: while stopped (edge ramp-out) the
            // position does not advance, so re-firing the same events every
            // block would double-trigger notes.
            // Live-event delivery (g13.018): the stage's ring drains into
            // the same per-block scratch REGARDLESS of transport — a live-
            // played instrument sounds while stopped. Events at or past the
            // block start map to their in-block offset; events already in
            // the past clamp to offset 0 ("now"); events past the block end
            // clamp to the last frame (delivered once, this block). Compiled
            // events stay `playing`-gated exactly as before.
            if let Some(processor) = processor {
                if events.is_empty() && live_events.is_empty() {
                    let _ = processor.process(scratch, frame_count, channels);
                } else {
                    event_scratch.clear();
                    if playing && !events.is_empty() {
                        for (segment_index, (segment_start, segment_offset, segment_frames)) in
                            segments.iter().take(segment_count).enumerate()
                        {
                            if *segment_frames == 0 {
                                continue;
                            }
                            let discontinuity_from = if segment_index == 0 {
                                self.event_discontinuity_from
                            } else {
                                self.loop_region.map(|(_, loop_end)| loop_end)
                            };
                            if let Some(from) = discontinuity_from {
                                append_plugin_state_chase(
                                    events,
                                    from,
                                    *segment_start,
                                    *segment_offset as u32,
                                    event_scratch,
                                );
                            }
                            let segment_end = *segment_start + *segment_frames as u64;
                            let begin =
                                events.partition_point(|event| event.frame < *segment_start);
                            for event in events[begin..]
                                .iter()
                                .take_while(|event| event.frame < segment_end)
                            {
                                if event_scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                                    break;
                                }
                                event_scratch.push(RenderBlockPluginEvent {
                                    offset_frames: (*segment_offset as u64
                                        + (event.frame - segment_start))
                                        as u32,
                                    channel: event.channel,
                                    kind: event.kind,
                                });
                            }
                        }
                    }
                    if !live_events.is_empty() {
                        let last_offset = frame_count.saturating_sub(1) as u64;
                        let mut dropped = 0u64;
                        for event in live_events.iter() {
                            if event_scratch.len() == PLUGIN_EVENTS_PER_BLOCK_CAPACITY {
                                dropped += 1;
                                continue;
                            }
                            let offset = event
                                .frame
                                .saturating_sub(block_start_frame)
                                .min(last_offset);
                            event_scratch.push(RenderBlockPluginEvent {
                                offset_frames: offset as u32,
                                channel: event.channel,
                                kind: event.kind,
                            });
                        }
                        live_events.clear();
                        if dropped > 0 {
                            self.shared
                                .live_event_drop_count
                                .fetch_add(dropped, Ordering::Relaxed);
                        }
                        // Restore the sorted-by-offset contract after the
                        // append: stable in-place insertion sort (alloc-free;
                        // the compiled prefix is already sorted, so this is
                        // near-linear, and ties keep compiled-before-live and
                        // push order).
                        insertion_sort_events_by_offset(event_scratch);
                    }
                    let _ = processor.process_with_events(
                        scratch,
                        frame_count,
                        channels,
                        event_scratch,
                    );
                }
            }
        }

        // Meters: every stage's scratch for this block is final now (the
        // boundary write below only reads the master's scratch).
        Self::publish_meters(&self.shared, plan, frame_count);

        // Hardware boundary: adapt the master stage's scratch onto the stream
        // with the master's smoothed gain. Equal formats copy; a narrower
        // stream applies the install-time downmix matrix; a wider stream
        // carries the master's channels and leaves the extras silent (the
        // creative mix never upmixes at the hardware stage).
        let master = &plan.stages[plan.master_index];
        let output_channels = master.channels;
        let boundary_matrix = &plan.boundary_matrix;
        let boundary_matrix_valid = boundary_matrix.len() == output_channels * stream_channels;
        for frame_index in 0..frame_count {
            let master_gain =
                master.block_gain_begin + master.block_gain_slope * frame_index as f32;
            let source_base = frame_index * output_channels;
            let dest_base = frame_index * stream_channels;
            if stream_channels < output_channels && boundary_matrix_valid {
                for dest_channel in 0..stream_channels {
                    let mut acc = 0.0f32;
                    for source_channel in 0..output_channels {
                        acc += master.scratch[source_base + source_channel]
                            * boundary_matrix[source_channel * stream_channels + dest_channel];
                    }
                    frames[dest_base + dest_channel] = acc * master_gain;
                }
            } else {
                // Equal formats, a wider stream (extras stay zero-filled),
                // or a stale boundary (stream changed without a reinstall):
                // copy the overlapping channels.
                for channel in 0..output_channels.min(stream_channels) {
                    frames[dest_base + channel] =
                        master.scratch[source_base + channel] * master_gain;
                }
            }
        }

        // Loop-wrap declick: linear micro-fade out over the frames before
        // the wrap point and in over the frames after it, applied to the
        // output buffer only on blocks that wrap (cheap; never touches
        // non-looping renders). When the wrap lands exactly on the block
        // boundary only the fade-out applies — the next block starts from
        // `loop_start` behind the still-open edge envelope.
        if let Some(wrap_offset) = wrap_offset {
            let fade_out = LOOP_WRAP_FADE_FRAMES.min(wrap_offset);
            for step in 0..fade_out {
                let frame_index = wrap_offset - fade_out + step;
                let gain = (fade_out - 1 - step) as f32 / fade_out as f32;
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= gain;
                }
            }
            let fade_in = LOOP_WRAP_FADE_FRAMES.min(frame_count - wrap_offset);
            for step in 0..fade_in {
                let frame_index = wrap_offset + step;
                let gain = (step + 1) as f32 / fade_in as f32;
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= gain;
                }
            }
        }

        // Master soft limiter: guards the stream buffer after the boundary
        // write so the creative graph stays untouched; linked gain across
        // the stream's channels.
        if let Some(limiter) = plan.limiter.as_mut() {
            for frame_index in 0..frame_count {
                let base = frame_index * stream_channels;
                limiter.process_frame(&mut frames[base..base + stream_channels]);
            }
        }

        // Transport edge envelope over the mixed block: ramps toward the
        // gate target (zero while a seek is in flight) and never steps.
        // Under the live render posture the envelope holds open while
        // stopped — monitoring and live-played notes must stay audible, so
        // the stop ramp-out does not apply (timeline content is gated at
        // the clip level instead). The seek ramp-out/in is unchanged.
        let edge_target = if self.pending_seek.is_some() {
            0.0
        } else if self.playing || live_render {
            1.0
        } else {
            0.0
        };
        if self.edge_gain != edge_target || edge_target < 1.0 {
            let edge_step = 1.0 / (EDGE_RAMP_SECONDS * plan.sample_rate_hz as f32).max(1.0);
            for frame_index in 0..frame_count {
                self.edge_gain += (edge_target - self.edge_gain).clamp(-edge_step, edge_step);
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= self.edge_gain;
                }
            }
        }

        // Timeline tail: latched while playing, held through the stop
        // ramp-out (edge still open, posture off), cleared once the edge
        // closes or the posture cuts timeline content at the stop.
        self.timeline_tail =
            self.playing || (self.timeline_tail && !live_render && self.edge_gain > 0.0);

        // After a wrap block the clock reads `loop_start + remainder`;
        // otherwise it advances linearly by the block. Stopped live-render
        // blocks hold the position (the transport does not advance while
        // stopped) and preserve any pending discontinuity for the next
        // timeline-active block.
        if timeline_active {
            self.position_frames = end_position;
            self.shared
                .position_frames
                .store(self.position_frames, Ordering::Relaxed);

            // Seek lands at the envelope's zero crossing; the next block
            // ramps back in from the new position.
            // Any discontinuity consumed by this block is complete. A
            // pending seek applied below installs the source for the NEXT
            // block.
            self.event_discontinuity_from = None;
        }
        if self.edge_gain <= 0.0 {
            if let Some(position) = self.pending_seek.take() {
                self.apply_seek(position);
            }
        }
    }
}
