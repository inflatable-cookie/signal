# g11 Batch Cards

Use this folder for ready execution cards under the active `g11` generation.

## Rule

- create a card only when the work is specific enough to execute without fresh
  design decisions
- sequence cards through the active `g11` milestone
- auto-continue when the next card is already `ready` and governing refs still
  match
- stop on a planning gap, contract contradiction, or failed evidence gate

## File pattern

- `NNN-g11-MMM-<slug>.md`

## Current cards

- `001-g11-001-bridge-backend-factory.md` — `complete`
- `002-g11-001-render-plane-consumer-wiring.md` — `complete`
- `003-g11-001-host-edge-proof-and-closeout.md` — `complete`
- `004-g11-002-multiplexing-design-note.md` — `complete`
- `005-g11-002-broker-multiplexing.md` — `ready`
- `006-g11-002-host-assembly-integration.md` — `ready` (auto-start after 005)
- `007-g11-002-continuity-proof-and-closeout.md` — `ready` (auto-start after 006)

## Next Task

Execute `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
