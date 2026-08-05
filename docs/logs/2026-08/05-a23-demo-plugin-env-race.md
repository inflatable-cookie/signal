# A23 - Process-Global Demo Plugin Environment Race

Status: complete
Created: 2026-08-05
Scope: `signal-host-local` intermittent plugin discovery failure

## How It Surfaced

The `test` release gate failed on `signal-host-local --lib`. It passed on
immediate rerun, then failed `2` in `12`:

```
local host should boot: RuntimeError { kind: InvalidRequest,
  message: "plugin type was not discovered in the last local VST3 scan" }
```

Different tests each time — `boot_default_never_scans_system_plugin_directories`,
`boot_default_reports_running_stream_and_topology`,
`ensure_plugin_sandbox_rejects_undiscovered_plugin_types` — all failing at the
same line, `boot_default()`.

The same message shape had already appeared on CI for CLAP in
`public_host_edge_cross_adapter_parity`. Two crates, two formats, one symptom:
plugin discovery intermittently returning empty.

## The Mechanism

`ensure_default_demo_plugin_override` writes three process-global environment
variables — `SIGNAL_HOST_DEMO_PLUGIN_FORMAT`, `_ROOT`, `_TYPE_ID` — pointing at
a freshly built temporary fixture root, and restores them when its guard drops.

There was no lock. `tests/support/public_host_edge_plugins.rs` already had one
for its own copy of this pattern; `src/host_support/demo.rs` never got it.

Two failure paths follow, and the fix needed both.

**Guard against guard.** Two tests installing overrides concurrently each point
the variables at their own root, and whichever scans second reads the other's.

**Guard against unguarded boot.** This is the one that mattered more, and it is
not obvious. Exactly one test in the module installs the override; every other
test calls `boot_default()` with no guard at all. Those unguarded boots still
*read* the variables, so a boot running while a guard is installing or tearing
down sees a root that is half-built or already deleted, scans it, finds nothing,
and fails.

The existing code knew about the hazard and worked around it in one place. A
comment in `host_tests.rs` reads: "a parallel test may have installed a fixture
override via `ensure_default_demo_plugin_override` (process-global env), so
assert the safety property directly". That weakened one assertion to tolerate
the race rather than removing it, and left `boot_default()` itself failing
outright whenever it lost.

## The Fix

A `Mutex` in `demo.rs`, held by `DemoBootstrapGuard` for its entire life rather
than just across the writes, so the variables cannot change under a scan.

Readers hold it too. `booted_host()` in `host_tests.rs` takes the same lock
across `boot_default()`, which is what closes the second path. Locking only the
writers cut the failure rate from `2/12` to `1/25` without eliminating it —
useful evidence that the remaining path was real rather than residual noise.

## Measurement

- Before: `2` failures in `12` runs of `--lib`.
- Writers locked only: `1` in `25`. Better, not fixed.
- Writers and readers locked: `0` in `30` on `--lib`, `0` in `12` across the
  whole crate.

## A Compile Error Wearing A Test Failure's Clothes

Mid-fix the crate-internal path was wrong and the hammer loop reported `30/30`
failures. That was `cargo test` failing to build, not thirty flaky runs.

Worth recording because the same loop later reported `0/30` while
`cargo build --all-targets` was still broken: `cargo test --lib` compiles with
`cfg(test)` set, so a symbol gated to tests satisfied the test build and failed
the library build. A pass rate means nothing without knowing the thing compiled
for the configuration being claimed.

## Findings State

`A23` closed. `A18` has a mechanism and a fix awaiting listening. `A19`, `A21`,
`A22` closed earlier today; `A20` relocated to the soak lane.

## Next Task

None for `A23`. The `g10.041` listening pack is the open item.
