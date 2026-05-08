# 2026-04-10 - g09 Reopened For Production Readiness Recovery

Status: active
Owner: core-product
Roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Card: `docs/roadmaps/g09/batch-cards/035-g09-014-readiness-rubric-and-gap-inventory.md`

## Summary

Recovered the planning surfaces after the `g09` completion criterion changed.

The previous `g09` closeout was no longer trustworthy once the requirement was
clarified: `g09` is only complete when the existing crates reach
production-ready grade for their intended role, not merely when audit
remediation and demo proof are complete.

## Recovery decision

- reopened `g09`
- added contract `080` to freeze the new production-readiness gate
- promoted new milestone `g09.014`
- promoted ready card `035` for the readiness rubric and per-crate gap
  inventory

## Validation Run

- `effigy tasks`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/035-g09-014-readiness-rubric-and-gap-inventory.md`.
