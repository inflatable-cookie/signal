# 2026-03-15 21:29:58 UTC - g06.014 Runtime Device Supervision Receipts Tranche

## Summary

Materialized the first shared runtime-owned device supervision receipt family
for `g06.014`. This batch turns the contract from Batch 14.1 into real runtime,
supervisor, and stable host-edge state without reopening host-local restart
policy.

## Work completed

- added `RuntimeDeviceSupervisionSnapshot` and aligned restart or fault-boundary
  enums to `signal-runtime`
- threaded the new snapshot through runtime observation and supervisor report
  surfaces
- enriched `signal-host-local` shared observation and supervisor reports with
  host I/O evidence so device-loss, restart, and exhaustion truth lands on the
  same runtime-owned surface
- added focused runtime and local-host proof coverage for recovered and
  exhausted device-loss episodes
- updated the active roadmap, contract, architecture reference, and generation
  next-task pointers for Batch 14.2

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-runtime runtime_device_supervision_snapshot_ -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_tracks_device_loss_ -- --nocapture`
- `cargo test -p signal-host-server --lib --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- explicit faulted-device consumer proof and broader shared acceptance still
  belong to Batch 14.3
- richer hardware-matrix breadth, clock drift, endpoint topology, and external
  I/O work remain later `g06` lanes

## Next Task

Continue `g06.014` with Batch 14.3 by adding focused proofs for recovery,
exhaustion, and explicit faulted hardware outcomes across shared runtime,
supervisor, and stable host-edge surfaces before widening into later hardware
and media-service queues.
