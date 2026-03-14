# 003 - Plugin Isolation Policy, Transport Rebind, And Shared-Sandbox Continuity Depth

Status: active
Owner: core-product
Created: 2026-03-13
Depends on: g06.002
Vision tags: `PLUGINS`, `RECOVERY`, `SANDBOX`

## Problem

Signal's current plugin path is CLAP-first and runtime-owned, but products like
Loophole still need stronger reusable truth for how several hosted instances
share one sandbox boundary, how placement and isolation policy chooses those
boundaries, what rebind means after interruption, and when continuity must fail
explicitly.

## Goals

- [ ] deepen runtime-owned rebind and continuity semantics for shared plugin
  sandbox boundaries
- [ ] define a flexible runtime-owned placement and isolation policy surface
  that can support:
  - all isolated by default except explicit verified allow rules
  - all in-process by default except explicit deny rules
  - isolation by plugin format, vendor, capability, or other reusable filters
  - later policy presets without reopening the shared contract
- [ ] make multi-instance plugin transport recovery explicit instead of
  host-reconstructed
- [ ] align plugin lifecycle, fault, and recall state through one reusable path

## Non-Goals

- [ ] no product-specific plugin library or preset UX
- [ ] no new backend breadth yet beyond the existing adapter path

## Execution Plan

### Batch 3.1 - Shared-Sandbox Recovery Contract

- [ ] define placement rule vocabulary, sandbox grouping keys, and isolation
  outcome receipts before adapter breadth widens further
- [ ] define rebind, shared-boundary degradation, and terminal plugin outcomes
- [ ] document multi-instance continuity semantics at the runtime boundary

### Batch 3.2 - Runtime Rebind Depth

- [ ] add runtime-owned policy evaluation and sandbox-assignment meaning for
  in-process, shared-sandbox, and stricter isolated placement
- [ ] deepen transport-session and sandbox receipts for shared-instance recovery
- [ ] keep lifecycle and fault surfaces aligned with the new continuity meaning

### Batch 3.3 - Multi-Instance Proof

- [ ] add focused proofs for shared-sandbox degradation, recovery, and terminal
  failure across several plugin instances
- [ ] add focused proofs for allowlist, denylist, and by-format placement
  behavior without host-local rule reconstruction

## Acceptance Criteria

- [ ] Signal has an explicit flexible plugin isolation and placement policy
  contract
- [ ] Signal has explicit multi-instance sandbox continuity semantics
- [ ] products can observe plugin recovery truth without local reconstruction
- [ ] later CLAP, VST3, and AU depth can reuse one placement-policy surface
- [ ] later adapter breadth can build on one runtime-owned recovery model

## Risks And Mitigations

- Risk: isolation policy becomes product-local rule sprawl.
- Mitigation: freeze one reusable rule vocabulary and outcome receipt shape in
  Signal before adding product presets.
- Risk: multi-instance semantics stay CLAP-harness-specific.
- Mitigation: freeze format-neutral continuity meaning before adapter widening.
- Risk: host-local product logic keeps owning rebind semantics.
- Mitigation: require runtime/export receipts to carry the key outcomes directly.

## Evidence Requirements

- [ ] log each meaningful plugin-continuity tranche
- [ ] run focused multi-instance recovery and placement-policy validation
- [ ] record any deferred adapter-specific behavior explicitly

## Next Task

Continue `g06.003` with Batch 3.1 by defining placement-rule vocabulary,
sandbox grouping keys, and shared rebind or terminal continuity semantics
before deeper runtime policy evaluation and multi-instance proof work.
