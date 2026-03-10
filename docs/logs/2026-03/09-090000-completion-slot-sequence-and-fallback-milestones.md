---
title: Completion Slot Sequence And Fallback Milestones
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, completion-slot, fallback, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
the exact completion-slot path around brokered render work, including explicit
fallback application during timed-out blocks.

# Changes

- Added typed completion-slot transition records to `signal-runtime` and
  exposed them through shared diagnostics plus `completion_slot_sequence` in
  text and JSON supervisor reports.
- Updated local and server block execution paths to emit runtime-owned
  completion-slot milestones for ready-for-processing, processing, completed,
  timed-out, invalidated, and fallback-applied transitions.
- Tightened runtime, host, and supervisor-tool tests so soak and export
  surfaces assert the new completion-slot sequence alongside dispatch,
  invalidation, and recovery data.
- Updated the README, package map, roadmap, and supervisor export contract to
  freeze `completion_slot_sequence` as part of the shared runtime-facing
  reporting surface.

# Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Extend the shared runtime event stream further into brokered execution by
adding typed broker/control failures around block payload read/write and lease
attachment/teardown, so soak analysis can correlate completion-slot and
invalidation transitions with concrete transport-side failure points.
