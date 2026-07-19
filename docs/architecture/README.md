# Architecture

Status: active
Updated: 2026-07-19

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
- `product-guardrails.md`
- `package-map.md`
- `dsp-analysis-feature-reference.md`
- `offline-time-stretch-synthesis.md`
- `offline-time-stretch-successor-brief.md`
- `graph-runtime-feature-reference.md`
- related contracts under `docs/contracts/`

## Next Task

Implement the frozen offline stretch successor only in the isolated `g10.030`
Batch 30.3 worktree. Keep `main` on the retained baseline until admission.
