# 004 - g11.002 Multiplexing Design Note

Status: complete
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.002
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/architecture/shared-sandbox-multiplexing.md, docs/roadmaps/g11/002-shared-sandbox-tier.md
Auto-start next card: yes

## Objective

Freeze the SharedSandbox v1 multiplexing shape against Contract `014` and the
existing sandbox broker protocol, without production-code edits.

## Scope

Closed. Docs-only. Operator product pull 2026-08-17. Grouping is plugin type
identity. No new contract.

## Acceptance Criteria

- [x] multiplexing map exists under `docs/architecture/`
- [x] wire compatibility, grouping key, receipt proof, and non-goals are explicit
- [x] Batch 2.1 is bounded enough to execute without fresh planning decisions
- [x] no research brief opened

## Validation

- `effigy qa:docs`

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-002-batch-2-0-multiplexing-design-note.md`

## Stop Conditions

None fired.

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
