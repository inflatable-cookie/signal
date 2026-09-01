# Papercuts wave 41 — LocalRuntimeHost Send boundary

Status: closeout
Date: 2026-09-01
Owner: papercuts worker
Handoff: `docs/handoffs/20260901-114200-papercuts-wave41-local-host-send-boundary.md`
Branch: `worker/papercuts-wave41-local-host-send-boundary`

## Summary

Signal PR 17 introduced `LocalRuntimeHost::with_hardware` and stored
`hardware: Box<dyn HardwareBackend>`. Loophole's `LiveHost` implements
`pulse_authority::TransportDriver: Send`, so the erased backend made
`pulse-signal-link` fail to compile.

Repair: require `Send` only at the host's erased injection boundary —
`Box<dyn HardwareBackend + Send>` — without changing the shared
`HardwareBackend` trait, runtime behavior, callback paths, or
`TransportDriver`.

## Reproducer (before)

From clean Loophole against Signal `main` at
`6e59d23505bad985d34f5d2cfa139311406d99ce`:

```text
cargo check -p pulse-signal-link
# error[E0277]: `(dyn HardwareBackend + 'static)` cannot be sent between
# threads safely
#   --> crates/pulse-signal-link/src/live.rs:577:26
#    | impl TransportDriver for LiveHost
# note: required by a bound in `TransportDriver`
#   --> crates/pulse-authority/src/transport.rs:125:28
#    | pub trait TransportDriver: Send
```

## Chosen bound

- Host field / `with_hardware`: `Box<dyn HardwareBackend + Send>`
- Shared trait `HardwareBackend` unchanged (no new supertrait)
- No `unsafe impl Send`, mutex wrapper, or thread-affinity workaround

## Implementors checked

| Implementor | Location | Send basis |
| --- | --- | --- |
| `LocalHardwareBackend` | `signal-host-local` host_support | owned policy/device/diagnostics data only |
| `SimulatedHardwareBackend` | `signal-hardware` | owned identity/device/lifecycle/diagnostics data only |

Both satisfy the host bound without unsafe. Public impact: injectors must
pass a `Send` backend; production `LocalRuntimeHost::new` still uses the real
local/cpal path.

## After (downstream)

Temporary path remap of Loophole workspace Signal deps to this worktree
(restored immediately; lockfile reverted; no Loophole commit):

```text
cargo check -p pulse-signal-link
# Finished `dev` profile … (ok; TransportDriver for LiveHost compiles)
```

## Unchanged boundaries

- `LocalRuntimeHost::new` real cpal path retained
- `with_hardware` remains the headless injection seam
- No audio callback / lifecycle / IPC / pin / lockfile / workflow changes
- Signal broker option-2 decision not reopened
- Loophole tracker and worktree untouched

## Files

- `crates/signal-host-local/src/host.rs` — `+ Send` on erased backend
- `crates/signal-host-local/src/host_tests.rs` — `local_runtime_host_is_send`
- `crates/signal-host-local/src/host_support/hardware.rs` —
  `local_hardware_backend_is_send`
- `crates/signal-hardware/src/simulated/tests.rs` —
  `simulated_hardware_backend_is_send`
- `PAPERCUTS.md` — matching entry closed in this PR
- this evidence log

## Validation

```text
cargo test -p signal-hardware --lib simulated::
# ok (includes simulated_hardware_backend_is_send)

cargo test -p signal-host-local --lib
# ok 13 passed (Send proofs + real/simulated boot coverage)

cargo check -p signal-host-local
# ok

cargo clippy -p signal-host-local -p signal-hardware -- -D warnings
# ok

# downstream evidence (paths restored; lockfile reverted)
cargo check -p pulse-signal-link
# ok against this worktree

effigy fmt
effigy qa:docs
effigy qa:northstar
git diff --check
```

Skipped broad `effigy qa` / full workspace suites: out of scope for this
bounded Send-boundary repair; focused crate proof + downstream compile cover
the review oracle. Handoff named `fmt:rust:check`; this repo exposes `fmt`
(`cargo fmt --all -- --check`).

## Next Task

Orchestrator reviews the worker PR head with material-risk scrutiny and merges
when required checks pass. Do not merge from this worker lane. After merge,
Loophole broker-contract worker may resume with this Signal prerequisite.
