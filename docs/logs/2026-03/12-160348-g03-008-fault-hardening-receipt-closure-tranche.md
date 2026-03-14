# g03.008 - Fault Hardening Receipt Closure Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/008-engine-profiling-soak-harnesses-and-runtime-fault-hardening.md`

## Summary

Completed Batch 8.2 and closed `g03.008`. The runtime-owned hardening contract
now covers the degraded boundary across routed prework gating, plugin-chain
quarantine/unavailability, and delegated offline render failure without
dropping back to raw snapshot-only assertions.

## Shipped

- expanded `RuntimeProfilingReceipt` with degradation and routed plugin-chain
  counters so routing gate state, degraded bound plugin counts, plugin-chain
  depth, and fault-adjacent session pressure stay visible on the reusable live
  runtime receipt surface
- expanded `RuntimeSoakReceipt` with readiness, plugin lifecycle, and recall
  counts so recovery/quarantine export can pin plugin-chain degraded state on
  the same runtime-owned soak contract
- added typed `RuntimeOfflineRenderProfilingReceipt` and
  `RuntimeOfflineRenderSoakReceipt` derivation on `RuntimeOfflineRenderResult`
  so delegated or stale offline render boundary state can be consumed without
  parsing artifact reports
- upgraded focused runtime tests to prove:
  - routed plugin-bound prework gate state through live receipts
  - quarantined plugin lifecycle and unavailable recall through live receipts
  - delegated-unavailable offline render boundary through typed offline receipts

## Residual Risk

- this tranche freezes the contract and focused acceptance spine, but it does
  not introduce threshold/fail-gate policy for longer-duration benchmark runs
- there is still no next generation open for whatever broader post-`g03`
  product or deployment queue follows

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `cargo test -p signal-supervisor-tools`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Completion

`g03.008` is complete, and `g03` is closed. Open the next generation only when
maintainers want the next reusable Signal roadmap queue.
