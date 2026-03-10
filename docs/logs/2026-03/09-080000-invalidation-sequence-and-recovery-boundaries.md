---
title: Invalidation Sequence And Recovery Boundaries
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, invalidation, recovery, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
explicit completion-region and lease-epoch invalidation milestones during
recovery, rather than leaving those transitions implicit.

# Changes

- Added typed broker invalidation records to `signal-runtime` and exposed them
  through shared diagnostics plus `invalidation_sequence` in text and JSON
  supervisor reports.
- Added `invalidate_active_epoch()` to the CLAP lifecycle harness so recovery
  can drive a real completion-slot invalidation and lease-epoch invalidation
  step before teardown.
- Updated local and server recovery flows to emit completion-region and
  lease-epoch invalidation milestones immediately after the shared recovery
  cycle event and before transport teardown.
- Tightened runtime, CLAP harness, host, and supervisor-tool tests so the new
  invalidation path is asserted through the shared event stream and export
  layer.
- Updated the README, package map, and supervisor export contract to freeze
  `invalidation_sequence` as part of the shared runtime-facing reporting
  surface.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-plugin-clap -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-plugin-clap --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Add block-readiness and fallback-application milestones to the shared runtime
event stream so soak analysis can correlate invalidation and timeout behavior
with the exact completion-slot transitions around failed render work.
