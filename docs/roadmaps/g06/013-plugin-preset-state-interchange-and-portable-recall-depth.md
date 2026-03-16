# 013 - Plugin Preset, ARA Context, State Interchange, And Portable Recall Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.011, g06.012
Vision tags: `PLUGINS`, `STATE`, `INTERCHANGE`

## Problem

Signal already owns plugin recall and delegated execution surfaces, but broader
adapter breadth and product portability both need a clearer reusable answer for
state interchange, preset-family meaning, ARA-capable clip/plugin context, and
portable recall constraints.

## Goals

- [ ] define portable plugin state and preset interchange semantics where
  they are actually reusable
- [ ] define the first reusable ARA-capable plugin context vocabulary where
  clip/region/document metadata must cross the Signal runtime boundary
- [ ] align recall ownership, exported state receipts, and cross-adapter
  fallback behavior
- [ ] support later product recall and migration work without host-local blobs
  becoming the only practical path

## Non-Goals

- [ ] no product-specific preset browser or tagging UX
- [ ] no promise of lossless interchange where adapters cannot support it
- [ ] no full product-local clip editor workflow or arrangement policy

## Execution Plan

### Batch 13.1 - Interchange And ARA Contract

- [x] define portable preset/state interchange vocabulary, ARA context
  descriptors, and fallback classes
- [x] record what remains adapter-private or non-portable explicitly

### Batch 13.2 - Runtime Recall Depth

- [x] deepen runtime-owned recall/export surfaces to carry the new interchange
  and ARA-context meaning
- [x] keep render/delegation/host-edge receipts aligned to the same state truth

### Batch 13.3 - Portability Proof

- [x] add focused proofs for portable versus non-portable recall outcomes and
  bounded ARA-context transfer across the widened adapter set

## Acceptance Criteria

- [ ] Signal has explicit portable plugin state/preset interchange meaning and
  the first bounded ARA-capable context contract
- [ ] products can observe portable versus non-portable recall outcomes clearly
- [ ] state portability does not reopen host-local ownership

## Risks And Mitigations

- Risk: portability claims overpromise across adapters.
- Mitigation: freeze supported, degraded, and unsupported interchange classes explicitly.
- Risk: ARA planning drifts into product clip-edit workflows before the runtime
  boundary is clear.
- Mitigation: keep this milestone on runtime-owned clip/region/document context
  descriptors and plugin capability meaning only.
- Risk: products keep using opaque blobs because the portable path is unclear.
- Mitigation: require typed runtime receipts for portability outcomes.

## Evidence Requirements

- [ ] log each meaningful preset/recall tranche
- [ ] run focused portability validation for runtime-owned recall/export surfaces
- [ ] record explicit deferred preset breadth that remains out of scope

## Batch 13.1 Outcome

Batch 13.1 freezes the first bounded preset-state, portable recall, and
ARA-capable context contract in
`docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md`.

The repo now has an explicit rule set for:

- separating runtime recall payload from later interchange payload and preset
  descriptor meaning
- classifying later preset/state outcomes as `Portable`, `Guarded`,
  `NativeOnly`, `ContextOnly`, or `Unsupported` instead of leaving portability
  implicit
- keeping ARA-capable planning bounded to document, source, and region context
  descriptors rather than product-local editor workflow semantics
- giving Batch 13.2 one fixed portability target before deeper runtime recall,
  export, and host-edge receipt work begins

## Batch 13.2 Outcome

Batch 13.2 turns the portability contract into a real runtime-owned receipt
layer instead of leaving it as prose over the older recall payloads.

The repo now has:

- explicit runtime-owned portability classification on `RuntimePluginRecallPayload`
  through `Portable`, `Guarded`, `NativeOnly`, `ContextOnly`, and `Unsupported`
  receipt meaning
- additive preset descriptor and bounded ARA document/source/region context
  carried through plugin recall snapshots, handoff snapshots, execution
  topology summaries, and supervisor export
- runtime-side setters that seed preset and ARA context into the same
  sandbox-owned recall path instead of forcing host-local augmentation
- focused runtime and stable host-edge proofs that the widened recall payload
  survives shared observation and `supervisor_report()` delivery without
  reopening host-owned portability taxonomy

## Batch 13.3 Outcome

Batch 13.3 closes the bounded recall portability consumer seam:

- downstream-style `signal-runtime` proofs now consume portable versus guarded,
  native-only, context-only, and unsupported recall outcomes directly through
  shared plugin-chain and recall-handoff surfaces
- both stable host edges now prove that `supervisor_report()` forwards the
  same runtime-owned preset descriptor and bounded ARA document/source/region
  context truth without host-local portability classes or adapter-private
  preset reconstruction
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.recall-portability-boundary` descriptor plus the repo-owned
  `effigy acceptance:recall-portability-boundary` task
- `g06.014` can now deepen device supervision, restart-state machine, and
  hardware fault-boundary work on top of one closed preset-state and portable
  recall baseline

## Next Task

Continue `g06.014` with Batch 14.1 by freezing the runtime-owned device
supervision, restart-state machine, exhaustion, and fault-boundary contract
before deeper hardware recovery depth begins.
