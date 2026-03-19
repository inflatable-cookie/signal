# g07 Milestones

Status: complete
Updated: 2026-03-19

## Why this generation matters next

`g06` is now closed as the runtime-hardening and feature-baseline queue.
Loophole's next product-visible pressure is no longer only resilience and
first adapter breadth. The remaining reusable Signal work now shifts toward
deeper routing and multichannel semantics, Linux-native runtime breadth,
external MIDI and control-surface substrate, and fuller sample-domain
time-stretch capability.

This generation therefore focuses on feature-expansion depth that still belongs
inside Signal-owned runtime, adapter, hardware, and media substrate:

- canonical multichannel, sidechain, multi-bus, and complex plugin-I/O meaning
- spatial execution beyond the current narrow adapter contract
- LV2 and stronger Linux-native plugin and hardware backend breadth
- external MIDI endpoint, controller-expression, and control-surface substrate
- sample-domain time-stretch, transform preview, and post-warp artifact depth
- acceptance evidence strong enough for Loophole to trust the widened feature
  surface without rebuilding it host-locally

## Chorus-derived intake

This generation is intentionally shaped by repeated Chorus demand in:

- `chorus/specs/engine/spatial-adapters.md`
- `chorus/specs/engine/graph-snapshot-contract.md`
- `chorus/specs/engine/schedule-and-graph.md`
- `chorus/specs/signal-audio-backend-plugin-hosting.md`
- `chorus/specs/ipc/pulse/routing.md`
- `chorus/specs/ipc/pulse/control.md`
- `chorus/specs/ipc/pulse/hardware-io.md`
- `chorus/roadmaps/g03/017-hardware-integration-monitoring-and-external-io-depth.md`
- `chorus/roadmaps/g03/018-media-library-indexing-waveform-analysis-and-preview-services.md`
- `chorus/roadmaps/g05/014-control-surface-scripting-and-advanced-hardware-extensibility-depth.md`

## Dependency order

1. freeze multichannel and routing substrate first
2. deepen sidechain, multi-bus, and complex plugin-I/O behavior second
3. widen spatial execution on top of the new routing contract
4. extend Linux plugin and hardware breadth on the now-explicit topology model
5. add external MIDI and control-surface substrate after endpoint and backend
   identity are stronger
6. deepen sample-domain time-stretch after routing, media, and analysis
   surfaces are ready
7. close with acceptance and Loophole-facing feature-readiness evidence

## Milestone map

- `g07.001` `complete`
  - canonical multichannel layout and channel-role substrate
- `g07.002` `complete`
  - sidechain routing and secondary-input execution depth
- `g07.003` `complete`
  - multi-bus graph execution and auxiliary topology depth
- `g07.004` `complete`
  - plugin complex I/O topology and multi-output instrument depth
- `g07.005` `complete`
  - spatial adapter execution baseline
- `g07.006` `complete`
  - surround bed, object, and mix-policy expansion
- `g07.007` `complete`
  - LV2 adapter baseline and Linux-native plugin lifecycle depth
- `g07.008` `complete`
  - Linux cross-adapter plugin parity and sandbox policy
- `g07.009` `complete`
  - Linux audio backend portability across ALSA, JACK, and PipeWire
- `g07.010` `complete`
  - Linux backend clocking, duplex, and endpoint-topology parity
- `g07.011` `complete`
  - external MIDI endpoint graph and device-identity baseline
- `g07.012` `complete`
  - MIDI 2.0, MPE, and richer controller-expression depth
- `g07.013` `complete`
  - control-surface transport, mapping, and feedback baseline
- `g07.014` `complete`
  - advanced hardware extensibility and scripting-safe device policy
- `g07.015` `complete`
  - sample-domain time-stretch engine baseline
- `g07.016` `complete`
  - warp-marker, transient-anchor, and tempo-assist analysis depth
- `g07.017` `complete`
  - post-warp render, cache, and transform-artifact depth
- `g07.018` `complete`
  - low-latency audition, scrub, and preview transform services
- `g07.019` `complete`
  - multichannel, Linux, time-stretch, and control-surface acceptance depth
- `g07.020` `complete`
  - generation closeout and Loophole feature-readiness gate

## Lane structure

### Lane A - Routing, Multichannel, And Spatial Execution

`001 -> 002 -> 003 -> 004 -> 005 -> 006`

Freeze the reusable routing and channel-layout substrate before widening
spatial and complex plugin behavior.

### Lane B - Linux Plugin And Backend Breadth

`007 -> 008 -> 009 -> 010`

Extend plugin and hardware portability into a deliberate Linux-native posture
instead of treating it as a side effect of generic backend work.

### Lane C - MIDI Hardware And Control Surfaces

`011 -> 012 -> 013 -> 014`

Create reusable external MIDI, controller-expression, and control-surface
substrate that product hosts can observe and configure without owning the low
level transport.

### Lane D - Time-Stretch And Transform Services

`015 -> 016 -> 017 -> 018`

Deepen the current bounded warp surface into fuller sample-domain stretch,
transform analysis, and preview or artifact services.

### Lane E - Acceptance And Promotion

`019 -> 020`

Turn the widened feature surface into durable runtime evidence before claiming
the generation is ready for downstream reliance.

## Working rules for this thread

- keep routing, plugin, hardware, MIDI, and stretch semantics inside
  Signal-owned crates and typed surfaces
- do not promote product-local browser, console, or editor UX into this
  generation
- treat Linux breadth, LV2, control-surface substrate, and time-stretch as
  reusable runtime work rather than host convenience wrappers
- prefer machine-readable receipts, descriptors, and Effigy tasks over prose
  claims
- keep `g07` as the single active queue and push any non-generation-critical
  spillover back into backlog instead of reopening `g06`

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
