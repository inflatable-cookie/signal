# g10.031 Linked STN Geometry Reconciliation

Date: 2026-07-21
Batch: 31.48
Status: complete; geometry-audited v4 candidate ready

## Scope

Reconcile the frozen `R_v<=57` bound against the `R_v=59` counterexample.
Audit every median extent and dependent memory row. Change documentation only.

## Exact Result

Two independent exact-integer evaluators covered every
`F=8000..192000`. Both produced:

- `Q_h<=17`; first maximum at `F=16534`
- `Q_v<=97`; first maximum at `F=8000`
- `R_h<=19`; first maximum at `F=17500`
- `R_v<=59`; first maximum at `F=8000`

At `F=8000`, `N_t=2048`, `N_s=256`, and `A_s=64`.
`round(1800*N_s/F)=58`; the frozen nearest-odd midpoint rule chooses `59`.
The old `57` bound was wrong. The formula and tie rule remain unchanged.
Positive rational `round` is now explicit: nearest integer, exact half chooses
the larger integer. This matches the audited result and removes a candidate
implementation choice.

## Memory Result

`Q_v=97` already dominates shared median-selection scratch, now frozen as
`max(Q_h,Q_v,R_h,R_v)=97` `f64` scalars. Correcting `R_v` changes no ring or
packed-memory model.

Reconfirmed maxima include first residual `53248`, each component ring
`147712`, claim arena `98816`, live events `39`, envelope `32772`, output
finalization `139520`, peak tracks `16383`, and bin states `16385`.
Short/source state remains `9.700 MiB`; category ceilings remain `89 MiB`;
`7 MiB` remains unassigned below the `96 MiB` actual gate.

## Fresh Authority

The canonical brief now freezes:

- candidate: `GeometryAuditedBoundedLinkedStnNoiseMorph`
- worktree: `signal-candidate-31-49`
- branch:
  `candidate/g10-031-geometry-audited-bounded-linked-stn-noise-morph`
- private module:
  `creative_geometry_audited_bounded_linked_stn_noise_morph`
- corrected `R_v` maximum `59`
- unchanged `28` structural and synthetic owners

This identity does not recover Batch 31.47 source or evidence. One fresh
checkpoint must pass construction before objective admission.

## Repository Result

- canonical brief, contract, roadmap, front doors, and closeout log updated
- no DSP, test, harness, fixture, dependency, API, route, cache, artifact,
  product, Loophole, or Chorus change
- unrelated plugin worktree changes preserved and unstaged

## Next Task

Run Batch 31.49 only in the fresh worktree and branch above. Implement
geometry-audited v4 once, complete compile and construction, freeze one
checkpoint, then run structural and synthetic admission in order. Stop before
listening on any miss. Do not recover Batch 31.47, merge, or push.
