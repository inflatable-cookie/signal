# Roadmaps

Status: active
Updated: 2026-03-12

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

No Signal generation is currently open. `g01` established the Rust
workspace, engine, host/device path, and plugin/runtime baseline; `g02`
completed the first reusable DSP and deep-analysis expansion on top of that
foundation; `g03` completed the next engine-oriented runtime depth queue; and
`g04` completed the reusable contract, scheduling, portability, conformance,
and release-baseline queue.

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

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Next Task

COMPLETE. `g04` closed on 2026-03-12. The next likely queue is recorded in
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md` and
should only be promoted when maintainers want to open the post-`g04`
generation.
