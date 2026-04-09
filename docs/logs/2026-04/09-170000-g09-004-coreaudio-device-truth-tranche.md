# 2026-04-09 - g09.004 CoreAudio device truth tranche

## Summary

Took the first real CoreAudio depth tranche by replacing the hard-coded default
device shell with bounded real device enumeration and baseline diagnostics in
`signal-hardware-coreaudio`.

## Work Completed

- updated `/crates/signal-hardware-coreaudio/Cargo.toml`
  - added `serde_json` so the backend can consume bounded JSON inventory from
    `system_profiler`
- replaced `/crates/signal-hardware-coreaudio/src/lib.rs`
  - removed the production hard-coded `coreaudio:default-output` device shell
  - added bounded CoreAudio inventory discovery from
    `system_profiler SPAudioDataType -json`
  - normalizes discovered device identity into shared
    `AudioDeviceDescriptor` records
  - derives baseline healthy versus degraded diagnostics from discovered device
    truth instead of assuming a synthetic default output
  - keeps loss/restart simulation methods, but now anchors them to the current
    real default-output device id when one exists
  - added fixture-backed backend tests for healthy inventory, degraded
    inventory, and restart/loss diagnostics
- updated `/crates/signal-host-local/src/host_tests/reports/report_surfaces/boot_and_topology/boot_summary.rs`
  and `/crates/signal-host-local/src/host_tests/reports/report_surfaces/boot_and_topology/topology_reports/host_io.rs`
  - removed hard-coded dependency on the old fake CoreAudio default device id
  - assertions now follow the runtime-owned contract meaning instead of one
    literal synthetic device identity
- updated `/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  - recorded `Batch 4.1 Tranche 1 Outcome`
  - checked the evidence item that actually completed in this tranche

## Validation

Passed:

- `cargo check -p signal-hardware-coreaudio`
- `cargo test -p signal-hardware-coreaudio`
- `cargo check -p signal-host-local`
- `effigy health`

Blocked by pre-existing unrelated host test-tree issues:

- `cargo test -p signal-host-local host_tests::reports::report_surfaces::boot_and_topology::boot_summary::local_host_boot_summary_exposes_negotiated_hardware_contract -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local host_tests::reports::report_surfaces::boot_and_topology::topology_reports::host_io::local_host_shared_report_surfaces_topology_aware_host_io -- --exact --nocapture --test-threads=1`

These still fail before reaching the targeted test bodies because the
`signal-host-local` lib-test tree has pre-existing unresolved split-test module
paths.

Blocked by a separate pre-existing default-boot runtime issue:

- `cargo test -p signal-host-local --test public_host_edge_external_io -- --nocapture --test-threads=1`

That public host-edge lane now fails at `boot_default()` with:
`plugin format Clap is not supported here yet on the local host sandbox path`.
This is a real host-path issue, but not one introduced by the CoreAudio device
inventory change.

## Outcome

The CoreAudio backend is no longer pretending there is always a single fake
default device. The remaining open work in the macOS lane is now clearly at the
host-proof and AU-execution layers: the backend device truth is real enough to
build on, but the stable macOS host proof needs to move off the blocked local
default boot path before `g09.004` can claim an honest AU-plus-device runtime
path.
