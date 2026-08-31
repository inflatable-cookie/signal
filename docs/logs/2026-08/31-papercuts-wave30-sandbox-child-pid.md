# Papercuts wave 30 — sandbox broker child PID

Status: closeout
Date: 2026-08-31
Owner: papercuts worker
Handoff: `docs/handoffs/20260831-221016-papercuts-wave30-sandbox-child-pid.md`
Branch: `worker/papercuts-wave30-sandbox-child-pid`

## Summary

`SandboxBrokerClientSession` already owned the spawned `std::process::Child`
but offered no public PID accessor. Hosts needed an external `ps` probe for
crash evidence and `sandbox_pid` reporting.

Added `SandboxBrokerClientSession::child_pid() -> u32`, a direct read of
`Child::id()` on the owned child. No lifecycle, wire, or protocol change.
Loophole `PAPERCUTS.md` tracker entry left open for orchestrator closeout
after merge and downstream proof.

## Files

- `crates/signal-runtime/src/sandbox_broker_support/client_session/session.rs`
  — public `child_pid` accessor
- `crates/signal-runtime/src/sandbox_broker_support/tests.rs`
  — `child_pid_reports_owned_child_id` (unix): spawn stand-in child via
  `spawn_command`, assert accessor equals owned `Child::id()`, then `kill`

## Downstream follow-up

After this PR merges, orchestrator closes the Loophole cross-repo tracker
(“`SandboxBrokerClientSession` exposes no child pid”) once hosts can report
`sandbox_pid` from the accessor. Broker-packaging decision (wave 29) stays
open and out of scope.

## Validation

```text
cargo test -p signal-runtime --lib child_pid
# ok 1 passed (child_pid_reports_owned_child_id)

cargo check -p signal-runtime --lib
# ok

git diff --check
# clean

effigy qa:docs
# exit 0
```

## Next Task

Orchestrator reviews the worker PR head and merges when the gate passes.
Do not merge from this worker lane. After merge, close the Loophole tracker
entry in a separate docs closeout; leave broker-packaging open.
