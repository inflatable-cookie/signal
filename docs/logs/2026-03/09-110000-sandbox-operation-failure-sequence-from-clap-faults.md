---
title: Sandbox Operation Failure Sequence From Clap Faults
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, clap, sandbox, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
typed sandbox-emitted attach, flush, and protocol failures from the CLAP
harness path, distinct from host-visible broker I/O failures.

# Changes

- Added typed sandbox operation failure records to `signal-runtime` and exposed
  them through shared diagnostics plus `sandbox_operation_failure_sequence` in
  text and JSON supervisor reports.
- Updated local and server runtime hosts to derive typed sandbox operation
  failures from `SandboxFailure` envelopes emitted by the CLAP harness while
  preserving the existing generic plugin fault path.
- Tightened runtime, host, and supervisor-tool tests so the new sequence is
  pinned in the shared report/export surface and remains absent in the normal
  soak paths covered by current fixtures.
- Updated the README, package map, roadmap, and supervisor export contract to
  freeze `sandbox_operation_failure_sequence` as part of the shared
  runtime-facing reporting surface.

# Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Move the current sandbox-operation failure classification closer to the CLAP
harness itself, so the shared supervisor report can consume direct typed
sandbox failure stages instead of relying on host-side derivation from
`SandboxFailure` strings.
