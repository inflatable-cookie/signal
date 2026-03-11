# 2026-03-11 15:02:13 GMT - g01.009 typed sandbox instance state tranche

## Summary

Advanced `g01.009` through a meaningful `009.2` control-path batch by making
the CLAP/sandbox lifecycle messages carry typed plugin instance state and typed
fault payloads instead of only IDs, epochs, and raw error strings.

This batch does not close `009.2` entirely because the CLAP path is still
fixture-backed for descriptor discovery, but it does move the sandbox control
surface from generic lifecycle supervision toward real plugin-state transport.

## What changed

- expanded `crates/signal-ipc/src/lib.rs` with:
  - `PluginProcessConfigurationPayload`
  - `PluginFaultPayload`
  - `PluginInstanceStatePayload`
- widened plugin IPC lifecycle responses so create/prepare/activate/heartbeat/
  deactivate/reset/destroy messages can carry typed plugin instance state
- widened sandbox failure payloads so they carry a typed fault object instead
  of only `error_kind` and `detail`
- updated `crates/signal-plugin-clap/src/lib.rs` so the sandbox lifecycle
  harness now projects lifecycle/readiness/process-configuration state into
  those responses and emits typed fault payloads on failure events
- updated `crates/signal-plugin-sandbox/src/main.rs` so the sandbox example now
  prints the last lifecycle/readiness state rather than only raw request and
  response names
- added stronger CLAP tests that pin:
  - active lifecycle responses carrying typed instance state
  - sandbox failures carrying typed fault kind/severity

## Validation

- `cargo test -p signal-ipc`
- `cargo test -p signal-plugin-clap`
- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Ownership notes

- `signal-ipc` remains the owner of the wire payload shapes, but those payloads
  now carry plugin-neutral lifecycle/readiness/fault semantics rather than only
  transport metadata
- `signal-plugin-clap` remains the owner of CLAP-specific control handling, but
  now publishes state through the shared typed IPC surface instead of leaving
  later consumers to infer behavior from message names and epochs
- host/runtime crates are still only compile-validated against this schema in
  this tranche; they do not yet consume the richer instance-state payloads for
  reporting or recovery decisions

## Follow-on

The next batch should finish the remaining `009.2` discovery/control work by
replacing the fixture-only CLAP descriptor path with a concrete instance-facing
surface and then teaching host/runtime observation and recovery paths to use
the typed sandbox instance state directly.
