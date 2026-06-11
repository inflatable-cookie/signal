# 010 - Graph Shaped Plans And Mixer Realization

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.008
Vision tags: `ENGINE-SPINE`, `MIXER`, `RENDER-PLANE`

## Problem

The render plane is a flat stereo lane/clip player: lanes sum straight into
the callback buffer, there is no pan (identical samples on both channels),
no busses, no sends, no solo/mute semantics, and `channels.min(2)` bakes in
the stereo assumption. Pulse's console model (channel chains, busses, sends)
renders knobs in Aura that move no audio beyond lane gain. Every future
feature — inserts, sends, PDC, metering targets, automation targets — needs
node addresses that do not exist.

The control/render split accommodates this unchanged: a compiled DAG is
still an immutable plan; the mailbox/retire lifecycle stays.

## Goals

- [ ] compiled node schedule: lane → bus → master as a topologically-ordered flat execution list (indices, not pointers), scratch buffers owned by the plan (the buffer pool IS the plan, preallocated at compile)
- [ ] equal-power pan per lane (constant-power law in signal-dsp), realized in the schedule
- [ ] mute/solo realized at compile (pulse decides audibility; plan carries resolved gains) with declick via the existing gain smoothing
- [ ] sends as extra schedule edges (post-fader tap + gain) feeding bus nodes
- [ ] channel count carried per edge; kill the channels.min(2) stereo bake-in
- [ ] pulse compiles its console/bus/send model into the new plan vocabulary
- [ ] golden-file render test harness: plan in, hashed PCM out, gating every render-plane change from now on

## Execution Plan

### Batch 10.1 - Schedule And Scratch Buffers

- [ ] node schedule compile + plan-owned scratch buffers
- [ ] executor walks the schedule; soak stays zero-alloc

### Batch 10.2 - Pan, Mute/Solo, Sends

- [ ] equal-power pan law in signal-dsp + pan node
- [ ] sends/busses as edges; mute/solo resolved at compile

### Batch 10.3 - Pulse Cutover And Goldens

- [ ] pulse console model compiled into bus topology
- [ ] golden render tests; Aura mixer knobs audibly work

## Acceptance Criteria

- [ ] soak example zero-alloc with a bussed plan
- [ ] pan/mute/solo/send audible in Loophole with declick
- [ ] golden render suite green and wired into CI

## Next Task

g10.011 (stable identity) — addressing for state handoff and parameters.
