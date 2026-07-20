# g10.031 Seed Authority Reassessment

Date: 2026-07-20
Batch: 31.30
Status: complete; fresh candidate authority frozen

## Evidence

Batch 31.25 passed the normative mono renderer, synthetic sources, metrics,
and concealed mono listening. Batch 31.29 specified the same mono system but
failed one `16x` replica row and two `4x` pitch rows. Neither brief froze the
candidate seed. Batch 31.29's helpers chose seed `17`; the earlier passing
receipt did not record its seed.

Pinned PaulXStretch keeps one transform geometry, fractional source
accumulator, magnitude-renewal path, and adjacent-frame blend across the
retained ratios. Current source and Signal evidence do not support a `4x`/`16x`
algorithm or resolution switch.

## Decision

The Batch 31.29 checkpoint remains rejected. Its fixed-resolution range
diagnosis is withdrawn because stochastic request identity was not controlled.
No candidate was repaired, rerun, or recovered.

Fresh authority is `SeedAuditedSourceRelativeRenewalSpectral`. It retains the
complete source-relative renderer and freezes the audited address vector's seed
as `ADMISSION_SEED` for every synthetic and listening candidate render. This is
one evidence-authority correction, not a seed sweep. Public seed/reroll
exposure remains closed pending a later multi-seed character review.

No DSP, tests, harnesses, fixtures, APIs, routes, cache, Loophole, or Chorus
surface changed.

## Next Task

Run Batch 31.31 only. Implement the seed-audited brief once in
`signal-candidate-31-31`, complete construction `1/1`, freeze one checkpoint,
then run structural `15/15` and synthetic `9/9` once in order. Stop on the
first miss. Do not alter `ADMISSION_SEED` or push.
