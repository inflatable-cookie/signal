# 002 - SharedSandbox Tier

Status: deferred
Owner: core-product
Created: 2026-08-17
Depends on: g11.001
Vision tags: `PLUGINS`, `SANDBOX`, `RECOVERY`
Governing refs: `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/roadmaps/g11/001-production-host-assembly-wiring.md`

## Problem

Signal models three isolation tiers in `PluginIsolationTier`:

- `InProcess`
- `DedicatedSandbox` — shipped; one plugin per broker child
- `SharedSandbox` — modeled in runtime receipts and placement vocabulary, but
  bridge/host code rejects it in v1

Products that want several compatible plugin instances under one sandbox
boundary still pay one child process per plugin today. That is correct for crash
isolation, but expensive when runtime placement policy selects shared boundaries.

## Research posture

**No separate research lane is required.**

SharedSandbox semantics are already frozen in Contract `014`:

- placement rules and grouping keys
- shared-boundary blast radius
- rebind and terminal outcomes
- runtime-owned receipts in `signal-runtime`

What remains is **implementation and proof**, not a new strategic question. Treat
this milestone like adapter breadth work: extend the existing broker + bridge
stack under Contract `014`, do not open a speculative study program.

Optional Batch 2.0 (docs-only, not ready until product pull):

- map broker multiplexing on the existing sandbox protocol
- name memory/CPU tradeoffs vs DedicatedSandbox
- list the runtime receipts that must gain member-instance proof

That design note belongs in this roadmap if needed; it is not a research
generation.

## Goals

- [ ] implement SharedSandbox in `signal-plugin-sandbox` broker multiplexing
- [ ] add a bridge backend tier that preserves Contract `014` grouping semantics
- [ ] prove multi-instance continuity and terminal blast radius through runtime
  receipts and focused tests
- [ ] integrate SharedSandbox selection through the production host assembly
  opened by `g11.001`

## Non-Goals

- [ ] changing Contract `014` isolation vocabulary
- [ ] product browser or trust UX
- [ ] replacing DedicatedSandbox as the default isolation tier
- [ ] vendor certification matrices

## Execution Plan

### Batch 2.0 - Multiplexing Design Note (optional, docs-only)

Status: not ready

Promotion trigger: Loophole or another consumer names SharedSandbox as a
blocking dependency **and** `g11.001` is complete.

- [ ] document broker multiplexing design against Contract `014`
- [ ] name proof surfaces and stop conditions
- [ ] confirm no contract gap requires a new research brief

### Batch 2.1 - Broker Multiplexing Implementation

Status: blocked on Batch 2.0 or explicit operator waiver

- [ ] extend sandbox broker to host multiple plugin instances in one child
- [ ] preserve crash attribution and typed recovery receipts per Contract `014`

### Batch 2.2 - Bridge Tier And Host Assembly Integration

Status: blocked on Batch 2.1

- [ ] add SharedSandbox backend beside `ShmPluginProcessor`
- [ ] route placement outcomes from runtime through `g11.001` host assembly

### Batch 2.3 - Continuity Proof And Closeout

Status: blocked on Batch 2.2

- [ ] prove shared-boundary degradation and terminal outcomes on runtime receipts
- [ ] close milestone and update Contract `072` remaining-gaps table if needed

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

- [ ] one log per completed batch
- [ ] continuity proof references Contract `014` rules explicitly
- [ ] milestone closeout updates `docs/roadmaps/g11/README.md`

## Next Task

Do not open this milestone until `g11.001` closes. When product pull exists,
start with Batch 2.0 unless the operator explicitly waives the design note and
names a bounded implementation scope.
