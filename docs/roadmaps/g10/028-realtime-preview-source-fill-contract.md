# 028 - RealtimePreview Source Fill Contract

Status: resolved through `g10.040`

Resolved 2026-08-05. The source fill contract this roadmap was to define is
frozen in `g10.040` Batch 40.2 and implemented in Batch 40.3: a non-realtime
producer fills an SPSC ring, the callback consumes `block / ratio` frames from
it, demand is published as one atomic frame counter, and underrun emits silence
with a reported shortfall rather than a block indistinguishable from a healthy
one.

The prefill and latency numbers are `ceil(block / ratio_min) * 2 + window_size`
frames, reported as `window_size + prefill`.
Owner: dsp
Created: 2026-07-09
Depends on: g10.026, g10.027
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
Vision tags: `DSP`, `STRETCH`, `REALTIME`

## Problem

`g10.027` proved source-projection reporting for fixed and dynamic ratios, but
the callback path still does not own how projected source demand is filled.
`RealtimePreviewCallbackState::process` remains `QuantumLocked`: one caller
input block produces one output block.

Render-plane use needs a stronger contract. The callback state must report or
own bounded source input demand, source-buffer readiness, underrun policy, and
fill behavior without allocation before `CallbackSafeStreaming` can open.

## Goals

- [ ] define bounded source-fill semantics for projected output quanta
- [ ] add a callback-safe source-demand/fill report
- [ ] prove underrun behavior is deterministic and click-bounded
- [ ] prove no allocation for source-fill bookkeeping under the callback flag
- [ ] keep render-plane integration blocked until fill behavior passes

## Non-Goals

- [ ] no render-plane integration in this roadmap
- [ ] no Loophole or Chorus product workflow planning
- [ ] no quality claim that RealtimePreview equals OfflineHighQuality
- [ ] no whole-buffer preview fallback on the audio callback

## Execution Plan

### Batch 28.1 - Fill Contract Shape

- [ ] define source-fill readiness states for `Ready`, `Partial`, and
  `Underrun`
- [ ] report projected source range, available source range, missing frames,
  fill strategy, and output silence/declick range
- [ ] keep `RealtimePreviewStreamingContract.audio_thread_processing_supported`
  false

### Batch 28.2 - Callback-Safe Fill State

- [ ] add preallocated source-fill bookkeeping to `RealtimePreviewCallbackState`
- [ ] prove fixed-ratio source demand can be satisfied without allocation
- [ ] prove underrun/fill reports remain monotonic under partial source input

### Batch 28.3 - Exposure Reassessment

- [ ] combine scheduled source projection with fill readiness
- [ ] prove dynamic-ratio fill continuity across ratio-change seams
- [ ] decide whether `CallbackSafeStreaming`/`SourceProjected` can open or
  whether render-plane adapter work needs one more roadmap

## Acceptance Criteria

- [ ] source-fill reports are deterministic for fixed and dynamic ratios
- [ ] underrun behavior is bounded, explicit, and click-controlled
- [ ] callback fill bookkeeping allocates zero bytes after construction
- [ ] support flag remains closed until both projection and fill pass together
- [ ] next integration boundary is explicit: open callback streaming or defer
  to a render-plane adapter roadmap

## Validation

- `cargo test -p signal-dsp-stretch realtime_preview_source_fill`
- `cargo test -p signal-dsp-stretch realtime_preview_callback`
- focused no-allocation test for source-fill bookkeeping
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- `effigy qa:docs`

## Progress

- 2026-07-09: Opened after `g10.027` closed source projection but kept
  `audio_thread_processing_supported=false`. This roadmap owns the missing
  source-fill and underrun behavior before callback-safe render-plane exposure.
- 2026-07-09: Paused after correctness audit. Source projection is a separate
  arithmetic state machine and is not coupled to the callback kernel's actual
  input consumption. Resume only after `g10.029` establishes trustworthy DSP
  boundary behavior, promotion gates, and streaming geometry.

## Next Task

Do not start Batch 28.1. Continue `g10.029`; reassess this roadmap after the
kernel and source-consumption contract are stable. Keep
`audio_thread_processing_supported=false`.
