# g10.031 Complex-Relation Candidate Rejection

Date: 2026-07-19
Status: Batch 31.8 complete; current diffusive owner closed

## Result

Implemented `ContinuousExcitationComplexRelation` once in the disposable
`signal-candidate-31-8` worktree. The candidate added only its private six-file
module and one private declaration. Compile-only validation passed.

The first admitted coefficient proof failed exact anti-phase enumeration:
actual `-1+0i`, expected `+1-0i`. The proof required both componentwise
negation and negated channel-swap. For exact anti-phase stereo, common polarity
is itself the channel swap, so componentwise negation equals plain swap and
cannot also equal negated swap.

The frozen rule treats any relation-proof miss as terminal. The proof was not
corrected or rerun. The prior common-polarity renderer row, remaining
structural controls, crest gate, synthetic gates, and listening stayed closed.

## Cleanup

Deleted the candidate worktree, branch, module, tests, and build state. No DSP,
harness, fixture, report mode, public API, dependency, route, or generated audio
entered `main`. The three unrelated binaural/reverb edits remained untouched.

## Decision

`ContinuousExcitationComplexRelation` and the current diffusive owner are
closed. Contract `085` now requires creative range-owner reassessment. It does
not authorize proof repair, another diffusive variant, or early work on Cloud,
Cyclic, routing, cache, or product integration.

## Validation

- candidate compile-only Effigy suite: passed
- coefficient relation proof: failed; one test run, zero passed
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed

## Next Task

Run `g10.031` Batch 31.9 only. Reassess ownership of the creative `4x` through
`16x` range at architecture level and freeze one honest direction or close the
range promise. Do not implement a candidate in the same batch.
