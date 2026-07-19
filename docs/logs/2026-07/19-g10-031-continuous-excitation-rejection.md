# g10.031 Continuous-Excitation Candidate Rejection

Date: 2026-07-19
Status: Batch 31.6 complete; candidate rejected
Roadmap: `g10.031`
Contract: `085`

## Result

Implemented the frozen `ContinuousExcitationSpectral` brief once in the
disposable `signal-candidate-31-6` worktree on
`candidate/g10-031-continuous-excitation`.

The candidate contained only the private six-file `creative_excitation` family
and one private `lib.rs` declaration. Twelve of thirteen structural controls
passed. Passed rows included:

- exact length, finite output, normalization coverage, and exact silence
- deterministic repetition, seed variation, and duration-independent state
- no allocation after frame processing began
- continuous full-complex excitation and `1e-6` flat-envelope reconstruction
- duplicate stereo, exact channel swap, exact anti-phase fallback, and `space`

General common-polarity covariance differed by `0.0013287` against the frozen
`1e-6` bound. The shared source-orientation bit flipped correctly and channel
swap was exact. The remaining failure was the polar per-bin native-relation
reconstruction.

Structural admission stopped there. The prior neutral `Dream`, `4x` crest row,
remaining synthetic gates, long-form audio, and stereo listening did not open.

## Decision

Reject and delete the candidate. Do not repair the relation locally or open a
third implementation. The worktree, branch, private module, tests, and build
state were removed. No candidate code entered `main`.

The continuous output-synchronous excitation remains architecture evidence,
not an admitted owner. Batch 31.7 must decide whether one direct
value-symmetric complex relation can own all linked transformations as a
complete law, or close the current diffusive owner.

## Validation

- candidate structural filter: `12/13` passed; common polarity failed
- candidate crest and listening filters: not run
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- disposable worktree and candidate branch: absent

## Next Task

Run Batch 31.7 only. Reassess linked-relation ownership in documentation. Do
not implement another candidate, rerun the crest row, produce audio, or open
later owners or product routing.
