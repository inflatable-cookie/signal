# g10.040 Batch 40.2 - Complete Streaming Brief

Status: complete
Created: 2026-08-05
Scope: freeze the RealtimePreview callback streaming design

## What Is Frozen

Ratio range, state inventory and ceiling, source fill and underrun policy, one
ratio scheduler, latency report, alignment tolerance, seam evidence, and the
evidence order. Batch 40.3 implements it once and does not renegotiate it.

## Both Ends Of The Ratio Range Are Derived

The maximum is not a choice. Contract `046`'s overlap law requires
`analysis_hop * ratio <= 0.75 * window_size`, and at the frozen `128`/`512`
geometry that is exactly `3.0`. Exceeding it needs the contract's hop
reduction, which changes the geometry and therefore the state inventory, so it
is out of scope. High ratios are cheap — `0.20%` of budget at `3.0` — so the
limit is spectral coverage, not cost.

The minimum is bounded work, from the Batch 40.1 measurements:

| ratio | stereo load, `128`-frame callback |
| --- | --- |
| `3.0` | `0.20%` |
| `1.0` | `0.59%` |
| `0.5` | `1.18%` |
| `0.25` | `2.36%` |
| `0.125` | `4.72%` |

`0.25` is four-times-faster playback at `2.36%`. Widening to `0.125` costs
`4.72%` and doubles the source ring — affordable, so that end is a product
decision rather than an engineering limit. What is not negotiable is having a
floor: `sanitize_ratio` accepts any positive value today, which makes bounded
work unsatisfiable regardless of how much headroom exists.

Out-of-range ratios are rejected at plan time, not clamped silently.

## The Ceiling Follows The Design

Measured with an allocation-counting global allocator, stereo:

| block | current state |
| --- | --- |
| `128` | `141.3 KiB` |
| `512` | `180.3 KiB` |
| `4096` | `544.3 KiB` |

Plus one source ring at `ceil(block / ratio_min) * 2 + window_size` frames,
which is `260.0 KiB` at block `4096` and `ratio_min = 0.25`.

Ceiling: `1 MiB` stereo at `MAX_BLOCK_FRAMES`, against a computed `804.3 KiB`.

The order matters. `g10.039` froze a memory ceiling before its design existed
and moved it three times; Contract `046` records that a memory bound is a
consequence of a working design rather than something that can be fixed ahead
of one. This ceiling was computed from a measured state plus a sized ring, not
estimated.

## The Surviving Scheduler Is The One That Looks Wrong

The state carries two ratio schedulers, field for field — eleven each,
`current/active/pending`, request and apply frames, alignment error, change
count. The `source_projection_*` set survives and the output-side set is
deleted.

That direction is counterintuitive: the output-side scheduler is the one the
working kernel uses. But it computes synthesis advance, which the quantum-locked
design needed only because it had no way to ask for source. The projection
scheduler computes the source advance a ratio implies, which is exactly what
drives demand in the new model.

The `g10.027` projection machinery was never wrong. Nothing consumed it. This
brief makes it the single authority, which also removes the report-versus-kernel
disagreement recorded in the roadmap's Problem section — they disagreed because
they were two schedulers, not because the reporting was wrong.

## Underrun Must Be Distinguishable

On underrun the callback emits silence for the missing span, increments a
counter, and reports the shortfall in frames. It must not stall, must not
advance past unfilled source, and must not return a report indistinguishable
from a normal block.

That last clause is the lesson of the whole lane. The present defect survived
three roadmaps precisely because `process` returns `Ok` with
`input_frames == output_frames == frame_count` while discarding source. A
report that cannot express failure hides it.

## Evidence Order

Six gates, each blocking the next: allocation-free execution; bounded work
across the range, measured; continuous output at sustained ratios either side
of `1.0` with no dropped source; underrun reported as silence; alignment within
`128` frames; seam correlation against a whole-buffer control.

Any failure rejects the candidate under Contract `084` Rule 2 and reverts it
from `main`. Structural conformance may iterate under Rule 11; the acoustic
checkpoint is one-shot.

`RealtimePreviewStretcher` is out of scope in both directions — `loophole/pulse`
consumes it and this lane must not touch it.

## Next Task

Open Batch 40.3, the isolated implementation.
