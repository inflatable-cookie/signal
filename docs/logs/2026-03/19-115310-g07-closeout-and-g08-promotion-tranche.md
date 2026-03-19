# 2026-03-19 - g07 Closeout And g08 Promotion Tranche

## Summary

Closed `g07` with an explicit reusable-substrate verdict, promoted `g08` as the
next active generation, and seeded the first `g08` milestone around live Linux
audio backend ownership.

## Why this tranche matters

`g07.020` could not end as a permanent provisional review. This tranche turns
the runnable `g07` closeout gate into a real verdict and names the next active
queue, so the remaining Linux live-ownership, immersive, device-protocol, and
workflow depth moves forward under one explicit generation instead of a vague
deferred tail.

## What changed

- updated the shared generation-closeout descriptor in
  `crates/signal-supervisor-tools/src/main.rs` so it now records `promote-g08`
  and `sufficient-for-promotion` readiness areas for `g07`
- closed the `g07.020` roadmap and contract surfaces
- marked `g07` complete and promoted `g08` as the single active queue
- added `docs/roadmaps/g08/README.md` and
  `docs/roadmaps/g08/001-live-linux-audio-backend-ownership-and-session-lifecycle-substrate.md`
  to seed the next generation

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools generation_closeout_json_reports_combined_boundary_and_next_queue -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g07-closeout --repo .`
- `effigy test --plan --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

The `g07` verdict remains a reusable Signal substrate verdict, not a Loophole
product-launch verdict. Full live Linux ownership, richer immersive routing,
vendor-protocol device depth, and preview-browser workflow services are now
explicit `g08` work rather than hidden closeout debt.

## Next task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
