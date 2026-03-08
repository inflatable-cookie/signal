# 2026-03-08 17:05:00 - Runtime Host Services And CLAP Block Protocol

Status: complete
Owner: core-product

## Summary

Deepened the Signal trust-edge/runtime batch beyond typed shells:

- `signal-runtime` now carries real host-facing control state for hardware
  projection, applied projections, diagnostics, backend policy, and event
  emission.
- `signal-plugin` now defines a concrete minimum sandbox control plane and data
  plane surface:
  - sandbox control commands and responses,
  - shared-memory layout sizing,
  - render-context and block-dispatch DTOs,
  - stricter completion state-machine transitions.
- `signal-plugin-clap` now shapes that generic sandbox surface into a first
  CLAP block protocol with:
  - explicit minimum extension set,
  - advertised capabilities,
  - prepare-plan generation,
  - shared-memory block-header construction.
- `signal-host-local` and `signal-host-server` now use reusable runtime-host
  service structs instead of owning inline placeholder supervision in `main.rs`.
- `signal-plugin-sandbox` now reports the same handshake/control vocabulary as
  the library types rather than a stale one-off placeholder.

## Files

- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/runtime.rs`
- `signal-plugin/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `signal-plugin-sandbox/src/main.rs`

## Validation

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Notes

- This batch still stops at contract-level orchestration. The host services now
  speak stable typed control and block DTOs, but they do not yet drive a real
  plugin instance lifecycle or shared-memory transport implementation.
- The next useful step is to encode the control-plane messages in `signal-ipc`
  and use them to drive a real sandbox prepare/activate/reset flow against the
  new block protocol.

## Next Task

Implement the first real sandbox lifecycle slice by defining `signal-ipc`
plugin-domain envelopes for handshake/load/create/prepare/activate, then thread
those messages through `signal-host-local`, `signal-host-server`, and
`signal-plugin-sandbox` so the CLAP protocol is exercised by one coherent
control path.
