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

Resolve the paused `g10.030` architecture checkpoint. The event-sealed brief
and its multiresolution phase-vocoder family are rejected; keep the retained
baseline frozen until the operator either closes the program or commissions a
different renderer family.
