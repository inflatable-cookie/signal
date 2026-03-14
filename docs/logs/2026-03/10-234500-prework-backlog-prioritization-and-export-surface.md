---
title: Prework Backlog Prioritization And Export Surface
status: closed
owner: codex
updated: 2026-03-10
tags: [signal, runtime, scheduler, anticipative, reporting]
---

## Summary

Finished the backlog-prioritization slice for the runtime-owned prework lane.

## What Changed

- extended `signal-runtime` prework scheduling so pending future targets are
  classified as `Immediate`, `NearTerm`, or `Deferred`
- kept elevated-pressure service focused on nearer backlog classes while
  deferred targets remain queued for later service
- exposed backlog counts plus the last serviced target block/backlog class in
  the shared compact report, multiline report, and JSON export surfaces
- aligned README, architecture, contract, and roadmap notes so the new
  scheduler behavior is explicit rather than only covered by tests

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_prework_service_lane_yields_under_critical_pressure -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_service_lane_throttles_under_elevated_pressure -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_service_lane_preserves_deferred_targets_under_elevated_pressure -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy locks showed up in this batch
- the main change is scheduler-facing behavior and visibility, not a new host
  policy seam
