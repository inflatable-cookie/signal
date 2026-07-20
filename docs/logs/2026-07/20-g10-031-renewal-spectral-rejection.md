# g10.031 RenewalSpectral Candidate Rejection

Date: 2026-07-20
Status: Batch 31.18 complete; candidate rejected
Roadmap: `g10.031`
Contract: `085`

## Result

Implemented the frozen `RenewalSpectral` brief once in disposable worktree
`/Users/tom/Dev/projects/signal-candidate-31-18` on branch
`candidate/g10-031-renewal-spectral`, based on docs commit `2b188741`.

The private candidate contained only
`creative_renewal/{mod,plan,analysis,phase,synthesis,tests}.rs` and one private
`lib.rs` declaration. It implemented the exact sample-centred map, long
magnitude analysis, counter-addressed frame/bin phase renewal, linked mid/side
law, equal-power frame crossfade, exterior envelope, exact crop, and bounded
duration-independent state.

Compile-only validation passed. The complete structural gate then passed:

- exact request rejection, target length, finiteness, silence, and boundaries
- deterministic output, seed variation, map, interpolation, and phase address
- linked duplicate, swap, polarity, anti-phase, delay, mixed, and `space` laws
- state below `32 MiB`, duration-independent capacity, and zero processing
  allocations

## Stop

The mandated first crest row failed. Neutral `Dream`, `space=0`, `4x`, and the
frozen deterministic uniform-noise source measured `8.263162 dB` of
crest-factor growth against the frozen `6 dB` ceiling.

The dominant cause is uncontrolled cross-bin waveform summation after complete
independent phase renewal. The structural synthesis law is deterministic and
bounded in state, but it does not bound stochastic waveform crest. This is the
same broad crest-ownership problem exposed by the earlier independent-bin
diffusive candidate, now reproduced without its carrier, magnitude recurrence,
or rolling overlap law.

The candidate was not corrected or rerun. Remaining crest rows, comparator-
calibrated synthetic tests, concealed mono listening, and independent stereo
listening did not run.

## Cleanup

Deleted the disposable worktree, branch, private module, tests, and candidate
listening state. Candidate build state was removed from its active paths and
moved to Trash, where it remains recoverable. No candidate DSP, harness,
fixture, report mode, public API, cache, route, Loophole, or Chorus surface
entered `main`. The three unrelated binaural/reverb edits remain untouched.

## Validation

- candidate compile-only gate: passed
- candidate structural gate: passed, one complete named filter
- candidate first crest row: failed at `8.263162 dB` versus `6 dB`
- later synthetic and listening gates: not opened
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- disposable worktree and candidate branch: absent

`effigy doctor` retains the known god-file and attention-marker findings. This
batch does not expand into them.

## Next Task

Run Batch 31.19 only. Reassess neutral-`Dream` crest ownership at architecture
level or close the owner. Do not tune or reimplement `RenewalSpectral`.
