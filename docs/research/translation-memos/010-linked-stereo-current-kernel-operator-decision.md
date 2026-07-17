# Linked-Stereo Current-Kernel Operator Decision

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7R
Contract: `082`, Rule 31H

## Question

Five bounded batches tested whether tracked peak ownership can repair linked
stereo inside Signal's current coherent weighted-predictor kernel. This review
decides whether another local intervention remains justified or whether peak
state requires a separately designed phase-field kernel.

## Consolidated Evidence

| Renderer or ablation | Calibrated failures | Complete improvements | Metric regressions | Local failures |
| --- | ---: | ---: | ---: | ---: |
| current relational coherent kernel | `20/48` | - | - | - |
| channel-independent recurrence | `40/48` | `8/48` vs current | `40/48` vs current | `34/48` |
| independent plus shared peak regions | `29/48` | `26/48` vs independent | `22/48` vs independent | `32/48` vs current |
| late reference-safe tracked overlay | `25/48` | `0/48` vs current | `48/48` vs current | `34/48` |
| complete peak-owned regions | `23/48` | `2/48` vs current | `46/48` vs current | `27/48` |
| Rubber Band R3 standard control | `0/48` | comparator only | comparator only | not the Signal local gate |

All Signal candidates are repeat-stable, structurally exact, mechanics-exact,
mono-parity safe, and silent-peer safe. The failures are quality failures, not
inactive code or broken plumbing.

Batch 29.7P locates the late overlay's conflict across anchors, interiors, and
boundaries. Batch 29.7Q then corrects that order and recovers two failures plus
seven local rows. Operator order was a real fault. It was not the whole fault.

## Kernel Boundary

The current coherent kernel translates Signalsmith Stretch's pure time-stretch
topology. It builds one continuous phase field from horizontal transport plus
weighted observations from both frequency directions, then makes one
greatest-energy channel own each bin while preserving peer relation. Pure time
stretch uses an identity frequency map; peak mapping is not part of its phase
owner.

Rubber Band R3 uses a different composition. Ordinary phase advance, tracked
peak advance, local offsets, reset, unlocked ranges, kick guidance, channel
linking, and simultaneous frequency-scale ownership are resolved inside one
state-complete phase-vocoder kernel. Its linked peak path is not an isolated
operator that can be transferred faithfully into a continuous weighted field.

The failed sequence therefore crossed kernel families:

1. 29.7M replaced relational recurrence with independent recurrence plus peak
   regions
2. 29.7O restored relational recurrence but overlaid independently advanced
   peak fields after integration
3. 29.7Q constructed complete peak-owned regions but still switched between a
   peak-state law and a continuous predictor without the surrounding phase
   states and synthesis policy

The monotonic improvement from `29` to `25` to `23` failures shows that each
ownership correction mattered. Failure to beat the `20`-failure baseline shows
that another owner, range, picker, scale, or blend variant is not a bounded
test of the same hypothesis.

## Decision

Close linked tracked-peak work inside the current coherent kernel. That kernel
retains one `Relational` linked-stereo phase owner and remains report-only. Do
not add `TrackedPeak` as an overlay, region replacement, seed, or local state.

This does not reject tracked peaks generally. It rejects transferring one
phase-vocoder state into this kernel without its complete phase-state and
synthesis context. Any further peak-state work requires a separately
contracted kernel family with explicit ownership of:

- ordinary horizontal advance
- vertical or region coherence
- tracked peak trajectories
- reset and unlocked states
- linked-channel decisions
- representation and scale assignment
- overlap synthesis continuity

Transient reset remains a separate future question. It must not be introduced
as parameter rescue for these steady controls.

## Next Research Question

Compare two complete, clean-room linked phase-field families before selecting
another renderer:

1. joint multichannel phase-gradient integration without explicit peak state
2. a state-complete peak-locked phase vocoder with ordinary, tracked, reset,
   unlocked, and linked-channel ownership designed together

Rubber Band remains architecture and behavioral evidence only. Signalsmith and
the current Signal kernel remain continuous-field controls. Selection must
define representation, phase state, stereo ownership, transient policy,
synthesis continuity, licensing boundary, and objective gates as one system.

## Outcome

Batch 29.7S completes that comparison. Translation memo 011 closes joint PGHI
for the next renderer and selects one separate shared-rotation region-locked
phase-vocoder proof. The current coherent kernel remains unchanged.

## Rejected And Deferred

- reject another current-kernel peak owner, picker, predecessor, range, offset,
  threshold, or blend variant
- reject treating the `23/48` result as evidence for a parameter sweep
- reject listening or dynamic-ratio work before a fixed-ratio objective pass
- defer renderer implementation until one complete family is contracted
- keep realtime, routing, production, and Batch 29.8 closed

## Sources

- [Signalsmith Stretch 1.3.2 pinned source](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h)
- [Signalsmith Stretch design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/)
- [Laroche and Dolson, Improved Phase Vocoder Time-Scale Modification of Audio](https://doi.org/10.1109/89.759041)
- [Ottosen and Dörfler, A Phase Vocoder based on Nonstationary Gabor Frames](https://arxiv.org/abs/1612.05156)
- [Průša and Søndergaard, Real-Time Spectrogram Inversion Using Phase Gradient Heap Integration](https://dafx.de/paper-archive/2016/dafxpapers/03-DAFx-16_paper_02-PN.pdf)
- [Průša and Holighaus, Phase Vocoder Done Right](https://ltfat.org/notes/ltfatnote050.pdf)
- [Rubber Band R3 source architecture, GPL architecture evidence only](https://github.com/breakfastquay/rubberband/tree/v4.0.0/src/finer)
