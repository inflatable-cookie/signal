# g10.031 Compensated Renewal Compile Rejection

Date: 2026-07-20
Status: Batch 31.21 complete; candidate rejected at compile-only validation
Roadmap: `g10.031`
Contract: `085`

## Result

Implemented the frozen `CompensatedRenewalSpectral` brief once in disposable
worktree `/Users/tom/Dev/projects/signal-candidate-31-21` on branch
`candidate/g10-031-compensated-renewal`, based on docs commit `f0c34964`.

The private candidate contained only
`creative_compensated_renewal/{mod,plan,analysis,phase,synthesis,tests}.rs` and
one private `lib.rs` declaration. No public API, report mode, fixture, cache,
route, dependency, Loophole, or Chorus surface changed.

## Stop

Compile-only validation failed in the structural test suite. Rust could not
infer the type of one `Option` accumulator used to compare side-component
spectral power across `space` values.

The stopped gate was compile-only validation. No renderer executed. Structural
admission, the full reference-relative synthetic matrix, concealed mono
listening, and independent stereo listening did not run.

The dominant cause is an incomplete test type declaration. This result says
nothing about the compensated blend, crest behavior, pitch, smear, event
placement, or listening quality. The topology remains untested.

## Cleanup

The candidate was not corrected or rerun. Deleted the disposable worktree,
branch, private module, tests, and candidate build state. The ignored PaulX
comparator evidence under
`target/creative-stretch-paulx-reference-31-20/` remains intact. No candidate
DSP or harness surface entered `main`.

## Validation

- candidate compile-only validation: failed before renderer execution
- structural, synthetic, mono, and stereo gates: not opened
- disposable worktree and candidate branch: absent
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- known `effigy doctor` findings: unchanged at 57 god-file and 5 attention
  markers

## Next Task

Run Batch 31.22 only. Freeze fresh complete candidate authority for the
still-untested compensated-renewal topology. Do not implement DSP in the same
batch.
