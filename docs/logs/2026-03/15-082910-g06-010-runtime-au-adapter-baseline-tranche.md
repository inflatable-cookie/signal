# 2026-03-15 08:29:10 UTC - g06.010 Runtime AU Adapter Baseline Tranche

## Summary

Landed the first real runtime-owned AU adapter baseline by adding a bounded
`signal-plugin-au` crate and wiring both host paths to feed AU discovery and
lifecycle truth back into the shared runtime-owned receipt family.

## Work completed

- added the new `signal-plugin-au` crate with:
  - macOS component-root presets
  - Audio Unit component identity projection
  - bounded shared-memory session planning
- wired `signal-host-local` to:
  - export AU discovery through runtime-owned scan receipts
  - record AU sandbox lifecycle, instance-state, and transport attachment
- wired `signal-host-server` to the same AU runtime-owned discovery and
  lifecycle seam for the shared headless host path
- added focused AU adapter and host proof coverage
- updated the roadmap, contract, and reference surfaces to move the active queue
  to AU conformance proof in Batch 10.3

## Validation

- `cargo fmt --all`
- `cargo test -p signal-plugin-au -- --nocapture`
- `cargo test -p signal-host-local local_host_au_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_au_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`

## Deferred scope

- no public downstream-style AU conformance proof exists yet
- richer AU parameter-tree, preset, editor, and event-model depth remains later
  cross-adapter work

## Next Task

Continue `g06.010` with Batch 10.3 by proving the AU path remains consumable
through shared runtime, supervisor, and stable host-edge surfaces without
private host glue or adapter-local reconstruction.
