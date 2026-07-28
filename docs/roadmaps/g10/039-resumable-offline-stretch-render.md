# 039 - Resumable Offline Stretch Render

Status: active; Batch 39.4 rejected by listening and reverted; renderer defect open
Owner: dsp
Created: 2026-07-27
Updated: 2026-07-27
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

Status: complete

Documentation only. No crate source changed.

- [x] enumerate every piece of renderer state that currently resets at a chunk
  or dynamic-ratio segment boundary, and what each reset costs audibly
- [x] measure the current chunked artifact path against a single whole-buffer
  render of the same source to quantify the boundary cost
- [x] amend Contract `046` with the resumable offline render boundary: what
  state must persist, what determinism is promised across chunk sizes, and what
  memory bound applies
- [x] decide whether the ratio curve is consumed by the renderer directly or
  stays a caller-side segmentation concern
- [x] change documentation only

Result:

- six pieces of state reset at every join: `synthesis_phase`, `previous_phase`,
  `previous_magnitudes`, `previous_energy`, the `frame_index == 0` branch, and
  the overlap-add accumulation
- the two detector baselines had not been named before. A transient landing on
  the first frame after a join is undetectable, because flux and energy have
  nothing to compare against. That is the leading hypothesis for `A18`: both
  sides of the `g10.036` listening round were segmented renders, so both
  carried it
- measured at the shipped `30`-second chunk policy on `60` seconds of stereo
  material at ratio `1.25`, against a single-chunk control: correlation
  `0.389976`, peak sample difference `1.0752`, and a seam step of `-240 dBFS`
  against the control's own `-45.14 dBFS` signal step
- the seam is flat and the renders still diverge. Sample continuity is not
  phase continuity, which is precisely why segment-length tuning could never
  fix what the listening rounds heard. Every export longer than the chunk
  policy carries this today
- ratio-curve ownership decided: the renderer consumes the curve. Caller-side
  segmentation is what creates the joins, and a state-carrying renderer needs
  the active ratio per analysis frame rather than a pre-cut list of spans.
  `plan_offline_stretch_chunks` stays the bounded-memory authority and stops
  being a segmentation authority, which also closes the `g10.036` scope
  boundary where a dense curve could still cut sub-window chunks
- chunk-size independence is frozen as exact, not tolerance-bounded. A
  tolerance would readmit the defect the lane exists to remove
- this is not a byte-exact lane. Batch 39.2 must state which `g10.036` controls
  survive before implementation opens, and the behavior version and cache
  schema advance again when it lands

### Batch 39.2 - Complete Resumable Render Brief

Status: complete

Documentation only. No crate source changed.

- [x] freeze the API shape: construction, per-chunk render, carried state,
  flush, and reset
- [x] freeze the state inventory with a geometry-derived capacity for each
  item and a total memory ceiling
- [x] prove the ceiling is duration-independent
- [x] freeze the equivalence law
- [x] freeze the evidence order and the cleanup and rejection rules
- [x] state which `g10.036` byte-exact controls survive a state-carrying
  renderer, and which must be re-baselined
- [x] change documentation only

Result:

- API is `new`, `render`, `flush`, `reset`. Output is not one-to-one with
  input, so the renderer holds up to one window of unsynthesized source and
  `flush` drains it. The ratio curve lives in the config, per the Batch 39.1
  ownership decision
- six state items persist, and the `frame_index == 0` branch becomes a
  construction-time condition rather than a per-call one. That is what actually
  removes the join
- memory ceiling frozen at `8 MiB`. Derived, not asserted: `249932 B` at the
  retained `2048` window in stereo, `7995468 B` at the frozen `65536` maximum,
  leaving `393140 B` headroom
- the maximum window is new. `with_window` clamps only to a power of two at or
  above `64` with no upper limit, so "bounded by geometry" was not a number
  until this brief froze one
- equivalence is exact: for any chunk partition, concatenated output must be
  bit-identical to one whole-source render plus `flush`
- surviving controls named. Seven `g10.036` owners survive unchanged, three
  change meaning because coalescing and seams disappear with the joins, and
  `segmented_render_matches_whole_render_at_constant_ratio` must flip from
  `0.034` correlation to passing at `0.99`

### Batch 39.3 - Isolated Implementation

Status: structural gates green; acoustic checkpoint not yet opened

- [x] implement the frozen brief once, isolated per Contract `084` Rule 2 —
  isolation waived by the operator, so the candidate lives on `main`
- [x] prove chunk-size independence across at least three chunk sizes
- [x] prove the memory ceiling holds for long sources
- [x] prove dynamic-ratio transitions carry state instead of concatenating
- [x] measure seam artifacts against the Batch 39.1 baseline

| gate | first attempt | now |
| --- | --- | --- |
| `G1` chunk-size independence, static | failed at chunk `1024` | identical across four partitions |
| `G1b` chunk-size independence, dynamic | failed at chunk `2048` | identical across three partitions |
| `G2` memory ceiling | `11665468 B` against `8388608 B` | `8519740 B` against `9437184 B` |
| `G3` output length | passed | passed |
| `G4` correlation against a whole render | `-0.082711` | `1.000000` |

`G4` is the target inherited from `g10.036`, where the segmented path measured
`0.034`. The renderer is bit-identical to a whole render, not merely correlated.

The defect was not emission scheduling. `render` pushed an entire caller chunk
into a fixed input ring, so a `144000`-frame whole-source call against an
`8192`-frame ring overran it seventeen times. That inverts the first reading:
the chunked renders were not wrong, the whole-render reference was the most
corrupted case, and `G3` passing throughout was the clue that scheduling was
always right. `render` and `flush` now feed the ring in capacity-bounded slices.

Fixing `G2` exposed an error in the Batch 39.2 brief, not only in the code. The
inventory counted the overlap-add and normalization rings and omitted the input
ring, so its `8 MiB` ceiling came from an incomplete list. Contract `046` is
corrected to `9 MiB` against a measured `8519740 B`. Duration independence holds
exactly: a `1000`-frame source and a ten-minute source both measure `266300 B`.

No acoustic evidence exists yet, and no production path calls the renderer, so
`segmented_render_matches_whole_render_at_constant_ratio` in
`transparent_correctness_owners` stays `#[ignore]`d: it measures the shipped
segmented path, which Batch 39.4 replaces.

### Batch 39.4 - Render Plane Adoption

Status: rejected by listening and reverted from `main`

Listening found all three adopted specimens silent. Measured RMS: `0.000000`
against roughly `0.1457` for the shipped side. An envelope over the full render
shows audio to `3.8 s` and silence for the remaining `108 s`.

`render` feeds the input ring in capacity-bounded slices; when the output ring
cannot advance, `drain` makes no progress, and the escape added for that case
broke out of the loop and silently dropped the rest of the caller's chunk. The
renderer then padded to its contracted length with zeros, so nothing downstream
noticed.

Five structural gates missed it because every one measured a *relationship*
between renders rather than whether a render contains audio. Chunk-size
independence holds because silence is consistently silent; output length holds
because the renderer pads; correlation holds because both renders share the same
short prefix and the same silence. A renderer emitting nothing but zeros would
have passed four of the five.

`G5` is new and closes that gap: every decile of the output must carry signal.
It reproduces the defect inside the crate suite at `2.5 s`, so no listening
round was needed to catch it. It is `#[ignore]`d with the measured reason while
the defect is open.

The artifact path is reverted to the legacy per-chunk renderer, the behavior
version is back to `signal-stretch-behavior-2026-07-27`, and the adoption
helpers are removed. `ResumableOfflineStretch` stays in the crate, unwired.

The `g10.036` seam pulse and `A18` therefore both remain shipped, and both seam
smoothers stay.

- [x] replace per-chunk stretcher construction with the resumable renderer
- [ ] remove both seam smoothers — measurement says they are still needed; the
  legacy branch still creates the boundaries they patch
- [x] prove artifact output is chunk-independent
- [x] keep the chunk plan as the memory-bounding authority

Sixty seconds of stereo at ratio `1.25` through the artifact path: a `2`-chunk
and a `12`-chunk render are bit-identical, correlation `1.000000`. Batch 39.1
measured `0.389976` on the shipped path.

Two corrections during the batch. The first probe compared the adopted path
against a control that took the `is_single_chunk` branch, so it compared two
algorithms rather than two chunk policies and read as a failure at `0.028056`.
That mistake exposed a real defect in the first wiring: routing only
multi-chunk artifacts through the resumable renderer let **source length select
the algorithm**, so a short and a long artifact of one source would render
differently under one cache key. Every supported configuration now takes the
resumable path regardless of chunk count.

Adoption is partial. The resumable renderer owns the default path with no pitch
shift; selector paths and pitch composition still use the legacy per-chunk path,
which is why both smoothers are retained rather than deleted. Removal moves to
Batch 39.5, conditional on adopting the remaining paths.

`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances to
`signal-stretch-behavior-2026-07-27-resumable`, correctly invalidating earlier
artifacts. The corpus report is unchanged, as expected: it measures the crate's
renderers, not the artifact path.

New finding `A21`: `fake_clocked_soak` is wall-clock dependent in the `A20`
class, failing once under parallel load and passing three times alone.

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

Fix the ring deadlock in `ResumableOfflineStretch` before any further adoption.
`render` must never discard source: if the output ring cannot advance, the
renderer must emit before accepting more input, and the ring sizes must let
emission always progress. Activate `G5` as part of that work, and re-run the
listening round only once `G5` passes.

Batch 39.5 then closes the lane and decides whether the remaining offline paths
adopt the resumable renderer, which is what would let both seam smoothers be
deleted.
