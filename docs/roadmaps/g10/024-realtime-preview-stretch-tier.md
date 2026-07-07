# 024 - RealtimePreview Stretch Tier

Status: planned
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

- [ ] define a bounded-latency streaming stretcher state separate from the
  offline whole-buffer engine
- [ ] report input latency, output latency, and ratio-change alignment
  tolerance
- [ ] support dynamic ratio changes with bounded work per render quantum
- [ ] preserve stereo image and transient timing well enough for edit preview
- [ ] prove no allocation, blocking, locks, or unbounded work on the audio
  thread
- [ ] integrate through anticipative pre-rendering or a proven RT-safe state
  object only

## Execution Plan

### Batch 24.1 - Streaming Contract

- [ ] state object, latency model, ratio-change alignment contract, and
  unsupported-mode behavior

### Batch 24.2 - Preview DSP Prototype

- [ ] bounded-latency pitch-preserving preview algorithm with dynamic ratio
  changes
- [ ] preview quality metrics against the corpus subset

### Batch 24.3 - Render-Plane Safety Proof

- [ ] integration only after the state object proves callback-safe behavior
- [ ] no allocation/blocking/lock/unbounded-work test coverage

## Acceptance Criteria

- [ ] preview latency is explicit and testable
- [ ] ratio automation lands within the documented tolerance
- [ ] preview degradation is honest at extreme ratios
- [ ] render-plane realtime safety remains intact

## Validation

- focused `signal-dsp-stretch` streaming tests
- focused render-plane realtime-safety tests if/when the tier enters render
  plans

## Progress

- 2026-07-07: opened as active g10 RealtimePreview planning. This remains
  planned until offline evidence and the streaming contract are ready.

## Next Task

Do not start this before g10.021 and the first g10.022 DSP quality pass clarify
the preview quality target.
