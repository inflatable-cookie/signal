# 020 - Generation Closeout And Loophole Feature-Readiness Gate

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.019
Vision tags: `CLOSEOUT`, `READINESS`, `LOOPHOLE`

## Problem

`g07` should end with a clear statement of which feature fronts are now strong
enough for Loophole to rely on and which remain intentionally deferred.

## Goals

- [ ] define the generation-closeout and feature-readiness gate for `g07`
- [ ] identify what later Loophole work may now assume safely
- [ ] record any intentionally deferred feature fronts without leaving ambiguity

## Non-Goals

- [ ] no new feature expansion here
- [ ] no product-launch review detached from runtime evidence

## Execution Plan

### Batch 20.1 - Closeout Scope

- [x] map the final evidence needed across routing, Linux, control, and stretch work
- [x] identify which downstream assumptions are now safe

### Batch 20.2 - Readiness Gate

- [x] define the feature-readiness gate from the completed generation evidence
- [x] keep downstream assumptions tied to concrete runtime receipts

### Batch 20.3 - Closeout Output

- [x] log the generation closeout and name the next deferred or active queue cleanly

## Acceptance Criteria

- [x] `g07` closes with explicit feature-readiness signals for Loophole
- [x] deferred work is named cleanly instead of left ambiguous
- [x] later planning can build on the closeout without rediscovering feature scope

## Risks And Mitigations

- Risk: closeout claims outrun the actual acceptance evidence.
- Mitigation: bind all readiness claims to the focused `g07.019` outputs.

## Evidence Requirements

- [x] log the closeout milestone explicitly
- [x] run the final closeout validation needed for generation status
- [x] record the next queue or backlog posture clearly

## Batch 20.1 Outcome

Batch 20.1 freezes the closeout policy in
`docs/contracts/051-generation-closeout-and-loophole-feature-readiness-gate-contract.md`.
That contract locks the authority chain for the final `g07` closeout gate,
Loophole-facing feature-readiness meaning, and explicit deferred posture on top
of the now-closed `g07.019` integrated acceptance lane instead of leaving the
generation finish line as an open-ended review.

It also makes the closeout posture explicit:

- the integrated `g07` acceptance lane is the required fast-path base of the
  final gate
- one machine-readable closeout descriptor and Effigy gate task will be
  required once implemented
- broader rerun confidence and the already-advisory `complex-io`/`lv2` breadth
  stay advisory rather than quietly becoming blockers
- richer Linux live backend ownership, immersive render depth, and
  product-local controller or preview workflows remain deferred

That gives Batch 20.2 one fixed target for the actual closeout descriptor and
repo-owned gate task instead of reopening closeout policy while wiring the
final surface.

## Batch 20.2 Outcome

Batch 20.2 turns that frozen policy into a runnable shared surface. The
machine-readable `g07` closeout descriptor now lives in
`crates/signal-supervisor-tools/src/main.rs`, and Effigy now owns the matching
repo gate as `acceptance:g07-closeout` in `effigy.toml`.

This batch intentionally stops short of the final generation verdict. The
descriptor now reports:

- `g07`-specific contract and roadmap anchors instead of the older `g06`
  promotion record
- the required grouped acceptance base from `g07.019`
- a runnable closeout-gate status and provisional Loophole-facing readiness
  areas tied back to the closed routing, Linux, controller, and stretch seams
- explicit residual risk and next-queue posture that keeps the final
  promotion-or-backlog decision reserved for Batch 20.3

That gives Batch 20.3 one repo-owned closeout gate and one typed readiness
record to review, rather than forcing the final verdict to invent its own
surface or collapse back into prose-only judgment.

## Batch 20.3 Outcome

Batch 20.3 closes `g07` with an explicit reusable-substrate verdict. The
machine-readable generation-closeout descriptor now records a real promotion
decision: `g07` is strong enough on routing, Linux breadth, controller or
advanced-hardware substrate, and sample-domain stretch or preview services to
promote `g08` as the next active generation.

The verdict is intentionally narrow and reusable:

- it is a Signal substrate verdict, not a Loophole product-launch verdict
- the grouped `g07` acceptance lane and the repo-owned `g07` closeout gate are
  now sufficient to stop treating bounded `g07` feature-expansion depth as a
  blocker for the next queue
- richer Linux live backend ownership, immersive routing, vendor-protocol
  hardware depth, and product-local preview workflows stay explicit `g08`
  scope instead of silently blocking closeout

That means `g07` now closes cleanly, `g08` becomes the single active queue,
and the remaining deferred fronts move forward as a named next generation
instead of a vague backlog tail.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
