# 027 - RealtimePreview Source-Projected Callback

Status: complete
Owner: dsp
Created: 2026-07-09
Depends on: g10.014, g10.026
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
Vision tags: `DSP`, `STRETCH`, `REALTIME`

## Problem

`g10.026` proved callback-local RealtimePreview DSP: preallocated state,
bounded mono and linked-stereo processing, dynamic-ratio scheduling, seam
evidence, and no allocation under the callback flag.

That still is not render-plane time-stretch playback. The current callback API
is `QuantumLocked`: callers pass one source/input quantum and receive one
output quantum. Ratio changes affect the STFT synthesis path, but the public
contract does not yet own how source media advances when output playback is
slower or faster than source time.

Render-plane use requires `SourceProjected`: the callback state must own or
report ratio-projected source advancement, output position, latency, and
underrun/fill behavior without allocation.

## Goals

- [x] define source-projected callback semantics for dynamic ratio streams
- [x] report fixed-ratio source span, output span, and integer source demand
  for projected output quanta
- [x] prove fixed-ratio source advancement follows the active ratio within
  documented tolerance
- [x] prove stateful source advancement follows the active ratio within
  documented tolerance
- [x] keep all source-projection state preallocated and callback-safe
- [x] keep render-plane integration blocked until source projection is proven

## Non-Goals

- [x] no render-plane integration in this roadmap
- [x] no Loophole or Chorus product workflow planning
- [x] no whole-buffer preview fallback on the audio callback
- [x] no claim that RealtimePreview equals OfflineHighQuality quality

## Execution Plan

### Batch 27.1 - Source Projection Contract

- [x] add a source-projection report shape to `signal-dsp-stretch`
- [x] define fixed-ratio source advance for `ratio > 1.0`, `ratio == 1.0`,
  and `ratio < 1.0`
- [x] keep `RealtimePreviewStreamingContract.audio_thread_processing_supported`
  false while the mode is `QuantumLocked`

### Batch 27.2 - Source-Projected State

- [x] add callback state for fractional source cursors and bounded input
  demand
- [x] prove deterministic source consumption and output production for fixed
  ratios
- [x] preserve no-allocation coverage under the counting allocator

### Batch 27.3 - Dynamic Ratio Projection

- [x] combine scheduled ratio changes with source-projected advancement
- [x] prove source/output position continuity across ratio changes
- [x] reassess whether `CallbackSafeStreaming` can be exposed without render
  integration

## Acceptance Criteria

- [x] fixed-ratio source advance matches the active ratio within tolerance
- [x] dynamic-ratio source advance stays monotonic and bounded at seams
- [x] stateful callback reports enough position data for a render plan to
  remain sample-domain honest
- [x] process still allocates zero bytes after construction
- [x] support flag remains closed until the source projection contract passes

## Validation

- `cargo test -p signal-dsp-stretch realtime_preview_callback`
- focused no-allocation test for source-projected callback processing
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- `effigy qa:docs`

## Progress

- 2026-07-09: opened after `g10.026` completed callback-local DSP proof and
  exposed `QuantumLocked` as the current callback timeline mode. This roadmap
  owns the missing source/output projection contract before any render-plane
  callback support flag can open.
- 2026-07-09: Started Batch 27.1 with a fixed-ratio source projection report
  in `signal-dsp-stretch`. The report maps output start/end frames to
  fractional source start/end frames plus integer source demand for `ratio >
  1.0`, `ratio == 1.0`, `ratio < 1.0`, fractional ratios, and sanitized invalid
  ratios. The stream contract remains `QuantumLocked` and
  `audio_thread_processing_supported=false`.
- 2026-07-09: Completed Batch 27.2 with callback-owned fractional source
  projection cursors, bounded input-demand reporting, deterministic fixed-ratio
  source/output projection tests, reset coverage, and counting-allocator
  coverage for projection calls under the callback flag. The stream contract
  remains `QuantumLocked`; render-plane integration is still blocked.
- 2026-07-09: Completed the dynamic-ratio projection part of Batch 27.3.
  Scheduled source projection now uses callback-owned pending/active ratio
  state, applies changes on the analysis-hop grid, reports output/source seam
  frames, and proves monotonic source/output continuity across tempo-ramp
  changes. The public stream contract remains `QuantumLocked` and
  `audio_thread_processing_supported=false`.
- 2026-07-09: Closed Batch 27.3 reassessment. The proven reporting is not
  enough to expose `CallbackSafeStreaming`: the callback path still lacks an
  owned source-buffer fill and underrun policy for variable input demand. The
  public unsupported reason now points at `SourceBufferingContract`, and
  `g10.028` owns the next Signal implementation batch.

## Next Task

Continue `g10.028` by implementing the source-fill and underrun contract needed
before `RealtimePreview` can expose `CallbackSafeStreaming` or render-plane
integration.
