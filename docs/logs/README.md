# Logs

Status: active
Updated: 2026-04-09

## Why this section matters now

Logs are the evidence layer for Signal’s library/runtime rebuild.

## Scope

Use this section for:

- batch evidence
- decision logs
- reset-era cleanup and archive notes
- implementation validation records

## Segmentation Model

- `logs/YYYY-MM/`
- `logs/templates/`

## Working Rule

- log by meaningful batch
- keep evidence concise and explicit
- prefer manual validation notes unless repeated pain justifies automation
- in the strict lane, a bare `continue` should resolve through the previous
  closeout's `Next Task`, not through a giant reminder prompt

## Current lane

- strict-lane spec:
  `docs/specs/001-g09-lane-first-strict-adoption.md`
- current ready card:
  `docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`

## Recent active-lane evidence

- `2026-04/09-245000-g09-005-linux-lv2-acceptance-boundary-and-promotion.md`
- `2026-04/09-260000-g09-006-shared-cycle-and-watchdog-helper-tranche.md`
- `2026-04/09-271500-g09-006-shared-runtime-block-shell-tranche.md`
- `2026-04/09-281500-g09-strict-lane-surface-install.md`
- `2026-04/09-164347-g09-006-sandbox-session-shared-broker-shell-tranche.md`
- `2026-04/09-172019-g09-006-au-vst3-preparation-closeout.md`
- `2026-04/09-173500-g09-006-closeout-and-g09-007-strict-handoff.md`
- `2026-04/09-180254-g09-007-offline-preview-assembly-carveout.md`
- `2026-04/09-183500-g09-007-runtime-tests-front-door-normalization-closeout.md`
- `2026-04/09-190500-g09-007-closeout-and-g09-008-strict-handoff.md`
- `2026-04/09-201000-g09-008-graph-and-primitive-invariants-tranche.md`
- `2026-04/09-211500-g09-008-clap-sandbox-protocol-hardening-tranche.md`
- `2026-04/09-221500-g09-008-shared-memory-lifecycle-hardening-tranche.md`
- `2026-04/09-223500-g09-008-closeout-and-g09-009-strict-handoff.md`
- `2026-04/09-230500-g09-009-resampler-quality-tier-foundation-tranche.md`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`.
