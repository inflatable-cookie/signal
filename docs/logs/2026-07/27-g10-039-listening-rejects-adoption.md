# 2026-07-27 g10.039 Listening Rejects The Adoption

Status: adoption reverted from `main`; renderer defect open

## Operator Findings

All three `A` specimens were silent. The key assigned `A` to the adopted side in
every case, so this is not a preference result: the adopted artifact path
produces silence.

| case | `A` | `B` | verdict |
| --- | --- | --- | --- |
| `D1` | adopted | shipped | `A` silent; `B` clean, regular tick, no secondary pulsing |
| `D2` | adopted | shipped | `A` silent; `B` clean except a low-mid pop on the ticks |
| `D3` | adopted | shipped | `A` silent; `B` clean, slight low-end pop on the ticks |

Measured RMS on the pack files confirms it: `0.000000` for all three `A` sides
against roughly `0.1457` for `B`.

The `B` observations are the shipped renderer and match the `A18` finding
already recorded: low-end pops on transients, subtle, present before this lane.

## The Defect

An RMS envelope over the full adopted render, ratio `1.25`, `90` seconds of
source:

| output position | rms |
| --- | --- |
| `0.0 s` | `0.026772` |
| `3.8 s` onward, every sample point to `108.8 s` | `0.000000` |

The renderer emits under four seconds of audio and then stops.

`render` feeds the input ring in capacity-bounded slices and drains between
them. When the output ring cannot emit — because the safe-emission point has
not advanced past the read cursor — `drain` returns zero progress while the
input ring is full. The escape added for that case breaks out of the render
loop, **silently dropping the rest of the caller's chunk**. The two rings can
deadlock, and the code chose to discard source rather than fail.

The output then reaches its contracted length by padding with zeros, so nothing
downstream noticed.

## Why Five Structural Gates Missed It

This is the part worth keeping.

`G1` chunk-size independence passes because silence is consistently silent: two
partitions of a stalled render agree exactly. `G3` output length passes because
the renderer pads to the target itself. `G4` correlation passes because it
compares two renders that share the same short prefix and the same silence.

Every gate measured a *relationship* between renders. None asserted that a
render contains audio. A renderer that emitted nothing but zeros would have
passed four of the five gates I wrote.

`G5` is new and closes that: every decile of the output must carry signal. It
reproduces the defect inside the crate's own suite, failing at `2.5 s`, so this
never needed a listening round to catch. It is `#[ignore]`d with the measured
reason while the defect is open.

## Reverted

The artifact path is back on the legacy per-chunk renderer.
`SIGNAL_STRETCH_BEHAVIOR_VERSION` is back to `signal-stretch-behavior-2026-07-27`,
because shipped output is once again the pre-adoption renderer, and the
adoption helpers are removed from `signal-render-plane`.

`ResumableOfflineStretch` stays in the crate. It is unwired, and its five
structural gates plus the new failing one are the record of exactly how far it
got.

## What This Costs

The `g10.036` seam pulse is still shipped. The adoption that was supposed to
remove it is withdrawn, so `A18` and the pulse both remain open, and both seam
smoothers stay.

## Validation Run

- pack RMS measurement confirming the silent side
- RMS envelope over the full adopted render
- `cargo build --workspace --all-targets`: no errors, no warnings
- `cargo test --workspace`: green
- `effigy qa:docs`

## Next Task

Fix the ring deadlock in `ResumableOfflineStretch` before any further adoption.
`render` must never discard source: if the output ring cannot advance, the
renderer must emit further before accepting more input, and the two ring sizes
must be chosen so that emission can always progress. Activate `G5` as part of
that work, and re-run the listening round only once `G5` passes.
