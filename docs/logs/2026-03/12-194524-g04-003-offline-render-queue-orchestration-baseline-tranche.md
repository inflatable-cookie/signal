# g04.003 Offline Render Queue Orchestration Baseline Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/contracts/`, `docs/architecture/`, `docs/roadmaps/g04/`

## Summary

Completed Batch 3.2 of `g04.003` by making the offline render queue the first
runtime-owned deferred-work orchestration baseline.

## What changed

- added typed deferred-service orchestration surfaces in `signal-runtime`
  around `RuntimeDeferredServiceReceipt` plus class/decision/reason enums
- upgraded the existing offline render queue path so runtime state now decides
  whether the queue should `Run`, `Throttle`, or `Defer`
- made healthy non-running runtime drain the queue, live runtime throttle queue
  advancement to bounded progress, and safe-mode or recovery-sensitive state
  defer queue execution without dropping requests
- kept the orchestration baseline off the audio-thread path by reusing the
  existing offline render control-path helper rather than widening realtime
  execution responsibilities
- updated the contract, reference, and roadmap trail to move `g04.003` from
  Batch 3.2 implementation into Batch 3.3 validation and consumer proof

## Why this tranche

`g04.003` needed one real deferred service path to stop treating queue cadence
as an implicit host concern. The offline render queue already had runtime-owned
requests, results, and progress receipts, so it was the narrowest place to
attach the first reusable orchestration decision model.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib runtime_offline_render_queue`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.003` with Batch 3.3 and prove the new deferred-work baseline
through playback/capture pressure and recovery-sensitive consumer paths,
exposing enough receipts that hosts can observe policy outcomes without
rebuilding queue timing locally.
