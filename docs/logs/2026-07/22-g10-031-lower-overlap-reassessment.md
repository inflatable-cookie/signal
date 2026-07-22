# g10.031 Lower-Overlap Reassessment

Date: 2026-07-22
Status: Batch 31.68 complete; lower overlap paused; Batch 31.69 ready
Scope: docs-only coherent/Dream architecture decision

## Result

No complete `2x..4x` overlap architecture exists without changing an admitted
renderer.

`OfflineHighQuality` supports arbitrary positive fixed ratios on its centered
padded STFT scheduler. Private `DirectRenewalDream` supports only exact `4x`,
`8x`, and `16x` on a different long-window scheduler and asymmetric boundary
envelope. It cannot produce the mandatory exact `2x` or interior overlap
probes.

A hard switch at `4x`, post-resampling, a second stretch pass, or an
exact-`4x` output blend does not supply both owners on Contract `085`'s one
map, one boundary alignment, and complete probe set. The lower overlap remains
paused. Neither renderer changes or fails.

## Scope

This batch changed documentation only. It added no DSP, candidate harness,
ratio support, blend, route, control, cache, dynamic-ratio state, Loophole, or
Chorus surface.

## Reopening Boundary

Reopening requires either one separately admitted complete lower creative
owner covering every required `2x..4x` ratio, or a newly versioned and
re-admitted generalized Dream renderer. An adapter or parameter sweep is not
enough.

## Next Task

Batch 31.69 only. Study one complete `LayeredCloud` owner for the future
`32x..100x` range and exact `16x`/`32x` boundary obligations. Change
documentation only. Freeze at most one source-backed complete owner brief or
close the lane.
