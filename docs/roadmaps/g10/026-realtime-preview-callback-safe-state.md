# 026 - RealtimePreview Callback-Safe State

Status: active
Owner: dsp
Created: 2026-07-09
Depends on: g10.014, g10.024
Vision tags: `DSP`, `STRETCH`, `REALTIME`

## Problem

`g10.024` produced a control-side RealtimePreview prototype and metric harness,
but direct render-plane integration remains blocked. The current prototype
allocates and processes whole preview buffers. A true callback path needs a
preallocated state object with bounded per-quantum work, explicit latency, and
ratio-change alignment proof before any render-plane plan can use it.

## Goals

- [x] define the callback-safe RealtimePreview state contract and reset rules
- [x] preallocate all scratch, rings, windows, spectra, and output buffers
  outside the audio callback
- [ ] process bounded render quanta with no allocation, locks, blocking, I/O,
  or unbounded loops
- [ ] apply ratio changes through a documented sample-domain alignment
  tolerance
- [ ] preserve the existing prototype metric surface as the preview quality
  baseline
- [x] keep render-plane integration blocked until the state object passes the
  RT-safety proof

## Non-Goals

- [ ] no render-plane integration before the state object proves callback-safe
  behavior
- [ ] no whole-buffer fallback on the audio callback
- [ ] no Chorus or Loophole integration planning in this roadmap
- [ ] no claim that preview quality matches OfflineHighQuality or Rubber Band

## Execution Plan

### Batch 26.1 - State Contract And Allocation Proof Harness

- [x] define config, state, reset, latency, supported channel count, and
  unsupported-mode errors
- [x] add a focused no-allocation test harness around the callback-facing
  process method
- [x] keep `RealtimePreviewStreamingContract` reporting
  `audio_thread_processing_supported=false` until the process method passes
  the proof

### Batch 26.2 - Streaming DSP State

- [ ] implement bounded mono streaming state with preallocated STFT/ring
  scratch
- [ ] add linked-stereo state only after mono proves bounded work
- [ ] prove deterministic output length, bounded latency, and no per-block
  allocation for fixed ratios

### Batch 26.3 - Dynamic Ratio And Render-Plane Gate

- [ ] add ratio-change scheduling with documented alignment tolerance
- [ ] prove dynamic-ratio timing and seam behavior against the preview corpus
  subset
- [ ] flip callback support and add render-plane integration only after the
  callback-safe state object passes no-allocation/no-lock/no-blocking coverage

## Acceptance Criteria

- [x] callback-facing process method allocates zero bytes after construction
- [ ] process method does not lock, block, perform I/O, or loop over unbounded
  input
- [ ] fixed-ratio output stays deterministic and within latency contract
- [ ] dynamic-ratio changes land within documented tolerance
- [ ] `RealtimePreviewStreamingContract` only reports callback support when
  the implementation and tests prove it
- [ ] render-plane realtime safety remains intact

## Validation

- `cargo test -p signal-dsp-stretch realtime_preview`
- focused no-allocation tests for the callback-facing process path
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- focused render-plane realtime-safety tests only if Batch 26.3 reaches
  render-plane integration

## Progress

- 2026-07-09: opened as the structural continuation of `g10.024`. This is the
  only ready RealtimePreview continuation; more prototype metric tweaks stay
  out of scope.
- 2026-07-09: Batch 26.1 landed the callback-state contract shell in
  `signal-dsp-stretch`. The state validates stream config and callback block
  geometry, owns preallocated scratch, supports reset, and has a
  counting-allocator proof that the callback-facing `process` path allocates
  zero bytes. Processing still returns explicit unsupported status, and
  `RealtimePreviewStreamingContract` still reports callback processing
  unsupported.
- 2026-07-09: Started Batch 26.2 without enabling callback DSP. The callback
  state now preallocates input/output/normalization rings, window coefficients,
  per-channel spectral buffers, phase state, and FFT plans during construction.
  The allocation proof now covers repeated callback-facing process attempts and
  resets. Actual streaming phase-vocoder processing remains the next step.

## Next Task

Continue Batch 26.2 by implementing the mono streaming phase-vocoder process
loop against the preallocated state. Do not add render-plane integration and do
not run the whole-buffer prototype on the audio callback.
