# Papercuts wave 32 — headless LocalRuntimeHost hardware seam

Status: closeout
Date: 2026-08-31
Owner: papercuts worker
Handoff: `docs/handoffs/20260831-223724-papercuts-wave32-local-host-headless.md`
Branch: `worker/papercuts-wave32-local-host-headless`

## Summary

`LocalRuntimeHost` eagerly owned a concrete cpal-backed `LocalHardwareBackend`,
so `boot_default` failed with `DeviceUnavailable` when no default output device
existed. Signal already had `HardwareBackend` and `SimulatedHardwareBackend`;
the host lacked an injection path over that contract.

Added `LocalRuntimeHost::with_hardware(runtime, Box<dyn HardwareBackend>)`.
`LocalRuntimeHost::new(runtime)` still constructs the real local/cpal backend.
`LocalHardwareBackend` now implements `HardwareBackend`; boot negotiates the
default output through the trait (`default_output_device` + `negotiate_stream`)
without changing negotiation, diagnostics, policy, boot order, discovery, or
lifecycle semantics. No audio callback stream is opened.

Loophole `PAPERCUTS.md` tracker entry left open for orchestrator closeout after
merge and downstream proof.

## Public construction API

- `LocalRuntimeHost::new(runtime)` — unchanged real local/cpal path
- `LocalRuntimeHost::with_hardware(runtime, Box<dyn HardwareBackend>)` —
  explicit injection for tests and headless consumers

## Files

- `crates/signal-host-local/src/host.rs` — `Box<dyn HardwareBackend>` field;
  `with_hardware`; `new` delegates to local/cpal default
- `crates/signal-host-local/src/host_support/hardware.rs` —
  `HardwareBackend` for `LocalHardwareBackend` (behavior unchanged)
- `crates/signal-host-local/src/host_support/boot_entrypoints.rs` — default
  output prep via trait methods
- `crates/signal-host-local/src/host_tests.rs` —
  `boot_default_with_injected_simulated_hardware_reports_simulated_stream`

## Downstream follow-up

After this PR merges, orchestrator closes the Loophole cross-repo tracker
(“`LocalRuntimeHost` cannot boot headless”) once live-host consumers can boot
over an injected simulated backend. Broker-packaging decision stays open and
out of scope.

## Validation

```text
cargo test -p signal-host-local --lib
# ok 11 passed (includes boot_default_with_injected_simulated_hardware_reports_simulated_stream
# and existing boot_default_* real-path tests)

cargo check -p signal-host-local
# ok

git diff --check
# clean

effigy qa:docs
# exit 0

effigy qa:northstar
# exit 0
```

## Next Task

Orchestrator reviews the worker PR head and merges when the gate passes.
Do not merge from this worker lane. After merge, close the Loophole tracker
entry in a separate docs closeout; leave broker-packaging open.
