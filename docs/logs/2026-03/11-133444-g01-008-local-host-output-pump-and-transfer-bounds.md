---
title: g01.008 local host output pump and transfer bounds
status: complete
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, hardware, host, runtime, roadmap, g01, g01.008]
---

## Summary

Completed the first real `008.2` execution slice by adding a host-owned output
pump on top of the negotiated CoreAudio stream contract. `signal-host-local`
now runs runtime engine blocks through an explicit host/device transfer boundary
instead of handing synthetic buffers straight into runtime without a host-side
callback/pump seam.

## What landed

- `signal-host-local` now retains the negotiated `HardwareStreamConfig` as the
  active output stream after boot
- realtime engine processing in `signal-host-local` now flows through a
  host-owned output pump path rather than directly invoking runtime from the
  test harness with no device-transfer layer
- added explicit host-side stream state and transfer policy:
  - `LocalAudioStreamState`
  - `LocalAudioTransferPolicy`
  - `LocalAudioPumpSummary`
- added bounded transfer behavior between runtime output and host/device output:
  - callback frames capped to negotiated stream buffer size
  - transfer channels capped to negotiated output channels
  - unwritten host output samples zero-filled
  - excess runtime output samples counted as dropped rather than copied blindly
- projected the new pump state into `LocalRuntimeHostSummary` and the
  `signal-host-local` CLI summary output

## Contract outcome

- host/device buffer ownership remains at the host edge in `signal-host-local`
  rather than leaking into generic runtime APIs
- runtime still receives an `AudioBuffer` and returns an engine result, while
  the host now owns:
  - callback-frame sizing
  - output-channel bounds
  - zero-fill vs drop accounting
  - running/stopped/faulted stream state
- the current batch proves a real host pump boundary and bounded transfer
  semantics, but it does not yet prove an explicit lane/bus topology shape at
  the host validation level

## Validation

- `cargo test -p signal-host-local`
- `cargo check -p signal-host-local`

## Next

Finish `008.2` with a topology-aware host validation path: run a clearer
track/bus/output or console/output graph shape through the new CoreAudio-backed
pump boundary, then carry that exercised path into the first `008.3`
diagnostics/failure-handling tranche.
