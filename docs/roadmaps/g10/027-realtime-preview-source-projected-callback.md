# 027 - RealtimePreview Source-Projected Callback

Status: active
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

- [ ] define source-projected callback semantics for fixed ratio and dynamic
  ratio streams
- [ ] report source frames consumed, output frames produced, and source/output
  positions per callback quantum
- [ ] prove source advancement follows the active ratio within documented
  tolerance
- [ ] keep all source-projection state preallocated and callback-safe
- [ ] keep render-plane integration blocked until source projection is proven

## Non-Goals

- [ ] no render-plane integration in this roadmap
- [ ] no Loophole or Chorus product workflow planning
- [ ] no whole-buffer preview fallback on the audio callback
- [ ] no claim that RealtimePreview equals OfflineHighQuality quality

## Execution Plan

### Batch 27.1 - Source Projection Contract

- [ ] add a source-projection report shape to `signal-dsp-stretch`
- [ ] define fixed-ratio source advance for `ratio > 1.0`, `ratio == 1.0`,
  and `ratio < 1.0`
- [ ] keep `RealtimePreviewStreamingContract.audio_thread_processing_supported`
  false while the mode is `QuantumLocked`

### Batch 27.2 - Source-Projected State

- [ ] add callback state for fractional source cursors and bounded input
  demand
- [ ] prove deterministic source consumption and output production for fixed
  ratios
- [ ] preserve no-allocation coverage under the counting allocator

### Batch 27.3 - Dynamic Ratio Projection

- [ ] combine scheduled ratio changes with source-projected advancement
- [ ] prove source/output position continuity across ratio changes
- [ ] reassess whether `CallbackSafeStreaming` can be exposed without render
  integration

## Acceptance Criteria

- [ ] fixed-ratio source advance matches the active ratio within tolerance
- [ ] dynamic-ratio source advance stays monotonic and bounded at seams
- [ ] callback reports enough position data for a render plan to remain
  sample-domain honest
- [ ] process still allocates zero bytes after construction
- [ ] support flag remains closed until the source projection contract passes

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

## Next Task

Start Batch 27.1 by adding the source-projection report shape and fixed-ratio
source-advance contract to `signal-dsp-stretch`. Keep
`audio_thread_processing_supported=false` and do not add render-plane
integration.
