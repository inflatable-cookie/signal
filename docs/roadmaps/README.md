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

The most recently completed Signal generation is now `g03`. `g01` established
the Rust workspace, engine, host/device path, and plugin/runtime baseline;
`g02` completed the first reusable DSP and deep-analysis expansion on top of
that foundation; `g03` completed the next engine-oriented runtime depth queue.

The most recently completed continuation runway was:

- shared streaming spectral and resampling substrate
- rhythm structure and tempo continuity depth
- tonal and harmonic analysis depth
- loudness and dynamics depth
- transient/timbral descriptor packs
- embedding and benchmark hardening

The newly completed continuation runway is:

- routed mixer graph, buses, and topology depth
- runtime metering, loudness, and diagnostics export
- automation playback and control-resolution depth
- tempo-map, warp, clip-processing, and render substrate
- plugin device-chain execution, latency compensation, and state recall
- offline render, freeze, and stem export pipeline
- profiling, soak harnesses, and runtime hardening

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Next Task

COMPLETE. `g03` is closed after the routed engine, offline render, and
hardening runway landed. Open the next generation when maintainers want the
next continuation queue.
