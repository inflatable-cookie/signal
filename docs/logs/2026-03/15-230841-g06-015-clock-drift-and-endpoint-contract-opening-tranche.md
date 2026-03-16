# 2026-03-15 23:08:41 UTC - g06.015 Clock Drift And Endpoint Contract Opening Tranche

## Summary

Opened `g06.015` by freezing the first runtime-owned contract for clock drift,
discontinuity, duplex mismatch, partial availability, resync, and
endpoint-topology meaning. This batch keeps later hardware and external-I/O
depth on one shared runtime boundary instead of reopening backend-private drift
heuristics or host-local endpoint models.

## Work completed

- added the new contract:
  - `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`
- marked Batch 15.1 complete in the active roadmap:
  - `docs/roadmaps/g06/015-clock-domain-drift-duplex-mismatch-and-endpoint-topology-depth.md`
- updated the contract index and generation or roadmap next-task pointers
- refreshed the hardware architecture reference so the host-I/O section now
  records the contract-frozen drift and endpoint-topology rule

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- this batch freezes topology and drift meaning, not runtime receipt depth
- network-audio, distributed clocking, monitoring-role detail, and loopback
  topology semantics remain later work

## Next Task

Continue `g06.015` with Batch 15.2 by materializing stronger runtime-owned
clock-domain drift, discontinuity, duplex mismatch, and endpoint-topology
receipts while keeping host-edge and supervisor export aligned to the same
state model.
