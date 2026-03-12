# 005 - Clip Rendering, Fades, And Nondestructive Processing Depth

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.004
Vision tags: `ENGINE`, `MEDIA`, `RENDER`

## Problem

Signal can process blocks and analyze media, but it still lacks a clear
reusable clip-processing substrate for fades, gain shaping, and nondestructive
render stages. Without that layer, later render/export and product editing
features will keep open-coding clip treatment outside Signal.

## Goals

- [x] define reusable clip-processing stages for fades and nondestructive treatment
- [x] keep clip render behavior compatible with tempo/warp and automation playback
- [x] establish one engine-owned clip render path for later offline export work

## Non-Goals

- [ ] no full comping or edit-decision-list workflow surface
- [ ] no final mastering or stem UX here

## Execution Plan

### Batch 5.1 - Clip Processing Contract

- [x] define fade, gain-shape, and ordered clip-treatment semantics in reusable Signal crates
- [x] separate realtime-safe render steps from heavier clip-preparation helpers where needed

### Batch 5.2 - Engine Proof

- [x] thread nondestructive clip processing through runtime-facing execution or render helpers
- [x] validate clip treatment against automation and warped timing cases

## Acceptance Criteria

- [x] fades and nondestructive clip treatment are reusable Signal behavior
- [x] clip render behavior composes with automation and warp semantics
- [x] later export/freeze work can reuse one stable clip-processing seam

## Risks and Mitigations

- Risk: clip processing becomes a product-only layer despite depending on engine timing.
- Mitigation: keep render stages and ordering rules inside Signal-owned crates.

## Evidence Requirements

- [x] log the clip-processing tranche
- [x] run focused render/runtime checks for fades and clip treatment ordering
- [x] record any explicitly deferred destructive-processing scope

## Next Task

Execute `g03.006` by making plugin/device-chain execution, latency
compensation, and degraded state/state-recall semantics explicit on top of the
now-stable timing and clip-render substrate.
