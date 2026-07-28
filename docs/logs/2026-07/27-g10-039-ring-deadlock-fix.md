# 2026-07-27 g10.039 Ring Deadlock Fix

Status: all six gates pass; renderer still unwired

## The Deadlock

The input and output rings were both twice the window, and emission released
only frames below `synthesis_start - window`. Those two facts meet exactly:

- the write frontier needs `synthesis_start + window < output_read + 2 * window`
- which is `synthesis_start - window < output_read`
- and `output_read` could only advance to `synthesis_start - window`

The two conditions touch at the same point, so neither side could move. `render`
then hit its capacity check, made no progress, and took the escape branch that
dropped the rest of the caller's chunk.

## The Fix

Three changes, each addressing one part of it:

The output and normalization rings are four times the window, separate from the
input ring, which stays at twice. The write frontier and the emission limit are
now a full window apart instead of touching.

Emission releases everything below `synthesis_start` rather than below
`synthesis_start - window`. The frame about to be written covers
`[synthesis_start, +window)`, so anything below that is final. The old bound was
conservative by exactly the amount that caused the stall.

`render` no longer has an escape that discards source. If a drain fails to
advance the analysis cursor it returns an error, which is unreachable with
correct ring sizing but fails loudly rather than silently truncating. That
escape is what turned a stall into `108` seconds of silence.

## Gates

| gate | result |
| --- | --- |
| `G1` chunk-size independence, static ratio | identical across four partitions |
| `G1b` chunk-size independence, dynamic ratio | identical across three partitions |
| `G2` memory ceiling, duration independence | `10616892 B` against `12582912 B` |
| `G3` output length matches target | passed at four ratios |
| `G4` correlation against a whole render | `1.000000` |
| `G5` audio across the whole source | every decile carries signal |

`G5` is the gate that mattered and did not exist until the listening round
forced it. It reproduced the defect inside the crate suite at `2.5 s`.

## The Ceiling Moved Twice

Both moves were the same mistake: deriving a bound before the design that has to
meet it was settled.

| published | figure | why it was wrong |
| --- | --- | --- |
| Batch 39.2 brief | `8 MiB` | inventory omitted the input ring |
| first correction | `9 MiB` | assumed output rings of twice the window, which deadlocks |
| now | `12 MiB` | output rings of four times the window, measured at `10616892 B` |

Contract `046` records all three and the lesson: a memory ceiling is a
consequence of a working design, not a constraint that can be frozen ahead of
one.

Duration independence still holds exactly. A `1000`-frame source and a ten-minute
source both measure `331836 B` at the retained geometry.

## Still Unwired

The artifact path remains on the legacy per-chunk renderer. Re-adoption needs
its own batch and its own listening round, and this time the pack should not be
built until a render has been checked for content first.

The `g10.036` seam pulse, `A18`, and both seam smoothers all remain shipped.

## Validation Run

- `cargo test -p signal-dsp-stretch --test resumable_gates`: `6` passed
- `cargo test --workspace`: green
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged
- corpus report unchanged
- `effigy qa:docs`

## Next Task

Re-adopt the resumable renderer in the offline artifact path, with a content
check on the rendered artifact before any listening pack is built. Then re-run
the `g10.039` listening round.
