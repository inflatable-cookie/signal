# Roadmaps

Status: active
Updated: 2026-03-22

## Why this section matters now

Roadmaps turn the Signal library/runtime strategy into executable batches.

## Scope

Use this section for:

- active implementation milestones
- generation control
- backlog and deferred work

## Layout

- `g*/`: generation roadmaps and closure records
- `generation-index.md`: generation history and rollover notes
- `backlog/`: deferred work only
- `templates/`: roadmap authoring support

## Current posture

`g06` and `g07` are now complete and `g08` is now the active generation.
`g01` established the Rust workspace, engine, host/device
path, and plugin/runtime baseline; `g02` completed the first reusable DSP and
deep-analysis expansion on top of that foundation; `g03` completed the next
engine-oriented runtime depth queue; `g04` completed the reusable contract,
scheduling, portability, conformance, and release-baseline queue; `g05`
completed the widened backend, host-edge, publication-packaging,
downstream-automation, and generation-closeout queue; `g06` then closed the
next deeper Signal-owned runway around runtime recovery, instrumentation,
plugin-format breadth, MIDI/event expansion, hardware and external-I/O depth,
media-service depth, and shared acceptance evidence that tangibly moved
Loophole forward; `g07` then closed the bounded feature-expansion queue around
routing, Linux breadth, control-surface substrate, and sample-domain transform
services; and `g08` is now the active live-ownership and workflow-depth queue.

The previously completed continuation runway was:

- shared streaming spectral and resampling substrate
- rhythm structure and tempo continuity depth
- tonal and harmonic analysis depth
- loudness and dynamics depth
- transient/timbral descriptor packs
- embedding and benchmark hardening

The newly completed continuation runway was:

- routed mixer graph, buses, and topology depth
- runtime metering, loudness, and diagnostics export
- automation playback and control-resolution depth
- tempo-map, warp, clip-processing, and render substrate
- plugin device-chain execution, latency compensation, and state recall
- offline render, freeze, and stem export pipeline
- profiling, soak harnesses, and runtime hardening

The latest completed continuation runway was:

- crate/public-contract maturity and schema-freeze baseline
- multicore scheduling and anticipative execution depth
- runtime work orchestration and deferred-service policy
- hardware backend portability and clock-domain boundary depth
- plugin backend breadth and host-neutral delegation contracts
- consumer conformance, export stability, and release packaging

The latest completed continuation runway was:

- backend-neutral plugin capability and adapter breadth baseline
- shared host convenience API and consumer-edge contracts
- publication-grade packaging manifests and release automation receipts
- downstream conformance soak and release-acceptance automation
- generation closeout and promotion gate

The latest completed continuation runway was:

- runtime interruption, resumability, and recovery truth
- profiling, causal diagnostics, and deferred-work orchestration
- VST3 and AU adapter breadth plus richer generic MIDI/event semantics
- hardware supervision, external I/O, monitoring, and loopback depth
- media indexing, waveform analysis, preview, and metadata services
- fault injection, long-session soak, and Loophole-facing runtime readiness

The newly completed continuation runway was:

- spatial, multichannel, sidechain, and complex plugin-I/O depth
- LV2 plus deeper Linux plugin and hardware backend breadth
- external MIDI, control-surface, and advanced hardware device substrate
- fuller sample-domain time-stretch and transform-service depth

The newly active continuation runway is:

- live Linux audio backend ownership and session lifecycle depth
- deeper LV2, complex plugin-I/O, and backend-native protocol breadth
- immersive routing, room-policy, and richer device-protocol substrate
- preview-device, audition, and product-adjacent workflow services that remain
  runtime-owned

The deferred continuation scope after `g06` is:

- broader unstable `server soak` and recovery-overlap depth
- wider advisory rerun confidence beyond the bounded closeout gate

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Next Task

Continue `g08.015` with Batch 15.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared cross-backend device protocol and live
workflow seam while keeping backend-specific depth explicit and non-blocking.
