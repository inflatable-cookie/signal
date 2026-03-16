# 2026-03-15 23:35:09 UTC - g06.015 Runtime Clock And Topology Receipts Tranche

## Summary

Completed Batch 15.2 of `g06.015` by materializing runtime-owned drift,
discontinuity, duplex-mismatch, endpoint-topology, and partial-availability
receipts on the shared host clocking seam. This batch also removed the local
host's duplicate `host_io` recomputation so the stable host edge and embedded
supervisor observation now share the same first-observation transition truth.

## Work completed

- widened `signal-runtime` host clocking and external-I/O DTOs with:
  - drift-state classification
  - discontinuity-state classification
  - duplex-mismatch classification
  - endpoint-topology classification
  - partial-availability visibility
- updated `signal-host-local` to derive those fields in one runtime-owned
  projection path and reuse one `host_io` receipt per outward report
- added focused runtime and local-host proofs for:
  - same-clock output-only steady state
  - cross-clock fallback
  - aggregate-clock topology
  - duplex cross-clock mismatch
  - duplex partial availability
  - faulted recovery-constrained hardware state
  - return-to-direct continuity
- marked Batch 15.2 complete in:
  - `docs/roadmaps/g06/015-clock-domain-drift-duplex-mismatch-and-endpoint-topology-depth.md`
- extended the active contract and architecture reference for the new receipt
  family

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_external_io_snapshot_ -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_ -- --nocapture`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- this batch establishes runtime-owned receipt depth, not the final shared
  proof boundary
- richer endpoint-role detail, monitoring or loopback topology semantics, and
  network-audio scope remain later work

## Next Task

Continue `g06.015` with Batch 15.3 by adding focused proofs for drift, duplex
mismatch, and endpoint-topology observation across shared runtime, supervisor,
and stable host-edge surfaces before later external-I/O work widens.
