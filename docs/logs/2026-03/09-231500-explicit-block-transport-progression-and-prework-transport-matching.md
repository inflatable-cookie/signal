---
title: explicit block transport progression and prework transport matching
status: closed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, scheduler, transport, prework]
---

# Summary

Made transport progression explicit in the local and server host block loops
and aligned runtime prework invalidation with the same transport fields used by
the prework matcher, so queued next-block work can survive matching transport
updates instead of being retired by loop-metadata differences.

# What changed

- Updated local and server runtime hosts to apply deterministic block-start
  transport projections before engine execution instead of relying only on
  implicit post-block transport advancement.
- Updated both hosts to queue next-block prework against the next block's
  expected transport projection as well as its expected parameter epoch.
- Tightened `SignalRuntime::apply_transport_projection(...)` so prework
  invalidation compares the scheduler-facing transport fields
  (`playing`, `tempo_bpm`, `timeline_position_samples`) instead of full
  `TransportProjection` equality.
- Extended the runtime queued-prework proof to exercise matching future
  transport projection as well as matching future parameter epoch.
- Kept the local and server timeout-recovery proofs green under the explicit
  per-block transport path.

# Validation

- `cargo test -p signal-runtime runtime_consumes_primed_prework_for_the_next_block -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

# Notes

- This batch did not hit any stale Effigy locks.
- The server host now follows the same future-state priming contract as the
  local host even though its default runtime profile still keeps anticipative
  execution disabled.
