# 2026-03-08 18:45:00 - Shared Memory Lease And Sandbox Fault Flow

Status: complete
Owner: core-product

## Summary

Moved the Signal sandbox path from lifecycle-only prepare records to owned
shared-memory lease tracking and typed sandbox failure events.

This batch adds:

- shared-memory lease ownership and epoch invalidation tracking in
  `signal-plugin`,
- explicit `sandbox.failure` event envelopes in `signal-ipc`,
- prepare/activate payload fields for `processing_epoch` and
  `shared_memory_lease_id`,
- runtime observation support for plugin sandbox fault events in
  `signal-runtime`,
- host-side translation from sandbox failure envelopes into runtime events and
  surfaced lease identifiers,
- stricter CLAP lifecycle harness behavior that invalidates leases on epoch
  mismatch instead of returning untyped string errors.

## Files

- `signal-plugin/src/lib.rs`
- `signal-ipc/src/lib.rs`
- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/runtime.rs`
- `signal-runtime/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-host-local/Cargo.toml`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/Cargo.toml`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `signal-plugin-sandbox/src/main.rs`
- `docs/architecture/package-map.md`

## Validation

- `cargo check -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server -p signal-plugin-sandbox -p signal-runtime`
- `cargo test -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-runtime`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`
- `effigy tasks --repo .`
- `effigy unlock workspace --repo .`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Validation Notes

- The targeted IPC/plugin/runtime crates passed their focused `cargo` checks and
  tests.
- The host and sandbox smoke runs now report concrete lease identifiers,
  confirming the prepare/activate path is carrying real lease metadata.
- `effigy` validation remains the repo-owned baseline for this batch. If a stale
  workspace lock is present, clear it with `effigy unlock workspace --repo .`
  before rerunning the repo suite.

## Notes

- The lease model is still in-memory and process-local. It now gives the
  protocol the right ownership and invalidation semantics, but it is not yet an
  OS-backed shared-memory broker.
- Timeout, crash, and teardown recovery are now represented in the event model,
  but only protocol-violation and epoch-mismatch behavior is exercised in the
  current shell flow.

## Next Task

Implement the first OS-backed transport layer for plugin sandboxes: broker real
shared-memory regions through `signal-ipc`, bind lease rollover and teardown to
runtime supervisor control paths, and add exercised timeout/crash recovery
sequences for the CLAP sandbox path.
