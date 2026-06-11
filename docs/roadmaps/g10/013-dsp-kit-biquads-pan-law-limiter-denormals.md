# 013 - DSP Kit Biquads Pan Law Limiter Denormals

Status: planned
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

## Next Task

Inserts become possible once plugins or built-in effects exist to fill them.
