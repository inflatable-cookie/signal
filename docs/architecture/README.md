# Architecture

Status: active
Updated: 2026-03-10

## Why this section matters now

Architecture defines Signal as a reusable library system rather than a
Loophole-specific engine app.

## Scope

Use this section for:

- crate and package boundaries
- embeddable runtime shape
- trust-edge adapter boundaries
- generic library invariants

Keep milestone sequencing in `roadmaps/`.

## Active Entry Points

- `system-architecture.md`
- `package-map.md`
- `dsp-analysis-feature-reference.md`
- related contracts under `docs/contracts/`

## Next Task

Keep `system-architecture.md` and `package-map.md` aligned with the clean
library posture, then move legacy implementation-shaped material behind a
reference boundary instead of leaving it in the active path.
