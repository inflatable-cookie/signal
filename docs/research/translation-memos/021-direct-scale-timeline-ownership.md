# Direct Scale-Timeline Ownership

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AR
Contract: `082`, Rule 31Y

## Question

Why does Rule 31X preserve every ordinary/unlocked coefficient relation but
still fail `40/48` calibrated stereo rows? Which complete source-backed
topology remains admissible?

## Finding 1: Unlocked Was Not A Linked State

The Rule 31W trace named independent `Unlocked` commits as the first difference
from an exact current-relation observer. That observer was not a professional
synthesis invariant.

Pinned Rubber Band R3 `4.0.0` source is explicit:

1. every channel computes its own ordinary instantaneous-frequency recurrence
2. reset and kick use that channel's current analysis phase
3. high-unlocked bins use that channel's ordinary recurrence
4. channel borrowing is considered only in the locked peak branch
5. borrowing requires both channels inside the link range and compatible prior
   peak ownership
6. the peer keeps its local analysis-relative offset from the borrowed peak

Signalsmith projects all peers from one reference inside a different complete
single-grid predictor. Bungee applies one common rotation inside a different
whole-region kernel. Neither law supports replacing Rubber Band-style
unlocked recurrence with unconditional common rotation.

Rule 31X therefore hybridized incompatible kernels. Its local improvement is
real but not promotable. Exact same-atom relation is a diagnostic, not a state
definition.

## Finding 2: Signal Added A Meta-Slice Layer

Rule 31R selected direct simultaneous low, middle, and high transforms. Each
scale owns one coefficient timeline, one phase state, one inverse transform,
and one per-channel overlap-add path. Output frequencies are exclusive across
scales.

The bounded implementation changed that shape. It analyses each `32H` outer
slice into an inner frequency-adaptive frame. Every global coefficient tick is
represented in two independently windowed outer fields. One dominant field
drives state; its decision is projected into both active fields; two inverse
slices then overlap in the waveform domain.

For ordinary/unlocked Rule 31X, projection simplifies to the same rotation on
each local layer coefficient. That proves the projection adds no new local
relation error. It does not make the two modified fields one direct scale
timeline. In general,

`sum_s h_s D_s M C_s(h_s x) != D M A x`.

Identity proves the left side equals `x` when `M` is inert. It says nothing
about closure after phase-state modification. The two-layer construction is a
valid bounded representation and a rejected quality topology.

Rubber Band, Signalsmith, and Bungee all avoid this extra ownership seam. Their
normal frame overlap belongs to one coefficient timeline per active kernel;
they do not duplicate one phase-state tick across independently analysed
meta-slices.

## Recorded Evidence

Rule 31X reports zero coefficient relation error but still reaches:

- `40/48` calibrated failures
- `44/48` Signal-relative local-row failures
- maximum normalized-Gram residual `0.8700034314389535`
- compression image errors above `5 dB` mid/side delta and `0.25` relation
  residual on representative whole rows

The failure is not a missing peer phase copy. It is complete-kernel
non-conformance: over-linked unlocked state plus an extra meta-sliced
synthesis field.

## Selected Boundary

Close `NormalizedSlicedMaterialPolicy` as a quality candidate. Retain its
identity, capacity, and boundary proofs as representation evidence only.

The only admissible next family is a direct frequency-partitioned scale
timeline:

1. one bounded source/output centre schedule shared by all channels and scales
2. one direct STFT frame sequence per low, middle, and high scale
3. one coefficient and phase-state owner per scale/time/bin; no outer
   meta-slice or dominant-layer projection
4. exhaustive, nonoverlapping physical-frequency ownership across scales
5. channel-local ordinary recurrence, reset, attack, and unlocked states
6. cross-channel borrowing only in predecessor-compatible locked peak regions
7. peer magnitude and current peak-relative analysis offset retained when
   borrowing
8. per-channel inverse/window overlap-add per scale; scale sums remain inside
   that channel
9. fixed input, output, guidance, peak, and phase rings; no duration-sized
   coefficient store

This is the topology already selected from Rubber Band in memo 019. The next
work restores conformance instead of inventing another phase law.

## Old Prototype Boundary

Batch 29.6CH does not reopen. It had direct `1024/2048/4096` transforms, but it
also had incomplete state semantics, unconditional same-bin channel projection,
hard peak locking, dynamic valley policy, and independent per-scale overlap
normalization. Listening rejected the complete result.

Its code is evidence for implementation hazards, not a base to promote. Batch
29.7AR must state which representation mechanics remain valid and reject the
rest before any renderer runs.

## Decision

No renderer opens in this batch. First preregister a direct scale-timeline
representation with physical geometry, frequency coverage, overlap ownership,
fixed capacities, boundary schedule, and exact state order. The proof must show
that each active scale/time/bin has one state owner and that no coefficient is
projected between outer fields.

Objective retry, tuning, listening, holdout, dynamic ratio, realtime, routing,
cache, production, and product work remain closed.

## Sources

- [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [Rubber Band R3 stretcher](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/R3Stretcher.cpp)
- [Signalsmith Stretch source](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h)
- [Bungee synthesis source](https://github.com/bungee-audio-stretch/bungee/blob/746833f68a574d997ec50443e7cfd2d37b026302/src/Synthesis.cpp)
- [Rule 31X evidence](../../logs/2026-07/18-g10-029-reference-relative-unlocked-commit.md)
- [Shared-decision topology](./019-shared-decision-waveform-topology.md)
- [Bounded sliced representation](./020-bounded-normalized-sliced-integration.md)

## Next Task

Run Batch 29.7AR under Rule 31Y. Preregister the direct scale-timeline
representation and state order without implementing or rendering audio.
