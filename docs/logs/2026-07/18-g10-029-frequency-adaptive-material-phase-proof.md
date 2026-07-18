# g10.029 Frequency-Adaptive Material-Phase Proof

Date: 2026-07-18
Batch: 29.7Y Stage B
Status: complete; architecture passage rejected

## Change

Added the one authorized report-only `FrequencyAdaptiveMaterialPhase`
candidate on the passing Stage A frame. One channel-joint fuzzy map owns tonal,
transient, and noise guidance. Transient atoms use shoulder suppression and one
reset centre. Other atoms retain common peak-region rotation and receive one
deterministic channel-common noise perturbation. Supports, crossovers,
classifier law, transient law, diffusion law, seed, and objective gates remain
frozen.

## Result

The completed linked-stereo report rejects:

- calibrated failures: `36/48`
- local-consistency failures: `46/48`
- structural failures: zero
- current calibrated failures: `20/48`
- row-complete improvements: `16/48`
- rows with metric regressions: `32/48`
- silent-peer peak: exact zero
- evidence hash: `b986ed62e2cadefe`

Candidate IPD, mid/side, correlation, and aggregate relation errors rise
together. This happens before the common material operator can preserve them:
each channel independently polar-interpolates its source coefficient, so the
relative phase entering the shared operator is no longer constrained. This is
the leading architecture attribution, not a parameter-tuning result.

The monolithic objective process completes synthetic, stereo, and mechanics
work, then remains CPU-bound in the repeated six-row, five-second mono corpus
after more than five hours. It is stopped because the mandatory stereo gate has
already made passage impossible. No complete mono verdict or aggregate proof
summary is claimed.

## Decision

Reject Stage B passage. Do not tune the candidate and do not export a concealed
listening pack. Keep Batch 29.8, dynamic ratio, realtime, routing, cache, and
product work closed.

Batch 29.7Z owns a no-DSP architecture reassessment. It must either define a
source-backed relation-preserving coefficient-resampling law plus a bounded
sliced proof shape, or close this family.
