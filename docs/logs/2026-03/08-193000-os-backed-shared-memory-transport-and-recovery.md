# 2026-03-08 19:30:00 - OS-Backed Shared Memory Transport And Recovery

Status: complete
Owner: core-product

## Summary

Moved the Signal plugin-sandbox path from in-memory lease placeholders to a real
OS-backed shared-memory transport slice, then bound timeout/crash recovery into
the local and server runtime-host supervisor flow.

This batch adds:

- an OS-backed shared-memory broker in `signal-ipc` using mapped files,
- transport metadata in plugin prepare request/response payloads,
- lease transport binding in `signal-plugin`,
- CLAP lifecycle preparation that allocates brokered regions and sandbox-side
  attachment that validates the negotiated region before activation,
- supervisor-owned sandbox teardown/restart flow in the local and server hosts,
- exercised timeout and crash recovery tests that roll processing to a new
  epoch with a new lease and region.

## Files

- `signal-ipc/Cargo.toml`
- `signal-ipc/src/lib.rs`
- `signal-plugin/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-runtime/src/interfaces.rs`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `signal-plugin-sandbox/src/main.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `cargo check -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-runtime -p signal-host-local -p signal-host-server -p signal-plugin-sandbox`
- `cargo test -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Validation Notes

- The targeted Rust crates passed focused `cargo check` and `cargo test`.
- The host and sandbox smoke runs now report concrete region IDs produced by the
  OS-backed shared-memory broker, confirming the default prepare/activate path
  is no longer in-memory only.
- Recovery coverage is currently host-test driven: timeout and crash sequences
  are exercised in the local and server host tests by tearing down the current
  region, restarting the sandbox, and re-preparing epoch `2`.

## Notes

- The transport is OS-backed via mapped files and carries real region path/id
  metadata through `signal-ipc`, but block data itself is not yet written into
  the brokered region.
- Timeout/crash recovery now exists at the supervisor boundary, but heartbeat,
  destroy/deactivate/reset, and per-block deadline handling still need to be
  bound to the same transport path.

## Next Task

Implement the first brokered block-processing slice: write block headers and
render context into the shared region, add watchdog/heartbeat messages in
`signal-ipc`, and extend the CLAP sandbox path through deactivate/reset/destroy
with deadline-miss handling over the same brokered transport.
