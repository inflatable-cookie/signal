//! Audio-thread helpers that fill stage scratch from compiled clips.

use std::sync::atomic::Ordering;

use signal_dsp::PolyphaseInterpolationTable;

use crate::live_input::{LIVE_INPUT_CHUNK_FRAMES, LIVE_INPUT_MAX_BACKLOG_FRAMES};
use crate::plan::{CompiledClip, CompiledSource};
use crate::{
    RenderBlockPluginEvent, StreamChunk, NOTE_POLYPHONY_LIMIT, STREAM_RETIRE_LOOKAHEAD_FRAMES,
};

/// Interpolation table shape for rate-converted media playback.
const RESAMPLE_TAPS: usize = 16;

/// Sample a sorted `(frame, value)` automation envelope at `frame`: linear
/// interpolation between breakpoints, clamped to the first/last values
/// outside the span. Binary search + arithmetic only — audio-thread safe.
#[inline]
pub(crate) fn sample_envelope(points: &[(u64, f32)], frame: u64) -> f32 {
    match points.binary_search_by(|(point_frame, _)| point_frame.cmp(&frame)) {
        Ok(index) => points[index].1,
        Err(0) => points[0].1,
        Err(index) if index == points.len() => points[points.len() - 1].1,
        Err(index) => {
            let (start_frame, start_value) = points[index - 1];
            let (end_frame, end_value) = points[index];
            let span = (end_frame - start_frame).max(1) as f64;
            let progress = (frame - start_frame) as f64 / span;
            start_value + (end_value - start_value) * progress as f32
        }
    }
}

/// Stable in-place insertion sort of a per-block event slice by
/// `offset_frames`. Used after live events append to a (sorted) compiled
/// prefix: near-linear on the mostly-sorted input, allocation-free, and
/// stability preserves compiled-before-live plus push order on equal
/// offsets — audio-thread safe.
#[inline]
pub(crate) fn insertion_sort_events_by_offset(events: &mut [RenderBlockPluginEvent]) {
    for index in 1..events.len() {
        let mut position = index;
        while position > 0 && events[position - 1].offset_frames > events[position].offset_frames {
            events.swap(position - 1, position);
            position -= 1;
        }
    }
}

/// Window gain for a frame inside a clip: per side, an explicit equal-power
/// fade when requested, otherwise the linear edge declick.
///
/// Start side with `fade_in_frames > 0`: `sin(π/2 · p/F)` over positions
/// `p = frame - start_frames ∈ [0, F)` (the first frame is exactly 0 — that
/// exactness is what makes an overlapped fade-out/fade-in pair sum to unity
/// POWER at every frame: the two quarter-wave arguments are complementary).
/// End side with `fade_out_frames > 0`: the mirror, `sin(π/2 · r/F)` over
/// `r = end_frames - frame ∈ (0, F]`. A requested fade REPLACES the declick
/// on its side; a zero-fade side keeps the declick ramp, byte-identical to
/// the historical behavior. The sides combine via `min`, which reproduces
/// the historical declick-only expression exactly and is inert wherever the
/// spans do not overlap.
///
/// Pure function of the frame position — stateless, so fades are correct
/// across block boundaries, seeks into the middle of a fade, and plan swaps.
#[inline]
pub(crate) fn clip_window_gain(
    frame: u64,
    start_frames: u64,
    end_frames: u64,
    edge_fade_frames: u64,
    fade_in_frames: u64,
    fade_out_frames: u64,
) -> f32 {
    let in_gain = if fade_in_frames > 0 {
        let position = frame - start_frames;
        if position < fade_in_frames {
            (std::f32::consts::FRAC_PI_2 * position as f32 / fade_in_frames as f32).sin()
        } else {
            1.0
        }
    } else if edge_fade_frames > 0 {
        ((frame - start_frames + 1) as f32 / edge_fade_frames as f32).min(1.0)
    } else {
        1.0
    };
    let out_gain = if fade_out_frames > 0 {
        let remaining = end_frames - frame;
        if remaining < fade_out_frames {
            (std::f32::consts::FRAC_PI_2 * remaining as f32 / fade_out_frames as f32).sin()
        } else {
            1.0
        }
    } else if edge_fade_frames > 0 {
        ((end_frames - frame) as f32 / edge_fade_frames as f32).min(1.0)
    } else {
        1.0
    };
    in_gain.min(out_gain)
}

/// Per-frame interpolated source read shared by the `Samples` and `Stream`
/// paths: polyphase windowed-sinc when rate-converted, clamped lerp at 1:1.
/// `fetch(source_frame, channel)` returns the source sample or `None` when
/// that frame is unavailable (off the buffer for samples, not currently
/// held for streams). Returns `None` — render silence, and for streams
/// count an underrun — when the center frame itself is unavailable;
/// unavailable outer sinc taps just contribute zero, matching the buffer
/// path's edge behavior. Arithmetic and accumulation order are identical to
/// the historical `Samples` implementation (golden-hash stable).
#[inline]
pub(crate) fn interpolate_source_frame(
    fetch: &impl Fn(i64, usize) -> Option<f32>,
    source_index: u64,
    fraction: f64,
    table: Option<&PolyphaseInterpolationTable>,
    channel: usize,
) -> Option<f32> {
    match table {
        // Rate conversion: polyphase windowed-sinc tap dot product (table
        // reads only — no allocation, no transcendentals).
        Some(table) => {
            fetch(source_index as i64, channel)?;
            let row = table.phase_row(fraction);
            let first = table.first_tap_offset();
            let mut acc = 0.0f32;
            for (tap, coefficient) in row.iter().enumerate() {
                let tap_index = source_index as i64 + first as i64 + tap as i64;
                if let Some(value) = fetch(tap_index, channel) {
                    acc += value * coefficient;
                }
            }
            Some(acc)
        }
        // 1:1 playback: direct read with last-frame clamp (`fetch` wraps
        // instead when the source loops).
        None => {
            let a = fetch(source_index as i64, channel)?;
            let b = fetch(source_index as i64 + 1, channel).unwrap_or(a);
            Some(a + (b - a) * fraction as f32)
        }
    }
}

/// Render a lane stage's clips into its scratch at unity gain (stage and edge
/// gains apply downstream, where the scratch is consumed). A clip source whose
/// channel count matches the stage writes directly; a mono or other-count
/// source is up/down-mixed into the stage format through the adapter matrix
/// precompiled on the compiled source.
/// `loop_end_frame` is the active loop region's end while the playhead is
/// inside the region (`None` otherwise): stream clips use it to retire held
/// chunks past the loop end, which would otherwise sit "ahead but within the
/// lookahead" forever on short loops and starve the held slots after a wrap.
pub(crate) fn render_clips_into_scratch(
    clips: &mut [CompiledClip],
    scratch: &mut [f32],
    channels: usize,
    block_start_frame: u64,
    frame_count: usize,
    loop_end_frame: Option<u64>,
    timeline_active: bool,
) {
    let block_end_frame = block_start_frame + frame_count as u64;
    for clip in clips.iter_mut() {
        // Timeline-positioned content is silent while the transport is not
        // advancing (g13.018 live render posture: a stopped-but-rendering
        // block must not replay the frozen position every callback). Live
        // input is position-independent — it always drains "now".
        if !timeline_active && !matches!(clip.source, CompiledSource::LiveInput { .. }) {
            continue;
        }
        // Skip clips entirely outside this block.
        if clip.end_frames <= block_start_frame || clip.start_frames >= block_end_frame {
            continue;
        }
        let clip_start = clip.start_frames;
        let clip_end = clip.end_frames;
        let clip_fade = clip.edge_fade_frames;
        let fade_in = clip.fade_in_frames;
        let fade_out = clip.fade_out_frames;
        match &mut clip.source {
            CompiledSource::Silence => {}
            CompiledSource::Tone { phase, step } => {
                let mut local_phase = *phase;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    let sample = local_phase.sin();
                    local_phase += *step;
                    if local_phase >= std::f32::consts::TAU {
                        local_phase -= std::f32::consts::TAU;
                    }
                    if frame >= clip_start && frame < clip_end {
                        let sample = sample
                            * clip_window_gain(
                                frame, clip_start, clip_end, clip_fade, fade_in, fade_out,
                            );
                        let base = frame_index * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += sample;
                        }
                    }
                }
                *phase = local_phase;
            }
            CompiledSource::Samples {
                buffer,
                step,
                loop_source,
                table,
                channel_adapter,
            } => {
                let source_frames = buffer.frame_count();
                if source_frames == 0 {
                    continue;
                }
                let data = &buffer.frames;
                let source_channels = buffer.channels.max(1) as usize;
                let frame_total = source_frames as i64;
                let loop_source = *loop_source;
                // Loop folding lives in the fetch: wrapped taps and the
                // wrapped lerp neighbour come out identical to the
                // historical inline implementation.
                let fetch = |source_frame: i64, channel: usize| -> Option<f32> {
                    let source_frame = if loop_source {
                        source_frame.rem_euclid(frame_total)
                    } else {
                        source_frame
                    };
                    if source_frame < 0 || source_frame >= frame_total {
                        return None;
                    }
                    Some(data[source_frame as usize * source_channels + channel])
                };
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    if frame < clip_start || frame >= clip_end {
                        continue;
                    }
                    // Source position via the rate ratio, anchored at the
                    // clip's window start.
                    let mut source_position = (frame - clip_start) as f64 * *step;
                    if loop_source {
                        source_position %= source_frames as f64;
                    }
                    let source_index = source_position as usize;
                    if source_index >= source_frames {
                        continue;
                    }
                    let fraction = source_position - source_index as f64;
                    let window_gain =
                        clip_window_gain(frame, clip_start, clip_end, clip_fade, fade_in, fade_out);
                    let base = frame_index * channels;
                    match channel_adapter {
                        // Matched channel counts: direct per-channel read (the
                        // historical stereo path when both are 2).
                        None => {
                            for channel in 0..channels {
                                if let Some(sample) = interpolate_source_frame(
                                    &fetch,
                                    source_index as u64,
                                    fraction,
                                    table.as_ref(),
                                    channel,
                                ) {
                                    scratch[base + channel] += sample * window_gain;
                                }
                            }
                        }
                        // Mismatched: up/down-mix each source channel into the
                        // stage channels via the precompiled adapter matrix.
                        Some(matrix) => {
                            for source_channel in 0..source_channels {
                                let Some(sample) = interpolate_source_frame(
                                    &fetch,
                                    source_index as u64,
                                    fraction,
                                    table.as_ref(),
                                    source_channel,
                                ) else {
                                    continue;
                                };
                                let sample = sample * window_gain;
                                for dest_channel in 0..channels {
                                    let coefficient =
                                        matrix[source_channel * channels + dest_channel];
                                    scratch[base + dest_channel] += sample * coefficient;
                                }
                            }
                        }
                    }
                }
            }
            CompiledSource::Stream {
                handle,
                held,
                step,
                table,
            } => {
                let total_frames = handle.inner.total_frames;
                if total_frames == 0 {
                    continue;
                }
                // Publish the read hint: clip-anchored source frame of the
                // first audible frame in this block. Seeks read as jumps.
                let first_audible = block_start_frame.max(clip_start);
                let needed_start = ((first_audible - clip_start) as f64 * *step) as u64;
                handle
                    .inner
                    .wanted_frame
                    .store(needed_start, Ordering::Relaxed);
                // Retire held chunks entirely behind the playhead (minus
                // the sinc tap margin) or stale past the seek window —
                // returned to the feeder, never freed here.
                let behind_cutoff = needed_start.saturating_sub(RESAMPLE_TAPS as u64);
                let mut ahead_cutoff = needed_start.saturating_add(STREAM_RETIRE_LOOKAHEAD_FRAMES);
                // Looping: chunks past the loop end (in this clip's source
                // frame space, plus the sinc tap margin) are useless until
                // the loop clears — retire them so refed wrap-target chunks
                // always find a held slot.
                if let Some(loop_end) = loop_end_frame {
                    let source_cap = ((loop_end.saturating_sub(clip_start)) as f64 * *step) as u64
                        + RESAMPLE_TAPS as u64;
                    ahead_cutoff = ahead_cutoff.min(source_cap.max(needed_start));
                }
                for slot in held.iter_mut() {
                    let stale = slot.as_ref().is_some_and(|chunk| {
                        chunk.end_frame() <= behind_cutoff || chunk.start_frame > ahead_cutoff
                    });
                    if stale {
                        let chunk = slot.take().expect("stale slot is occupied");
                        if let Err(chunk) = handle.inner.retired.try_push(chunk) {
                            // Retired mailbox full: hold the chunk and retry
                            // next block (never drop on the audio thread).
                            *slot = Some(chunk);
                        }
                    }
                }
                // Drain the chunk mailbox into free slots only — a popped
                // chunk must always have a home on this thread. Stale
                // arrivals occupy a slot for one block and retire on the
                // next pass.
                for slot in held.iter_mut() {
                    if slot.is_none() {
                        match handle.inner.chunks.try_pop() {
                            Some(chunk) => *slot = Some(chunk),
                            None => break,
                        }
                    }
                }
                // Backward jumps (loop wraps, seeks) can leave every held
                // slot occupied by valid-but-ahead chunks while the chunk
                // covering the read position waits in the mailbox. When
                // nothing held covers `needed_start` and no slot is free,
                // retire the furthest-ahead chunk so refed wrap-target
                // chunks find a home within a block or two (transient
                // underrun by design, never a permanent stall).
                let covered = held.iter().flatten().any(|chunk| {
                    needed_start >= chunk.start_frame && needed_start < chunk.end_frame()
                });
                if !covered {
                    let furthest = (0..held.len())
                        .filter(|index| held[*index].is_some())
                        .max_by_key(|index| {
                            held[*index]
                                .as_ref()
                                .map(|chunk| chunk.start_frame)
                                .unwrap_or(0)
                        });
                    let all_full = held.iter().all(|slot| slot.is_some());
                    if let (true, Some(index)) = (all_full, furthest) {
                        let chunk = held[index].take().expect("occupied slot");
                        if let Err(chunk) = handle.inner.retired.try_push(chunk) {
                            // Retired mailbox full: keep holding (never drop
                            // on the audio thread) and retry next block.
                            held[index] = Some(chunk);
                        }
                    }
                }
                let held: &[Option<StreamChunk>] = held;
                // Locate a source frame in the held chunks: linear scan over
                // a handful of slots, then an index.
                let fetch = |source_frame: i64, channel: usize| -> Option<f32> {
                    if source_frame < 0 {
                        return None;
                    }
                    let source_frame = source_frame as u64;
                    held.iter().flatten().find_map(|chunk| {
                        (source_frame >= chunk.start_frame && source_frame < chunk.end_frame())
                            .then(|| {
                                chunk.frames
                                    [(source_frame - chunk.start_frame) as usize * 2 + channel]
                            })
                    })
                };
                let mut underrun_frames = 0u64;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    if frame < clip_start || frame >= clip_end {
                        continue;
                    }
                    let source_position = (frame - clip_start) as f64 * *step;
                    let source_index = source_position as u64;
                    if source_index >= total_frames {
                        // Past the asset's end: silence by design, not an
                        // underrun.
                        continue;
                    }
                    let fraction = source_position - source_index as f64;
                    let window_gain =
                        clip_window_gain(frame, clip_start, clip_end, clip_fade, fade_in, fade_out);
                    let base = frame_index * channels;
                    let mut missing = false;
                    for channel in 0..channels.min(2) {
                        match interpolate_source_frame(
                            &fetch,
                            source_index,
                            fraction,
                            table.as_ref(),
                            channel,
                        ) {
                            Some(sample) => scratch[base + channel] += sample * window_gain,
                            None => missing = true,
                        }
                    }
                    if missing {
                        underrun_frames += 1;
                    }
                }
                if underrun_frames > 0 {
                    handle
                        .inner
                        .underrun_frames
                        .fetch_add(underrun_frames, Ordering::Relaxed);
                }
            }
            CompiledSource::Notes {
                buffer,
                steps,
                attack_frames,
                release_frames,
                max_duration_frames,
            } => {
                let notes = &buffer.notes;
                if notes.is_empty() {
                    continue;
                }
                // Audible span of this block inside the clip window, in
                // clip-relative frames (notes are clip-relative).
                let span_start = block_start_frame.max(clip_start);
                let span_end = block_end_frame.min(clip_end);
                if span_start >= span_end {
                    continue;
                }
                let local_start = span_start - clip_start;
                let local_end = span_end - clip_start;
                let attack_frames = *attack_frames;
                let release_frames = *release_frames;
                // Sorted overlap scan: a note sounding in this block starts
                // at most (longest duration + release tail) before it, so
                // binary-search the first candidate and walk until starts
                // pass the block end. Zero-alloc: search + arithmetic only.
                let lookback = max_duration_frames.saturating_add(release_frames);
                let window_lo = local_start.saturating_sub(lookback);
                let first = notes.partition_point(|note| note.start_frame < window_lo);
                let mut sounding = 0usize;
                for (note, step) in notes[first..].iter().zip(steps[first..].iter()) {
                    if note.start_frame >= local_end {
                        break;
                    }
                    let note_end = note.start_frame.saturating_add(note.duration_frames);
                    let tail_end = note_end.saturating_add(release_frames);
                    if tail_end <= local_start {
                        continue;
                    }
                    // Polyphony cap, per block: the scan runs in start
                    // order, so the earliest-started notes render and the
                    // rest skip deterministically.
                    if sounding >= NOTE_POLYPHONY_LIMIT {
                        break;
                    }
                    sounding += 1;
                    let render_lo = local_start.max(note.start_frame);
                    let render_hi = local_end.min(tail_end);
                    let amplitude = note.velocity.clamp(0.0, 1.0);
                    for local_frame in render_lo..render_hi {
                        // STATELESS voice: phase and envelope are pure
                        // functions of the note-relative position, so seeks
                        // and plan swaps reproduce the exact same samples.
                        let rel = local_frame - note.start_frame;
                        let phase = (rel as f64 * step) % std::f64::consts::TAU;
                        let attack_gain = if rel < attack_frames {
                            (rel + 1) as f32 / attack_frames as f32
                        } else {
                            1.0
                        };
                        let release_gain = if rel >= note.duration_frames {
                            1.0 - (rel - note.duration_frames) as f32 / release_frames as f32
                        } else {
                            1.0
                        };
                        let frame = clip_start + local_frame;
                        let window_gain = clip_window_gain(
                            frame, clip_start, clip_end, clip_fade, fade_in, fade_out,
                        );
                        let sample = phase.sin() as f32
                            * amplitude
                            * attack_gain
                            * release_gain
                            * window_gain;
                        let base = (frame - block_start_frame) as usize * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += sample;
                        }
                    }
                }
            }
            CompiledSource::LiveInput { handle } => {
                // Position-independent: the clip window only gates WHETHER
                // live input is audible in this block; the content is always
                // the ring's "now". Frames pop in order across segments and
                // blocks, so monitored audio is continuous.
                let span_start = block_start_frame.max(clip_start);
                let span_end = block_end_frame.min(clip_end);
                let span = (span_end - span_start) as usize;
                if span == 0 {
                    continue;
                }
                let ring = &handle.inner.ring;
                let mut chunk = [0.0f32; LIVE_INPUT_CHUNK_FRAMES * 2];
                // Backlog trim: input pushed while the executor was not
                // draining (transport stopped, plan absent) is stale — it
                // would replay as added monitoring latency. Discard down to
                // the bounded backlog before rendering. Alloc-free.
                let buffered = ring.len() / 2;
                let keep = span + LIVE_INPUT_MAX_BACKLOG_FRAMES;
                if buffered > keep {
                    let mut discard = buffered - keep;
                    while discard > 0 {
                        let take = discard.min(LIVE_INPUT_CHUNK_FRAMES);
                        let popped = ring.pop_slice(&mut chunk[..take * 2]) / 2;
                        if popped == 0 {
                            break;
                        }
                        discard -= popped;
                    }
                }
                // Drain up to `span` frames into the window's buffer span at
                // unity (stage/edge gains apply downstream), with the same
                // window edge declick as every other source.
                let mut frame_offset = (span_start - block_start_frame) as usize;
                let mut remaining = span;
                while remaining > 0 {
                    let take = remaining.min(LIVE_INPUT_CHUNK_FRAMES);
                    let popped = ring.pop_slice(&mut chunk[..take * 2]) / 2;
                    for index in 0..popped {
                        let frame = block_start_frame + (frame_offset + index) as u64;
                        let window_gain = clip_window_gain(
                            frame, clip_start, clip_end, clip_fade, fade_in, fade_out,
                        );
                        let base = (frame_offset + index) * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += chunk[index * 2 + channel] * window_gain;
                        }
                    }
                    frame_offset += popped;
                    remaining -= popped;
                    if popped < take {
                        // Ring dry: the rest of the span is an underrun —
                        // silence (scratch untouched) plus the counter.
                        handle
                            .inner
                            .underrun_frames
                            .fetch_add(remaining as u64, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }
    }
}
