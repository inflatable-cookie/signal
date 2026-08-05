# g10.040 Batch 40.1 - Feasibility And Design Reassessment

Status: complete
Created: 2026-08-05
Scope: RealtimePreview callback tier, reachable-or-not decision

## Decision

Reachable. The stop condition — I/O or unbounded work required on the callback
— is not met. Batch 40.2 opens.

## CPU Was Never The Blocker

This tier has stalled across three roadmaps on latency and reporting work. The
first thing this batch did was measure the kernel's actual cost, and the number
reframes the whole lane.

Steady state at ratio `1.0`, `48 kHz`, release, `4000` iterations after a
`64`-callback warmup:

| channels | block | per callback | per spectral frame | budget | load |
| --- | --- | --- | --- | --- | --- |
| `1` | `128` | `8.0us` | `8.0us` | `2666.7us` | `0.3%` |
| `1` | `256` | `15.9us` | `7.9us` | `5333.3us` | `0.3%` |
| `1` | `512` | `31.8us` | `8.0us` | `10666.7us` | `0.3%` |
| `2` | `128` | `15.6us` | `15.6us` | `2666.7us` | `0.6%` |
| `2` | `256` | `31.5us` | `15.8us` | `5333.3us` | `0.6%` |
| `2` | `512` | `62.7us` | `15.7us` | `10666.7us` | `0.6%` |

Per-spectral-frame cost is flat across block sizes, so it projects. Load scales
as `1/ratio`; stereo at block `128` reaches `9.4%` at one-sixteenth speed.

Three roadmaps were spent on a tier whose kernel uses under one percent of its
callback budget. The measurement took one temporary probe, since removed.

## The Defect

`process` is quantum-locked: `frame_count` in, `frame_count` out, regardless of
ratio. `next_analysis_frame` advances `analysis_hop` per spectral frame while
`next_synthesis_frame` advances `analysis_hop * ratio`, so the cursors diverge
by construction.

At ratio above `1.0` synthesis outruns the output ring and the loop breaks on
its guard. Nothing is analysed, but input keeps being pushed at `frame_count`
per callback, so the gap widens until the input-ring guard fires and assigns
`next_analysis_frame = input_write_frame - ring_frames`, discarding every
unanalysed frame in between. `process` then returns `Ok` reporting
`input_frames == output_frames == frame_count`.

The `g10.027` projection reports the correct source advance beside this. Nothing
consumes it, which is exactly why the report and the kernel disagree.

## Bounded Work Needs A Bounded Ratio

`sanitize_ratio` accepts any finite positive value. Since work scales as
`1/ratio`, Contract `046`'s bounded-work requirement is unsatisfiable as
written. A minimum ratio is a gate precondition, not a tuning detail, and it is
added to Batch 40.2's freeze list.

## Latency Is Affordable Because Preview Is Playback

One window: `512` frames, `10.67ms` at `48 kHz`. Four quanta at a `128`-frame
block.

That is affordable because RealtimePreview plays back a stored asset at a
changed rate rather than monitoring a live signal. The window latency is a
start-up delay before playback begins, not a round-trip cost on something the
operator is playing. The distinction is what makes the tier viable, and it had
not been stated.

## Source Ownership

The render plane pulls preview output and the preview state pulls source
frames. Both, at different boundaries — treating it as a single either/or is
part of why it looked unresolvable.

A non-realtime producer owns the reader and fills an SPSC ring; all I/O lives
there. The callback consumes `block / ratio` source frames per callback from
that ring, publishes how far ahead it needs the source filled as an atomic the
producer reads, and on underrun emits silence for the missing span and reports
it. It must not stall, must not skip source, and must not return `Ok` as though
the block were normal — that last behaviour is the present defect.

## A Naming Hazard

The roadmap's Problem section says no workspace consumer imports any
`RealtimePreview` type. True of the callback surface, false of the name.

`RealtimePreviewStretcher` is a whole-buffer control-side prototype, not a
callback object, and `loophole/pulse` consumes it in `render_plan.rs` to
pre-stretch and cache assets. Closing the tier and deleting "the RealtimePreview
surface" would have broken a shipping consumer. Only
`RealtimePreviewCallbackState` and the six never-constructed enum variants are
dead. Batch 40.5 must name types, not the prefix.

## Contract 046 Callback Gate

Bounded work is conditional on the frozen minimum ratio. No-allocation,
no-locks, no-I/O and deterministic latency are all satisfied by the model above.
Linked stereo is already supported. Dynamic-ratio alignment machinery exists but
is unconsumed. Seam evidence is not yet produced and belongs to Batch 40.2.

## Next Task

Open Batch 40.2, the complete streaming brief.
