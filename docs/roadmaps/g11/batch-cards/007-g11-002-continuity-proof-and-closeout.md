# 007 - g11.002 Continuity Proof And Closeout

Status: ready
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.002
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md, docs/architecture/shared-sandbox-multiplexing.md, docs/roadmaps/g11/002-shared-sandbox-tier.md
Auto-start next card: no
Depends on: 006-g11-002-host-assembly-integration.md

## Objective

Prove Contract `014` shared-boundary blast radius on runtime receipts, then
close `g11.002`.

## Scope

- one proof that child death / terminal outcome is visible for every member
  of a SharedSandbox grouping key
- public host-edge or crate test is enough; do not add a product UX
- refresh docs so SharedSandbox is no longer "unimplemented"
- update Contract `072` remaining-gaps table
- close the milestone and generation front doors

Out of scope: vendor/format grouping, replacing DedicatedSandbox default,
opening `g12`.

## Acceptance Criteria

- [ ] killing the shared child marks all member snapshots with the same
  boundary continuity class (Restartable or Terminal per Contract `014` /
  `012`)
- [ ] `shared_boundary_member_count` stays accurate through the fault
- [ ] DedicatedSandbox crash isolation proof is untouched
- [ ] inventory, integration map, Contract `072`, and `g11` front doors match
  the landed behavior
- [ ] Next Task does not start a new generation by implication

## Validation

- focused host-local or sandbox test for shared-boundary terminal fan-out
- `effigy qa:docs`
- `effigy validate`

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-002-batch-2-3-continuity-proof-and-closeout.md`

## Stop Conditions

- blast radius cannot be explained without a host-private process map
- Contract `014` appears to need a vocabulary change
- validation failure changes the multiplexing shape

## Next Task

Stop for operator review of the `g11.002` PR. Do not start a follow-on
generation from this card.
