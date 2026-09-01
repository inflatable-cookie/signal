# Papercuts wave 39 sandbox broker prebuilt contract

Status: closeout
Date: 2026-09-01
Owner: papercuts worker
Handoff: `docs/handoffs/20260901-110725-papercuts-wave39-sandbox-broker-prebuilt-contract.md`
Branch: `worker/papercuts-wave39-sandbox-broker-prebuilt-contract`
Base: `07595bb2` (origin/main at lane start)
Decision: `docs/triage/2026-09-01-sandbox-broker-prebuilt-contract.md` (option 2)
Prior diagnosis (preserved): `docs/logs/2026-08/31-papercuts-wave29-sandbox-broker-consumer-diagnosis.md`
Canonical tracker: Loophole `PAPERCUTS.md` (left open)

## Summary

Implemented the selected option-2 boundary: stable Cargo does not supply the
broker executable; consumers must set `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` to
a compatible prebuilt path before startup. Signal now documents that contract
in the consuming runbook, ships one Effigy/script provisioning entry point that
prints a host-local absolute path, hardens the missing-env diagnostic, and
proves the provisioned binary answers broker wire startup without consumer-side
Cargo.

No empty library / `CARGO_BIN_EXE_*` workaround. No release assets, workflow
edits, Loophole edits, or protocol changes.

## Changed files

- `docs/reference/consuming-signal.md` — prebuilt broker contract section
- `docs/triage/README.md` — index the option-2 decision note
- `effigy.toml` — `broker:provision`, `broker:prove-prebuilt-contract`
- `scripts/provision-sandbox-broker.sh` — explicit host-local provisioner
- `scripts/prove-sandbox-broker-prebuilt-contract.sh` — focused boundary proof
- `crates/signal-runtime/src/sandbox_broker_support/client_session/spawn.rs` —
  actionable missing-env message + runbook/provisioner pointer
- `crates/signal-runtime/src/sandbox_broker_support/tests.rs` — missing-env proof
- `docs/logs/2026-09/01-papercuts-wave39-sandbox-broker-prebuilt-contract.md`
  (this log)

Preserved without rewrite:
`docs/logs/2026-08/31-papercuts-wave29-sandbox-broker-consumer-diagnosis.md`

## Remaining limits

- Provisioned binaries are host- and profile-local (`SIGNAL_BROKER_TARGET_DIR`,
  `SIGNAL_BROKER_PROFILE`); not a cross-machine release asset (option 1 deferred).
- In-tree Signal host tests may still use workspace `cargo run` helpers; that
  is workspace convenience, not the external consumer contract.
- Loophole tracker stays open until a separate consumer revalidation/closeout.

## Validation

```text
effigy broker:prove-prebuilt-contract
# missing-env unit test + provision absolute path + status/shutdown receipts

cargo check -p signal-runtime
git diff --check
effigy qa:docs
```

## Next Task

Orchestrator reviews the worker PR head and merges when the gate passes.
Loophole then revalidates the prebuilt path and closes the cross-repository
tracker only when its acceptance criterion is met. Do not merge from this
worker lane.
