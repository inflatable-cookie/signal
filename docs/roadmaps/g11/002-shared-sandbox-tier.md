# 002 - SharedSandbox Tier

Status: complete
Owner: core-product
Created: 2026-08-17
Updated: 2026-08-17
Depends on: g11.001
Vision tags: `PLUGINS`, `SANDBOX`, `RECOVERY`
Governing refs: `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/architecture/shared-sandbox-multiplexing.md`, `docs/architecture/production-host-assembly-integration.md`

## Problem

Signal models three isolation tiers in `PluginIsolationTier`:

- `InProcess`
- `DedicatedSandbox` — shipped; one plugin per broker child
- `SharedSandbox` — one broker child, many plugin instances that share a
  grouping key

Products that want several compatible plugin instances under one sandbox
boundary no longer pay one child process per plugin when runtime placement
selects SharedSandbox.

## Research posture

**No separate research lane is required.** Contract `014` owns semantics.
Batch 2.0 froze the multiplexing map at
`docs/architecture/shared-sandbox-multiplexing.md`. Implementation and proof
landed in Batches 2.1–2.3.

## Goals

- [x] implement SharedSandbox in `signal-plugin-sandbox` broker multiplexing
- [x] reuse `ShmPluginProcessor` per member lease (no new audio-thread backend)
- [x] prove multi-instance continuity and terminal blast radius through runtime
  receipts and focused tests
- [x] integrate SharedSandbox selection through the `g11.001` host assembly

## Non-Goals

- [x] changing Contract `014` isolation vocabulary
- [x] product browser or trust UX
- [x] replacing DedicatedSandbox as the default isolation tier
- [x] vendor certification matrices
- [x] vendor/format grouping (v1 grouping is plugin identity only)

## Execution Plan

### Batch 2.0 - Multiplexing Design Note (docs-only)

Status: complete

Product pull: operator, 2026-08-17. Grouping: same plugin type identity.

- [x] document broker multiplexing against Contract `014`
- [x] name proof surfaces and stop conditions
- [x] confirm no contract gap requires a new research brief

Design note: `docs/architecture/shared-sandbox-multiplexing.md`.

### Batch 2.1 - Broker Multiplexing Implementation

Status: complete

- [x] extend sandbox broker to host multiple plugin instances in one child
- [x] keep DedicatedSandbox single-slot commands unchanged
- [x] preserve crash attribution per Contract `014`

Card: `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.

### Batch 2.2 - Host Assembly Integration

Status: complete

- [x] route `PluginIsolationTier::SharedSandbox` through the `g11.001` factory
- [x] attach `ShmPluginProcessor` from each member lease
- [x] record runtime grouping key and member count

Card: `docs/roadmaps/g11/batch-cards/006-g11-002-host-assembly-integration.md`.

### Batch 2.3 - Continuity Proof And Closeout

Status: complete

- [x] prove shared-boundary degradation and terminal outcomes on runtime receipts
- [x] close milestone and update Contract `072` remaining-gaps table

Card: `docs/roadmaps/g11/batch-cards/007-g11-002-continuity-proof-and-closeout.md`.

## Acceptance Criteria

- [x] runtime placement can select SharedSandbox without host-local heuristics
- [x] one shared boundary failure is explainable through runtime-owned receipts
  for all member instances
- [x] DedicatedSandbox behavior remains unchanged for existing paths
- [x] docs no longer describe SharedSandbox as "unimplemented" without pointing
  at this milestone and Contract `014`

## Risks and Mitigations

- Risk: shared boundary hides per-plugin crash isolation.
- Mitigation: keep DedicatedSandbox as default; SharedSandbox only via explicit
  runtime placement policy.

- Risk: broker multiplexing reintroduces synthetic lifecycle behavior.
- Mitigation: require the same real `process()` and transport proof bar as
  DedicatedSandbox.

## Evidence Requirements

- [x] Batch 2.0 log
- [x] one log per remaining completed batch
- [x] continuity proof references Contract `014` rules explicitly
- [x] milestone closeout updates `docs/roadmaps/g11/README.md`

## Next Task

Stop for operator review of the `g11.002` PR. Do not start a follow-on
generation from this milestone.
