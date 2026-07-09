# 024 - RealtimePreview Stretch Tier

Status: active
Owner: dsp
Created: 2026-07-07
Depends on: g10.014, g10.021, g10.022
Vision tags: `DSP`, `STRETCH`, `REALTIME`

## Problem

Signal has realtime-safe repitch and offline high-quality stretch, but no
pitch-preserving realtime preview tier. Editing workflows need bounded-latency
preview stretch with dynamic ratio changes, but it must not compromise the
render-plane callback contract.

## Goals

- [x] define a bounded-latency preview contract separate from the offline
  whole-buffer engine
- [x] report input latency, output latency, and ratio-change alignment
  tolerance
- [ ] support dynamic ratio changes with bounded work per render quantum
- [x] preserve stereo image and transient timing well enough for edit preview
  in the synthetic preview metric subset
- [ ] prove no allocation, blocking, locks, or unbounded work on the audio
  thread
- [ ] integrate through anticipative pre-rendering or a proven RT-safe state
  object only

## Execution Plan

### Batch 24.1 - Streaming Contract

- [x] latency model, ratio-change alignment contract, and unsupported-mode
  behavior

### Batch 24.2 - Preview DSP Prototype

- [x] bounded-latency pitch-preserving preview algorithm with dynamic ratio
  changes
- [x] preview quality metrics against the corpus subset

### Batch 24.3 - Render-Plane Safety Proof

- [ ] integration only after the state object proves callback-safe behavior
- [ ] no allocation/blocking/lock/unbounded-work test coverage

## Acceptance Criteria

- [x] preview latency is explicit and testable
- [ ] ratio automation lands within the documented tolerance
- [x] preview degradation is honest at extreme ratios
- [x] render-plane realtime safety remains intact

## Validation

- focused `signal-dsp-stretch` streaming tests
- focused render-plane realtime-safety tests if/when the tier enters render
  plans

## Progress

- 2026-07-07: opened as active g10 RealtimePreview planning. Initial execution
  waited for offline evidence and the preview contract.
- 2026-07-09: Batch 24.1 landed in `signal-dsp-stretch`: preview stream
  contract reports input/output latency and ratio-change tolerance, rejects
  invalid stream shapes, and explicitly blocks direct audio-thread processing.
  The RealtimePreview tier is now a prototype with a shorter-window
  transient-reset path for mono, linked stereo, and dynamic-ratio preview.
- 2026-07-09: Batch 24.2 landed synthetic RealtimePreview metrics. The report
  covers timing drift, dynamic tempo-ramp seams, loop boundary clicks, stereo
  image delta, transient smear at extreme ratios, and independent pitch-shift
  error against the draft baseline.

## Next Task

Run Batch 24.3 only when the callback-safe state object is ready to prove no
allocation, blocking, locks, or unbounded work. Until then, keep
RealtimePreview routed through anticipative pre-rendering.
