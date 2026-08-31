# Papercuts wave 29 sandbox broker consumer — diagnosis

Status: diagnosis-only (no repair PR)
Date: 2026-08-31
Owner: papercuts worker
Handoff: `docs/handoffs/20260831-220404-papercuts-wave29-sandbox-broker-consumer.md`
Branch: `worker/papercuts-wave29-sandbox-broker-consumer`
Planning base: `eecd0605009e62f8dbe0f1a0403162b592a09a92`
Canonical tracker: Loophole `PAPERCUTS.md` (open; do not close here)

## Summary

Confirmed the Signal-side source of “Signal sandbox broker binary is not
consumable as a Cargo dependency.” `signal-plugin-sandbox` is bin-only.
An out-of-tree consumer that path-depends on it gets Cargo’s
`ignoring invalid dependency … missing a lib target` warning and never
builds the broker binary.

The tracker’s proposed fix — add a lib target (even empty) with a
`binary_path()` helper so `CARGO_BIN_EXE_*` / path resolution works — does
**not** create a stable executable boundary for dependents. Proved with
minimal fixtures on `cargo 1.97.1` / `rustc 1.97.1`.

No Signal-owned repair lands in this lane without widening into unstable
Cargo artifact deps (`bindeps`) or a packaging/release redesign. Env-var
escape hatch and consumer on-demand `cargo build` remain the working paths.
Loophole tracker stays open.

## Reproduction

### A. Current package is bin-only

`crates/signal-plugin-sandbox/Cargo.toml` has no `[lib]` / `src/lib.rs`;
only `src/main.rs`. In-tree tests use `env!("CARGO_BIN_EXE_signal-plugin-sandbox")`
(own-package only). Host / runtime consumers use
`SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` (or `cargo run -p signal-plugin-sandbox`
in workspace tests). External consumer
`loophole/crates/pulse-signal-link/tests/plugin_isolation.rs` either reads
that env var or builds into `target/signal-broker` from the Signal checkout.

### B. Real out-of-tree dependency (isolated workspace)

```toml
[package]
name = "real-consumer"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
signal-plugin-sandbox = { path = "<signal>/crates/signal-plugin-sandbox" }
```

```text
warning: real-consumer … ignoring invalid dependency `signal-plugin-sandbox`
  which is missing a lib target
Finished `dev` profile … (broker package never compiled)
```

### C. Empty lib does not expose the binary

Minimal workspace: `broker` with `src/lib.rs` + `src/main.rs`; `consumer`
depends on `broker`.

| Observation | Result |
| --- | --- |
| Cargo accepts the dependency | yes (no “ignoring invalid dependency”) |
| `CARGO_BIN_EXE_broker` in consumer tests | `None` |
| `target/debug/broker` after `cargo test -p consumer` | absent |
| Only the lib artifact is built for the dep | yes |

### D. Lib helper re-exporting `option_env!("CARGO_BIN_EXE_…")`

- Own-package integration test: `CARGO_BIN_EXE_broker` set; binary exists.
- Same helper compiled into a dependent: still `None` at consumer test time
  (`env!`/`option_env!` bind when the **lib** unit is compiled, not when the
  consumer’s tests run, and dependents do not get dep-bin env vars).

### E. Artifact / bindeps

```text
unknown Cargo.toml feature `bindeps`
This feature can be enabled via -Zbindeps or the `[unstable]` section …
```

Stable Cargo has no supported “depend on this package’s binary and get a
path” mechanism. Unstable `bindeps` is a packaging redesign outside this
lane.

## Why no repair PR

A valid repair must let an external consumer obtain a broker executable
**without** an on-demand source-checkout build, while preserving wire
protocol, process lifecycle, realtime guarantees, and the env-var escape
hatch.

| Candidate | Verdict |
| --- | --- |
| Empty/`binary_path()` lib target | Silences the invalid-dep warning only; does not build or path the binary |
| Runtime helper that shells `cargo build` | Still on-demand compile; same friction the tracker reports |
| Workspace pre-build script | Local convenience, not a Cargo dependency boundary for git-tag consumers |
| Unstable artifact deps / shipping prebuilts | Packaging or release redesign; out of handoff scope |

Therefore: diagnosis only; no claim of a Cargo consumer fix; no PR against
`main`.

## Changed files

- `docs/logs/2026-08/31-papercuts-wave29-sandbox-broker-consumer-diagnosis.md`
  (this log)

No production code, package shape, or Loophole/Pulse edits.

## Validation

```text
# Fixture proofs (temp workspaces; cargo 1.97.1):
# - bin-only dep → ignoring invalid dependency; CARGO_BIN_EXE None
# - lib+bin dep → lib only; no target/debug/broker; CARGO_BIN_EXE None
# - own-package test → CARGO_BIN_EXE set and path exists
# - lib helper option_env in consumer → None
# - real path dep on signal-plugin-sandbox → ignoring invalid dependency

git diff --check
# clean (log-only)

effigy qa:docs
# run at closeout
```

## Next Task

Orchestrator reviews this diagnosis head. Keep the Loophole tracker open.
Any real fix needs an explicit packaging decision (stable artifact story or
distributed broker binary), not a lib-target papercut. Do not merge a
cosmetic lib-only change from this worker lane.
