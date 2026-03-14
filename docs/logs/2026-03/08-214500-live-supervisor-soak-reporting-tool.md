# 2026-03-08 21:45:00 - Live Supervisor Soak Reporting Tool

Status: complete
Owner: core-product

## Summary

Moved supervisor soak inspection out of synthetic demos and into a dedicated
tooling path that runs the real local/server runtime hosts.

This batch adds:

- library targets for `signal-host-local` and `signal-host-server`,
- a new `signal-supervisor-tools` crate for running real host scenarios,
- CLI support for `default`, `timeout`, `crash`, `heartbeat`, `soak`, and
  `mixed` scenarios across local/server profiles,
- live reporting of host summary, lease/timeline continuity, automation
  continuity, and shared `RuntimeSupervisorReport` output,
- updated workspace docs so the new tool crate is part of the stable Signal
  layout.

## Files

- `Cargo.toml`
- `crates/signal-host-local/src/lib.rs`
- `crates/signal-host-local/src/main.rs`
- `crates/signal-host-server/src/lib.rs`
- `crates/signal-host-server/src/main.rs`
- `crates/signal-supervisor-tools/Cargo.toml`
- `crates/signal-supervisor-tools/src/main.rs`
- `README.md`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- local soak`
- `cargo run -p signal-supervisor-tools -- server mixed`
- `effigy validate`
- `effigy health`

## Validation Notes

- The new tool runs against the real host boot paths rather than synthetic
  recorder events, so the reported timeline and supervision data reflect the
  same code exercised by local/server host smoke tests.
- The host `main` binaries remain intact, but they now consume library targets
  instead of owning the only public entrypoint to those assemblies.

## Notes

- The current tool output is optimized for human inspection. The next obvious
  step is structured export so longer soak runs can be asserted by external
  automation instead of only read in terminal output.

## Next Task

Deepen the supervisor soak-reporting tool into reusable fixtures or machine-
readable exports so longer restart/lease-rollover runs can be inspected by
automation as well as humans, then fold richer runtime timeline details into
the shared supervisor report surface.
