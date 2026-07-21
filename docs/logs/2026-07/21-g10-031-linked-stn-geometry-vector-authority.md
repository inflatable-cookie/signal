# g10.031 Linked STN Geometry-Vector Authority

Date: 2026-07-21
Batch: 31.52
Status: complete; construction-bound v6 candidate ready

## Scope

Audit every linked-STN geometry vector, transition, tie, witness, and
geometry-derived construction assertion. Bind one exact table into
construction and structural authority. Change documentation only.

## Audit Receipt

- base: `13dd157df36a6d0a81634254a19a7a123a7df95a`
- evaluators: separately implemented Ruby and JavaScript integer programs
- sample-rate domain: every integer `8000..192000`
- rows: `184001`
- binary row order:
  `F,N_t,A_t,N_s,A_s,N_r,A_r,H,Q_h,Q_v,R_h,R_v`
- encoding: twelve little-endian `u32` values per row, ascending `F`
- bytes: `8832048`
- SHA-256:
  `22d14913f01143007a114fad7a97d44a7e2b07cf5b254b92bc59c7f805e73697`
- FNV-1a-64: `7ffb5aa02900893e`
- maxima: `Q_h=17`, `Q_v=97`, `R_h=19`, `R_v=59`
- first witnesses: `16534`, `8000`, `17500`, `8000`
- 8 kHz row:
  `(2048,256,256,64,1024,256,128,9,97,9,59)`

Both evaluators also agree on every transform transition, positive-round tie
set, upward odd count, and geometry-derived capacity maximum and first/last
witness. No further contradiction was found.

## Decision

Freeze fresh `ConstructionBoundZeroPreservingLinkedStnNoiseMorph` v6.
`GEOMETRY_SPEC` is its sole literal geometry table. Construction compares the
renderer against a separately coded oracle across the full domain, then checks
sentinels, transitions, tie sets, witnesses, and fingerprint. Construction and
`S02` call one shared authority assertion; `S02` cannot add a second geometry
table.

All renderer, exact-zero, stereo, memory, quality, gate-order, receipt, and
cleanup rules remain unchanged. No candidate DSP, test, harness, dependency,
API, route, cache, artifact, product, Loophole, or Chorus surface entered
`main`. Pre-existing plugin edits remain unstaged and untouched.

## Next Task

Run Batch 31.53 only in worktree `signal-candidate-31-53` on branch
`candidate/g10-031-construction-bound-zero-preserving-linked-stn-noise-morph`.
Implement once, require compile and construction `1/1`, checkpoint, then run
structural and synthetic admission in order. Stop, delete, and close on the
first miss. Do not repair Batch 31.51, merge, or push.
