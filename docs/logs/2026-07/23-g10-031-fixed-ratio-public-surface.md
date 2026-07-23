# g10.031 Fixed-Ratio Public Surface

Date: 2026-07-23
Batch: 31.75
Status: complete; Batch 31.76 wrapper implementation ready

## Result

Signal can expose the accepted fixed-ratio effect without claiming the paused
router. The frozen public boundary is a separate offline, fallible
`CreativeStretch` request:

- mono or interleaved stereo input
- sample rate and exact target frames
- exact `4x`, `8x`, or `16x`
- fixed semantic character `Dream`
- normalized `space`, default `0.5`
- fixed admitted seed
- typed request, capacity, and processing errors

`DirectRenewalDream` remains an internal renderer identity. Unsupported target
lengths fail; they do not fall back to `OfflineHighQuality`.

## Boundaries

The existing `TimeStretcher` trait, stretch tiers, offline path enum, and
transparent cache schema do not describe this request and remain unchanged.
Public seed/reroll, `motion`, `detail`, pitch, reverse, dynamic ratio, routing,
cache, artifacts, runtime DTOs, Loophole, and Chorus stay absent.

The wrapper is whole-buffer, allocating, deterministic offline work. It is not
audio-thread safe.

## Scope

This batch changed documentation only. The implementation gate permits one new
public wrapper module, crate-root wiring, fixed-seed visibility, and focused
tests. The four acoustic renderer files must remain byte-identical, and public
output must match the private renderer byte-for-byte.

## Next Task

Run Batch 31.76 only. Implement the frozen exact-ratio `CreativeStretch`
wrapper and focused tests without widening acoustic DSP, cache, routing, tiers,
controls, runtime, Loophole, Chorus, or cross-repo surfaces.
