# 2026-04-09 - g09 strict lane surface install

## Summary

Installed the first lane-first strict Northstar surface around Signal's active
`g09` queue.

## What changed

- added `docs/architecture/product-guardrails.md`
- added `docs/contracts/001-working-rules.md`
- added `docs/specs/` with one active strict-lane spec and one ready batch card
- refreshed the docs front doors so the active strict lane is explicit
- attached the strict surface to the live `g09.006` milestone without changing
  the substantive product roadmap

## Validation

- `effigy health`
- `effigy qa:docs`

## Outcome

Signal is now in lane-first stricter adoption for the active `g09` queue.
The broader repo is still baseline, but the active runtime and host
consolidation lane now has:

- explicit product guardrails
- explicit working rules
- one active strict-lane spec
- one ready batch card for the paused thread to resume from

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/002-g09-006-sandbox-session-consolidation.md`.
