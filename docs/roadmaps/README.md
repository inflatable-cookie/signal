# Roadmaps

Status: active
Updated: 2026-03-11

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

There is no active Signal generation right now. `g01` established the Rust
workspace, engine, host/device path, and plugin/runtime baseline; `g02`
completed the first reusable DSP and deep-analysis expansion on top of that
foundation.

The most recently completed continuation runway was:

- shared streaming spectral and resampling substrate
- rhythm structure and tempo continuity depth
- tonal and harmonic analysis depth
- loudness and dynamics depth
- transient/timbral descriptor packs
- embedding and benchmark hardening

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Next Task

The current roadmap generation queue is complete. Open `g03` only when a new
sequenced continuation boundary is preferable to a backlog item.
