# Runtime Interface Module Mapping

Date: 2026-03-08
Owner: core-product

## Summary

Mapped the runtime-host interface contract and first sandbox protocol pieces
onto real Rust modules inside Signal rather than leaving them as docs-only
surfaces.

## Work completed

- split `signal-runtime` into real interface and runtime modules:
  - typed lifecycle, projection, observation, and supervisor traits
  - runtime DTOs and error types
  - a shell `SignalRuntime` implementation with tests for lifecycle,
    configuration, projection epoching, and safe-mode readiness
- expanded `signal-plugin` with sandbox protocol primitives:
  - shared-memory layout descriptions
  - block-processing header
  - completion-state machine
- expanded `signal-plugin-clap` with:
  - minimum extension-set reporting
  - first CLAP-oriented shared-memory header wrapper
- expanded `signal-hardware` and `signal-hardware-coreaudio` with:
  - backend policy records
  - backend health surface
  - a `HardwareBackend` trait and CoreAudio implementation shell
- updated `signal-host-local` and `signal-host-server` so they implement a
  first host-supervisor shell against the runtime-host supervisor trait

## Validation

- `cargo test --workspace`
- `git diff --check`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`

## Notes

This batch intentionally stops at type and ownership structure. The runtime,
host, and sandbox implementations are still shells, but they are now shells
with real interface contracts and a first completion-state model instead of
loose placeholder printing.

## Next Task

Deepen the runtime-host behavior layer: replace the host-shell supervisor
printing with richer runtime services, define the concrete CLAP block protocol
over the shared-memory layout, and start threading the typed runtime interfaces
through real `signal-runtime` control paths.
