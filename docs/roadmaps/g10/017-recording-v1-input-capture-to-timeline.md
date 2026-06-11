# 017 - Recording V1 Input Capture To Timeline

Status: in-progress (capture + Aura record flow landed; monitoring deferred)
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

- [x] input device enumeration + negotiated input streams (mirror the output contract; dedicated owner threads)
- [x] RT capture path: input callback → lock-free ring → non-RT writer thread → WAV (hound), no allocation in the callback
- [x] latency-aligned placement: captured audio lands on the timeline compensated by reported input+output latency
- [ ] monitoring: input → output passthrough through the render plane with a monitor gain node
- [x] pulse wiring: arm/record drives capture sessions; takes become media assets + clips (host-composite over ToggleTrackArm + ImportMediaAsset/PlaceMediaAssetOnTrack; deeper take/commit vocabulary stays in pulse-recording for later)
- [x] Aura: record button works end-to-end (arm focused track, count-in optional later)

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

## Progress (2026-06-11)

- Input contract (`signal-hardware/src/input_stream.rs`) mirrors the output
  contract exactly: `InputStreamSpec`/`InputCaptureFn`/`InputStreamHandle`
  (negotiated rate/channels, `input_latency_micros`, `device_name`,
  `last_error`)/`InputStreamBackend`. `CpalInputBackend` +
  `enumerate_input_devices()` live in signal-hardware-output-cpal (crate
  name now noted as historical; rename is a later hygiene item) with the
  same owner-thread, negotiation, and latency-atomic patterns as output.
  Measured ~5.3 ms input latency on the MacBook Pro microphone.
- Capture path (`signal-hardware/src/capture.rs`): `SpscRing` (power-of-two
  f32 ring, atomics only, drop-and-count overruns, never blocks) +
  `CaptureSession` (callback only pushes into the ring; writer thread
  drains to a Float32 WAV at the negotiated rate; `stop()` drops the
  stream, drains fully, finalizes, returns `CaptureReport`).
  `FakeInputBackend` synthesizes a 440 Hz sine for CI; e2e test verifies
  frame count, RMS, and zero-crossing rate; a counting-allocator
  integration test proves the callback path allocates nothing.
- Aura record flow: `services/recording.rs` owns the capture session
  (separate from playback); `aura_start_recording` anchors the take at the
  transport position on the armed track and rolls the transport if
  stopped; `aura_stop_recording` imports the WAV and places it at
  `record_start - (input+output latency in samples)` (saturating at 0,
  unit-tested). Record/Stop button in the TimelinePanel toolbar, enabled
  when a track is armed.
- Deferred (named follow-ups): live input monitoring through the render
  plane (input ring drained by a live-input stage + monitor gain node) was
  explicitly out of scope for this execution; count-in belongs to g10.019;
  acceptance check "take aligns with a reference click within a frame
  tolerance" still owed as a hardware listening test.

## Progress (2026-06-11)

- Input contract mirrors the output contract exactly (negotiated honesty,
  latency from cpal timestamps, device identity); CpalInputBackend reuses
  the owner-thread + negotiation patterns; input enumeration added.
  Measured input latency ~5.3 ms (MacBook Pro Microphone). The
  signal-hardware-output-cpal crate name is recorded as historical; rename
  is a hygiene item.
- SpscRing (power-of-two, acquire/release atomics, drop-and-count overrun
  semantics — the callback never blocks) + CaptureSession (callback pushes
  only; writer thread drains to Float32 WAV at the negotiated config; stop
  drains fully and reports frames/overruns/latency). FakeInputBackend
  synthesizes a known sine for CI. Counting-allocator test proves the
  capture callback body does zero allocations.
- Latency-aligned placement: aligned = max(0, record_start − (input+output
  latency in frames)); unit-tested values recorded. Aura recording service
  owns its own input stream apart from playback; aura_start_recording
  anchors at the transport position, requires an armed track (pulse's
  recording model drives arming), starts transport if stopped;
  aura_stop_recording imports + places the take on the armed track at the
  aligned position and leaves transport running. Record button in the
  timeline toolbar, enabled when armed.
- Deferred: live input monitoring through the render plane (named
  follow-up), count-in (g10.019), click-alignment hardware listening test.

## Next Task

g10.018 (disk streaming) right behind — recordings make long files routine.
