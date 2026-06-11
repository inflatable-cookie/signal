# 015 - WYSIWYG Bounce On The Render Plane

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.010
Vision tags: `EXPORT`, `TRUTH`

## Problem

signal-runtime's offline render drives the simulation graph with linear
export resampling — a different mix than playback. Bounce must be the same
executor over the same compiled plan, faster than realtime. Pulse's
render-queue job model already exists and waits for a real engine behind
it.

## Goals

- [ ] offline driver: instantiate a controller/executor pair without a stream, install the same RenderPlanSpec pulse compiles for playback, loop render_block faster than realtime into a WAV writer (hound)
- [ ] TPDF dither at bit-depth reduction on export
- [ ] stems = per-bus capture from the schedule (cheap once g10.010 lands)
- [ ] loudness-normalized export option reusing signal-analysis-loudness
- [ ] retire signal-runtime's simulation-graph offline path once pulse's render queue targets the new driver
- [ ] golden test: bounce of a reference plan is sample-identical to a captured realtime render of the same plan

## Execution Plan

### Batch 15.1 - Offline Driver

- [ ] executor loop + WAV write + dither; pulse render-queue wiring

### Batch 15.2 - Equivalence And Retirement

- [ ] bounce==playback golden test; stems; retire simulation offline path

## Acceptance Criteria

- [ ] exported WAV sample-identical to realtime capture of the same plan
- [ ] export runs faster than realtime on reference material
- [ ] simulation offline path deleted with its tests

## Next Task

g10.020 (runtime endgame) becomes unblocked once this lands.
