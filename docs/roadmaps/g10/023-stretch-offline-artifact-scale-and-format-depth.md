# 023 - Stretch Offline Artifact Scale And Format Depth

Status: active
Owner: core-product
Created: 2026-07-07
Depends on: g10.018, g10.021, g10.022
Vision tags: `STRETCH`, `EXPORT`, `MEMORY`

## Problem

The current OfflineHighQuality artifact path proves the policy-gated
render/export/freeze shape with cacheable stereo PCM. Real projects need
long-media behavior, bounded memory, clear chunk boundaries, pitch automation
policy, and channel-layout depth before the path is ready for broad product
use.

## Goals

- [ ] add bounded-memory long-media processing or chunked artifact rendering
- [ ] define overlap/crossfade rules for chunk boundaries and warp-marker seams
- [ ] support pitch automation or reject it with a product-visible capability
  contract
- [ ] widen linked processing beyond stereo when the channel-layout contract is
  ready
- [ ] harden cache invalidation around media identity, engine version,
  projection epoch, ratio/pitch curves, and warp markers
- [ ] add export/freeze/cache soak coverage with realistic source durations

## Execution Plan

### Batch 23.1 - Long-Media Artifact Shape

- [x] bounded-memory processing plan and deterministic chunk identity
- [ ] chunk boundary overlap/crossfade policy

### Batch 23.2 - Capability Boundaries

- [ ] pitch automation support decision or explicit product-visible rejection
- [ ] multichannel support decision beyond linked stereo

### Batch 23.3 - Soak And Cache Hardening

- [ ] realistic-duration export/freeze/cache soak tests
- [ ] cache invalidation coverage for engine, projection, curve, marker, and
  media changes

## Acceptance Criteria

- [ ] peak memory is bounded and documented by tier
- [ ] chunked output is deterministic and click-safe at chunk boundaries
- [ ] unsupported pitch or channel behavior is explicit and observable
- [ ] cache hits, writes, and invalidations remain auditable through runtime
  receipts

## Validation

- `cargo test -p signal-render-plane`
- `cargo test -p signal-host-local --test public_host_edge_media_service`
- focused long-media/cache tests once added

## Progress

- 2026-07-07: opened as active g10 stretch scale and format work after the
  initial policy-gated stereo artifact path landed.
- 2026-07-09: added `signal-dsp-stretch` offline chunk planning primitives:
  deterministic payload ranges, render-context ranges, static per-chunk ratio,
  exact output coordinates, dynamic-ratio boundary preservation, and bounded
  maximum source payload policy. This does not yet change render-plane
  materialization.

## Next Task

Continue Batch 23.1 by wiring render/export/freeze offline artifact
materialization to the chunk plan, then implement the boundary
trim/crossfade policy without changing cache identity semantics.
