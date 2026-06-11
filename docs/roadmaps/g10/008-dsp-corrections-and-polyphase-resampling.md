# 008 - DSP Corrections And Polyphase Resampling

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.002
Vision tags: `DSP`, `RESAMPLING`, `CORRECTNESS`

## Problem

Three correctness items and one quality upgrade. (1) signal-dsp's
`ExponentialRamp::set_target` silently flips negative targets positive
(`signum().max(1.0)` is always 1.0). (2) signal-graph rebuilds stage state
every block — its Delay and LowPass stages lose their buffers/state at every
block boundary, and its "realtime" path allocates per block despite docs
claiming RT use; nothing production-audio consumes it (telemetry for Pulse
only). (3) signal-dsp-resample is a correct but naive windowed-sinc with a
Vec-returning, per-tap-`sin()` API unusable on an RT thread. (4) The render
plane's `Samples` source uses two-point linear interpolation — audible
aliasing on rate-mismatched media.

## Goals

- [ ] fix `ExponentialRamp` sign handling (support negative targets or
      reject them explicitly)
- [ ] polyphase windowed-sinc table module in signal-dsp: precomputed
      (≈16 taps × 512 phases, Kaiser β≈8-10), built control-side, cutoff
      scaled per clip ratio
- [ ] render plane consumes the table: replace the lerp inner loop with a
      tap dot-product; table ships inside the compiled plan like sample Arcs;
      zero-alloc soak stays green; linear stays as the cheap fallback tier
- [ ] known-answer resampling test: resampled sine SNR above a stated floor
- [ ] decide signal-graph's fate explicitly: demote to the telemetry slice
      Pulse consumes (documented as non-RT, offline-only) or delete after
      decoupling Pulse; fix or delete the broken stateful stages — no
      half-state
- [ ] retire signal-dsp-resample's comparison-report ceremony; share the
      sinc kernel math with the new table module so it exists once

## Non-Goals

- [ ] no full sample-rate-conversion service (disk streaming, time-stretch
      stay backlog)
- [ ] no signal-graph successor design — that is a rebuild item, designed
      around the render plane when a product feature needs graphs

## Execution Plan

### Batch 8.1 - Small Fixes

- [ ] ExponentialRamp fix + tests; DelayLine `min(1)` default surprise; flush
      threshold comment honesty

### Batch 8.2 - Polyphase Table

- [ ] table builder + storage in compiled plan; render-path tap loop; soak +
      SNR known-answer tests

### Batch 8.3 - Graph Disposition

- [ ] inventory exactly which graph snapshots Pulse reads; demote or delete
      accordingly; remove "realtime thread" claims from docs either way

## Acceptance Criteria

- [ ] negative-target ramp behaves as documented
- [ ] 44.1k→48k sine through the render plane beats linear interpolation by a
      measured SNR margin recorded in the log
- [ ] zero-alloc soak green with the polyphase path active
- [ ] signal-graph either consumed-and-honest or gone

## Risks and Mitigations

- Risk: tap loop cost at many simultaneous rate-converted clips.
- Mitigation: per-clip quality tier (linear for previews), measure in soak.

## Evidence Requirements

- [ ] SNR figures and soak output in the progress log

## Next Task

g10.009 (workspace consolidation) after the demolition lanes land.
