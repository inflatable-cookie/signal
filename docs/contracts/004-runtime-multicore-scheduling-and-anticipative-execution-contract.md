# 004 Runtime Multicore Scheduling And Anticipative Execution Contract

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable contract for how Signal runtime exposes multicore
scheduling and anticipative execution decisions, so later `g04` work can deepen
execution behavior without pushing scheduler interpretation back into
host-local policy.

## Authority hierarchy

The scheduler contract has one source of truth with narrower projections built
from it:

1. `RuntimeEngineBlockSnapshot` is the authoritative per-block execution truth.
   It owns:
   - planning-group and phase counts
   - lane counts and lane order
   - dispatch counts, boundaries, and dispatch order
   - anticipative eligibility counts
   - scheduler topology compatibility and issue reporting
   - prework queue, service-state, and service-pressure detail
2. `RuntimeSchedulerSnapshot` is the live lifecycle/control-state view for the
   scheduler. It answers whether the runtime is configured, ready, actively
   servicing anticipative work, degraded, or stopped.
3. `RuntimeSchedulerExportSummary` is the narrow consumer-facing digest derived
   from `RuntimeEngineBlockSnapshot`. It is the preferred report/export shape
   for automation that does not need the full block snapshot.
4. `RuntimeExecutionTopologySummary` provides the semantic topology context for
   why nodes land in specific planning groups, lanes, and routed aggregates. It
   is explanatory context for scheduler choices, not a replacement authority
   for current execution counts or dispatch state.

## Deterministic versus profile-varying behavior

### Deterministic under the same graph and runtime mode

The following should remain runtime-owned and deterministic when the active
graph projection, runtime configuration, and anticipative enablement do not
change:

- planning-group assignment derived from graph execution classes and
  anticipative eligibility
- phase order derived from those planning groups
- lane order derived from phase order
- topology compatibility and topology issue classification
- the rule that anticipative execution, when present, precedes realtime lanes
- the rule that realtime dispatch terminates the declared execution topology

### Allowed to vary by runtime profile or live operating state

The following may vary with runtime profile, forecast policy, or live degraded
state, but runtime must surface those choices explicitly through the typed
scheduler surfaces above:

- whether anticipative execution is enabled at all
- schedule stream availability and resulting multicore width
- dispatch counts and prepared-versus-realtime handoff counts
- prework queue depth, pending-target makeup, and service budgets
- prework service pressure, semantic policy, throttling, or yielding
- plugin-gate, transport-gate, recovery-overlap, or lingering-session effects
  on anticipative work

Hosts and tools may observe those changes, but they must not recompute them
from callback timing, thread layout, or local graph traversal.

## Canonical inspection surfaces

Consumers should read scheduler state in this order:

- use `RuntimeSchedulerSnapshot` when the question is about lifecycle state
  such as `Stopped`, `ReadyIdle`, `RealtimeOnly`, `Anticipative`, or
  `Degraded`
- use `RuntimeEngineBlockSnapshot` when the question is about the exact current
  execution decision for a processed block
- use `RuntimeSchedulerExportSummary` when the question is about stable
  report/export automation over the currently frozen scheduler digest
- use `RuntimeSchedulerTopologySummary` and `RuntimeExecutionTopologySummary`
  when the question is why runtime chose a given lane/dispatch shape or whether
  a host would need reinterpretation for missing topology ownership ids

`RuntimeSupervisorReport` and `RuntimeObservationReport` remain the shared
delivery surfaces that carry those scheduler receipts into export and host
observation paths.

## Host and consumer rule

Hosts, tools, and downstream consumers must not create a parallel scheduler
model from private runtime internals, callback-thread assumptions, or local
graph traversal when the typed runtime scheduler surfaces already expose the
needed information.

If a later consumer needs scheduler detail that is not present in the typed
runtime-owned surfaces above, that detail should be promoted into
`signal-runtime` and exported through the same report path rather than inferred
in host-local code.

## Current proof boundary

The current contract is grounded in focused runtime proofs that already assert:

- topology compatibility and issue surfacing through scheduler export
- anticipative versus realtime lane and dispatch reporting
- compatible schedule-stream width can widen anticipative service budget under
  normal pressure without widening host-local policy ownership
- compatible schedule-stream width can also widen requested anticipative
  service cadence, while elevated pressure, plugin gates, and transport gates
  still clamp or yield that widened request through the same runtime-owned
  receipts
- schedule projection and running forecast-plan refresh/rebuild paths must reuse
  the same widened runtime-owned service policy rather than falling back to a
  separate host-local or single-cycle refresh model
- pressure-, plugin-, and transport-gated prework service state
- restart, reconfigure, and mixed execution-class graph transitions preserve the
  same schedule-width policy and keep scheduler receipts coherent across those
  lifecycle changes
- focused stress fixtures cover mixed execution-class churn, invalidation-heavy
  transition bursts, and constrained anticipative windows so the current
  contract is pinned beyond one-off happy paths

Batch 2.1 freezes the interpretation of those receipts. Later `g04.002` work
may deepen multicore execution itself, but it should do so by extending the
same runtime-owned scheduler contract rather than replacing it.

The deferred risks after `g04.002` are explicit: schedule-stream width is still
only a bounded proxy for multicore capacity, there is still no true cost-aware
or work-stealing dispatcher, and long-duration threshold/fail-gate benchmark
policy remains a later regression concern rather than part of this contract.

## Next Task

Open `g04.003` with Batch 3.1 and define the runtime-owned deferred-work
contract for render finalization, analysis jobs, delegated merge work, and
report/materialization services on top of the closed scheduler substrate.
