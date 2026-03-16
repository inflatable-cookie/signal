# 2026-03-16 08:57:21 UTC - g06.016 Runtime Monitoring And Loopback Receipts Tranche

## Summary

Materialized the first runtime-owned external-I/O monitoring, tap-point, and
loopback receipt family for `g06.016`. The runtime now exports explicit
monitoring and loopback meaning through the shared `RuntimeExternalIoSnapshot`
surface, with local and server hosts aligned to that same model instead of
reconstructing monitor-path state independently.

## Work completed

- widened `RuntimeExternalIoSnapshot` with explicit:
  - external-I/O health
  - device-change state
  - primary role
  - monitoring state
  - monitoring tap point
  - loopback state
- added `Unavailable` classifications so runtime observation can carry a
  bounded external-I/O receipt even when no live host-I/O monitoring seam is
  available
- threaded the shared snapshot through `RuntimeObservationReport` and related
  observation rendering or JSON export
- updated `signal-host-local` to feed one live host-I/O summary into both
  device-supervision and external-I/O observation surfaces
- kept `signal-host-server` aligned by exporting the same runtime-owned
  snapshot shape with explicit unavailable monitoring or loopback state rather
  than a server-private model
- extended focused runtime and host proofs around:
  - clock-fallback and recovery classification
  - duplex and endpoint-topology classification
  - explicit unavailable server-host monitoring state

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_external_io_snapshot_ -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_topology_aware_host_io -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_duplex_ -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_tracks_device_loss_restart_failure -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_unavailable_external_io_monitoring_state -- --nocapture`
- `cargo test -p signal-host-server --lib --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Deferred scope

- the downstream proof boundary for monitoring, tap-point, and loopback
  receipts still belongs to Batch 16.3
- the stable host-edge side of this boundary remains local-host centric until a
  broader live server-host hardware seam is promoted
- calibration workflows, waveform analysis, and richer loopback measurement
  algorithms remain outside this batch

## Next Task

Continue `g06.016` with Batch 16.3 by adding focused consumer-facing proofs
that monitoring, tap-point, and loopback receipts remain consumable through
shared runtime, supervisor, and stable host-edge surfaces without host-local
monitor reconstruction.
