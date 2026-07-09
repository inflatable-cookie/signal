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

- [x] add bounded-memory long-media processing or chunked artifact rendering
- [ ] define overlap/crossfade rules for chunk boundaries and warp-marker seams
- [x] support pitch automation or reject it with a product-visible capability
  contract
- [ ] widen linked processing beyond stereo when the channel-layout contract is
  ready
- [x] harden cache invalidation around media identity, engine version,
  projection epoch, ratio/pitch curves, and warp markers
- [x] add export/freeze/cache soak coverage with realistic source durations

## Execution Plan

### Batch 23.1 - Long-Media Artifact Shape

- [x] bounded-memory processing plan and deterministic chunk identity
- [x] chunk boundary overlap/crossfade policy

### Batch 23.2 - Capability Boundaries

- [x] pitch automation support decision or explicit product-visible rejection
- [x] multichannel support decision beyond linked stereo

### Batch 23.3 - Soak And Cache Hardening

- [x] realistic-duration export/freeze/cache soak tests
- [x] cache invalidation coverage for engine, projection, curve, marker, and
  media changes

## Acceptance Criteria

- [ ] peak memory is bounded and documented by tier
- [x] chunked output is deterministic and click-safe at chunk boundaries
- [x] unsupported pitch or channel behavior is explicit and observable
- [x] cache hits, writes, and invalidations remain auditable through runtime
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
- 2026-07-09: wired render-plane OfflineHighQuality default-path
  materialization to bounded chunk plans. Multi-chunk artifacts now render
  chunk payloads with source overlap context, trim to exact output
  coordinates, smooth chunk boundaries, and record chunk-count/source-span
  summaries in the materialization receipt. Selector paths remain whole-buffer
  static-ratio materialization until their chunked behavior has separate
  evidence.
- 2026-07-09: closed Batch 23.2 as explicit capability boundaries. Artifact
  planning now distinguishes unsupported capability from invalid identity or
  missing promotion: pitch automation is rejected in favor of one static pitch
  shift, and non-stereo materialization remains blocked until the channel-layout
  contract is ready. Runtime snapshots carry the same unsupported-capability
  readiness, and materialization/cache receipts now expose chunk-count and
  source-span summaries.
- 2026-07-09: closed Batch 23.3 with focused render-plane coverage for cache
  identity and receipt auditability. Materialization receipts now have tests
  proving distinct hashes/keys for engine version, media identity, projection
  epoch, ratio curve, pitch curve, and warp-marker changes. Chunk-policy
  changes are observable through receipt chunk summaries, and a bounded
  realistic-duration chunked artifact renders through the export fixture.
- 2026-07-09: added rendered chunk-boundary seam evidence. The test builds raw
  chunk payload joins, measures seam discontinuity, applies the artifact
  boundary smoothing policy, and verifies the measured seam click drops. Current
  memory posture is explicit but not fully closed: OfflineHighQuality default
  path processing is bounded by `max_chunk_render_source_frames`, while the
  materialized artifact buffer still holds full output PCM in memory until a
  streaming artifact writer/cache target lands.

## Next Task

Pause g10.023 code changes at the current contract boundary. Remaining open
items are structural: marker-specific warp seams need a warp-marker render
segmentation contract, full peak-memory closure needs a streaming artifact
writer/cache target, and multichannel widening stays blocked until the
channel-layout contract is ready.
