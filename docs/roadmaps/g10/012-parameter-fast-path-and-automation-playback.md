# 012 - Parameter Fast Path And Automation Playback

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.011
Vision tags: `ENGINE-SPINE`, `AUTOMATION`

## Problem

Every gain tweak recompiles the whole plan — clips, Kaiser tables — and
ships it through the mailbox. Correct for topology, wrong for continuous
gestures: a 60 Hz fader drag at hundreds of clips is a scaling cliff. And
pulse's automation lanes (real, modeled, UI-visible) reach no audio: the
engine has static per-lane gains only.

## Goals

- [ ] SetParam command (node index + parameter + target) riding the existing mailbox, resolved against a compile-time parameter table in the active plan; plan recompiles reserved for structural edits
- [ ] compiled automation envelopes: per-parameter sorted breakpoint arrays in the plan, sampled per block into sample-accurate ramps (generalizing the existing gain-smoothing slope logic)
- [ ] gain automation end-to-end: pulse compiles automation lanes into envelopes; pan follows
- [ ] property test on envelopes: no sample step exceeds the slope bound under arbitrary command sequences
- [ ] Aura/pulse host: fader drags dispatch SetParam, not plan recompiles

## Execution Plan

### Batch 12.1 - SetParam

- [ ] parameter table in compiled plan; mailbox command; host wiring

### Batch 12.2 - Automation Envelopes

- [ ] breakpoint arrays + per-block sampling
- [ ] pulse compiles automation lanes; property tests

## Acceptance Criteria

- [ ] fader drag produces zero plan recompiles
- [ ] automation lane audibly modulates gain sample-accurately
- [ ] envelope property tests green

## Next Task

g10.013 (DSP kit) and g10.014 (observability) can run parallel.
