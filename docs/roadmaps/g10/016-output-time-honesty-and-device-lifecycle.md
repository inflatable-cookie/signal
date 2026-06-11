# 016 - Output Time Honesty And Device Lifecycle

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.014
Vision tags: `HARDWARE`, `TIME`, `ROBUSTNESS`

## Problem

The playhead counts frames rendered, not played: cpal's OutputCallbackInfo
timestamps are discarded, so output latency is unknown and recording
alignment is impossible. Device disappearance or a default-device change
mid-session leaves a faulted stream the host must notice by accident;
sample-rate changes force a manual replumb.

## Goals

- [ ] consume OutputCallbackInfo: publish a frames-rendered → DAC-time mapping beside the stream clock; expose output latency to hosts
- [ ] device-change/disconnect detection and renegotiation (reopen, recompile at new rate — Aura already has the recompile hook)
- [ ] buffer-size change handling without state loss
- [ ] xrun-injection harness (artificially starve the callback, assert recovery posture)
- [ ] UI playhead optionally DAC-time corrected (cosmetic now, mandatory for recording)

## Execution Plan

### Batch 16.1 - DAC Mapping

- [ ] timestamp plumbing, latency reporting, corrected playhead

### Batch 16.2 - Lifecycle

- [ ] device-change renegotiation, buffer-size changes, xrun injection tests

## Acceptance Criteria

- [ ] reported output latency within measurement tolerance of round-trip estimate
- [ ] yanking the default device recovers playback at the new device's rate
- [ ] xrun injection test green

## Next Task

g10.017 (recording) hard-depends on latency reporting.
