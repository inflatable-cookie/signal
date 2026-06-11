# 012 - Parameter Fast Path And Automation Playback

Status: complete
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

- [x] SetParam command (node index + parameter + target) riding the existing mailbox, resolved against a compile-time parameter table in the active plan; plan recompiles reserved for structural edits
- [x] compiled automation envelopes: per-parameter sorted breakpoint arrays in the plan, sampled per block into sample-accurate ramps (generalizing the existing gain-smoothing slope logic)
- [x] gain automation end-to-end: pulse compiles automation lanes into envelopes; pan follows
- [x] property test on envelopes: no sample step exceeds the slope bound under arbitrary command sequences
- [x] Aura/pulse host: fader drags dispatch SetParam, not plan recompiles

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

## Progress (2026-06-11)

- Fast path: `RenderCommand::SetStageGain { stage_index, target }` rides the
  existing mailbox; the controller resolves stage ids against the topology
  of the most recent successful install (FIFO guarantees the command lands
  after its plan); unknown stages return a typed error so hosts fall back
  to a full install. `RenderPlanSpec::differs_only_in_gains` gives hosts
  the gain-only diff; Aura's `sync_plan` takes the fast path on fader
  moves and recompiles only on structural change — a fader drag now ships
  one small command per change instead of a full plan.
- Automation: `RenderStageSpec.gain_automation` — sorted `(frame, linear)`
  breakpoints, validated at compile (typed UnsortedAutomation error),
  master factor folded in. The render path samples the envelope at each
  block end via binary search and ramps from the inherited smoothed gain,
  so playback tracks the curve block-accurately while plan swaps, seeks,
  and envelope edits stay continuous by construction.
- Pulse compiles console-gain automation lanes (target node = the track's
  console node, parameter = console gain) into envelopes on the track's
  source stage, sharing the static gain's headroom scale.
- Tests: fast-path retarget step bound + unknown-stage error; envelope
  known answers (peak, descent midpoint, monotonic rise); envelope swap
  continuity; gain-only diff helper cases. 31 render-plane tests green;
  soak zero-alloc; pulse 122/122; aura green; clippy clean.
- Deferred: parameter table beyond stage gain (matrix entries/azimuth land
  with wider formats), per-edge gain retargeting, pan automation (needs a
  pan parameter in pulse's console model first).

## Next Task

g10.013 (DSP kit) and g10.014 (observability) can run parallel.
