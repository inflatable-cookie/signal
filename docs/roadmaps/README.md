# Roadmaps

Status: active
Updated: 2026-03-10

## Why this section matters now

Roadmaps turn the Signal library/runtime strategy into executable batches.

## Scope

Use this section for:

- active implementation milestones
- generation control
- backlog and deferred work

## Layout

- `g01/`: current active generation
- `generation-index.md`: generation history and rollover notes
- `backlog/`: deferred work only
- `templates/`: roadmap authoring support

## Current posture

`g01` is no longer just a shell-bootstrap queue. It now has a defined next-stage
engine and DSP runway so a dedicated Signal implementation thread can work in
parallel on:

- core DSP/control kernels
- executable graph routing and parameter timing
- runtime transport and scheduler ownership
- host/device-backed audio execution
- plugin and sandbox processing

## Working Rule

- keep one active queue
- log by meaningful batch
- move deferred scope into backlog instead of leaving it half-active

## Next Task

Advance `g01.004`, then open `g01.005` as the first algorithm/engine-heavy
milestone in the new detailed Signal runtime runway.
