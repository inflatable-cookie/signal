# 020 - Long-Session Soak, Promotion Gate, And Loophole-Readiness Closeout

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.019
Vision tags: `ACCEPTANCE`, `SOAK`, `CLOSEOUT`

## Problem

`g06` will only be worth the planning cost if it closes with stronger
long-session evidence and a clear answer for whether the widened Signal runtime
actually moved Loophole forward on both hardening and feature breadth.

## Goals

- [x] define the final `g06` soak and promotion gate
- [x] combine runtime recovery, profiling, plugin breadth, hardware, and media
  evidence into one closeout surface
- [x] make Loophole-facing readiness explicit rather than implicit

## Non-Goals

- [ ] no product launch-readiness review outside Signal's reusable boundary
- [ ] no remote/distributed profile generation closeout yet

## Execution Plan

### Batch 20.1 - Soak And Promotion Scope

- [x] define the bounded long-session soak expectations and promotion criteria
- [x] decide which `g06` evidence is required, advisory, or deferred

### Batch 20.2 - Closeout Surface

- [x] implement the combined `g06` closeout descriptor, task, and receipts
- [x] keep the outputs machine-readable and downstream-consumable

### Batch 20.3 - Readiness Review

- [x] review the generation against Loophole-facing runtime and feature-depth needs
- [x] record the next backlog or generation handoff clearly

## Acceptance Criteria

- [x] `g06` has bounded long-session soak evidence
- [x] the widened runtime and feature surface is summarized through one closeout gate
- [x] Loophole-facing readiness is explicit enough to guide the next Signal generation

## Risks And Mitigations

- Risk: closeout becomes a vague summary instead of a gate.
- Mitigation: require typed receipts and required-versus-advisory policy.
- Risk: generation claims outrun actual reusable evidence.
- Mitigation: tie the final gate to the integrated acceptance and soak receipts only.

## Evidence Requirements

- [x] log each meaningful closeout tranche
- [x] run the final closeout and soak validation tasks actually used for promotion
- [x] record the next backlog or generation handoff explicitly

## Batch 20.1 Outcome

Batch 20.1 freezes the closeout policy in
`docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md`.
That contract locks the authority chain for bounded soak, promotion-gate, and
Loophole-facing readiness on top of the now-closed `g06.019` integrated
acceptance lane, rather than letting the generation end as a prose-only review.

It also makes the closeout posture explicit:

- the integrated acceptance lane is the required fast-path base of the final
  gate
- one bounded long-session soak lane and one machine-readable closeout
  descriptor will be required once implemented
- broader reruns and extra confidence passes stay advisory
- unstable server-host recovery-overlap depth, remote soak orchestration, and
  product launch readiness remain deferred

That gives Batch 20.2 one fixed target for the actual closeout descriptor,
bounded soak lane, and Effigy gate task instead of reopening closeout policy
while wiring the final surface.

## Batch 20.2 Outcome

Batch 20.2 turns that frozen policy into a runnable `g06` closeout surface.
`signal-supervisor-tools` now exposes both a machine-readable
`signal.g06.long-session-soak-lane` descriptor and an updated `g06`
generation-closeout descriptor, while Effigy now owns:

- `effigy acceptance:g06-soak-lane`
- `effigy acceptance:g06-closeout`

The soak lane keeps the bounded policy explicit instead of letting long-session
evidence sprawl: local `soak` and local `mixed` are now required, the
integrated acceptance lane remains visible as advisory context, and the broader
`server soak` path stays explicitly deferred because the existing recovery-
overlap attach-limit issue is still not stable enough for the closeout gate.

The updated closeout descriptor also stops pretending `g06` is just a renamed
`g05` release gate. It now points at the actual `g06` authority chain:
integrated acceptance, bounded soak, validation, residual deferred scope, and
the still-pending Loophole-facing readiness review that belongs to Batch 20.3.
That gives the generation one repo-owned gate surface instead of a policy-only
document and a stale closeout descriptor.

## Batch 20.3 Outcome

Batch 20.3 closes `g06` with an explicit readiness verdict instead of leaving
the generation in a permanent review state. The machine-readable generation
closeout descriptor now records a real promotion decision: `g06` is strong
enough on runtime hardening, adapter breadth, hardware and external-I/O
substrate, and media plus analysis-service substrate to promote `g07` as the
next active generation.

The verdict is intentionally narrow and reusable:

- it is a Signal substrate verdict, not a Loophole product-launch verdict
- the bounded integrated lane and bounded soak lane are now sufficient to stop
  treating `g06` hardening as a blocker for the next feature-forward queue
- deferred unstable `server soak` depth and broader advisory rerun confidence
  remain explicit residual scope instead of silently blocking `g07`

That means `g06` now closes cleanly, `g07` becomes the single active queue, and
the remaining unstable soak or deeper confidence work stays visible as deferred
scope rather than muddying the next generation boundary.

## Next Task

Continue `g07.001` with Batch 1.1 by freezing the canonical multichannel
layout and channel-role contract before widening sidechain, spatial, Linux, or
time-stretch implementation depth.
