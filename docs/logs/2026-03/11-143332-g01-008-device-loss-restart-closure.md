# 2026-03-11 14:33:32 GMT - g01.008 device-loss/restart closure

## Summary

Finished the remaining `g01.008` diagnostics/failure-handling work by making
device-loss recovery and restart-failure behavior explicit at the host/device
trust edge instead of leaving those paths implied by timeout-style recovery.

This batch completed the last open `008.3` roadmap item and closes
`g01.008`.

## What changed

- added mutable simulated backend diagnostics to
  `crates/signal-hardware-coreaudio/src/lib.rs` so the CoreAudio shell can
  report device disconnect, restart attempt, restart failure, and recovery
  transitions
- added explicit device-loss and restart-failure recovery paths to
  `crates/signal-host-local/src/host.rs`
- made host-local preserve `StopReason::DeviceReconfigure` across device-loss
  handling instead of dropping that stop cause from the final host summary
- fixed the local audio pump fault path so restart-failure reports stay
  `Faulted` instead of being flattened to `Stopped`
- cleared stale pump graph ownership on fault so shared host/runtime reports do
  not imply a healthy current graph transfer after restart failure
- added host report tests for successful device-loss recovery and failed
  restart after device loss

## Validation

Simulated validation completed:

- `cargo test -p signal-hardware-coreaudio`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

Real hardware smoke validation did not run in this tranche. The exercised
device-loss and restart paths are simulation-backed CoreAudio shell scenarios,
not physical device unplug/replug checks.

## Ownership notes

- CoreAudio-specific failure simulation remains owned by
  `signal-hardware-coreaudio` and `signal-host-local`
- runtime remains the authority for transport/control/degradation state; the
  host edge only reports device-backed failure/restart transitions into that
  shared diagnostics surface
- the current CoreAudio backend is still a shell with simulated diagnostics,
  which is acceptable for `g01.008` baseline closure but still leaves real
  hardware restart behavior for later hardening

## Result

`g01.008` is complete. The next active milestone is `g01.009`.
