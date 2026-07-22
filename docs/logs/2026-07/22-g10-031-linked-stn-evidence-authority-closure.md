# g10.031 Linked STN Evidence-Authority Closure

Date: 2026-07-22
Status: Batch 31.63 complete; roadmap paused
Scope: docs-only retained-ref reassessment

## Evidence

Inspected local ref
`refs/signal-evidence/creative/linked-stn/31-58-acoustic` at commit
`61922465b446dfce8ed086bc5dd61f4a9619a837`, tree
`fc57cd4c5eeb3c889293de3e8236863ca5513e7c`. Candidate DSP did not execute.

The frozen Effigy plan resolves to unoptimized
`cargo nextest run --workspace 'conformance_bound_linked_stn_synthetic_'`.
No repo or user nextest config froze timeout, concurrency, ordering, or failure
capture.

Executable `Y09` omitted four canonical hard paths:

- swapped-input render and swap-back commutation
- duplicate stereo against separately rendered mono
- descriptor-diagonal preservation
- non-decreasing residual side energy across `space`

Construction checked `28` unique owner IDs and non-null pointers. It did not
bind assertions or receipt fields to owners.

`Y09` contained `5` controls by `3` ratios by `3` spaces and rendered every row
twice: `45` rows, `90` full stereo renders, and `80,640,000` output frames,
plus exact-length source and output band FFTs. It persisted no row before owner
completion. The Batch 31.62 command returned no assertion, numeric row, output
digest, or generated receipt.

## Decision

The checkpoint has invalid executable evidence. It has neither a synthetic
pass nor an acoustic rejection. Its debug-profile non-completion does not prove
release-profile computational infeasibility.

This repeats Batch 31.53's incomplete-executable-authority class after Contract
`085` Rule 11 was introduced to prevent it. Another evidence-only identity
would be protocol churn around the same large renderer. Linked STN closes
without promotion. The PaulX-like neutral `Dream` target remains active and
unadmitted.

Rule 11 now requires construction to bind every canonical acoustic assertion
and receipt field to a compile-linked owner manifest. Future authority must
also freeze row/render counts, build profile, runner execution envelope,
failure capture, and incremental receipt persistence before checkpoint.

The local evidence ref was deleted after this decision. Candidate worktree,
branch, source, and build state remained absent. Production, routing, cache,
product exposure, Loophole, and Chorus remained unchanged.

## Next Task

Operator checkpoint. Decide whether to commission one materially simpler,
source-backed complete creative owner study or pause `g10.031` indefinitely.
No prior candidate or implementation batch is ready.
