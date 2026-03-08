# 2026-03-08 22:45:00 - Supervisor Export Contract And Placement Decision

Status: complete
Owner: core-product

## Summary

Converted the supervisor export/report behavior from an implementation detail
into an explicit Signal contract and recorded the current placement decision for
automation continuity.

This batch adds:

- `002-supervisor-export-schema-and-report-boundary.md` as the canonical
  contract for the `signal.supervisor.export` envelope and the shared
  `supervisor_report` versus `host_summary` split,
- explicit documentation that timeline continuity is shared/runtime-facing,
- explicit documentation that automation continuity is still transitional even
  though it now travels through `RuntimeAutomationSnapshot`,
- updated docs entry points so the new contract is discoverable from Signal’s
  root docs and architecture pages.

## Files

- `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`
- `docs/contracts/README.md`
- `docs/README.md`
- `docs/architecture/system-architecture.md`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `git diff --check`

## Validation Notes

- This batch is docs-only. No code or runtime behavior changed, so I did not
  rerun the heavier Cargo or Effigy validation passes again after the previous
  green batch.

## Notes

- The key unresolved architectural question is now crisp: automation continuity
  is no longer undocumented, but it is still not runtime-owned in the same way
  block-sequence continuity is.

## Next Task

Make the automation continuity placement decision explicit in code: either move
it into runtime-owned state, or keep it host-attached and trim the shared
report/API so the transitional boundary is narrower and more honest.
