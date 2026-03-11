---
title: g01.007 realtime scheduler enforcement tranche
status: completed
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, runtime, roadmap, g01, scheduler, prework]
---

# Summary

Completed the first `007.2` enforcement slice by making `signal-runtime`
exercise scheduler maintenance against real engine blocks instead of only
through manual forecast/service calls.

# What Changed

- added `enforce_scheduler_after_engine_block(...)` in
  `crates/signal-runtime/src/runtime.rs` so realtime block processing now
  reconciles the future prework window after each processed block
- kept transport truth narrow: realtime blocks do not silently rewrite runtime
  transport state, and tests now apply current forecast state explicitly when
  they need forecast-backed block compatibility
- added real-block-backed tests for:
  - normal-pressure future-window extension after realtime execution
  - elevated-pressure deferred backlog retention after realtime execution
  - restart/reconfigure scheduler-window coherence across continued realtime
    processing
- reconciled older forecast/prework assertions to the current scheduler
  accounting after the new realtime enforcement path

# Validation

- `cargo test -p signal-runtime`

# Notes

- this tranche closes the first `007.2` roadmap item: runtime now proves the
  prework service lane and realtime lane against real engine blocks
- pressure/recovery escalation is still only partially tightened; the next batch
  should focus on scheduler-state transitions and recovery-aware pressure policy
  rather than adding more isolated prework mechanics
