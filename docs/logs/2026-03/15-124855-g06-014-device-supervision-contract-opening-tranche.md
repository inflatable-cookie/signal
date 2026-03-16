# 2026-03-15 12:48:55 UTC - g06.014 Device Supervision Contract Opening Tranche

## Summary

Opened `g06.014` by freezing the first runtime-owned device supervision,
restart-state machine, exhaustion, and hardware fault-boundary contract. This
batch keeps restart ownership inside Signal-owned runtime surfaces before the
later hardware, clocking, and external-I/O milestones widen.

## Work completed

- added the new contract:
  - `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
- marked Batch 14.1 complete in the active roadmap:
  - `docs/roadmaps/g06/014-device-supervision-restart-state-machine-and-fault-boundary-depth.md`
- updated the contract index and generation or roadmap next-task pointers
- refreshed the architecture reference so the host-I/O section now records the
  contract-frozen supervision rule

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- this batch freezes supervision meaning, not runtime receipt depth
- exhaustive hardware certification, device-setup UX, remote hardware scope,
  and later clock-drift or endpoint-topology semantics remain later work

## Next Task

Continue `g06.014` with Batch 14.2 by materializing stronger runtime-owned
device supervision, restart, exhaustion, and fault-boundary receipts while
keeping host-edge and supervisor export aligned to the same state model.
