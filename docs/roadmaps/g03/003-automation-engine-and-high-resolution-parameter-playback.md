# 003 - Automation Engine And High-Resolution Parameter Playback

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.001, g03.002
Vision tags: `ENGINE`, `AUTOMATION`, `CONTROL`

## Problem

Signal has parameter event application, but not yet a deliberate engine-level
automation playback model across routed mixer and plugin/device-chain style
execution. Without that depth, later render and clip-processing work will
either under-spec automation behavior or reimplement control semantics outside
the runtime.

## Goals

- [x] deepen sample-accurate automation playback across routed engine targets
- [x] make automation timing and smoothing semantics explicit and testable
- [x] expose enough runtime observation to validate control playback against engine output

## Non-Goals

- [x] no workflow/editor semantics for automation authoring
- [x] no modulation-matrix product breadth yet

## Execution Plan

### Batch 3.1 - Automation Contract Deepening

- [x] expand reusable automation segment, smoothing, and target semantics where the current parameter batch model is too thin
- [x] pin block-boundary and cross-block playback expectations for deterministic render behavior

### Batch 3.2 - Runtime Playback Proof

- [x] thread higher-resolution automation playback through `signal-graph` and `signal-runtime`
- [x] validate automation-driven output changes on routed mixer and plugin-backed node fixtures

## Acceptance Criteria

- [x] automation playback semantics are explicit enough for later render and plugin-chain milestones
- [x] control timing remains Signal-owned engine behavior instead of host-local interpretation
- [x] focused fixtures prove deterministic parameter playback through multiple blocks

## Risks and Mitigations

- Risk: automation depth gets trapped in product-specific editing semantics.
- Mitigation: keep the milestone scoped to execution and observation behavior only.

## Evidence Requirements

- [x] log the automation playback tranche
- [x] run focused graph/runtime tests for multi-block automation playback
- [x] capture any unresolved smoothing or target-model choices explicitly

## Next Task

Execute `g03.004` by defining reusable tempo-map ownership, warp modes, and
realized playback state surfaces before proving degraded and not-ready warp
reporting through `signal-runtime`.
