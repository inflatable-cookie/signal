# g06 Milestones

Status: active
Updated: 2026-03-14

## Why this generation matters now

`g05` closed Signal's first broad reusable product boundary, but Loophole's
current bottleneck is no longer contract shape alone. The remaining pressure is
whether Signal can supply deeper runtime recovery truth, actionable execution
instrumentation, broader real plugin/backend functionality, stronger hardware
and media services, and integrated soak evidence without pushing those concerns
back into product hosts.

This generation therefore mixes hardening and feature breadth deliberately:

- runtime interruption, resumability, and recovery semantics that products can
  trust without host-local reconstruction
- profiling, causal diagnostics, and deferred-work policy that turn later
  optimization into data-driven work
- concrete feature breadth where the current Signal surface is still narrow,
  especially VST3, AU, Linux-hosted plugin coverage, ARA-capable clip/plugin
  context, generic MIDI/event semantics beyond the CLAP-first path, and
  explicit user-selectable plugin isolation policy instead of one fixed
  sandboxing stance
- hardware supervision, monitoring, external I/O, and media-service depth that
  Loophole still needs as reusable library/runtime substrate
- shared fault-injection, soak, and promotion evidence strong enough to support
  Loophole's remaining `g03` hardening and cutover work

## Chorus-Derived Intake

This generation is intentionally aligned to active or explicit Loophole demand
surfaced in Chorus:

- `chorus g03.016` needs real runtime profiling, scheduler metrics, and
  background orchestration substrate
- `chorus g03.017` needs stronger hardware supervision, monitoring, and
  external-I/O depth
- `chorus g03.018` needs waveform analysis, preview, and media-service depth
- `chorus g03.019` needs integrated acceptance harnesses, fault injection, and
  long-session soak evidence
- Chorus vision and architecture still call out bounded AU/CLAP/VST3 scope,
  MIDI/editor foundations, hardware MIDI/audio I/O, monitoring, sidechain,
  ARA-capable clip/plugin integration, and plugin lifecycle depth as important
  product-facing capability fronts that should land in reusable runtime
  substrate before product-local wrappers expand again

## Dependency order

1. freeze interruption and resumability contracts first
2. deepen recording, plugin, and render recovery truth second
3. add profiling and deferred-work policy before optimization and soak claims
4. widen plugin-format and MIDI/event breadth on top of the now-explicit
   runtime-owned lifecycle model
5. deepen hardware, external-I/O, and media services after recovery and
   instrumentation exist
6. close with fault-injection, soak, and Loophole-facing readiness evidence

## Milestone map

- `g06.001` `complete`
  - runtime interruption taxonomy and resumability contract
- `g06.002` `complete`
  - recording continuity, MIDI capture, and checkpoint truth
- `g06.003` `active`
  - plugin isolation policy, transport rebind, and shared-sandbox continuity depth
- `g06.004` `planned`
  - offline render execution recovery and resumability depth
- `g06.005` `planned`
  - runtime fault-cause attribution and diagnostic receipts
- `g06.006` `planned`
  - per-block execution timing and pressure snapshots
- `g06.007` `planned`
  - graph critical-path, hot-node, and worker-lane instrumentation
- `g06.008` `planned`
  - deferred-work scheduler priority, backpressure, and cancellation policy
- `g06.009` `planned`
  - VST3 adapter baseline and runtime-owned lifecycle depth
- `g06.010` `planned`
  - AU adapter baseline and runtime-owned lifecycle depth
- `g06.011` `planned`
  - backend capability parity, Linux plugin support, and cross-adapter conformance depth
- `g06.012` `planned`
  - generic MIDI, note-expression, and plugin-event model expansion
- `g06.013` `planned`
  - plugin preset, ARA context, state interchange, and portable recall depth
- `g06.014` `planned`
  - device supervision, restart-state machine, and fault-boundary depth
- `g06.015` `planned`
  - clock-domain drift, duplex mismatch, and endpoint-topology depth
- `g06.016` `planned`
  - external I/O, monitoring tap-point, and loopback measurement contracts
- `g06.017` `planned`
  - media indexing, waveform analysis, and preview service baseline
- `g06.018` `planned`
  - analysis metadata extraction and library-service depth
- `g06.019` `planned`
  - fault-injection harnesses and multi-backend acceptance depth
- `g06.020` `planned`
  - long-session soak, promotion gate, and Loophole-readiness closeout

## Lane structure

### Lane A - Recovery And Continuity

`001 -> 002 -> 003 -> 004 -> 005`

Make runtime interruption, recovery, and fault ownership strong enough that
products observe recovery truth instead of inventing it.

### Lane B - Profiling And Orchestration

`006 -> 007 -> 008`

Turn scheduler, graph, and deferred-work behavior into typed, actionable
runtime evidence.

### Lane C - Plugin Breadth And Event Semantics

`009 -> 010 -> 011 -> 012 -> 013`

Extend the current CLAP-first runtime path into broader, still-runtime-owned
plugin functionality across more formats, platforms, plugin-context depth, and
explicit placement or isolation policy.

### Lane D - Hardware And Media Services

`014 -> 015 -> 016 -> 017 -> 018`

Deepen reusable hardware supervision, external-I/O, monitoring, waveform, and
analysis-service substrate.

### Lane E - Acceptance And Promotion

`019 -> 020`

Prove the generation through reusable fault-injection, soak, and downstream
runtime evidence.

## Working rules for this thread

- keep recovery, profiling, plugin, hardware, and media semantics inside
  Signal-owned crates and typed surfaces
- do not promote product-local plugin browser, session, or UI workflow behavior
  into this generation
- treat AU/VST3, Linux plugin breadth, ARA-capable plugin context, MIDI/event
  breadth, plugin isolation policy, and media services as reusable runtime
  work, not host-convenience wrappers
- prefer machine-readable receipts, descriptors, and Effigy tasks over prose
  claims
- keep one active queue and move anything not generation-critical back into
  backlog rather than stretching `g06` into remote/distributed product scope

## Next Task

Continue `g06.003` with Batch 3.1 by freezing placement-rule vocabulary,
sandbox grouping keys, and shared plugin rebind or continuity semantics before
widening deeper recovery implementation or feature breadth.
