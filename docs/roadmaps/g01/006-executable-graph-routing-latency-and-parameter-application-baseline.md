# Roadmap g01.006: Executable Graph Routing, Latency, and Parameter Application Baseline

Status: complete
Owner: core-product
Created: 2026-03-10
Depends on: g01.005
Vision tags: RT, DSP, ENG
Target envelope: turn `signal-graph` from a promising execution shell into a
real reusable graph-processing substrate with deterministic routing, latency,
tail, parameter-event semantics, and the first credible node-oriented topology
contract that later mixer concepts can inherit.

## Problem

The runtime layer already has meaningful execution state, but the graph layer
still needs a stronger contract for how audio actually moves through nodes and
how time-sensitive control changes are applied. Without that:

1. runtime scheduling rules will be built on a graph seam that is too vague,
2. host/plugin integration will paper over routing and timing ambiguities with
   ad hoc glue,
3. sample-accurate parameter behavior will drift between runtime, graph, and
   plugin-backed nodes,
4. later console-node, track-lane, and mixer-topology work will be forced to
   retrofit meaning onto a graph seam that never declared those shapes.

## Goals

- define a stable executable graph node and bus contract
- implement deterministic routing/mixing behavior across fan-in, fan-out, and
  silence paths
- make latency, tail, and reset semantics explicit in graph-owned structures
- support sample-accurate or bounded-sub-block parameter event application
- keep the node and bus model extensible enough for track-lane, console-node,
  bus, send, and return semantics without inventing a second graph later

## Non-Goals

- building a full DAW editing model in this batch
- implementing plugin-specific processing inside `signal-graph`
- solving distributed or remote execution policy here

## Execution Plan

### 006.1 Node and buffer contract

- [x] tighten the executable node contract for input/output buses, channel
      counts, scratch buffers, and reset lifecycle
- [x] define graph-owned silence and channel-adaptation rules so nodes do not
      invent incompatible assumptions locally
- [x] make node execution-class and planning metadata authoritative enough for
      runtime scheduling to rely on them
- [x] define the first topology-facing node metadata needed so future
      track-lane, console-node, bus, send, and return concepts can map onto the
      graph cleanly instead of relying on host-only naming or hidden side
      tables

### 006.2 Routing and latency semantics

- [x] implement deterministic routing/mix rules for direct edges, fan-in, and
      fan-out paths
- [x] make graph validation reject or explicitly classify unsupported cycles and
      feedback cases
- [x] calculate and surface latency/tail contribution through graph-owned
      structures instead of leaving that as host-local lore
- [x] extend block reports so graph execution outcomes expose enough detail for
      runtime diagnostics and scheduling decisions
- [x] add fixtures that prove node-oriented routing shapes such as track-lane to
      bus, bus to console-node, and send/return-style fan-out can be expressed
      without redefining core routing rules

### 006.3 Parameter-event application

- [x] define the graph-side parameter event shape and its relationship to
      runtime batches
- [x] support sub-block splitting or another explicit bounded strategy for
      applying time-sensitive events inside a processing block
- [x] add fixtures that exercise gain/filter/delay-style nodes under parameter
      movement rather than testing only static steady-state processing
- [x] record how transport-facing state and parameter events share timing rules
      so runtime can remain the authority above a deterministic graph seam
- [x] record which node-oriented mixer semantics are deliberately deferred after
      this batch, so later console and lane work builds on declared graph
      assumptions rather than folklore

Timing note:
- runtime remains authoritative for transport state, block selection, and
  parameter batch epoch assignment
- graph owns only block-local parameter-event interpretation, including sample
  offsets relative to the current block and the bounded sub-block application
  strategy used inside node processing

Deferred mixer and node semantics:
- console-node, lane-strip, and bus-level mix policies such as pan law, solo,
  mute, and stem ownership are still deferred; `signal-graph` only establishes
  the topology and processing contract they will sit on
- plugin-backed node parameter interpretation remains a runtime/trust-edge
  concern until plugin processing is deepened in later milestones
- dynamic filter and delay kernels are currently rebuilt per graph block; cross
  block state retention is deferred because prework and prepared-dispatch
  handoff would need explicit state snapshot semantics, not just routed buffers

## Acceptance Signals

1. `signal-graph` exposes a credible node/bus/routing contract rather than a
   mostly structural placeholder.
2. Routing, latency, and parameter timing behavior are explicit enough that
   runtime and plugin work can build on them without redefining graph meaning.
3. Tests demonstrate deterministic behavior for mixing, latency reporting, and
   parameter application under realistic node patterns.
4. The graph seam is explicitly compatible with later console-node, track-lane,
   and bus-topology work even if that richer topology is not fully implemented
   yet.

## Risks and Mitigations

- Risk: routing semantics get overfit to one demo graph shape.
- Mitigation: include fixtures that cover fan-in, fan-out, silence, and mixed
  stateful/stateless node paths.
- Risk: graph and runtime both try to own timing behavior.
- Mitigation: keep runtime authoritative for transport/scheduler state while
  graph owns only the processing contract used inside each block.

## Evidence Requirements

- [x] meaningful graph-contract or routing batches logged under `docs/logs/YYYY-MM/`
- [x] closure evidence must name the fixtures or tests used to prove routing and
      parameter timing behavior
- [x] any deferred graph cases such as feedback or multibus complexity recorded
      explicitly instead of silently omitted
- [x] closure evidence must state how future node-oriented mixer concepts fit
      the graph seam established here

## Next Task

Begin `g01.007` by making `signal-runtime` the explicit authority for transport
progression, block-clock truth, and scheduler invalidation on top of the now
completed graph routing and parameter-application seam.
