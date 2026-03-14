# 2026-03-08 18:05:00 - Plugin IPC Lifecycle Envelopes And Sandbox Flow

Status: complete
Owner: core-product

## Summary

Moved the Signal plugin-sandbox control plane from typed local DTOs to a real
shared envelope surface in `signal-ipc`, then threaded one coherent CLAP
lifecycle across both host assemblies and the sandbox shell.

This batch adds:

- plugin-domain message names and typed envelopes in `signal-ipc`,
- typed payloads for:
  - `sandbox.handshake`
  - `sandbox.loadPluginType`
  - `sandbox.createInstance`
  - `sandbox.prepareInstance`
  - `sandbox.activateInstance`
- a CLAP lifecycle builder in `signal-plugin-clap` that emits those envelopes,
- a CLAP lifecycle harness in `signal-plugin-clap` that validates order and
  returns correlated responses,
- host startup flow in `signal-host-local` and `signal-host-server` that now
  exercises `handshake -> load -> create -> prepare -> activate`,
- a sandbox shell that consumes the same lifecycle path instead of emitting a
  one-off handshake printout.

## Files

- `signal-ipc/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-plugin-clap/Cargo.toml`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `signal-plugin-sandbox/src/main.rs`
- `signal-plugin-sandbox/Cargo.toml`

## Validation

- `cargo check --workspace`
- `cargo test -p signal-ipc -p signal-plugin -p signal-plugin-clap`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`
- `effigy unlock workspace`
- `effigy health`
- `effigy validate`
- `effigy test`

## Validation Notes

- `cargo test --workspace` is currently not clean because unrelated
  `signal-analysis-rhythm` preset-surface tests fail:
  - `beat_tracker_calibrates_dropout_variant_monotonicity`
  - `beat_tracker_calibrates_harmonic_rhythm_variant_monotonicity`
  - `beat_tracker_calibrates_named_preset_families`
  - `beat_tracker_matches_named_preset_surface_expectations`
- Those failures are outside this engine/protocol batch and were not changed
  here. The touched IPC/plugin/host crates passed their targeted tests.
- `effigy validate` and `effigy test` passed after clearing a
  stale Effigy workspace lock with `effigy unlock workspace`.

## Notes

- The protocol path is now coherent, but it is still envelope-only: prepare
  responses currently confirm shared-memory sizing and epoch values rather than
  allocating owned regions or brokering OS-backed shared memory.
- The CLAP lifecycle harness is intentionally strict about message ordering so
  later process-backed sandbox work can preserve the same semantics.

## Next Task

Implement the first transport-backed sandbox slice by adding plugin-domain
error/failure envelopes in `signal-ipc`, binding them to runtime supervisor
events, and replacing prepare-time sizing records with real shared-memory
ownership and epoch invalidation flow.
