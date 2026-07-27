# 2026-07-27 g10.036 Batch 36.4 Listening Round One

Status: revision 2 rendered; second listening round pending operator

The first concealed pack was auditioned. One case rejected the corrected side
on a defect the objective rows had not measured, so the segment minimum was
re-chosen and a second pack was built.

## Operator Findings

| case | concealed A | concealed B | operator verdict |
| --- | --- | --- | --- |
| `C1` | post | pre | A: secondary rhythmic pulse, sounds like segments overlapping, click smeary. B: a whole octave below the source |
| `C2` | pre | post | pretty much identical, both very slight smear on the tick |
| `C3` | pre | post | both mostly clean, both pop in the low end on the ticks |
| `C5` | pre | post | A: a lot of very loud pops at high speed. B: no pops, clean, ticks smeared |

Read against the key:

- `C5` confirms the Batch 36.3 overlap correction. The pre side is the loud-pop
  side; the corrected side is clean. The residual smear at ratio `4.0` is
  ordinary phase-vocoder behavior, not a defect this lane introduced
- `C2` is a tie. The mono seam correction is inaudible on this material, which
  is consistent with the objective rows: the pre-correction click sits at
  `-28.94 dBFS` under a sustained chord
- `C1` rejects the corrected side's new artifact. The pre side is unusable —
  an octave error — so no case prefers the pre-correction renderer, but the
  corrected side was not acceptable either
- `C3` reports low-end pops on ticks in **both** sides. Both are renders from
  this batch's pre and post renderers, so that defect predates this work

## The C1 Artifact Was Real

The reported "secondary rhythmic pulse ... like segments overlapping" was
measured, not dismissed. Envelope modulation at the segment period, against the
same material rendered whole as a floor:

| render | modulation at segment period |
| --- | --- |
| dense curve, `window + 8 hops` minimum | `0.545 dB` |
| same material rendered whole | `0.044 dB` |

The period matched the segment length exactly: `6144` source frames at ratio
`2.0` is `12288` output frames, `3.9 Hz`. The renderer was modulating once per
segment join.

## Cause And Revision

Dynamic-ratio segments render independently and are butt-joined, so every join
leaves an envelope dip. Fewer joins, less pulse. Swept on the same material:

| minimum | source frames | joins | modulation | dense-curve pitch error |
| --- | --- | --- | --- | --- |
| `window + 8 hops` | `6144` | `31` | `0.545 dB` | `2.8` cents |
| `window + 16 hops` | `10240` | `18` | `0.268 dB` | `2.8` cents |
| `window + 32 hops` | `18432` | `10` | `0.115 dB` | `0` cents |
| `window + 64 hops` | `34816` | `5` | `0.039 dB` | `0` cents |

`window + 64 hops` reaches the whole-render floor, but its `725 ms` minimum
swallows realistic tempo-ramp spans, including the `500 ms` spans in the `C3`
case. `window + 32 hops` is frozen instead: `18432` source frames, `384 ms` at
48 kHz. It cuts the modulation `4.7x` and takes dense-curve pitch to exactly
`440.0 Hz`.

Contract `046` is amended with the sweep, the chosen value, and an explicit
statement that segment-rate modulation cannot reach zero while segments render
independently. It is a recorded limitation of this law, not a tuning target.
`g10.039` removes it by carrying renderer state across the join.

## Cost Made Visible

The longer minimum has a price: a curve whose spans are shorter than `384 ms`
loses its individual ratio changes and renders at the mean ratio over the
merged span. Total output length still holds exactly.

This surfaced as a test failure rather than silently.
`offline_high_quality_dynamic_ratio_smoothing_reduces_segment_seams` derived
its seam positions from a curve with `333 ms` spans, which now merge to one
segment, leaving no seams to measure. The owner tests the smoother, not the
segmentation law, so it now takes explicit boundaries.

`sub_minimum_curve_spans_render_at_their_mean_ratio` is new and records the
cost directly, using that same `333 ms` curve.

## New Finding: A18

Low-end pops on transients in both `C3` sides. Both sides are this batch's own
pre- and post-correction renderers over the same source, so the defect is
pre-existing and is not caused by `g10.036`. It is unmeasured so far and needs
its own reproduction before any fix is designed.

Candidate mechanisms, none confirmed: the transient phase reset introducing a
low-frequency step at the reset frame, or the seam smoother's decaying DC
offset. It is recorded here for triage rather than routed to a batch, because
`g10.036`'s scope is the four audited defects and this is a fifth.

## Revision 2 Pack

`~/Downloads/signal-listening-pack-36-4-rev2`. Both sides are corrected
renders: `128 ms` minimum against `384 ms` minimum, randomized per case in
`key.tsv`.

- `C1` dense curve: does the secondary pulse drop
- `C3` tempo ramp: are the ramp spans still intact, and is anything worse
- `C5` supplied unpaired and unchanged; it already passed

## Validation Run

- `cargo test -p signal-dsp-stretch`: `182` lib tests, `11` owners, green
- segment-minimum sweep and modulation measurement through temporary probes,
  deleted after use
- `effigy qa:docs`

## Next Task

Operator action: audition `~/Downloads/signal-listening-pack-36-4-rev2`, fill
`notes.tsv`, then open `key.tsv`. Batch 36.4 is admitted if the `384 ms`
minimum is preferred or tied on both cases. On admission, execute Batch 36.5.
Triage `A18` separately; it is not part of this lane.
