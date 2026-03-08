# 2026-03-08 21:00:00 - Runtime Owned Block Sequence Timeline

Status: complete
Owner: core-product

## Summary

Moved sustained block-sequence ownership out of the local/server host recovery
fixtures and into `signal-runtime`.

This batch adds:

- a runtime-owned timeline snapshot surface in `signal-runtime`,
- monotonic block-sequence allocation and continuity aggregation in
  `SignalRuntime`,
- automatic timeline reset on fresh runtime configure,
- local/server host usage of runtime-owned sequence allocation instead of
  host-local counters,
- host summaries that read sequence continuity from runtime snapshots rather
  than duplicated recovery state,
- updated architecture and roadmap docs to reflect the new boundary.

## Files

- `crates/signal-runtime/Cargo.toml`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-server/src/host.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `effigy health --repo .`
- `effigy validate --repo .`

## Validation Notes

- Runtime tests now assert that configure resets the block timeline and that
  timeline continuity tracks lease rollover inside `signal-runtime`.
- Host recovery and soak tests stayed green after removing host-local sequence
  cursors, which confirms the restart and rollover behavior is preserved with
  runtime-owned state.

## Notes

- The next missing piece is not more continuity bookkeeping inside the hosts.
  It is supervisor-facing tooling that can render live soak runs from the real
  host/runtime stack using the shared report surfaces already in `signal-runtime`.

## Next Task

Upgrade the supervisor report example into a live soak-reporting tool that
consumes real host output and runtime timeline state, then expose that report
shape through reusable supervisor fixtures outside the host `main` binaries.
