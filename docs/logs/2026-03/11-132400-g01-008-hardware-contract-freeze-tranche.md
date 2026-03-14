---
title: g01.008 hardware contract freeze tranche
status: complete
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, hardware, host, roadmap, g01, g01.008]
---

## Summary

Opened `g01.008` with the first trust-edge batch: freeze the shared host/device
contract in `signal-hardware`, provide a simulation seam, and move
`signal-host-local` onto negotiated device/stream metadata instead of
hard-coding output-device details inline.

## What landed

- expanded `signal-hardware` from a minimal policy shell into a reusable
  contract layer for:
  - device enumeration descriptors
  - stream requests and negotiated stream configs
  - lifecycle ownership and restart policy
  - backend diagnostics and negotiation errors
- added `SimulatedHardwareBackend` so host logic can exercise enumeration,
  negotiation, and diagnostics without needing real hardware in every test run
- upgraded `signal-hardware-coreaudio` to expose a default output device and
  negotiate a concrete output stream contract instead of only returning a raw
  sample-rate/buffer tuple
- refactored `signal-host-local` to prepare hardware through that negotiated
  contract, then project the chosen device/stream contract and backend
  diagnostics back out through `LocalRuntimeHostSummary`

## Contract outcome

The host/device boundary is now explicit enough to keep later device callback
work at the trust edge:

- `signal-hardware` owns device descriptors, stream negotiation, lifecycle
  ownership, restart policy, and device diagnostics
- `signal-hardware-coreaudio` owns CoreAudio-flavored default-device and stream
  negotiation behavior
- `signal-host-local` consumes negotiated hardware contracts rather than
  reconstructing them locally
- `signal-runtime` still only receives the runtime-safe subset it needs through
  `HardwareConfigRequest`

## Validation

- `cargo test -p signal-hardware`
- `cargo test -p signal-hardware-coreaudio`
- `cargo test -p signal-host-local`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Next

Take the first `008.2` execution batch: add the initial CoreAudio-backed host
stream path plus bounded buffer-transfer rules in `signal-host-local`, then
prove one topology-aware engine path runs through that boundary without
flattening node/lane/bus structure into callback-local glue.
