# g10.029 Relation-Owned Sliced Material Proof

Date: 2026-07-18
Batch: 29.7AA Stage B
Status: complete; family closed

## Change

Added the one frozen relation-owned material candidate to the passing sliced
frame. Source slices use bounded caching. Both output layers share one global
source position, material state, deterministic reference, relation identity,
region rotation, and perturbation key. Peer magnitude remains layer- and
channel-owned. The five defined/undefined/silent relation controls are exact.

## Result

Synthetic structure, repeat, hidden-gain, and bounded-execution gates pass.
Peak live source slices are five; peak live output slices are two. Coefficient
relation error is `4.44e-16` on synthetic material and `1.78e-15` on calibrated
stereo. Active calibrated rows have zero undefined relations. Exact relation
mechanics exercise two-defined, one-defined, undefined, zero-peer, and silent
states with zero error.

The sample-domain result rejects:

- calibrated failures: `44/48`
- local-consistency failures: `46/48`
- row-complete improvements: `16/48`
- rows with metric regressions: `32/48`
- frozen current failures: `20/48`
- silent-peer peak: exact zero
- stereo evidence hash: `225ab337875b3962`

Frozen mono-parity mechanics also miss even though the internal swap, polarity,
hard-pan, duplicate, identity, repeat, and structure controls remain exact to
`4.41e-14`. The stereo stop gate prevents the long mono corpus from running.

## Decision

Close relation-owned material transport on the current frequency-adaptive
sliced synthesis family. The independent source-interpolation bug is fixed,
but exact per-atom and per-layer relation is not sufficient through the
redundant band and slice synthesis sum. Do not tune, repair overlap, export a
listening pack, or open Batch 29.8.

## Next Task

Run Batch 29.7AB as a no-renderer joint-synthesis architecture reassessment.
Attribute the first post-coefficient relation divergence and review primary
multichannel redundant-frame synthesis evidence before selecting any new DSP
family.
