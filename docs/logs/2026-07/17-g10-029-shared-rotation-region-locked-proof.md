# g10.029 Shared-Rotation Region-Locked Proof

Date: 2026-07-17
Batch: 29.7T
Status: complete; objective passage rejected

## Change

Added one report-only fixed-grid `SharedRotationRegionLocked` kernel. Joint
maximum-channel energy defines peaks and valley-bounded regions. A deterministic
owner advances each tracked peak. One common rotation owns every channel and
bin in the region. `TrackedRegion`, `ResetRegion`, and `Silent` are complete.

The kernel does not call the coherent weighted predictor. Production, routing,
cache identity, dynamic ratio, realtime, listening, and Batch 29.8 remain
unchanged.

## Result

- calibrated stereo failures: `20/48` current, `1/48` candidate
- row-complete stereo improvements: `30/48`
- stereo rows with any metric regression: `18/48`
- local-consistency failures: `11/48`, all tone rows
- sole calibrated miss: short off-bin `2.0x` tone, `0.009708 rad` whole IPD
  against `0.006 rad`
- mono corpus: zero hard failures, zero row-complete regressions
- mechanics: zero structure/identity/mono-parity/hard-pan/swap/gain failures;
  polarity error `5.20e-14`; silent peer exact zero
- states: `58,352` tracked, `1,445` reset, `165` silent, `59,797` regions,
  `7,710` owner switches

An initial shared-normalization bookkeeping defect divided output by two. The
final proof uses one overlap-weight buffer per channel. No phase, peak, region,
trajectory, reset, window, or threshold policy changed.

## Evidence

- stereo: `eff52febad8c0fb8`
- mechanics: `ad907a31d6ae940a`
- mono corpus: `c062525dfa1da3ff`

All repeat exactly.

## Decision

Reject passage. The family is materially stronger than the current linked
kernel, but the frozen zero-failure and zero-local-failure bar is unmet. Batch
29.7U owns tone-continuity operator review. It may identify one source-backed
operator-law correction or close the family; it may not tune this renderer.
