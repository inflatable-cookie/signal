# g10.040 Batch 40.3 - Isolated Implementation

Status: complete; seven gates green
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

## The Quality Metric Had To Be Calibrated Before It Could Be Used

Same material through the candidate, the whole-buffer preview at the same
`512`/`128` geometry, and the offline renderer at `2048`/`512`.

Level and spectrum match: RMS within `0.05%` of the whole-buffer preview at
every ratio, brightness within `1%`.

Waveform correlation looked bad at ratio `0.5` — `0.5153`, rising to `0.7025`
under a symmetric lag search, against `0.9992` at unity. No underrun, so not
starvation. The temptation is to read that as a defect at compression ratios.

The control says otherwise. Running the *same* whole-buffer algorithm against
itself with the source shifted half an analysis hop:

| ratio | grid-phase control | candidate vs whole-buffer |
| --- | --- | --- |
| `0.5` | `0.0639` | `0.7025` |
| `1.0` | `1.0000` | `0.9992` |
| `1.5` | `0.6179` | `0.9947` |
| `2.0` | `0.6246` | `0.9978` |

Identical DSP scores `0.064` against itself at ratio `0.5`. Waveform correlation
cannot resolve phase-vocoder quality away from unity, so `0.5153` was never
evidence of anything — the candidate clears the metric's own floor by `11x`.

`G7` is therefore self-calibrating: the control decides which standard applies.
Near-perfect control means the metric is reliable and the candidate must be
near-perfect too; a destroyed control means beating the floor is the only claim
available. A fixed correlation threshold would have measured frame-grid phase
and called it quality — the same species of error as the decile gate above, one
step further along.

At ratio `1.0` the control is degenerate: identity ratio returns the input
verbatim, so a shifted source still scores `1.0`. The gate takes `0.99` there.

## Admission

Seven gates green and no measured quality regression. Objective evidence cannot
settle whether the candidate sounds right, and Contract `084` Rule 5 makes
listening the promotion authority, so Batch 40.4 may open the integration but
admission still needs a listening round.

## Next Task

Open Batch 40.4, render-plane integration.
