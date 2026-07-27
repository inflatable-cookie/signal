# 039 - Resumable Offline Stretch Render

Status: planned; blocked on `g10.038`
Owner: dsp
Created: 2026-07-27
Depends on: `g10.036`, `g10.038`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `RENDER`, `ARCHITECTURE`

## Problem

`signal-dsp-stretch` offers only whole-buffer entry points: `stretch_mono`,
`stretch_interleaved_stereo`, and their pitch and dynamic-ratio variants. Every
caller that needs bounded memory must therefore slice the source, render each
slice with context, crop, and patch the joins itself.

`signal-render-plane` does exactly that. `plan_offline_stretch_chunks` produces
the plan, and `materialize_chunked_offline_stretch_artifact_frames` renders each
chunk through a freshly constructed `OfflineHighQualityStretcher`. Phase state,
transient history, and peak tracking reset at every chunk boundary, and the
result is patched by a second copy of the same seam hack the crate uses
internally.

That hack, audit finding `A8`, is not a crossfade. It takes the two samples
either side of a boundary, computes their midpoint, and adds a linearly
decaying offset over `256` frames on each side. It removes the step and injects
a low-frequency ramp derived from two single samples. There are now two
implementations of it — one in the crate, one in the render plane — patching
discontinuities that exist only because the renderer cannot carry state across
a boundary.

`g10.036` Batch 36.4 ships seam parity across channel counts as a stopgap and
says so explicitly. This lane is the durable answer. Contract `046` already
requires that dynamic-ratio output "preserves continuous algorithm state or
uses an explicitly measured transition mechanism rather than raw segment
concatenation"; the current implementation satisfies neither.

## Generation Runway

This lane advances the `g10` runway from *a correct whole-buffer renderer* to
*a renderer that stays correct at export scale*. It closes the last structural
finding from the 2026-07-27 audit and removes the duplicated seam hack from
both crates.

The visible runway is:

1. state-boundary audit and contract amendment
2. complete resumable-render brief
3. one isolated implementation
4. render-plane adoption and hack removal
5. closeout

The next planning checkpoint is Batch 39.2, where the brief must show that
carried state is bounded and duration-independent, the failure class that
closed several `g10.031` candidates.

## Goals

- [ ] one renderer that carries phase, transient, and peak state across chunk
  and dynamic-ratio boundaries
- [ ] bounded, duration-independent working state
- [ ] deterministic output independent of chunk size
- [ ] both copies of the seam hack removed rather than duplicated a third time
- [ ] `signal-render-plane` stops constructing a stretcher per chunk

## Non-Goals

- no new renderer family, detector, window, or phase strategy
- no realtime or callback claim; this is offline render only
- no cache identity change beyond what `g10.037` already froze
- no creative-character work
- no artifact format, storage, or export-schema change
- no Loophole or Chorus surface

## Execution Plan

### Batch 39.1 - State Boundary Audit And Contract Amendment

Status: blocked on `g10.038` Batch 38.6

Documentation only.

- [ ] enumerate every piece of renderer state that currently resets at a chunk
  or dynamic-ratio segment boundary, and what each reset costs audibly
- [ ] measure the current chunked artifact path against a single whole-buffer
  render of the same source to quantify the boundary cost
- [ ] amend Contract `046` with the resumable offline render boundary: what
  state must persist, what determinism is promised across chunk sizes, and what
  memory bound applies
- [ ] decide whether the ratio curve is consumed by the renderer directly or
  stays a caller-side segmentation concern
- [ ] change documentation only

### Batch 39.2 - Complete Resumable Render Brief

Status: blocked on Batch 39.1

Documentation only.

- [ ] freeze the API shape: construction, per-chunk render, carried state,
  flush, and reset
- [ ] freeze the state inventory with a geometry-derived capacity for each
  item and a total memory ceiling
- [ ] prove the ceiling is duration-independent
- [ ] freeze the equivalence law: chunked render output versus whole-buffer
  render output, and the tolerance if it is not bit-exact
- [ ] freeze the evidence order and the cleanup and rejection rules
- [ ] change documentation only

### Batch 39.3 - Isolated Implementation

Status: blocked on Batch 39.2

- [ ] implement the frozen brief once, isolated per Contract `084` Rule 2
- [ ] prove chunk-size independence across at least three chunk sizes
- [ ] prove the memory ceiling holds for long sources
- [ ] prove dynamic-ratio transitions carry state instead of concatenating
- [ ] measure seam artifacts against the Batch 39.1 baseline

### Batch 39.4 - Render Plane Adoption

Status: blocked on Batch 39.3

- [ ] replace per-chunk stretcher construction with the resumable renderer
- [ ] remove `smooth_artifact_chunk_boundaries_interleaved` from the render
  plane and the internal seam smoother from the crate, once measurement shows
  they are no longer needed
- [ ] prove artifact output is unchanged or improved by the frozen seam metric
- [ ] keep the chunk plan as the memory-bounding authority

### Batch 39.5 - Closeout

Status: blocked on Batch 39.4

- [ ] run `effigy validate`, the full crate suite, and the corpus report
- [ ] update Contract `046` and the `g10` front doors
- [ ] state explicitly whether any seam mechanism remains and why
- [ ] name the next ready batch in `g10.040`

## Acceptance Criteria

- [ ] chunked and whole-buffer renders agree within the frozen equivalence law
- [ ] output is independent of chunk size
- [ ] working state is bounded and duration-independent, proven for long
  sources
- [ ] dynamic-ratio boundaries carry state rather than concatenate segments
- [ ] no seam hack remains in either crate unless Batch 39.5 records why
- [ ] the render plane constructs one renderer per artifact, not one per chunk
- [ ] full crate suite and corpus report pass

## Risks and Mitigations

- Risk: carried state grows with source duration, the failure class that closed
  several `g10.031` candidates. Mitigation: Batch 39.2 must freeze a
  geometry-derived capacity per item and prove duration independence before any
  implementation opens.
- Risk: chunked output stops matching whole-buffer output. Mitigation: the
  equivalence law and its tolerance are frozen before implementation, not
  chosen after measurement.
- Risk: this reopens successor research. Mitigation: the transform is
  unchanged; only its state lifetime and entry points change, and byte-exact
  whole-buffer output remains a control.
- Risk: removing the seam hack exposes an artifact it was masking. Mitigation:
  Batch 39.4 removes it only after measurement shows the boundary is clean.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the Batch 39.1 boundary-cost measurement against a whole-buffer control
- [ ] the frozen state inventory with capacities and total ceiling
- [ ] chunk-size independence proof across at least three sizes
- [ ] before/after seam metric for the artifact path
- [ ] commands actually run

## Next Task

Blocked. Open Batch 39.1 after `g10.038` Batch 38.6 closes.
