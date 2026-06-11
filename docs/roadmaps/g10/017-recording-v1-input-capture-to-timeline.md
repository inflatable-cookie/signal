# 017 - Recording V1 Input Capture To Timeline

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.016
Vision tags: `RECORDING`, `PRODUCT`

## Problem

Loophole can only import. Pulse's take/arm/commit model (pulse-recording)
is real and waits for audio behind it; cpal input streams were never wired
anywhere. Recording is the largest product unlock left and forces the
latency discipline g10.016 provides.

## Goals

- [ ] input device enumeration + negotiated input streams (mirror the output contract; dedicated owner threads)
- [ ] RT capture path: input callback → lock-free ring → non-RT writer thread → WAV (hound), no allocation in the callback
- [ ] latency-aligned placement: captured audio lands on the timeline compensated by reported input+output latency
- [ ] monitoring: input → output passthrough through the render plane with a monitor gain node
- [ ] pulse wiring: arm/record/commit drives capture sessions; takes become media assets + clips
- [ ] Aura: record button works end-to-end (arm focused track, count-in optional later)

## Execution Plan

### Batch 17.1 - Input Streams

- [ ] input contract + cpal input backend + enumeration

### Batch 17.2 - Capture

- [ ] ring + writer + WAV; latency-aligned placement

### Batch 17.3 - Monitoring And Product Wiring

- [ ] monitor path; pulse arm/commit; Aura record flow

## Acceptance Criteria

- [ ] recorded take plays back aligned with a reference click within a frame tolerance
- [ ] capture callback allocation-free under the counting allocator
- [ ] record→stop→clip-on-timeline works in Loophole

## Next Task

g10.018 (disk streaming) right behind — recordings make long files routine.
