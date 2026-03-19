# 2026-03-17 - g07.009 runtime Linux backend baselines tranche

## Summary

Completed Batch 9.2 of `g07.009` by materializing the first runtime-owned
Linux audio backend baselines across ALSA, JACK, and PipeWire on top of the
bounded contract frozen in Batch 9.1.

This tranche turns Linux backend portability into typed shared hardware and
runtime evidence instead of leaving ALSA, JACK, and PipeWire as future
backend-private host concepts.

## Key changes

- widened `signal-hardware` with:
  - `HardwareBackendIdentity`
  - `LinuxAudioBackendKind`
  - typed backend identity on `AudioDeviceDescriptor`
  - typed backend identity on `BackendPolicyRecord`
- added simulated Linux backend baselines in `signal-hardware` for:
  - ALSA default output
  - JACK duplex graph
  - PipeWire duplex graph
- gave those simulated baselines distinct lifecycle and clock posture so Linux
  backend differences land through one shared hardware contract instead of
  backend-private host glue
- widened `RuntimeHostHardwareSummary` and `RuntimeExternalIoSnapshot` so
  runtime-owned export now carries:
  - Linux backend identity
  - Linux backend portability band
- kept non-Linux hardware explicit as `NotLinux` / `Unsupported` instead of
  silently falling outside the Linux portability surface
- aligned local-host and shared runtime fixtures so the new host-hardware
  shape compiles through the existing observation and supervisor report paths

## Validation

- `cargo fmt --all`
- `cargo test -p signal-hardware -- --nocapture`
- `cargo test -p signal-runtime runtime_host_hardware_summary_classifies_linux_backend_baselines -- --nocapture`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-runtime --test public_contract_boundary --no-run`
- `cargo test -p signal-host-local --lib --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Residual risk

This tranche closes the typed baseline and shared diagnostic classification,
not the consumer-boundary proof seam. Batch 9.3 still needs downstream-style
runtime, supervisor, and stable host-edge proof that Linux backend identity,
portability-band, and fallback receipts remain consumable without backend-
private Linux capability matrices.

## Next Task

Continue `g07.009` with Batch 9.3 by adding focused proofs that the widened
Linux backend identity, portability-band, and fallback receipts remain
consumable through shared runtime, supervisor, and stable host-edge surfaces
without backend-private Linux capability matrices.
