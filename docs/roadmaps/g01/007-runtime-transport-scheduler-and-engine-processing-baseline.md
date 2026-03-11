# Roadmap g01.007: Runtime Transport, Scheduler, and Engine Processing Baseline

Status: queued
Owner: core-product
Created: 2026-03-10
Depends on: g01.006
Vision tags: RT, RES, ENG
Target envelope: harden `signal-runtime` into the authoritative engine-control
layer for transport truth, scheduler policy, block execution, and shared
diagnostics across embedded and hosted Signal deployments, while preserving the
node-oriented graph topology that later console and track-lane work depends on.

## Problem

Signal already has meaningful runtime work in flight, but the next dedicated
engine thread needs an explicit milestone that ties graph execution, transport,
and scheduler behavior into one coherent runtime contract. Without that:

1. transport semantics risk drifting between host wrappers and runtime,
2. prework/scheduler features can outpace the guarantees of the actual engine
   block path,
3. host diagnostics will describe engine behavior that runtime does not yet
   own rigorously enough,
4. scheduler and execution policy may accidentally flatten node-oriented mixer
   structures into opaque host behavior.

## Goals

- make runtime the explicit authority for transport progression and block clock
- tighten the realtime/prework scheduler state machine around real engine work
- expose transport, scheduler, and engine diagnostics through shared runtime
  observation surfaces
- prove that recovery, restart, seek, and loop transitions leave engine state
  coherent
- ensure runtime executes graph plans in a way that remains compatible with
  future console-node, track-lane, and lane-to-bus mixer topology

## Non-Goals

- connecting to real audio hardware in this batch
- building the full plugin-host processing path in this batch
- implementing product-level editing or arrangement authority

## Execution Plan

### 007.1 Transport truth and engine clock

- [ ] define the runtime-owned block clock and transport progression rules for
      play, stop, seek, and loop transitions
- [ ] make transport epoch changes explicit invalidation boundaries for queued
      prework and cached engine state
- [ ] extend engine block context/reporting so transport state is visible at the
      exact point processing occurs
- [ ] keep transport progression attached to node-oriented graph execution
      context so later lane and console work does not need a second timing model

### 007.2 Scheduler enforcement on real engine work

- [ ] ensure the prework service lane and realtime lane are exercised against
      real engine blocks rather than only scheduler metadata
- [ ] tighten scheduler-state transitions, backlog classes, and pressure policy
      around observed engine and recovery conditions
- [ ] prove that reconfigure, restart, recovery, and role/profile changes leave
      the scheduler and engine in a consistent state
- [ ] validate that scheduler phases and lane ordering can represent future
      track-lane and console-node execution groups without host-specific
      reinterpretation

### 007.3 Diagnostics and supervisor surfaces

- [ ] expose transport, block, scheduler, and degradation state through shared
      runtime snapshots and supervisor exports
- [ ] add tests that cover seek, loop wrap, restart, degraded recovery, and
      prework invalidation against real engine-processing paths
- [ ] document any remaining host-only behavior that still needs to be promoted
      into runtime authority before host/device work deepens
- [ ] expose enough node/lane execution detail that later mixer-topology
      debugging does not require host-local reconstruction

## Acceptance Signals

1. Runtime is the unambiguous source of truth for transport progression and
   scheduler state during engine block execution.
2. Recovery, restart, and transport jumps are covered by tests that validate
   engine/scheduler coherence rather than only lifecycle control flow.
3. Shared diagnostics surfaces are detailed enough that host assemblies can
   report engine behavior without recomputing it locally.

## Risks and Mitigations

- Risk: scheduler work keeps growing as an isolated subsystem without enough
  proof on the actual engine path.
- Mitigation: require engine-block-backed tests and diagnostics for every major
  scheduler transition added in this milestone.
- Risk: transport policy gets partially reintroduced in host code.
- Mitigation: keep host work restricted to projecting runtime-owned state,
  while runtime remains authoritative for progression and invalidation.

## Evidence Requirements

- [ ] every transport/scheduler tranche logged under `docs/logs/YYYY-MM/`
- [ ] validation evidence must include runtime-focused tests, not only host
      smoke runs
- [ ] closure log must state which scheduler behaviors remain deliberately
      deferred to later milestones

## Next Task

Open `g01.008` once runtime owns the engine-processing and transport/scheduler
truth cleanly enough to attach real device-backed host execution and runtime
diagnostics without reshaping core authority boundaries.
