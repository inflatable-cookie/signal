# g10.029 Material Transport Architecture Reassessment

Date: 2026-07-18
Batch: 29.7Z
Status: complete; one final proof selected

## Attribution

Batch 29.7Y independently polar-interpolates each channel, then applies one
common material operator. That order does not preserve linked phase.

Exact counterexample:

- reference phase: `0` to `+170` degrees
- peer phase: `0` to `-170` degrees
- independent midpoint relation: `-170` degrees
- endpoint-relation midpoint: `+10` degrees
- error before material operation: `180` degrees

The common operator cancels from channel difference and cannot repair the
error. This explains the simultaneous IPD, mid/side, correlation, and aggregate
relation failures with zero structural failure and exact silent peer.

## Research Decision

Dorran-Lawlor-Coyle and pinned Signalsmith evidence make the current
peer/reference relation explicit after one reference phase result. Memo 014
selects the same invariant in Signal terms: sample reference phase once,
interpolate peer/reference relation once, keep peer magnitude, then apply the
frozen common material operator.

Holighaus et al. supply a fixed sliced frequency-adaptive frame with exact
sliced reconstruction and linear work. The selected Signal geometry is:

- transform span: `16384`
- outer advance: `8192`
- coefficient lattice: `512`
- outer windows: identical
  `h[n] = sin(pi (n + 0.5) / 16384)` analysis/synthesis pair
- overlap law: `h[n]^2 + h[n + 8192]^2 = 1`
- maximum active slices: two

The sliced frame is a new exact representation. It is not claimed
coefficient-equivalent to the rejected whole-source transform.

## Promotion

- translation memo 014: promoted
- architecture: relation-owned sliced successor selected
- contract `082`: Rule 31J
- roadmap: Batch 29.7AA ready at Stage A only

No DSP, tuning, listening export, dynamic ratio, realtime, routing, cache, or
product surface changed.

## Next Task

Run Batch 29.7AA Stage A. Implement only sliced identity, relation mechanics,
boundaries, duration-independent peak working memory, linear counted work, and
repeat. Keep material transport closed until Stage A passes.
