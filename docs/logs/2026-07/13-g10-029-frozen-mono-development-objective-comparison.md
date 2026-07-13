# Frozen Mono Development Objective Comparison

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BY`
Status: candidate rejected before listening

## Result

The event-owned successor passes every structural gate on the nine frozen mono
development rows, but loses broadly to current Signal on real-source quality.
No concealed pack was exported.

| Candidate regression against current | Rows |
| --- | ---: |
| event placement | `6/9` |
| post-event replica ratio | `7/9` |
| static spectral residual | `9/9` |
| formant-envelope residual | `9/9` |

Tonal movement improves in `7/9` rows. That is useful evidence for active
physical-frequency transport, but it does not offset the event and spectral
regressions in the complete synthesis.

## Frozen Report

The local report is
`target/stretch-successor-by-development-objective.tsv`. It contains current
Signal, the event-owned successor, and the already captured Rubber Band R3
render for every row. Fields cover exact length, finiteness, full-render
integrity, event placement, crest, replica ratio, tonal movement, static
spectral residual, unsupported mass, envelope texture, formant residual and
centroid shift, and boundary growth and level.

- rows: `9`
- modes: `3`
- renders: `27`
- hard failures: `0`
- candidate hard failures: `0`
- candidate outputs distinct from current: `9/9`
- holdout reads: `0`
- listening exports: `0`
- report SHA-256:
  `9cdaedf39d80c1cefcbc34d2d78f42d30c8c1c7835467fb913b32ffca511e14f`
- manifest hash: `2abde0a10417b469`
- render hash: `4359fd9e43ff6a9c`
- measurement hash: `18823a809bb4b2cc`
- aggregate hash: `10d25f8404262480`

## Measurement Boundary

The established production/candidate transient matcher finds one event in four
source excerpts and none in five. Event-only fields for those five rows use one
declared strongest-onset fallback, applied identically to all three modes.
That produces `15` fallback renders. The rejection does not depend on fallback
timing: static spectral and formant residual regress independently in all nine
rows.

Captured external audio is behavioural evidence only. No comparator command ran
and no comparator implementation was inspected.

## Decision

Do not spend another operator round on this candidate. Batch 29.6BZ will keep
the same rows and policies frozen while comparing ordinary adaptive synthesis,
tracked transport without anchors, tracked transport with anchors, and the
event-owned result. That assigns the regression to frame geometry, active-peak
transport, transient reset, or overlap ownership before another design.

## Next Task

Execute Batch 29.6BZ under Rule 30U. Keep holdout, listening export, tuning,
linked stereo, dynamic ratio, cache, and product routing closed.
