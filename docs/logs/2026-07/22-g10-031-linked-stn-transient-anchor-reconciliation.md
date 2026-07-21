# g10.031 Linked STN Transient-Anchor Reconciliation

Date: 2026-07-22
Status: Batch 31.59 complete; Batch 31.60 ready
Scope: docs-only authority recovery

## Stop Evidence

Batch 31.58 started from `90b3610c` in isolated worktree
`signal-candidate-31-58`, branch
`candidate/g10-031-conformance-bound-linked-stn-noise-morph`. The retained
stop is commit `ae618c90827ddd748dc224632920ee32f785cc65`, tree
`de551fc6fa458d500239ac603ed26dee1a4458d6`.

Focused compile, construction `1/1`, independent full-buffer `S05`, and
bounded-allocation `S17` passed. `S09` then exposed contradictory frozen
authority. At isolated impulse source sample `p=48000`, reconstructed rising
and falling derivative powers were `0x3feffffffffffffe` and
`0x3ff0000000000000`, two non-negative `f64` encodings apart. Exact comparison
selected `p+1`; `Y03` requires authored `p`.

The candidate stopped before a formal clean pass, synthetic execution,
rendered audio, listening, or acoustic ref. Its nine-round conformance ledger,
source, branch, and worktree remain retained. No candidate code entered
`main`.

## Authority Decision

Transient refinement now computes unchanged non-negative finite `f64` scores
and compares their ordered bit patterns. Distances `0..4` are one equality
class owned by the earliest sample. Distance `5` or greater selects the larger
score. Non-finite scores reject processing. The boundary is fixed, structural,
and not a tunable acoustic threshold.

`S09` owns distances `0`, `4`, `5`, and the observed distance-two impulse pair.
`Y03` now states separately that refinement retains authored source sample `p`
and the event ledger stores its exact mapped target anchor.

Contract `085` Rule 11 permits the retained pre-acoustic worktree to resume
under the same candidate identity after this docs closeout is applied there.
All partial passes remain diagnostic. Full compile, construction `1/1`, and
structural `18/18` conformance must restart twice from one clean commit before
any acoustic ref.

## Scope

Changed the canonical linked-STN brief, Contract `085`, `g10.031`, roadmap
front doors, and this log. Candidate DSP, harnesses, production code, routing,
Loophole, and Chorus did not change. Three unrelated pre-existing plugin-host
edits in the main worktree were preserved outside this batch.

## Next Task

Apply this Batch 31.59 docs closeout commit to the retained candidate worktree.
Implement only the four-ULP transient-refinement comparison, target-ledger
semantics, and direct `S09`/`Y03` ownership. Commit the clean resumed tree, then
restart full conformance twice before creating an acoustic ref. Do not run
acoustic gates, alter production or `main`, merge, or push.
