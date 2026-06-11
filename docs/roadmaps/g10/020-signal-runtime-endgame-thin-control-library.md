# 020 - Signal Runtime Endgame Thin Control Library

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.015
Vision tags: `DEMOLITION`, `ARCHITECTURE`

## Problem

After WYSIWYG bounce retires the simulation offline path, signal-runtime's
last real jobs are the media/decode pipeline pieces pulse uses, broker/
sandbox sessions, and the observation slice host-local serves. The
engine-block simulation, topology narration snapshots, prework scheduler
welded to the simulated engine, and transport-session concurrency model
have no remaining product consumer. host-local (7.5k) + runtime (52.6k)
should shrink to under ~10k combined.

## Goals

- [x] inventory pulse's true consumption (signal_bridge + host-local surface) and freeze it as the target API
- [x] delete the engine-block simulation path and its snapshot/observation surfaces beyond the consumed slice
- [x] extract keepers into right-sized homes: media decode/analysis pipeline, broker sessions, plugin contracts consumption
- [x] prework scheduler: record the policy vocabulary in a design note, then delete — re-derive against the render plane when anticipative rendering is scheduled
- [x] host-local collapses into the thin bridge pulse actually needs (possibly merged into one crate)
- [x] workspace member count and README updated; CHANGELOG entry

## Execution Plan

### Batch 20.1 - Freeze And Cut

- [x] consumption inventory; simulation deletion; keeper extraction

### Batch 20.2 - Collapse

- [x] host-local merge/shrink; front-door truth pass

## Acceptance Criteria

- [x] pulse builds and passes against the reduced surface unchanged
- [~] runtime+host-local combined under ~10k LoC (landed at ~15k src; the
  remainder is consumed functionality — media decode/analysis pipeline,
  sandbox broker sessions, plugin discovery records, host I/O observation,
  and event diagnostics — not simulation)
- [x] no simulation execution path remains in the workspace (engine-block
  path, prework scheduler, signal-graph block execution, and the
  clip-render path all deleted)

## Progress

- 2026-06-11: executed in full. Frozen consumed surface inventoried from
  pulse's `signal_bridge.rs` and host-local; pulse needed zero source or
  test edits (the boot latency pin of 536 samples survives because graph
  latency is now reported from the declared plan latency, 24 samples for
  the demo graph, plus the negotiated 512-sample output latency).
- Deleted from `signal-runtime`: engine-block simulation + per-block
  bookkeeping, anticipative prework scheduler (vocabulary preserved in
  `docs/architecture/prework-scheduler-design-note.md`), transport-session
  concurrency/leases, planning/scheduler/timeline/automation/metering
  narration snapshots, deferred-service receipt stubs, plugin recall
  ARA/preset and pin-matrix capture carve-outs, spatial execution family,
  preview-transform/transform-artifact/stretch/marker stack and the
  clip-render simulation, profiling/soak/performance-trace receipts, LV2
  capture machinery, and the integration tests that pinned them.
- `signal-host-local`: boot no longer pumps 8 simulated engine blocks; a
  negotiated output stream is the boot proof and the audio pump summary
  shrank to the stream state pulse reads. Output-pump/transfer machinery
  deleted.
- `signal-graph`: kept as the plan model (specs, contracts, planning and
  contract summaries); the offline block-execution engine, bus state,
  stage processors, and parameter-event machinery were deleted.
- LoC (src, wc -l): signal-runtime 43.9k → 12.8k; signal-host-local
  2.5k → 2.2k; signal-graph 2.9k → 1.1k. External boundary tests trimmed
  from 9.2k to 2.8k.
- g10 stays open; closure is the owner's call.

## Next Task

Generation g10 closure becomes a real conversation after this lands.
