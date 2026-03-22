# 2026-03-22 - g08.020 Batch 20.2 closeout gate descriptor tranche

## Summary

Materialized the first machine-readable `g08` closeout descriptor and
repo-owned closeout gate on top of the closed `g08.019` integrated acceptance
seam.

## Work completed

- retargeted the shared generation-closeout descriptor in
  `signal-supervisor-tools` from the older `g07` posture to a provisional
  `g08` closeout-review posture
- added the runnable `acceptance:g08-closeout` Effigy task
- updated the roadmap, contract, and feature-reference trail so Batch 20.3 is
  now the single remaining `g08` closeout batch

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools generation_closeout_json_reports_combined_boundary_and_next_queue -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g08-closeout`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.020` with Batch 20.3 by recording the final `g08` closeout
verdict and the next queue cleanly from the now-runnable closeout gate
instead of leaving promotion or backlog posture implicit.
