# g10.031 Seed-Audited Renewal Rejection

Date: 2026-07-20
Batch: 31.31
Status: complete; candidate rejected and deleted

## Evidence

- worktree: `signal-candidate-31-31`
- branch: `candidate/g10-031-seed-audited-source-relative-renewal`
- immutable checkpoint: `790119b7936d5166ffb814f9401ba1398d2d5db9`
- compile: pass
- construction: exactly `1/1`
- structural: exactly `15/15`
- synthetic: nine selected; six passed, `Y02` failed, `Y08` and `Y09`
  cancelled
- listening: not run

`Y02` completed its pitch matrix before asserting. The `8x` chord row measured
`13.351828347` cents maximum partial error against an
`11.331375778`-cent PaulX-relative ceiling.

`Y04` passed impulse and impulse-train rows at `4x`, `8x`, and `16x`. The
audited seed therefore removed Batch 31.29's replica miss but did not produce
robust tonal pitch.

Batch 31.29 failed two `4x` pitch rows under seed `17`; Batch 31.31 failed the
`8x` chord under `ADMISSION_SEED`. This is the same tonal-coherence failure
class across different seeds, material, and ratios. Contract `084` Rule 7
requires architecture reassessment, not another seed or parameter attempt.

Cleanup deleted the worktree, branch, checkpoint, module, tests, build state,
and candidate artifacts. No DSP, harness, fixture, API, route, cache,
Loophole, or Chorus surface entered `main`.

## Next Task

Run Batch 31.32 only. Reassess the renewal family at architecture level against
the repeated tonal-pitch failure and pinned source. Either identify one
materially different, source-backed complete renderer with intrinsic tonal
coherence or close the family. Do not implement DSP, sweep parameters, or
push.
