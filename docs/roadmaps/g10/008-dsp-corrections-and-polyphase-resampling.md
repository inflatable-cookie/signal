# 008 - DSP Corrections And Polyphase Resampling

Status: complete
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

- [x] fix `ExponentialRamp` sign handling (support negative targets or
      reject them explicitly)
- [x] polyphase windowed-sinc table module in signal-dsp: precomputed
      (≈16 taps × 512 phases, Kaiser β≈8-10), built control-side, cutoff
      scaled per clip ratio
- [x] render plane consumes the table: replace the lerp inner loop with a
      tap dot-product; table ships inside the compiled plan like sample Arcs;
      zero-alloc soak stays green; linear stays as the cheap fallback tier
- [x] known-answer resampling test: resampled sine SNR above a stated floor
- [x] decide signal-graph's fate explicitly: demote to the telemetry slice
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

## Progress (2026-06-11)

- Batch 8.1: ExponentialRamp documented and enforced as magnitude-domain
  (debug assert on negative targets, bogus copysign line removed; LinearRamp
  is the signed option). DelayLine default-delay `min(1)` typo fixed to the
  full capacity. Denormal threshold comment now states honestly that 1e-20
  also flushes vanishing normals by design.
- Batch 8.2: `PolyphaseInterpolationTable` in signal-dsp (16 taps × 512
  phases, Kaiser β=9, unity-DC-normalised phases, cutoff = min(1, 1/ratio)).
  Render plane builds one table per distinct cutoff at plan compile and the
  Samples render path does a tap dot product for rate-converted clips (1:1
  playback keeps the direct read). Loop wrap via rem_euclid across taps.
  Known-answer gates: kernel test 44.1k→48k sine sinc SNR > 60 dB and
  > linear + 20 dB; render-plane e2e test plays a 44.1k clip on a 48k
  stream and asserts > 60 dB against the analytic sine. Soak still zero
  callback allocations.
- Batch 8.3: signal-graph demoted honestly — crate and ExecutableGraph docs
  now state offline/simulation execution for the control plane, never the
  audio callback; zero "realtime thread" claims remain. The broken stateful
  stages (Delay/LowPass, rebuilt per block so state never survived a block
  boundary) deleted along with their parameters, processors, metrics,
  report fields, and runtime snapshot projections; nothing outside tests
  ever constructed them. Pulse unaffected.
- signal-dsp-resample comparison-report retirement folded into g10.009's
  hygiene pass (crate untouched this packet).

## Next Task

g10.009 (workspace consolidation) after the demolition lanes land.
