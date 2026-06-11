# 015 - WYSIWYG Bounce On The Render Plane

Status: complete
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

## Progress (2026-06-11)

- Offline driver (`signal-render-plane/src/offline.rs`): `render_plan_to_pcm`
  drives a fresh controller/executor pair over the same compiled plan —
  boundary as identity copy, transport drained while inaudible then the edge
  envelope snapped open via a crate-private hook (realtime behavior
  untouched), blocks looped into interleaved PCM. Stems captured post-fader
  from stage scratch with the exact block-gain ramp the edges consume, so
  unity stems sum to the master bit-tight. `write_wav` with
  Float32/Int24/Int16; integer paths TPDF-dithered (LCG, no rand dep).
  Six tests: manual-loop sample identity, faster-than-realtime, stems sum,
  dither round-trip bounds + decorrelation, float bit-exactness, no fade-in.
- Pulse: `bounce_mix_to_wav` (extent = last clip end + 1 s tail, Float32),
  BounceReport, render-queue job registration when the queue is idle;
  e2e test bounces a tone project to a hound-readable wav.
- Aura: `aura_export_mix` (save dialog, mirrors import pattern) + Export mix
  toolbar button enabled when clips exist.
- Retirement: signal-runtime's simulation offline path deleted (~10.2k LoC
  net): offline_render_delivery, runtime_offline_render_session, deferred
  offline executors + receipt machinery, the RuntimeOffline* serde report
  family, engine render_offline_block, 11 trait methods + host-local
  passthroughs, all offline test suites. Media decode/analysis pipeline
  kept; the real clip-processing API moved to runtime_media_services.
  Bounce now has exactly one meaning: the playback engine, faster.
- Gates: signal workspace + clippy + fmt + zero-alloc soak green; pulse
  123/123; aura cargo/vite/vitest green, svelte-check at baseline 8.

## Next Task

g10.020 (runtime endgame) becomes unblocked once this lands.
