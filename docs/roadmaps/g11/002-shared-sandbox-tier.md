# 002 - SharedSandbox Tier

Status: active
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
- `SharedSandbox` — modeled in runtime receipts and placement vocabulary, but
  broker/host code still rejects extra instances in one child

Products that want several compatible plugin instances under one sandbox
boundary still pay one child process per plugin today. That is correct for crash
isolation, but expensive when runtime placement policy selects shared boundaries.

## Research posture

**No separate research lane is required.** Contract `014` owns semantics.
Batch 2.0 froze the multiplexing map at
`docs/architecture/shared-sandbox-multiplexing.md`. Remaining work is
implementation and proof.

## Goals

- [ ] implement SharedSandbox in `signal-plugin-sandbox` broker multiplexing
- [ ] reuse `ShmPluginProcessor` per member lease (no new audio-thread backend)
- [ ] prove multi-instance continuity and terminal blast radius through runtime
  receipts and focused tests
- [ ] integrate SharedSandbox selection through the `g11.001` host assembly

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

Status: ready

- [ ] extend sandbox broker to host multiple plugin instances in one child
- [ ] keep DedicatedSandbox single-slot commands unchanged
- [ ] preserve crash attribution per Contract `014`

Card: `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.

### Batch 2.2 - Host Assembly Integration

Status: blocked on Batch 2.1

- [ ] route `PluginIsolationTier::SharedSandbox` through the `g11.001` factory
- [ ] attach `ShmPluginProcessor` from each member lease
- [ ] record runtime grouping key and member count

### Batch 2.3 - Continuity Proof And Closeout

Status: blocked on Batch 2.2

- [ ] prove shared-boundary degradation and terminal outcomes on runtime receipts
- [ ] close milestone and update Contract `072` remaining-gaps table

## Acceptance Criteria

- [ ] runtime placement can select SharedSandbox without host-local heuristics
- [ ] one shared boundary failure is explainable through runtime-owned receipts
  for all member instances
- [ ] DedicatedSandbox behavior remains unchanged for existing paths
- [ ] docs no longer describe SharedSandbox as "unimplemented" without pointing
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
- [ ] one log per remaining completed batch
- [ ] continuity proof references Contract `014` rules explicitly
- [ ] milestone closeout updates `docs/roadmaps/g11/README.md`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
