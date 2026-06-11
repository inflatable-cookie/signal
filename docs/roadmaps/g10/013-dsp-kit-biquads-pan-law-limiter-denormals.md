# 013 - DSP Kit Biquads Pan Law Limiter Denormals

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.010
Vision tags: `DSP`, `CORRECTNESS`

## Problem

signal-dsp has one filter: a one-pole lowpass. A credible engine needs the
RBJ cookbook biquad family, a soft limiter guarding the master (format-wide, per-channel detection with linked gain), and denormal
protection before any feedback DSP ships (delay lines, reverbs, filter
memories all decay into the denormal range and burn CPU).

## Goals

- [ ] RBJ cookbook biquads in signal-dsp (LP/HP/BP/notch/shelf/peak), coefficient math verified against published known answers; biquad state per channel, channel count from the edge format (a14: no stereo assumption)
- [ ] stateless coefficient calc split from per-node state structs (slotting into g10.011's handoff)
- [ ] soft limiter (lookahead-free soft knee) as a master-node option
- [ ] denormal guard on the callback thread (FTZ/DAZ or DC-offset; measured, documented)
- [ ] known-answer tests: magnitude response at fc/fs points within tolerance for each biquad type

## Execution Plan

### Batch 13.1 - Biquads

- [ ] cookbook family + known-answer tests

### Batch 13.2 - Limiter And Denormals

- [ ] master limiter node; FTZ guard; soak with feedback content

## Acceptance Criteria

- [ ] biquad magnitude responses match cookbook within 0.1 dB at test points
- [ ] master cannot exceed 0 dBFS with limiter engaged
- [ ] denormal guard measurable in a decay-tail benchmark

## Progress (2026-06-11)

- signal-dsp: RBJ cookbook biquads (LP/HP/BP-0dB/notch/peaking/shelves;
  stateless f64 coefficient math + per-channel TDF2 state; known answers
  within 0.1 dB — LP fc reads −3.0103 dB, peaking +6 reads +6.0000, notch
  fc −106 dB); soft limiter (linked max-abs detection, quadratic knee into
  rational saturation asymptotic to 0 dBFS, one-pole release, alloc-free
  per-frame process); DenormalGuard RAII (x86_64 MXCSR FTZ+DAZ, aarch64
  FPCR FZ, restores on drop, !Send).
- Render-plane integration: `RenderPlanSpec.master_limiter:
  Option<RenderLimiterSpec>` — limiter runs on the stream buffer after the
  boundary write, before the transport envelope; its recovery gain inherits
  across plan swaps; DenormalGuard wraps every render_block. Integration
  test proves a 2× hot mix stays ≤ 0 dBFS limited and exceeds it unlimited.
  Soak still zero-alloc. 49 dsp unit tests + 2 doc tests.
- Per a14/a21: no stereo assumptions anywhere — biquad state per channel,
  limiter linked across however many channels the frame carries.

## Next Task

Inserts become possible once plugins or built-in effects exist to fill them.
