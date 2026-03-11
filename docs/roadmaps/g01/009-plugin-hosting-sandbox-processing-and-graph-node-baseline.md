# Roadmap g01.009: Plugin Hosting, Sandbox Processing, and Graph-Node Baseline

Status: queued
Owner: core-product
Created: 2026-03-10
Depends on: g01.008
Vision tags: RT, RES, PLUG
Target envelope: make plugin execution a real trust-edge runtime path by
connecting `signal-plugin`, `signal-plugin-clap`, and `signal-plugin-sandbox`
to the same graph/runtime engine contract used by native Signal processing.
That contract must also remain compatible with node-oriented mixer ideas such
as console nodes, track lanes, sends, returns, and mixed native/plugin paths.

## Problem

Signal already treats plugins as an important trust-edge concern, but a future
implementation thread needs a milestone that turns the current shells and
supervisor boundaries into a real plugin-processing lane. Without that:

1. native graph/runtime work will mature without a stable plugin-backed node
   contract,
2. sandbox recovery behavior will stay disconnected from actual render work,
3. plugin-specific transport, parameter, latency, and fault semantics will keep
   leaking into host wrappers,
4. plugin processing will be bolted onto the side of the engine instead of
   participating in the same console/lane-oriented graph model as native nodes.

## Goals

- freeze the first plugin-neutral processing contract in `signal-plugin`
- implement a concrete CLAP-backed adapter and sandbox lifecycle path
- integrate plugin-backed nodes into graph/runtime execution, including latency,
  parameter, and transport behavior
- make degraded recovery and fault reporting meaningful under real plugin work
- ensure plugin-backed nodes can occupy the same future node-oriented mixer
  topology as native processing rather than forcing a separate plugin lane model

## Non-Goals

- complete every plugin format in this batch
- solve marketplace, preset-browser, or product-level plugin UX concerns
- broaden into remote distribution policy beyond the needed sandbox/runtime seam

## Execution Plan

### 009.1 Plugin-neutral contract

- [ ] define descriptor, parameter, audio bus, state, and processing contracts
      in `signal-plugin`
- [ ] freeze the minimal lifecycle states needed by runtime and sandbox control
      paths
- [ ] align plugin fault and readiness taxonomy with runtime-owned diagnostics

### 009.2 CLAP and sandbox execution lane

- [ ] implement descriptor discovery, instance lifecycle, prepare/activate/
      process/deactivate/reset behavior for the first CLAP path
- [ ] connect sandbox transport, heartbeat, and control boundaries to real
      plugin processing rather than only lifecycle supervision
- [ ] ensure fault envelopes emitted by the CLAP/sandbox layers stay typed and
      consumable by runtime recovery policy

### 009.3 Graph/runtime integration

- [ ] add plugin-backed node execution to the graph/runtime path with explicit
      latency, tail, and bypass behavior
- [ ] route parameter and transport updates into plugin processing on the same
      timing contract used by native graph nodes
- [ ] validate degraded recovery, sandbox restart, and fallback behavior while
      real plugin work is attached to the engine
- [ ] prove plugin-backed nodes can participate in the same emerging
      track-lane, console-node, and bus-oriented graph semantics as native nodes

## Acceptance Signals

1. Signal can execute at least one real plugin-backed graph path through the
   same runtime and graph contracts used for native processing.
2. Sandbox and recovery policies are exercised against real plugin render work
   rather than only lifecycle simulations.
3. Plugin timing, latency, and fault semantics are explicit enough that later
   formats can build on them without changing the core contract.

## Risks and Mitigations

- Risk: plugin-specific edge cases pollute generic graph/runtime layers.
- Mitigation: keep plugin lifecycle and ABI details behind `signal-plugin*`
  crates, promoting only format-neutral execution semantics upward.
- Risk: sandbox recovery logic grows disconnected from render reality.
- Mitigation: require runtime and supervisor evidence from real plugin-backed
  engine paths before considering this milestone complete.

## Evidence Requirements

- [ ] meaningful plugin/sandbox batches logged under `docs/logs/YYYY-MM/`
- [ ] closure evidence must include at least one real plugin-backed execution
      scenario, not only descriptor scan or lifecycle smoke tests
- [ ] any remaining format-specific gaps recorded explicitly for follow-on
      generations

## Next Task

Use the closure of `g01.009` to decide whether `g02` should split by deeper
engine specialization, broader platform/device support, or analysis/media
expansion now that the first end-to-end shared Signal processing stack is real.
