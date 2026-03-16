# 2026-03-16 19:09:55 UTC - g06.019 Cross-Family Integrated Evidence Closure And g06.020 Handoff

## Summary

Closed `g06.019` by turning the integrated acceptance lane into a real
cross-family evidence surface. The shared lane now proves one supervisor export
can carry combined recovery, deferred-work, adapter, hardware, and media or
analysis-library receipts instead of only grouping previously closed boundary
tasks.

## Work completed

- added the focused cross-family export proof and aligned integrated-lane
  validation steps:
  - `crates/signal-supervisor-tools/src/main.rs`
- promoted the same proof into the grouped Effigy lane:
  - `effigy.toml`
- recorded the integrated-evidence closure in:
  - `docs/roadmaps/g06/019-fault-injection-harnesses-and-multi-backend-acceptance-depth.md`
  - `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
- moved the active queue to `g06.020` in:
  - `docs/roadmaps/g06/README.md`
  - `docs/roadmaps/g06/020-long-session-soak-promotion-gate-and-loophole-readiness-closeout.md`
  - `docs/roadmaps/README.md`
  - `docs/roadmaps/generation-index.md`
  - `docs/contracts/README.md`
  - `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools integrated_acceptance_lane_json_reports_required_and_advisory_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json`
- `effigy acceptance:integrated-acceptance-lane --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- longer-session soak thresholds, rerun policy, and promotion gates still
  belong to `g06.020`
- unstable broader server-host recovery-overlap scenarios remain outside the
  bounded required lane
- this closes integrated acceptance depth, not the final generation-closeout
  gate or Loophole-facing readiness decision

## Next Task

Continue `g06.020` with Batch 20.1 by freezing the bounded long-session soak,
promotion-gate, and Loophole-readiness policy on top of the now-closed
integrated acceptance lane.
