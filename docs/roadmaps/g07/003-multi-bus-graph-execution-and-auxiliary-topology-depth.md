# 003 - Multi-Bus Graph Execution And Auxiliary Topology Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.001, g07.002
Vision tags: `ROUTING`, `GRAPH`, `EXECUTION`

## Problem

Signal's current graph work is credible for baseline buses and sends, but the
next Loophole feature wave needs stronger reusable truth for auxiliary buses,
multiple bus edges, and richer routing topology.

## Goals

- [ ] define runtime-owned multi-bus and auxiliary topology semantics
- [ ] support richer send, return, and bus execution without host-local patches
- [ ] keep graph, render, and diagnostics surfaces aligned on one topology model

## Non-Goals

- [ ] no product-specific console workflow work
- [ ] no final distributed routing behavior yet

## Execution Plan

### Batch 3.1 - Topology Contract

- [x] define bus-role, auxiliary-path, and multi-bus connection semantics
- [x] align the contract with graph snapshot and schedule expectations

### Batch 3.2 - Execution Depth

- [x] implement stronger multi-bus execution and observation depth
- [x] keep diagnostics and render surfaces aligned with the widened topology

### Batch 3.3 - Focused Proof

- [x] add focused proofs for richer bus and auxiliary routing behavior

## Acceptance Criteria

- [x] Signal has explicit multi-bus and auxiliary topology semantics
- [x] later plugin-I/O and spatial work can reuse the same routing model
- [x] hosts no longer need to flatten richer topology into baseline bus assumptions

## Risks And Mitigations

- Risk: bus expansion reopens product-local routing policy.
- Mitigation: freeze runtime topology meaning before host workflow breadth widens.

## Evidence Requirements

- [ ] log each meaningful multi-bus tranche
- [ ] run focused topology and execution validation
- [ ] record deferred topology cases explicitly

## Batch 3.1 Outcome

Batch 3.1 freezes the first reusable multi-bus and auxiliary-topology contract
in `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`.
That contract makes bus role, auxiliary path, connection identity, attachment
class, and fallback outcome explicit Signal-owned routing meaning on top of the
closed multichannel and sidechain boundaries rather than leaving richer send,
return, submix, or parallel-process depth to host-local patch conventions.

It also gives Batch 3.2 one fixed target:

- richer bus topology must now stay explicit across graph planning, runtime
  execution, render previews, and later diagnostics
- multi-bus connection identity is now bounded typed vocabulary instead of
  informal edge or send-return hints
- fallback outcomes are now runtime-owned policy rather than host convenience
  behavior
- later complex plugin-I/O and spatial milestones can widen the same routing
  family instead of reopening multi-bus meaning

This keeps `g07.003` broad and prevents the next runtime batch from getting
lost in per-host topology interpretation.

## Batch 3.2 Outcome

Batch 3.2 turns the frozen multi-bus contract into runtime-owned execution
receipts instead of leaving it as a documentation-only target.

The runtime now derives:

- explicit `RuntimeBusConnectionSummary` receipts with connection identity,
  source and target bus role, auxiliary-path grouping, attachment class, and
  fallback outcome
- explicit `RuntimeAuxiliaryPathSummary` receipts that keep send or return and
  submix paths visible across execution topology instead of flattening them
  into send-return group counts alone
- aligned `bus_connection_count`, `auxiliary_path_count`, `bus_connections`,
  and `auxiliary_paths` surfaces on `RuntimeExecutionTopologySummary`,
  `RuntimeMeteringSnapshot`, and
  `RuntimeOfflineRenderChainDependencyPreview`

This batch keeps one topology model across live execution, offline render
dependency preview, and diagnostic metering rather than letting each surface
reconstruct richer bus relationships separately.

It also leaves the next boundary explicit:

- the widened receipt family is now real and tested on runtime surfaces
- public runtime, supervisor, and stable host-edge consumer proof still
  belongs to Batch 3.3
- broader complex plugin-I/O and spatial routing depth remain later `g07`
  work rather than being smuggled into this batch

## Batch 3.3 Outcome

Batch 3.3 closes the shared consumer seam for the widened multi-bus receipt
family.

Public runtime, both stable host edges, and `signal-supervisor-tools` now all
prove the same runtime-owned multi-bus connection and auxiliary-path truth:

- runtime consumers can inspect bus-role, connection-identity, and
  auxiliary-path receipts through public `RuntimeObservationReport`
- stable local and server host edges forward the same routing receipts through
  `supervisor_report()` without host-local reinterpretation
- the machine-readable `signal.runtime.multi-bus-boundary` descriptor and
  repo-owned `acceptance:multi-bus-boundary` task make the proof seam
  inspectable and runnable rather than doc-only

This closes `g07.003` as a bounded shared routing milestone:

- multichannel, sidechain, and multi-bus now each have explicit runtime-owned
  consumer seams
- later plugin complex-I/O depth can reuse this topology substrate instead of
  reopening routing semantics
- spatial and richer plugin-bus work remain deliberately deferred rather than
  being implied by this closure

## Next Task

Continue `g07.004` with Batch 4.1 by freezing the backend-neutral complex
plugin-I/O, multi-output instrument, and bus-capable FX contract on top of the
closed multichannel, sidechain, and multi-bus routing boundaries.
