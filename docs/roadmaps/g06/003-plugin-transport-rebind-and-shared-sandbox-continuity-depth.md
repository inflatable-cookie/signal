# 003 - Plugin Isolation Policy, Transport Rebind, And Shared-Sandbox Continuity Depth

Status: complete
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

- [x] define placement rule vocabulary, sandbox grouping keys, and isolation
  outcome receipts before adapter breadth widens further
- [x] define rebind, shared-boundary degradation, and terminal plugin outcomes
- [x] document multi-instance continuity semantics at the runtime boundary

## Progress Notes

- 2026-03-14: completed Batch 3.1 by freezing contract `014`, defining one
  shared runtime-owned vocabulary for placement rules, placement policy,
  sandbox grouping keys, isolation outcomes, shared-sandbox boundaries, rebind,
  shared-boundary degradation, and terminal sandbox outcomes. The contract also
  froze the first multi-instance continuity rules so later runtime receipts and
  host-edge export extend one runtime-owned blast-radius model instead of
  host-local grouping heuristics.

### Batch 3.2 - Runtime Rebind Depth

- [x] add runtime-owned policy evaluation and sandbox-assignment meaning for
  in-process, shared-sandbox, and stricter isolated placement
- [x] deepen transport-session and sandbox receipts for shared-instance recovery
- [x] keep lifecycle and fault surfaces aligned with the new continuity meaning

- 2026-03-14: completed Batch 3.2 by adding a runtime-owned
  `RuntimePluginPlacementPolicy` shell, widening plugin lifecycle and chain
  snapshots with placement outcome, grouping key, matched rule, shared-boundary
  member count, continuity class, and rebindability, and exporting
  `plugin_lifecycle_snapshot` through observation or supervisor JSON so shared
  sandbox restartable versus terminal truth no longer requires host-local
  reconstruction.

### Batch 3.3 - Multi-Instance Proof

- [x] add focused proofs for shared-sandbox degradation, recovery, and terminal
  failure across several plugin instances
- [x] add focused proofs for allowlist, denylist, and by-format placement
  behavior without host-local rule reconstruction

- 2026-03-14: completed Batch 3.3 by proving shared-boundary blast radius,
  recovery, and terminal continuity across several plugin instances, adding
  allowlist, denylist, and by-format policy proofs on runtime-owned lifecycle
  and chain receipts, and promoting the consumer-facing boundary into the
  machine-readable `signal.runtime.plugin-continuity-boundary` descriptor plus
  the repo-owned `effigy acceptance:plugin-continuity` task.

## Acceptance Criteria

- [x] Signal has an explicit flexible plugin isolation and placement policy
  contract
- [x] Signal has explicit multi-instance sandbox continuity semantics
- [x] products can observe plugin recovery truth without local reconstruction
- [x] later CLAP, VST3, and AU depth can reuse one placement-policy surface
- [x] later adapter breadth can build on one runtime-owned recovery model

## Risks And Mitigations

- Risk: isolation policy becomes product-local rule sprawl.
- Mitigation: freeze one reusable rule vocabulary and outcome receipt shape in
  Signal before adding product presets.
- Risk: multi-instance semantics stay CLAP-harness-specific.
- Mitigation: freeze format-neutral continuity meaning before adapter widening.
- Risk: host-local product logic keeps owning rebind semantics.
- Mitigation: require runtime/export receipts to carry the key outcomes directly.

## Evidence Requirements

- [x] log each meaningful plugin-continuity tranche
- [x] run focused multi-instance recovery and placement-policy validation
- [x] record any deferred adapter-specific behavior explicitly

## Next Task

Continue `g06.004` with Batch 4.1 by freezing resumable, restartable,
recoverable, and terminal offline-render session outcomes, then align render
checkpoint survival and interruption meaning with the shared `g06.001`
taxonomy before runtime session-depth work begins.
