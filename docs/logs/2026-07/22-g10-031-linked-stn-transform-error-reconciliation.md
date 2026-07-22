# g10.031 Linked STN Transform-Error Reconciliation

Date: 2026-07-22
Status: Batch 31.61 complete; Batch 31.62 ready
Scope: docs-only authority recovery

## Batch 31.60 Stop

Batch 31.59 docs applied cleanly to retained worktree
`signal-candidate-31-58`. The four-ULP comparison and mapped target ledger
compiled. `S09` then stopped on the frozen `0.65` impulse-train event at source
`p=58103`.

Reconstructed rising and falling derivative powers were approximately
`0.4224999690055853` and `0.42249996900558584`, bit patterns
`0x3fdb0a3d4f5c2900` and `0x3fdb0a3d4f5c290a`. Their distance is `10`, so the
four-ULP rule still selected `p+1` against `Y03`'s authored `p`.

The retained stop is commit `4cb82a2ef7731aeaf306d3955766c75c9863aa89`,
tree `6083e84604bb95f561fd6b7c25aef55b9a49b12a`. Conformance ledger round
`10A` records the exact correction and failure. `S10`, a complete structural
round, synthetic execution, rendered audio, listening, and acoustic-ref
creation did not run. Candidate code did not enter `main`.

## Authority Decision

Representation-local ULP counting is removed. It does not describe one stable
relative error across score exponents and therefore cannot own the complete
reconstructed-transform path.

For non-negative finite current score `a` and later challenger `b`, define
`tau=64*f64::EPSILON*max(a,b)`. The challenger wins only when `b>a` and
`b-a>tau`; otherwise earliest owns the numerical tie. `64` is the next power of
two above four rounding sites across the maximum `12` short-transform stages.
It binds analysis, inverse, normalized WOLA, derivative, square, and channel
sum as one path. There is no absolute floor and scores are unchanged.

`S09` owns zero, exact equality, `1.0` versus challengers `64` and `65` ULPs
higher, and both observed impulse pairs. `S10` owns exact mapped target-ledger
anchors. Compiled `Y03` owns authored source anchors and mapped target anchors.

Contract `085` Rule 11 permits one retained pre-acoustic resume after this docs
closeout is applied. Earlier partial passes remain diagnostic. Full compile,
construction `1/1`, and structural `18/18` conformance must restart twice from
one clean commit before any acoustic ref.

## Scope

Changed the canonical linked-STN brief, Contract `085`, `g10.031`, active
front doors, and this log. Candidate DSP, acoustic execution, production,
routing, Loophole, and Chorus did not change on `main`. Three unrelated
pre-existing plugin-host edits remain preserved outside this batch.

## Next Task

Apply this Batch 31.61 docs closeout to the retained candidate worktree.
Replace only the ULP comparison and direct `S09`, `S10`, and compiled `Y03`
ownership. Commit one clean resumed tree, then restart complete compile,
construction `1/1`, and structural `18/18` conformance twice before creating
an acoustic ref. Do not run acoustic gates, alter production or `main`, merge,
or push.
