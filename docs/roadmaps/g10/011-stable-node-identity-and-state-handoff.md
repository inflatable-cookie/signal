# 011 - Stable Node Identity And State Handoff

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.010
Vision tags: `ENGINE-SPINE`, `RT-SAFETY`

## Problem

Plan-swap state inheritance does O(lanes²) string compares on the audio
thread and matches clips by zip-index, which breaks the moment a clip is
inserted mid-lane. Retained per-node DSP state (filter memories, delay
lines, streaming cursors later) is impossible without stable identity.

## Goals

- [ ] stable u64 ids on nodes and clips in the plan spec (pulse supplies them from its own ids)
- [ ] controller precomputes an old→new inheritance map at install time (it knows the previous plan) and ships it inside the plan; executor inheritance becomes O(n) index copies, zero string work on the audio thread
- [ ] heavy per-node state (delay buffers, later ring handles) moves between plans via the map (Box swap), never reset and never freed on the audio thread
- [ ] property test: arbitrary insert/remove/reorder edit sequences preserve tone phase and smoothed gains for surviving nodes

## Execution Plan

### Batch 11.1 - Identity And Map

- [ ] ids in spec + compile; controller-side map construction
- [ ] executor O(n) inherit; state moves by map

### Batch 11.2 - Proofs

- [ ] property test over edit sequences
- [ ] soak with continuous plan churn stays zero-alloc

## Acceptance Criteria

- [ ] no string comparison remains in render_block or inherit paths
- [ ] mid-lane clip insert preserves neighbouring clip state
- [ ] churn soak green

## Next Task

g10.012 (parameter fast path + automation).
