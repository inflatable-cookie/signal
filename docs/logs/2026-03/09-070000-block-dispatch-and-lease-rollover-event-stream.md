---
title: Block Dispatch And Lease Rollover Event Stream
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, block-dispatch, lease-rollover, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
brokered block dispatch/completion work and lease rollover milestones in
addition to lifecycle, transport, and heartbeat events.

# Changes

- Added typed block-dispatch records for requested, completed, and timed-out
  brokered render work with lease, epoch, block sequence, and completion
  state.
- Added typed lease-rollover records emitted by `signal-runtime` when block
  sequencing crosses from one shared-memory lease generation to another.
- Updated local and server hosts to emit block-dispatch events around real
  brokered render work and to route block-sequence tracking through the new
  runtime-owned rollover path.
- Extended runtime diagnostics and supervisor export rendering so
  `block_dispatch_sequence` and `lease_rollover_sequence` are available in
  shared text and JSON reports.
- Tightened runtime, host, and supervisor-tool tests so dispatch and rollover
  behavior are asserted through the shared event stream.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Add completion-region invalidation and explicit epoch/lease invalidation
milestones to the shared runtime event stream so soak analysis can correlate
render failures with broker invalidation boundaries.
