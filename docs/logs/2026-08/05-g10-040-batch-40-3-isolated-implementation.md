# g10.040 Batch 40.3 - Isolated Implementation

Status: structural gates green; quality comparison open
Created: 2026-08-05
Scope: source-owning RealtimePreview streaming kernel

## What Landed

`RealtimePreviewStreamState` in `crates/signal-dsp-stretch/src/realtime_preview_stream.rs`,
isolated per Contract `084` Rule 2: nothing in the workspace constructs it and
`realtime_preview.rs` is untouched.

It has no input parameter. `push_source` is the non-realtime producer's entry
point and `render` is the callback's, so the callback pulls whatever source the
active ratio demands instead of being handed a fixed block. One ratio scheduler,
the source-projection one Batch 40.2 kept. Ratios outside `[0.25, 3.0]` are
rejected rather than clamped. Underrun emits silence and reports the shortfall.

Six gates in `tests/realtime_preview_stream_gates.rs`, in the frozen order. All
pass, and the full seven-gate release lane passes with them.

## The First G3 Was Worthless

Written as "every decile of output carries signal". It passed on the first run.

Run against the shipped quantum-locked kernel it also passes — `0/9` deciles
silent at ratio `0.5` and at `2.0`. That kernel does not fall quiet when it
drops source; it keeps emitting, from the wrong place. So the gate could not
fail for the reason it existed.

This is the `g10.039` failure in a new costume. There, five structural gates all
passed a renderer emitting nothing but zeros, because each measured a
relationship between renders rather than content. Here the gate measured
loudness, and dropping source preserves loudness. Both times the gate was
satisfied by the very defect it was written to catch, and both times the only
way to find that out was to run it against a known-bad implementation.

## The Replacement, And What It Measured

A linear `200` to `3000 Hz` sweep, so position in the sweep encodes position in
the source. Render `4s` of output from a `12s` source and ask what frequency the
output reached — that says exactly how much source was consumed — and whether it
ever moved backward, which says source was skipped.

At ratio `2.0`, `8s` of source and `8s` of output:

| kernel | reached | ratio implies | max backward jump |
| --- | --- | --- | --- |
| shipped | `2895 Hz` | `1600 Hz` | `141 Hz` |
| candidate | `1594 Hz` | `1600 Hz` | `12 Hz` |

The shipped kernel consumed almost the whole source to fill an output span that
should have taken half of it. That is the quantum lock, measured rather than
described, and the `141 Hz` backward jump is the ring guard skipping ahead.

The candidate lands within `0.4%` of the value the ratio predicts and stays
monotone. The expected number is derived from the ratio, not fitted to the
result, so the gate fails a kernel that drops source on the jump and one that
ignores ratio on the frequency.

## A Test Defect Worth Separating From A Kernel Defect

The first run of the new G3 failed at ratio `0.5` with `256` underrun frames.
That was the test, not the kernel: at ratio `0.5` a `4s` output consumes `8s` of
source, and the source was `8s` long, so it ran out on the final blocks. Source
extended to `12s`.

Recording it because "the gate failed, so loosen the gate" is the reflex this
generation has been trying to break. Here the gate was right and the fixture was
wrong, which is a different fix.

## Remaining

The quality comparison against the offline renderer is the acoustic half of
Batch 40.3 and has not been run. Batch 40.4 must not open until it has.

## Next Task

Measure preview quality against the offline renderer on the same material.
