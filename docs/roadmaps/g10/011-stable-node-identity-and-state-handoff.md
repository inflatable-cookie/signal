# 011 - Stable Node Identity And State Handoff

Status: complete
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

- [x] stable u64 ids on nodes and clips in the plan spec (pulse supplies them from its own ids)
- [x] controller precomputes an old→new inheritance map at install time (it knows the previous plan) and ships it inside the plan; executor inheritance becomes O(n) index copies, zero string work on the audio thread
- [x] heavy per-node state (delay buffers, later ring handles) moves between plans via the map (Box swap), never reset and never freed on the audio thread
- [x] property test: arbitrary insert/remove/reorder edit sequences preserve tone phase and smoothed gains for surviving nodes

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

## Progress (2026-06-11)

- `RenderClipSpec.clip_id: u64` joins the stage ids from g10.010 (pulse
  supplies FNV-1a of its clip ids). The controller retains an identity
  snapshot of the last installed plan's topology (stage ids + clip ids in
  topo order) and precomputes `inherit_stage_map` / `inherit_clip_maps`
  into each new plan at install time. `last_topology` only updates when the
  mailbox accepts the install, so a rejected send cannot desync the maps.
- Executor `inherit_state` is now O(stages + clips) index copies — zero
  identity comparisons on the audio thread; mid-lane clip inserts no
  longer cross-wire neighbour state (the zip-index bug is dead).
- Heavy-state movement (delay buffers, streaming cursors) rides the same
  maps when those states exist; today the carried state is smoothed gains
  + tone phases.
- Tests: mid-lane insert continuity (survivor keeps phase, step < 0.05),
  stage reorder continuity, and a seeded-LCG churn test (24 installs
  mid-play adding/removing lanes, inserting clips, jittering gains — the
  surviving tone never steps). 27 render-plane tests green; soak still
  zero-alloc; pulse 122/122; aura green; clippy clean.
- `install_plan` is now `&mut self` (controller carries the topology
  snapshot); hosts updated.

## Next Task

g10.012 (parameter fast path + automation).
