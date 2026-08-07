use std::sync::atomic::Ordering;
use std::sync::mpsc::TrySendError;
use std::time::Instant;

use signal_dsp::DenormalGuard;

use crate::plan::CompiledNode;
use crate::plan_render::{
    insertion_sort_events_by_offset, render_clips_into_scratch, sample_envelope,
};
use crate::plugin_events::append_plugin_state_chase;
use crate::{RenderBlockPluginEvent, MAX_BLOCK_FRAMES, PLUGIN_EVENTS_PER_BLOCK_CAPACITY};

use super::super::{EDGE_RAMP_SECONDS, GAIN_SMOOTHING_SECONDS, LOOP_WRAP_FADE_FRAMES};
use super::RenderPlaneExecutor;

impl RenderPlaneExecutor {
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
