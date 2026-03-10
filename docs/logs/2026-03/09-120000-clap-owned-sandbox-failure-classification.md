---
title: Clap Owned Sandbox Failure Classification
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, clap, sandbox, failure-classification]
---

# Summary

Moved sandbox-operation failure classification ownership into
`signal-plugin-clap` so host assemblies no longer derive CLAP fault stages by
pattern-matching raw `SandboxFailure` strings themselves.

# Changes

- Added a typed CLAP-side sandbox failure classifier to
  `signal-plugin-clap` and covered it with a focused unit test.
- Updated local and server hosts to consume that typed CLAP classification and
  map it into the shared runtime `sandbox_operation_failure_sequence`.
- Removed duplicated host-local string classification logic while preserving
  the existing generic plugin fault path.
- Updated the README, package map, roadmap, and supervisor export contract so
  the ownership boundary is explicit: `signal-plugin-clap` owns CLAP sandbox
  failure semantics, while `signal-runtime` owns the shared reporting surface.

# Validation

- `cargo check -p signal-plugin-clap -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-plugin-clap --no-run`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`

# Next Task

Decide whether broker-visible host failures and CLAP-emitted sandbox
operation failures should remain separate sequences long term, or be grouped
under one higher-level shared transport-fault model with explicit source
labels.
