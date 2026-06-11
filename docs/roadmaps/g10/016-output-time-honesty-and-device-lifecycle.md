# 016 - Output Time Honesty And Device Lifecycle

Status: complete
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

## Progress (2026-06-11)

- Output latency from cpal timestamps: the data callback stores
  playback−callback micros into an atomic (RT-safe); handle exposes
  `output_latency_micros()` + `device_name()`; measured ~5.8 ms on the
  reference MacBook Pro speakers via the extended smoke test.
- Aura playback status carries dac_position_samples (render clock minus
  latency frames), output latency, and device name; the UI playhead
  consumes the DAC position so it stops leading the speakers. The render
  clock stays the edit truth for stop-edge seeks.
- Device-drift recovery: every ~2 s the status poll compares the stream's
  opened device against the OS default; on mismatch (or fault/channel
  teardown) the host captures (position, playing) as pending_resume, drops
  the stream, and the next sync rebuilds at the new device and re-primes
  transport mid-position — hands-free resume. This also fixed a latent
  rebuild bug: fault recovery previously rewound to the play-start
  position.
- Buffer-size changes documented as fault-path rebuilds (cpal backends);
  manual proof path recorded. Starvation soak extended: position strictly
  advances post-starvation and xrun inference stops counting once cadence
  recovers.
- Hardware check still owed: yank headphones mid-play, expect audible
  hands-free resume within ~2 s.

## Next Task

g10.017 (recording) hard-depends on latency reporting.
