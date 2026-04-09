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
- current ready card: none

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
- `2026-04/09-233500-g09-009-resampler-proof-and-semantic-handoff-tranche.md`
- `2026-04/09-240500-g09-009-semantic-calibration-baseline-tranche.md`
- `2026-04/09-241500-g09-009-confidence-closeout-and-g09-010-handoff.md`
- `2026-04/09-250500-g09-010-worker-containment-closeout-and-policy-ready.md`
- `2026-04/09-254500-g09-010-tempo-state-unification-closeout-and-meter-ready.md`
- `2026-04/09-261500-g09-010-meter-plan-shell-closeout-and-trigger-ready.md`
- `2026-04/09-271000-g09-010-trigger-cause-normalization-closeout-and-context-ready.md`
- `2026-04/09-280000-g09-010-stage-plan-context-closeout-and-planning-pause.md`
- `2026-04/09-283500-g09-010-regression-corpus-ready-handoff.md`
- `2026-04/09-290500-g09-010-regression-closeout-and-g09-011-handoff.md`
- `2026-04/09-300500-g09-011-demo-program-shape-closeout-and-launch-ready.md`
- `2026-04/09-303500-g09-011-demo-launch-evidence-closeout-and-matrix-ready.md`
- `2026-04/09-311500-g09-011-coverage-matrix-closeout-and-g09-012-handoff.md`
- `2026-04/09-320500-g09-012-sandbox-lifecycle-demo-bootstrap-closeout.md`
- `2026-04/09-323500-g09-012-runtime-inspector-ready-handoff.md`
- `2026-04/09-330500-g09-012-runtime-inspector-closeout-and-planning-pause.md`
- `2026-04/09-333500-g09-012-host-bootstrap-fix-ready-handoff.md`
- `2026-04/09-341500-g09-012-host-bootstrap-fix-closeout-and-planning-pause.md`
- `2026-04/09-344500-g09-012-host-comparison-bootstrap-ready-handoff.md`
- `2026-04/09-350500-g09-012-clap-host-fix-ready-correction.md`
- `2026-04/09-360500-g09-012-clap-host-fix-closeout-and-host-comparison-reactivation.md`
- `2026-04/09-370500-g09-012-host-comparison-bootstrap-closeout-and-planning-pause.md`
- `2026-04/09-374500-g09-012-hardware-diagnostics-ready-handoff.md`
- `2026-04/09-380500-g09-012-hardware-diagnostics-bootstrap-closeout.md`

## Next Task

Re-enter planning for the active strict `g09` lane before promoting another
`g09.012` batch card.
