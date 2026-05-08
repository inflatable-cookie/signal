# 001 - Install g09 Strict Lane Surfaces

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.006 strict-lane setup
Governing refs: docs/contracts/001-working-rules.md, docs/specs/001-g09-lane-first-strict-adoption.md
Auto-start next card: yes, if the active `g09.006` boundary stays current

## Objective

Install the minimum strict Northstar docs surface around the active `g09` lane.

## Scope

- add Signal product guardrails and working rules
- add the `docs/specs/` strict-lane surface
- refresh Signal front doors so the current lane is explicit

## Acceptance Criteria

- the minimum strict surface exists
- the front doors point at the active strict lane
- the next ready card is explicit

## Completion Notes

- Installed the first lane-first strict tranche around `g09`.
- Left the active implementation boundary in one explicit ready card for
  `g09.006`.

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/002-g09-006-sandbox-session-consolidation.md`.
